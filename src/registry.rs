//! RFC-030: the registry.cohdl.org client — three-tier namespace grammar,
//! HTTP transport, the package archive format, the local content cache, and
//! the credentials store.
//!
//! Transport is the system `curl` binary: the zero-dependency constitution
//! covers the compiler crate, and RFC-030 grants no dependency exception —
//! shelling out to the platform's own HTTP client keeps the crate clean
//! (documented in docs/compliance-report.md). The archive is uncompressed
//! POSIX tar (the RFC's ".tar.gz (or equivalent)" — DEFLATE is not worth
//! hand-rolling for kilobytes of source text).

use crate::deps::{PackageDiag, Version};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The one official registry (RFC-030); `COHDL_REGISTRY` overrides it for
/// development and tests.
pub fn registry_url() -> String {
    std::env::var("COHDL_REGISTRY").unwrap_or_else(|_| "https://registry.cohdl.org".to_string())
}

/// `$COHDL_HOME` (default `$HOME/.cohdl`): the content cache and credentials.
pub fn cohdl_home() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("COHDL_HOME") {
        return Some(PathBuf::from(p));
    }
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".cohdl"))
}

/// The registry content cache: one package family dir per name (scoped names
/// nest naturally — `@sparkfun/power` → `registry/@sparkfun/power/`).
pub fn cache_root() -> Option<PathBuf> {
    cohdl_home().map(|h| h.join("registry"))
}

// ---------------------------------------------------------------------------
// Three-tier namespace grammar (structural — the name's shape IS its tier)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tier {
    /// Bare name — CoHDL official, never first-come-first-served.
    Official,
    /// `@brand/name` — verified manufacturer.
    Brand(String),
    /// `@contrib/name` — open community namespace.
    Contrib,
}

impl Tier {
    pub fn describe(&self) -> String {
        match self {
            Tier::Official => "official (bare name — reserved for CoHDL's own packages)".into(),
            Tier::Brand(b) => format!("manufacturer (`@{b}/…` — verified brand account)"),
            Tier::Contrib => "community (`@contrib/…` — open namespace)".into(),
        }
    }
}

/// Validate a registry package name against the closed three-tier grammar
/// (RFC-030): bare `name`, `@brand/name`, or `@contrib/name`, each segment
/// in RFC-016's package-name grammar.
pub fn name_tier(name: &str) -> Result<Tier, String> {
    if let Some(rest) = name.strip_prefix('@') {
        let Some((scope, pkg)) = rest.split_once('/') else {
            return Err(format!(
                "`{name}` is not a valid package name — a scoped name is `@scope/name`"
            ));
        };
        if crate::project::valid_package_name(scope).is_err()
            || crate::project::valid_package_name(pkg).is_err()
            || pkg.contains('/')
        {
            return Err(format!(
                "`{name}` is not a valid package name — each segment uses letters, digits, `_`, `-`"
            ));
        }
        if scope == "contrib" {
            Ok(Tier::Contrib)
        } else {
            Ok(Tier::Brand(scope.to_string()))
        }
    } else {
        crate::project::valid_package_name(name).map_err(|e| e.to_string())?;
        Ok(Tier::Official)
    }
}

/// Split a `name@X.Y.Z` argument (the `cohdl add` pin form). The `@` that
/// starts a scoped name is not a version separator.
pub fn split_name_version(arg: &str) -> (String, Option<String>) {
    let split_at = if let Some(rest) = arg.strip_prefix('@') {
        rest.find('@').map(|i| i + 1)
    } else {
        arg.find('@')
    };
    match split_at {
        Some(i) => (arg[..i].to_string(), Some(arg[i + 1..].to_string())),
        None => (arg.to_string(), None),
    }
}

// ---------------------------------------------------------------------------
// Credentials (~/.cohdl/credentials.toml — never committed anywhere)
// ---------------------------------------------------------------------------

