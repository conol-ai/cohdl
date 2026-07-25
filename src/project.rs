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
    /// Absolute on-disk path per entry in `files` (same order) — used by the
    /// LSP (RFC-014) for URI mapping and unsaved-buffer overlays.
    pub abs_paths: Vec<PathBuf>,
}

/// Load a project from a directory (with `cohdl.toml`) or a single `.cohdl`
/// file. `std_dir` of `None` means "compile without the std library".
pub fn load_project(path: &Path, std_dir: Option<&Path>) -> Result<Project, String> {
    let deps: Vec<(String, PathBuf)> = std_dir
        .map(|d| vec![("std".to_string(), d.to_path_buf())])
        .unwrap_or_default();
    load_project_with_deps(path, &deps)
}

/// RFC-029: load a project against an explicit, already-resolved dependency
/// set — (package name, on-disk dir) pairs, std included (or absent for a
/// std-less build). Each dependency is an ordinary package (`cohdl.toml` +
/// `src/`, the same shape as a project; std included) whose files join the
/// compile under its package name as the module root (RFC-016). A bare
/// directory of `.cohdl` files is accepted too — the `--std` dev-override
/// escape hatch.
pub fn load_project_with_deps(path: &Path, deps: &[(String, PathBuf)]) -> Result<Project, String> {
    let mut files = Vec::new();
    let mut abs_paths = Vec::new();
    for (name, dir) in deps {
        ensure_package_dir(dir, name)?;
        let src = dir.join("src");
        let content_dir = if src.is_dir() { src } else { dir.clone() };
        let before = files.len();
        collect_cohdl_files(&content_dir, &content_dir, name, &mut files, &mut abs_paths)?;
        if files.len() == before {
            return Err(format!(
                "dependency `{}` package `{}` contains no .cohdl files",
                name,
                dir.display()
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
        reject_std_package(&name)?;
        files.push((path.display().to_string(), content));
        abs_paths.push(path.canonicalize().unwrap_or_else(|_| path.to_path_buf()));
        return Ok(Project {
            name,
            dir: path
                .parent()
                .map(Path::to_path_buf)
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or_else(|| PathBuf::from(".")),
            top: None,
            files,
            abs_paths,
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
    reject_std_package(&manifest.name)?;
    crate::registry::name_tier(&manifest.name)
        .map_err(|e| format!("{}: {}", manifest_path.display(), e))?;
    if let Some((dep, _)) = deps.iter().find(|(n, _)| *n == manifest.name) {
        return Err(format!(
            "{}: package name `{}` collides with a dependency of the same name",
            manifest_path.display(),
            dep
        ));
    }

    let src_dir = path.join("src");
    if !src_dir.is_dir() {
        return Err(format!("`{}` has no `src/` directory", path.display()));
    }
    let before = files.len();
    collect_cohdl_files(&src_dir, path, "", &mut files, &mut abs_paths)?;
    if files.len() == before {
        return Err(format!("`{}` contains no .cohdl files", src_dir.display()));
    }

    Ok(Project {
        name: manifest.name,
        dir: path.to_path_buf(),
        top: manifest.top,
        files,
        abs_paths,
    })
}

/// A loaded file set: (display name, content) pairs plus the parallel
/// absolute-path vector.
pub type LoadedFiles = (Vec<(String, String)>, Vec<PathBuf>);

/// Just the std library's files, for callers assembling a synthetic project
/// (the LSP's not-on-disk buffer fallback). `None` std_dir → empty.
pub fn load_std_files(std_dir: Option<&Path>) -> Option<LoadedFiles> {
    let mut files = Vec::new();
    let mut abs = Vec::new();
    if let Some(dir) = std_dir {
        collect_cohdl_files(dir, dir, "std", &mut files, &mut abs).ok()?;
        // A std directory that exists but holds no `.cohdl` files is the same
        // error `load_project` rejects — NOT a silent std-less project. The
        // LSP phantom-buffer fallback relied on the old always-`Some` return
        // to publish a false-clean `[]` (review F8); return `None` so the
        // caller surfaces "cannot load the std library" instead.
        if files.is_empty() {
            return None;
        }
    }
    Some((files, abs))
}

/// Recursively collect `.cohdl` files in sorted order. Display names are
/// `prefix/relative/path` (prefix distinguishes std files in diagnostics).
fn collect_cohdl_files(
    dir: &Path,
    base: &Path,
    prefix: &str,
    out: &mut Vec<(String, String)>,
    abs: &mut Vec<PathBuf>,
) -> Result<(), String> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("cannot read `{}`: {}", dir.display(), e))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for entry in entries {
        if entry.is_dir() {
            collect_cohdl_files(&entry, base, prefix, out, abs)?;
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
            abs.push(entry.canonicalize().unwrap_or(entry));
        }
    }
    Ok(())
}

/// RFC-016: `std` is the standard library's package root — a project that
/// claimed it would MERGE into std's namespace (duplicate errors and
/// cascading diagnostics with spans inside std/ files the user cannot
/// edit; adversarial finding). Rejected up front, precisely.
fn reject_std_package(name: &str) -> Result<(), String> {
    if crate::pipeline::package_root(name) == "std" {
        return Err(format!(
            "package name `{}` is reserved for the standard library — pick another `[package] name`",
            name
        ));
    }
    Ok(())
}

/// The package name becomes the basename of every emitted artifact
/// (`<name>.net`, `<name>.xml`, …) and the root of every module path, so it
/// must be a plain identifier-shaped token. Rejecting path separators,
/// `.`/`..`, and absolute markers closes a directory-traversal hole: a
/// manifest `name = "../escaped"` otherwise wrote — and the stale-artifact
/// cleanup deleted — files outside the output directory (review F4).
pub fn valid_package_name(name: &str) -> Result<(), String> {
    let ok = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        && !name.starts_with('-');
    if ok {
        Ok(())
    } else {
        Err(format!(
            "package name `{}` is not a valid identifier — use letters, digits, `_`, and `-` (no path separators, `.`, or `..`); it becomes both the module root and the output-file basename",
            name
        ))
    }
}

/// The parsed `cohdl.toml`. RFC-029: `[package] version` and the
/// `[dependencies]` section are now real; `deps_raw` is `None` when the
/// manifest carries no `[dependencies]` section at all (the pre-RFC-029
/// state the CLI's migration path detects), `Some(entries)` otherwise —
/// `(name, raw version string, 1-based line)`, validated by
/// `deps::validate_deps`.
pub struct Manifest {
    pub name: String,
    pub top: Option<String>,
    pub version: Option<String>,
    pub deps_raw: Option<Vec<(String, String, u32)>>,
    /// Display-only `[package]` metadata: never affects a verdict, a
    /// designator, or an emitted byte — the registry records it per published
    /// version and the web UI shows it (`cohdl publish` echoes it).
    pub description: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
}

/// RFC-029: the manifest alone (no source collection) — the CLI resolves
/// dependencies from this before any `.cohdl` file is opened.
pub fn peek_manifest(project_dir: &Path) -> Result<(PathBuf, Manifest), String> {
    let manifest_path = project_dir.join("cohdl.toml");
    let text = fs::read_to_string(&manifest_path).map_err(|_| {
        format!(
            "no `cohdl.toml` found in `{}` (a project directory needs a manifest; or pass a single .cohdl file)",
            project_dir.display()
        )
    })?;
    let manifest =
        parse_manifest(&text).map_err(|e| format!("{}: {}", manifest_path.display(), e))?;
    Ok((manifest_path, manifest))
}

/// Minimal TOML subset: `[section]` headers and `key = "value"` pairs.
fn parse_manifest(text: &str) -> Result<Manifest, String> {
    let mut section = String::new();
    let mut name = None;
    let mut top = None;
    let mut version = None;
    let mut description = None;
    let mut license = None;
    let mut repository = None;
    let mut deps_raw: Option<Vec<(String, String, u32)>> = None;
    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(s) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = s.trim().to_string();
            if section == "dependencies" && deps_raw.is_none() {
                deps_raw = Some(Vec::new());
            }
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!("line {}: expected `key = \"value\"`", lineno + 1));
        };
        // RFC-030: scoped dependency names are quoted TOML keys
        // (`"@sparkfun/power" = "1.0.0"`).
        let key = key.trim().trim_matches('"');
        let value = value
            .trim()
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .ok_or_else(|| format!("line {}: expected a quoted string value", lineno + 1))?
            .to_string();
        match (section.as_str(), key) {
            ("package", "name") => name = Some(value),
            ("package", "version") => version = Some(value),
            ("package", "description") => description = Some(value),
            ("package", "license") => license = Some(value),
            ("package", "repository") => repository = Some(value),
            ("design", "top") => top = Some(value),
            ("dependencies", dep) => deps_raw
                .as_mut()
                .expect("[dependencies] header seen")
                .push((dep.to_string(), value, lineno as u32 + 1)),
            _ => {} // tolerated: future fields
        }
    }
    Ok(Manifest {
        name: name.ok_or("missing `[package] name`")?,
        top,
        version,
        deps_raw,
        description,
        license,
        repository,
    })
}

/// RFC-029: refuse a dependency dir that is actually a *versioned root*
/// (version subdirectories, neither a `src/` nor `.cohdl` files of its own)
/// — recursing into one would silently merge every version of the package
/// into the compile.
fn ensure_package_dir(dir: &Path, name: &str) -> Result<(), String> {
    if dir.join("src").is_dir() {
        return Ok(()); // ordinary package shape (cohdl.toml + src/)
    }
    let has_top_level_cohdl = fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .any(|e| e.path().extension().is_some_and(|x| x == "cohdl"))
        })
        .unwrap_or(false);
    if !has_top_level_cohdl {
        if let Some((newest, newest_dir)) = crate::deps::newest_available(dir, name) {
            return Err(format!(
                "`{}` is a package family dir (versions live in its subdirectories), not a package — pass a specific package directory (e.g. `{}`, which declares {} {})",
                dir.display(),
                newest_dir.display(),
                name,
                newest
            ));
        }
    }
    Ok(())
}

