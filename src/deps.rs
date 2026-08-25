//! RFC-029: package dependency versioning — `[dependencies]` validation,
//! registry resolution, and the `cohdl.lock` record.
//!
//! Everything here runs at project load, before any `.cohdl` parsing: an
//! invalid dependency entry (E1101), an unresolvable version (E1102), a
//! locked-hash mismatch (E1103), or a closure version conflict (E1108)
//! gates the entire pipeline. Exact versions only — range syntax is
//! rejected permanently (hardware libraries have no "safe patch"
//! assumption: a patch-level footprint fix moves real copper). Resolution
//! covers the TRANSITIVE dependency closure (RFC-029 amendment,
//! user-directed 2026-08-25): every resolved package's own
//! `[dependencies]` joins the work set, the project's pin is the single
//! authority when names collide, and the lock records the closure.

use crate::hash;
use std::collections::{BTreeMap, VecDeque};
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
        if let Err(reason) = crate::registry::name_tier(name) {
            diags.push(PackageDiag::error(
                "E1101",
                manifest_display,
                *line,
                format!("dependency `{name}`: {reason}"),
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
/// dependency name, uniform for every package including std: the
/// project-local `deps/<name>/` family dir, then the compiler's library
/// root (`lib/<name>/`), then the RFC-030 content cache.
///
/// A family dir offers versions by MANIFEST, never by directory name: it is
/// either itself a package (`cohdl.toml` + `src/`) or a container of
/// arbitrarily-named subdirectories that each are one. The `[package]
/// version` a manifest declares is the sole version authority — a path
/// component that happens to spell a version is convention, not mechanism.
pub struct Registry {
    /// The discovered library root (`lib/`) — the packages that ship with
    /// the compiler, each an ordinary family dir under it. std is one of
    /// them (`lib/std/`), reached by the same rule as every other name.
    pub lib_root: Option<PathBuf>,
    /// `<project>/deps`.
    pub project_deps: PathBuf,
    /// RFC-030: the registry content cache (`~/.cohdl/registry`), populated
    /// by `cohdl install`/`add`/`update`; scoped names nest naturally.
    pub cache_root: Option<PathBuf>,
}

impl Registry {
    /// The family dirs to search for `name`, in precedence order.
    pub fn families(&self, name: &str) -> Vec<PathBuf> {
        let mut f = vec![self.project_deps.join(name)];
        if let Some(lib) = &self.lib_root {
            f.push(lib.join(name));
        }
        if let Some(cache) = &self.cache_root {
            f.push(cache.join(name));
        }
        f
    }
}

/// Whether `dir` is a library root: a directory at least one of whose
/// immediate subdirectories is a readable package family (`lib/std/`,
/// `lib/passives/`, …). Used for root DISCOVERY, so it is soft — a family
/// that cannot be read simply does not count — and it asks about packages,
/// never about one privileged name: an installed binary's ancestors include
/// `/usr/lib`, and content is what tells the two apart.
pub fn is_library_root(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.filter_map(|e| e.ok()).any(|e| {
        let path = e.path();
        path.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| newest_available(&path, name).is_some())
    })
}

/// Every version of `name` a family dir offers, discovered by reading
/// manifests: the dir itself if it carries a `cohdl.toml`, else each
/// immediate subdirectory that does. Hard errors (for resolution contexts):
/// an unparseable manifest, a package declaring a different name than the
/// family it lives under, a declared version that is not an exact `X.Y.Z`,
/// or two packages declaring the same identity.
pub fn available_versions(family: &Path, name: &str) -> Result<Vec<(Version, PathBuf)>, String> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if family.join("cohdl.toml").is_file() {
        candidates.push(family.to_path_buf());
    } else if family.is_dir() {
        let mut subs: Vec<PathBuf> = std::fs::read_dir(family)
            .map_err(|e| format!("cannot read `{}`: {}", family.display(), e))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir() && p.join("cohdl.toml").is_file())
            .collect();
        subs.sort();
        candidates.extend(subs);
    }

    let mut out: Vec<(Version, PathBuf)> = Vec::new();
    for dir in candidates {
        let Some((pkg_name, pkg_version)) = read_package_identity(&dir) else {
            return Err(format!(
                "package at `{}` has a cohdl.toml with no `[package] name`",
                dir.display()
            ));
        };
        if pkg_name != name {
            return Err(format!(
                "package at `{}` declares name `{}` but lives under the `{}` family",
                dir.display(),
                pkg_name,
                name
            ));
        }
        let Some(raw) = pkg_version else {
            return Err(format!(
                "package at `{}` declares no `[package] version`",
                dir.display()
            ));
        };
        let version = parse_exact_version(&raw)
            .map_err(|e| format!("package at `{}`: {}", dir.display(), e))?;
        if let Some((_, other)) = out.iter().find(|(v, _)| *v == version) {
            return Err(format!(
                "two packages declare `{} {}`: `{}` and `{}` — a version is one immutable identity",
                name,
                version,
                other.display(),
                dir.display()
            ));
        }
        out.push((version, dir));
    }
    Ok(out)
}