pub fn read_token() -> Option<String> {
    let path = cohdl_home()?.join("credentials.toml");
    let text = std::fs::read_to_string(path).ok()?;
    for line in text.lines() {
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == "token" {
                return Some(v.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

pub fn write_token(token: &str) -> Result<PathBuf, String> {
    let home = cohdl_home().ok_or("cannot determine $COHDL_HOME (set HOME or COHDL_HOME)")?;
    std::fs::create_dir_all(&home).map_err(|e| e.to_string())?;
    let path = home.join("credentials.toml");
    std::fs::write(&path, format!("token = \"{}\"\n", token)).map_err(|e| e.to_string())?;
    Ok(path)
}

// ---------------------------------------------------------------------------
// HTTP via the system curl
// ---------------------------------------------------------------------------

pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

fn run_curl(args: &[String]) -> Result<HttpResponse, String> {
    let out = Command::new("curl")
        .args(["-sS", "-w", "\n%{http_code}", "--max-time", "60"])
        .args(args)
        .output()
        .map_err(|e| {
            format!("cannot run `curl`: {e} (the registry client uses the system curl)")
        })?;
    if !out.status.success() && out.stdout.is_empty() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    // The status code rides the last line (after our \n marker).
    let body = out.stdout;
    let split = body
        .iter()
        .rposition(|b| *b == b'\n')
        .ok_or("malformed curl output")?;
    let status: u16 = String::from_utf8_lossy(&body[split + 1..])
        .trim()
        .parse()
        .map_err(|_| "malformed curl status".to_string())?;
    Ok(HttpResponse {
        status,
        body: body[..split].to_vec(),
    })
}

pub fn http_get(url: &str) -> Result<HttpResponse, String> {
    run_curl(&[url.to_string()])
}

/// GET following redirects, with a longer deadline (a repeated `--max-time`
/// overrides run_curl's 60s — curl takes the last occurrence). GitHub
/// release downloads bounce through a CDN, which plain `http_get`
/// deliberately does not follow (`cohdl self-update` is the only caller).
pub fn http_get_follow(url: &str) -> Result<HttpResponse, String> {
    run_curl(&[
        "-L".to_string(),
        "--max-time".to_string(),
        "300".to_string(),
        url.to_string(),
    ])
}

pub fn http_post(
    url: &str,
    body_file: Option<&Path>,
    token: Option<&str>,
    content_type: &str,
) -> Result<HttpResponse, String> {
    let mut args = vec![
        "-X".to_string(),
        "POST".to_string(),
        "-H".to_string(),
        format!("Content-Type: {content_type}"),
    ];
    if let Some(t) = token {
        args.push("-H".to_string());
        args.push(format!("Authorization: Bearer {t}"));
    }
    if let Some(f) = body_file {
        args.push("--data-binary".to_string());
        args.push(format!("@{}", f.display()));
    }
    args.push(url.to_string());
    run_curl(&args)
}

// ---------------------------------------------------------------------------
// Minimal JSON field extraction (the registry's responses are flat objects;
// the crate's hand-rolled-JSON discipline applies to parsing too)
// ---------------------------------------------------------------------------

/// The string value of a top-level `"key": "value"` pair.
pub fn json_str_field(body: &[u8], key: &str) -> Option<String> {
    let text = std::str::from_utf8(body).ok()?;
    let pat = format!("\"{key}\"");
    let at = text.find(&pat)?;
    let rest = &text[at + pat.len()..];
    let rest = rest.trim_start().strip_prefix(':')?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let mut out = String::new();
    let mut chars = rest.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                'n' => out.push('\n'),
                't' => out.push('\t'),
                other => out.push(other),
            },
            other => out.push(other),
        }
    }
    None
}

/// Every string element of a top-level `"key": ["a", "b", …]` array.
pub fn json_str_array(body: &[u8], key: &str) -> Vec<String> {
    let Some(text) = std::str::from_utf8(body).ok() else {
        return Vec::new();
    };
    let pat = format!("\"{key}\"");
    let Some(at) = text.find(&pat) else {
        return Vec::new();
    };
    let rest = &text[at + pat.len()..];
    let Some(open) = rest.find('[') else {
        return Vec::new();
    };
    let Some(close) = rest[open..].find(']') else {
        return Vec::new();
    };
    rest[open + 1..open + close]
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

// ---------------------------------------------------------------------------
// Package archive: uncompressed POSIX tar over the RFC-029 hash file set
// ---------------------------------------------------------------------------

fn tar_header(path: &str, size: u64) -> [u8; 512] {
    let mut h = [0u8; 512];
    let name = path.as_bytes();
    h[..name.len().min(100)].copy_from_slice(&name[..name.len().min(100)]);
    h[100..107].copy_from_slice(b"0000644"); // mode
    h[108..115].copy_from_slice(b"0000000"); // uid
    h[116..123].copy_from_slice(b"0000000"); // gid
    let size_oct = format!("{:011o}", size);
    h[124..124 + 11].copy_from_slice(size_oct.as_bytes());
    h[136..147].copy_from_slice(b"00000000000"); // mtime: epoch, deterministic
    h[156] = b'0'; // regular file
    h[257..262].copy_from_slice(b"ustar");
    h[263..265].copy_from_slice(b"00");
    // Checksum: spaces while summing, then written in octal.
    h[148..156].copy_from_slice(b"        ");
    let sum: u32 = h.iter().map(|b| *b as u32).sum();
    let chk = format!("{:06o}\0 ", sum);
    h[148..156].copy_from_slice(chk.as_bytes());
    h
}

/// Pack a package dir into a deterministic uncompressed tar: the RFC-029
/// hash file set (every regular file, dotfiles excluded), sorted by
/// `/`-normalized relative path, epoch mtimes.
pub fn pack_tar(dir: &Path) -> Result<Vec<u8>, String> {
    let mut files: Vec<(String, PathBuf)> = Vec::new();
    collect_files(dir, dir, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = Vec::new();
    for (rel, path) in files {
        let content =
            std::fs::read(&path).map_err(|e| format!("cannot read `{}`: {e}", path.display()))?;
        out.extend_from_slice(&tar_header(&rel, content.len() as u64));
        out.extend_from_slice(&content);
        let pad = (512 - content.len() % 512) % 512;
        out.extend(std::iter::repeat_n(0u8, pad));
    }
    out.extend(std::iter::repeat_n(0u8, 1024)); // end-of-archive
    Ok(out)
}

fn collect_files(dir: &Path, base: &Path, out: &mut Vec<(String, PathBuf)>) -> Result<(), String> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("cannot read `{}`: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for entry in entries {
        let name = entry.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with('.') {
            continue;
        }
        if entry.is_dir() {
            collect_files(&entry, base, out)?;
        } else if entry.is_file() {
            let rel = entry
                .strip_prefix(base)
                .unwrap_or(&entry)
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            out.push((rel, entry));
        }
    }
    Ok(())
}

/// Unpack a plain tar into `dir`. Path-traversal-safe: every entry must stay
/// under the target (the same containment discipline the build artifacts
/// already enforce).
pub fn unpack_tar(data: &[u8], dir: &Path) -> Result<(), String> {
    let mut off = 0usize;
    while off + 512 <= data.len() {
        let h = &data[off..off + 512];
        if h.iter().all(|b| *b == 0) {
            break; // end-of-archive
        }
        let name_end = h[..100].iter().position(|b| *b == 0).unwrap_or(100);
        let name = std::str::from_utf8(&h[..name_end]).map_err(|_| "bad tar entry name")?;
        let size_field = std::str::from_utf8(&h[124..136]).map_err(|_| "bad tar size")?;
        let size = usize::from_str_radix(size_field.trim_end_matches('\0').trim(), 8)
            .map_err(|_| "bad tar size")?;
        let kind = h[156];
        off += 512;
        if kind == b'0' || kind == 0 {
            if name.split('/').any(|seg| seg == ".." || seg.is_empty()) || name.starts_with('/') {
                return Err(format!("tar entry `{name}` escapes the target directory"));
            }
            let target = dir.join(name);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let content = data.get(off..off + size).ok_or("truncated tar entry")?;
            std::fs::write(&target, content).map_err(|e| e.to_string())?;
        }
        off += size.div_ceil(512) * 512;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// High-level registry operations
// ---------------------------------------------------------------------------

/// E1204 for callers outside this module (the CLI's `publish`/`login`
/// paths). A registry that was never reached — curl reports status 0 — is a
/// different mistake from a rejected publish (E1202) or a rejected token
/// (E1201), and must never be reported as one of those.
pub fn unreachable(detail: String) -> PackageDiag {
    e1204(detail)
}

fn e1204(detail: String) -> PackageDiag {
    PackageDiag::error("E1204", &registry_url(), 0, detail).with_help(
        "registry unreachable is a different failure from a hash mismatch (E1103) — check the network, COHDL_REGISTRY, or vendor the package under deps/".to_string(),
    )
}

/// All published versions of a package, newest-first.
pub fn published_versions(name: &str) -> Result<Vec<Version>, PackageDiag> {
    let url = format!("{}/packages/{}", registry_url(), name);
    let resp = http_get(&url).map_err(e1204)?;
    if resp.status == 404 {
        return Err(PackageDiag::error(
            "E1203",
            &registry_url(),
            0,
            format!("package `{name}` is not published on the registry"),
        ));
    }
    if resp.status != 200 {
        return Err(e1204(format!("GET {url} returned {}", resp.status)));
    }
    let mut versions: Vec<Version> = json_str_array(&resp.body, "versions")
        .iter()
        .filter_map(|v| crate::deps::parse_exact_version(v).ok())
        .collect();
    versions.sort();
    versions.reverse();
    Ok(versions)
}

/// Download one exact version into the cache; returns (package dir, the
/// registry's authoritative content hash). The unpacked content is
/// re-hashed locally and MUST match the server's hash before anything is
/// recorded (RFC-029's guarantee applies from the very first byte).
pub fn download_into_cache(name: &str, version: Version) -> Result<(PathBuf, String), PackageDiag> {
    let reg = registry_url();
    let meta_url = format!("{reg}/packages/{name}/{version}");
    let resp = http_get(&meta_url).map_err(e1204)?;
    if resp.status == 404 {
        return Err(PackageDiag::error(
            "E1203",
            &reg,
            0,
            format!("`{name} {version}` is not published on the registry"),
        ));
    }
    if resp.status != 200 {
        return Err(e1204(format!("GET {meta_url} returned {}", resp.status)));
    }
    let server_hash = json_str_field(&resp.body, "hash")
        .ok_or_else(|| e1204("registry response carries no `hash`".to_string()))?;

    let tar_url = format!("{reg}/packages/{name}/{version}.tar");
    let tar = http_get(&tar_url).map_err(e1204)?;
    if tar.status != 200 {
        return Err(e1204(format!("GET {tar_url} returned {}", tar.status)));
    }

    let cache = cache_root()
        .ok_or_else(|| e1204("cannot determine the cache dir (set HOME or COHDL_HOME)".into()))?;
    let dest = cache.join(name).join(version.to_string());
    let _ = std::fs::remove_dir_all(&dest);
    std::fs::create_dir_all(&dest).map_err(|e| e1204(e.to_string()))?;
    unpack_tar(&tar.body, &dest).map_err(e1204)?;

    let local = crate::hash::package_content_hash(&dest).map_err(e1204)?;
    if local != server_hash {
        let _ = std::fs::remove_dir_all(&dest);
        return Err(PackageDiag::error(
            "E1206",
            &reg,
            0,
            format!(
                "downloaded `{name} {version}` re-hashes as {local}, but the registry declares {server_hash} — refusing to cache corrupted content"
            ),
        ));
    }
    Ok((dest, server_hash))
}
