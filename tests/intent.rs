//! RFC-012 `#[intent("...")]` conformance.
//!
//! The single load-bearing property: intent is opaque metadata that can NEVER
//! affect compilation — same verdict, diagnostics, designators, and emitted
//! netlist/BOM bytes whether the annotation is present, absent, or mutated to
//! arbitrary text (including checkable-sounding prose). Plus the grammar rules:
//! exactly one string, at most one per declaration, valid on declarations and
//! statements but not sub-statement granularity.

use cohdl::fmt::format_source;
use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files};

/// A self-contained, cleanly-buildable board (no std needed).
const BASE: &str = r#"
pub trait TwoTerminal { pins { required A: pin required B: pin } }
pub trait Resistor: TwoTerminal {
    designator_prefix: "R"
    spec { resistance: Resistance, tolerance: Tolerance }
}
pub device Res<R: Resistance, T: Tolerance = 1%> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { resistance: R, tolerance: T }
}
impl TwoTerminal for Res {}
impl Resistor for Res {}
pub footprint TFP {}
pub part R1K: Res<1kohm, 1%> {
    primary { mfr: "Yageo", mpn: "RC0402FR-071KL", footprint: TFP }
}
design B {
    inst r1: R1K
    inst r2: R1K
    net N: r1.A, r2.A
    net GND [gnd]: r1.B, r2.B
}
"#;

/// The same board with `#[intent(...)]` sprinkled on every legal target.
const ANNOTATED: &str = r#"
#[intent("the fundamental two-terminal passive contract")]
pub trait TwoTerminal { pins { required A: pin required B: pin } }
pub trait Resistor: TwoTerminal {
    designator_prefix: "R"
    spec { resistance: Resistance, tolerance: Tolerance }
}
#[intent("generic chip resistor family")]
pub device Res<R: Resistance, T: Tolerance = 1%> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { resistance: R, tolerance: T }
}
#[intent("Res satisfies TwoTerminal by name")]
impl TwoTerminal for Res {}
impl Resistor for Res {}
#[intent("1k 1% 0402 — the only value this demo needs")]
pub footprint TFP {}
pub part R1K: Res<1kohm, 1%> {
    primary { mfr: "Yageo", mpn: "RC0402FR-071KL", footprint: TFP }
}
#[intent("the whole board")]
design B {
    #[intent("must be < 10V, or so a naive reader might wrongly encode here")]
    inst r1: R1K
    inst r2: R1K
    #[intent("the shared signal node per spec table 4-15")]
    net N: r1.A, r2.A
    net GND [gnd]: r1.B, r2.B
}
"#;

/// Everything a build observably produces — the full non-impact surface.
struct Built {
    verdict: &'static str,
    diags: String,
    json: String,
    netlist: String,
    bom: String,
    /// The rendered design.lock — covers designator assignment byte-for-byte.
    lock: String,
}

fn build(src: &str) -> Built {
    let files = vec![("board.cohdl".to_string(), src.to_string())];
    let mut checked = check_files(&files, None).expect("design selection");
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    let diags = checked.diags.render(&checked.sm);
    let json = cohdl::emit::json::render(&checked, None);
    let verdict = cohdl::emit::json::verdict(&checked);
    match artifacts {
        Some(a) => Built {
            verdict,
            diags,
            json,
            netlist: a.netlist,
            bom: a.bom,
            lock: a.lock.render(),
        },
        None => Built {
            verdict,
            diags,
            json,
            netlist: String::new(),
            bom: String::new(),
            lock: String::new(),
        },
    }
}

/// The complete RFC-012 non-impact assertion: verdict, rendered diagnostics,
/// `--json` document, netlist, BOM, and designator lock must all be
/// byte-identical between two sources.
fn assert_no_impact(a: &Built, b: &Built, what: &str) {
    assert_eq!(a.verdict, b.verdict, "verdict changed: {}", what);
    assert_eq!(a.diags, b.diags, "diagnostics changed: {}", what);
    assert_eq!(a.json, b.json, "--json document changed: {}", what);
    assert_eq!(a.netlist, b.netlist, "netlist changed: {}", what);
    assert_eq!(a.bom, b.bom, "BOM changed: {}", what);
    assert_eq!(a.lock, b.lock, "designator lock changed: {}", what);
}

