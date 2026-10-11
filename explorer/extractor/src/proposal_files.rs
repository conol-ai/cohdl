use crate::document_registry::read_bounded;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub const JSON_LIMIT: u64 = 8 * 1024 * 1024;
pub const LOG_LIMIT: u64 = 2 * 1024 * 1024;
pub const INPUT_LIMIT: usize = 2048;
pub const RUN_LIMIT: usize = 128;
pub const LOG_NAMES: [&str; 6] = [
    "provider.stdout.log",
    "provider.stderr.log",
    "compiler.stdout.log",
    "compiler.stderr.log",
    "apply-compiler.stdout.log",
    "apply-compiler.stderr.log",
];
pub const RECORD_NAMES: [&str; 5] = [
    "run.json",
    "bundle.json",
    "proposal.json",
    "validation.json",
    "receipt.json",
];

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub path: String,
    pub sha256: Option<String>,
}

pub fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

pub fn relative(value: &str) -> bool {
    value.is_ascii()
        && value.len() <= 1024
        && crate::document_registry::safe_relative(value)
        && value
            .split('/')
            .all(|part| !part.is_empty() && !part.contains([':', '%']) && !secret_component(part))
}

fn secret_component(part: &str) -> bool {
    let lower = part.to_ascii_lowercase();
    [
        "env",
        "secret",
        "secrets",
        "credential",
        "credentials",
        "id_rsa",
        "id_ed25519",
    ]
    .into_iter()
    .any(|prefix| {
        lower
            .strip_prefix(prefix)
            .is_some_and(|suffix| suffix.is_empty() || suffix.starts_with(['.', '_', '-']))
    })
}

pub fn confined(project: &Path, value: &str) -> Result<PathBuf, String> {
    if !relative(value) {
        return Err("Unsupported or private path".into());
    }
    let root = project
        .canonicalize()
        .map_err(|_| "Project is unavailable")?;
    let mut path = root.clone();
    for part in value.split('/') {
        path.push(part);
        match std::fs::symlink_metadata(&path) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err("Symlinks are not proposal inputs".into())
            }
            Ok(_) => (),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => return Err("Path is not readable".into()),
        }
    }
    if path.exists() && !path.canonicalize().is_ok_and(|p| p.starts_with(&root)) {
        return Err("Path escapes project".into());
    }
    Ok(path)
}

pub fn bytes(project: &Path, value: &str, limit: u64) -> Result<Option<Vec<u8>>, String> {
    let file = confined(project, value)?;
    match std::fs::symlink_metadata(&file) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("File is not readable".into()),
        Ok(_) => read_bounded(&file, limit).map(Some),
    }
}

pub fn run_ids(project: &Path) -> Result<Vec<String>, String> {
    let directory = confined(project, "explorer/proposals")?;
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let entries = std::fs::read_dir(directory).map_err(|_| "Proposals are not readable")?;
    let mut ids = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| "Proposal entry is not readable")?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !identifier(&name) {
            continue;
        }
        if !entry
            .file_type()
            .is_ok_and(|t| t.is_dir() && !t.is_symlink())
        {
            continue;
        }
        if ids.len() == RUN_LIMIT {
            return Err("Proposal count exceeds 128 runs".into());
        }
        ids.push(name);
    }
    ids.sort();
    Ok(ids)
}

pub fn manifest(value: &serde_json::Value) -> Result<Vec<Entry>, String> {
    let values = value.as_array().ok_or("Snapshot must be an array")?;
    if values.len() > INPUT_LIMIT {
        return Err("Snapshot exceeds 2048 inputs".into());
    }
    let mut previous: Option<&str> = None;
    for value in values {
        if !value
            .as_object()
            .is_some_and(|o| o.len() == 2 && o.contains_key("sha256"))
        {
            return Err("Snapshot fields are invalid".into());
        }
        let path = value["path"]
            .as_str()
            .filter(|p| relative(p))
            .ok_or("Invalid snapshot path")?;
        if previous.is_some_and(|p| p >= path) {
            return Err("Snapshot must be sorted and unique".into());
        }
        if !value["sha256"].is_null() && !hash(&value["sha256"]) {
            return Err("Invalid snapshot hash".into());
        }
        previous = Some(path);
    }
    serde_json::from_value(value.clone()).map_err(|_| "Invalid snapshot".into())
}

