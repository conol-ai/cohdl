//! RFC-029: package dependency versioning — `[dependencies]` validation,
//! registry resolution, and the `cohdl.lock` record.
//!
//! Everything here runs at project load, before any `.cohdl` parsing: an
//! invalid dependency entry (E1101), an unresolvable version (E1102), or a
//! locked-hash mismatch (E1103) gates the entire pipeline. Exact versions
//! only — range syntax is rejected permanently (hardware libraries have no
//! "safe patch" assumption: a patch-level footprint fix moves real copper).

use crate::hash;
use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Versions: exact semver triples, canonical form only
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Parse an exact `X.Y.Z` version. Rejects range operators, wildcards,
/// pre-release/build suffixes, and non-canonical components (leading zeros).
pub fn parse_exact_version(s: &str) -> Result<Version, String> {
    const RANGE_CHARS: [char; 8] = ['^', '~', '>', '<', '*', ',', '=', ' '];
    if s.contains(RANGE_CHARS) {
        return Err(format!(
            "`{s}` is a version range — CoHDL requires exact versions (a hardware \
             library's \"patch\" can move real copper; every bump is an explicit `cohdl update`)"
        ));
    }
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("`{s}` is not an exact `X.Y.Z` version"));
    }
    let mut nums = [0u32; 3];
    for (i, p) in parts.iter().enumerate() {
        if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!(
                "`{s}` is not an exact `X.Y.Z` version (component `{p}` is not a number)"
            ));
        }
        if p.len() > 1 && p.starts_with('0') {
            return Err(format!(
                "`{s}` is not canonical — component `{p}` has a leading zero"
            ));
        }
        nums[i] = p
            .parse()
            .map_err(|_| format!("`{s}`: component `{p}` is out of range"))?;
    }
    Ok(Version {
        major: nums[0],
        minor: nums[1],
        patch: nums[2],
    })
}

/// Best-effort "nearest valid exact version" for the E1101 help line:
/// strip range operators, take the first comma-clause, pad to three parts.
pub fn suggest_exact(s: &str) -> Option<Version> {
    let first = s.split(',').next()?.trim();
    let stripped: String = first
        .trim_start_matches(['^', '~', '>', '<', '='])
        .trim()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    if stripped.is_empty() {
        return None;
    }
    let mut nums = [0u32; 3];
    for (i, p) in stripped.split('.').take(3).enumerate() {
        nums[i] = p.parse().ok()?;
    }
    Some(Version {
        major: nums[0],
        minor: nums[1],
        patch: nums[2],
    })
}

// ---------------------------------------------------------------------------
// Diagnostics (pre-source: file + line, no span)
// ---------------------------------------------------------------------------

/// A package-resolution diagnostic — E11xx, anchored to a manifest or lock
/// file line rather than a `.cohdl` span (nothing has been parsed yet).
#[derive(Debug, Clone)]
pub struct PackageDiag {
    pub code: &'static str,
    pub severity: &'static str, // "error" | "warning"
    pub message: String,
    /// Display path of the file the diagnostic anchors to.
    pub file: String,
    /// 1-based line; 0 = the file as a whole.
    pub line: u32,
    pub help: Vec<String>,
}

impl PackageDiag {
    pub fn error(code: &'static str, file: &str, line: u32, message: String) -> Self {
        PackageDiag {
            code,
            severity: "error",
            message,
            file: file.to_string(),
            line,
            help: Vec::new(),
        }
    }

    pub fn warning(code: &'static str, file: &str, line: u32, message: String) -> Self {
        PackageDiag {
            severity: "warning",
            ..PackageDiag::error(code, file, line, message)
        }
    }

    pub fn with_help(mut self, help: String) -> Self {
        self.help.push(help);
        self
    }
}

