//! RFC-024 conformance: array-typed instances and indexed references.
//!
//! The RFC was REDESIGNED (2026-07-19) from its own withdrawn first draft. The
//! first draft made `inst sw[1..=13]: SW_KEY` pure name-expansion sugar with
//! indexing usable only inside a net's member list. The accepted design instead
//! makes `inst NAME: [Device; N]` one real array-typed instance whose elements
//! are addressable as `NAME[i]` EVERYWHERE an ordinary instance reference is
//! valid — net members, `place`, `decouple`, and `fn`-call arguments. The
//! range/list fan-out survives as sugar over that real mechanism, still scoped
//! to net-member lists.
//!
//! So the load-bearing tests here are `indexed_reference_works_in_place` and
//! `indexed_reference_works_in_fn_call_args` — the positions the first draft
//! could not serve, which are exactly why it was withdrawn.

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};

fn check(src: &str) -> (cohdl::pipeline::Checked, String) {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    (checked, rendered)
}

fn netlist(src: &str) -> String {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "clean build expected:\n{}",
        checked.diags.render(&checked.sm)
    );
    artifacts.expect("build").netlist
}

const LIB: &str = r#"
pub trait Sw { designator_prefix: "SW" }
pub trait Dio { designator_prefix: "D" }
pub device SwDev { pins { A: 1 [passive], B: 2 [passive] } }
pub device DioDev { pins { Anode: 1 [passive], Cathode: 2 [passive] } }
impl Sw for SwDev {}
impl Dio for DioDev {}
pub footprint FP {}
pub part SW_KEY: SwDev { primary { mfr: "m", mpn: "sw", footprint: FP } }
pub part D_1N4148W: DioDev { primary { mfr: "m", mpn: "d", footprint: FP } }
pub device McuDev { pins { ROW0: 1 [passive], ROW1: 2 [passive], COL0: 3 [passive], COL1: 4 [passive] } }
pub part MCU: McuDev { primary { mfr: "m", mpn: "u", footprint: FP } }
pub fn pair(a: Pin, b: Pin) {
    inst x: D_1N4148W
    net _: a, x.Anode
    net _: b, x.Cathode
}
"#;

// ---------------------------------------------------------------------------
// Declaration.

#[test]
fn array_typed_instance_declares_n_real_elements() {
    let src = format!(
        "{LIB}
design B {{
    inst sw: [SW_KEY; 4]
    inst mcu: MCU
    net ROW0: mcu.ROW0, sw[0].A, sw[1].A, sw[2].A, sw[3].A
    net ROW1: mcu.ROW1, sw[0].B, sw[1].B, sw[2].B, sw[3].B
    nc: mcu.COL0, mcu.COL1
}}"
    );
    let (checked, rendered) = check(&src);
    assert!(!rendered.contains("error"), "{}", rendered);
    assert_eq!(checked.ir.as_ref().expect("ir").instances.len(), 5); // 4 + mcu
                                                                     // Each element is fully real and gets its OWN designator — RFC-005 applies
                                                                     // to an array element exactly as to a hand-written instance.
    let net = netlist(&src);
    for d in ["SW1", "SW2", "SW3", "SW4"] {
        assert!(
            net.contains(&format!("(ref \"{}\")", d)),
            "missing designator {d}:\n{net}"
        );
    }
}

#[test]
fn two_arrays_of_the_same_device_are_independent() {
    // The RFC's own "two independent chains of the same LED part" case.
    let src = format!(
        "{LIB}
design B {{
    inst key_leds: [SW_KEY; 2]
    inst ambient_leds: [SW_KEY; 3]
    inst mcu: MCU
    net ROW0: mcu.ROW0, key_leds[0..=1].A, ambient_leds[0..=2].A
    net ROW1: mcu.ROW1, key_leds[0..=1].B, ambient_leds[0..=2].B
    nc: mcu.COL0, mcu.COL1
}}"
    );
    let (checked, rendered) = check(&src);
    assert!(!rendered.contains("error"), "{}", rendered);
    assert_eq!(checked.ir.as_ref().unwrap().instances.len(), 6);
}

#[test]
fn array_name_collision_is_e201() {
    let src = format!(
        "{LIB}
design B {{
    inst sw: [SW_KEY; 2]
    inst sw: [SW_KEY; 3]
    inst mcu: MCU
    net ROW0: mcu.ROW0, sw[0].A
}}"
    );
    let (_c, r) = check(&src);
    assert!(r.contains("E201"), "{}", r);
}

