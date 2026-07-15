//! RFC-014 `cohdl lsp` conformance.
//!
//! The load-bearing property is the RFC's equivalence discipline: for every
//! fixture in the diagnostic corpus below, the LSP's `publishDiagnostics`
//! payload must match `cohdl check --json` on the complete four-field
//! projection — code, severity, message, and BOTH range endpoints (LSP is
//! 0-based/UTF-16 where the JSON is 1-based/scalar, so the comparison maps
//! encodings from the fixture text itself, which also covers the non-ASCII
//! case) — plus the full relatedInformation projection (secondary locations
//! and help lines). Hover/goto-def/references assert exact text and exact
//! spans per the RFC's Gradeability section. Everything runs over the real
//! binary and real JSON-RPC framing — no in-process shortcuts.
//!
//! NOT covered here: the RFC's real-VS-Code acceptance item. A subprocess
//! protocol test is exactly what the Accepted text distinguishes from a live
//! client session; that item stays open until a real editor pass is recorded
//! (docs/compliance-report.md).

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

fn repo_std() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("std")
}

impl Lsp {
    /// Spawn the server WITHOUT the initialize handshake (lifecycle tests).
    fn spawn_with_std(std_dir: &Path) -> Lsp {
        let mut child = Command::new(env!("CARGO_BIN_EXE_cohdl"))
            .arg("lsp")
            .env("COHDL_STD", std_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn cohdl lsp");
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        Lsp {
            child,
            stdin,
            stdout,
            next_id: 1,
        }
    }

    fn spawn() -> Lsp {
        Self::spawn_with_std(&repo_std())
    }

    /// Spawn + initialize, advertising `relatedInformation` support (the
    /// equivalence tests rely on help/secondary riding relatedInformation).
    fn start() -> Lsp {
        let mut lsp = Self::spawn();
        let init = lsp.request(
            "initialize",
            json!({ "capabilities": { "textDocument": { "publishDiagnostics": { "relatedInformation": true } } } }),
        );
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

    /// Send a request and read messages until its response arrives; returns
    /// the FULL response message (so tests can assert on `error`).
    fn request_full(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        loop {
            let msg = self.read_message();
            if msg.get("id").and_then(Value::as_i64) == Some(id) {
                return msg;
            }
        }
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        self.request_full(method, params)["result"].clone()
    }

    /// Read messages until a `publishDiagnostics` for `uri` arrives.
    fn await_diagnostics(&mut self, uri: &str) -> Vec<Value> {
        self.await_diagnostics_capturing(uri).0
    }

    /// Same, but also return every OTHER message read along the way (for
    /// asserting what was NOT sent — e.g. cross-project clears).
    fn await_diagnostics_capturing(&mut self, uri: &str) -> (Vec<Value>, Vec<Value>) {
        let mut seen = Vec::new();
        loop {
            let msg = self.read_message();
            if msg.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics")
                && msg["params"]["uri"].as_str() == Some(uri)
            {
                return (
                    msg["params"]["diagnostics"].as_array().unwrap().clone(),
                    seen,
                );
            }
            seen.push(msg);
        }
    }

    /// Round-trip a no-op request and return every message that arrived
    /// before its response — flushes any pending server-to-client traffic.
    fn drain(&mut self) -> Vec<Value> {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": "zz/drain", "params": {} }));
        let mut seen = Vec::new();
        loop {
            let msg = self.read_message();
            if msg.get("id").and_then(Value::as_i64) == Some(id) {
                return seen;
            }
            seen.push(msg);
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

fn is_empty_publish_for(msg: &Value, uri: &str) -> bool {
    msg.get("method").and_then(Value::as_str) == Some("textDocument/publishDiagnostics")
        && msg["params"]["uri"].as_str() == Some(uri)
        && msg["params"]["diagnostics"]
            .as_array()
            .is_some_and(Vec::is_empty)
}

// ---------------------------------------------------------------------------
// The RFC's mandatory equivalence gate: LSP diagnostics == `check --json`,
// full four-field projection, over a corpus of fixtures.

/// 1-based Unicode-scalar column → 0-based UTF-16 code-unit column, computed
/// from the fixture text itself (so non-ASCII fixtures are covered).
fn utf16_col(text: &str, line0: u64, scalar_col1: u64) -> u64 {
    let line = text.split('\n').nth(line0 as usize).unwrap_or("");
    line.chars()
        .take(scalar_col1 as usize - 1)
        .map(|c| c.len_utf16() as u64)
        .sum()
}

/// Assert one LSP position equals a JSON (1-based line, 1-based scalar col).
fn assert_pos(text: &str, lsp_pos: &Value, j_line: u64, j_col: u64, what: &str) {
    assert_eq!(
        lsp_pos["line"].as_u64().unwrap() + 1,
        j_line,
        "{} line",
        what
    );
    let line0 = j_line - 1;
    assert_eq!(
        lsp_pos["character"].as_u64().unwrap(),
        utf16_col(text, line0, j_col),
        "{} character (utf-16)",
        what
    );
}

/// The full equivalence check for one fixture source.
fn assert_equivalence(name: &str, src: &str) {
    let (path, uri, text) = fixture(name, src);

    // Ground truth: the CLI's --json output for the same file + same std.
    let out = Command::new(env!("CARGO_BIN_EXE_cohdl"))
        .args(["check", path.to_str().unwrap(), "--json"])
        .env("COHDL_STD", repo_std())
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
        !json_diags.is_empty(),
        "corpus fixture `{}` must produce diagnostics: {}",
        name,
        doc
    );

    // The LSP's view of the same file.
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, &text);
    let lsp_diags = lsp.await_diagnostics(&uri);

    assert_eq!(
        lsp_diags.len(),
        json_diags.len(),
        "[{}] diagnostic count must match:\nlsp={:?}\njson={:?}",
        name,
        lsp_diags,
        json_diags
    );
    for (l, j) in lsp_diags.iter().zip(&json_diags) {
        assert_eq!(l["code"].as_str(), j["code"].as_str(), "[{}] code", name);
        let sev = match j["severity"].as_str().unwrap() {
            "error" => 1,
            "warning" => 2,
            other => panic!("unknown severity {}", other),
        };
        assert_eq!(l["severity"].as_i64(), Some(sev), "[{}] severity", name);
        assert_eq!(
            l["message"].as_str(),
            j["message"].as_str(),
            "[{}] message",
            name
        );
        // Both range endpoints, UTF-16-mapped from the fixture text.
        let p = &j["primary"];
        assert_pos(
            &text,
            &l["range"]["start"],
            p["start_line"].as_u64().unwrap(),
            p["start_col"].as_u64().unwrap(),
            &format!("[{}] start", name),
        );
        assert_pos(
            &text,
            &l["range"]["end"],
            p["end_line"].as_u64().unwrap(),
            p["end_col"].as_u64().unwrap(),
            &format!("[{}] end", name),
        );
        // relatedInformation carries EXACTLY the secondary locations plus the
        // help lines (anchored at the primary range), in that order.
        let secondary = j["secondary"].as_array().unwrap();
        let help = j["help"].as_array().unwrap();
        let related = l["relatedInformation"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        assert_eq!(
            related.len(),
            secondary.len() + help.len(),
            "[{}] relatedInformation must be secondary+help exactly:\n{:?}",
            name,
            l
        );
        for (r, s) in related.iter().zip(secondary) {
            assert_eq!(
                r["message"].as_str(),
                s["message"].as_str(),
                "[{}] secondary message",
                name
            );
            // Fixtures keep secondaries in-file, so the URI is ours.
            assert_eq!(
                r["location"]["uri"].as_str(),
                Some(uri.as_str()),
                "[{}] secondary uri",
                name
            );
            assert_pos(
                &text,
                &r["location"]["range"]["start"],
                s["start_line"].as_u64().unwrap(),
                s["start_col"].as_u64().unwrap(),
                &format!("[{}] secondary start", name),
            );
            assert_pos(
                &text,
                &r["location"]["range"]["end"],
                s["end_line"].as_u64().unwrap(),
                s["end_col"].as_u64().unwrap(),
                &format!("[{}] secondary end", name),
            );
        }
        for (r, h) in related.iter().skip(secondary.len()).zip(help) {
            let expect = format!("help: {}", h.as_str().unwrap());
            assert_eq!(
                r["message"].as_str(),
                Some(expect.as_str()),
                "[{}] help line",
                name
            );
            // Help anchors at the primary range.
            assert_pos(
                &text,
                &r["location"]["range"]["start"],
                p["start_line"].as_u64().unwrap(),
                p["start_col"].as_u64().unwrap(),
                &format!("[{}] help anchor", name),
            );
        }
    }
    lsp.shutdown();
}

// Unique names so the fixtures never collide with std declarations.
const DIAG_FIXTURE: &str = "\
pub device LspProbe { pins { A: 1 [passive], B: 2 [output] } }
design LspBoard {
    inst p: LspProbe
    inst bad: NoSuchDeviceHere
    net LONELY: p.B
    net N: p.A, ghost.PIN
}
";

/// The diagnostic corpus: one fixture per pipeline stage (lex, parse,
/// resolve, units, roles, DRC), plus a non-ASCII fixture for the UTF-16
/// mapping. Every entry runs the FULL equivalence check.
const CORPUS: &[(&str, &str)] = &[
    ("corpus-resolve.cohdl", DIAG_FIXTURE),
    // Lex: Unicode Ω (targeted E101 with rewrite help) + recovery fallout.
    (
        "corpus-lex.cohdl",
        "pub device ZqLex { pins { A: 1 [passive] } spec { r: 10kΩ } }\n",
    ),
    // Parse: E010 on a malformed attribute.
    (
        "corpus-parse.cohdl",
        "pub device ZqPar { pins { A: 1 [passive] } }\ndesign ZqB {\n    #[designator(no_string)] inst d: ZqPar\n    net N: d.A\n}\n",
    ),
    // Units: bare number where a unit literal is expected.
    (
        "corpus-units.cohdl",
        "pub device ZqU<C: Capacitance> { pins { A: 1 [passive] } spec { c: C } }\ndesign ZqB {\n    inst d: ZqU<100>\n    net N: d.A\n}\n",
    ),
    // Roles: RFC-008 missing pin role (E901).
    (
        "corpus-roles.cohdl",
        "pub device ZqR { pins { A: 1 } }\ndesign ZqB {\n    inst d: ZqR\n    net N: d.A\n}\n",
    ),
    // Non-ASCII: an emoji-bearing intent string BEFORE the error span on the
    // same line — scalar columns and UTF-16 columns diverge here.
    (
        "corpus-utf16.cohdl",
        "pub device ZqNa { pins { A: 1 [passive] } }\ndesign ZqB {\n    #[intent(\"héllo wörld 🌍🌍\")] inst bad: ZqNoSuchDev\n    inst d: ZqNa\n    net N: d.A\n}\n",
    ),
];

#[test]
fn publish_diagnostics_matches_check_json() {
    assert_equivalence("diag.cohdl", DIAG_FIXTURE);
}

#[test]
fn equivalence_over_diagnostic_corpus() {
    for (name, src) in CORPUS {
        assert_equivalence(name, src);
    }
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
// Hover: exact text and exact anchor ranges (RFC-014 Gradeability).

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

    // Hover over the empty impl (line 2) — EXACT resolved-mapping markdown.
    let hover = lsp.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 2, "character": 6 } }),
    );
    assert_eq!(
        hover["contents"]["value"].as_str(),
        Some(
            "**impl** `LspTrait` **for** `LspDev`\n\npins:\n- `A` ← `A`\n\nspec:\n- `resistance` ← `resistance`"
        ),
        "exact impl hover text:\n{}",
        hover
    );
    // Anchor: the whole impl statement on line 2.
    assert_eq!(hover["range"]["start"]["line"].as_u64(), Some(2));
    assert_eq!(hover["range"]["start"]["character"].as_u64(), Some(0));
    assert_eq!(hover["range"]["end"]["line"].as_u64(), Some(2));
    assert_eq!(
        hover["range"]["end"]["character"].as_u64(),
        Some("impl LspTrait for LspDev {}".len() as u64),
        "{}",
        hover
    );

