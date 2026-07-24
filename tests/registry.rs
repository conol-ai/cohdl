//! RFC-030: the registry client — CLI verbs (login/publish/add/remove/
//! install/update) exercised against a hand-rolled mock HTTP registry, plus
//! the three-tier name grammar and the tar round-trip. Each test isolates
//! its own COHDL_HOME (cache + credentials) and COHDL_REGISTRY.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;

fn cohdl() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cohdl"))
}

fn tmp_dir(tag: &str) -> PathBuf {
    let tmp = std::env::temp_dir().join(format!("cohdl-reg-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    tmp
}

/// A canned package the mock registry serves.
struct MockPkg {
    name: String,
    version: String,
    tar: Vec<u8>,
    hash: String,
}

/// Build a real on-disk package (cohdl.toml + src/lib.cohdl), pack it with
/// the client's own tar writer, and hash it with the compiler's own recipe —
/// the mock then serves bytes whose hash genuinely verifies.
fn make_pkg(stage: &Path, name: &str, version: &str) -> MockPkg {
    let dir = stage.join(format!("stage-{}", version));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("cohdl.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"{version}\"\n"),
    )
    .unwrap();
    std::fs::write(
        dir.join("src/lib.cohdl"),
        format!(
            "pub device W{} {{ pins {{ A: 1 [passive], B: 2 [passive] }} }}\n",
            version.replace('.', "_")
        ),
    )
    .unwrap();
    let tar = cohdl::registry::pack_tar(&dir).unwrap();
    let hash = cohdl::hash::package_content_hash(&dir).unwrap();
    MockPkg {
        name: name.to_string(),
        version: version.to_string(),
        tar,
        hash,
    }
}

/// Minimal HTTP/1.1 mock registry on a loopback port. Routes per RFC-030's
/// contract; publishes require `Bearer tok123`. Runs until the process ends.
fn spawn_mock(pkgs: Vec<MockPkg>, publish_response: Option<(u16, String)>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut s) = stream else { continue };
            let mut buf = Vec::new();
            let mut tmp = [0u8; 4096];
            // Read headers.
            let header_end = loop {
                let n = s.read(&mut tmp).unwrap_or(0);
                if n == 0 {
                    break None;
                }
                buf.extend_from_slice(&tmp[..n]);
                if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                    break Some(pos + 4);
                }
            };
            let Some(header_end) = header_end else {
                continue;
            };
            let head = String::from_utf8_lossy(&buf[..header_end]).to_string();
            let mut lines = head.lines();
            let request = lines.next().unwrap_or_default().to_string();
            let mut content_length = 0usize;
            let mut expect_continue = false;
            let mut authed = false;
            for l in lines {
                let ll = l.to_ascii_lowercase();
                if let Some(v) = ll.strip_prefix("content-length:") {
                    content_length = v.trim().parse().unwrap_or(0);
                }
                if ll.starts_with("expect:") && ll.contains("100-continue") {
                    expect_continue = true;
                }
                if ll.starts_with("authorization:") && l.contains("Bearer tok123") {
                    authed = true;
                }
            }
            if expect_continue {
                let _ = s.write_all(b"HTTP/1.1 100 Continue\r\n\r\n");
            }
            let mut body = buf[header_end..].to_vec();
            while body.len() < content_length {
                let n = s.read(&mut tmp).unwrap_or(0);
                if n == 0 {
                    break;
                }
                body.extend_from_slice(&tmp[..n]);
            }

            let mut parts = request.split_whitespace();
            let method = parts.next().unwrap_or_default().to_string();
            let path = parts.next().unwrap_or_default().to_string();
            let (status, ctype, resp): (u16, &str, Vec<u8>) =
                route(&pkgs, &publish_response, &method, &path, authed);
            let _ = s.write_all(
                format!(
                    "HTTP/1.1 {} X\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    status,
                    ctype,
                    resp.len()
                )
                .as_bytes(),
            );
            let _ = s.write_all(&resp);
        }
    });
    url
}