#[test]
fn non_positive_length_is_e211() {
    let src = format!("{LIB}\ndesign B {{\n    inst sw: [SW_KEY; 0]\n    inst mcu: MCU\n}}");
    let (_c, r) = check(&src);
    assert!(r.contains("E211") && r.contains("1 or more"), "{}", r);
}

// ---------------------------------------------------------------------------
// Indexed references — valid EVERYWHERE (the redesign's whole point).

#[test]
fn indexed_reference_works_in_place() {
    // The first draft could NOT do this. It is why the draft was withdrawn.
    let src = format!(
        "{LIB}
design B {{
    inst sw: [SW_KEY; 2]
    inst mcu: MCU
    net ROW0: mcu.ROW0, sw[0].A, sw[1].A
    net ROW1: mcu.ROW1, sw[0].B, sw[1].B
    nc: mcu.COL0, mcu.COL1
    layout {{
        place sw[0] at (-5mm, 0mm)
        place sw[1] at (5mm, 0mm) rotate 90
    }}
}}"
    );
    let (checked, rendered) = check(&src);
    assert!(!rendered.contains("error"), "{}", rendered);
    let layout = &checked.ir.as_ref().unwrap().layout;
    assert_eq!(layout.placements.len(), 2, "both elements placed");
}

#[test]
fn indexed_reference_works_in_fn_call_args() {
    // Also impossible in the first draft.
    let src = format!(
        "{LIB}
design B {{
    inst sw: [SW_KEY; 2]
    inst mcu: MCU
    pair(sw[0].A, sw[0].B)
    pair(sw[1].A, sw[1].B)
    nc: mcu.ROW0, mcu.ROW1, mcu.COL0, mcu.COL1
}}"
    );
    let (_c, rendered) = check(&src);
    assert!(!rendered.contains("error"), "{}", rendered);
}

#[test]
fn bare_unindexed_array_reference_is_rejected() {
    // "NAME alone is never itself a valid instance reference" — the one new
    // rule the RFC says must be taught.
    let src = format!(
        "{LIB}
design B {{
    inst sw: [SW_KEY; 2]
    inst mcu: MCU
    net ROW0: mcu.ROW0, sw.A
}}"
    );
    let (_c, r) = check(&src);
    assert!(r.contains("E211"), "{}", r);
    assert!(r.contains("array-typed"), "{}", r);
    assert!(r.contains("sw[0]"), "must suggest indexing:\n{}", r);
}

#[test]
fn indexing_a_non_array_is_rejected() {
    let src = format!(
        "{LIB}
design B {{
    inst mcu: MCU
    net ROW0: mcu[0].ROW0
}}"
    );
    let (_c, r) = check(&src);
    assert!(
        r.contains("E211") && r.contains("not an array-typed"),
        "{}",
        r
    );
}

#[test]
fn out_of_bounds_index_is_e202_naming_the_valid_range() {
    for (expr, where_) in [
        ("net ROW0: mcu.ROW0, sw[2].A", "net member"),
        ("layout { place sw[5] at (0mm, 0mm) }", "place"),
    ] {
        let src = format!(
            "{LIB}
design B {{
    inst sw: [SW_KEY; 2]
    inst mcu: MCU
    {expr}
}}"
        );
        let (_c, r) = check(&src);
        assert!(r.contains("E202"), "{where_}:\n{r}");
        assert!(
            r.contains("valid indices are 0..=1") && r.contains("length 2"),
            "{where_} must name the valid range:\n{r}"
        );
    }
}

// ---------------------------------------------------------------------------
// Range/list fan-out — sugar over the real mechanism, net-member-only.

#[test]
fn range_and_list_fan_out_in_net_members() {
    let src = format!(
        "{LIB}
design B {{
    inst d: [D_1N4148W; 13]
    inst mcu: MCU
    net COL0: mcu.COL0, d[0..=12 step 4].Cathode
    net COL1: mcu.COL1, d[1, 5, 9].Cathode
    net ROW0: mcu.ROW0, d[0..=12].Anode
    nc: mcu.ROW1,
        d[2].Cathode, d[3].Cathode, d[6].Cathode, d[7].Cathode,
        d[10].Cathode, d[11].Cathode
}}"
    );
    let net = netlist(&src);
    assert!(net.contains("COL0") && net.contains("COL1"), "{net}");
}

