//! RFC-010: structured diagnostics — `cohdl check --json` / `cohdl build --json`.
//!
//! A versioned, byte-stable JSON re-projection of the exact `Diagnostic`
//! values the pipeline already produced. Nothing is invented: every field maps
//! 1:1 onto the real `Diagnostic`/`Label`/`Span` types (src/diag.rs,
//! src/span.rs), with spans resolved to 1-based line/col via the existing
//! `SourceMap`. The plain-text renderer and this emitter read the *same*
//! `Diagnostics` list, so they can never disagree on which diagnostics exist —
//! the RFC-010 equivalence guarantee, checked in tests/json_output.rs.
//!
//! Hand-rolled (zero external dependencies, per the project constitution) and
//! deterministic: `checked.diags` is already sorted, so same source → same
//! bytes.

use crate::diag::{Label, Severity};
use crate::pipeline::Checked;
use crate::span::{SourceMap, Span};
use std::fmt::Write as _;

/// Bumped only on a breaking change to this schema's *shape* — never for new
/// diagnostic codes or messages (those are ordinary content).
pub const SCHEMA_VERSION: u32 = 1;

/// Emitted-artifact paths for `build --json`, present only on a passing build.
pub struct BuildArtifacts {
    pub netlist: String,
    pub bom: String,
}

/// A span resolved to the JSON schema's location shape (1-based line/col).
pub struct JsonLoc {
    pub file: String,
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
    pub message: String,
}

/// One diagnostic in the schema's shape — the structured model the JSON string
/// is serialized from, exposed so the equivalence test can compare it directly
/// against the plain-text renderer's view of the same `Diagnostic`s.
pub struct JsonDiag {
    pub code: &'static str,
    pub severity: &'static str,
    pub message: String,
    pub primary: JsonLoc,
    pub secondary: Vec<JsonLoc>,
    pub help: Vec<String>,
}

fn loc(sm: &SourceMap, span: Span, message: &str) -> JsonLoc {
    let start = sm.line_col(span.file, span.start);
    let end = sm.line_col(span.file, span.end);
    JsonLoc {
        file: sm.name(span.file).to_string(),
        start_line: start.line,
        start_col: start.col,
        end_line: end.line,
        end_col: end.col,
        message: message.to_string(),
    }
}

fn label_loc(sm: &SourceMap, label: &Label) -> JsonLoc {
    loc(sm, label.span, &label.message)
}

/// The structured diagnostic model, one entry per pipeline `Diagnostic`, in the
/// same order the plain-text renderer emits them.
pub fn model(checked: &Checked) -> Vec<JsonDiag> {
    checked
        .diags
        .iter()
        .map(|d| JsonDiag {
            code: d.code,
            severity: match d.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            },
            message: d.message.clone(),
            primary: label_loc(&checked.sm, &d.primary),
            secondary: d.secondary.iter().map(|l| label_loc(&checked.sm, l)).collect(),
            help: d.help.clone(),
        })
        .collect()
}

/// The verdict string, computed identically to the CLI's exit-code logic:
/// any error-severity diagnostic ⇒ "fail".
pub fn verdict(checked: &Checked) -> &'static str {
    if checked.diags.has_errors() {
        "fail"
    } else {
        "pass"
    }
}

/// Render the full `--json` document. `build` is `Some` only for a passing
/// `build` invocation (adds the `build` object naming emitted artifacts).
pub fn render(checked: &Checked, build: Option<&BuildArtifacts>) -> String {
    let diags = model(checked);
    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(out, "  \"schema_version\": {},", SCHEMA_VERSION);
    let _ = writeln!(out, "  \"verdict\": \"{}\",", verdict(checked));

    if diags.is_empty() {
        out.push_str("  \"diagnostics\": []");
    } else {
        out.push_str("  \"diagnostics\": [\n");
        for (i, d) in diags.iter().enumerate() {
            write_diag(&mut out, d);
            out.push_str(if i + 1 < diags.len() { ",\n" } else { "\n" });
        }
        out.push_str("  ]");
    }

    if let Some(b) = build {
        out.push_str(",\n  \"build\": {\n");
        let _ = writeln!(out, "    \"netlist\": {},", json_str(&b.netlist));
        let _ = writeln!(out, "    \"bom\": {}", json_str(&b.bom));
        out.push_str("  }");
    }
    out.push_str("\n}\n");
    out
}

fn write_diag(out: &mut String, d: &JsonDiag) {
    out.push_str("    {\n");
    let _ = writeln!(out, "      \"code\": {},", json_str(d.code));
    let _ = writeln!(out, "      \"severity\": {},", json_str(d.severity));
    let _ = writeln!(out, "      \"message\": {},", json_str(&d.message));
    out.push_str("      \"primary\": ");
    write_loc(out, &d.primary, 6);
    out.push_str(",\n");
    if d.secondary.is_empty() {
        out.push_str("      \"secondary\": [],\n");
    } else {
        out.push_str("      \"secondary\": [\n");
        for (i, l) in d.secondary.iter().enumerate() {
            out.push_str("        ");
            write_loc(out, l, 8);
            out.push_str(if i + 1 < d.secondary.len() { ",\n" } else { "\n" });
        }
        out.push_str("      ],\n");
    }
    if d.help.is_empty() {
        out.push_str("      \"help\": []\n");
    } else {
        out.push_str("      \"help\": [\n");
        for (i, h) in d.help.iter().enumerate() {
            let _ = write!(out, "        {}", json_str(h));
            out.push_str(if i + 1 < d.help.len() { ",\n" } else { "\n" });
        }
        out.push_str("      ]\n");
    }
    out.push_str("    }");
}

/// Serialize a location object. `indent` is the column the opening brace sits
/// at; nested fields indent two further.
fn write_loc(out: &mut String, l: &JsonLoc, indent: usize) {
    let pad = " ".repeat(indent + 2);
    out.push_str("{\n");
    let _ = writeln!(out, "{}\"file\": {},", pad, json_str(&l.file));
    let _ = writeln!(out, "{}\"start_line\": {},", pad, l.start_line);
    let _ = writeln!(out, "{}\"start_col\": {},", pad, l.start_col);
    let _ = writeln!(out, "{}\"end_line\": {},", pad, l.end_line);
    let _ = writeln!(out, "{}\"end_col\": {},", pad, l.end_col);
    let _ = writeln!(out, "{}\"message\": {}", pad, json_str(&l.message));
    let _ = write!(out, "{}}}", " ".repeat(indent));
}

/// A JSON string literal, escaped per RFC 8259.
fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
