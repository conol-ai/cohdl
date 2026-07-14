//! RFC-014 `cohdl lsp` conformance.
//!
//! The load-bearing property is the RFC's equivalence test: the LSP's
//! `publishDiagnostics` payload for a file must match `cohdl check --json`'s
//! diagnostics for the same file, field-for-field (code, severity, message,
//! position — LSP is 0-based/UTF-16 where the JSON is 1-based/scalar, so the
//! comparison maps encodings, not meanings). Hover/goto-def/references are
//! fixture-driven per the RFC's Gradeability section. Everything runs over the
//! real binary and real JSON-RPC framing — no in-process shortcuts.

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct Lsp {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
}

impl Lsp {
    fn start() -> Lsp {
        let mut child = Command::new(env!("CARGO_BIN_EXE_cohdl"))
            .arg("lsp")
            .env(
                "COHDL_STD",
                Path::new(env!("CARGO_MANIFEST_DIR")).join("std"),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn cohdl lsp");
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut lsp = Lsp {
            child,
            stdin,
            stdout,
            next_id: 1,
        };
        let init = lsp.request("initialize", json!({ "capabilities": {} }));
        assert!(
            init["capabilities"]["hoverProvider"].as_bool() == Some(true),
            "server must advertise hover: {}",
            init
        );
        lsp.notify("initialized", json!({}));
        lsp
    }

    fn send(&mut self, msg: &Value) {
        let payload = serde_json::to_string(msg).unwrap();
        write!(
            self.stdin,
            "Content-Length: {}\r\n\r\n{}",
            payload.len(),
            payload
        )
        .unwrap();
        self.stdin.flush().unwrap();
    }

    fn notify(&mut self, method: &str, params: Value) {
        self.send(&json!({ "jsonrpc": "2.0", "method": method, "params": params }));
    }

    /// Send a request and read messages until its response arrives; any
    /// notifications read along the way are dropped.
    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        loop {
            let msg = self.read_message();
            if msg.get("id").and_then(Value::as_i64) == Some(id) {
                return msg["result"].clone();
            }
        }
    }

    /// Read messages until a `publishDiagnostics` for `uri` arrives.
    fn await_diagnostics(&mut self, uri: &str) -> Vec<Value> {
        loop {
            let msg = self.read_message();
            if msg.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics")
                && msg["params"]["uri"].as_str() == Some(uri)
            {
                return msg["params"]["diagnostics"].as_array().unwrap().clone();
            }
        }
    }

    fn read_message(&mut self) -> Value {
        let mut content_length = 0usize;
        loop {
            let mut line = String::new();
            let n = self.stdout.read_line(&mut line).expect("read header");
            assert!(n > 0, "server closed its stdout unexpectedly");
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            if let Some(v) = line.strip_prefix("Content-Length:") {
                content_length = v.trim().parse().unwrap();
            }
        }
        let mut buf = vec![0u8; content_length];
        self.stdout.read_exact(&mut buf).expect("read payload");
        serde_json::from_slice(&buf).expect("valid JSON-RPC payload")
    }

    fn shutdown(mut self) {
        let _ = self.request("shutdown", Value::Null);
        self.notify("exit", Value::Null);
        let status = self.child.wait().expect("server exit");
        assert!(status.success(), "clean shutdown/exit must be exit 0");
    }
}

