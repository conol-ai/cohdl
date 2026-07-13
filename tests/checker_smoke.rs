//! Smoke tests for the semantic pipeline (declaration checks + expansion).
//! The exit-criteria fixtures live in tests/fixtures.rs; these are the fast
//! development-loop checks.

use cohdl::check::{check_declarations, check_design};
use cohdl::diag::Diagnostics;
use cohdl::span::SourceMap;

fn compile(src: &str) -> (cohdl::resolve::World, Diagnostics, SourceMap) {
    let mut sm = SourceMap::new();
    let f = sm.add_file("test.cohdl", src);
    let mut diags = Diagnostics::new();
    let tokens = cohdl::lex::lex(f, src, &mut diags);
    let file = cohdl::parse::parse(tokens, &mut diags);
    let world = check_declarations(vec![file], &mut diags);
    (world, diags, sm)
}

fn compile_design(
    src: &str,
    design: &str,
) -> (Option<cohdl::ir::DesignIr>, Diagnostics, SourceMap) {
    let (world, mut diags, sm) = compile(src);
    let ir = check_design(&world, design, &mut diags);
    (ir, diags, sm)
}

const STD_SNIPPET: &str = r#"
pub trait TwoTerminal {
    designator_prefix: "U"
    pins {
        required A: pin
        required B: pin
    }
}

pub trait Capacitor: TwoTerminal {
    designator_prefix: "C"
    spec {
        capacitance: Capacitance
        voltage_rating: Voltage
    }
}

pub device MLCC<C: Capacitance, V: Voltage = 10V> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C, voltage_rating: V }
}

impl TwoTerminal for MLCC {}
impl Capacitor for MLCC {}
"#;

#[test]
fn clean_design_type_checks() {
    let src = format!(
        r#"{STD_SNIPPET}
pub device MCU {{
    pins {{
        required VDD: 1 [power_in]
        required GND: 2 [passive]
        optional TEST: 3 [passive]
    }}
}}

design Board {{
    inst mcu: MCU
    inst c1: MLCC<100nF, 16V>
    net VDD [3.3V]: mcu.VDD, c1.A
    net GND [gnd]: mcu.GND, c1.B
}}
"#
    );
    let (ir, diags, sm) = compile_design(&src, "Board");
    assert!(!diags.has_errors(), "{}", diags.render(&sm));
    let ir = ir.unwrap();
    assert_eq!(ir.instances.len(), 2);
    assert_eq!(ir.nets.len(), 2);
    let c1 = &ir.instances["Board::c1"];
    assert_eq!(c1.specs["capacitance"].text, "100nF");
    assert_eq!(c1.specs["voltage_rating"].text, "16V");
    assert!(c1.impl_traits.contains("Capacitor"));
}

#[test]
fn nested_fn_expansion_two_levels() {
    let src = format!(
        r#"{STD_SNIPPET}
pub device Ferrite_Bead {{
    pins {{ IN: 1 [passive], OUT: 2 [passive] }}
}}

pub device MCU {{
    pins {{ required VDD: 1 [passive] }}
}}

fn decoupling_cap<V: Voltage>(pin: Pin) {{
    inst c: MLCC<100nF, V>
    net _: pin, c.A
    nc: c.B
}}

fn power_rail<V: Voltage>(vdd_pin: Pin) {{
    inst ferrite: Ferrite_Bead
    net _: vdd_pin, ferrite.IN
    decoupling_cap::<V>(ferrite.OUT)
}}

design Board {{
    inst mcu: MCU
    power_rail::<3.3V>(mcu.VDD)
}}
"#
    );
    let (ir, diags, sm) = compile_design(&src, "Board");
    assert!(!diags.has_errors(), "{}", diags.render(&sm));
    let ir = ir.unwrap();
    // mcu + ferrite + nested cap.
    assert_eq!(ir.instances.len(), 3, "{:#?}", ir.instances.keys());
    let cap_path = ir
        .instances
        .keys()
        .find(|k| k.contains("decoupling_cap"))
        .expect("nested instance exists");
    // Substitution threaded outward-in: V resolved to 3.3V.
    assert_eq!(ir.instances[cap_path].specs["voltage_rating"].text, "3.3V");
    // Call-chain naming.
    assert!(
        cap_path.starts_with("Board::__fn0_power_rail::__fn1_decoupling_cap::"),
        "path was {}",
        cap_path
    );
}

#[test]
fn cyclic_fn_call_detected() {
    let src = r#"
fn a(pin: Pin) { b(pin) }
fn b(pin: Pin) { a(pin) }
pub device MCU { pins { required VDD: 1 [passive] } }
design Board {
    inst mcu: MCU
    a(mcu.VDD)
}
"#;
    let (_, diags, sm) = compile_design(src, "Board");
    let rendered = diags.render(&sm);
    assert!(rendered.contains("E501"), "{}", rendered);
    assert!(rendered.contains("a → b → a"), "{}", rendered);
}

#[test]
fn unresolved_required_pin() {
    let src = r#"
pub device MCU {
    pins {
        required VDD: 1 [passive]
        required GND: 2 [passive]
    }
}
design Board {
    inst mcu: MCU
    net VDD [3.3V]: mcu.VDD, mcu.VDD
}
"#;
    let (_, diags, sm) = compile_design(src, "Board");
    let rendered = diags.render(&sm);
    assert!(rendered.contains("E701"), "{}", rendered);
    assert!(rendered.contains("Board::mcu.GND"), "{}", rendered);
}

