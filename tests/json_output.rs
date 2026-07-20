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

// ---------------------------------------------------------------------------
// A minimal JSON parser (no external deps) so the document is actually
// DECODED, not merely grepped — the review flagged brace-counting as too weak.

#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Obj(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    fn as_str(&self) -> &str {
        match self {
            Json::Str(s) => s,
            _ => panic!("expected string, got {:?}", self),
        }
    }
    fn as_u32(&self) -> u32 {
        match self {
            Json::Num(n) => *n as u32,
            _ => panic!("expected number, got {:?}", self),
        }
    }
    fn as_arr(&self) -> &[Json] {
        match self {
            Json::Arr(a) => a,
            _ => panic!("expected array, got {:?}", self),
        }
    }
}

fn parse_json(text: &str) -> Json {
    let bytes: Vec<char> = text.chars().collect();
    let mut pos = 0usize;
    let v = parse_value(&bytes, &mut pos);
    skip_ws(&bytes, &mut pos);
    assert_eq!(pos, bytes.len(), "trailing garbage after JSON document");
    v
}

fn skip_ws(b: &[char], pos: &mut usize) {
    while *pos < b.len() && b[*pos].is_whitespace() {
        *pos += 1;
    }
}

fn parse_value(b: &[char], pos: &mut usize) -> Json {
    skip_ws(b, pos);
    match b[*pos] {
        '{' => {
            *pos += 1;
            let mut fields = Vec::new();
            skip_ws(b, pos);
            if b[*pos] == '}' {
                *pos += 1;
                return Json::Obj(fields);
            }
            loop {
                skip_ws(b, pos);
                let key = match parse_value(b, pos) {
                    Json::Str(s) => s,
                    other => panic!("object key must be a string, got {:?}", other),
                };
                skip_ws(b, pos);
                assert_eq!(b[*pos], ':', "expected ':' after key");
                *pos += 1;
                let val = parse_value(b, pos);
                fields.push((key, val));
                skip_ws(b, pos);
                match b[*pos] {
                    ',' => *pos += 1,
                    '}' => {
                        *pos += 1;
                        return Json::Obj(fields);
                    }
                    c => panic!("expected ',' or '}}' in object, got {:?}", c),
                }
            }
        }
        '[' => {
            *pos += 1;
            let mut items = Vec::new();
            skip_ws(b, pos);
            if b[*pos] == ']' {
                *pos += 1;
                return Json::Arr(items);
            }
            loop {
                items.push(parse_value(b, pos));
                skip_ws(b, pos);
                match b[*pos] {
                    ',' => *pos += 1,
                    ']' => {
                        *pos += 1;
                        return Json::Arr(items);
                    }
                    c => panic!("expected ',' or ']' in array, got {:?}", c),
                }
            }
        }
        '"' => {
            *pos += 1;
            let mut s = String::new();
            loop {
                match b[*pos] {
                    '"' => {
                        *pos += 1;
                        return Json::Str(s);
                    }
                    '\\' => {
                        *pos += 1;
                        match b[*pos] {
                            '"' => s.push('"'),
                            '\\' => s.push('\\'),
                            '/' => s.push('/'),
                            'n' => s.push('\n'),
                            'r' => s.push('\r'),
                            't' => s.push('\t'),
                            'u' => {
                                let hex: String = b[*pos + 1..*pos + 5].iter().collect();
                                let cp = u32::from_str_radix(&hex, 16).expect("hex escape");
                                s.push(char::from_u32(cp).expect("BMP scalar"));
                                *pos += 4;
                            }
                            c => panic!("bad escape \\{}", c),
                        }
                        *pos += 1;
                    }
                    c => {
                        s.push(c);
                        *pos += 1;
                    }
                }
            }
        }
        't' => {
            *pos += 4;
            Json::Bool(true)
        }
        'f' => {
            *pos += 5;
            Json::Bool(false)
        }
        'n' => {
            *pos += 4;
            Json::Null
        }
        _ => {
            let start = *pos;
            while *pos < b.len() && (b[*pos].is_ascii_digit() || "-+.eE".contains(b[*pos])) {
                *pos += 1;
            }
            let text: String = b[start..*pos].iter().collect();
            Json::Num(text.parse().expect("number"))
        }
    }
}