/// Write a fixture file and return (path, file:// uri, text).
fn fixture(name: &str, text: &str) -> (PathBuf, String, String) {
    let dir = std::env::temp_dir().join(format!("cohdl-lsp-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, text).unwrap();
    let canonical = path.canonicalize().unwrap();
    let uri = format!("file://{}", canonical.display());
    (canonical, uri, text.to_string())
}

fn did_open(lsp: &mut Lsp, uri: &str, text: &str) {
    lsp.notify(
        "textDocument/didOpen",
        json!({ "textDocument": { "uri": uri, "languageId": "cohdl", "version": 1, "text": text } }),
    );
}

// Unique names so the fixture never collides with std declarations.
const DIAG_FIXTURE: &str = "\
pub device LspProbe { pins { A: 1 [passive], B: 2 [output] } }
design LspBoard {
    inst p: LspProbe
    inst bad: NoSuchDeviceHere
    net LONELY: p.B
    net N: p.A, ghost.PIN
}
";

// ---------------------------------------------------------------------------
// The RFC's mandatory equivalence test: LSP diagnostics == `check --json`.

#[test]
fn publish_diagnostics_matches_check_json() {
    let (path, uri, text) = fixture("diag.cohdl", DIAG_FIXTURE);

    // Ground truth: the CLI's --json output for the same file + same std.
    let out = Command::new(env!("CARGO_BIN_EXE_cohdl"))
        .args(["check", path.to_str().unwrap(), "--json"])
        .env(
            "COHDL_STD",
            Path::new(env!("CARGO_MANIFEST_DIR")).join("std"),
        )
        .output()
        .unwrap();
    let doc: Value = serde_json::from_slice(&out.stdout).expect("check --json parses");
    let json_diags: Vec<&Value> = doc["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|d| d["primary"]["file"].as_str() == Some(path.to_str().unwrap()))
        .collect();
    assert!(
        json_diags.len() >= 2,
        "fixture must produce diagnostics: {}",
        doc
    );

    // The LSP's view of the same file.
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, &text);
    let lsp_diags = lsp.await_diagnostics(&uri);

    assert_eq!(
        lsp_diags.len(),
        json_diags.len(),
        "diagnostic count must match:\nlsp={:?}\njson={:?}",
        lsp_diags,
        json_diags
    );
    for (l, j) in lsp_diags.iter().zip(&json_diags) {
        assert_eq!(l["code"].as_str(), j["code"].as_str(), "code");
        let sev = match j["severity"].as_str().unwrap() {
            "error" => 1,
            "warning" => 2,
            other => panic!("unknown severity {}", other),
        };
        assert_eq!(l["severity"].as_i64(), Some(sev), "severity");
        assert_eq!(l["message"].as_str(), j["message"].as_str(), "message");
        // Encoding map: LSP 0-based; JSON 1-based (ASCII fixture, so UTF-16
        // char == scalar col).
        assert_eq!(
            l["range"]["start"]["line"].as_u64().unwrap() + 1,
            j["primary"]["start_line"].as_u64().unwrap(),
            "start line"
        );
        assert_eq!(
            l["range"]["start"]["character"].as_u64().unwrap() + 1,
            j["primary"]["start_col"].as_u64().unwrap(),
            "start col"
        );
        assert_eq!(
            l["range"]["end"]["line"].as_u64().unwrap() + 1,
            j["primary"]["end_line"].as_u64().unwrap(),
            "end line"
        );
        // Help lines ride relatedInformation, prefixed.
        let help = j["help"].as_array().unwrap();
        if !help.is_empty() {
            let related = l["relatedInformation"].as_array().unwrap();
            for h in help {
                let expect = format!("help: {}", h.as_str().unwrap());
                assert!(
                    related
                        .iter()
                        .any(|r| r["message"].as_str() == Some(expect.as_str())),
                    "help line missing from relatedInformation: {}",
                    expect
                );
            }
        }
    }
    lsp.shutdown();
}

// ---------------------------------------------------------------------------
// Unsaved-buffer overlay: didChange text wins over the disk.

#[test]
fn did_change_overlay_updates_diagnostics() {
    let (_path, uri, text) = fixture("overlay.cohdl", DIAG_FIXTURE);
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, &text);
    let first = lsp.await_diagnostics(&uri);
    assert!(first.iter().any(|d| d["code"] == "E202"), "{:?}", first);

    // Fix the unknown device in the BUFFER only (disk untouched).
    let fixed = text
        .replace("inst bad: NoSuchDeviceHere\n", "")
        .replace(", ghost.PIN", "");
    lsp.notify(
        "textDocument/didChange",
        json!({
            "textDocument": { "uri": uri, "version": 2 },
            "contentChanges": [ { "text": fixed } ],
        }),
    );
    let second = lsp.await_diagnostics(&uri);
    assert!(
        !second.iter().any(|d| d["code"] == "E202"),
        "overlay must reflect the unsaved buffer:\n{:?}",
        second
    );
    // The dangling-driver warning (D003 on the [output] pin) remains.
    assert!(second.iter().any(|d| d["code"] == "D003"), "{:?}", second);
    lsp.shutdown();
}

// ---------------------------------------------------------------------------
// Hover: the DR-013 empty-impl resolved mapping, and pin obligation/role.

#[test]
fn hover_empty_impl_and_pin() {
    let src = "\
pub trait LspTrait { pins { required A: pin } spec { resistance: Resistance } }
pub device LspDev { pins { A: 1 [passive] } spec { resistance: 1kohm } }
impl LspTrait for LspDev {}
design LspB {
    inst d: LspDev
    net N: d.A
}
";
    let (_path, uri, text) = fixture("hover.cohdl", src);
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, &text);
    let _ = lsp.await_diagnostics(&uri);

    // Hover over the empty impl (line 2, inside `impl LspTrait for LspDev {}`).
    let hover = lsp.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 2, "character": 6 } }),
    );
    let md = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(
        md.contains("`A` ← `A`"),
        "resolved pin mapping on hover:\n{}",
        md
    );
    assert!(
        md.contains("`resistance` ← `resistance`"),
        "resolved spec mapping on hover:\n{}",
        md
    );

    // Hover over the device pin declaration `A` (line 1).
    let col = src.lines().nth(1).unwrap().find("A:").unwrap() as u64;
    let hover = lsp.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 1, "character": col } }),
    );
    let md = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(
        md.contains("required") && md.contains("`passive`"),
        "pin obligation/role on hover:\n{}",
        md
    );
    lsp.shutdown();
}

