//! RFC-030: the registry client — CLI verbs (login/publish/add/remove/
//! install/update) exercised against a hand-rolled mock HTTP registry, plus
//! the three-tier name grammar and the tar round-trip. Each test isolates
//! its own COHDL_HOME (cache + credentials) and COHDL_REGISTRY.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver};

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

/// One-request registry for `cohdl search`. Returning the request target over
/// a channel lets the test prove curl encoded the query rather than merely
/// proving the mock was permissive enough to answer it.
fn spawn_search_mock(status: u16, body: &str) -> (String, Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let body = body.as_bytes().to_vec();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let Some(Ok(mut stream)) = listener.incoming().next() else {
            return;
        };
        let mut request = Vec::new();
        let mut chunk = [0u8; 2048];
        loop {
            let n = stream.read(&mut chunk).unwrap_or(0);
            if n == 0 {
                return;
            }
            request.extend_from_slice(&chunk[..n]);
            if request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                break;
            }
        }
        let head = String::from_utf8_lossy(&request);
        let target = head
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or_default()
            .to_string();
        let _ = tx.send(target);
        let _ = stream.write_all(
            format!(
                "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .as_bytes(),
        );
        let _ = stream.write_all(&body);
    });
    (url, rx)
}

struct CapturedRequest {
    method: String,
    target: String,
    headers: std::collections::BTreeMap<String, String>,
    body: Vec<u8>,
}

/// One-request registry that captures an upload only after curl has sent the
/// complete declared body. Header names are normalized because HTTP field
/// names are case-insensitive; values and body bytes are left untouched.
fn spawn_upload_capture_mock() -> (String, Receiver<CapturedRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let Some(Ok(mut stream)) = listener.incoming().next() else {
            return;
        };
        let mut request = Vec::new();
        let mut chunk = [0u8; 4096];
        let header_end = loop {
            let n = stream.read(&mut chunk).unwrap_or(0);
            if n == 0 {
                return;
            }
            request.extend_from_slice(&chunk[..n]);
            if let Some(pos) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n") {
                break pos + 4;
            }
        };
        let head = String::from_utf8(request[..header_end].to_vec()).unwrap();
        let mut lines = head.split("\r\n");
        let request_line = lines.next().unwrap_or_default();
        let mut request_parts = request_line.split_whitespace();
        let method = request_parts.next().unwrap_or_default().to_string();
        let target = request_parts.next().unwrap_or_default().to_string();
        let mut headers = std::collections::BTreeMap::new();
        for line in lines.filter(|line| !line.is_empty()) {
            if let Some((name, value)) = line.split_once(':') {
                headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_string());
            }
        }
        if headers
            .get("expect")
            .is_some_and(|value| value.eq_ignore_ascii_case("100-continue"))
        {
            stream.write_all(b"HTTP/1.1 100 Continue\r\n\r\n").unwrap();
        }
        let content_length: usize = headers
            .get("content-length")
            .expect("curl must send a fixed content length")
            .parse()
            .unwrap();
        let mut body = request[header_end..].to_vec();
        while body.len() < content_length {
            let n = stream.read(&mut chunk).unwrap_or(0);
            if n == 0 {
                break;
            }
            body.extend_from_slice(&chunk[..n]);
        }
        body.truncate(content_length);
        tx.send(CapturedRequest {
            method,
            target,
            headers,
            body,
        })
        .unwrap();
        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
            )
            .unwrap();
    });
    (url, rx)
}

