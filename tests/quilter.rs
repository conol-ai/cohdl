//! RFC-027 conformance: Quilter physics-constraint attributes + CSV export.
//!
//! The hard contract is the CSV file set: headers and column order match the
//! real supplied template files exactly, scales are the templates' own (mA,
//! nF, ohm, GHz), pins are PAD numbers with multi-pad pins flattened to one
//! row per pad, and a design with no physics facts emits NO files (byte-compat
//! with every pre-RFC-027 build).

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};
use std::collections::BTreeMap;

fn check(src: &str) -> (cohdl::pipeline::Checked, String) {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    (checked, rendered)
}

fn build_csvs(src: &str) -> Option<BTreeMap<String, String>> {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "clean build expected:\n{}",
        checked.diags.render(&checked.sm)
    );
    artifacts
        .expect("build")
        .quilter
        .map(|v| v.into_iter().collect())
}

/// MCU-ish device with a MULTI-PAD pin (VDD: 2, 6) + crystal pins, a cap, a
/// crystal, and a converter trio — every attribute kind exercised.
const LIB: &str = r#"
pub trait Ic { designator_prefix: "U" }
pub trait Cap { designator_prefix: "C" }
pub trait Ind { designator_prefix: "L" }
pub trait Crystal { designator_prefix: "Y" }
pub footprint FP {}
pub device Mcu { pins { VDD: 2, 6 [power_in], GND: 4 [power_in], XIN: 3 [passive], XOUT: 5 [passive], DP: 1 [bidirectional], DM: 7 [bidirectional] } }
impl Ic for Mcu {}
pub part MCU: Mcu { primary { mfr: "m", mpn: "mcu", footprint: FP } }
pub device Cap2 { pins { A: 1 [passive], B: 2 [passive] } }
impl Cap for Cap2 {}
pub part C100N: Cap2 { primary { mfr: "m", mpn: "c", footprint: FP } }
pub device Ind2 { pins { A: 1 [passive], B: 2 [passive] } }
impl Ind for Ind2 {}
pub part L2U2: Ind2 { primary { mfr: "m", mpn: "l", footprint: FP } }
pub device Xtal { pins { XIN: 1 [passive], XOUT: 3 [passive], GND: 2, 4 [passive] } }
impl Crystal for Xtal {}
pub part X8M: Xtal { primary { mfr: "m", mpn: "y", footprint: FP } }
pub device Buck { pins { VIN: 1 [power_in], SW: 2 [output], GND: 3 [power_in] } }
impl Ic for Buck {}
pub part BUCK: Buck { primary { mfr: "m", mpn: "b", footprint: FP } }
"#;

const BODY: &str = r#"
design B {
    #[bga_fanout]
    inst mcu: MCU
    #[bypass(mcu.VDD, 100nF)]
    inst c1: C100N
    #[crystal_oscillator(mcu, XIN, XOUT)]
    inst y1: X8M
    #[switching_converter(inductor: l1, input_capacitor: c1, output_capacitor: c2)]
    inst u2: BUCK
    inst l1: L2U2
    inst c2: C100N

    #[ground(primary)]
    net GND: mcu.GND, c1.B, c2.B, y1.GND, u2.GND
    #[high_current(500mA, power_pour)]
    net VIN: u2.VIN, c1.A
    #[impedance(50ohm, frequency: 1GHz)]
    net SW: u2.SW, l1.A
    net XI: mcu.XIN, y1.XIN
    net XO: mcu.XOUT, y1.XOUT
    net DP: mcu.DP, l1.B
    net DM: mcu.DM, c2.A
    net VDD_N: mcu.VDD, y1.GND

    layout {
        diff_pair(DP, DM) [differential_impedance: 100ohm, single_ended_impedance: 50ohm, frequency: 1GHz]
    }
}
"#;

fn full() -> String {
    format!("{LIB}{BODY}")
}

// ---------------------------------------------------------------------------
// The CSV contract.

#[test]
fn csv_set_matches_the_supplied_templates() {
    let csvs = build_csvs(&full()).expect("physics facts present");
    assert_eq!(csvs.len(), 8, "all eight files, always, as a set");
    // Headers are the supplied templates', byte for byte.
    assert!(csvs["bga_components.csv"].starts_with("component,generate_fanout\n"));
    assert!(csvs["bypass_capacitors.csv"]
        .starts_with("capacitor,bypassed_component,bypassed_pin,capacitance\n"));
    assert!(csvs["crystal_oscillators.csv"]
        .starts_with("crystal,crystal_parent,parent_signal_pin_1,parent_signal_pin_2\n"));
    assert!(csvs["differential_pairs.csv"].starts_with(
        "positive_net_name,negative_net_name,differential_impedance,single_ended_impedance,frequency\n"
    ));
    assert!(csvs["ground_nets.csv"].starts_with("net_name,is_primary,use_region_pour\n"));
    assert!(csvs["high_current_nets.csv"].starts_with("net_name,max_current,use_power_pour\n"));
    assert!(csvs["single_ended_impedance_signals.csv"]
        .starts_with("net_name,single_ended_impedance,frequency\n"));
    assert!(csvs["switching_converters.csv"]
        .starts_with("switching_converter,inductor,input_capacitor,output_capacitor\n"));
}