/// The newest version a family dir offers (soft: discovery contexts —
/// unpinned single-file/LSP resolution, root discovery, migration hints —
/// where an unreadable family simply offers nothing).
pub fn newest_available(family: &Path, name: &str) -> Option<(Version, PathBuf)> {
    available_versions(family, name)
        .ok()?
        .into_iter()
        .max_by_key(|(v, _)| *v)
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
    /// (package name, on-disk dir) for the full dependency closure, in
    /// resolution order — ready for `load_project_with_deps`.
    pub deps: Vec<(String, PathBuf)>,
    pub lock: LockFile,
    pub lock_changed: bool,
}

/// The two knobs the closure walk exposes (RFC-029 amendment, user-directed
/// 2026-08-25: resolution covers the transitive dependency closure, not only
/// the manifest's direct entries).
#[derive(Default)]
pub struct ResolveOpts<'a> {
    /// Names never pulled in transitively. The CLI passes `std` here when a
    /// std override or `--no-std` has already settled std outside the
    /// registry — a dependency's own `std` pin must not re-introduce it.
    /// This is a caller-context escape, not a property of any package name.
    pub skip_transitive: &'a [String],
    /// Fetch a missing (name, version) into some searched family — the
    /// RFC-030 verbs wire this to the registry download. Offline contexts
    /// (check/build/LSP) pass `None` and keep the E1102
    /// "run `cohdl install`" contract.
    #[allow(clippy::type_complexity)]
    pub fetch: Option<&'a mut dyn FnMut(&str, Version) -> Result<(), PackageDiag>>,
}

/// Which manifest first demanded a package — the conflict rule's authority
/// order. The project's own manifest is the single authority: its pin wins
/// over any dependency's pin; two *dependencies* pinning different versions
/// with no project pin is E1108 (exact pins cannot be merged).
enum Requirer {
    Project,
    Dep(String),
}

/// One pending resolution: a dependency entry plus the manifest that
/// declares it (where its diagnostics anchor).
struct Work {
    entry: DepEntry,
    /// `None` = the project's own manifest.
    required_by: Option<String>,
    /// Display path of the declaring manifest.
    display: String,
}