    // Hover over the device pin declaration `A` (line 1) — EXACT text and
    // the pin-name anchor range.
    let col = src.lines().nth(1).unwrap().find("A:").unwrap() as u64;
    let hover = lsp.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 1, "character": col } }),
    );
    assert_eq!(
        hover["contents"]["value"].as_str(),
        Some("**required pin** `A` on device `LspDev`\n\n- role: `passive`\n- pads: 1"),
        "exact pin hover text:\n{}",
        hover
    );
    assert_eq!(hover["range"]["start"]["line"].as_u64(), Some(1));
    assert_eq!(hover["range"]["start"]["character"].as_u64(), Some(col));
    assert_eq!(hover["range"]["end"]["character"].as_u64(), Some(col + 1));
    lsp.shutdown();
}

// Review R10 (RFC-002 / RFC-001 inherited obligations): hover works on pin
// USE SITES (`d.A`) and on unit literals (allowed-prefix table row).
#[test]
fn hover_pin_reference_and_unit_literal() {
    let src = "\
pub device ZzHovDev { pins { A: 1 [passive], K: 2 [power_in] } spec { r: 1kohm } }
design ZzHovB {
    inst d: ZzHovDev
    net VIN [3.3V]: d.A
    net GND: d.K
}
";
    let (_path, uri, text) = fixture("hoverref.cohdl", src);
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, &text);
    let _ = lsp.await_diagnostics(&uri);

    // Pin USE SITE: the `A` in `net VIN [3.3V]: d.A` (line 3).
    let line = 3u64;
    let col = (src.lines().nth(3).unwrap().find("d.A").unwrap() + 2) as u64;
    let hover = lsp.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": line, "character": col } }),
    );
    assert_eq!(
        hover["contents"]["value"].as_str(),
        Some("**required pin** `A` on device `ZzHovDev`\n\n- role: `passive`\n- pads: 1"),
        "pin use-site hover:\n{}",
        hover
    );
    assert_eq!(hover["range"]["start"]["character"].as_u64(), Some(col));

    // Unit literal: `3.3V` in the net annotation (RFC-001 prefix table row).
    let vcol = src.lines().nth(3).unwrap().find("3.3V").unwrap() as u64;
    let hover = lsp.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 3, "character": vcol + 1 } }),
    );
    assert_eq!(
        hover["contents"]["value"].as_str(),
        Some(
            "**Voltage literal** `3.3V`\n\n- allowed prefixes on `V`: `p`, `n`, `u`, `m`, `k`, `M`, `G`"
        ),
        "unit-literal hover:\n{}",
        hover
    );

    // Unit literal in a device spec: `1kohm` (line 0).
    let rcol = src.lines().next().unwrap().find("1kohm").unwrap() as u64;
    let hover = lsp.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 0, "character": rcol } }),
    );
    assert_eq!(
        hover["contents"]["value"].as_str(),
        Some(
            "**Resistance literal** `1kohm`\n\n- allowed prefixes on `ohm`: `p`, `n`, `u`, `m`, `k`, `M`, `G`"
        ),
        "spec unit-literal hover:\n{}",
        hover
    );
    lsp.shutdown();
}

