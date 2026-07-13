//! RFC-013 layout-constraint conformance.
//!
//! Two load-bearing properties: (1) the four constraint kinds are type-checked
//! against their closed vocabulary (E1001-E1004) with precise spans; (2) layout
//! metadata is zero-impact — adding/removing/mutating a `layout {}` block or a
//! `#[placement_hint]` never changes the schematic verdict, any RFC-001–011
//! diagnostic, a designator, or the emitted `.net`/BOM bytes. Only the separate
//! `layout.json` artifact differs.

use cohdl::fmt::format_source;
use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files};

const BASE: &str = r#"
pub trait TwoTerminal { pins { required A: pin required B: pin } }
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
impl TwoTerminal for Res {}
pub part R1: Res { primary { mfr: "m", mpn: "n", footprint: "fp" } }
design Board {
    inst r1: R1
    inst r2: R1
    net USB_DP: r1.A, r2.A
    net USB_DM: r1.B, r2.B
}
"#;

const WITH_LAYOUT: &str = r#"
pub trait TwoTerminal { pins { required A: pin required B: pin } }
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
impl TwoTerminal for Res {}
pub part R1: Res { primary { mfr: "m", mpn: "n", footprint: "fp" } }
design Board {
    #[placement_hint("near the USB connector")]
    inst r1: R1
    inst r2: R1
    net USB_DP: r1.A, r2.A
    net USB_DM: r1.B, r2.B
    layout {
        net_class HighSpeed { USB_DP, USB_DM }
        diff_pair(USB_DP, USB_DM)
        length_match(USB_DP, USB_DM) [tolerance: "0.15mm"]
    }
}
"#;

struct Built {
    diags: String,
    netlist: String,
    bom: String,
    layout: Option<String>,
}

fn build(src: &str) -> Built {
    let files = vec![("board.cohdl".to_string(), src.to_string())];
    let mut checked = check_files(&files, None).expect("selection");
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    let diags = checked.diags.render(&checked.sm);
    match artifacts {
        Some(a) => Built {
            diags,
            netlist: a.netlist,
            bom: a.bom,
            layout: a.layout,
        },
        None => Built {
            diags,
            netlist: String::new(),
            bom: String::new(),
            layout: None,
        },
    }
}

fn check_err(src: &str) -> String {
    let files = vec![("f.cohdl".to_string(), src.to_string())];
    let mut checked = check_files(&files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let r = checked.diags.render(&checked.sm);
    assert!(checked.diags.has_errors(), "expected an error:\n{}", src);
    r
}

// ---------------------------------------------------------------------------
// Zero-impact: the whole point.

#[test]
fn layout_is_zero_impact_on_netlist_and_diagnostics() {
    let base = build(BASE);
    let laid = build(WITH_LAYOUT);
    assert!(base.netlist.contains("(export"), "base must build cleanly");
    assert_eq!(base.netlist, laid.netlist, "layout changed the netlist");
    assert_eq!(base.bom, laid.bom, "layout changed the BOM");
    assert_eq!(base.diags, laid.diags, "layout changed the diagnostics");
    // BASE emits no layout.json; the annotated one does.
    assert!(base.layout.is_none(), "base has no layout artifact");
    assert!(laid.layout.is_some(), "annotated design emits layout.json");
}

#[test]
fn mutating_layout_content_is_zero_impact() {
    let laid = build(WITH_LAYOUT);
    let mutated = build(
        &WITH_LAYOUT
            .replace("HighSpeed", "Critical")
            .replace("0.15mm", "10mil")
            .replace("near the USB connector", "corner of the board"),
    );
    assert_eq!(
        laid.netlist, mutated.netlist,
        "mutating layout moved the netlist"
    );
    assert_eq!(laid.bom, mutated.bom, "mutating layout moved the BOM");
    // The layout artifact itself DOES reflect the change.
    assert_ne!(
        laid.layout, mutated.layout,
        "layout.json should reflect the edit"
    );
}

#[test]
fn layout_json_shape() {
    let laid = build(WITH_LAYOUT);
    let json = laid.layout.expect("layout.json");
    for needle in [
        "\"schema_version\": 1",
        "\"name\": \"HighSpeed\"",
        "\"p\": \"USB_DP\"",
        "\"n\": \"USB_DM\"",
        "\"tolerance\": \"0.15mm\"",
        "\"hint\": \"near the USB connector\"",
        "\"designator\": \"U1\"",
    ] {
        assert!(
            json.contains(needle),
            "layout.json missing {}:\n{}",
            needle,
            json
        );
    }
    // Byte-stable: two builds produce identical bytes.
    assert_eq!(json, build(WITH_LAYOUT).layout.unwrap());
}

// ---------------------------------------------------------------------------
// The closed-vocabulary checks (E1001-E1004).

#[test]
fn unknown_net_is_e1001() {
    let r = check_err(&BASE.replace(
        "design Board {",
        "design Board {\n    layout { diff_pair(USB_DP, MISSING) }",
    ));
    assert!(r.contains("E1001") && r.contains("MISSING"), "{}", r);
}

#[test]
fn duplicate_net_class_is_e1002() {
    let r = check_err(&BASE.replace(
        "design Board {",
        "design Board {\n    layout { net_class C { USB_DP } net_class C { USB_DM } }",
    ));
    assert!(r.contains("E1002"), "{}", r);
}

#[test]
fn diff_pair_wrong_arity_is_e1003() {
    let one = check_err(&BASE.replace(
        "design Board {",
        "design Board {\n    layout { diff_pair(USB_DP) }",
    ));
    assert!(one.contains("E1003"), "{}", one);
    let three = check_err(&BASE.replace(
        "net USB_DM: r1.B, r2.B",
        "net USB_DM: r1.B, r2.B\n    layout { diff_pair(USB_DP, USB_DM, USB_DP) }",
    ));
    assert!(three.contains("E1003"), "{}", three);
}

#[test]
fn length_match_too_few_is_e1004() {
    let r = check_err(&BASE.replace(
        "design Board {",
        "design Board {\n    layout { length_match(USB_DP) }",
    ));
    assert!(r.contains("E1004"), "{}", r);
}

// ---------------------------------------------------------------------------
// fmt (RFC-009) round-trips layout blocks + placement hints idempotently.

#[test]
fn fmt_normalizes_layout() {
    let src = "design B{net A: x.p\nnet C: y.p\nlayout{net_class K{A,C} diff_pair(A,C) length_match(A,C)[tolerance:\"1mm\"]}}";
    let once = format_source("l.cohdl", src).unwrap();
    assert!(once.contains("layout {"), "{}", once);
    assert!(once.contains("net_class K { A, C }"), "{}", once);
    assert!(once.contains("diff_pair(A, C)"), "{}", once);
    assert!(
        once.contains("length_match(A, C) [tolerance: \"1mm\"]"),
        "{}",
        once
    );
    let twice = format_source("l.cohdl", &once).unwrap();
    assert_eq!(
        once, twice,
        "layout formatting is not idempotent:\n{}",
        once
    );
}

#[test]
fn fmt_round_trips_placement_hint() {
    let src = "design B{#[placement_hint(\"corner\")]inst r: R1}";
    let once = format_source("p.cohdl", src).unwrap();
    assert!(once.contains("#[placement_hint(\"corner\")]"), "{}", once);
    let twice = format_source("p.cohdl", &once).unwrap();
    assert_eq!(once, twice);
}