/// Resolve and verify the full dependency closure against the registry and
/// the prior lock. Every resolved package's own `[dependencies]` joins the
/// work set; the lock records the closure. This is RFC-029's load-bearing
/// guarantee: a locked version whose content hash no longer matches is a
/// hard error, never a warning.
pub fn resolve(
    manifest_display: &str,
    lock_display: &str,
    deps: &[DepEntry],
    registry: &Registry,
    prior_lock_text: Option<&str>,
    update: Update,
    mut opts: ResolveOpts<'_>,
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

    // The closure walk: the project's entries seed the settled map (the
    // project pin is the authority for its name), then each resolved
    // package's own `[dependencies]` joins the queue. BFS in declaration
    // order keeps the walk — and every diagnostic it can emit —
    // deterministic.
    let mut settled: BTreeMap<String, (Version, Requirer)> = BTreeMap::new();
    let mut queue: VecDeque<Work> = VecDeque::new();
    for dep in deps {
        settled.insert(dep.name.clone(), (dep.version, Requirer::Project));
        queue.push_back(Work {
            entry: dep.clone(),
            required_by: None,
            display: manifest_display.to_string(),
        });
    }

    while let Some(work) = queue.pop_front() {
        let dep = &work.entry;
        // -- locate the package: versions are discovered from manifests
        //    (the directory name is never consulted), searching the
        //    project-local family then the global one --
        let families = registry.families(&dep.name);
        let locate = |diags: &mut Vec<PackageDiag>| -> Result<Option<PathBuf>, Vec<Version>> {
            let mut available: Vec<Version> = Vec::new();
            for family in &families {
                match available_versions(family, &dep.name) {
                    Ok(versions) => {
                        if let Some((_, dir)) = versions.iter().find(|(v, _)| *v == dep.version) {
                            return Ok(Some(dir.clone()));
                        }
                        available.extend(versions.iter().map(|(v, _)| *v));
                    }
                    Err(e) => {
                        diags.push(PackageDiag::error("E1106", &work.display, dep.line, e));
                        return Ok(None);
                    }
                }
            }
            Err(available)
        };
        let mut found: Option<PathBuf> = None;
        let mut available: Vec<Version> = Vec::new();
        let mut hard_failed = false;
        match locate(&mut diags) {
            Ok(Some(dir)) => found = Some(dir),
            Ok(None) => hard_failed = true, // E1106 already pushed
            Err(avail) => available = avail,
        }
        if hard_failed {
            continue;
        }
        // Not on disk anywhere: give the caller's fetch hook one shot at
        // populating a family (the RFC-030 cache), then look again.
        if found.is_none() {
            if let Some(fetch) = opts.fetch.as_deref_mut() {
                match fetch(&dep.name, dep.version) {
                    Ok(()) => match locate(&mut diags) {
                        Ok(Some(dir)) => found = Some(dir),
                        Ok(None) => continue,
                        Err(avail) => available = avail,
                    },
                    Err(d) => {
                        diags.push(d);
                        continue;
                    }
                }
            }
        }
        let Some(dir) = found else {
            available.sort();
            available.dedup();
            let requirer_note = match &work.required_by {
                Some(pkg) => format!(" (required by `{pkg}`)"),
                None => String::new(),
            };
            let mut d = PackageDiag::error(
                "E1102",
                &work.display,
                dep.line,
                format!(
                    "cannot resolve `{} = \"{}\"`{} — no package on disk declares that version",
                    dep.name, dep.version, requirer_note
                ),
            )
            .with_help(format!(
                "searched: {}",
                families
                    .iter()
                    .map(|c| format!("`{}`", c.display()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            if !available.is_empty() {
                d = d.with_help(format!(
                    "available: {}",
                    available
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            d = d.with_help(
                "run `cohdl install` to fetch published packages from the registry".to_string(),
            );
            diags.push(d);
            continue;
        };

        // -- content hash --
        let actual = match hash::package_content_hash(&dir) {
            Ok(h) => h,
            Err(e) => {
                diags.push(PackageDiag::error("E1102", &work.display, dep.line, e));
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
        out.deps.push((dep.name.clone(), dir.clone()));

        // -- the package's own [dependencies] join the closure --
        let dep_manifest_display = dir.join("cohdl.toml").display().to_string();
        let raw = match read_manifest_deps_raw(&dir) {
            Ok(raw) => raw,
            Err(e) => {
                diags.push(PackageDiag::error("E1101", &dep_manifest_display, 0, e));
                continue;
            }
        };
        let sub_entries = match validate_deps(&dep_manifest_display, &raw) {
            Ok(entries) => entries,
            Err(mut sub_diags) => {
                diags.append(&mut sub_diags);
                continue;
            }
        };
        for sub in sub_entries {
            if opts.skip_transitive.iter().any(|s| s == &sub.name) {
                continue;
            }
            match settled.get(&sub.name) {
                None => {
                    settled.insert(
                        sub.name.clone(),
                        (sub.version, Requirer::Dep(dep.name.clone())),
                    );
                    queue.push_back(Work {
                        entry: sub.clone(),
                        required_by: Some(dep.name.clone()),
                        display: dep_manifest_display.clone(),
                    });
                }
                Some((chosen, _)) if *chosen == sub.version => {} // agreement
                Some((_, Requirer::Project)) => {}                // the project pin wins
                Some((chosen, Requirer::Dep(first))) => {
                    diags.push(
                        PackageDiag::error(
                            "E1108",
                            &dep_manifest_display,
                            sub.line,
                            format!(
                                "dependency version conflict: `{}` is required as {} by `{}` and as {} by `{}`",
                                sub.name, chosen, first, sub.version, dep.name
                            ),
                        )
                        .with_help(format!(
                            "exact pins cannot be merged — pin `{}` in this project's [dependencies] to choose the version every package compiles against",
                            sub.name
                        )),
                    );
                }
            }
        }
    }

    // Locked entries no longer in the closure are dropped (the lock mirrors
    // the resolved dependency closure exactly).
    let stale: Vec<String> = out
        .lock
        .entries
        .keys()
        .filter(|k| !settled.contains_key(k.as_str()))
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

/// The `[dependencies]` a package's own manifest declares — raw
/// (name, value, 1-based line) triples, the same shape the project loader
/// hands `validate_deps`. A manifest with no `[dependencies]` section
/// declares none; a malformed line inside the section is a hard error (the
/// closure cannot be trusted past it). Callers have already resolved the
/// package, so a missing manifest is an error too — except the bare-directory
/// escape hatch (`--std` dev override), which has no manifest to read.
pub fn read_manifest_deps_raw(dir: &Path) -> Result<Vec<(String, String, u32)>, String> {
    let path = dir.join("cohdl.toml");
    if !path.is_file() {
        return Ok(Vec::new()); // bare-directory override package
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("cannot read `{}`: {}", path.display(), e))?;
    let mut section = String::new();
    let mut out = Vec::new();
    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(s) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            section = s.trim().to_string();
            continue;
        }
        if section != "dependencies" {
            continue;
        }
        let malformed = || {
            format!(
                "cannot read `{}` line {}: expected `name = \"X.Y.Z\"` under [dependencies]",
                path.display(),
                lineno + 1
            )
        };
        let Some((key, value)) = line.split_once('=') else {
            return Err(malformed());
        };
        let key = key.trim().trim_matches('"');
        let Some(value) = value
            .trim()
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
        else {
            return Err(malformed());
        };
        out.push((key.to_string(), value.to_string(), lineno as u32 + 1));
    }
    Ok(out)
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