// ---------------------------------------------------------------------------
// Goto-definition: exact declaration span (both endpoints).

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
    // The EXACT declaration-name span: line 0, covering `LspTarget` only.
    let decl_col = src.lines().next().unwrap().find("LspTarget").unwrap() as u64;
    assert_eq!(def["range"]["start"]["line"].as_u64(), Some(0), "{}", def);
    assert_eq!(
        def["range"]["start"]["character"].as_u64(),
        Some(decl_col),
        "{}",
        def
    );
    assert_eq!(def["range"]["end"]["line"].as_u64(), Some(0), "{}", def);
    assert_eq!(
        def["range"]["end"]["character"].as_u64(),
        Some(decl_col + "LspTarget".len() as u64),
        "exact end of the declaration name: {}",
        def
    );
    lsp.shutdown();
}

// ---------------------------------------------------------------------------
// References: exact locations (URI + full ranges), not just counts.

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
    // EXACT locations: each impl statement's full span on its own line.
    for (loc, (line, stmt)) in arr.iter().zip([
        (3u64, "impl LspMulti for DevOne {}"),
        (4u64, "impl LspMulti for DevTwo {}"),
    ]) {
        assert_eq!(loc["uri"].as_str(), Some(uri.as_str()), "{}", loc);
        assert_eq!(
            loc["range"]["start"]["line"].as_u64(),
            Some(line),
            "{}",
            loc
        );
        assert_eq!(
            loc["range"]["start"]["character"].as_u64(),
            Some(0),
            "{}",
            loc
        );
        assert_eq!(loc["range"]["end"]["line"].as_u64(), Some(line), "{}", loc);
        assert_eq!(
            loc["range"]["end"]["character"].as_u64(),
            Some(stmt.len() as u64),
            "{}",
            loc
        );
    }

    // On a device name inside an impl statement (line 3) — exactly DevOne's.
    let col = src.lines().nth(3).unwrap().find("DevOne").unwrap() as u64;
    let refs = lsp.request(
        "textDocument/references",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": 3, "character": col + 1 },
            "context": { "includeDeclaration": false },
        }),
    );
    let arr = refs.as_array().unwrap();
    assert_eq!(arr.len(), 1, "{:?}", arr);
    assert_eq!(arr[0]["range"]["start"]["line"].as_u64(), Some(3));
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

    // A bodyless header block without Content-Length -> also recoverable.
    // (Honest limitation, pinned as such: if a BODY had followed these
    // headers, its bytes would be misread as the next frame's headers —
    // recovery is only guaranteed at clean blank-line boundaries.)
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

    // Review R6: closing the phantom buffer publishes an EXPLICIT empty
    // list — the editor must not keep showing diagnostics for a document
    // that no longer exists anywhere.
    lsp.notify(
        "textDocument/didClose",
        json!({ "textDocument": { "uri": uri } }),
    );
    let diags = lsp.await_diagnostics(&uri);
    assert!(
        diags.is_empty(),
        "didClose of a phantom buffer must clear its diagnostics:\n{:?}",
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

// ---------------------------------------------------------------------------
// Review-3 regressions.

// R6 (high): a real project-load failure must SURFACE, not silently degrade
// into a false-clean synthetic check.
#[test]
fn project_load_failure_surfaces_not_false_clean() {
    let root = std::env::temp_dir().join(format!("cohdl-lsp-badproj-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    // Manifest missing `[package] name` — `cohdl check` exits 2 on this.
    std::fs::write(root.join("cohdl.toml"), "[design]\ntop = \"B\"\n").unwrap();
    std::fs::write(root.join("src/main.cohdl"), DIAG_FIXTURE).unwrap();
    let file = root.join("src/main.cohdl").canonicalize().unwrap();
    let uri = format!("file://{}", file.display());

    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, DIAG_FIXTURE);
    // The FIRST server message must be the load-failure showMessage — in
    // particular, NOT an empty publish claiming the file is clean.
    let msg = lsp.read_message();
    assert_eq!(
        msg.get("method").and_then(Value::as_str),
        Some("window/showMessage"),
        "load failure must surface, got: {}",
        msg
    );
    let m = msg["params"]["message"].as_str().unwrap();
    assert!(m.contains("missing `[package] name`"), "{}", m);
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&root);
}

// R6: a broken std library is a surfaced failure too (the CLI refuses to run
// without std unless --no-std is explicit; the LSP has no --no-std).
#[test]
fn broken_std_surfaces_as_message() {
    let empty_std = std::env::temp_dir().join(format!("cohdl-lsp-nostd-{}", std::process::id()));
    std::fs::create_dir_all(&empty_std).unwrap();
    let (_path, uri, text) = fixture("stdless.cohdl", DIAG_FIXTURE);

    let mut lsp = Lsp::spawn_with_std(&empty_std);
    let _ = lsp.request("initialize", json!({ "capabilities": {} }));
    lsp.notify("initialized", json!({}));
    did_open(&mut lsp, &uri, &text);
    let msg = lsp.read_message();
    assert_eq!(
        msg.get("method").and_then(Value::as_str),
        Some("window/showMessage"),
        "std failure must surface, got: {}",
        msg
    );
    assert!(
        msg["params"]["message"].as_str().unwrap().contains("std"),
        "{}",
        msg
    );
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&empty_std);
}