// ---------------------------------------------------------------------------
// Goto-definition: a use site resolves to the declaration.

#[test]
fn goto_definition_resolves_inst_type() {
    let src = "\
pub device LspTarget { pins { A: 1 [passive] } }
design LspB {
    inst d: LspTarget
    net N: d.A
}
";
    let (_path, uri, text) = fixture("def.cohdl", src);
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, &text);
    let _ = lsp.await_diagnostics(&uri);

    // Cursor on `LspTarget` in `inst d: LspTarget` (line 2).
    let col = src.lines().nth(2).unwrap().find("LspTarget").unwrap() as u64;
    let def = lsp.request(
        "textDocument/definition",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 2, "character": col + 2 } }),
    );
    assert_eq!(def["uri"].as_str(), Some(uri.as_str()), "{}", def);
    // The declaration's name is on line 0.
    assert_eq!(def["range"]["start"]["line"].as_u64(), Some(0), "{}", def);
    let decl_col = src.lines().next().unwrap().find("LspTarget").unwrap() as u64;
    assert_eq!(
        def["range"]["start"]["character"].as_u64(),
        Some(decl_col),
        "{}",
        def
    );
    lsp.shutdown();
}

// ---------------------------------------------------------------------------
// References: find all impls for a trait / of a device (DR-013's ask).

#[test]
fn references_lists_all_impls() {
    let src = "\
pub trait LspMulti { pins { required A: pin } }
pub device DevOne { pins { A: 1 [passive] } }
pub device DevTwo { pins { A: 1 [passive] } }
impl LspMulti for DevOne {}
impl LspMulti for DevTwo {}
design LspB {
    inst d: DevOne
    net N: d.A
}
";
    let (_path, uri, text) = fixture("refs.cohdl", src);
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, &text);
    let _ = lsp.await_diagnostics(&uri);

    // On the trait name at its declaration (line 0).
    let col = src.lines().next().unwrap().find("LspMulti").unwrap() as u64;
    let refs = lsp.request(
        "textDocument/references",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": 0, "character": col + 1 },
            "context": { "includeDeclaration": false },
        }),
    );
    let arr = refs.as_array().unwrap();
    assert_eq!(arr.len(), 2, "two impls of LspMulti:\n{}", refs);

    // On a device name inside an impl statement (line 3) — one impl.
    let col = src.lines().nth(3).unwrap().find("DevOne").unwrap() as u64;
    let refs = lsp.request(
        "textDocument/references",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": 3, "character": col + 1 },
            "context": { "includeDeclaration": false },
        }),
    );
    // DevOne appears in exactly one impl (plus LspMulti matches that same
    // impl — cursor is on the device name, so device matching applies).
    assert_eq!(refs.as_array().unwrap().len(), 1, "{}", refs);
    lsp.shutdown();
}

// ---------------------------------------------------------------------------
// Adversarial-verification regressions (RFC-014 round 1).

// Finding 1/5 (high): paths needing percent-encoding must not lose diagnostics.
#[test]
fn percent_encoded_paths_keep_diagnostics() {
    let dir = std::env::temp_dir().join(format!("cohdl lsp space {}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("space fixture.cohdl");
    std::fs::write(&path, DIAG_FIXTURE).unwrap();
    let canonical = path.canonicalize().unwrap();
    let uri = format!(
        "file://{}",
        canonical.display().to_string().replace(' ', "%20")
    );

    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, DIAG_FIXTURE);
    let diags = lsp.await_diagnostics(&uri);
    assert!(
        diags.iter().any(|d| d["code"] == "E202"),
        "diagnostics must survive percent-encoded paths:\n{:?}",
        diags
    );
    // goto-def also works through the encoded URI.
    let col = DIAG_FIXTURE
        .lines()
        .nth(2)
        .unwrap()
        .find("LspProbe")
        .unwrap() as u64;
    let def = lsp.request(
        "textDocument/definition",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 2, "character": col + 1 } }),
    );
    assert!(def.is_object(), "definition must resolve: {}", def);
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&dir);
}

