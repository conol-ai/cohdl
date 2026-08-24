//! End-to-end `cohdl self-update` (no RFC — distribution tooling; see
//! docs/compliance-report.md): a std-only local HTTP server plays GitHub —
//! serving a release list, a `.tar.gz` asset, and its sha256sums — and a
//! COPY of the real binary must discover the release, verify the hash, and
//! atomically replace itself with the archive's payload. No network, no
//! external test dependencies; the archive is packed with the system `tar`,
//! the same tool self-update unpacks with.
#![cfg(unix)]

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Serve `routes` (path -> body) on an ephemeral local port, forever; the
/// thread dies with the test process. Returns the base URL.
fn serve(routes: HashMap<String, Vec<u8>>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            let Ok(mut conn) = conn else { continue };
            // Read the request head (curl sends no body for GET).
            let mut head = Vec::new();
            let mut byte = [0u8; 1];
            while !head.ends_with(b"\r\n\r\n") {
                match conn.read(&mut byte) {
                    Ok(1) => head.push(byte[0]),
                    _ => break,
                }
            }
            let head = String::from_utf8_lossy(&head);
            let path = head.split_whitespace().nth(1).unwrap_or("").to_string();
            let response = match routes.get(&path) {
                Some(body) => {
                    let mut r = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    )
                    .into_bytes();
                    r.extend_from_slice(body);
                    r
                }
                None => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .to_vec(),
            };
            let _ = conn.write_all(&response);
        }
    });
    format!("http://{addr}")
}

/// A fresh scratch dir per test (temp_dir may be shared across runs).
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "cohdl-self-update-test-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Copy the real compiled binary somewhere disposable and report its release
/// target triple (printed by `--version` in parentheses).
fn binary_under_test(dir: &Path) -> (PathBuf, String) {
    let real = PathBuf::from(env!("CARGO_BIN_EXE_cohdl"));
    let copy = dir.join("cohdl-under-test");
    std::fs::copy(&real, &copy).expect("copy binary");
    // GitHub's Linux runners can briefly return ETXTBSY when the freshly
    // copied executable is spawned immediately. Retry only that transient
    // kernel error; every other spawn failure remains an immediate failure.
    let out = (0..20)
        .find_map(
            |attempt| match Command::new(&copy).arg("--version").output() {
                Ok(out) => Some(out),
                Err(error)
                    if error.kind() == std::io::ErrorKind::ExecutableFileBusy && attempt < 19 =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    None
                }
                Err(error) => panic!("run --version: {error}"),
            },
        )
        .expect("ETXTBSY persisted while running --version");
    assert!(out.status.success(), "--version failed");
    let text = String::from_utf8_lossy(&out.stdout);
    let target = text
        .split('(')
        .nth(1)
        .and_then(|t| t.split(')').next())
        .expect("--version prints the target triple")
        .to_string();
    (copy, target)
}

/// Pack `payload` as the single `cohdl` entry of a `.tar.gz`, with the
/// system tar (what self-update unpacks with).
fn payload_archive(dir: &Path, payload: &[u8]) -> Vec<u8> {
    let stage = dir.join("stage");
    std::fs::create_dir_all(&stage).expect("stage dir");
    std::fs::write(stage.join("cohdl"), payload).expect("payload");
    let archive = dir.join("payload.tar.gz");
    let status = Command::new("tar")
        .arg("-C")
        .arg(&stage)
        .arg("-czf")
        .arg(&archive)
        .arg("cohdl")
        .status()
        .expect("run tar");
    assert!(status.success(), "tar pack failed");
    std::fs::read(&archive).expect("read archive")
}

fn run_self_update(copy: &Path, base: &str, extra: &[&str]) -> std::process::Output {
    let mut cmd = Command::new(copy);
    cmd.arg("self-update")
        .args(extra)
        .env("COHDL_SELF_UPDATE_API", format!("{base}/releases"))
        .env("COHDL_SELF_UPDATE_DOWNLOAD", format!("{base}/dl"));
    cmd.output().expect("run self-update")
}

/// The two paginated release-list routes the client walks: page 1 carries
/// the JSON, page 2 is the empty page that stops pagination.
fn release_routes(json: &[u8]) -> [(String, Vec<u8>); 2] {
    [
        ("/releases?page=1".to_string(), json.to_vec()),
        ("/releases?page=2".to_string(), b"[]".to_vec()),
    ]
}