#[test]
fn fan_out_equals_hand_written_indexed_references() {
    // The sugar is defined as a pure textual expansion over NAME[i].
    let sugared = format!(
        "{LIB}
design B {{
    inst sw: [SW_KEY; 4]
    inst mcu: MCU
    net ROW0: mcu.ROW0, sw[0..=3].A
    net ROW1: mcu.ROW1, sw[0, 1, 2, 3].B
    nc: mcu.COL0, mcu.COL1
}}"
    );
    let explicit = format!(
        "{LIB}
design B {{
    inst sw: [SW_KEY; 4]
    inst mcu: MCU
    net ROW0: mcu.ROW0, sw[0].A, sw[1].A, sw[2].A, sw[3].A
    net ROW1: mcu.ROW1, sw[0].B, sw[1].B, sw[2].B, sw[3].B
    nc: mcu.COL0, mcu.COL1
}}"
    );
    assert_eq!(netlist(&sugared), netlist(&explicit));
}

#[test]
fn strided_equals_explicit_list() {
    let strided = format!(
        "{LIB}
design B {{
    inst d: [D_1N4148W; 13]
    inst mcu: MCU
    net COL0: mcu.COL0, d[0..=12 step 4].Cathode
    net COL1: mcu.COL1, d[0..=12 step 4].Anode
    nc: mcu.ROW0, mcu.ROW1,
        d[1].Anode, d[1].Cathode, d[2].Anode, d[2].Cathode, d[3].Anode, d[3].Cathode,
        d[5].Anode, d[5].Cathode, d[6].Anode, d[6].Cathode, d[7].Anode, d[7].Cathode,
        d[9].Anode, d[9].Cathode, d[10].Anode, d[10].Cathode, d[11].Anode, d[11].Cathode
}}"
    );
    let listed = strided
        .replace("d[0..=12 step 4].Cathode", "d[0, 4, 8, 12].Cathode")
        .replace("d[0..=12 step 4].Anode", "d[0, 4, 8, 12].Anode");
    assert_eq!(netlist(&strided), netlist(&listed));
}

#[test]
fn range_or_list_outside_net_members_is_e211() {
    // place and fn args take ONE element — "a range at once" has no meaning.
    for expr in [
        "layout { place sw[0..=1] at (0mm, 0mm) }",
        "pair(sw[0..=1].A, sw[0].B)",
        "nc: sw[0..=1].A",
    ] {
        let src = format!(
            "{LIB}
design B {{
    inst sw: [SW_KEY; 2]
    inst mcu: MCU
    {expr}
}}"
        );
        let (_c, r) = check(&src);
        assert!(r.contains("E211"), "`{expr}` must be rejected:\n{r}");
    }
}

// ---------------------------------------------------------------------------
// fmt.

#[test]
fn array_syntax_round_trips_through_fmt() {
    use cohdl::fmt::format_source;
    let src = "design B {\n    inst key_leds: [RGB; 13]\n    inst mcu: MCU\n    net V: mcu.A, key_leds[0].VDD, key_leds[0..=12].GND, key_leds[1, 5, 9].DIN\n    net W: key_leds[0..=12 step 4].DOUT\n    layout {\n        place key_leds[0] at (1mm, 2mm) rotate 90\n    }\n}\n";
    let once = format_source("b.cohdl", src).unwrap();
    for want in [
        "inst key_leds: [RGB; 13]",
        "key_leds[0].VDD",
        "key_leds[0..=12].GND",
        "key_leds[1, 5, 9].DIN",
        "key_leds[0..=12 step 4].DOUT",
        "place key_leds[0] at (1mm, 2mm) rotate 90",
    ] {
        assert!(once.contains(want), "fmt dropped `{want}`:\n{once}");
    }
    let twice = format_source("b.cohdl", &once).unwrap();
    assert_eq!(once, twice, "fmt not idempotent:\n{once}");
}

#[test]
fn unstrided_range_does_not_gain_a_step_in_fmt() {
    use cohdl::fmt::format_source;
    let src = "design B {\n    net N: mcu.A, sw[0..=3].A\n}\n";
    let once = format_source("b.cohdl", src).unwrap();
    assert!(once.contains("sw[0..=3].A"), "{}", once);
    assert!(
        !once.contains("step"),
        "implicit stride stays implicit:\n{}",
        once
    );
}