#[test]
fn csv_rows_scale_and_flatten_like_the_templates() {
    let csvs = build_csvs(&full()).expect("csvs");
    // Multi-pad pin flattens: VDD is pads 2 AND 6 -> two bypass rows, nF scale.
    assert_eq!(
        csvs["bypass_capacitors.csv"],
        "capacitor,bypassed_component,bypassed_pin,capacitance\nC1,U1,2,100\nC1,U1,6,100\n"
    );
    // Crystal pins are PAD numbers of the parent (XIN=3, XOUT=5).
    assert_eq!(
        csvs["crystal_oscillators.csv"],
        "crystal,crystal_parent,parent_signal_pin_1,parent_signal_pin_2\nY1,U1,3,5\n"
    );
    // mA scale + lowercase booleans.
    assert_eq!(
        csvs["high_current_nets.csv"],
        "net_name,max_current,use_power_pour\nVIN,500,true\n"
    );
    assert_eq!(
        csvs["ground_nets.csv"],
        "net_name,is_primary,use_region_pour\nGND,true,false\n"
    );
    // ohm + GHz scales.
    assert_eq!(
        csvs["single_ended_impedance_signals.csv"],
        "net_name,single_ended_impedance,frequency\nSW,50,1\n"
    );
    assert_eq!(
        csvs["differential_pairs.csv"],
        "positive_net_name,negative_net_name,differential_impedance,single_ended_impedance,frequency\nDP,DM,100,50,1\n"
    );
    assert_eq!(
        csvs["switching_converters.csv"],
        "switching_converter,inductor,input_capacitor,output_capacitor\nU2,L1,C1,C2\n"
    );
    assert_eq!(
        csvs["bga_components.csv"],
        "component,generate_fanout\nU1,true\n"
    );
}

#[test]
fn no_physics_facts_emits_no_files() {
    let src = full();
    // Strip every attribute and the diff_pair bracket.
    let stripped: String = src
        .lines()
        .filter(|l| !l.trim_start().starts_with("#["))
        .collect::<Vec<_>>()
        .join("\n")
        .replace(
            " [differential_impedance: 100ohm, single_ended_impedance: 50ohm, frequency: 1GHz]",
            "",
        );
    assert!(
        build_csvs(&stripped).is_none(),
        "pre-RFC-027 builds stay file-identical"
    );
}

// ---------------------------------------------------------------------------
// Diagnostics.

#[test]
fn duplicate_primary_ground_is_e1009() {
    let src = full().replace(
        "net XI: mcu.XIN, y1.XIN",
        "#[ground(primary)]\n    net XI: mcu.XIN, y1.XIN",
    );
    let (_c, r) = check(&src);
    assert!(r.contains("E1009"), "{r}");
    assert!(r.contains("at most one `#[ground(primary)]`"), "{r}");
}

#[test]
fn unresolved_references_are_e1009() {
    // Unknown instance.
    let src = full().replace("#[bypass(mcu.VDD, 100nF)]", "#[bypass(nosuch.VDD, 100nF)]");
    let (_c, r) = check(&src);
    assert!(
        r.contains("E1009") && r.contains("`nosuch` is not an instance"),
        "{r}"
    );
    // Unknown pin.
    let src = full().replace("#[bypass(mcu.VDD, 100nF)]", "#[bypass(mcu.NOPE, 100nF)]");
    let (_c, r) = check(&src);
    assert!(r.contains("E1009") && r.contains("no pin `NOPE`"), "{r}");
}

