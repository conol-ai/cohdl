use crate::http_files::confined_file;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub fn scan_mtime(project: &Path) -> Vec<(PathBuf, SystemTime)> {
    let mut out = Vec::new();
    let mut stack = vec![project.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                if path
                    .file_name()
                    .is_some_and(|n| n != "out" && n != "target")
                {
                    stack.push(path);
                }
                continue;
            }
            if path
                .extension()
                .is_some_and(|x| x == "cohdl" || x.eq_ignore_ascii_case("dxf"))
                || path.file_name().is_some_and(|n| n == "cohdl.toml")
            {
                if let Ok(metadata) = path.metadata() {
                    out.push((path, metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH)));
                }
            }
        }
    }
    out.sort();
    out
}

pub fn scan_views(project: &Path, dist: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
    let mut out = Vec::new();
    for root in [project, dist] {
        let Some(directory) = root
            .join("views")
            .canonicalize()
            .ok()
            .filter(|p| root.canonicalize().is_ok_and(|r| p.starts_with(r)))
        else {
            continue;
        };
        let Ok(entries) = std::fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(name) = name.to_str().filter(|n| n.ends_with(".view.json")) else {
                continue;
            };
            let bytes =
                confined_file(root, &format!("views/{name}")).and_then(|p| std::fs::read(p).ok());
            out.push((root.join("views").join(name), bytes));
        }
    }
    out.sort();
    out
}