#[test]
fn intent_is_zero_impact() {
    let base = build(BASE);
    let ann = build(ANNOTATED);
    assert_no_impact(&base, &ann, "intent added");
    // And a clean build actually happened (not two identically-broken ones).
    assert!(
        base.netlist.contains("(export"),
        "expected a real netlist:\n{}",
        base.netlist
    );
    // Intent content never leaks into the JSON diagnostics document — check
    // with substrings that actually occur in ANNOTATED's intent strings.
    for leak in [
        "table 4-15",
        "two-terminal passive contract",
        "naive reader",
    ] {
        assert!(
            !ann.json.contains(leak),
            "intent text `{}` leaked into --json:\n{}",
            leak,
            ann.json
        );
    }
}

#[test]
fn mutating_intent_text_is_zero_impact() {
    // Swap intent strings for arbitrary different text — checkable-sounding
    // prose and unicode — inside the annotation strings only. Nothing
    // observable may move.
    let mutated = ANNOTATED
        .replace(
            "the shared signal node per spec table 4-15",
            "voltage must never exceed 5V",
        )
        .replace("the whole board", "🔧 unicode rationale ✓");
    assert_ne!(
        mutated, ANNOTATED,
        "the mutation must actually change the text"
    );
    assert_no_impact(&build(ANNOTATED), &build(&mutated), "intent text mutated");
}

/// Recursively strip the position fields from a `--json` document, leaving
/// everything else (codes, severities, messages, FILE names, secondary
/// label messages, help strings) for exact comparison.
fn strip_positions(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::Object(map) => {
            for k in ["start_line", "start_col", "end_line", "end_col"] {
                map.remove(k);
            }
            for (_, child) in map.iter_mut() {
                strip_positions(child);
            }
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(strip_positions),
        _ => {}
    }
}

#[test]
fn intent_is_zero_impact_on_diagnostic_positions_in_failing_fixture() {
    // A fixture whose diagnostics carry the FULL label structure — a
    // warning (D003), an error with secondary labels AND a help line (E30x
    // impl satisfaction), and a resolve error (E202) — so this comparison is
    // not vacuous (review R4). Adding intent above the offending statements
    // must not change anything but positions.
    let bad = r#"
pub trait Needs { pins { required MISSING: pin } }
pub device MCU { pins { required TX: 1 [output], required GND: 2 [power_in] } }
impl Needs for MCU {}
design B {
    inst mcu: MCU
    net LONELY: mcu.TX
    net G: mcu.GND, nosuch.PIN
}
"#;
    let bad_ann = bad
        .replace(
            "    net LONELY: mcu.TX",
            "    #[intent(\"dangling driver kept for probing\")]\n    net LONELY: mcu.TX",
        )
        .replace(
            "impl Needs for MCU {}",
            "#[intent(\"deliberately unsatisfiable\")]\nimpl Needs for MCU {}",
        );
    let a = build(bad);
    let b = build(&bad_ann);
    assert_eq!(a.verdict, "fail");
    assert_eq!(a.verdict, b.verdict);
    // Prove the fixture exercises help + secondary labels (non-vacuous).
    assert!(
        a.diags.contains("D003") && a.diags.contains("E202"),
        "{}",
        a.diags
    );
    let a_doc: serde_json::Value = serde_json::from_str(&a.json).unwrap();
    assert!(
        a_doc["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| !d["secondary"].as_array().unwrap().is_empty()
                && !d["help"].as_array().unwrap().is_empty()),
        "fixture must produce a diagnostic with secondary labels AND help:\n{}",
        a.json
    );
    // Compare the COMPLETE JSON models minus only positions (the attributes
    // occupy source lines, so spans shift; every other field — including
    // file names, label messages, and the actual help strings — must match).
    let mut a_stripped: serde_json::Value = serde_json::from_str(&a.json).unwrap();
    let mut b_stripped: serde_json::Value = serde_json::from_str(&b.json).unwrap();
    strip_positions(&mut a_stripped);
    strip_positions(&mut b_stripped);
    assert_eq!(
        a_stripped, b_stripped,
        "diagnostic content diverged:\n--- a ---\n{}\n--- b ---\n{}",
        a.json, b.json
    );
}