#[test]
fn contradictory_net_and_nc() {
    let src = r#"
pub device MCU {
    pins { required VDD: 1 [passive] }
}
design Board {
    inst mcu: MCU
    net VDD: mcu.VDD, mcu.VDD
    nc: mcu.VDD
}
"#;
    let (_, diags, sm) = compile_design(src, "Board");
    let rendered = diags.render(&sm);
    assert!(rendered.contains("E702"), "{}", rendered);
}

#[test]
fn wrong_unit_generic_arg() {
    let src = format!(
        r#"{STD_SNIPPET}
design Board {{
    inst c1: MLCC<16V, 100nF>
    net X: c1.A, c1.B
}}
"#
    );
    let (_, diags, sm) = compile_design(&src, "Board");
    let rendered = diags.render(&sm);
    assert!(rendered.contains("E402"), "{}", rendered);
    assert!(
        rendered.contains("expected `Capacitance`, found `Voltage`"),
        "{}",
        rendered
    );
}

#[test]
fn missing_subtrait_impl_chain_diag() {
    let src = r#"
pub trait TwoTerminal {
    pins { required A: pin, required B: pin }
}
pub trait Capacitor: TwoTerminal {
    spec { capacitance: Capacitance }
}
pub device Cap<C: Capacitance> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C }
}
impl Capacitor for Cap {}
"#;
    let (_, diags, sm) = compile(src);
    let rendered = diags.render(&sm);
    assert!(rendered.contains("E302"), "{}", rendered);
    assert!(
        rendered.contains("requires `impl TwoTerminal for Cap`"),
        "{}",
        rendered
    );
}

#[test]
fn impl_mapping_and_trait_role_access() {
    let src = r#"
pub trait TwoTerminal {
    pins { required A: pin, required B: pin }
}
pub device TantalumCap<C: Capacitance> {
    pins { Anode: 1 [passive], Cathode: 2 [passive] }
    spec { capacitance: C }
}
impl TwoTerminal for TantalumCap {
    pins { A: Anode, B: Cathode }
}
pub device MCU { pins { required VDD: 1 [passive] } }

fn hook<D: TwoTerminal>(target: D, pin: Pin) {
    net _: pin, target.A
    nc: target.B
}

design Board {
    inst mcu: MCU
    inst c: TantalumCap<10uF>
    hook::<TantalumCap>(c, mcu.VDD)
}
"#;
    let (ir, diags, sm) = compile_design(src, "Board");
    assert!(!diags.has_errors(), "{}", diags.render(&sm));
    let ir = ir.unwrap();
    // target.A resolved through the impl mapping to the Anode pin.
    let net = ir
        .nets
        .iter()
        .find(|n| n.members.contains(&("Board::c".into(), "Anode".into())))
        .expect("net with mapped pin");
    assert!(net.members.contains(&("Board::mcu".into(), "VDD".into())));
    assert!(ir.nc_pins.contains(&("Board::c".into(), "Cathode".into())));
}

#[test]
fn trait_bound_violation_at_instantiation() {
    let src = r#"
pub trait Capacitor {
    spec { capacitance: Capacitance }
}
pub device Resistor<R: Resistance> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { resistance: R }
}
pub device MCU { pins { required VDD: 1 [passive] } }

fn add_decoupling<D: Capacitor>(target: D, pin: Pin) {
    net _: pin, target.A
}

design Board {
    inst mcu: MCU
    inst r1: Resistor<10kohm>
    add_decoupling::<Resistor>(r1, mcu.VDD)
    net X: r1.A, r1.B
}
"#;
    let (_, diags, sm) = compile_design(src, "Board");
    let rendered = diags.render(&sm);
    assert!(rendered.contains("E403"), "{}", rendered);
    assert!(
        rendered.contains("`Resistor` does not implement `Capacitor`"),
        "{}",
        rendered
    );
}

#[test]
fn same_fn_two_call_sites_no_collision() {
    let src = format!(
        r#"{STD_SNIPPET}
pub device MCU {{
    pins {{ required VDD: 1 [passive], required VDDA: 2 [passive] }}
}}

fn decoupling_cap<V: Voltage>(pin: Pin) {{
    inst c: MLCC<100nF, V>
    net _: pin, c.A
    nc: c.B
}}

design Board {{
    inst mcu: MCU
    decoupling_cap::<3.3V>(mcu.VDD)
    decoupling_cap::<5V>(mcu.VDDA)
}}
"#
    );
    let (ir, diags, sm) = compile_design(&src, "Board");
    assert!(!diags.has_errors(), "{}", diags.render(&sm));
    let ir = ir.unwrap();
    let caps: Vec<&String> = ir
        .instances
        .keys()
        .filter(|k| k.contains("decoupling_cap"))
        .collect();
    assert_eq!(
        caps.len(),
        2,
        "distinct call sites must not collide: {caps:?}"
    );
}

#[test]
fn duplicate_impl_rejected() {
    let src = r#"
pub trait T { pins { required A: pin } }
pub device D { pins { A: 1 [passive] } }
impl T for D {}
impl T for D {}
"#;
    let (_, diags, sm) = compile(src);
    let rendered = diags.render(&sm);
    assert!(rendered.contains("E303"), "{}", rendered);
}
