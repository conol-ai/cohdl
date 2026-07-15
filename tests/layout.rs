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
pub footprint TFP {}
pub part R1: Res { primary { mfr: "m", mpn: "n", footprint: TFP } }
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
pub footprint TFP {}
pub part R1: Res { primary { mfr: "m", mpn: "n", footprint: TFP } }
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
    verdict: &'static str,
    diags: String,
    json: String,
    netlist: String,
    bom: String,
    /// Rendered design.lock — designator assignment, byte-for-byte.
    lock: String,
    layout: Option<String>,
}

fn build(src: &str) -> Built {
    let files = vec![("board.cohdl".to_string(), src.to_string())];
    let mut checked = check_files(&files, None).expect("selection");
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
            layout: a.layout,
        },
        None => Built {
            verdict,
            diags,
            json,
            netlist: String::new(),
            bom: String::new(),
            lock: String::new(),
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
    // The full observable surface: verdict, diagnostics, --json document,
    // netlist, BOM, and designator lock.
    assert_eq!(base.verdict, laid.verdict, "layout changed the verdict");
    assert_eq!(base.netlist, laid.netlist, "layout changed the netlist");
    assert_eq!(base.bom, laid.bom, "layout changed the BOM");
    assert_eq!(base.diags, laid.diags, "layout changed the diagnostics");
    assert_eq!(base.json, laid.json, "layout changed the --json document");
    assert_eq!(base.lock, laid.lock, "layout changed the designator lock");
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
    assert_eq!(laid.verdict, mutated.verdict, "verdict moved");
    assert_eq!(laid.diags, mutated.diags, "diagnostics moved");
    assert_eq!(
        laid.json, mutated.json,
        "mutating layout moved the --json document (review R4: this was the
         missing surface — reply2 claimed it was compared)"
    );
    assert_eq!(laid.lock, mutated.lock, "designator lock moved");
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
fn merged_away_net_is_not_a_false_e1001() {
    // ZZZ shares pin r1.A with USB_DP, so the two merge and the group keeps the
    // smaller name (USB_DP). A layout reference to ZZZ — a genuinely declared
    // net — must NOT spuriously E1001, and must resolve to the merged name in
    // layout.json (regression: validation was against post-merge names).
    let src = BASE.replace(
        "net USB_DM: r1.B, r2.B",
        "net USB_DM: r1.B, r2.B\n    net ZZZ: r1.A\n    layout { net_class C { ZZZ } }",
    );
    let built = build(&src);
    assert!(
        !built.diags.contains("E1001"),
        "declared-but-merged net wrongly flagged:\n{}",
        built.diags
    );
    let json = built.layout.expect("layout.json");
    assert!(
        json.contains("\"USB_DP\""),
        "merged name not used:\n{}",
        json
    );
    assert!(
        !json.contains("\"ZZZ\""),
        "pre-merge name leaked into layout.json:\n{}",
        json
    );
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

// A pair/group must contain DISTINCT final nets — both when the same name is
// repeated and when two source names merge into one electrical net.
#[test]
fn diff_pair_same_net_is_rejected() {
    // Direct repetition.
    let r = check_err(&BASE.replace(
        "design Board {",
        "design Board {\n    layout { diff_pair(USB_DP, USB_DP) }",
    ));
    assert!(r.contains("E1003") && r.contains("distinct"), "{}", r);

    // Merged aliases: ZZZ shares pin r1.A with USB_DP, so they are one net.
    let r = check_err(&BASE.replace(
        "net USB_DM: r1.B, r2.B",
        "net USB_DM: r1.B, r2.B\n    net ZZZ: r1.A\n    layout { diff_pair(USB_DP, ZZZ) }",
    ));
    assert!(
        r.contains("E1003") && r.contains("resolve to the same net"),
        "{}",
        r
    );
}

#[test]
fn length_match_all_same_net_is_rejected() {
    let r = check_err(&BASE.replace(
        "design Board {",
        "design Board {\n    layout { length_match(USB_DP, USB_DP, USB_DP) }",
    ));
    assert!(r.contains("E1004") && r.contains("distinct"), "{}", r);
}

// Re-verification residual #1: a partially-merged group deduplicates on
// emission — the merged name appears once, first-occurrence order.
#[test]
fn partially_merged_group_deduplicates_in_layout_json() {
    // ZZZ merges with USB_DP (shared pin r1.A); USB_DM stays distinct. The
    // group is legal (2 distinct nets) but must not list USB_DP twice.
    let src = BASE.replace(
        "net USB_DM: r1.B, r2.B",
        "net USB_DM: r1.B, r2.B\n    net ZZZ: r1.A\n    layout { net_class G { USB_DP, ZZZ, USB_DM }\n    length_match(USB_DP, ZZZ, USB_DM) }",
    );
    let built = build(&src);
    assert!(!built.diags.contains("error"), "{}", built.diags);
    let json = built.layout.expect("layout.json");
    assert!(
        json.contains("\"nets\": [\"USB_DP\", \"USB_DM\"]"),
        "merged alias must dedup to one entry:\n{}",
        json
    );
    assert!(!json.contains("USB_DP\", \"USB_DP"), "{}", json);
}

// Re-verification residual #2: the fn-scope E1002 message names the SOURCE
// class name with scope context — never the raw mangled identity.
#[test]
fn fn_scope_duplicate_class_message_uses_source_name() {
    let src = r#"
pub device D { pins { A: 1 [passive], B: 2 [passive] } }
fn pair(a: Pin, b: Pin) {
    net N: a, b
    layout {
        net_class K { N }
        net_class K { N }
    }
}
design Board {
    inst d1: D
    inst d2: D
    pair(d1.A, d2.A)
    net GND: d1.B, d2.B
}
"#;
    let r = check_err(src);
    assert!(r.contains("E1002"), "{}", r);
    assert!(
        r.contains("duplicate `net_class` name `K` (in `__fn0_pair`)"),
        "message must lead with the source name:\n{}",
        r
    );
}

// RFC-013 says `<Time-or-length-unit>` — Time and (since RFC-018 made
// Length real) mm literals are legal unquoted; every other unit type is
// still rejected.
#[test]
fn tolerance_non_time_units_are_rejected() {
    for bad in ["5V", "100nF", "1kohm"] {
        let r = check_err(
            &WITH_LAYOUT.replace("[tolerance: \"0.15mm\"]", &format!("[tolerance: {}]", bad)),
        );
        assert!(
            r.contains("E110") && r.contains("`Time`"),
            "`{}` must be rejected as a tolerance:\n{}",
            bad,
            r
        );
    }
    // The accepted RFC-013 example, finally representable (RFC-018 Length).
    let ok = build(&WITH_LAYOUT.replace("[tolerance: \"0.15mm\"]", "[tolerance: 0.15mm]"));
    assert!(
        !ok.diags.contains("error"),
        "unquoted mm tolerance must parse:\n{}",
        ok.diags
    );
    assert!(ok
        .layout
        .expect("layout.json")
        .contains("\"tolerance\": \"0.15mm\""));
}

// A layout-bearing fn is reusable: fn-local net_class names get call-chain
// identity (like fn-local nets, RFC-006), so two calls do not collide.
#[test]
fn layout_bearing_fn_is_reusable() {
    let src = r#"
pub trait TwoTerminal { pins { required A: pin required B: pin } }
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
impl TwoTerminal for Res {}
pub footprint TFP {}
pub part R1: Res { primary { mfr: "m", mpn: "n", footprint: TFP } }
fn routed_pair(a: Pin, b: Pin, c: Pin, d: Pin) {
    net DP: a, b
    net DM: c, d
    layout {
        net_class HighSpeed { DP, DM }
        diff_pair(DP, DM)
    }
}
design Board {
    inst d1: R1
    inst d2: R1
    inst d3: R1
    inst d4: R1
    routed_pair(d1.A, d2.A, d1.B, d2.B)
    routed_pair(d3.A, d4.A, d3.B, d4.B)
}
"#;
    let built = build(src);
    assert!(
        !built.diags.contains("error"),
        "two calls must not collide:\n{}",
        built.diags
    );
    let json = built.layout.expect("layout.json");
    // Two scoped classes, two diff pairs — each call's constraints survive.
    assert_eq!(
        json.matches("HighSpeed").count(),
        2,
        "expected two scoped HighSpeed classes:\n{}",
        json
    );
    assert_eq!(json.matches("\"p\":").count(), 2, "{}", json);
}

// Tolerance accepts an RFC-001 unit literal (pass-through text) as well as the
// quoted-string escape hatch (for length units RFC-001 cannot represent).
#[test]
fn tolerance_accepts_unit_literal_and_string() {
    let unit = build(&WITH_LAYOUT.replace("[tolerance: \"0.15mm\"]", "[tolerance: 1ms]"));
    assert!(
        !unit.diags.contains("error"),
        "unit-literal tolerance must parse:\n{}",
        unit.diags
    );
    assert!(
        unit.layout
            .expect("layout.json")
            .contains("\"tolerance\": \"1ms\""),
        "unit literal passes through as its source text"
    );

    let string = build(WITH_LAYOUT);
    assert!(string
        .layout
        .expect("layout.json")
        .contains("\"tolerance\": \"0.15mm\""));
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
        // `1mm` lexes as a Length literal since RFC-018 — canonical form is
        // unquoted.
        once.contains("length_match(A, C) [tolerance: 1mm]"),
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

// ---------------------------------------------------------------------------
// board_outline — the pragmatic extension (rectangular board perimeter, E1006).

fn with_outline(outline: &str) -> String {
    BASE.replace(
        "net USB_DM: r1.B, r2.B",
        &format!("net USB_DM: r1.B, r2.B\n    layout {{ {} }}", outline),
    )
}

#[test]
fn board_outline_projects_to_layout_json() {
    let built = build(&with_outline(
        "board_outline { at: (1mm, -2mm), size: (51mm, 21mm) }",
    ));
    assert!(
        !built.diags.contains("E1006"),
        "unexpected error:\n{}",
        built.diags
    );
    let json = built.layout.expect("layout.json");
    assert!(
        json.contains("\"board_outline\": { \"at\": [1, -2], \"size\": [51, 21] }"),
        "board_outline missing/wrong in layout.json:\n{}",
        json
    );
    // Byte-stable.
    assert_eq!(
        json,
        build(&with_outline(
            "board_outline { at: (1mm, -2mm), size: (51mm, 21mm) }"
        ))
        .layout
        .unwrap()
    );
}

#[test]
fn no_board_outline_is_json_null() {
    let built = build(&with_outline("net_class C { USB_DP }"));
    let json = built.layout.expect("layout.json");
    assert!(json.contains("\"board_outline\": null"), "{}", json);
}

#[test]
fn board_outline_is_zero_impact_on_netlist_and_bom() {
    let base = build(BASE);
    let laid = build(&with_outline(
        "board_outline { at: (0mm, 0mm), size: (10mm, 10mm) }",
    ));
    assert_eq!(
        base.netlist, laid.netlist,
        "board_outline moved the netlist"
    );
    assert_eq!(base.bom, laid.bom, "board_outline moved the BOM");
    assert_eq!(
        base.verdict, laid.verdict,
        "board_outline moved the verdict"
    );
    assert_eq!(
        base.lock, laid.lock,
        "board_outline moved the designator lock"
    );
}

#[test]
fn board_outline_non_length_is_e1006() {
    let r = check_err(&with_outline(
        "board_outline { at: (0mm, 0mm), size: (10V, 10mm) }",
    ));
    assert!(r.contains("E1006") && r.contains("Length"), "{}", r);
}

#[test]
fn board_outline_non_positive_size_is_e1006() {
    let r = check_err(&with_outline(
        "board_outline { at: (0mm, 0mm), size: (0mm, 10mm) }",
    ));
    assert!(r.contains("E1006") && r.contains("positive"), "{}", r);
}

#[test]
fn duplicate_board_outline_is_e1006() {
    let two = "board_outline { at: (0mm, 0mm), size: (10mm, 10mm) } \
               board_outline { at: (0mm, 0mm), size: (20mm, 20mm) }";
    let r = check_err(&with_outline(two));
    assert!(r.contains("E1006") && r.contains("at most one"), "{}", r);
}

#[test]
fn board_outline_inside_fn_is_e1006() {
    // A fn body carrying a board_outline is rejected — a board has one
    // physical perimeter, not one per sub-circuit call.
    let src = r#"
pub trait TwoTerminal { pins { required A: pin required B: pin } }
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
impl TwoTerminal for Res {}
pub footprint TFP {}
pub part R1: Res { primary { mfr: "m", mpn: "n", footprint: TFP } }
pub fn sub(x: Pin) {
    inst r: R1
    net _: x, r.A
    layout { board_outline { at: (0mm, 0mm), size: (5mm, 5mm) } }
}
design Board {
    inst r2: R1
    net N: r2.A
    sub(r2.B)
}
"#;
    let r = check_err(src);
    assert!(r.contains("E1006") && r.contains("fn"), "{}", r);
}

#[test]
fn placement_projects_to_layout_json() {
    let built = build(&with_outline(
        "board_outline { at: (0mm, 0mm), size: (10mm, 10mm) } place r1 at (1mm, -2mm)",
    ));
    assert!(
        !built.diags.contains("E1007"),
        "unexpected error:\n{}",
        built.diags
    );
    let json = built.layout.expect("layout.json");
    assert!(
        json.contains("\"placements\": [")
            && json.contains("\"instance\": \"Board::r1\"")
            && json.contains("\"at\": [1, -2]"),
        "placement missing/wrong in layout.json:\n{}",
        json
    );
}

#[test]
fn placement_is_zero_impact_on_netlist_and_bom() {
    let base = build(BASE);
    let laid = build(&with_outline("place r1 at (0mm, 0mm)"));
    assert_eq!(base.netlist, laid.netlist, "place moved the netlist");
    assert_eq!(base.bom, laid.bom, "place moved the BOM");
    assert_eq!(base.lock, laid.lock, "place moved the designator lock");
}

#[test]
fn place_unknown_instance_is_e1007() {
    let r = check_err(&with_outline("place nope at (0mm, 0mm)"));
    assert!(r.contains("E1007") && r.contains("nope"), "{}", r);
}

#[test]
fn place_non_length_is_e1007() {
    let r = check_err(&with_outline("place r1 at (0mm, 3V)"));
    assert!(r.contains("E1007") && r.contains("Length"), "{}", r);
}

#[test]
fn duplicate_place_is_e1007() {
    let r = check_err(&with_outline(
        "place r1 at (0mm, 0mm) place r1 at (1mm, 1mm)",
    ));
    assert!(r.contains("E1007") && r.contains("more than once"), "{}", r);
}

#[test]
fn place_inside_fn_is_e1007() {
    let src = r#"
pub trait TwoTerminal { pins { required A: pin required B: pin } }
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
impl TwoTerminal for Res {}
pub footprint TFP {}
pub part R1: Res { primary { mfr: "m", mpn: "n", footprint: TFP } }
pub fn sub(x: Pin) {
    inst r: R1
    net _: x, r.A
    layout { place r at (0mm, 0mm) }
}
design Board {
    inst r2: R1
    net N: r2.A
    sub(r2.B)
}
"#;
    let r = check_err(src);
    assert!(r.contains("E1007") && r.contains("fn"), "{}", r);
}

#[test]
fn fmt_round_trips_place() {
    let src = "design B{layout{place hdr at (0mm,-1mm)}}";
    let once = format_source("b.cohdl", src).unwrap();
    assert!(once.contains("place hdr at (0mm, -1mm)"), "{}", once);
    let twice = format_source("b.cohdl", &once).unwrap();
    assert_eq!(once, twice, "place formatting is not idempotent:\n{}", once);
}

#[test]
fn fmt_round_trips_board_outline() {
    let src = "design B{layout{board_outline{at:(0mm,-1mm),size:(51mm,21mm)}}}";
    let once = format_source("b.cohdl", src).unwrap();
    assert!(
        once.contains("board_outline { at: (0mm, -1mm), size: (51mm, 21mm) }"),
        "{}",
        once
    );
    let twice = format_source("b.cohdl", &once).unwrap();
    assert_eq!(
        once, twice,
        "board_outline formatting is not idempotent:\n{}",
        once
    );
}
