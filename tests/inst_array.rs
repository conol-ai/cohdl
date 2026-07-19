//! RFC-024 conformance: instance arrays and range references.
//!
//! The RFC defines BOTH constructs purely as expansion sugar — an exact
//! equivalence to fully-hand-written `inst`/`net` statements. So the load-
//! bearing test here is `expansion_equivalence_is_byte_identical`: the sugared
//! design and the hand-written one must emit the same netlist bytes. Everything
//! else checks the grammar, the two structural diagnostics, and fmt round-trip.

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

/// Shared preamble: a two-pin switch-ish device and a diode-ish device.
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
"#;

// ---------------------------------------------------------------------------
// Instance arrays.

#[test]
fn array_declares_individually_addressable_instances() {
    let src = format!(
        "{LIB}
design B {{
    inst sw[1..=4]: SW_KEY
    inst mcu: MCU
    net ROW0: mcu.ROW0, sw1.A, sw2.A, sw3.A, sw4.A
    net ROW1: mcu.ROW1, sw1.B, sw2.B, sw3.B, sw4.B
    nc: mcu.COL0, mcu.COL1
}}"
    );
    let (checked, rendered) = check(&src);
    assert!(!rendered.contains("error"), "{}", rendered);
    let ir = checked.ir.as_ref().expect("ir");
    // Four real instances, each hierarchically named exactly as hand-typed.
    for n in ["sw1", "sw2", "sw3", "sw4"] {
        assert!(
            ir.instances.keys().any(|k| k.ends_with(n)),
            "missing expanded instance {n}: {:?}",
            ir.instances.keys().collect::<Vec<_>>()
        );
    }
}

#[test]
fn array_start_need_not_be_one() {
    // The RFC calls this out explicitly: `inst led[14..=29]` is a real second
    // array of the same device with a different starting number.
    let src = format!(
        "{LIB}
design B {{
    inst sw[1..=2]: SW_KEY
    inst sw[14..=15]: SW_KEY
    inst mcu: MCU
    net ROW0: mcu.ROW0, sw1.A, sw2.A, sw14.A, sw15.A
    net ROW1: mcu.ROW1, sw1.B, sw2.B, sw14.B, sw15.B
    nc: mcu.COL0, mcu.COL1
}}"
    );
    let (checked, rendered) = check(&src);
    assert!(!rendered.contains("error"), "{}", rendered);
    let ir = checked.ir.as_ref().expect("ir");
    assert_eq!(ir.instances.len(), 5); // 2 + 2 + mcu
}

#[test]
fn overlapping_arrays_collide() {
    let src = format!(
        "{LIB}
design B {{
    inst sw[1..=4]: SW_KEY
    inst sw[3..=6]: SW_KEY
    inst mcu: MCU
    net ROW0: mcu.ROW0, sw1.A
}}"
    );
    let (_c, rendered) = check(&src);
    // Overlap collides on the SHARED element names — the same E201 an author
    // would get from two ordinary `inst` statements sharing a name.
    assert!(rendered.contains("E201"), "{}", rendered);
    assert!(
        rendered.contains("sw3"),
        "must name the colliding element:\n{}",
        rendered
    );
}

#[test]
fn array_collides_with_ordinary_inst() {
    let src = format!(
        "{LIB}
design B {{
    inst sw2: SW_KEY
    inst sw[1..=3]: SW_KEY
    inst mcu: MCU
    net ROW0: mcu.ROW0, sw1.A
}}"
    );
    let (_c, rendered) = check(&src);
    assert!(rendered.contains("E201"), "{}", rendered);
}

// ---------------------------------------------------------------------------
// Range references in net-member lists.

#[test]
fn range_stride_and_list_references_expand() {
    let src = format!(
        "{LIB}
design B {{
    inst sw[1..=4]: SW_KEY
    inst d[1..=13]: D_1N4148W
    inst mcu: MCU
    net ROW0: mcu.ROW0, sw[1..=4].A
    net ROW1: mcu.ROW1, sw[1..=4].B
    net COL0: mcu.COL0, d[1..=13 step 4].Cathode
    net COL1: mcu.COL1, d[2, 6, 10].Cathode
    nc: d1.Anode, d2.Anode, d3.Anode, d4.Anode, d5.Anode, d6.Anode, d7.Anode,
        d8.Anode, d9.Anode, d10.Anode, d11.Anode, d12.Anode, d13.Anode,
        d3.Cathode, d4.Cathode, d7.Cathode, d8.Cathode, d11.Cathode, d12.Cathode
}}"
    );
    let net = netlist(&src);
    // ROW0 got all four switch A pins.
    for r in ["SW1", "SW2", "SW3", "SW4"] {
        assert!(net.contains(r), "expected {r} in netlist:\n{net}");
    }
    // The strided form selected d1, d5, d9, d13 — the RFC's own example.
    assert!(net.contains("COL0"), "{net}");
    assert!(net.contains("COL1"), "{net}");
}

#[test]
fn expansion_equivalence_is_byte_identical() {
    // The RFC's core claim: sugar expands to exactly what an author would have
    // hand-written. Same designators, same nets, same NETLIST BYTES.
    let sugared = format!(
        "{LIB}
design B {{
    inst sw[1..=4]: SW_KEY
    inst mcu: MCU
    net ROW0: mcu.ROW0, sw[1..=4].A
    net ROW1: mcu.ROW1, sw[1, 2, 3, 4].B
    nc: mcu.COL0, mcu.COL1
}}"
    );
    let hand = format!(
        "{LIB}
design B {{
    inst sw1: SW_KEY
    inst sw2: SW_KEY
    inst sw3: SW_KEY
    inst sw4: SW_KEY
    inst mcu: MCU
    net ROW0: mcu.ROW0, sw1.A, sw2.A, sw3.A, sw4.A
    net ROW1: mcu.ROW1, sw1.B, sw2.B, sw3.B, sw4.B
    nc: mcu.COL0, mcu.COL1
}}"
    );
    assert_eq!(
        netlist(&sugared),
        netlist(&hand),
        "array/range sugar must be byte-identical to the hand-written form"
    );
}