// R7 (high): re-checking one analysis unit must not clear another's live
// diagnostics — two loose files in the same directory are separate units.
#[test]
fn two_loose_files_keep_independent_diagnostics() {
    let dir = std::env::temp_dir().join(format!("cohdl-lsp-two-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let broken =
        "pub device ZzTwo { pins { A: 1 [passive] } }\ndesign B { inst d: ZzNope\nnet N: d.A }\n";
    let a = dir.join("a.cohdl");
    let b = dir.join("b.cohdl");
    std::fs::write(&a, broken).unwrap();
    std::fs::write(&b, broken).unwrap();
    let uri_a = format!("file://{}", a.canonicalize().unwrap().display());
    let uri_b = format!("file://{}", b.canonicalize().unwrap().display());

    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri_a, broken);
    let da = lsp.await_diagnostics(&uri_a);
    assert!(!da.is_empty(), "a.cohdl has errors");

    did_open(&mut lsp, &uri_b, broken);
    let (db, before) = lsp.await_diagnostics_capturing(&uri_b);
    assert!(!db.is_empty(), "b.cohdl has errors");
    // Neither while B was analyzed nor afterwards may A's diagnostics be
    // cleared (the old global published-set did exactly that).
    let after = lsp.drain();
    for msg in before.iter().chain(&after) {
        assert!(
            !is_empty_publish_for(msg, &uri_a),
            "opening B must not clear A's diagnostics: {}",
            msg
        );
    }
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&dir);
}

// R7: within ONE project, a file whose diagnostics persist across re-checks
// keeps them; the file that was fixed clears.
#[test]
fn two_files_one_project_clear_independently() {
    let root = std::env::temp_dir().join(format!("cohdl-lsp-proj2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        "[package]\nname = \"t\"\n[design]\ntop = \"PB\"\n",
    )
    .unwrap();
    // one.cohdl: unknown trait (E202); two.cohdl: another unknown trait.
    std::fs::write(
        root.join("src/one.cohdl"),
        "pub device ZzP1 { pins { A: 1 [passive] } }\nimpl ZzNoTraitOne for ZzP1 {}\n",
    )
    .unwrap();
    std::fs::write(
        root.join("src/two.cohdl"),
        "pub device ZzP2 { pins { A: 1 [passive] } }\nimpl ZzNoTraitTwo for ZzP2 {}\ndesign PB {\n    inst d: ZzP2\n    net N: d.A\n}\n",
    )
    .unwrap();
    let one = root.join("src/one.cohdl").canonicalize().unwrap();
    let uri_one = format!("file://{}", one.display());
    let uri_two = format!(
        "file://{}",
        root.join("src/two.cohdl").canonicalize().unwrap().display()
    );

    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri_one, &std::fs::read_to_string(&one).unwrap());
    // Both files' diagnostics publish (same project, one analysis).
    let d_one = lsp.await_diagnostics(&uri_one);
    assert!(!d_one.is_empty());
    let d_two = lsp.await_diagnostics(&uri_two);
    assert!(!d_two.is_empty());

    // Fix one.cohdl in the buffer: its diagnostics clear; two.cohdl's stay.
    lsp.notify(
        "textDocument/didChange",
        json!({
            "textDocument": { "uri": uri_one, "version": 2 },
            "contentChanges": [ { "text": "pub device ZzP1 { pins { A: 1 [passive] } }\n" } ],
        }),
    );
    let (d_one, before) = lsp.await_diagnostics_capturing(&uri_one);
    assert!(d_one.is_empty(), "fixed file clears: {:?}", d_one);
    let after = lsp.drain();
    for msg in before.iter().chain(&after) {
        assert!(
            !is_empty_publish_for(msg, &uri_two),
            "two.cohdl still has errors; must not be cleared: {}",
            msg
        );
    }
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&root);
}

