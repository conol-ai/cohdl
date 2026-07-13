//! Project loading (provisional-syntax.md §1): `cohdl.toml` manifest, all
//! `.cohdl` files under `src/`, plus the std library.

use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Project {
    /// Output file base name.
    pub name: String,
    /// Project root — `design.lock` and the out dir live here.
    pub dir: PathBuf,
    /// `[design] top` from the manifest, if any.
    pub top: Option<String>,
    /// (display name, content), std first, deterministic order.
    pub files: Vec<(String, String)>,
}

/// Load a project from a directory (with `cohdl.toml`) or a single `.cohdl`
/// file. `std_dir` of `None` means "compile without the std library".
pub fn load_project(path: &Path, std_dir: Option<&Path>) -> Result<Project, String> {
    let mut files = Vec::new();
    if let Some(std_dir) = std_dir {
        collect_cohdl_files(std_dir, std_dir, "std", &mut files)?;
        if files.is_empty() {
            return Err(format!(
                "std library directory `{}` contains no .cohdl files",
                std_dir.display()
            ));
        }
    }

    if path.is_file() {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("cannot read `{}`: {}", path.display(), e))?;
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "design".to_string());
        files.push((path.display().to_string(), content));
        return Ok(Project {
            name,
            dir: path
                .parent()
                .map(Path::to_path_buf)
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or_else(|| PathBuf::from(".")),
            top: None,
            files,
        });
    }

    if !path.is_dir() {
        return Err(format!("`{}` is not a file or directory", path.display()));
    }

    let manifest_path = path.join("cohdl.toml");
    let manifest_text = fs::read_to_string(&manifest_path).map_err(|_| {
        format!(
            "no `cohdl.toml` found in `{}` (a project directory needs a manifest; or pass a single .cohdl file)",
            path.display()
        )
    })?;
    let manifest = parse_manifest(&manifest_text)
        .map_err(|e| format!("{}: {}", manifest_path.display(), e))?;

    let src_dir = path.join("src");
    if !src_dir.is_dir() {
        return Err(format!("`{}` has no `src/` directory", path.display()));
    }
    let before = files.len();
    collect_cohdl_files(&src_dir, path, "", &mut files)?;
    if files.len() == before {
        return Err(format!("`{}` contains no .cohdl files", src_dir.display()));
    }

    Ok(Project {
        name: manifest.name,
        dir: path.to_path_buf(),
        top: manifest.top,
        files,
    })
}

/// Recursively collect `.cohdl` files in sorted order. Display names are
/// `prefix/relative/path` (prefix distinguishes std files in diagnostics).
fn collect_cohdl_files(
    dir: &Path,
    base: &Path,
    prefix: &str,
    out: &mut Vec<(String, String)>,
) -> Result<(), String> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("cannot read `{}`: {}", dir.display(), e))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for entry in entries {
        if entry.is_dir() {
            collect_cohdl_files(&entry, base, prefix, out)?;
        } else if entry.extension().is_some_and(|e| e == "cohdl") {
            let content = fs::read_to_string(&entry)
                .map_err(|e| format!("cannot read `{}`: {}", entry.display(), e))?;
            let rel = entry.strip_prefix(base).unwrap_or(&entry);
            let display = if prefix.is_empty() {
                rel.display().to_string()
            } else {
                format!("{}/{}", prefix, rel.display())
            };
            out.push((display, content));
        }
    }
    Ok(())
}

struct Manifest {
    name: String,
    top: Option<String>,
}

/// Minimal TOML subset: `[section]` headers and `key = "value"` pairs.
fn parse_manifest(text: &str) -> Result<Manifest, String> {
    let mut section = String::new();
    let mut name = None;
    let mut top = None;
    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(s) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = s.trim().to_string();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!("line {}: expected `key = \"value\"`", lineno + 1));
        };
        let key = key.trim();
        let value = value
            .trim()
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .ok_or_else(|| format!("line {}: expected a quoted string value", lineno + 1))?
            .to_string();
        match (section.as_str(), key) {
            ("package", "name") => name = Some(value),
            ("design", "top") => top = Some(value),
            _ => {} // tolerated: version, future fields
        }
    }
    Ok(Manifest {
        name: name.ok_or("missing `[package] name`")?,
        top,
    })
}

/// Locate the std library: `--std` flag > `COHDL_STD` env > `std/` next to
/// the executable's repo root (dev builds) > `std/` in the current directory.
pub fn find_std_dir(flag: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(p) = flag {
        return Some(p);
    }
    if let Ok(p) = std::env::var("COHDL_STD") {
        return Some(PathBuf::from(p));
    }
    if let Ok(exe) = std::env::current_exe() {
        // target/{debug,release}/cohdl → repo root's std/.
        for ancestor in exe.ancestors().skip(1) {
            let candidate = ancestor.join("std");
            if candidate.join("prelude.cohdl").is_file()
                || (candidate.is_dir() && ancestor.join("Cargo.toml").is_file())
            {
                return Some(candidate);
            }
        }
    }
    let cwd_std = PathBuf::from("std");
    if cwd_std.is_dir() {
        return Some(cwd_std);
    }
    None
}
