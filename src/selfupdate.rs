//! `cohdl self-update`: replace the running binary with the newest compiler
//! release published on GitHub (`vX.Y.Z` tags on conol-ai/cohdl).
//!
//! Distribution has no RFC — it is repository tooling, not language surface
//! (ledgered in docs/compliance-report.md). The crate stays dependency-free:
//! transport is the system `curl` (the RFC-030 precedent), the download is
//! verified with src/hash.rs's own SHA-256, and versions are RFC-029 exact
//! triples. The `.tar.gz` is unpacked by the system `tar` — RFC-030's own
//! archive is hand-rolled precisely because it is UNcompressed; DEFLATE is
//! not worth hand-rolling, so the system-tool route extends to tar here.
//!
//! The artifact contract is shared with .github/workflows/release-cohdl.yml
//! (which produces the artifacts) and install.sh (which consumes them the
//! same way this module does): a release tagged `vX.Y.Z` carries one
//! `cohdl-vX.Y.Z-<target>.tar.gz` per supported target — containing the
//! single `cohdl` binary (`cohdl.exe` on Windows) — plus `sha256sums.txt`
//! in `sha256sum` format. Changing any of these names is a three-place
//! change, never one.

use crate::deps::{parse_exact_version, Version};
use crate::registry;
use std::path::Path;

/// The GitHub repository releases are published on.
pub const REPO: &str = "conol-ai/cohdl";

/// The release target this binary self-updates to. On Linux this is always
/// the musl triple regardless of how the running binary was linked — the
/// published Linux builds are static musl precisely so one artifact runs on
/// every distribution.
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
pub const TARGET: &str = "aarch64-apple-darwin";
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
pub const TARGET: &str = "x86_64-apple-darwin";
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub const TARGET: &str = "x86_64-unknown-linux-musl";
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
pub const TARGET: &str = "aarch64-unknown-linux-musl";
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
pub const TARGET: &str = "x86_64-pc-windows-msvc";
#[cfg(not(any(
    all(
        target_os = "macos",
        any(target_arch = "aarch64", target_arch = "x86_64")
    ),
    all(
        target_os = "linux",
        any(target_arch = "aarch64", target_arch = "x86_64")
    ),
    all(target_os = "windows", target_arch = "x86_64"),
)))]
pub const TARGET: &str = "";

/// The release-list endpoint. `COHDL_SELF_UPDATE_API` overrides it for
/// tests (the same dev-override shape as COHDL_REGISTRY).
fn releases_api_url() -> String {
    std::env::var("COHDL_SELF_UPDATE_API")
        .unwrap_or_else(|_| format!("https://api.github.com/repos/{REPO}/releases?per_page=100"))
}

/// The asset download base; `<base>/vX.Y.Z/<asset>`. `COHDL_SELF_UPDATE_DOWNLOAD`
/// overrides it for tests.
fn download_base() -> String {
    std::env::var("COHDL_SELF_UPDATE_DOWNLOAD")
        .unwrap_or_else(|_| format!("https://github.com/{REPO}/releases/download"))
}

/// Every string value of `"key": "value"` in `body`, in document order.
/// GitHub's release list is an array of objects — the registry's flat-object
/// `json_str_field` stops at the first match, so the scan continues here.
/// Same hand-rolled-JSON discipline (and escape handling) as the registry's.
fn json_str_values(body: &[u8], key: &str) -> Vec<String> {
    let Ok(text) = std::str::from_utf8(body) else {
        return Vec::new();
    };
    let pat = format!("\"{key}\"");
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find(&pat) {
        rest = &rest[at + pat.len()..];
        let Some(after_colon) = rest.trim_start().strip_prefix(':') else {
            continue;
        };
        let Some(mut chars) = after_colon
            .trim_start()
            .strip_prefix('"')
            .map(|s| s.chars())
        else {
            continue;
        };
        let mut value = String::new();
        loop {
            match chars.next() {
                None => return out, // unterminated string: stop scanning
                Some('"') => break,
                Some('\\') => match chars.next() {
                    None => return out,
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some(other) => value.push(other),
                },
                Some(other) => value.push(other),
            }
        }
        out.push(value);
        rest = chars.as_str();
    }
    out
}