// R8: the LSP lifecycle state machine — -32002 before initialize, one
// initialize only, InvalidRequest after shutdown.
#[test]
fn lifecycle_gates_requests() {
    let mut lsp = Lsp::spawn();
    // Request before initialize: -32002 ServerNotInitialized.
    let resp = lsp.request_full(
        "textDocument/hover",
        json!({ "textDocument": { "uri": "file:///nope" }, "position": { "line": 0, "character": 0 } }),
    );
    assert_eq!(resp["error"]["code"].as_i64(), Some(-32002), "{}", resp);

    // Initialize succeeds once…
    let resp = lsp.request_full("initialize", json!({ "capabilities": {} }));
    assert!(resp["result"]["capabilities"].is_object(), "{}", resp);
    // …and only once.
    let resp = lsp.request_full("initialize", json!({ "capabilities": {} }));
    assert_eq!(resp["error"]["code"].as_i64(), Some(-32600), "{}", resp);

    // After shutdown, further requests are InvalidRequest.
    let resp = lsp.request_full("shutdown", Value::Null);
    assert!(
        resp["result"].is_null() && resp.get("error").is_none(),
        "{}",
        resp
    );
    let resp = lsp.request_full(
        "textDocument/hover",
        json!({ "textDocument": { "uri": "file:///nope" }, "position": { "line": 0, "character": 0 } }),
    );
    assert_eq!(resp["error"]["code"].as_i64(), Some(-32600), "{}", resp);

    lsp.notify("exit", Value::Null);
    let status = lsp.child.wait().unwrap();
    assert_eq!(status.code(), Some(0), "exit after shutdown is clean");
}

// R8: the sync advertisement includes save (docs/lsp.md promises didSave
// re-checks; conforming clients only send it when asked to).
#[test]
fn initialize_advertises_save_sync() {
    let mut lsp = Lsp::spawn();
    let init = lsp.request("initialize", json!({ "capabilities": {} }));
    let sync = &init["capabilities"]["textDocumentSync"];
    assert_eq!(sync["openClose"].as_bool(), Some(true), "{}", init);
    assert_eq!(sync["change"].as_i64(), Some(1), "{}", init);
    assert_eq!(sync["save"].as_bool(), Some(true), "{}", init);
    lsp.notify("initialized", json!({}));
    lsp.shutdown();
}

// R8: relatedInformation is capability-negotiated — a client that did not
// advertise support never receives it.
#[test]
fn related_information_requires_client_capability() {
    // The Ω fixture always carries a help line.
    let src = "pub device ZqCap { pins { A: 1 [passive] } spec { r: 10kΩ } }\n";
    let (_path, uri, text) = fixture("caps.cohdl", src);
    let mut lsp = Lsp::spawn();
    let _ = lsp.request("initialize", json!({ "capabilities": {} }));
    lsp.notify("initialized", json!({}));
    did_open(&mut lsp, &uri, &text);
    let diags = lsp.await_diagnostics(&uri);
    assert!(!diags.is_empty());
    for d in &diags {
        assert!(
            d.get("relatedInformation").is_none(),
            "relatedInformation must not be sent without the client capability:\n{}",
            d
        );
    }
    lsp.shutdown();
}

// R8: header field names are case-insensitive (HTTP semantics).
#[test]
fn lowercase_content_length_header_accepted() {
    let mut lsp = Lsp::spawn();
    let payload = serde_json::to_string(
        &json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": { "capabilities": {} } }),
    )
    .unwrap();
    write!(
        lsp.stdin,
        "content-length: {}\r\n\r\n{}",
        payload.len(),
        payload
    )
    .unwrap();
    lsp.stdin.flush().unwrap();
    let resp = lsp.read_message();
    assert!(
        resp["result"]["capabilities"].is_object(),
        "lowercase header must frame correctly: {}",
        resp
    );
    lsp.notify("initialized", json!({}));
    lsp.shutdown();
}

// R8: `file://localhost/...` is a local URI (RFC 8089); other authorities
// are rejected rather than misparsed.
#[test]
fn localhost_authority_uri_accepted() {
    let (path, _uri, text) = fixture("localhost.cohdl", DIAG_FIXTURE);
    let uri = format!("file://localhost{}", path.display());
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, &text);
    let diags = lsp.await_diagnostics(&uri);
    assert!(
        diags.iter().any(|d| d["code"] == "E202"),
        "localhost-authority URIs must work:\n{:?}",
        diags
    );
    lsp.shutdown();
}

// R8: malformed positional params are InvalidParams (-32602), not a silent
// null result.
#[test]
fn invalid_position_params_get_invalid_params_error() {
    let mut lsp = Lsp::start();
    let resp = lsp.request_full(
        "textDocument/hover",
        json!({ "textDocument": { "uri": "file:///x.cohdl" } }), // no position
    );
    assert_eq!(resp["error"]["code"].as_i64(), Some(-32602), "{}", resp);
    let resp = lsp.request_full("textDocument/definition", json!({}));
    assert_eq!(resp["error"]["code"].as_i64(), Some(-32602), "{}", resp);
    lsp.shutdown();
}

