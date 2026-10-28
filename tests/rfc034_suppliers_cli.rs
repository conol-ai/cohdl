//! RFC-034 §5 online audit against a deterministic loopback registry.

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Reply {
    status: u16,
    body: String,
}

#[derive(Clone, Default)]
struct Mock {
    routes: Arc<Mutex<BTreeMap<String, Reply>>>,
}

impl Mock {
    fn route(&self, prefix: &str, body: String) {
        self.routes
            .lock()
            .unwrap()
            .insert(prefix.to_string(), Reply { status: 200, body });
    }

    fn route_status(&self, prefix: &str, status: u16, body: &str) {
        self.routes.lock().unwrap().insert(
            prefix.to_string(),
            Reply {
                status,
                body: body.to_string(),
            },
        );
    }

    fn start(self) -> (String, Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut bytes = Vec::new();
                let mut buf = [0; 4096];
                let header_end = loop {
                    let count = stream.read(&mut buf).unwrap_or(0);
                    if count == 0 {
                        break None;
                    }
                    bytes.extend_from_slice(&buf[..count]);
                    if let Some(pos) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                        break Some(pos + 4);
                    }
                };
                let Some(header_end) = header_end else {
                    continue;
                };
                let head = String::from_utf8_lossy(&bytes[..header_end]);
                let target = head
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or("")
                    .to_string();
                let _ = tx.send(target.clone());
                let decoded = percent_decode(&target);
                let routes = self.routes.lock().unwrap();
                let (path, query) = decoded.split_once('?').unwrap_or((decoded.as_str(), ""));
                let kind = query
                    .split('&')
                    .next()
                    .and_then(|part| part.split('=').next());
                let exact = |key: &String| {
                    let Some((_, route_query)) = key.split_once('?') else {
                        return false;
                    };
                    route_query != "pn=" && route_query != "mpn="
                };
                let reply = routes
                    .get(&target)
                    .filter(|_| exact(&target))
                    .or_else(|| routes.get(&decoded).filter(|_| exact(&decoded)))
                    .or_else(|| {
                        let generic = format!("{path}?{}", kind.unwrap_or(""));
                        routes
                            .get(&format!("{generic}="))
                            .or_else(|| routes.get(path))
                    })
                    .cloned()
                    .map(|reply| validate_request(&decoded, reply))
                    .unwrap_or(Reply {
                        status: 500,
                        body: "no route".to_string(),
                    });
                let reason = if reply.status == 200 { "OK" } else { "ERROR" };
                write!(
                    stream,
                    "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    reply.status,
                    reason,
                    reply.body.len(),
                    reply.body
                )
                .unwrap();
            }
        });
        (url, rx)
    }
}

fn validate_request(target: &str, reply: Reply) -> Reply {
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let mut params = Vec::new();
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        params.push((key, value));
    }
    let invalid = params
        .iter()
        .any(|(_, value)| value.is_empty() || value.chars().any(|ch| ch.is_control()));
    let over_limit = if path == "/manufacturers/normalize" {
        ["mfr", "mpn"]
            .iter()
            .any(|key| params.iter().filter(|(name, _)| name == key).count() > 100)
            || params.iter().any(|(_, value)| value.len() > 256)
    } else if path.starts_with("/suppliers/") && path.ends_with("/parts") {
        params.len() != 1
            || !matches!(params[0].0, "pn" | "mpn")
            || params[0].1.split(',').count() > 100
            || params[0].1.split(',').any(|value| value.len() > 256)
    } else {
        false
    };
    if invalid || over_limit {
        Reply {
            status: 400,
            body: r#"{"error":"invalid query"}"#.to_string(),
        }
    } else {
        reply
    }
}

fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap();
                out.push(u8::from_str_radix(hex, 16).unwrap());
                index += 3;
            }
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(out).unwrap()
}