/// The newest compiler release among `tags`: the maximum over every tag of
/// the form `vX.Y.Z`. Tags of any other shape — the VS Code extension's
/// `vscode-v*`, a hypothetical `v1.0.0-rc1` — fail RFC-029 exact-version
/// parsing and are skipped, so "newest" is decided by version order, never
/// by the API's list order.
pub fn latest_release(tags: &[String]) -> Option<Version> {
    tags.iter()
        .filter_map(|tag| tag.strip_prefix('v'))
        .filter_map(|v| parse_exact_version(v).ok())
        .max()
}

/// Every `tag_name` across the paginated release list. GitHub caps
/// `per_page` at 100 and keeps serving later pages until an empty one — a
/// single-page read could be starved outright once >100 extension
/// (`vscode-v*`) releases postdate the newest compiler tag. The page cap is
/// a runaway guard, far above any plausible release count.
fn fetch_all_tags(api: &str) -> Result<Vec<String>, String> {
    let sep = if api.contains('?') { '&' } else { '?' };
    let mut tags = Vec::new();
    for page in 1..=20u32 {
        let url = format!("{api}{sep}page={page}");
        let resp =
            registry::http_get_follow(&url).map_err(|e| format!("cannot reach {url}: {e}"))?;
        if resp.status != 200 {
            return Err(format!(
                "GET {url} returned HTTP {} (0 means the request never completed — check the network)",
                resp.status
            ));
        }
        let page_tags = json_str_values(&resp.body, "tag_name");
        if page_tags.is_empty() {
            break;
        }
        tags.extend(page_tags);
    }
    Ok(tags)
}

/// The release asset for `version` on this platform.
pub fn asset_name(version: Version) -> String {
    format!("cohdl-v{version}-{TARGET}.tar.gz")
}

/// The hex digest `sha256sums.txt` declares for `asset`. Lines are
/// `sha256sum` format — `<hex>  <name>` — tolerating the `*<name>` binary
/// marker some sha256sum modes emit.
pub fn expected_hash(sums: &str, asset: &str) -> Option<String> {
    for line in sums.lines() {
        let mut fields = line.split_whitespace();
        let (Some(hex), Some(name)) = (fields.next(), fields.next()) else {
            continue;
        };
        if name == asset || name.strip_prefix('*') == Some(asset) {
            return Some(hex.to_string());
        }
    }
    None
}

fn fetch_ok(url: &str) -> Result<Vec<u8>, String> {
    let resp = registry::http_get_follow(url).map_err(|e| format!("cannot fetch {url}: {e}"))?;
    if resp.status != 200 {
        return Err(format!("GET {url} returned HTTP {}", resp.status));
    }
    Ok(resp.body)
}