/// Independent JSON-whitespace projection for the transport assertion. This
/// deliberately does not call the production compactor under test.
fn compact_json_for_assertion(pretty: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(pretty.len());
    let mut in_string = false;
    let mut escaped = false;
    for &byte in pretty {
        if in_string {
            out.push(byte);
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else if byte == b'"' {
            in_string = true;
            out.push(byte);
        } else if !byte.is_ascii_whitespace() {
            out.push(byte);
        }
    }
    assert!(
        !in_string,
        "the generated docs JSON must end outside a string"
    );
    out
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

fn run_output(url: &str, home: &Path, args: &[&str]) -> std::process::Output {
    cohdl()
        .env("COHDL_REGISTRY", url)
        .env("COHDL_HOME", home)
        .args(args)
        .output()
        .unwrap()
}

// ---------------------------------------------------------------------------

#[test]
fn search_finds_packages_and_parts_and_url_encodes_the_query() {
    let tmp = tmp_dir("search_human");
    let response = r#"{
      "query":"stm32 f0&usb",
      "packages":{"results":[{
        "name":"@st/stm32","tier":"brand","latest":"1.4.0",
        "description":"Fast \u001b[31mred\u061c\n MCU package","updated":"2026-08-24T10:00:00Z"
      }],"has_more":true},
      "parts":{"results":[{
        "package":"@st/stm32","tier":"brand","version":"1.4.0",
        "fq":"st::stm32::STM32F072CBT6","name":"STM32F072CBT6",
        "device":"st::stm32::STM32F072","intent":"USB \u001b]8;;bad\u0007\u200fcontroller",
        "manufacturer":"STMicroelectronics","mpn":"STM32F072CBT6","primary":false
      }],"has_more":false}
    }"#;
    let (url, request) = spawn_search_mock(200, response);
    let out = run_output(&url, &tmp.join("home"), &["search", "stm32 f0&usb"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.stderr.is_empty(), "search results do not use stderr");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Packages\n"), "{stdout}");
    assert!(
        stdout.contains("@st/stm32@1.4.0 [Verified manufacturer]"),
        "{stdout}"
    );
    assert!(stdout.contains("Fast red MCU package"), "{stdout}");
    assert!(stdout.contains("Parts\n"), "{stdout}");
    assert!(
        stdout.contains("STM32F072CBT6 — STMicroelectronics · STM32F072CBT6 · alternate"),
        "{stdout}"
    );
    assert!(stdout.contains("USB controller"), "{stdout}");
    assert!(
        stdout.contains("More package matches are available"),
        "{stdout}"
    );
    assert!(!stdout.contains('\u{1b}'), "terminal ESC must be removed");
    assert!(
        !stdout.contains('\u{061c}'),
        "Arabic bidi control must be removed"
    );
    assert!(
        !stdout.contains('\u{200f}'),
        "direction mark must be removed"
    );

    let target = request.recv().unwrap();
    assert_eq!(
        target, "/search?q=stm32+f0%26usb",
        "query syntax must be URL-encoded as data, not concatenated"
    );
}

#[test]
fn search_url_encoding_preserves_a_unicode_query() {
    let tmp = tmp_dir("search_unicode_query");
    let body = r#"{"query":"电源 usb&c","packages":{"results":[],"has_more":false},"parts":{"results":[],"has_more":false}}"#;
    let (url, request) = spawn_search_mock(200, body);
    let out = run_output(&url, &tmp.join("home"), &["search", "电源 usb&c"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        request.recv().unwrap().to_ascii_lowercase(),
        "/search?q=%e7%94%b5%e6%ba%90+usb%26c"
    );
}

#[test]
fn search_normalizes_unicode_whitespace_and_bom_before_the_request() {
    let tmp = tmp_dir("search_trim_query");
    let body = r#"{"query":"stm32","packages":{"results":[],"has_more":false},"parts":{"results":[],"has_more":false}}"#;
    let (url, request) = spawn_search_mock(200, body);
    let out = run_output(
        &url,
        &tmp.join("home"),
        &["search", "\u{feff}\u{0085}stm32\u{2003}\u{feff}"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(request.recv().unwrap(), "/search?q=stm32");
}

#[test]
fn search_json_is_deterministic_and_preserves_nullable_fields() {
    let tmp = tmp_dir("search_json");
    let response = r#"{"parts":{"has_more":true,"results":[{"primary":false,"mpn":null,"manufacturer":"Acme","intent":null,"device":"parts::D","name":"P","fq":"parts::P","version":"2.0.0","tier":"contrib","package":"@contrib/parts"}]},"packages":{"has_more":false,"results":[{"updated":"now","description":"Rocket \ud83d\ude80","latest":"2.0.0","tier":"contrib","name":"@contrib/parts"}]},"query":"emoji parts"}"#;
    let (url, _) = spawn_search_mock(200, response);
    let out = run_output(
        &url,
        &tmp.join("home"),
        &["search", "emoji parts", "--json"],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.stderr.is_empty());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let expected = r#"{
  "query": "emoji parts",
  "packages": {
    "results": [
      {
        "name": "@contrib/parts",
        "tier": "contrib",
        "latest": "2.0.0",
        "description": "Rocket 🚀",
        "updated": "now"
      }
    ],
    "has_more": false
  },
  "parts": {
    "results": [
      {
        "package": "@contrib/parts",
        "tier": "contrib",
        "version": "2.0.0",
        "fq": "parts::P",
        "name": "P",
        "device": "parts::D",
        "intent": null,
        "manufacturer": "Acme",
        "mpn": null,
        "primary": false
      }
    ],
    "has_more": true
  }
}
"#;
    assert_eq!(stdout, expected, "stable key order and formatting");
}

#[test]
fn empty_search_is_a_success_in_human_and_json_modes() {
    let tmp = tmp_dir("search_empty");
    let body = r#"{"query":"no matches","packages":{"results":[],"has_more":false},"parts":{"results":[],"has_more":false}}"#;

    let (url, _) = spawn_search_mock(200, body);
    let out = run_output(&url, &tmp.join("home1"), &["search", "no matches"]);
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "No packages or parts matched `no matches`.\n"
    );
    assert!(out.stderr.is_empty());

    let (url, _) = spawn_search_mock(200, body);
    let out = run_output(
        &url,
        &tmp.join("home2"),
        &["search", "no matches", "--json"],
    );
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("\"results\": []"), "{stdout}");
    assert!(stdout.contains("\"has_more\": false"), "{stdout}");
    assert!(!stdout.contains("No packages"), "JSON contains no prose");
    assert!(out.stderr.is_empty());
}

#[test]
fn malformed_search_response_is_e1204_in_human_and_json_modes() {
    let tmp = tmp_dir("search_malformed");
    // Missing the mandatory `parts` section: a 200 is not enough to trust a
    // response as protocol-valid.
    let body = r#"{"query":"bad response","packages":{"results":[],"has_more":false}}"#;
    let (url, _) = spawn_search_mock(200, body);
    let out = run_output(&url, &tmp.join("home1"), &["search", "bad response"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("E1204"), "{stderr}");
    assert!(stderr.contains("malformed response"), "{stderr}");

    let (url, _) = spawn_search_mock(200, body);
    let out = run_output(
        &url,
        &tmp.join("home2"),
        &["search", "bad response", "--json"],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stderr.is_empty(), "JSON failure remains one stdout doc");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("\"verdict\": \"fail\""), "{stdout}");
    assert!(stdout.contains("\"code\": \"E1204\""), "{stdout}");
}

#[test]
fn search_decoder_rejects_ambiguous_or_wrong_typed_json() {
    let tmp = tmp_dir("search_strict_json");
    let cases = [
        (
            r#"{"query":"bad json","query":"bad json","packages":{"results":[],"has_more":false},"parts":{"results":[],"has_more":false}}"#,
            "duplicate object key",
        ),
        (
            r#"{"query":"bad json","packages":{"results":[{"name":"x","tier":"official","latest":"1.0.0","description":"\ud800","updated":"now"}],"has_more":false},"parts":{"results":[],"has_more":false}}"#,
            "high surrogate",
        ),
        (
            r#"{"query":"bad json","packages":{"results":[],"has_more":false},"parts":{"results":[{"package":"parts","tier":"official","version":"1.0.0","fq":"parts::P","name":"P","device":"parts::D","intent":null,"manufacturer":null,"mpn":null,"primary":null}],"has_more":false}}"#,
            "must be a boolean",
        ),
        (
            r#"{"query":"bad json","packages":{"results":[{"name":"parts","tier":"official","latest":"1.0.0","updated":"now"}],"has_more":false},"parts":{"results":[],"has_more":false}}"#,
            "missing `description`",
        ),
        (
            r#"{"query":"bad json","packages":{"results":[],"has_more":false},"parts":{"results":[{"package":"parts","tier":"official","version":"1.0.0","fq":"parts::P","name":"P","device":"parts::D","intent":null,"manufacturer":null,"primary":true}],"has_more":false}}"#,
            "missing `mpn`",
        ),
        (
            r#"{"query":"bad json","packages":{"results":[{"name":"parts","tier":"trusted","latest":"1.0.0","description":null,"updated":"now"}],"has_more":false},"parts":{"results":[],"has_more":false}}"#,
            "must be official, brand, or contrib",
        ),
        (
            r#"{"query":"different","packages":{"results":[],"has_more":false},"parts":{"results":[],"has_more":false}}"#,
            "does not match requested query",
        ),
    ];
    for (index, (body, needle)) in cases.iter().enumerate() {
        let (url, _) = spawn_search_mock(200, body);
        let out = run_output(
            &url,
            &tmp.join(format!("home{index}")),
            &["search", "bad json"],
        );
        assert_eq!(out.status.code(), Some(1), "case {index}");
        assert!(out.stdout.is_empty(), "case {index}");
        let stderr = String::from_utf8(out.stderr).unwrap();
        assert!(stderr.contains("E1204"), "case {index}: {stderr}");
        assert!(stderr.contains(needle), "case {index}: {stderr}");
    }
}

#[test]
fn search_decoder_enforces_default_result_and_transport_bounds() {
    let tmp = tmp_dir("search_response_bounds");
    let row =
        r#"{"name":"p","tier":"official","latest":"1.0.0","description":null,"updated":"now"}"#;
    let body = format!(
        "{{\"query\":\"too many\",\"packages\":{{\"results\":[{}],\"has_more\":true}},\"parts\":{{\"results\":[],\"has_more\":false}}}}",
        std::iter::repeat_n(row, 21).collect::<Vec<_>>().join(",")
    );
    let (url, _) = spawn_search_mock(200, &body);
    let out = run_output(&url, &tmp.join("home1"), &["search", "too many"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("exceeds the default limit of 20"));

    let oversized = " ".repeat(1024 * 1024 + 1);
    let (url, _) = spawn_search_mock(200, &oversized);
    let out = run_output(&url, &tmp.join("home2"), &["search", "too large"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("E1204"));
}

#[test]
fn unreachable_search_registry_is_e1204() {
    let tmp = tmp_dir("search_unreachable");
    let out = run_output(
        "http://127.0.0.1:9",
        &tmp.join("home"),
        &["search", "dead registry"],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("E1204"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("retry the search"), "{stderr}");
    assert!(
        !stderr.contains("vendor the package"),
        "search must not suggest a project dependency workaround: {stderr}"
    );
}

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

/// Like `make_pkg`, with a `[dependencies]` section of its own — the
/// transitive-closure fixtures (RFC-029 amendment, 2026-08-25).
fn make_pkg_with_deps(stage: &Path, name: &str, version: &str, deps: &str) -> MockPkg {
    let dir = stage.join(format!("stage-{}-{}", name.replace('/', "_"), version));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("cohdl.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"{version}\"\n\n[dependencies]\n{deps}"),
    )
    .unwrap();
    std::fs::write(
        dir.join("src/lib.cohdl"),
        "pub device T { pins { A: 1 [passive], B: 2 [passive] } }\n",
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

#[test]
fn install_fetches_the_transitive_closure() {
    let tmp = tmp_dir("install-transitive");
    let home = tmp.join("home");
    let sub = make_pkg(&tmp, "@contrib/subgadget", "1.0.0");
    let top = make_pkg_with_deps(
        &tmp,
        "@contrib/gadgets",
        "2.1.0",
        "\"@contrib/subgadget\" = \"1.0.0\"\n",
    );
    let url = spawn_mock(vec![top, sub], None);
    let proj = tmp.join("proj");
    make_project(&proj, "\"@contrib/gadgets\" = \"2.1.0\"\n");

    let (ok, err) = run(&url, &home, &["install", proj.to_str().unwrap()]);
    assert!(ok, "{err}");
    assert!(err.contains("2 fetched"), "direct + transitive: {err}");
    let lock = std::fs::read_to_string(proj.join("cohdl.lock")).unwrap();
    assert!(
        lock.contains("name = \"@contrib/subgadget\""),
        "the transitive dependency is locked: {lock}"
    );
}

#[test]
fn add_fetches_the_transitive_closure_into_the_cache() {
    let tmp = tmp_dir("add-transitive");
    let home = tmp.join("home");
    let sub = make_pkg(&tmp, "@contrib/subgadget", "1.0.0");
    let top = make_pkg_with_deps(
        &tmp,
        "@contrib/gadgets",
        "2.1.0",
        "\"@contrib/subgadget\" = \"1.0.0\"\n",
    );
    let url = spawn_mock(vec![top, sub], None);
    let proj = tmp.join("proj");
    make_project(&proj, "");

    let (ok, err) = run(
        &url,
        &home,
        &["add", "@contrib/gadgets@2.1.0", proj.to_str().unwrap()],
    );
    assert!(ok, "{err}");
    assert!(
        err.contains("fetched @contrib/subgadget 1.0.0"),
        "the added package's own dependency is fetched too: {err}"
    );
    assert!(
        home.join("registry/@contrib/subgadget/1.0.0/cohdl.toml")
            .is_file(),
        "the transitive package lands in the cache"
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
        "[package]\nname = \"@contrib/mylib\"\nversion = \"0.1.0\"\nlicense = \"MIT\"\n",
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
fn docs_publish_sends_compact_length_delimited_authenticated_sidecar() {
    let tmp = tmp_dir("docs_publish_transport");
    let home = tmp.join("home");
    let pkg_dir = tmp.join("docs-upload");
    std::fs::create_dir_all(pkg_dir.join("src")).unwrap();
    std::fs::write(
        pkg_dir.join("cohdl.toml"),
        "[package]\nname = \"@contrib/docs-upload\"\nversion = \"0.1.0\"\nlicense = \"MIT\"\n\n[dependencies]\n",
    )
    .unwrap();
    std::fs::write(
        pkg_dir.join("src/lib.cohdl"),
        "#[intent(\"two  spaces stay\")]\npub device Tiny { pins { A: 1 [passive] } }\n",
    )
    .unwrap();
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("credentials.toml"), "token = \"tok123\"\n").unwrap();
    let std_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("lib/std");
    let pretty_path = tmp.join("api.json");
    let (url, captured) = spawn_upload_capture_mock();
    let uploaded = cohdl()
        .env("COHDL_REGISTRY", &url)
        .env("COHDL_HOME", &home)
        .env("COHDL_STD", &std_dir)
        .args([
            "docs",
            pkg_dir.to_str().unwrap(),
            "--out",
            pretty_path.to_str().unwrap(),
            "--publish",
        ])
        .output()
        .unwrap();
    assert!(
        uploaded.status.success(),
        "{}",
        String::from_utf8_lossy(&uploaded.stderr)
    );

    // --out preserves the human-readable artifact. Compact it independently
    // so this assertion does not compare the production compactor with itself.
    let expected_body = compact_json_for_assertion(&std::fs::read(pretty_path).unwrap());
    assert!(
        expected_body.windows(16).any(|w| w == b"two  spaces stay"),
        "whitespace inside JSON strings must survive compaction"
    );
    assert!(!expected_body.contains(&b'\n'));
    let request = captured
        .recv_timeout(std::time::Duration::from_secs(5))
        .expect("docs upload reaches the loopback registry");
    assert_eq!(request.method, "PUT");
    assert_eq!(request.target, "/packages/@contrib/docs-upload/0.1.0/docs");
    assert_eq!(
        request.headers.get("authorization").map(String::as_str),
        Some("Bearer tok123")
    );
    assert_eq!(
        request
            .headers
            .get("x-cohdl-api-docs-schema")
            .map(String::as_str),
        Some("1")
    );
    assert_eq!(request.body, expected_body, "exact compact JSON bytes");
    let expected_content_length = request.body.len().to_string();
    assert_eq!(
        request.headers.get("content-length").map(String::as_str),
        Some(expected_content_length.as_str()),
        "Content-Length must describe the exact uploaded body"
    );
    let expected_sha256 = cohdl::hash::sha256_hex(&request.body);
    let sent_sha256 = request
        .headers
        .get("x-cohdl-api-docs-sha256")
        .expect("upload checksum header");
    assert_eq!(sent_sha256, &expected_sha256);
    assert_eq!(sent_sha256.len(), 64);
    assert!(
        sent_sha256
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "SHA-256 must be lowercase hexadecimal: {sent_sha256}"
    );
}

#[test]
fn publish_echoes_the_metadata_and_documents_it_sends() {
    let tmp = tmp_dir("publish_meta");
    let home = tmp.join("home");
    // The server answers with the document index it built from the archive.
    let url = spawn_mock(
        Vec::new(),
        Some((
            200,
            r#"{"hash":"sha256:feedface","docs":["README.md","docs/ds.pdf"]}"#.to_string(),
        )),
    );
    let pkg_dir = tmp.join("mylib");
    std::fs::create_dir_all(pkg_dir.join("src")).unwrap();
    std::fs::write(
        pkg_dir.join("cohdl.toml"),
        "[package]\nname = \"@contrib/mylib\"\nversion = \"0.1.0\"\ndescription = \"A tiny library.\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    std::fs::write(
        pkg_dir.join("src/lib.cohdl"),
        "#[doc(\"README.md\")]\npub device M { pins { A: 1 [passive] } }\n",
    )
    .unwrap();
    std::fs::write(pkg_dir.join("README.md"), "# mylib\n").unwrap();
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("credentials.toml"), "token = \"tok123\"\n").unwrap();

    let (ok, err) = run(&url, &home, &["publish", pkg_dir.to_str().unwrap()]);
    assert!(ok, "{err}");
    // Declared metadata is echoed verbatim; an undeclared key says so rather
    // than going out silently empty.
    assert!(err.contains("description: A tiny library."), "{err}");
    assert!(err.contains("license: MIT"), "{err}");
    assert!(
        err.contains("repository: — (no `[package] repository` in the manifest)"),
        "{err}"
    );
    assert!(
        err.contains("documents: README.md, docs/ds.pdf"),
        "server's document index echoed: {err}"
    );
}

#[test]
fn publish_without_a_license_never_reaches_the_network() {
    let tmp = tmp_dir("publish_nolicense");
    let home = tmp.join("home");
    // A port nothing listens on: if the CLI attempted an upload at all, the
    // failure would be the unreachable-registry diagnostic (E1204) instead of
    // the license refusal. That is what proves the check is pre-flight.
    let url = "http://127.0.0.1:1".to_string();
    let pkg_dir = tmp.join("mylib");
    std::fs::create_dir_all(pkg_dir.join("src")).unwrap();
    std::fs::write(
        pkg_dir.join("cohdl.toml"),
        "[package]\nname = \"@contrib/mylib\"\nversion = \"0.1.0\"\ndescription = \"No license here.\"\n",
    )
    .unwrap();
    std::fs::write(
        pkg_dir.join("src/lib.cohdl"),
        "pub device M { pins { A: 1 [passive] } }\n",
    )
    .unwrap();
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("credentials.toml"), "token = \"tok123\"\n").unwrap();

    let (ok, err) = run(&url, &home, &["publish", pkg_dir.to_str().unwrap()]);
    assert!(!ok);
    assert!(err.contains("`[package] license`"), "{err}");
    assert!(err.contains("cohdl.toml"), "names the file to fix: {err}");
    assert!(
        !err.contains("E1204"),
        "refused before any upload attempt: {err}"
    );
    assert!(!err.contains("published"), "{err}");

    // A declared license is all the gate wants — no license list to satisfy.
    std::fs::write(
        pkg_dir.join("cohdl.toml"),
        "[package]\nname = \"@contrib/mylib\"\nversion = \"0.1.0\"\nlicense = \"LicenseRef-Acme-Proprietary\"\n",
    )
    .unwrap();
    let (ok, err) = run(&url, &home, &["publish", pkg_dir.to_str().unwrap()]);
    assert!(!ok, "still fails — but now only because the host is dead");
    assert!(
        err.contains("E1204"),
        "the license gate is passed and the upload was attempted: {err}"
    );
    assert!(
        !err.contains("HTTP 0"),
        "an unreached registry is E1204, not a bare status line: {err}"
    );

    // An empty license is silence with extra steps.
    std::fs::write(
        pkg_dir.join("cohdl.toml"),
        "[package]\nname = \"@contrib/mylib\"\nversion = \"0.1.0\"\nlicense = \"   \"\n",
    )
    .unwrap();
    let (ok, err) = run(&url, &home, &["publish", pkg_dir.to_str().unwrap()]);
    assert!(!ok);
    assert!(err.contains("`[package] license`"), "{err}");
}

#[test]
fn publish_surfaces_a_server_side_manifest_rejection() {
    let tmp = tmp_dir("publish_400");
    let home = tmp.join("home");
    let url = spawn_mock(
        Vec::new(),
        Some((
            400,
            r#"{"error":"the manifest declares `[package] name = \"other\"` but this publishes `@contrib/mylib`"}"#
                .to_string(),
        )),
    );
    let pkg_dir = tmp.join("mylib");
    std::fs::create_dir_all(pkg_dir.join("src")).unwrap();
    std::fs::write(
        pkg_dir.join("cohdl.toml"),
        "[package]\nname = \"@contrib/mylib\"\nversion = \"0.1.0\"\nlicense = \"MIT\"\n",
    )
    .unwrap();
    std::fs::write(
        pkg_dir.join("src/lib.cohdl"),
        "pub device M { pins { A: 1 [passive] } }\n",
    )
    .unwrap();
    std::fs::create_dir_all(&home).unwrap();
    std::fs::write(home.join("credentials.toml"), "token = \"tok123\"\n").unwrap();

    let (ok, err) = run(&url, &home, &["publish", pkg_dir.to_str().unwrap()]);
    assert!(!ok);
    // The server's own message reaches the user — never a bare `HTTP 400`.
    assert!(err.contains("E1202"), "{err}");
    assert!(err.contains("the manifest declares"), "{err}");
    assert!(!err.contains("publish failed: HTTP 400"), "{err}");
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
        "[package]\nname = \"stdlike\"\nversion = \"0.1.0\"\nlicense = \"MIT\"\n",
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