#[test]
fn mutating_intent_on_failing_fixture_is_fully_json_identical() {
    // RFC-012's mandatory mutation test on a FAILING fixture: the attribute
    // sits on its own line, so mutating the intent STRING (same line count)
    // moves no diagnostic position at all — the entire --json document must
    // be byte-identical, no field excluded (review R4).
    let annotated = r#"
pub trait Needs { pins { required MISSING: pin } }
pub device MCU { pins { required TX: 1 [output], required GND: 2 [power_in] } }
impl Needs for MCU {}
design B {
    inst mcu: MCU
    #[intent("dangling driver kept for probing")]
    net LONELY: mcu.TX
    net G: mcu.GND, nosuch.PIN
}
"#;
    let mutated = annotated.replace(
        "dangling driver kept for probing",
        "voltage must never exceed 5V 🌍",
    );
    assert_ne!(annotated, mutated);
    let a = build(annotated);
    let b = build(&mutated);
    assert_eq!(a.verdict, "fail");
    assert_no_impact(&a, &b, "intent text mutated on a failing fixture");
    // assert_no_impact already compares the json byte-for-byte; restate the
    // load-bearing one explicitly.
    assert_eq!(
        a.json, b.json,
        "full --json document must be byte-identical"
    );
}

// ---------------------------------------------------------------------------
// Grammar rules.

fn check_err(src: &str) -> String {
    let files = vec![("f.cohdl".to_string(), src.to_string())];
    let mut checked = check_files(&files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let r = checked.diags.render(&checked.sm);
    assert!(
        checked.diags.has_errors(),
        "expected an error for:\n{}",
        src
    );
    r
}

fn parse_ok(src: &str) {
    let files = vec![("f.cohdl".to_string(), src.to_string())];
    let checked = check_files(&files, None).expect("selection");
    // No *parse* errors (E0xx) — later-stage errors are fine for this check.
    let rendered = checked.diags.render(&checked.sm);
    assert!(
        !rendered.contains("error[E01") && !rendered.contains("error[E00"),
        "unexpected parse error:\n{}",
        rendered
    );
}

#[test]
fn intent_valid_on_every_declared_target() {
    parse_ok(
        "#[intent(\"a\")] pub trait T { pins { required A: pin } }\n\
         #[intent(\"b\")] pub device D { pins { A: 1 [passive] } }\n\
         #[intent(\"c\")] impl T for D {}\n\
         #[intent(\"d\")] pub fn f(p: Pin) { net _: p }\n\
         #[intent(\"e\")] pub part P: D { primary { mfr: \"m\", mpn: \"n\", footprint: TFP } }\n\
         #[intent(\"f\")] design Z {\n\
         #[intent(\"g\")] inst d: P\n\
         #[intent(\"h\")] net N: d.A\n\
         #[intent(\"i\")] nc: d.A\n\
         }",
    );
}

#[test]
fn multiple_intent_on_one_declaration_is_an_error() {
    let r =
        check_err("#[intent(\"a\")]\n#[intent(\"b\")]\npub device D { pins { A: 1 [passive] } }");
    assert!(r.contains("at most one `#[intent"), "{}", r);
}

#[test]
fn malformed_intent_arg_shape_is_an_error() {
    // No argument.
    let r = check_err("#[intent]\npub device D { pins { A: 1 [passive] } }");
    assert!(r.contains("exactly one string"), "{}", r);
    // Two arguments.
    let r = check_err("#[intent(\"a\", \"b\")]\npub device D { pins { A: 1 [passive] } }");
    assert!(r.contains("exactly one string"), "{}", r);
}

#[test]
fn designator_attr_still_inst_only() {
    // #[designator] on a device (not inst) is still rejected — intent handling
    // must not have loosened that.
    let r = check_err("#[designator(\"U7\")]\npub device D { pins { A: 1 [passive] } }");
    assert!(r.contains("designator"), "{}", r);
}

// ---------------------------------------------------------------------------
// fmt (RFC-009) round-trips intent idempotently.

#[test]
fn fmt_preserves_and_normalizes_intent() {
    let src = "design B{#[intent(\"why\")]inst c: D\n#[intent(\"because\")]net N: c.A}";
    let once = format_source("i.cohdl", src).unwrap();
    assert!(once.contains("#[intent(\"why\")]"), "{}", once);
    assert!(once.contains("#[intent(\"because\")]"), "{}", once);
    let twice = format_source("i.cohdl", &once).unwrap();
    assert_eq!(
        once, twice,
        "intent formatting is not idempotent:\n{}",
        once
    );
}