fn tmp_dir(tag: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "cohdl-rfc034-audit-{tag}-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn project_design(tag: &str, entries: &str, extra_parts: &str, design: &str) -> PathBuf {
    let dir = project(tag, entries, extra_parts);
    let path = dir.join("src/main.cohdl");
    let source = std::fs::read_to_string(&path).unwrap();
    let source = source.replace("design Board { inst p: P nc: p.A }", design);
    std::fs::write(path, source).unwrap();
    dir
}

fn project(tag: &str, entries: &str, extra_parts: &str) -> PathBuf {
    let dir = tmp_dir(tag);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("cohdl.toml"),
        "[package]\nname = \"audit_fixture\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    let mut marker_fields = Vec::new();
    if entries.contains("mfr: \"\"") {
        marker_fields.push("\"mfr\"");
    }
    if entries.contains("mpn: \"\"") {
        marker_fields.push("\"mpn\"");
    }
    let source_marker = if marker_fields.is_empty() {
        String::new()
    } else {
        format!("#[source_unspecified({})]\n", marker_fields.join(","))
    };
    std::fs::write(
        dir.join("src/main.cohdl"),
        format!(
            r#"
pub device D {{ pins {{ A: 1 [passive] }} }}
pub footprint FP {{}}
{source_marker}pub part P: D {{
{entries}
}}
{extra_parts}
design Board {{ inst p: P nc: p.A }}
"#
        ),
    )
    .unwrap();
    dir
}

fn run(dir: &Path, url: &str, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cohdl"))
        .env("COHDL_REGISTRY", url)
        .args(["suppliers", dir.to_str().unwrap(), "--no-std"])
        .args(extra)
        .output()
        .unwrap()
}

fn json_string(value: &str) -> String {
    serde_json::to_string(value).unwrap()
}

fn catalog(pn: &str, mfr: &str, mpn: &str, stock: u64) -> String {
    catalog_keys(pn, mfr, mpn, stock, mfr, mpn)
}

fn catalog_keys(
    pn: &str,
    mfr: &str,
    mpn: &str,
    stock: u64,
    mfr_key: &str,
    mpn_key: &str,
) -> String {
    format!(
        r#"{{"pn":{},"mpn":{},"mpn_key":{},"manufacturer":{},"manufacturer_key":{},"package":"0402","class":"resistor","stock":{stock},"links":[],"fetched_at":"2026-10-28"}}"#,
        json_string(pn),
        json_string(mpn),
        json_string(mpn_key),
        json_string(mfr),
        json_string(mfr_key)
    )
}