/// The `cohdl self-update [--check]` command.
pub fn run(check_only: bool) -> Result<bool, String> {
    if TARGET.is_empty() {
        return Err(
            "self-update has no published build for this platform — build from source \
             (https://github.com/conol-ai/cohdl)"
                .to_string(),
        );
    }
    let current =
        parse_exact_version(env!("CARGO_PKG_VERSION")).expect("crate version is an exact triple");

    let tags = fetch_all_tags(&releases_api_url())?;
    let Some(latest) = latest_release(&tags) else {
        return Err(format!(
            "no compiler release (a `vX.Y.Z` tag) found at https://github.com/{REPO}/releases"
        ));
    };
    if latest <= current {
        eprintln!("  cohdl {current} is up to date (newest release: v{latest})");
        return Ok(true);
    }
    if check_only {
        eprintln!(
            "  cohdl {current} -> v{latest} is available; run `cohdl self-update` to install it"
        );
        return Ok(true);
    }

    let asset = asset_name(latest);
    let base = download_base();
    let sums = fetch_ok(&format!("{base}/v{latest}/sha256sums.txt"))?;
    let sums = String::from_utf8_lossy(&sums).into_owned();
    let Some(expected) = expected_hash(&sums, &asset) else {
        return Err(format!(
            "release v{latest} publishes no build for {TARGET} (no `{asset}` in sha256sums.txt)"
        ));
    };
    eprintln!("  downloading {asset} ...");
    let archive = fetch_ok(&format!("{base}/v{latest}/{asset}"))?;
    let actual = crate::hash::sha256_hex(&archive);
    if actual != expected {
        return Err(format!(
            "`{asset}` hashes as {actual}, but sha256sums.txt declares {expected} — \
             refusing to install a corrupted download"
        ));
    }

    // macOS's current_exe may return the symlink the user invoked (Linux's
    // /proc/self/exe is kernel-resolved): canonicalize so the swap replaces
    // the real binary — never turning a symlink node into a divergent copy
    // that leaves the actual install stale.
    let exe = std::env::current_exe()
        .map_err(|e| format!("cannot locate the running executable: {e}"))?;
    let exe = std::fs::canonicalize(&exe).map_err(|e| {
        format!(
            "cannot resolve the running executable `{}`: {e}",
            exe.display()
        )
    })?;
    let dir = exe
        .parent()
        .ok_or("the running executable has no parent directory")?;
    sweep_stale(dir);

    // Stage and unpack in the executable's OWN directory, never temp_dir():
    // /tmp is world-writable on multi-user systems, where a predictable path
    // could be squatted between hash verification and install — and the
    // executable's directory must be user-writable for the final rename
    // anyway, which staging here also makes a same-filesystem move.
    // `create_dir` (not `_all`): never adopt a directory something else made.
    let workdir = dir.join(format!(".cohdl-update-{}", std::process::id()));
    std::fs::create_dir(&workdir)
        .map_err(|e| format!("cannot create `{}`: {e}", workdir.display()))?;
    let result = install_from_archive(&workdir, &asset, &archive, &exe, current, latest);
    let _ = std::fs::remove_dir_all(&workdir);
    result
}