#[test]
fn strided_equals_explicit_list() {
    let strided = format!(
        "{LIB}
design B {{
    inst d[1..=13]: D_1N4148W
    inst mcu: MCU
    net COL0: mcu.COL0, d[1..=13 step 4].Cathode
    net COL1: mcu.COL1, d[1..=13 step 4].Anode
    nc: mcu.ROW0, mcu.ROW1,
        d2.Anode, d2.Cathode, d3.Anode, d3.Cathode, d4.Anode, d4.Cathode,
        d6.Anode, d6.Cathode, d7.Anode, d7.Cathode, d8.Anode, d8.Cathode,
        d10.Anode, d10.Cathode, d11.Anode, d11.Cathode, d12.Anode, d12.Cathode
}}"
    );
    let listed = strided
        .replace("d[1..=13 step 4].Cathode", "d[1, 5, 9, 13].Cathode")
        .replace("d[1..=13 step 4].Anode", "d[1, 5, 9, 13].Anode");
    assert_eq!(netlist(&strided), netlist(&listed));
}

#[test]
fn out_of_range_index_is_e202_naming_the_first_bad_one() {
    let src = format!(
        "{LIB}
design B {{
    inst sw[1..=4]: SW_KEY
    inst mcu: MCU
    net ROW0: mcu.ROW0, sw[1..=6].A
}}"
    );
    let (_c, rendered) = check(&src);
    assert!(rendered.contains("E202"), "{}", rendered);
    // Names the FIRST invalid index (5), not every one of them.
    assert!(rendered.contains("index 5"), "{}", rendered);
    assert!(rendered.contains("sw5"), "{}", rendered);
    assert_eq!(
        rendered.matches("E202").count(),
        1,
        "one mistyped range must yield ONE diagnostic:\n{}",
        rendered
    );
}

// ---------------------------------------------------------------------------
// RFC-024's explicit scope boundary + malformed grammar (E211).

#[test]
fn index_selector_outside_net_members_is_e211() {
    // `nc` is not a net-member list.
    let src = format!(
        "{LIB}
design B {{
    inst sw[1..=4]: SW_KEY
    inst mcu: MCU
    net ROW0: mcu.ROW0, sw[1..=4].A
    nc: sw[1..=4].B
}}"
    );
    let (_c, rendered) = check(&src);
    assert!(rendered.contains("E211"), "{}", rendered);
    assert!(
        rendered.contains("only valid in a net's member list"),
        "{}",
        rendered
    );
}

#[test]
fn malformed_ranges_are_e211() {
    let base = |body: &str| format!("{LIB}\ndesign B {{\n    inst mcu: MCU\n{body}\n}}");
    // Empty range: end below start.
    let (_c, r) = check(&base("    inst sw[4..=1]: SW_KEY"));
    assert!(r.contains("E211") && r.contains("empty"), "{}", r);
    // Stride below 1.
    let (_c, r) = check(&base(
        "    inst sw[1..=4]: SW_KEY\n    net N: mcu.ROW0, sw[1..=4 step 0].A",
    ));
    assert!(r.contains("E211") && r.contains("stride"), "{}", r);
    // A stride in a DECLARATION is not a thing — arrays are contiguous.
    let (_c, r) = check(&base("    inst sw[1..=4 step 2]: SW_KEY"));
    assert!(r.contains("E211") && r.contains("contiguous"), "{}", r);
    // Nor is an index list.
    let (_c, r) = check(&base("    inst sw[1, 2, 3]: SW_KEY"));
    assert!(
        r.contains("E211") && r.contains("not an index list"),
        "{}",
        r
    );
}

// ---------------------------------------------------------------------------
// fmt (the construct-tracking trap: a formatter that silently drops the
// bracket would expand a 13-element array into ONE instance on reformat).

#[test]
fn arrays_and_ranges_round_trip_through_fmt() {
    use cohdl::fmt::format_source;
    let src = "design B {\n    inst sw[1..=13]: SW_KEY\n    inst led[14..=29]: RGB\n    net ROW0: mcu.ROW0, sw[1..=4].A\n    net COL0: mcu.COL0, d[1..=13 step 4].Cathode\n    net COL1: mcu.COL1, d[2, 6, 10].Cathode\n}\n";
    let once = format_source("b.cohdl", src).unwrap();
    for want in [
        "inst sw[1..=13]: SW_KEY",
        "inst led[14..=29]: RGB",
        "sw[1..=4].A",
        "d[1..=13 step 4].Cathode",
        "d[2, 6, 10].Cathode",
    ] {
        assert!(once.contains(want), "fmt dropped `{want}`:\n{once}");
    }
    let twice = format_source("b.cohdl", &once).unwrap();
    assert_eq!(once, twice, "fmt not idempotent:\n{once}");
}

#[test]
fn unstrided_range_does_not_gain_a_step_in_fmt() {
    use cohdl::fmt::format_source;
    let src = "design B {\n    net N: mcu.A, sw[1..=4].A\n}\n";
    let once = format_source("b.cohdl", src).unwrap();
    assert!(once.contains("sw[1..=4].A"), "{}", once);
    assert!(
        !once.contains("step"),
        "implicit stride must stay implicit:\n{}",
        once
    );
}