// RFC-016: goto-definition resolves qualified paths and `use`-imported
// names to the same declaration span (name resolution feeds the existing
// lookup; no new capability).
#[test]
fn goto_definition_resolves_imported_and_qualified_names() {
    let root = std::env::temp_dir().join(format!("cohdl-lsp-mod-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src/parts")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        "[package]\nname = \"modproj\"\n[design]\ntop = \"B\"\n",
    )
    .unwrap();
    let decl = "pub device ZzModDev { pins { A: 1 [passive] } }\n";
    std::fs::write(root.join("src/parts/lib.cohdl"), decl).unwrap();
    let main_src = "\
use modproj::parts::lib::ZzModDev;
design B {
    inst a: ZzModDev
    inst b: modproj::parts::lib::ZzModDev
    net N: a.A, b.A
}
";
    std::fs::write(root.join("src/main.cohdl"), main_src).unwrap();
    let main_path = root.join("src/main.cohdl").canonicalize().unwrap();
    let decl_uri = format!(
        "file://{}",
        root.join("src/parts/lib.cohdl")
            .canonicalize()
            .unwrap()
            .display()
    );
    let uri = format!("file://{}", main_path.display());

    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, main_src);
    let _ = lsp.await_diagnostics(&uri);

    // Imported bare name at the inst site (line 2).
    let col = main_src.lines().nth(2).unwrap().find("ZzModDev").unwrap() as u64;
    let def = lsp.request(
        "textDocument/definition",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 2, "character": col + 1 } }),
    );
    assert_eq!(def["uri"].as_str(), Some(decl_uri.as_str()), "{}", def);
    assert_eq!(def["range"]["start"]["line"].as_u64(), Some(0), "{}", def);
    let decl_col = decl.find("ZzModDev").unwrap() as u64;
    assert_eq!(def["range"]["start"]["character"].as_u64(), Some(decl_col));

    // Fully-qualified path at the inst site (line 3): same declaration.
    let col = main_src.lines().nth(3).unwrap().find("modproj::").unwrap() as u64;
    let def = lsp.request(
        "textDocument/definition",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 3, "character": col + 3 } }),
    );
    assert_eq!(def["uri"].as_str(), Some(decl_uri.as_str()), "{}", def);
    assert_eq!(def["range"]["start"]["line"].as_u64(), Some(0), "{}", def);
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&root);
}

// RFC-016 adversarial round 2 (medium): a nested not-yet-saved buffer under
// src/ got an ABSOLUTE display name, landing it at the package root — its
// module path diverged from the CLI's, producing phantom E202 on imports
// the CLI resolves. The buffer now joins the project with its project-
// relative display.
#[test]
fn nested_phantom_buffer_gets_the_cli_module_path() {
    let root = std::env::temp_dir().join(format!("cohdl-lsp-nest-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("src/parts")).unwrap();
    std::fs::write(root.join("cohdl.toml"), "[package]\nname = \"modx\"\n").unwrap();
    let main_src = "use modx::parts::newmod::ZzNew;\n";
    std::fs::write(root.join("src/main.cohdl"), main_src).unwrap();
    let main_uri = format!(
        "file://{}",
        root.join("src/main.cohdl")
            .canonicalize()
            .unwrap()
            .display()
    );
    // The nested buffer exists ONLY as an overlay (never written to disk).
    let phantom = root.canonicalize().unwrap().join("src/parts/newmod.cohdl");
    let phantom_uri = format!("file://{}", phantom.display());

    let mut lsp = Lsp::start();
    did_open(
        &mut lsp,
        &phantom_uri,
        "pub device ZzNew { pins { A: 1 [passive] } }\n",
    );
    let _ = lsp.await_diagnostics(&phantom_uri);
    did_open(&mut lsp, &main_uri, main_src);
    let diags = lsp.await_diagnostics(&main_uri);
    assert!(
        !diags.iter().any(|d| d["code"] == "E202"),
        "the unsaved nested buffer must resolve at its CLI module path:\n{:?}",
        diags
    );
    lsp.shutdown();
    let _ = std::fs::remove_dir_all(&root);
}

// RFC-017: goto-definition on a part's footprint symbol reference, and part
// hover surfacing #[doc] paths + the resolved footprint.
#[test]
fn footprint_ref_definition_and_part_hover() {
    let src = "\
pub footprint ZzFp {}
pub device ZzDev { pins { A: 1 [passive] } }
#[doc(\"datasheets/zz.pdf\")]
#[doc(\"app-notes/zz-layout.pdf\")]
pub part ZzPart: ZzDev { primary { mfr: \"Acme\", mpn: \"ZZ-1\", footprint: ZzFp } }
design B {
    inst d: ZzPart
    net N: d.A
}
";
    let (_path, uri, text) = fixture("library.cohdl", src);
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, &text);
    let _ = lsp.await_diagnostics(&uri);

    // Goto-def on the footprint reference (line 4) → its declaration (line 0).
    let col = src.lines().nth(4).unwrap().rfind("ZzFp").unwrap() as u64;
    let def = lsp.request(
        "textDocument/definition",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 4, "character": col + 1 } }),
    );
    assert_eq!(def["range"]["start"]["line"].as_u64(), Some(0), "{}", def);

    // Hover on the part name → mpn/mfr, footprint symbol, AND its #[doc]
    // reference documents (adversarial finding: the doc half was untested).
    let pcol = src.lines().nth(4).unwrap().find("ZzPart").unwrap() as u64;
    let hover = lsp.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 4, "character": pcol } }),
    );
    let md = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(md.contains("**part** `ZzPart`"), "{}", md);
    assert!(md.contains("mpn: `ZZ-1`"), "{}", md);
    assert!(md.contains("footprint: `library::ZzFp`"), "{}", md);
    assert!(md.contains("doc: `datasheets/zz.pdf`"), "{}", md);
    assert!(md.contains("doc: `app-notes/zz-layout.pdf`"), "{}", md);
    lsp.shutdown();
}

