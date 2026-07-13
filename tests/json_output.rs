//! RFC-010 equivalence: `--json` output must report the identical diagnostic
//! set as the plain-text renderer, field-for-field.
//!
//! Both renderers read the same `Diagnostics` list, so the load-bearing check
//! is that they agree on every diagnostic's code, severity, message, and
//! primary span — independently parsed out of the plain-text render and
//! compared against the JSON model. Any divergence is a bug in one renderer.

use cohdl::emit::json;
use cohdl::pipeline::{check_files, Checked};

fn check(src: &str) -> Checked {
    let files = vec![("fixture.cohdl".to_string(), src.to_string())];
    let mut checked = check_files(&files, None).expect("design selection");
    checked.diags.sort(&checked.sm);
    checked
}

/// One diagnostic as recovered from the plain-text render: severity, code,
/// message, and primary file/line/col.
#[derive(Debug, PartialEq)]
struct TextDiag {
    severity: String,
    code: String,
    message: String,
    file: String,
    line: u32,
    col: u32,
}

/// Parse the plain-text render back into structured diagnostics, in order.
fn parse_text(rendered: &str) -> Vec<TextDiag> {
    let mut out = Vec::new();
    let mut lines = rendered.lines().peekable();
    while let Some(line) = lines.next() {
        // Header: `error[E202]: message` / `warning[D003]: message`.
        let Some((severity, rest)) = line
            .strip_prefix("error[")
            .map(|r| ("error", r))
            .or_else(|| line.strip_prefix("warning[").map(|r| ("warning", r)))
        else {
            continue;
        };
        let (code, message) = rest.split_once("]: ").expect("header shape");
        // Next line is the primary arrow: ` --> file:line:col`.
        let arrow = lines.next().expect("arrow line");
        let loc = arrow
            .trim_start()
            .strip_prefix("--> ")
            .expect("arrow prefix");
        // Split off col and line from the right (file may itself be pathy).
        let (rest, col) = loc.rsplit_once(':').expect("col");
        let (file, l) = rest.rsplit_once(':').expect("line");
        out.push(TextDiag {
            severity: severity.to_string(),
            code: code.to_string(),
            message: message.to_string(),
            file: file.to_string(),
            line: l.parse().expect("line num"),
            col: col.parse().expect("col num"),
        });
    }
    out
}

/// The JSON model, projected into the same `TextDiag` shape for comparison.
fn json_as_text(checked: &Checked) -> Vec<TextDiag> {
    json::model(checked)
        .into_iter()
        .map(|d| TextDiag {
            severity: d.severity.to_string(),
            code: d.code.to_string(),
            message: d.message,
            file: d.primary.file,
            line: d.primary.start_line,
            col: d.primary.start_col,
        })
        .collect()
}

fn assert_equivalent(src: &str) {
    let checked = check(src);
    let from_text = parse_text(&checked.diags.render(&checked.sm));
    let from_json = json_as_text(&checked);
    assert_eq!(
        from_json, from_text,
        "JSON model diverged from plain-text render for source:\n{}",
        src
    );
    // Verdict is computed identically to the CLI exit-code logic.
    let expect = if checked.diags.has_errors() {
        "fail"
    } else {
        "pass"
    };
    assert_eq!(json::verdict(&checked), expect);
}

#[test]
fn equivalence_across_error_fixtures() {
    // A spread of mechanisms: resolution, unit-type, generics, variants, DRC.
    let fixtures = [
        // clean (declarations only)
        "pub trait T { pins { required A: pin } }",
        // unknown name + wrong-unit net annotation + unknown instance
        "design B {\n    inst c: Foo\n    net V [3.3F]: c.A\n}",
        // wrong unit at a generic site (E112) + bare number (separate fixture)
        "pub device M<C: Capacitance> {\n    pins { A: 1 [passive], B: 2 [passive] }\n    spec { capacitance: C }\n}\ndesign B {\n    inst c: M<16V>\n    net X: c.A, c.B\n}",
        // bare number as generic arg (E113)
        "pub device M<C: Capacitance> {\n    pins { A: 1 [passive], B: 2 [passive] }\n    spec { capacitance: C }\n}\ndesign B {\n    inst c: M<100>\n    net X: c.A, c.B\n}",
        // RFC-008: variant selector omitted (E904)
        "pub device V {\n    variants { X, Y }\n    pins[X] { A: 1 [passive] }\n    pins[Y] { A: 1 [passive] }\n}\ndesign B {\n    inst v: V\n    net N: v.A\n}",
        // missing pin role (E901)
        "pub device D {\n    pins { A: 1 }\n}",
    ];
    for src in fixtures {
        assert_equivalent(src);
    }
}

#[test]
fn schema_shape_and_escaping() {
    // A message that contains a backtick and a quote-ish glyph must serialize
    // as valid JSON with the code/verdict fields present.
    let checked = check("design B {\n    inst c: Foo\n    net V: c.A\n}");
    let doc = json::render(&checked, None);
    assert!(doc.starts_with("{\n  \"schema_version\": 1,\n"), "{}", doc);
    assert!(doc.contains("\"verdict\": \"fail\""), "{}", doc);
    assert!(doc.contains("\"code\": \"E202\""), "{}", doc);
    // Backticks in messages are fine as-is; embedded double quotes are escaped.
    assert!(!doc.contains("\"message\": \"\"unterminated"), "{}", doc);
    // Balanced braces (a cheap structural sanity check).
    let opens = doc.matches('{').count();
    let closes = doc.matches('}').count();
    assert_eq!(opens, closes, "unbalanced braces:\n{}", doc);
}

#[test]
fn clean_design_reports_pass_and_empty_diagnostics() {
    let checked = check("pub trait T { pins { required A: pin } }");
    let doc = json::render(&checked, None);
    assert!(doc.contains("\"verdict\": \"pass\""), "{}", doc);
    assert!(doc.contains("\"diagnostics\": []"), "{}", doc);
}