pub fn hash(value: &serde_json::Value) -> bool {
    value.as_str().is_some_and(|s| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

fn add(paths: &mut BTreeSet<String>, path: String) -> Result<(), String> {
    if !relative(&path) {
        return Err("Unsupported snapshot input".into());
    }
    paths.insert(path);
    if paths.len() > INPUT_LIMIT {
        return Err("Snapshot exceeds 2048 inputs".into());
    }
    Ok(())
}

fn source_file(path: &str) -> bool {
    path.ends_with(".cohdl")
        || path.ends_with(".dxf")
        || ["cohdl.toml", "cohdl.lock", "design.lock"]
            .contains(&path.rsplit('/').next().unwrap_or(""))
}

fn sources(project: &Path, directory: &str, paths: &mut BTreeSet<String>) -> Result<(), String> {
    let mut pending = vec![directory.to_owned()];
    let mut directories = 0usize;
    while let Some(relative) = pending.pop() {
        directories += 1;
        if directories > INPUT_LIMIT {
            return Err("Too many source directories".into());
        }
        let dir = confined(project, &relative)?;
        if !dir.exists() {
            continue;
        }
        for entry in std::fs::read_dir(dir).map_err(|_| "Source directory is unreadable")? {
            let entry = entry.map_err(|_| "Source entry is unreadable")?;
            let name = entry
                .file_name()
                .to_str()
                .ok_or("Non-ASCII input path")?
                .to_owned();
            let path = format!("{relative}/{name}");
            confined(project, &path)?;
            let kind = entry
                .file_type()
                .map_err(|_| "Source entry is unreadable")?;
            if kind.is_dir() {
                if pending.len() + paths.len() >= INPUT_LIMIT {
                    return Err("Too many source directories".into());
                }
                pending.push(path);
            } else if source_file(&path) {
                add(paths, path)?;
            }
        }
    }
    Ok(())
}

fn input_paths(
    project: &Path,
    design: &str,
) -> Result<(BTreeSet<String>, BTreeSet<String>), String> {
    if design.is_empty()
        || !design.as_bytes()[0].is_ascii_alphabetic() && !design.starts_with('_')
        || !design
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return Err("Invalid design identifier".into());
    }
    let mut paths = BTreeSet::new();
    for path in [
        "cohdl.toml",
        "cohdl.lock",
        "design.lock",
        "explorer/documents.json",
    ] {
        add(&mut paths, path.into())?;
    }
    for directory in ["src", "deps"] {
        sources(project, directory, &mut paths)?;
    }
    add(&mut paths, format!("views/{design}.view.json"))?;
    let views = confined(project, "views")?;
    if views.exists() {
        for entry in std::fs::read_dir(views).map_err(|_| "Views are unreadable")? {
            let name = entry
                .map_err(|_| "View is unreadable")?
                .file_name()
                .to_string_lossy()
                .into_owned();
            if name.ends_with(".view.json") {
                add(&mut paths, format!("views/{name}"))?;
            }
        }
    }
    let mut documents = BTreeSet::new();
    if let Some(bytes) = bytes(project, "explorer/documents.json", JSON_LIMIT)? {
        let registry: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| "Malformed document registry")?;
        if registry["schema"] != 1 {
            return Err("Unsupported document registry schema".into());
        }
        let records = registry["documents"]
            .as_array()
            .filter(|a| a.len() <= 512)
            .ok_or("Invalid document count")?;
        for record in records {
            if let Some(path) = record["path"].as_str() {
                add(&mut paths, path.into())?;
                documents.insert(path.into());
            } else if !record["path"].is_null() {
                return Err("Invalid document path".into());
            }
        }
    }
    Ok((paths, documents))
}

pub fn snapshot(project: &Path, design: &str) -> Result<Vec<Entry>, String> {
    let (paths, documents) = input_paths(project, design)?;
    let mut total = 0u64;
    let mut text_total = 0u64;
    let mut result = Vec::new();
    for path in paths {
        let document = documents.contains(&path);
        let limit = if document {
            64 * 1024 * 1024
        } else {
            JSON_LIMIT
        };
        let file = confined(project, &path)?;
        let size = match std::fs::metadata(&file) {
            Ok(meta) if meta.is_file() => meta.len(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
            _ => return Err("Snapshot input is not a readable file".into()),
        };
        total = total.checked_add(size).ok_or("Snapshot size overflow")?;
        if !document {
            text_total = text_total.checked_add(size).ok_or("Text size overflow")?;
        }
        if total > 256 * 1024 * 1024 || text_total > JSON_LIMIT {
            return Err("Snapshot exceeds total size limit".into());
        }
        let content = bytes(project, &path, limit)?;
        if content.as_ref().map_or(0, |b| b.len() as u64) != size {
            return Err("Snapshot input changed while reading".into());
        }
        result.push(Entry {
            path,
            sha256: content.map(|b| cohdl::hash::sha256_hex(&b)),
        });
    }
    Ok(result)
}

pub fn watch_snapshot(project: &Path) -> BTreeMap<String, (SystemTime, u64)> {
    let mut result = BTreeMap::new();
    let Ok(ids) = run_ids(project) else {
        return result;
    };
    for id in ids {
        for name in RECORD_NAMES
            .into_iter()
            .chain(LOG_NAMES)
            .chain(["request.txt"])
        {
            let relative = format!("explorer/proposals/{id}/{name}");
            let metadata = confined(project, &relative)
                .ok()
                .and_then(|p| p.metadata().ok());
            if let Some(metadata) = metadata.filter(|m| m.is_file()) {
                result.insert(
                    relative,
                    (
                        metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                        metadata.len(),
                    ),
                );
            }
        }
    }
    result
}