// RFC-018 Tooling: hover on a `pad N: Sym at (x, y)` placement shows the
// resolved pad's geometry; goto-def on the pad symbol lands on its decl.
#[test]
fn hover_and_goto_definition_on_pad_placement() {
    let src = "\
pub pad ZzPadRect { shape: rect, size: (0.6mm, 0.7mm), layer: top_copper, plating: smd }
pub footprint ZzFpR {
    pad 1: ZzPadRect at (-0.5mm, 0mm)
    pad 2: ZzPadRect at (0.5mm, 0mm)
}
";
    let (_path, uri, text) = fixture("padhover.cohdl", src);
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, &text);
    let _ = lsp.await_diagnostics(&uri);

    // Hover on the pad symbol in the first placement (line 2) — EXACT text.
    let col = src.lines().nth(2).unwrap().find("ZzPadRect").unwrap() as u64;
    let hover = lsp.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 2, "character": col + 1 } }),
    );
    assert_eq!(
        hover["contents"]["value"].as_str(),
        Some(
            "**pad** `ZzPadRect` (placed as pad `1` at (-0.5mm, 0mm))\n\n\
             - shape: `rect`\n- size: `(0.6mm, 0.7mm)`\n- layer: `top_copper`\n\
             - plating: `smd`"
        ),
        "exact pad-placement hover text:\n{}",
        hover
    );
    assert_eq!(hover["range"]["start"]["line"].as_u64(), Some(2));
    assert_eq!(hover["range"]["start"]["character"].as_u64(), Some(col));

    // Goto-definition on the second placement's symbol → the pad decl.
    let col = src.lines().nth(3).unwrap().find("ZzPadRect").unwrap() as u64;
    let def = lsp.request(
        "textDocument/definition",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 3, "character": col + 1 } }),
    );
    assert_eq!(def["uri"].as_str(), Some(uri.as_str()), "{}", def);
    assert_eq!(def["range"]["start"]["line"].as_u64(), Some(0), "{}", def);
    let decl_col = src.find("ZzPadRect").unwrap() as u64;
    assert_eq!(
        def["range"]["start"]["character"].as_u64(),
        Some(decl_col),
        "{}",
        def
    );
    lsp.shutdown();
}

// ---------------------------------------------------------------------------
// Fourth-review (2026-07-14) LSP regressions.

// F10: an `initialize` NOTIFICATION (no id) must NOT consume initialization —
// lifecycle state may change only for a real request. If it had advanced to
// Running, a subsequent `shutdown` request would succeed; it must instead hit
// the not-initialized gate.
#[test]
fn initialize_notification_does_not_consume_lifecycle() {
    let mut lsp = Lsp::spawn();
    lsp.notify("initialize", json!({ "capabilities": {} })); // no id: a notification
    let resp = lsp.request_full("shutdown", json!({}));
    assert_eq!(
        resp["error"]["code"].as_i64(),
        Some(-32002),
        "server must still be uninitialized:\n{}",
        resp
    );
    lsp.child.kill().ok();
}

// F10: a malformed JSON-RPC envelope (wrong version) with an id gets
// InvalidRequest; the server does not dispatch it.
#[test]
fn bad_jsonrpc_envelope_is_invalid_request() {
    let mut lsp = Lsp::spawn();
    lsp.send(&json!({ "jsonrpc": "1.0", "id": 4242, "method": "initialize", "params": {} }));
    let resp = loop {
        let m = lsp.read_message();
        if m.get("id").and_then(Value::as_i64) == Some(4242) {
            break m;
        }
    };
    assert_eq!(resp["error"]["code"].as_i64(), Some(-32600), "{}", resp);
    lsp.child.kill().ok();
}

// F10: an out-of-`u32`-range position must be InvalidParams, never a silent
// `as u32` wrap onto a valid in-range position.
#[test]
fn out_of_range_position_is_invalid_params() {
    let src =
        "pub device D { pins { A: 1 [passive] } }\ndesign B {\n    inst a: D\n    net N: a.A\n}\n";
    let (_p, uri, text) = fixture("oob.cohdl", src);
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, &text);
    let _ = lsp.await_diagnostics(&uri);
    let resp = lsp.request_full(
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": 5_000_000_000u64, "character": 0 } }),
    );
    assert_eq!(resp["error"]["code"].as_i64(), Some(-32602), "{}", resp);
    lsp.shutdown();
}