fn pn_response(supplier: &str, rows: &[(&str, Option<String>)]) -> String {
    let rows = rows
        .iter()
        .map(|(pn, catalog)| {
            format!(
                "{{\"pn\":{},\"catalog\":{}}}",
                json_string(pn),
                catalog.clone().unwrap_or_else(|| "\"unknown\"".to_string())
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(r#"{{"supplier":"{supplier}","snapshot_date":"2026-10-28","parts":[{rows}]}}"#)
}

fn mpn_response(supplier: &str, rows: &[(&str, Vec<String>)]) -> String {
    let rows = rows
        .iter()
        .map(|(mpn, rows)| {
            format!(
                "{{\"mpn\":{},\"catalog\":[{}]}}",
                json_string(mpn),
                rows.join(",")
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(r#"{{"supplier":"{supplier}","snapshot_date":"2026-10-28","parts":[{rows}]}}"#)
}

fn normalize(mfr: &[&str], mpn: &[&str], mfr_keys: &[&str], mpn_keys: &[&str]) -> String {
    fn side(name: &str, values: &[&str], keys: &[&str]) -> String {
        let rows = values
            .iter()
            .zip(keys)
            .map(|(value, key)| {
                format!(
                    "{{\"value\":{},\"key\":{}}}",
                    json_string(value),
                    json_string(key)
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!("\"{name}\":[{rows}]")
    }
    format!(
        "{{{},{}}}",
        side("mfr", mfr, mfr_keys),
        side("mpn", mpn, mpn_keys)
    )
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).unwrap()
}

#[test]
fn audit_ok_uses_registry_alias_keys_and_json_is_complete() {
    let dir = project(
        "ok-alias",
        r#"    primary { mfr: "Acme, Inc.", mpn: "R 1", footprint: FP, supplier acme "P-1" }
    alt { mfr: "Other", mpn: "R-ALT", supplier acme "P-ALT" }"#,
        "",
    );
    let mock = Mock::default();
    mock.route(
        "/suppliers/acme/parts",
        pn_response(
            "acme",
            &[
                (
                    "P-1",
                    Some(catalog_keys(
                        "P-1",
                        "Canonical Maker",
                        "R1",
                        2,
                        "canonical",
                        "r1",
                    )),
                ),
                (
                    "P-ALT",
                    Some(catalog_keys("P-ALT", "Other", "R-ALT", 3, "Other", "R-ALT")),
                ),
            ],
        ),
    );
    mock.route(
        "/manufacturers/normalize",
        normalize(
            &["Acme, Inc.", "Other"],
            &["R 1", "R-ALT"],
            &["canonical", "Other"],
            &["r1", "R-ALT"],
        ),
    );
    let (url, requests) = mock.start();
    let output = run(&dir, &url, &["--json"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
    let text = stdout(&output);
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["design"], "Board");
    assert_eq!(json["suppliers"][0]["entries"][0]["status"], "ok");
    assert_eq!(json["suppliers"][0]["entries"][1]["entry"], "alt[0]");
    assert_eq!(
        json["suppliers"][0]["entries"][0]["catalog"]["manufacturer_key"],
        "canonical"
    );
    assert_eq!(
        json["suppliers"][0]["entries"][0]["suggestions"],
        serde_json::json!([])
    );
    let targets: Vec<String> = requests.try_iter().collect();
    assert_eq!(targets.len(), 2);
    assert!(
        targets[0]
            .to_ascii_lowercase()
            .starts_with("/suppliers/acme/parts?pn=p-1%2cp-alt"),
        "{targets:?}"
    );
    assert!(targets[1]
        .to_ascii_lowercase()
        .contains("mfr=acme%2c+inc.&mfr=other&mpn=r+1&mpn=r-alt"));
    assert!(!text.contains("mapping"));
}

#[test]
fn unknown_and_mismatch_exit_three_without_becoming_protocol_errors() {
    for (tag, row, status) in [
        ("unknown", None, "unknown"),
        (
            "mismatch",
            Some(catalog("P-1", "Wrong", "Wrong", 2)),
            "mismatch",
        ),
    ] {
        let dir = project(
            tag,
            r#"    primary { mfr: "M", mpn: "N", footprint: FP, supplier acme "P-1" }"#,
            "",
        );
        let mock = Mock::default();
        mock.route(
            "/suppliers/acme/parts",
            pn_response("acme", &[("P-1", row)]),
        );
        {
            mock.route(
                "/manufacturers/normalize",
                normalize(&["M"], &["N"], &["m"], &["n"]),
            );
        }
        let (url, _) = mock.start();
        let output = run(&dir, &url, &["--json"]);
        assert_eq!(
            output.status.code(),
            Some(3),
            "{}{}",
            stdout(&output),
            stderr(&output)
        );
        let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
        assert_eq!(json["suppliers"][0]["entries"][0]["status"], status);
        assert!(!stdout(&output).contains("E1204"));
    }
}

#[test]
fn empty_source_fields_are_not_sent_or_compared_and_empty_mpn_has_no_suggestions() {
    let dir = project_design(
        "empty",
        r#"    primary { mfr: "", mpn: "", footprint: FP, supplier acme "P-EMPTY" }
    alt { mfr: "Alt", mpn: "N" }"#,
        r#"#[source_unspecified("mpn")]
pub part EMPTY: D { primary { mfr: "Alt", mpn: "", footprint: FP } }"#,
        "design Board { inst p: P inst empty: EMPTY nc: p.A nc: empty.A }",
    );
    let mock = Mock::default();
    mock.route(
        "/suppliers/acme/parts",
        pn_response(
            "acme",
            &[("P-EMPTY", Some(catalog("P-EMPTY", "M", "N", 1)))],
        ),
    );
    mock.route(
        "/suppliers/acme/parts?mpn=N",
        mpn_response("acme", &[("N", Vec::new())]),
    );
    mock.route(
        "/manufacturers/normalize",
        normalize(&["Alt", "Alt"], &["N"], &["alt", "alt"], &["n"]),
    );
    let (url, requests) = mock.start();
    let output = run(&dir, &url, &["--supplier", "acme", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let entries = json["suppliers"][0]["entries"].as_array().unwrap();
    let bound = entries
        .iter()
        .find(|entry| entry["pn"] == "P-EMPTY")
        .unwrap();
    let empty = entries
        .iter()
        .find(|entry| entry["part"] == "audit_fixture::EMPTY")
        .unwrap();
    assert_eq!(bound["status"], "ok");
    assert_eq!(empty["status"], "unbound");
    assert_eq!(empty["suggestions"], serde_json::json!([]));
    assert!(
        entries
            .iter()
            .filter(|entry| entry["status"] == "unbound")
            .count()
            == 2
    );
    let targets: Vec<String> = requests.try_iter().collect();
    assert_eq!(targets.len(), 3, "{targets:?}");
    assert!(targets
        .iter()
        .all(|target| !target.contains("mfr=&") && !target.contains("mpn=&")));
    let normalize_request = targets
        .iter()
        .find(|target| target.starts_with("/manufacturers/normalize?"))
        .unwrap();
    assert!(normalize_request.contains("mfr=Alt"));
    assert!(normalize_request.contains("mpn=N"));
}

#[test]
fn out_of_stock_and_unbound_do_not_change_exit_code_and_suggestions_prefer_mfr() {
    let dir = project(
        "stock-suggest",
        r#"    primary { mfr: "M", mpn: "N", footprint: FP, supplier acme "P-1" }
    alt { mfr: "M", mpn: "ALT-N" }
    alt { mpn: "NO-MFR" }"#,
        "",
    );
    let mock = Mock::default();
    mock.route(
        "/suppliers/acme/parts?pn=",
        pn_response(
            "acme",
            &[("P-1", Some(catalog_keys("P-1", "M", "N", 0, "m", "n")))],
        ),
    );
    mock.route(
        "/suppliers/acme/parts?mpn=",
        mpn_response(
            "acme",
            &[
                (
                    "ALT-N",
                    vec![
                        catalog_keys("S-NO", "Other", "ALT-N", 10, "other", "alt-n"),
                        catalog_keys("S-YES", "M", "ALT-N", 1, "m", "alt-n"),
                    ],
                ),
                (
                    "NO-MFR",
                    vec![catalog_keys("S-EMPTY", "M", "NO-MFR", 1, "m", "no-mfr")],
                ),
            ],
        ),
    );
    mock.route(
        "/manufacturers/normalize",
        normalize(
            &["M", "M"],
            &["N", "ALT-N", "NO-MFR"],
            &["m", "m"],
            &["n", "alt-n", "no-mfr"],
        ),
    );
    let (url, _) = mock.start();
    let output = run(&dir, &url, &["--supplier", "acme", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let entries = json["suppliers"][0]["entries"].as_array().unwrap();
    assert_eq!(entries[0]["status"], "out-of-stock");
    assert_eq!(entries[1]["suggestions"][0]["catalog"]["pn"], "S-YES");
    assert_eq!(entries[1]["suggestions"][0]["manufacturer_match"], true);
    assert_eq!(
        entries[2]["suggestions"][0]["manufacturer_match"],
        serde_json::Value::Null
    );
    assert!(entries.iter().all(|entry| entry.get("mapping").is_none()));
}

#[test]
fn without_explicit_supplier_no_unbound_entries_or_suggestion_queries() {
    let dir = project(
        "implicit",
        r#"    primary { mfr: "M", mpn: "N", footprint: FP, supplier acme "P-1" }
    alt { mfr: "M", mpn: "ALT" }"#,
        "",
    );
    let mock = Mock::default();
    mock.route(
        "/suppliers/acme/parts",
        pn_response(
            "acme",
            &[("P-1", Some(catalog_keys("P-1", "M", "N", 1, "m", "n")))],
        ),
    );
    mock.route(
        "/manufacturers/normalize",
        normalize(&["M"], &["N"], &["m"], &["n"]),
    );
    let (url, requests) = mock.start();
    let output = run(&dir, &url, &[]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
    assert!(!stdout(&output).contains("unbound"));
    assert_eq!(requests.try_iter().count(), 2);
}

#[test]
fn check_failure_and_invalid_arguments_make_no_requests_and_help_documents_three() {
    let dir = project(
        "check-fail",
        "    primary { mfr: \"M\", mpn: \"N\", supplier acme \"P-1\" }\n    BOGUS",
        "",
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let output = run(&dir, &url, &["--supplier", "acme"]);
    assert_eq!(output.status.code(), Some(1));
    listener.set_nonblocking(true).unwrap();
    assert!(
        listener.accept().is_err(),
        "offline check must make zero requests"
    );
    let bad = run(&dir, &url, &["--supplier", "INVALID"]);
    assert_eq!(bad.status.code(), Some(2));
    let help = Command::new(env!("CARGO_BIN_EXE_cohdl"))
        .arg("--help")
        .output()
        .unwrap();
    assert_eq!(help.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&help.stderr).contains("3  suppliers only"));
}

#[test]
fn malformed_protocol_is_e1204_exit_one_not_mismatch() {
    let dir = project(
        "protocol",
        r#"    primary { mfr: "M", mpn: "N", footprint: FP, supplier acme "P-1" }"#,
        "",
    );
    let mock = Mock::default();
    mock.route_status(
        "/suppliers/acme/parts",
        200,
        r#"{"supplier":"acme","snapshot_date":null,"parts":[{"pn":"P-1","catalog":{"pn":"P-1"}}]}"#,
    );
    let (url, _) = mock.start();
    let output = run(&dir, &url, &["--json"]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(json["diagnostics"][0]["code"], "E1204");
    assert!(!stdout(&output).contains("mismatch"));
}

#[test]
fn pn_and_normalize_each_split_at_one_hundred_with_stable_association() {
    let count = 101;
    let entries = (0..count)
        .map(|i| {
            format!(
                "    {} {{ mfr: \"M{i:03}\", mpn: \"N{i:03}\", supplier acme \"P{i:03}\", footprint: FP }}",
                if i == 0 { "primary" } else { "alt" }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let dir = project("batches", &entries, "");
    let mock = Mock::default();
    for (start, end) in [(0, 100), (100, 101)] {
        let rows = (start..end)
            .map(|i| {
                let pn = format!("P{i:03}");
                (
                    pn,
                    Some(catalog(
                        &format!("P{i:03}"),
                        &format!("M{i:03}"),
                        &format!("N{i:03}"),
                        1,
                    )),
                )
            })
            .collect::<Vec<_>>();
        let borrowed = rows
            .iter()
            .map(|(pn, row)| (pn.as_str(), row.clone()))
            .collect::<Vec<_>>();
        mock.route(
            &format!(
                "/suppliers/acme/parts?pn={}",
                (start..end)
                    .map(|i| format!("P{i:03}"))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            pn_response("acme", &borrowed),
        );
    }
    for start in [0, 100] {
        let end = (start + 100).min(count);
        let mfr = (start..end).map(|i| format!("M{i:03}")).collect::<Vec<_>>();
        let mpn = (start..end).map(|i| format!("N{i:03}")).collect::<Vec<_>>();
        let query = mfr
            .iter()
            .map(|value| format!("mfr={value}"))
            .chain(mpn.iter().map(|value| format!("mpn={value}")))
            .collect::<Vec<_>>()
            .join("&");
        let mfr_ref = mfr.iter().map(String::as_str).collect::<Vec<_>>();
        let mpn_ref = mpn.iter().map(String::as_str).collect::<Vec<_>>();
        mock.route(
            &format!("/manufacturers/normalize?{query}"),
            normalize(&mfr_ref, &mpn_ref, &mfr_ref, &mpn_ref),
        );
    }
    let (url, requests) = mock.start();
    let output = run(&dir, &url, &["--json"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
    let targets: Vec<String> = requests.try_iter().collect();
    assert_eq!(targets.len(), 4, "{targets:?}");
    assert!(
        targets[0].to_ascii_uppercase().contains("P000%2C") && targets[0].contains("P099"),
        "{targets:?}"
    );
    assert!(targets[1].contains("pn=P100"));
    assert!(
        targets[2].contains("mfr=M000")
            && targets[2].contains("mfr=M099")
            && targets[2].contains("mpn=N099")
    );
    assert!(targets[3].contains("mfr=M100") && targets[3].contains("mpn=N100"));
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(
        json["suppliers"][0]["entries"].as_array().unwrap().len(),
        count
    );
    assert!(json["suppliers"][0]["entries"]
        .as_array()
        .unwrap()
        .iter()
        .all(|entry| entry["status"] == "ok"));
}

#[test]
fn repeated_unbound_mpn_reuses_one_query_for_both_entries() {
    let dir = project(
        "repeat-mpn",
        r#"    primary { mfr: "M", mpn: "N", footprint: FP }
    alt { mfr: "M", mpn: "N" }"#,
        "",
    );
    let mock = Mock::default();
    mock.route(
        "/suppliers/acme/parts?mpn=N",
        mpn_response("acme", &[("N", vec![catalog("S-1", "M", "N", 1)])]),
    );
    mock.route(
        "/manufacturers/normalize",
        normalize(&["M", "M"], &["N", "N"], &["M", "M"], &["N", "N"]),
    );
    let (url, requests) = mock.start();
    let output = run(&dir, &url, &["--supplier", "acme", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
    let targets: Vec<String> = requests.try_iter().collect();
    assert_eq!(
        targets
            .iter()
            .filter(|target| target.contains("mpn=N") && target.starts_with("/suppliers/"))
            .count(),
        1
    );
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let entries = json["suppliers"][0]["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert!(entries
        .iter()
        .all(|entry| entry["suggestions"][0]["catalog"]["pn"] == "S-1"));
}

#[test]
fn only_instantiated_parts_are_audited_and_human_output_is_sorted() {
    let dir = project(
        "scope",
        r#"    primary { mfr: "M", mpn: "N", footprint: FP, supplier bolt "B-1" }
    alt { mfr: "M", mpn: "N", supplier acme "A-1" }"#,
        r#"pub part UNUSED: D { primary { mfr: "M", mpn: "UNUSED", footprint: FP, supplier acme "UNUSED-PN" } }"#,
    );
    let mock = Mock::default();
    for supplier in ["acme", "bolt"] {
        let pn = if supplier == "acme" { "A-1" } else { "B-1" };
        mock.route(
            &format!("/suppliers/{supplier}/parts"),
            pn_response(
                supplier,
                &[(pn, Some(catalog_keys(pn, "M", "N", 1, "m", "n")))],
            ),
        );
    }
    mock.route(
        "/manufacturers/normalize",
        normalize(&["M"], &["N"], &["m"], &["n"]),
    );
    let (url, requests) = mock.start();
    let output = run(&dir, &url, &[]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
    let text = stdout(&output);
    assert!(text.find("acme").unwrap() < text.find("bolt").unwrap());
    assert!(!text.contains("UNUSED"));
    assert_eq!(requests.try_iter().count(), 4);
}

#[test]
fn shared_pn_across_parts_is_deduplicated_and_distributed_to_each_entry() {
    let dir = project_design(
        "shared-pn",
        r#"    primary { mfr: "M", mpn: "N", footprint: FP, supplier acme "SHARED" }
    alt { mfr: "M", mpn: "N", supplier acme "SHARED" }"#,
        r#"pub part SECOND: D { primary { mfr: "M", mpn: "N", footprint: FP, supplier acme "SHARED" } }"#,
        "design Board { inst p: P inst second: SECOND nc: p.A nc: second.A }",
    );
    let mock = Mock::default();
    mock.route(
        "/suppliers/acme/parts?pn=SHARED",
        pn_response(
            "acme",
            &[(
                "SHARED",
                Some(catalog_keys("SHARED", "M", "N", 5, "m", "n")),
            )],
        ),
    );
    mock.route(
        "/manufacturers/normalize",
        normalize(
            &["M", "M", "M"],
            &["N", "N", "N"],
            &["m", "m", "m"],
            &["n", "n", "n"],
        ),
    );
    let (url, requests) = mock.start();
    let output = run(&dir, &url, &["--json"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
    let targets: Vec<String> = requests.try_iter().collect();
    assert_eq!(targets[0], "/suppliers/acme/parts?pn=SHARED");
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let entries = json["suppliers"][0]["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
    assert!(entries.iter().all(|entry| entry["status"] == "ok"));
    assert!(entries.iter().all(|entry| entry["pn"] == "SHARED"));
}

#[test]
fn unqueryable_values_are_not_sent_and_have_explicit_notes() {
    let long_mpn = "N".repeat(257);
    let long_mfr = "M".repeat(257);
    let entries = format!(
        r#"    primary {{ mfr: "{long_mfr}", mpn: "LONG-MFR", footprint: FP, supplier acme "GOOD" }}
    alt {{ mfr: "M", mpn: "{long_mpn}", supplier acme "BAD,PN" }}
    alt {{ mfr: "M", mpn: "LONG-PN", supplier acme "PN-LONG-FIELD" }}
    alt {{ mfr: "M", mpn: "BAD,MPN" }}
    alt {{ mfr: "M", mpn: "{long_mpn}" }}
    alt {{ mfr: "{long_mfr}", mpn: "LONG-MFR" }}"#
    );
    let dir = project("unqueryable", &entries, "");
    let mock = Mock::default();
    mock.route(
        "/suppliers/acme/parts?pn=",
        pn_response(
            "acme",
            &[
                ("GOOD", Some(catalog("GOOD", "M", "LONG-MFR", 1))),
                ("PN-LONG-FIELD", None),
            ],
        ),
    );
    mock.route(
        "/suppliers/acme/parts?mpn=LONG-MFR",
        mpn_response("acme", &[("LONG-MFR", Vec::new())]),
    );
    mock.route(
        "/manufacturers/normalize",
        normalize(
            &["M", "M", "M", "M"],
            &["LONG-MFR", "LONG-PN", "BAD,MPN", "LONG-MFR"],
            &["M", "M", "M", "M"],
            &["LONG-MFR", "LONG-PN", "BAD,MPN", "LONG-MFR"],
        ),
    );
    let (url, requests) = mock.start();
    let output = run(&dir, &url, &["--supplier", "acme", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
    let targets: Vec<String> = requests.try_iter().collect();
    assert!(targets
        .iter()
        .filter(|target| target.starts_with("/suppliers/"))
        .all(|target| {
            let decoded = percent_decode(target);
            !decoded.contains("BAD,PN")
                && !decoded.contains("BAD,MPN")
                && !decoded.contains(&"P".repeat(257))
                && !decoded.contains(&"N".repeat(257))
        }));
    assert!(targets
        .iter()
        .all(|target| !target.contains("MMMM") && !target.contains("NNNN")));
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let entries = json["suppliers"][0]["entries"].as_array().unwrap();
    assert_eq!(entries[0]["status"], "ok");
    assert_eq!(entries[0]["mismatches"], serde_json::json!([]));
    assert_eq!(entries[1]["status"], "unknown");
    assert_eq!(
        entries[1]["note"],
        "part number cannot be queried (contains ',')"
    );
    assert_eq!(entries[2]["status"], "unknown");
    assert!(entries[2].get("note").is_none());
    for index in [3, 4] {
        assert_eq!(entries[index]["status"], "unbound");
        assert_eq!(entries[index]["suggestions"], serde_json::json!([]));
        assert!(entries[index]["note"]
            .as_str()
            .unwrap()
            .contains("cannot be queried"));
    }
    assert_eq!(entries[5]["status"], "unbound");
    assert_eq!(entries[5]["suggestions"], serde_json::json!([]));
    let normalize_request = targets
        .iter()
        .find(|target| target.starts_with("/manufacturers/normalize?"))
        .unwrap();
    assert_eq!(normalize_request.matches("mfr=").count(), 4);
    assert_eq!(normalize_request.matches("mpn=").count(), 4);
}

#[test]
fn overlong_pn_fails_offline_before_any_request() {
    let long_pn = "P".repeat(257);
    let dir = project(
        "overlong-pn",
        &format!(
            r#"    primary {{ mfr: "M", mpn: "N", footprint: FP, supplier acme "{long_pn}" }}"#
        ),
        "",
    );
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let output = run(&dir, &url, &[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("E813"));
    listener.set_nonblocking(true).unwrap();
    assert!(
        listener.accept().is_err(),
        "overlong PN must make zero requests"
    );
}

#[test]
fn unlisted_supplier_has_empty_bindings_and_does_not_change_exit_code() {
    let dir = project(
        "unlisted",
        r#"    primary { mfr: "M", mpn: "N", footprint: FP, supplier acme "P-1" }"#,
        "",
    );
    let mock = Mock::default();
    mock.route_status(
        "/suppliers/acme/parts",
        404,
        "{ \n  \"error\" : \"supplier not found\" \n}",
    );
    let (url, requests) = mock.start();
    let human = run(&dir, &url, &[]);
    assert_eq!(
        human.status.code(),
        Some(0),
        "{}{}",
        stdout(&human),
        stderr(&human)
    );
    assert!(stdout(&human).contains("acme unlisted: registry has no catalog for this supplier"));
    let output = run(&dir, &url, &["--json"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(json["suppliers"][0]["catalog"], "unlisted");
    assert_eq!(json["suppliers"][0]["bindings"], serde_json::json!([]));
    assert_eq!(
        json["suppliers"][0]["note"],
        "registry has no catalog for this supplier"
    );
    assert!(json["suppliers"][0].get("entries").is_none());
    assert_eq!(requests.try_iter().count(), 2);
}

#[test]
fn other_not_found_is_still_e1204_and_readme_documents_exit_three() {
    let dir = project(
        "other-404",
        r#"    primary { mfr: "M", mpn: "N", footprint: FP, supplier acme "P-1" }"#,
        "",
    );
    let mock = Mock::default();
    mock.route_status("/suppliers/acme/parts", 404, r#"{"error":"not found"}"#);
    let (url, _) = mock.start();
    let output = run(&dir, &url, &["--json"]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}{}",
        stdout(&output),
        stderr(&output)
    );
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    assert_eq!(json["diagnostics"][0]["code"], "E1204");
    let readme =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md")).unwrap();
    assert!(readme.contains("`3` = `cohdl suppliers` found an `unknown` or `mismatched`"));
}