#[test]
fn self_update_replaces_the_running_binary() {
    let dir = scratch("replace");
    let (copy, target) = binary_under_test(&dir);

    let payload = b"FAKE-NEW-COHDL-BINARY\n".to_vec();
    let archive = payload_archive(&dir, &payload);
    let asset = format!("cohdl-v99.0.0-{target}.tar.gz");
    let sums = format!("{}  {asset}\n", cohdl::hash::sha256_hex(&archive));
    let mut routes = HashMap::from([
        (format!("/dl/v99.0.0/{asset}"), archive),
        ("/dl/v99.0.0/sha256sums.txt".to_string(), sums.into_bytes()),
    ]);
    // The vscode-v* tag ahead of the compiler tag mirrors the real
    // repository (extension releases share it) and must be skipped.
    routes.extend(release_routes(
        br#"[{"tag_name": "vscode-v100.0.0"}, {"tag_name": "v99.0.0"}]"#,
    ));
    let base = serve(routes);

    let out = run_self_update(&copy, &base, &[]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "self-update failed: {stderr}");
    assert!(
        stderr.contains("updated cohdl") && stderr.contains("99.0.0"),
        "unexpected output: {stderr}"
    );
    let installed = std::fs::read(&copy).expect("read replaced binary");
    assert_eq!(
        installed, payload,
        "the binary was not replaced by the payload"
    );
    // The staging workdir next to the binary must be gone after success.
    let leftovers: Vec<_> = std::fs::read_dir(&dir)
        .expect("read dir")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with(".cohdl-update-")
        })
        .collect();
    assert!(leftovers.is_empty(), "workdir residue: {leftovers:?}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn self_update_check_reports_without_installing() {
    let dir = scratch("check");
    let (copy, _) = binary_under_test(&dir);
    let before = std::fs::read(&copy).expect("read");

    let base = serve(HashMap::from(release_routes(
        br#"[{"tag_name": "v99.0.0"}]"#,
    )));

    let out = run_self_update(&copy, &base, &["--check"]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "--check failed: {stderr}");
    assert!(
        stderr.contains("v99.0.0 is available"),
        "unexpected: {stderr}"
    );
    assert_eq!(
        std::fs::read(&copy).expect("read"),
        before,
        "--check must not modify"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn self_update_refuses_a_corrupted_download() {
    let dir = scratch("corrupt");
    let (copy, target) = binary_under_test(&dir);
    let before = std::fs::read(&copy).expect("read");

    let archive = payload_archive(&dir, b"EVIL");
    let asset = format!("cohdl-v99.0.0-{target}.tar.gz");
    // sha256sums.txt declares a hash the served archive does not match.
    let sums = format!("{:0>64}  {asset}\n", "deadbeef");

    let mut routes = HashMap::from([
        (format!("/dl/v99.0.0/{asset}"), archive),
        ("/dl/v99.0.0/sha256sums.txt".to_string(), sums.into_bytes()),
    ]);
    routes.extend(release_routes(br#"[{"tag_name": "v99.0.0"}]"#));
    let base = serve(routes);

    let out = run_self_update(&copy, &base, &[]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "a hash mismatch must fail");
    assert!(
        stderr.contains("refusing to install"),
        "unexpected: {stderr}"
    );
    assert_eq!(
        std::fs::read(&copy).expect("read"),
        before,
        "must not install"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn self_update_up_to_date_is_a_no_op() {
    let dir = scratch("uptodate");
    let (copy, _) = binary_under_test(&dir);
    let before = std::fs::read(&copy).expect("read");

    // Only an older release exists (the crate version is at least 0.1.0).
    let base = serve(HashMap::from(release_routes(
        br#"[{"tag_name": "v0.0.1"}]"#,
    )));

    let out = run_self_update(&copy, &base, &[]);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "up-to-date failed: {stderr}");
    assert!(stderr.contains("is up to date"), "unexpected: {stderr}");
    assert_eq!(std::fs::read(&copy).expect("read"), before);

    let _ = std::fs::remove_dir_all(&dir);
}