/// Human rendering, matching the source-diagnostic style as closely as a
/// span-less diagnostic can.
pub fn render_human(diags: &[PackageDiag]) -> String {
    let mut out = String::new();
    for d in diags {
        out.push_str(&format!("{}[{}]: {}\n", d.severity, d.code, d.message));
        if d.line > 0 {
            out.push_str(&format!("  --> {}:{}\n", d.file, d.line));
        } else {
            out.push_str(&format!("  --> {}\n", d.file));
        }
        for h in &d.help {
            out.push_str(&format!("  = help: {}\n", h));
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Manifest [dependencies] validation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct DepEntry {
    pub name: String,
    pub version: Version,
    /// 1-based line of the entry in `cohdl.toml`.
    pub line: u32,
}

/// Validate raw `[dependencies]` pairs (name, value, line) from the manifest
/// into exact-version entries. E1101 for range syntax, non-canonical or
/// malformed versions, invalid names, and duplicates.
pub fn validate_deps(
    manifest_display: &str,
    raw: &[(String, String, u32)],
) -> Result<Vec<DepEntry>, Vec<PackageDiag>> {
    let mut entries: Vec<DepEntry> = Vec::new();
    let mut diags = Vec::new();
    for (name, value, line) in raw {
        if crate::project::valid_package_name(name).is_err() {
            diags.push(PackageDiag::error(
                "E1101",
                manifest_display,
                *line,
                format!("`{name}` is not a valid dependency name"),
            ));
            continue;
        }
        if entries.iter().any(|e| &e.name == name) {
            diags.push(PackageDiag::error(
                "E1101",
                manifest_display,
                *line,
                format!("dependency `{name}` is declared more than once"),
            ));
            continue;
        }
        match parse_exact_version(value) {
            Ok(version) => entries.push(DepEntry {
                name: name.clone(),
                version,
                line: *line,
            }),
            Err(reason) => {
                let mut d = PackageDiag::error(
                    "E1101",
                    manifest_display,
                    *line,
                    format!("dependency `{name}`: {reason}"),
                );
                if let Some(v) = suggest_exact(value) {
                    d = d.with_help(format!("did you mean `{name} = \"{v}\"`?"));
                }
                diags.push(d);
            }
        }
    }
    if diags.is_empty() {
        Ok(entries)
    } else {
        Err(diags)
    }
}

// ---------------------------------------------------------------------------
// cohdl.lock
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockEntry {
    pub version: Version,
    pub hash: String,
}

#[derive(Debug, Clone, Default)]
pub struct LockFile {
    /// name → entry; BTreeMap so rendering is byte-stable.
    pub entries: BTreeMap<String, LockEntry>,
}

impl LockFile {
    /// Parse the strict `[[package]]` table format `render` emits.
    pub fn parse(text: &str) -> Result<LockFile, String> {
        let mut entries = BTreeMap::new();
        let mut cur: Option<(Option<String>, Option<Version>, Option<String>)> = None;
        let mut close =
            |cur: &mut Option<(Option<String>, Option<Version>, Option<String>)>| -> Result<(), String> {
                if let Some((name, version, hash)) = cur.take() {
                    let name = name.ok_or("a [[package]] table is missing `name`")?;
                    let version = version
                        .ok_or_else(|| format!("[[package]] `{name}` is missing `version`"))?;
                    let hash =
                        hash.ok_or_else(|| format!("[[package]] `{name}` is missing `hash`"))?;
                    if entries
                        .insert(name.clone(), LockEntry { version, hash })
                        .is_some()
                    {
                        return Err(format!("[[package]] `{name}` appears more than once"));
                    }
                }
                Ok(())
            };
        for (lineno, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line == "[[package]]" {
                close(&mut cur)?;
                cur = Some((None, None, None));
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(format!("line {}: expected `key = \"value\"`", lineno + 1));
            };
            let value = value
                .trim()
                .strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .ok_or_else(|| format!("line {}: expected a quoted value", lineno + 1))?;
            let slot = cur
                .as_mut()
                .ok_or_else(|| format!("line {}: entry outside a [[package]] table", lineno + 1))?;
            match key.trim() {
                "name" => slot.0 = Some(value.to_string()),
                "version" => {
                    slot.1 = Some(
                        parse_exact_version(value)
                            .map_err(|e| format!("line {}: {}", lineno + 1, e))?,
                    )
                }
                "hash" => slot.2 = Some(value.to_string()),
                other => return Err(format!("line {}: unknown key `{}`", lineno + 1, other)),
            }
        }
        close(&mut cur)?;
        Ok(LockFile { entries })
    }

    /// Byte-stable rendering: header comment, one `[[package]]` table per
    /// dependency, sorted by name.
    pub fn render(&self) -> String {
        let mut out = String::from(
            "# cohdl.lock — generated by cohdl. Do not hand-edit; run `cohdl update` to change a pin.\n",
        );
        for (name, e) in &self.entries {
            out.push_str("\n[[package]]\n");
            out.push_str(&format!("name = \"{}\"\n", name));
            out.push_str(&format!("version = \"{}\"\n", e.version));
            out.push_str(&format!("hash = \"{}\"\n", e.hash));
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Registry + resolution
// ---------------------------------------------------------------------------

/// Where packages live on disk (RFC-029 defines the mechanism assuming
/// on-disk availability; hosting is future work). Search order per
/// dependency `name@X.Y.Z`:
///   1. `<project>/deps/<name>/<X.Y.Z>/`   (project-local packages)
///   2. `<global>/<name>/<X.Y.Z>/`          (the registry root — the
///      directory that contains the discovered `std/`; std itself resolves
///      to `<std_root>/<X.Y.Z>/`)
///
/// Every resolved dir is an ordinary package: `cohdl.toml` + `src/`.
pub struct Registry {
    /// The versioned std root (contains `X.Y.Z/` subdirectories).
    pub std_root: Option<PathBuf>,
    /// `<project>/deps`.
    pub project_deps: PathBuf,
}

impl Registry {
    fn candidates(&self, name: &str, version: Version) -> Vec<PathBuf> {
        let v = version.to_string();
        let mut c = vec![self.project_deps.join(name).join(&v)];
        if let Some(std_root) = &self.std_root {
            if name == "std" {
                c.push(std_root.join(&v));
            } else if let Some(parent) = std_root.parent() {
                c.push(parent.join(name).join(&v));
            }
        }
        c
    }
}

/// How the lock should treat already-locked entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Update {
    /// Ordinary build: verify locked hashes; only new entries (or entries
    /// whose manifest version changed) are (re-)resolved.
    No,
    /// `cohdl update`: re-resolve every dependency.
    All,
    /// `cohdl update --dep <name>`: re-resolve one.
    One(String),
}

pub struct Resolved {
    /// (package name, on-disk dir), in manifest order — ready for
    /// `load_project_with_deps`.
    pub deps: Vec<(String, PathBuf)>,
    pub lock: LockFile,
    pub lock_changed: bool,
}

/// Resolve and verify every dependency against the registry and the prior
/// lock. This is RFC-029's load-bearing guarantee: a locked version whose
/// content hash no longer matches is a hard error, never a warning.
pub fn resolve(
    manifest_display: &str,
    lock_display: &str,
    deps: &[DepEntry],
    registry: &Registry,
    prior_lock_text: Option<&str>,
    update: Update,
) -> Result<Resolved, Vec<PackageDiag>> {
    let prior = match prior_lock_text {
        Some(text) => match LockFile::parse(text) {
            Ok(l) => l,
            Err(e) => {
                return Err(vec![PackageDiag::error(
                    "E1107",
                    lock_display,
                    0,
                    format!("cannot parse cohdl.lock: {e}"),
                )
                .with_help(
                    "cohdl.lock is machine-generated — delete it and rebuild to re-resolve, or restore it from version control".to_string(),
                )]);
            }
        },
        None => LockFile::default(),
    };

    let mut out = Resolved {
        deps: Vec::new(),
        lock: prior.clone(),
        lock_changed: false,
    };
    let mut diags = Vec::new();

    for dep in deps {
        // -- locate the package content --
        let candidates = registry.candidates(&dep.name, dep.version);
        let Some(dir) = candidates.iter().find(|c| c.is_dir()).cloned() else {
            diags.push(
                PackageDiag::error(
                    "E1102",
                    manifest_display,
                    dep.line,
                    format!(
                        "cannot resolve `{} = \"{}\"` — no such package version on disk",
                        dep.name, dep.version
                    ),
                )
                .with_help(format!(
                    "looked in: {}",
                    candidates
                        .iter()
                        .map(|c| format!("`{}`", c.display()))
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            );
            continue;
        };

        // -- a package is a package: its own cohdl.toml must exist and agree
        //    (std is not special — RFC-029 makes it an ordinary package) --
        match read_package_identity(&dir) {
            None => {
                diags.push(
                    PackageDiag::error(
                        "E1106",
                        manifest_display,
                        dep.line,
                        format!(
                            "package at `{}` carries no cohdl.toml — every package declares `[package] name`/`version`",
                            dir.display()
                        ),
                    ),
                );
                continue;
            }
            Some((pkg_name, pkg_version)) => {
                if pkg_name != dep.name
                    || pkg_version.as_deref().map(parse_exact_version) != Some(Ok(dep.version))
                {
                    diags.push(PackageDiag::error(
                        "E1106",
                        manifest_display,
                        dep.line,
                        format!(
                            "package at `{}` declares itself `{} {}` but was resolved as `{} {}`",
                            dir.display(),
                            pkg_name,
                            pkg_version.as_deref().unwrap_or("(no version)"),
                            dep.name,
                            dep.version
                        ),
                    ));
                    continue;
                }
            }
        }

        // -- content hash --
        let actual = match hash::package_content_hash(&dir) {
            Ok(h) => h,
            Err(e) => {
                diags.push(PackageDiag::error("E1102", manifest_display, dep.line, e));
                continue;
            }
        };

        let force =
            matches!(update, Update::All) || matches!(&update, Update::One(n) if n == &dep.name);
        let prior_entry = prior.entries.get(&dep.name);
        match prior_entry {
            Some(locked) if !force && locked.version == dep.version => {
                // The locked case: hash must match, byte for byte.
                if locked.hash != actual {
                    diags.push(
                        PackageDiag::error(
                            "E1103",
                            lock_display,
                            0,
                            format!(
                                "locked package `{} {}` has changed on disk: locked {}, found {}",
                                dep.name, dep.version, locked.hash, actual
                            ),
                        )
                        .with_help(
                            "the content of a locked version must never change; if this bump is intentional, publish it as a new version and run `cohdl update`"
                                .to_string(),
                        ),
                    );
                    continue;
                }
            }
            _ => {
                // First resolution, manifest version change, or forced update:
                // (re-)record the row.
                let new_entry = LockEntry {
                    version: dep.version,
                    hash: actual,
                };
                if prior_entry != Some(&new_entry) {
                    out.lock_changed = true;
                }
                out.lock.entries.insert(dep.name.clone(), new_entry);
            }
        }
        out.deps.push((dep.name.clone(), dir));
    }

    // Locked entries no longer in the manifest are dropped (the lock mirrors
    // the manifest's dependency set exactly).
    let manifest_names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
    let stale: Vec<String> = out
        .lock
        .entries
        .keys()
        .filter(|k| !manifest_names.contains(&k.as_str()))
        .cloned()
        .collect();
    for k in stale {
        out.lock.entries.remove(&k);
        out.lock_changed = true;
    }

    if diags.is_empty() {
        Ok(out)
    } else {
        Err(diags)
    }
}

/// `[package] name`/`version` from a package's own `cohdl.toml` (required
/// for every registry-resolved package — None means there is no manifest).
fn read_package_identity(dir: &Path) -> Option<(String, Option<String>)> {
    let text = std::fs::read_to_string(dir.join("cohdl.toml")).ok()?;
    let mut section = String::new();
    let mut name = None;
    let mut version = None;
    for raw in text.lines() {
        let line = raw.trim();
        if let Some(s) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = s.trim().to_string();
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim().trim_matches('"');
            if section == "package" {
                match key.trim() {
                    "name" => name = Some(value.to_string()),
                    "version" => version = Some(value.to_string()),
                    _ => {}
                }
            }
        }
    }
    name.map(|n| (n, version))
}

/// The newest version directory under a versioned package root — the
/// resolution rule for unpinned targets (single-file checks, the LSP's
/// overlay analysis) where no manifest exists to pin against.
pub fn newest_version_in(root: &Path) -> Option<(Version, PathBuf)> {
    let mut best: Option<(Version, PathBuf)> = None;
    for entry in std::fs::read_dir(root).ok()?.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if let Ok(v) = parse_exact_version(name) {
            if best.as_ref().is_none_or(|(b, _)| v > *b) {
                best = Some((v, path));
            }
        }
    }
    best
}