/// Remove what an earlier crashed run may have stranded next to the binary:
/// `.cohdl-update-*` workdirs and, on Windows, the `.old` binary a completed
/// swap cannot delete while it still runs. Concurrent self-updates of the
/// same binary are already a last-write-wins race; the sweep does not try
/// to distinguish one.
fn sweep_stale(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let name = e.file_name();
        let Some(name) = name.to_str() else { continue };
        if name.starts_with(".cohdl-update-") {
            let _ = std::fs::remove_dir_all(e.path());
        }
        if cfg!(windows) && name.ends_with(".exe.old") {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

fn install_from_archive(
    workdir: &Path,
    asset: &str,
    archive: &[u8],
    exe: &Path,
    current: Version,
    latest: Version,
) -> Result<bool, String> {
    let archive_path = workdir.join(asset);
    std::fs::write(&archive_path, archive)
        .map_err(|e| format!("cannot write `{}`: {e}", archive_path.display()))?;
    let bin_name = if cfg!(windows) { "cohdl.exe" } else { "cohdl" };
    // Extract the one expected member by name (same as install.sh): nothing
    // else in the archive — expected or not — ever touches the disk.
    let out = std::process::Command::new("tar")
        .arg("-xzf")
        .arg(&archive_path)
        .arg("-C")
        .arg(workdir)
        .arg(bin_name)
        .output()
        .map_err(|e| format!("cannot run `tar`: {e} (self-update unpacks with the system tar)"))?;
    if !out.status.success() {
        return Err(format!(
            "tar could not unpack `{bin_name}` from `{asset}`: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let staged = workdir.join(bin_name);
    if !staged.is_file() {
        return Err(format!("`{asset}` does not contain `{bin_name}`"));
    }
    replace_exe(&staged, exe)?;
    eprintln!("  updated cohdl {current} -> {latest} ({})", exe.display());
    Ok(true)
}

/// Swap the running executable for the staged binary. The workdir lives in
/// the executable's own directory, so the rename is an atomic
/// same-filesystem move; error paths leave everything inside the workdir,
/// which the caller removes.
#[cfg(unix)]
fn replace_exe(staged: &Path, exe: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(staged, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| format!("cannot mark `{}` executable: {e}", staged.display()))?;
    std::fs::rename(staged, exe).map_err(|e| {
        format!(
            "cannot replace `{}`: {e} (is its directory writable by this user?)",
            exe.display()
        )
    })
}

/// Windows: a running executable can be renamed but never overwritten. The
/// live binary moves aside to `.old`, the new one moves in; the `.old` file
/// cannot be deleted while it still runs — the next run's sweep takes it.
#[cfg(windows)]
fn replace_exe(staged: &Path, exe: &Path) -> Result<(), String> {
    let old = exe.with_extension("exe.old");
    let _ = std::fs::remove_file(&old);
    std::fs::rename(exe, &old)
        .map_err(|e| format!("cannot move the running `{}` aside: {e}", exe.display()))?;
    std::fs::rename(staged, exe).map_err(|e| {
        // Best-effort rollback so a half-swap never leaves no `cohdl` at all.
        let _ = std::fs::rename(&old, exe);
        format!("cannot install the new `{}`: {e}", exe.display())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(s: &str) -> Version {
        parse_exact_version(s).unwrap()
    }

    fn tags_of(body: &[u8]) -> Vec<String> {
        json_str_values(body, "tag_name")
    }

    #[test]
    fn latest_release_is_version_max_not_list_order() {
        // The API returns newest-created first, but "newest" here must be
        // decided by version order (a re-tagged hotfix may be created later).
        let body = br#"[
            {"tag_name": "v0.2.0", "draft": false},
            {"tag_name": "v0.10.1", "draft": false},
            {"tag_name": "v0.9.9", "draft": false}
        ]"#;
        assert_eq!(latest_release(&tags_of(body)), Some(v("0.10.1")));
    }

    #[test]
    fn latest_release_skips_non_compiler_tags() {
        // vscode-v* (the extension's releases) and pre-release shapes fail
        // exact-version parsing and never win.
        let body = br#"[
            {"tag_name": "vscode-v9.9.9"},
            {"tag_name": "v1.0.0-rc1"},
            {"tag_name": "v0.1.0"}
        ]"#;
        assert_eq!(latest_release(&tags_of(body)), Some(v("0.1.0")));
        assert_eq!(
            latest_release(&tags_of(br#"[{"tag_name": "vscode-v9.9.9"}]"#)),
            None
        );
    }

    #[test]
    fn json_str_values_handles_spacing_and_escapes() {
        let body = br#"{"tag_name":"v1.2.3","x":{"tag_name" : "a\"b\nc"}}"#;
        assert_eq!(
            json_str_values(body, "tag_name"),
            vec!["v1.2.3".to_string(), "a\"b\nc".to_string()]
        );
        // Non-string values for the key are skipped, not misparsed.
        assert_eq!(
            json_str_values(br#"{"tag_name": 7}"#, "tag_name"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn expected_hash_reads_sha256sum_format() {
        let sums = "aaaa  cohdl-v0.2.0-aarch64-apple-darwin.tar.gz\n\
                    bbbb *cohdl-v0.2.0-x86_64-unknown-linux-musl.tar.gz\n";
        assert_eq!(
            expected_hash(sums, "cohdl-v0.2.0-aarch64-apple-darwin.tar.gz"),
            Some("aaaa".to_string())
        );
        assert_eq!(
            expected_hash(sums, "cohdl-v0.2.0-x86_64-unknown-linux-musl.tar.gz"),
            Some("bbbb".to_string())
        );
        assert_eq!(expected_hash(sums, "cohdl-v0.2.0-other.tar.gz"), None);
    }

    #[test]
    fn asset_name_carries_tag_and_target() {
        let name = asset_name(v("0.2.0"));
        assert!(name.starts_with("cohdl-v0.2.0-"));
        assert!(name.ends_with(".tar.gz"));
        assert!(name.contains(TARGET));
    }
}