#[test]
fn document_is_valid_json_and_fields_decode() {
    // Errors with GUARANTEED secondary labels (E906 duplicate variant carries
    // a "first declared here" secondary) + help, plus unit-type errors.
    let checked = check(
        "pub device V {\n    variants { X, X }\n    pins[X] { A: 1 [passive] }\n}\npub device D { pins { A: 1 [passive] } }\nimpl D for D {}\ndesign B {\n    inst c: Foo\n    net V [3.3F]: c.A\n}",
    );
    assert!(
        checked.diags.iter().any(|d| !d.secondary.is_empty()),
        "fixture must exercise secondary labels"
    );
    let doc = json::render(&checked, None);
    let parsed = parse_json(&doc);

    assert_eq!(parsed.get("schema_version").unwrap().as_u32(), 1);
    assert_eq!(parsed.get("verdict").unwrap().as_str(), "fail");
    let diags = parsed.get("diagnostics").unwrap().as_arr();
    assert_eq!(
        diags.len(),
        checked.diags.iter().count(),
        "decoded diagnostic count matches the pipeline's"
    );

    // Field-for-field equivalence against the REAL Diagnostic values — code,
    // severity, message, primary (file + start/end line/col + label), every
    // secondary label, and every help line.
    for (d, j) in checked.diags.iter().zip(diags) {
        assert_eq!(j.get("code").unwrap().as_str(), d.code);
        let sev = match d.severity {
            cohdl::diag::Severity::Error => "error",
            cohdl::diag::Severity::Warning => "warning",
        };
        assert_eq!(j.get("severity").unwrap().as_str(), sev);
        assert_eq!(j.get("message").unwrap().as_str(), d.message);

        let p = j.get("primary").unwrap();
        let start = checked
            .sm
            .line_col(d.primary.span.file, d.primary.span.start);
        let end = checked.sm.line_col(d.primary.span.file, d.primary.span.end);
        assert_eq!(p.get("start_line").unwrap().as_u32(), start.line);
        assert_eq!(p.get("start_col").unwrap().as_u32(), start.col);
        assert_eq!(p.get("end_line").unwrap().as_u32(), end.line);
        assert_eq!(p.get("end_col").unwrap().as_u32(), end.col);
        assert_eq!(p.get("message").unwrap().as_str(), d.primary.message);

        // Primary file is asserted too (not just positions).
        assert_eq!(
            p.get("file").unwrap().as_str(),
            checked.sm.name(d.primary.span.file)
        );

        let secondary = j.get("secondary").unwrap().as_arr();
        assert_eq!(secondary.len(), d.secondary.len(), "secondary label count");
        for (sl, sj) in d.secondary.iter().zip(secondary) {
            assert_eq!(sj.get("message").unwrap().as_str(), sl.message);
            assert_eq!(
                sj.get("file").unwrap().as_str(),
                checked.sm.name(sl.span.file)
            );
            let s = checked.sm.line_col(sl.span.file, sl.span.start);
            let e = checked.sm.line_col(sl.span.file, sl.span.end);
            assert_eq!(sj.get("start_line").unwrap().as_u32(), s.line);
            assert_eq!(sj.get("start_col").unwrap().as_u32(), s.col);
            assert_eq!(sj.get("end_line").unwrap().as_u32(), e.line);
            assert_eq!(sj.get("end_col").unwrap().as_u32(), e.col);
        }

        let help = j.get("help").unwrap().as_arr();
        assert_eq!(help.len(), d.help.len(), "help line count");
        for (h, hj) in d.help.iter().zip(help) {
            assert_eq!(hj.as_str(), h);
        }
    }
}

#[test]
fn build_object_decodes_with_and_without_optional_artifacts() {
    let checked = check("pub trait T { pins { required A: pin } }");
    let with = json::render(
        &checked,
        Some(&json::BuildArtifacts {
            netlist: "out/x.net".into(),
            bom: "out/x-bom.csv".into(),
            layout: Some("out/x-layout.json".into()),
            ipc2581: Some("out/x.xml".into()),
            quilter: None,
            kicad_mod: vec!["out/footprints/a.kicad_mod".into()],
        }),
    );
    let parsed = parse_json(&with);
    let build = parsed.get("build").unwrap();
    assert_eq!(build.get("netlist").unwrap().as_str(), "out/x.net");
    assert_eq!(build.get("bom").unwrap().as_str(), "out/x-bom.csv");
    assert_eq!(build.get("layout").unwrap().as_str(), "out/x-layout.json");
    assert_eq!(build.get("ipc2581").unwrap().as_str(), "out/x.xml");
    // RFC-018: kicad_mod is an array, present only when non-empty.
    let mods = parsed.get("build").unwrap().get("kicad_mod").unwrap();
    assert_eq!(mods.as_arr().len(), 1);

    let without = json::render(
        &checked,
        Some(&json::BuildArtifacts {
            netlist: "out/x.net".into(),
            bom: "out/x-bom.csv".into(),
            layout: None,
            ipc2581: None,
            kicad_mod: Vec::new(),
            quilter: None,
        }),
    );
    let parsed = parse_json(&without);
    assert!(parsed.get("build").unwrap().get("layout").is_none());
    assert!(parsed.get("build").unwrap().get("ipc2581").is_none());

    // RFC-015 without layout: ipc2581 is then the LAST key (comma handling).
    let ipc_only = json::render(
        &checked,
        Some(&json::BuildArtifacts {
            netlist: "out/x.net".into(),
            bom: "out/x-bom.csv".into(),
            layout: None,
            ipc2581: Some("out/x.xml".into()),
            kicad_mod: Vec::new(),
            quilter: None,
        }),
    );
    let parsed = parse_json(&ipc_only);
    let build = parsed.get("build").unwrap();
    assert!(build.get("layout").is_none());
    assert_eq!(build.get("ipc2581").unwrap().as_str(), "out/x.xml");
    // RFC-018: kicad_mod absent when no footprint projected.
    assert!(build.get("kicad_mod").is_none());
}

#[test]
fn schema_shape_and_escaping() {
    // A message that contains a backtick and a quote-ish glyph must serialize
    // as valid JSON with the code/verdict fields present — now checked by an
    // actual decode, not brace counting.
    let checked = check("design B {\n    inst c: Foo\n    net V: c.A\n}");
    let doc = json::render(&checked, None);
    let parsed = parse_json(&doc);
    assert!(doc.starts_with("{\n  \"schema_version\": 1,\n"), "{}", doc);
    assert_eq!(parsed.get("verdict").unwrap().as_str(), "fail");
    let diags = parsed.get("diagnostics").unwrap().as_arr();
    assert!(diags
        .iter()
        .any(|d| d.get("code").unwrap().as_str() == "E202"));
}

#[test]
fn clean_design_reports_pass_and_empty_diagnostics() {
    let checked = check("pub trait T { pins { required A: pin } }");
    let doc = json::render(&checked, None);
    assert!(doc.contains("\"verdict\": \"pass\""), "{}", doc);
    assert!(doc.contains("\"diagnostics\": []"), "{}", doc);
}
