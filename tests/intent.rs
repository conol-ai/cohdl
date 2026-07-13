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
pub part R1K: Res<1kohm, 1%> {
    primary { mfr: "Yageo", mpn: "RC0402FR-071KL", footprint: "Resistor_SMD:R_0402_1005Metric" }
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
pub part R1K: Res<1kohm, 1%> {
    primary { mfr: "Yageo", mpn: "RC0402FR-071KL", footprint: "Resistor_SMD:R_0402_1005Metric" }
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

/// Build a single-file project, returning (rendered diagnostics, netlist, BOM).
fn build(src: &str) -> (String, String, String) {
    let files = vec![("board.cohdl".to_string(), src.to_string())];
    let mut checked = check_files(&files, None).expect("design selection");
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    match artifacts {
        Some(a) => (rendered, a.netlist, a.bom),
        None => (rendered, String::new(), String::new()),
    }
}

#[test]
fn intent_is_zero_impact() {
    let (d_base, net_base, bom_base) = build(BASE);
    let (d_ann, net_ann, bom_ann) = build(ANNOTATED);

    // Byte-identical netlist, BOM, and diagnostics — intent changes nothing.
    assert_eq!(net_base, net_ann, "netlist changed when intent was added");
    assert_eq!(bom_base, bom_ann, "BOM changed when intent was added");
    assert_eq!(d_base, d_ann, "diagnostics changed when intent was added");
    // And a clean build actually happened (not two identically-broken ones).
    assert!(
        net_base.contains("(export"),
        "expected a real netlist:\n{}",
        net_base
    );
}

#[test]
fn mutating_intent_text_is_zero_impact() {
    // Swap intent strings for arbitrary different text — checkable-sounding
    // prose and unicode — inside the annotation strings only. Emitted bytes
    // must not move.
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

    let (_, net_ann, bom_ann) = build(ANNOTATED);
    let (_, net_mut, bom_mut) = build(&mutated);
    assert_eq!(
        net_ann, net_mut,
        "netlist changed when intent text was mutated"
    );
    assert_eq!(bom_ann, bom_mut, "BOM changed when intent text was mutated");
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
         #[intent(\"e\")] pub part P: D { primary { mfr: \"m\", mpn: \"n\", footprint: \"fp\" } }\n\
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