#[test]
fn wrong_target_and_bad_values_are_diagnosed() {
    // Net attr on an inst.
    let src = full().replace("#[bga_fanout]", "#[ground(primary)]");
    let (_c, r) = check(&src);
    assert!(
        r.contains("E1009") && r.contains("belongs on a `net` declaration"),
        "{r}"
    );
    // Bad ground kind.
    let src = full().replace("#[ground(primary)]", "#[ground(tertiary)]");
    let (_c, r) = check(&src);
    assert!(
        r.contains("E1009") && r.contains("primary, secondary"),
        "{r}"
    );
    // Unit mismatch is E110, naming expected vs actual (RFC-011 discipline).
    let src = full().replace("#[high_current(500mA, power_pour)]", "#[high_current(5V)]");
    let (_c, r) = check(&src);
    assert!(r.contains("E110") && r.contains("Current"), "{r}");
    // Missing required inductor.
    let src = full().replace(
        "#[switching_converter(inductor: l1, input_capacitor: c1, output_capacitor: c2)]",
        "#[switching_converter(input_capacitor: c1)]",
    );
    let (_c, r) = check(&src);
    assert!(
        r.contains("E1009") && r.contains("requires the `inductor:`"),
        "{r}"
    );
}

// ---------------------------------------------------------------------------
// fmt.

#[test]
fn attributes_and_diff_pair_bracket_round_trip_through_fmt() {
    use cohdl::fmt::format_source;
    let once = format_source("m.cohdl", &full()).unwrap();
    for want in [
        "#[bga_fanout]",
        "#[bypass(mcu.VDD, 100nF)]",
        "#[crystal_oscillator(mcu, XIN, XOUT)]",
        "#[switching_converter(inductor: l1, input_capacitor: c1, output_capacitor: c2)]",
        "#[ground(primary)]",
        "#[high_current(500mA, power_pour)]",
        "#[impedance(50ohm, frequency: 1GHz)]",
        "[differential_impedance: 100ohm, single_ended_impedance: 50ohm, frequency: 1GHz]",
    ] {
        assert!(once.contains(want), "fmt dropped `{want}`:\n{once}");
    }
    let twice = format_source("m.cohdl", &once).unwrap();
    assert_eq!(once, twice, "fmt not idempotent:\n{once}");
}

// ---------------------------------------------------------------------------
// RFC-028: attributes on fn Pin/Instance parameters.

#[test]
fn bypass_on_fn_pin_parameter_yields_one_row_per_call_site() {
    let src = format!(
        "{LIB}
pub fn dec(vdd: Pin, gnd: Pin) {{
    #[bypass(vdd, 100nF)]
    inst c: C100N
    net _: vdd, c.A
    net _: gnd, c.B
}}
design B {{
    inst mcu: MCU
    dec(mcu.VDD, mcu.GND)
    dec(mcu.DP, mcu.GND)
    nc: mcu.XIN, mcu.XOUT, mcu.DM
}}"
    );
    let csvs = build_csvs(&src).expect("csvs");
    // Call site 1 targets VDD (pads 2 AND 6 -> two rows); call site 2 targets
    // DP (pad 1). One independent resolution per call site, RFC-006 style.
    assert_eq!(
        csvs["bypass_capacitors.csv"],
        "capacitor,bypassed_component,bypassed_pin,capacitance\nC1,U1,2,100\nC1,U1,6,100\nC2,U1,1,100\n"
    );
}

#[test]
fn converter_on_fn_instance_parameters_resolves_per_call_site() {
    let src = format!(
        "{LIB}
pub fn phase(conv: impl Ic, l: impl Ind) {{
    #[switching_converter(inductor: l)]
    inst marker: C100N
    nc: marker.A, marker.B
}}
design B {{
    inst u9: BUCK
    inst l9: L2U2
    phase(u9, l9)
    net N1: u9.SW, l9.A
    net N2: u9.GND, l9.B
    nc: u9.VIN
}}"
    );
    let csvs = build_csvs(&src).expect("csvs");
    assert!(
        csvs["switching_converters.csv"].contains(",L1,,"),
        "inductor resolved through the Instance binding:\n{}",
        csvs["switching_converters.csv"]
    );
}

#[test]
fn unresolvable_bare_target_is_e1009() {
    let src = full().replace("#[bypass(mcu.VDD, 100nF)]", "#[bypass(nothing, 100nF)]");
    let (_c, r) = check(&src);
    assert!(
        r.contains("E1009")
            && r.contains("neither an `INST.PIN` reference nor a `Pin`-typed fn parameter"),
        "{r}"
    );
}

#[test]
fn bare_bypass_form_round_trips_through_fmt() {
    use cohdl::fmt::format_source;
    let src = "pub fn dec(vdd: Pin, gnd: Pin) {\n    #[bypass(vdd, 100nF)]\n    inst c: C100N\n    net _: vdd, c.A\n    net _: gnd, c.B\n}\n";
    let once = format_source("f.cohdl", src).unwrap();
    assert!(once.contains("#[bypass(vdd, 100nF)]"), "{once}");
    let twice = format_source("f.cohdl", &once).unwrap();
    assert_eq!(once, twice);
}