// Finding 2 (medium): one corrupt frame must not kill the session.
#[test]
fn malformed_frames_get_parse_errors_and_session_survives() {
    let (_path, uri, text) = fixture("robust.cohdl", DIAG_FIXTURE);
    let mut lsp = Lsp::start();

    // Garbage JSON with valid framing -> -32700 response, id null.
    let garbage = b"{this is : not json";
    write!(lsp.stdin, "Content-Length: {}\r\n\r\n", garbage.len()).unwrap();
    lsp.stdin.write_all(garbage).unwrap();
    lsp.stdin.flush().unwrap();
    let err = lsp.read_message();
    assert_eq!(err["error"]["code"].as_i64(), Some(-32700), "{}", err);
    assert!(err["id"].is_null(), "{}", err);

    // Headers without Content-Length -> also recoverable.
    lsp.stdin.write_all(b"X-Garbage: yes\r\n\r\n").unwrap();
    lsp.stdin.flush().unwrap();
    let err = lsp.read_message();
    assert_eq!(err["error"]["code"].as_i64(), Some(-32700), "{}", err);

    // The session still works end-to-end afterwards.
    did_open(&mut lsp, &uri, &text);
    let diags = lsp.await_diagnostics(&uri);
    assert!(!diags.is_empty(), "session must survive corrupt frames");
    lsp.shutdown();
}

// Finding 3 (medium): a buffer that does not exist on disk still checks.
#[test]
fn phantom_buffer_gets_diagnostics_from_overlay() {
    let dir = std::env::temp_dir().join(format!("cohdl-lsp-phantom-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    // NOTE: the path is never written to disk.
    let path = dir.canonicalize().unwrap().join("unsaved.cohdl");
    let uri = format!("file://{}", path.display());
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, DIAG_FIXTURE);
    let diags = lsp.await_diagnostics(&uri);
    assert!(
        diags.iter().any(|d| d["code"] == "E202"),
        "overlay-only buffer must be checked:\n{:?}",
        diags
    );
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&dir);
}

// Findings 4/8 (low): exit without shutdown is exit code 1 (LSP spec).
#[test]
fn exit_without_shutdown_is_exit_code_1() {
    let mut lsp = Lsp::start();
    lsp.notify("exit", Value::Null);
    let status = lsp.child.wait().unwrap();
    assert_eq!(status.code(), Some(1), "spec: exit w/o shutdown -> 1");
}

// Finding 6 (medium): a design-selection failure surfaces as showMessage,
// while declaration-stage diagnostics still publish. (Body-stage diagnostics
// cannot exist here — no design is selected, so no expansion runs; that
// matches the CLI exactly.)
#[test]
fn selection_error_surfaces_as_show_message() {
    let src = "\
pub device ZzSel { pins { A: 1 [passive] } }
impl ZzNoSuchTrait for ZzSel {}
design SelOne { inst d: ZzSel
net N: d.A }
design SelTwo { inst d: ZzSel
net N: d.A }
";
    let (_path, uri, text) = fixture("sel.cohdl", src);
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, &text);
    // Expect BOTH (each guaranteed to be sent): a publish carrying the
    // declaration-stage E202 (unknown trait), and a window/showMessage with
    // the selection error.
    let mut saw_show_message = false;
    let mut saw_e202 = false;
    while !(saw_show_message && saw_e202) {
        let msg = lsp.read_message();
        match msg.get("method").and_then(Value::as_str) {
            Some("window/showMessage") => {
                let m = msg["params"]["message"].as_str().unwrap_or("");
                assert!(m.contains("designs"), "{}", m);
                saw_show_message = true;
            }
            Some("textDocument/publishDiagnostics")
                if msg["params"]["uri"].as_str() == Some(uri.as_str()) =>
            {
                let ds = msg["params"]["diagnostics"].as_array().unwrap();
                if ds.iter().any(|d| d["code"] == "E202") {
                    saw_e202 = true;
                }
            }
            _ => {}
        }
    }
    lsp.shutdown();
}

// Finding 7 (low): goto-def works on a turbofish generic argument in a call.
#[test]
fn goto_definition_resolves_call_turbofish_arg() {
    let src = "\
pub device ZzTfDev { pins { A: 1 [passive] } }
pub trait ZzTfTrait { pins { required A: pin } }
impl ZzTfTrait for ZzTfDev {}
fn zzuse<D: ZzTfTrait>(target: D, p: Pin) {
    net _: p, target.A
}
design ZzTfB {
    inst d: ZzTfDev
    inst e: ZzTfDev
    zzuse::<ZzTfDev>(d, e.A)
    net N: d.A
}
";
    let (_path, uri, text) = fixture("tf.cohdl", src);
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, &text);
    let _ = lsp.await_diagnostics(&uri);
    // Cursor on `ZzTfDev` inside `zzuse::<ZzTfDev>(...)`.
    let line = src.lines().position(|l| l.contains("zzuse::<")).unwrap() as u64;
    let col = src
        .lines()
        .find(|l| l.contains("zzuse::<"))
        .unwrap()
        .find("ZzTfDev")
        .unwrap() as u64;
    let def = lsp.request(
        "textDocument/definition",
        json!({ "textDocument": { "uri": uri }, "position": { "line": line, "character": col + 1 } }),
    );
    assert_eq!(def["range"]["start"]["line"].as_u64(), Some(0), "{}", def);
    lsp.shutdown();
}