/// RFC-029: the `--std`/`COHDL_STD` *development override* — when present,
/// the returned dir is used verbatim as the std package (bypassing the
/// versioned registry), and the CLI emits the mandatory, unsuppressable
/// E1105 warning: the result is not reproducible.
pub fn std_override(flag: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(p) = flag {
        return Some(p);
    }
    std::env::var("COHDL_STD").ok().map(PathBuf::from)
}

/// RFC-029/030: locate the library root — the `lib/` directory whose
/// immediate subdirectories are the package family dirs shipped with the
/// compiler (`lib/std/`, and every other official library beside it).
/// Searched next to the executable's repo root (dev builds), then in the
/// current directory; a candidate counts only if it actually offers a
/// package, so an installed binary's `/usr/lib` is never mistaken for one.
/// This locates the root only — version *selection* comes from the
/// manifest's `[dependencies]` pin (or newest-available for unpinned
/// targets), never from here.
pub fn find_lib_root() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe() {
        // target/{debug,release}/cohdl → repo root's lib/.
        for ancestor in exe.ancestors().skip(1) {
            let candidate = ancestor.join("lib");
            if crate::deps::is_library_root(&candidate) {
                return Some(candidate);
            }
        }
    }
    let cwd_lib = PathBuf::from("lib");
    if crate::deps::is_library_root(&cwd_lib) {
        return Some(cwd_lib);
    }
    None
}

/// The newest std the library root offers — `(version, package dir)`. std is
/// resolved here exactly as any other library would be: its family dir is
/// `lib/std/`, and its manifest declares its version.
pub fn newest_std() -> Option<(crate::deps::Version, PathBuf)> {
    find_lib_root().and_then(|root| crate::deps::newest_available(&root.join("std"), "std"))
}

/// The unpinned std resolution: override verbatim, else the newest version
/// the library root offers. This is the rule for targets with no manifest
/// to pin against — single-file checks and the LSP's overlay analysis.
pub fn find_std_dir(flag: Option<PathBuf>) -> Option<PathBuf> {
    if let Some(p) = std_override(flag) {
        return Some(p);
    }
    newest_std().map(|(_, dir)| dir)
}