fn route(
    pkgs: &[MockPkg],
    publish_response: &Option<(u16, String)>,
    method: &str,
    path: &str,
    authed: bool,
) -> (u16, &'static str, Vec<u8>) {
    let json = "application/json";
    if method == "POST" && path == "/login" {
        return if authed {
            (200, json, br#"{"account":"tester"}"#.to_vec())
        } else {
            (401, json, br#"{"error":"bad token"}"#.to_vec())
        };
    }
    if method == "POST" && path.starts_with("/packages/") {
        if !authed {
            return (401, json, br#"{"error":"login required"}"#.to_vec());
        }
        let (status, body) = publish_response
            .clone()
            .unwrap_or((200, r#"{"hash":"sha256:feedface"}"#.to_string()));
        return (status, json, body.into_bytes());
    }
    if method == "GET" {
        let rest = path.strip_prefix("/packages/").unwrap_or_default();
        // `<name>/<version>.tar` | `<name>/<version>` | `<name>` — the name
        // itself may contain one `/` (scoped names).
        if let Some(stem) = rest.strip_suffix(".tar") {
            if let Some((name, version)) = split_last(stem) {
                if let Some(p) = pkgs.iter().find(|p| p.name == name && p.version == version) {
                    return (200, "application/x-tar", p.tar.clone());
                }
            }
            return (404, json, br#"{"error":"not found"}"#.to_vec());
        }
        if let Some((name, version)) = split_last(rest) {
            if version.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                if let Some(p) = pkgs.iter().find(|p| p.name == name && p.version == version) {
                    return (
                        200,
                        json,
                        format!(
                            r#"{{"name":"{}","version":"{}","hash":"{}"}}"#,
                            p.name, p.version, p.hash
                        )
                        .into_bytes(),
                    );
                }
                return (404, json, br#"{"error":"not found"}"#.to_vec());
            }
        }
        // Version list for `rest` as a whole name.
        let versions: Vec<String> = pkgs
            .iter()
            .filter(|p| p.name == rest)
            .map(|p| format!("\"{}\"", p.version))
            .collect();
        if versions.is_empty() {
            return (404, json, br#"{"error":"not found"}"#.to_vec());
        }
        return (
            200,
            json,
            format!(
                r#"{{"name":"{}","versions":[{}]}}"#,
                rest,
                versions.join(",")
            )
            .into_bytes(),
        );
    }
    (404, json, br#"{"error":"not found"}"#.to_vec())
}

/// Split `a/b/c` into (`a/b`, `c`).
fn split_last(s: &str) -> Option<(&str, &str)> {
    s.rfind('/').map(|i| (&s[..i], &s[i + 1..]))
}

/// A minimal consumer project (no design — declarations check only).
fn make_project(root: &Path, deps: &str) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        format!("[package]\nname = \"t\"\n\n[dependencies]\n{deps}"),
    )
    .unwrap();
    std::fs::write(
        root.join("src/main.cohdl"),
        "pub device Local { pins { A: 1 [passive] } }\n",
    )
    .unwrap();
}

fn run(url: &str, home: &Path, args: &[&str]) -> (bool, String) {
    let out = cohdl()
        .env("COHDL_REGISTRY", url)
        .env("COHDL_HOME", home)
        .args(args)
        .output()
        .unwrap();
    (
        out.status.success(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

// ---------------------------------------------------------------------------

#[test]
fn add_fetches_writes_manifest_and_lock() {
    let tmp = tmp_dir("add");
    let home = tmp.join("home");
    let pkg = make_pkg(&tmp, "@contrib/widgets", "1.0.0");
    let expected_hash = pkg.hash.clone();
    let url = spawn_mock(vec![pkg], None);
    let proj = tmp.join("proj");
    make_project(&proj, "");

    let (ok, err) = run(
        &url,
        &home,
        &["add", "@contrib/widgets", proj.to_str().unwrap()],
    );
    assert!(ok, "{err}");
    assert!(err.contains("community"), "tier shown: {err}");

    let manifest = std::fs::read_to_string(proj.join("cohdl.toml")).unwrap();
    assert!(
        manifest.contains("\"@contrib/widgets\" = \"1.0.0\""),
        "scoped names are quoted keys:\n{manifest}"
    );
    let lock = std::fs::read_to_string(proj.join("cohdl.lock")).unwrap();
    assert!(lock.contains("name = \"@contrib/widgets\""), "{lock}");
    assert!(
        lock.contains(&expected_hash),
        "server hash recorded: {lock}"
    );
    assert!(
        home.join("registry/@contrib/widgets/1.0.0/src/lib.cohdl")
            .is_file(),
        "content cached"
    );

    // The cached content satisfies a plain (offline) check.
    let (ok, err) = run(&url, &home, &["check", proj.to_str().unwrap(), "--no-std"]);
    assert!(ok, "{err}");
}

#[test]
fn install_fetches_only_whats_missing() {
    let tmp = tmp_dir("install");
    let home = tmp.join("home");
    let pkg = make_pkg(&tmp, "@contrib/gadgets", "2.1.0");
    let url = spawn_mock(vec![pkg], None);
    let proj = tmp.join("proj");
    make_project(&proj, "\"@contrib/gadgets\" = \"2.1.0\"\n");

    let (ok, err) = run(&url, &home, &["install", proj.to_str().unwrap()]);
    assert!(ok, "{err}");
    assert!(err.contains("1 fetched"), "{err}");
    assert!(proj.join("cohdl.lock").is_file());

    let (ok, err) = run(&url, &home, &["install", proj.to_str().unwrap()]);
    assert!(ok, "{err}");
    assert!(
        err.contains("0 fetched"),
        "second install is a no-op: {err}"
    );
}

#[test]
fn update_bumps_to_latest_published() {
    let tmp = tmp_dir("update");
    let home = tmp.join("home");
    let p1 = make_pkg(&tmp, "@contrib/bumpy", "1.0.0");
    let p2 = make_pkg(&tmp, "@contrib/bumpy", "1.1.0");
    let url = spawn_mock(vec![p1, p2], None);
    let proj = tmp.join("proj");
    make_project(&proj, "\"@contrib/bumpy\" = \"1.0.0\"\n");

    let (ok, err) = run(&url, &home, &["install", proj.to_str().unwrap()]);
    assert!(ok, "{err}");
    let (ok, err) = run(&url, &home, &["update", proj.to_str().unwrap()]);
    assert!(ok, "{err}");
    assert!(err.contains("1.0.0 -> 1.1.0"), "{err}");
    let manifest = std::fs::read_to_string(proj.join("cohdl.toml")).unwrap();
    assert!(
        manifest.contains("\"@contrib/bumpy\" = \"1.1.0\""),
        "{manifest}"
    );
    let lock = std::fs::read_to_string(proj.join("cohdl.lock")).unwrap();
    assert!(lock.contains("version = \"1.1.0\""), "{lock}");
}

#[test]
fn remove_prunes_manifest_and_lock_and_flags_absent() {
    let tmp = tmp_dir("remove");
    let home = tmp.join("home");
    let pkg = make_pkg(&tmp, "@contrib/gone", "1.0.0");
    let url = spawn_mock(vec![pkg], None);
    let proj = tmp.join("proj");
    make_project(&proj, "");

    let (ok, err) = run(
        &url,
        &home,
        &["add", "@contrib/gone", proj.to_str().unwrap()],
    );
    assert!(ok, "{err}");
    let (ok, err) = run(
        &url,
        &home,
        &["remove", "@contrib/gone", proj.to_str().unwrap()],
    );
    assert!(ok, "{err}");
    let manifest = std::fs::read_to_string(proj.join("cohdl.toml")).unwrap();
    assert!(!manifest.contains("gone"), "{manifest}");
    let lock = std::fs::read_to_string(proj.join("cohdl.lock")).unwrap();
    assert!(!lock.contains("gone"), "{lock}");

    let (ok, err) = run(
        &url,
        &home,
        &["remove", "@contrib/gone", proj.to_str().unwrap()],
    );
    assert!(!ok);
    assert!(err.contains("E1205"), "{err}");
}

#[test]
fn missing_package_and_unreachable_registry_are_distinct() {
    let tmp = tmp_dir("missing");
    let home = tmp.join("home");
    let url = spawn_mock(Vec::new(), None);
    let proj = tmp.join("proj");
    make_project(&proj, "");

    let (ok, err) = run(
        &url,
        &home,
        &["add", "@contrib/nope", proj.to_str().unwrap()],
    );
    assert!(!ok);
    assert!(err.contains("E1203"), "not published: {err}");

    // A dead port: unreachable, not "not found".
    let (ok, err) = run(
        "http://127.0.0.1:9",
        &home,
        &["add", "@contrib/nope", proj.to_str().unwrap()],
    );
    assert!(!ok);
    assert!(err.contains("E1204"), "unreachable: {err}");
    assert!(
        !err.contains("error[E1103]"),
        "never conflated with hash mismatch: {err}"
    );
}

#[test]
fn publish_needs_login_and_reports_server_hash() {
    let tmp = tmp_dir("publish");
    let home = tmp.join("home");
    let url = spawn_mock(Vec::new(), None);
    let pkg_dir = tmp.join("mylib");
    std::fs::create_dir_all(pkg_dir.join("src")).unwrap();
    std::fs::write(
        pkg_dir.join("cohdl.toml"),
        "[package]\nname = \"@contrib/mylib\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(
        pkg_dir.join("src/lib.cohdl"),
        "pub device M { pins { A: 1 [passive] } }\n",
    )
    .unwrap();

    // Without login: E1201.
    let (ok, err) = run(&url, &home, &["publish", pkg_dir.to_str().unwrap()]);
    assert!(!ok);
    assert!(err.contains("E1201"), "{err}");
    assert!(err.contains("cohdl login"), "{err}");

    // Store the token the mock accepts, then publish: the mock returns a
    // deliberately different hash — the E1206 warning must surface it.
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("credentials.toml"), "token = \"tok123\"\n").unwrap();
    let (ok, err) = run(&url, &home, &["publish", pkg_dir.to_str().unwrap()]);
    assert!(ok, "{err}");
    assert!(err.contains("published @contrib/mylib 0.1.0"), "{err}");
    assert!(err.contains("E1206"), "hash disagreement warned: {err}");
    assert!(err.contains("sha256:feedface"), "server hash shown: {err}");
}

#[test]
fn publish_namespace_rejection_is_surfaced() {
    let tmp = tmp_dir("ns");
    let home = tmp.join("home");
    let url = spawn_mock(
        Vec::new(),
        Some((
            403,
            r#"{"error":"bare names are reserved for CoHDL's official account"}"#.to_string(),
        )),
    );
    let pkg_dir = tmp.join("std2");
    std::fs::create_dir_all(pkg_dir.join("src")).unwrap();
    std::fs::write(
        pkg_dir.join("cohdl.toml"),
        "[package]\nname = \"stdlike\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(
        pkg_dir.join("src/lib.cohdl"),
        "pub device S { pins { A: 1 [passive] } }\n",
    )
    .unwrap();
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("credentials.toml"), "token = \"tok123\"\n").unwrap();

    let (ok, err) = run(&url, &home, &["publish", pkg_dir.to_str().unwrap()]);
    assert!(!ok);
    assert!(err.contains("E1202"), "{err}");
    assert!(
        err.contains("reserved for CoHDL"),
        "server reason relayed: {err}"
    );
}

// ---------------------------------------------------------------------------
// Library-level: grammar + archive round-trip
// ---------------------------------------------------------------------------

#[test]
fn three_tier_name_grammar() {
    use cohdl::registry::{name_tier, Tier};
    assert_eq!(name_tier("std").unwrap(), Tier::Official);
    assert_eq!(name_tier("sensors").unwrap(), Tier::Official);
    assert_eq!(
        name_tier("@sparkfun/power").unwrap(),
        Tier::Brand("sparkfun".to_string())
    );
    assert_eq!(name_tier("@contrib/widgets").unwrap(), Tier::Contrib);
    for bad in ["@nope", "@a/b/c", "@/x", "a b", "@contrib/"] {
        assert!(name_tier(bad).is_err(), "`{bad}` must be rejected");
    }
}

#[test]
fn name_version_split_respects_scopes() {
    use cohdl::registry::split_name_version;
    assert_eq!(split_name_version("widgets"), ("widgets".into(), None));
    assert_eq!(
        split_name_version("widgets@1.0.0"),
        ("widgets".into(), Some("1.0.0".into()))
    );
    assert_eq!(
        split_name_version("@sparkfun/power"),
        ("@sparkfun/power".into(), None)
    );
    assert_eq!(
        split_name_version("@sparkfun/power@1.0.0"),
        ("@sparkfun/power".into(), Some("1.0.0".into()))
    );
}

#[test]
fn tar_round_trip_preserves_content_hash() {
    let tmp = tmp_dir("tar");
    let src = tmp.join("pkg");
    std::fs::create_dir_all(src.join("src/nested")).unwrap();
    std::fs::write(
        src.join("cohdl.toml"),
        "[package]\nname = \"x\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();
    std::fs::write(
        src.join("src/lib.cohdl"),
        "pub device D { pins { A: 1 [passive] } }\n",
    )
    .unwrap();
    std::fs::write(src.join("src/nested/more.cohdl"), "pub footprint F {}\n").unwrap();
    let tar = cohdl::registry::pack_tar(&src).unwrap();
    let out = tmp.join("out");
    std::fs::create_dir_all(&out).unwrap();
    cohdl::registry::unpack_tar(&tar, &out).unwrap();
    assert_eq!(
        cohdl::hash::package_content_hash(&src).unwrap(),
        cohdl::hash::package_content_hash(&out).unwrap(),
        "pack→unpack is content-identical"
    );
    // Determinism: packing twice is byte-identical.
    assert_eq!(tar, cohdl::registry::pack_tar(&src).unwrap());
}

#[test]
fn unpack_rejects_path_traversal() {
    let tmp = tmp_dir("traverse");
    // A handcrafted tar whose entry name escapes the target.
    let mut evil = vec![0u8; 512];
    let name = b"../evil.txt";
    evil[..name.len()].copy_from_slice(name);
    evil[124..135].copy_from_slice(b"00000000004");
    evil[156] = b'0';
    evil[148..156].copy_from_slice(b"        ");
    let sum: u32 = evil.iter().map(|b| *b as u32).sum();
    evil[148..156].copy_from_slice(format!("{:06o}\0 ", sum).as_bytes());
    evil.extend_from_slice(b"boom");
    evil.extend(std::iter::repeat_n(0u8, 508));
    evil.extend(std::iter::repeat_n(0u8, 1024));
    let err = cohdl::registry::unpack_tar(&evil, &tmp).unwrap_err();
    assert!(err.contains("escapes"), "{err}");
}