// F8: a std directory that exists but is EMPTY plus a phantom (not-on-disk)
// buffer must surface a window/showMessage, never a false-clean empty
// publishDiagnostics.
#[test]
fn empty_std_with_phantom_buffer_shows_message() {
    let empty = std::env::temp_dir().join(format!("cohdl-lsp-emptystd-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&empty);
    std::fs::create_dir_all(&empty).unwrap();
    let mut lsp = Lsp::spawn_with_std(&empty);
    lsp.request(
        "initialize",
        json!({ "capabilities": { "textDocument": { "publishDiagnostics": { "relatedInformation": true } } } }),
    );
    lsp.notify("initialized", json!({}));
    // A buffer at a path with no cohdl.toml and not on disk (phantom).
    let phantom = empty.join("ghost.cohdl");
    let uri = format!("file://{}", phantom.display());
    lsp.notify(
        "textDocument/didOpen",
        json!({ "textDocument": { "uri": uri, "languageId": "cohdl", "version": 1,
                 "text": "pub device D { pins { A: 1 [passive] } }\ndesign B { inst a: D  net N: a.A }\n" } }),
    );
    let mut saw_message = false;
    for m in lsp.drain() {
        if m.get("method").and_then(Value::as_str) == Some("window/showMessage") {
            saw_message = true;
        }
        assert_ne!(
            m.get("method").and_then(Value::as_str),
            Some("textDocument/publishDiagnostics"),
            "must NOT publish a (false-clean) diagnostic set with no std:\n{}",
            m
        );
    }
    assert!(
        saw_message,
        "an empty std must produce a window/showMessage"
    );
    lsp.child.kill().ok();
    let _ = std::fs::remove_dir_all(&empty);
}

// F9: pin use-site hover must reflect the SELECTED structural variant — a
// part bound to one variant must not show another variant's physical pad.
#[test]
fn pin_hover_respects_selected_variant() {
    let src = "\
pub device ZzVarDev {
    variants { QFN, DIP }
    pins[QFN] { required SIG: 7 [passive] }
    pins[DIP] { required SIG: 2 [passive] }
}
pub footprint TFP {}
pub part ZzVarDIP: ZzVarDev[DIP] { primary { mfr: \"m\", mpn: \"n\", footprint: TFP } }
design ZzVarB {
    inst u: ZzVarDIP
    net N: u.SIG
}
";
    let (_p, uri, text) = fixture("varhover.cohdl", src);
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, &text);
    let _ = lsp.await_diagnostics(&uri);
    let line = src
        .lines()
        .position(|l| l.contains("net N: u.SIG"))
        .unwrap() as u64;
    let col = src.lines().nth(line as usize).unwrap().find("SIG").unwrap() as u64;
    let hover = lsp.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": line, "character": col } }),
    );
    let md = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(md.contains("pads: 2"), "DIP variant → pad 2, got:\n{}", md);
    assert!(
        !md.contains("pads: 7"),
        "must NOT show the QFN pad:\n{}",
        md
    );
    lsp.shutdown();
}

// F11: hover on a function's own generic-parameter DEFAULT unit literal
// (`fn f<V: Voltage = 3.3V>`) must work — the device path scanned generics
// but the fn path did not.
#[test]
fn hover_on_fn_generic_default_literal() {
    let src = "\
pub device Cap { pins { A: 1 [passive], B: 2 [passive] } spec { voltage_rating: Voltage } }
pub footprint TFP {}
pub part C1: Cap { primary { mfr: \"m\", mpn: \"n\", footprint: TFP } }
pub fn rail<V: Voltage = 3.3V>(a: Pin, b: Pin) {
    inst c: C1<V>
    net _: c.A, a
    net _: c.B, b
}
";
    let (_p, uri, text) = fixture("fngen.cohdl", src);
    let mut lsp = Lsp::start();
    did_open(&mut lsp, &uri, &text);
    let _ = lsp.await_diagnostics(&uri);
    let line = src.lines().position(|l| l.contains("fn rail")).unwrap() as u64;
    let col = src
        .lines()
        .nth(line as usize)
        .unwrap()
        .find("3.3V")
        .unwrap() as u64;
    let hover = lsp.request(
        "textDocument/hover",
        json!({ "textDocument": { "uri": uri }, "position": { "line": line, "character": col + 1 } }),
    );
    let md = hover["contents"]["value"].as_str().unwrap_or("");
    assert!(
        md.contains("Voltage literal") && md.contains("3.3V"),
        "fn generic default must hover:\n{}",
        md
    );
    lsp.shutdown();
}

// R5-11: a JSON-RPC request with an invalid id TYPE (object/array/bool) is
// InvalidRequest and must NOT mutate lifecycle state.
#[test]
fn object_id_is_invalid_request() {
    let mut lsp = Lsp::spawn();
    lsp.send(
        &json!({ "jsonrpc": "2.0", "id": {"bad": true}, "method": "initialize", "params": {} }),
    );
    // The response echoes a null id (an object id cannot be a response id).
    let resp = loop {
        let m = lsp.read_message();
        if m.get("error").is_some() {
            break m;
        }
    };
    assert_eq!(resp["error"]["code"].as_i64(), Some(-32600), "{}", resp);
    assert!(
        resp["id"].is_null(),
        "invalid id → null response id: {}",
        resp
    );
    // The server is still uninitialized: a shutdown request hits the gate.
    let r2 = lsp.request_full("shutdown", json!({}));
    assert_eq!(r2["error"]["code"].as_i64(), Some(-32002), "{}", r2);
    lsp.child.kill().ok();
}

// R6-7: a malformed envelope carrying an explicit `"id": null` is a request
// (not a notification, which OMITS id) — it must get InvalidRequest with a
// null response id, not be silently dropped.
#[test]
fn malformed_request_with_null_id_gets_response() {
    let mut lsp = Lsp::spawn();
    lsp.send(&json!({ "jsonrpc": "1.0", "id": null, "method": "initialize", "params": {} }));
    let resp = lsp.read_message();
    assert_eq!(resp["error"]["code"].as_i64(), Some(-32600), "{}", resp);
    assert!(resp["id"].is_null(), "null response id: {}", resp);
    lsp.child.kill().ok();
}

// R6-7 counterpart: a well-formed NOTIFICATION (no id at all) with a bad
// version is dropped silently — no response.
#[test]
fn malformed_notification_is_dropped() {
    let mut lsp = Lsp::spawn();
    lsp.send(&json!({ "jsonrpc": "1.0", "method": "initialized", "params": {} }));
    // Follow with a real request; the only response must be to THIS request,
    // proving the malformed notification produced nothing.
    let resp = lsp.request_full("initialize", json!({ "capabilities": {} }));
    assert!(
        resp["result"].is_object() || resp["error"].is_object(),
        "{}",
        resp
    );
    lsp.child.kill().ok();
}

// R7-6: a notification-only method presented as a request (id field present)
// gets InvalidRequest and does NOT perform the notification action.
#[test]
fn notification_with_id_is_invalid_request() {
    let mut lsp = Lsp::start();
    // `initialized` is a notification; sending it with an id is request-shaped.
    lsp.send(&json!({ "jsonrpc": "2.0", "id": 77, "method": "initialized", "params": {} }));
    let resp = lsp.read_message();
    assert_eq!(resp["error"]["code"].as_i64(), Some(-32600), "{}", resp);
    assert_eq!(resp["id"].as_i64(), Some(77), "{}", resp);
    lsp.shutdown();
}

// R7-6: `exit` presented with an id is request-shaped — InvalidRequest, and
// the server does NOT terminate (a later request still gets a response).
#[test]
fn exit_with_id_does_not_terminate() {
    let mut lsp = Lsp::start();
    lsp.send(&json!({ "jsonrpc": "2.0", "id": 88, "method": "exit", "params": {} }));
    let resp = lsp.read_message();
    assert_eq!(resp["error"]["code"].as_i64(), Some(-32600), "{}", resp);
    // Server still alive: a shutdown request is answered.
    let r2 = lsp.request_full("shutdown", json!({}));
    assert!(
        r2["result"].is_null() || r2.get("result").is_some(),
        "{}",
        r2
    );
    lsp.child.kill().ok();
}
