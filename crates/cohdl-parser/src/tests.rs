use super::*;
use pest::consumes_to;
use pest::parses_to;
use pest::Parser;

// ─── Helper: assert a rule successfully parses input ───
macro_rules! assert_parses {
    ($rule:expr, $input:expr) => {
        let input = $input.trim();
        assert!(
            CohdlParser::parse($rule, input).is_ok(),
            "Failed to parse {:?} with rule {:?}:\n{}",
            input,
            $rule,
            CohdlParser::parse($rule, input).unwrap_err()
        );
    };
}

macro_rules! assert_no_parse {
    ($rule:expr, $input:expr) => {
        assert!(
            CohdlParser::parse($rule, $input).is_err(),
            "Should NOT parse {:?} with rule {:?}",
            $input,
            $rule,
        );
    };
}

// ═══════════════════════════════════════════════
// Identifiers & paths
// ═══════════════════════════════════════════════

#[test]
fn ident_simple() {
    assert_parses!(Rule::ident, "foo");
    assert_parses!(Rule::ident, "_bar");
    assert_parses!(Rule::ident, "VDD_IO");
    assert_parses!(Rule::ident, "STM32F103");
    assert_parses!(Rule::ident, "A");
    assert_parses!(Rule::ident, "_internal_helper");
}

#[test]
fn ident_rejects_leading_digit() {
    assert_no_parse!(Rule::ident, "123abc");
}

#[test]
fn scoped_path_basic() {
    assert_parses!(Rule::scoped_path, "foo::bar");
    assert_parses!(Rule::scoped_path, "power::decoupling");
    assert_parses!(Rule::scoped_path, "passives::res_10k_0402");
    assert_parses!(Rule::scoped_path, "stm32::STM32F103");
    assert_parses!(Rule::scoped_path, "a::b::c");
}

// ═══════════════════════════════════════════════
// Literals
// ═══════════════════════════════════════════════

#[test]
fn eng_number_various() {
    assert_parses!(Rule::eng_number, "100nF");
    assert_parses!(Rule::eng_number, "10k");
    assert_parses!(Rule::eng_number, "3.3V");
    assert_parses!(Rule::eng_number, "48");
    assert_parses!(Rule::eng_number, "22R");
    assert_parses!(Rule::eng_number, "100m");
    assert_parses!(Rule::eng_number, "4.7uF");
    assert_parses!(Rule::eng_number, "6.3V");
    assert_parses!(Rule::eng_number, "10uF");
    assert_parses!(Rule::eng_number, "0.8");
    assert_parses!(Rule::eng_number, "0.6");
    assert_parses!(Rule::eng_number, "16V");
}

#[test]
fn integer_basic() {
    assert_parses!(Rule::integer, "1");
    assert_parses!(Rule::integer, "48");
    assert_parses!(Rule::integer, "0");
    assert_parses!(Rule::integer, "12345");
}

#[test]
fn string_literal_basic() {
    assert_parses!(Rule::string_literal, r#""Samsung""#);
    assert_parses!(Rule::string_literal, r#""CL05B104KO5NNNC""#);
    assert_parses!(Rule::string_literal, r#""RC0402FR-0710KL""#);
    assert_parses!(Rule::string_literal, r#""U1""#);
    assert_parses!(Rule::string_literal, r#""""#);
}

#[test]
fn boolean_literals() {
    assert_parses!(Rule::boolean, "true");
    assert_parses!(Rule::boolean, "false");
}

// ═══════════════════════════════════════════════
// Pin values
// ═══════════════════════════════════════════════

#[test]
fn pin_value_single() {
    assert_parses!(Rule::pin_value, "1");
    assert_parses!(Rule::pin_value, "48");
}

#[test]
fn pin_value_list() {
    assert_parses!(Rule::pin_list, "[1, 2, 3]");
    assert_parses!(Rule::pin_list, "[8, 23, 35, 47]");
    assert_parses!(Rule::pin_value, "[8, 23, 35, 47]");
}

#[test]
fn pin_value_range() {
    assert_parses!(Rule::pin_range, "[10..17]");
    assert_parses!(Rule::pin_value, "[10..17]");
}

// ═══════════════════════════════════════════════
// Pin-bus macro
// ═══════════════════════════════════════════════

#[test]
fn pin_bus_call_basic() {
    assert_parses!(Rule::pin_bus_call, "pin_bus!(PA, 10, 8)");
    assert_parses!(Rule::pin_bus_call, "pin_bus!(PB, 26, 16)");
}

// ═══════════════════════════════════════════════
// Generic parameters & arguments
// ═══════════════════════════════════════════════

#[test]
fn generic_params_simple() {
    assert_parses!(Rule::generic_params, "<pkg: Package>");
    assert_parses!(Rule::generic_params, "<C: Farads>");
    assert_parses!(Rule::generic_params, "<R: Ohms, pkg: Package>");
}

#[test]
fn generic_params_with_defaults() {
    assert_parses!(Rule::generic_params, "<pkg: Package = LQFP48>");
    assert_parses!(
        Rule::generic_params,
        "<C: Farads, V: Voltage = 10V, pkg: Package = C0402>"
    );
    assert_parses!(Rule::generic_params, "<R: Ohms, P: Package = R0402>");
}

#[test]
fn generic_params_impl_constraint() {
    assert_parses!(Rule::generic_params, "<P: impl Capacitor>");
    assert_parses!(
        Rule::generic_params,
        "<P: impl Capacitor + Polarized>"
    );
}

#[test]
fn generic_args_basic() {
    assert_parses!(Rule::generic_args, "<C: 100nF>");
    assert_parses!(Rule::generic_args, "<C: 100nF, V: 10V, pkg: C0402>");
    assert_parses!(Rule::generic_args, "<R: 10k, pkg: R0402>");
    assert_parses!(Rule::generic_args, "<pkg: LQFP64>");
    assert_parses!(Rule::generic_args, "<R: 22R>");
    assert_parses!(Rule::generic_args, "<C: 10uF, V: 6.3V>");
}

// ═══════════════════════════════════════════════
// Type expressions
// ═══════════════════════════════════════════════

#[test]
fn type_expr_simple() {
    assert_parses!(Rule::type_expr, "Net");
    assert_parses!(Rule::type_expr, "Pin");
    assert_parses!(Rule::type_expr, "Package");
    assert_parses!(Rule::type_expr, "BypassCap");
}

#[test]
fn type_expr_with_generics() {
    assert_parses!(Rule::type_expr, "MLCC<C: 100nF, V: 10V, pkg: C0402>");
    assert_parses!(Rule::type_expr, "Resistor<R: 10k, pkg: R0402>");
    assert_parses!(Rule::type_expr, "Tantalum<C: 10uF, V: 6.3V>");
    assert_parses!(Rule::type_expr, "STM32F103<pkg: LQFP64>");
    assert_parses!(Rule::type_expr, "SmallCap<C: 4.7nF>");
    assert_parses!(Rule::type_expr, "SeriesRes<R: 22R>");
}

#[test]
fn type_expr_scoped_with_generics() {
    assert_parses!(Rule::type_expr, "stm32::STM32F103<pkg: LQFP64>");
}

// ═══════════════════════════════════════════════
// Attributes
// ═══════════════════════════════════════════════

#[test]
fn attribute_allow() {
    assert_parses!(Rule::attribute, "#[allow(unconnected_pin)]");
    assert_parses!(Rule::attribute, "#[allow(voltage_derating)]");
}

#[test]
fn attribute_designator() {
    assert_parses!(Rule::attribute, r#"#[designator("U1")]"#);
    assert_parses!(Rule::attribute, r#"#[designator("C1")]"#);
}

#[test]
fn attribute_no_args() {
    assert_parses!(Rule::attribute, "#[some_attr]");
}

// ═══════════════════════════════════════════════
// Trait definitions
// ═══════════════════════════════════════════════

#[test]
fn trait_basic_pins() {
    assert_parses!(
        Rule::trait_def,
        "trait TwoTerminal {
            pins {
                A: Pin
                B: Pin
            }
        }"
    );
}

#[test]
fn trait_with_parent() {
    assert_parses!(
        Rule::trait_def,
        "trait Capacitor: TwoTerminal {
            spec {
                capacitance: Farads
                voltage_rating: Voltage
            }
        }"
    );
}

#[test]
fn trait_with_designator_prefix() {
    assert_parses!(
        Rule::trait_def,
        r#"trait Capacitor: TwoTerminal {
            designator_prefix: "C"
            spec {
                capacitance: Farads
                voltage_rating: Voltage
            }
        }"#
    );
}

#[test]
fn trait_with_spec_boolean() {
    assert_parses!(
        Rule::trait_def,
        "trait Polarized: TwoTerminal {
            spec { polarity: true }
        }"
    );
}

#[test]
fn trait_with_rule_blocks() {
    assert_parses!(
        Rule::trait_def,
        r#"trait Capacitor: TwoTerminal {
            spec {
                capacitance: Farads
                voltage_rating: Voltage
            }
            rule voltage_derating(level: Warning) {
                assert net_voltage(self.A, self.B) <= self.spec.voltage_rating * 0.8
                message: "Capacitor voltage exceeds 80% derating: {net_voltage(self.A, self.B)}V > {self.spec.voltage_rating * 0.8}V"
            }
            rule voltage_exceed(level: Error) {
                assert net_voltage(self.A, self.B) <= self.spec.voltage_rating
                message: "Capacitor voltage {net_voltage(self.A, self.B)}V exceeds voltage_rating {self.spec.voltage_rating}V"
            }
        }"#
    );
}

#[test]
fn trait_connector() {
    assert_parses!(
        Rule::trait_def,
        r#"trait Connector {
            designator_prefix: "J"
        }"#
    );
}

// ═══════════════════════════════════════════════
// Device definitions
// ═══════════════════════════════════════════════

#[test]
fn device_simple_passive() {
    assert_parses!(
        Rule::device_def,
        "device MLCC<C: Farads, V: Voltage = 10V, pkg: Package = C0402>: impl Capacitor {
            package: pkg
            pins { A: 1, B: 2 }
            spec {
                capacitance: C
                voltage_rating: V
            }
        }"
    );
}

#[test]
fn device_multi_trait() {
    assert_parses!(
        Rule::device_def,
        "device Electrolytic<C: Farads, V: Voltage>: impl Capacitor + Polarized {
            package: RADIAL_D5
            pins { A: 1, B: 2 }
            spec {
                capacitance: C
                voltage_rating: V
            }
        }"
    );
}

#[test]
fn device_with_rule_override() {
    assert_parses!(
        Rule::device_def,
        r#"device Electrolytic<C: Farads, V: Voltage>: impl Capacitor + Polarized {
            package: RADIAL_D5
            pins { A: 1, B: 2 }
            spec { capacitance: C, voltage_rating: V }
            rule voltage_derating(level: Warning) {
                assert net_voltage(self.A, self.B) <= self.spec.voltage_rating * 0.6
                message: "Electrolytic capacitor voltage exceeds 60% derating: {net_voltage(self.A, self.B)}V > {self.spec.voltage_rating * 0.6}V"
            }
        }"#
    );
}

#[test]
fn device_multiple_pins_blocks() {
    assert_parses!(
        Rule::device_def,
        "device STM32F103<pkg: Package = LQFP48> {
            package: pkg
            pins[LQFP48] {
                VDD_CORE: 1
                VDD_IO: 24
                GND: [8, 23, 35, 47]
                pin_bus!(PA, 10, 8)
                pin_bus!(PB, 18, 8)
                USB_DM: 20
                USB_DP: 21
            }
            pins[LQFP64] {
                VDD_CORE: 1
                VDD_IO: 24
                GND: [8, 23, 35, 47]
                pin_bus!(PA, 10, 16)
                pin_bus!(PB, 26, 16)
                USB_DM: 34
                USB_DP: 35
            }
        }"
    );
}

#[test]
fn device_no_generics_no_impl() {
    assert_parses!(
        Rule::device_def,
        "device SimplePart {
            package: QFN16
            pins {
                VDD: 1
                GND: 2
            }
        }"
    );
}

#[test]
fn device_resistor() {
    assert_parses!(
        Rule::device_def,
        "device Resistor<R: Ohms, pkg: Package = R0402>: impl Resistor {
            package: pkg
            pins { A: 1, B: 2 }
            spec { resistance: R }
        }"
    );
}

// ═══════════════════════════════════════════════
// Part definitions
// ═══════════════════════════════════════════════

#[test]
fn part_with_avl() {
    assert_parses!(
        Rule::part_def,
        r#"part mlcc_100nF_0402: MLCC<C: 100nF, V: 10V, pkg: C0402> {
            primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }
            alt { mfr: "Murata", mpn: "GRM155R61C104KA88D" }
            alt { mfr: "Yageo", mpn: "CC0402KRX5R9BB104" }
        }"#
    );
}

#[test]
fn part_resistor() {
    assert_parses!(
        Rule::part_def,
        r#"part res_10k_0402: Resistor<R: 10k, pkg: R0402> {
            primary { mfr: "Yageo", mpn: "RC0402FR-0710KL" }
            alt { mfr: "ROHM", mpn: "MCR01MZPF1002" }
            alt { mfr: "Walsin", mpn: "WR04X1002FTL" }
        }"#
    );
}

// ═══════════════════════════════════════════════
// Type aliases
// ═══════════════════════════════════════════════

#[test]
fn type_alias_no_params() {
    assert_parses!(
        Rule::type_def,
        "type BypassCap = MLCC<C: 100nF, V: 10V, pkg: C0402>"
    );
    assert_parses!(
        Rule::type_def,
        "type BulkCap = Tantalum<C: 10uF, V: 6.3V>"
    );
    assert_parses!(
        Rule::type_def,
        "type PullupRes = Resistor<R: 10k, pkg: R0402>"
    );
}

#[test]
fn type_alias_with_params() {
    assert_parses!(
        Rule::type_def,
        "type SmallCap<C: Farads> = MLCC<C: C, V: 10V, pkg: C0402>"
    );
    assert_parses!(
        Rule::type_def,
        "type PowerRes<R: Ohms> = Resistor<R: R, pkg: R2512>"
    );
    assert_parses!(
        Rule::type_def,
        "type SeriesRes<R: Ohms, P: Package = R0402> = Resistor<R: R, pkg: P>"
    );
}

// ═══════════════════════════════════════════════
// Function definitions
// ═══════════════════════════════════════════════

#[test]
fn fn_simple_decoupling() {
    assert_parses!(
        Rule::fn_def,
        "fn decoupling<P: impl Capacitor>(
            vdd: Net,
            gnd: Net,
            cap: P
        ) {
            inst c: cap
            net vdd: c.A
            net gnd: c.B
        }"
    );
}

#[test]
fn fn_value_generics_with_defaults() {
    assert_parses!(
        Rule::fn_def,
        "fn decoupling_val<C: Capacitance = 100nF, pkg: Package = C0402>(
            vdd: Net,
            gnd: Net
        ) {
            inst c: MLCC<C: C, pkg: pkg>
            net vdd: c.A
            net gnd: c.B
        }"
    );
}

#[test]
fn fn_multi_trait_bound() {
    assert_parses!(
        Rule::fn_def,
        "fn bulk_cap<P: impl Capacitor + Polarized>(
            vdd: Net,
            gnd: Net,
            cap: P
        ) {
            inst c: cap
            net vdd: c.A
            net gnd: c.B
        }"
    );
}

#[test]
fn fn_voltage_divider() {
    assert_parses!(
        Rule::fn_def,
        "fn voltage_divider<R1: Ohms, R2: Ohms>(
            vin: Net,
            vout: Net,
            gnd: Net
        ) {
            inst r_top: Resistor<R: R1>
            inst r_bot: Resistor<R: R2>
            net vin: r_top.A
            net vout: r_top.B, r_bot.A
            net gnd: r_bot.B
        }"
    );
}

#[test]
fn fn_no_generics() {
    assert_parses!(
        Rule::fn_def,
        "fn usb_termination(dm: Net, dp: Net, gnd: Net) {
            inst r_dm: SeriesRes<R: 22R>
            inst r_dp: SeriesRes<R: 22R>
            net dm: r_dm.A
            net dp: r_dp.A
            net gnd: r_dm.B, r_dp.B
        }"
    );
}

// ═══════════════════════════════════════════════
// Module definitions
// ═══════════════════════════════════════════════

#[test]
fn module_with_items() {
    assert_parses!(
        Rule::module_def,
        "module passives {
            pub type BypassCap = MLCC<C: 100nF, V: 10V, pkg: C0402>
            pub type PullupRes = Resistor<R: 10k, pkg: R0402>
            pub type SmallCap<C: Farads> = MLCC<C: C, V: 10V, pkg: C0402>
        }"
    );
}

#[test]
fn module_with_parts() {
    assert_parses!(
        Rule::module_def,
        r#"module passives {
            pub part res_10k_0402: Resistor<R: 10k, pkg: R0402> {
                primary { mfr: "Yageo", mpn: "RC0402FR-0710KL" }
            }
            pub part mlcc_100nF_0402: MLCC<C: 100nF, pkg: C0402> {
                primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }
            }
        }"#
    );
}

#[test]
fn module_with_device() {
    assert_parses!(
        Rule::module_def,
        "module stm32 {
            pub device STM32F103<pkg: Package = LQFP48> {
                package: pkg
                pins {
                    VDD: 1
                    GND: 2
                }
            }
        }"
    );
}

// ═══════════════════════════════════════════════
// Design definitions
// ═══════════════════════════════════════════════

#[test]
fn design_basic() {
    assert_parses!(
        Rule::design_def,
        "design MainBoard {
            inst mcu: STM32F103<pkg: LQFP64>
            inst c_bypass: BypassCap
        }"
    );
}

#[test]
fn design_with_calls() {
    assert_parses!(
        Rule::design_def,
        "design MainBoard {
            inst mcu: STM32F103<pkg: LQFP64>
            decoupling(vdd: mcu.VDD_IO, gnd: GND, cap: mlcc_100nF_0402)
        }"
    );
}

#[test]
fn design_with_scoped_call() {
    assert_parses!(
        Rule::design_def,
        "design MainBoard {
            inst mcu: STM32F103<pkg: LQFP64>
            power::ldo(vin: VIN, vout: VCC_3V3, gnd: GND)
        }"
    );
}

#[test]
fn design_with_attributes() {
    assert_parses!(
        Rule::design_def,
        r#"design MainBoard {
            #[designator("U1")]
            inst mcu: STM32F103<pkg: LQFP64>
            #[allow(unconnected_pin)]
            inst c_usb: mlcc_100nF_0402
            #[designator("C1")]
            inst c_bulk: BulkCap
        }"#
    );
}

// ═══════════════════════════════════════════════
// Use declarations
// ═══════════════════════════════════════════════

#[test]
fn use_single_import() {
    assert_parses!(Rule::use_decl, "use power::decoupling");
    assert_parses!(Rule::use_decl, "use stm32::STM32F103");
}

#[test]
fn use_group_import() {
    assert_parses!(
        Rule::use_decl,
        "use passives::{res_10k_0402, mlcc_100nF_0402}"
    );
    assert_parses!(Rule::use_decl, "use passives::{BypassCap, SmallCap}");
}

// ═══════════════════════════════════════════════
// Mod declarations
// ═══════════════════════════════════════════════

#[test]
fn mod_decl_basic() {
    assert_parses!(Rule::mod_decl, "mod common");
}

// ═══════════════════════════════════════════════
// Statements
// ═══════════════════════════════════════════════

#[test]
fn inst_stmt_simple() {
    assert_parses!(Rule::inst_stmt, "inst c: cap");
    assert_parses!(Rule::inst_stmt, "inst c_bypass: BypassCap");
    assert_parses!(Rule::inst_stmt, "inst r_pull: PullupRes");
    assert_parses!(Rule::inst_stmt, "inst c_filter: SmallCap<C: 4.7nF>");
    assert_parses!(Rule::inst_stmt, "inst mcu: STM32F103<pkg: LQFP64>");
}

#[test]
fn inst_stmt_inline_part() {
    assert_parses!(
        Rule::inst_stmt,
        r#"inst r_sense: Resistor<R: 100m, pkg: R2512> {
            primary { mfr: "Vishay", mpn: "WSL2512R1000FEA" }
            alt { mfr: "Bourns", mpn: "CSS2512FT0R100" }
        }"#
    );
}

#[test]
fn net_stmt_single() {
    assert_parses!(Rule::net_stmt, "net vdd: c.A");
    assert_parses!(Rule::net_stmt, "net gnd: c.B");
}

#[test]
fn net_stmt_multiple_endpoints() {
    assert_parses!(Rule::net_stmt, "net vout: r_top.B, r_bot.A");
    assert_parses!(Rule::net_stmt, "net gnd: r_dm.B, r_dp.B");
}

#[test]
fn net_stmt_ident_target_and_endpoints() {
    assert_parses!(Rule::net_stmt, "net GND: c.B");
}

#[test]
fn call_stmt_basic() {
    assert_parses!(
        Rule::call_stmt,
        "decoupling(vdd: mcu.VDD_IO, gnd: GND, cap: mlcc_100nF_0402)"
    );
}

#[test]
fn call_stmt_scoped() {
    assert_parses!(
        Rule::call_stmt,
        "power::ldo(vin: VIN, vout: VCC_3V3, gnd: GND)"
    );
}

#[test]
fn call_stmt_with_generic_overrides() {
    assert_parses!(
        Rule::call_stmt,
        "voltage_divider(vin: V_REF, vout: ADC_IN, gnd: GND, R1: 10k, R2: 4.7k)"
    );
}

#[test]
fn call_stmt_with_type_arg() {
    assert_parses!(
        Rule::call_stmt,
        "decoupling(vdd: mcu.VDD_ANA, gnd: GND, cap: Tantalum<C: 10uF, V: 6.3V>)"
    );
}

// ═══════════════════════════════════════════════
// Expression parsing (rule bodies)
// ═══════════════════════════════════════════════

#[test]
fn expr_comparison() {
    assert_parses!(
        Rule::expr,
        "net_voltage(self.A, self.B) <= self.spec.voltage_rating * 0.8"
    );
}

#[test]
fn expr_multiplication() {
    assert_parses!(Rule::expr, "self.spec.voltage_rating * 0.6");
}

#[test]
fn expr_fn_call() {
    assert_parses!(Rule::expr, "net_voltage(self.A, self.B)");
}

#[test]
fn expr_dot_path() {
    assert_parses!(Rule::expr, "self.spec.voltage_rating");
    assert_parses!(Rule::expr, "self.A");
    assert_parses!(Rule::expr, "mcu.VDD_IO");
}

// ═══════════════════════════════════════════════
// Rule blocks
// ═══════════════════════════════════════════════

#[test]
fn rule_block_warning() {
    assert_parses!(
        Rule::rule_block,
        r#"rule voltage_derating(level: Warning) {
            assert net_voltage(self.A, self.B) <= self.spec.voltage_rating * 0.8
            message: "Capacitor voltage exceeds 80% derating: {net_voltage(self.A, self.B)}V > {self.spec.voltage_rating * 0.8}V"
        }"#
    );
}

#[test]
fn rule_block_error() {
    assert_parses!(
        Rule::rule_block,
        r#"rule voltage_exceed(level: Error) {
            assert net_voltage(self.A, self.B) <= self.spec.voltage_rating
            message: "Capacitor voltage {net_voltage(self.A, self.B)}V exceeds voltage_rating {self.spec.voltage_rating}V"
        }"#
    );
}

// ═══════════════════════════════════════════════
// Interpolated strings
// ═══════════════════════════════════════════════

#[test]
fn interpolated_string_basic() {
    assert_parses!(
        Rule::interpolated_string,
        r#""Capacitor voltage {v}V exceeds {max}V""#
    );
}

#[test]
fn interpolated_string_with_expr() {
    assert_parses!(
        Rule::interpolated_string,
        r#""value: {self.spec.voltage_rating * 0.8}V""#
    );
}

// ═══════════════════════════════════════════════
// Pins block
// ═══════════════════════════════════════════════

#[test]
fn pins_block_unqualified() {
    assert_parses!(
        Rule::pins_block,
        "pins {
            A: Pin
            B: Pin
        }"
    );
}

#[test]
fn pins_block_with_values() {
    assert_parses!(Rule::pins_block, "pins { A: 1, B: 2 }");
}

#[test]
fn pins_block_qualified() {
    assert_parses!(
        Rule::pins_block,
        "pins[LQFP48] {
            VDD_CORE: 1
            GND: [8, 23, 35, 47]
            pin_bus!(PA, 10, 8)
            USB_DM: 20
        }"
    );
}

// ═══════════════════════════════════════════════
// Spec block
// ═══════════════════════════════════════════════

#[test]
fn spec_block_types() {
    assert_parses!(
        Rule::spec_block,
        "spec {
            capacitance: Farads
            voltage_rating: Voltage
        }"
    );
}

#[test]
fn spec_block_values() {
    assert_parses!(
        Rule::spec_block,
        "spec {
            capacitance: C
            voltage_rating: V
        }"
    );
}

#[test]
fn spec_block_comma_separated() {
    assert_parses!(Rule::spec_block, "spec { capacitance: C, voltage_rating: V }");
}

#[test]
fn spec_block_boolean() {
    assert_parses!(Rule::spec_block, "spec { polarity: true }");
}

// ═══════════════════════════════════════════════
// AVL entries
// ═══════════════════════════════════════════════

#[test]
fn avl_entry_primary() {
    assert_parses!(
        Rule::avl_entry,
        r#"primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }"#
    );
}

#[test]
fn avl_entry_alt() {
    assert_parses!(
        Rule::avl_entry,
        r#"alt { mfr: "Murata", mpn: "GRM155R61C104KA88D" }"#
    );
}

// ═══════════════════════════════════════════════
// Package declaration
// ═══════════════════════════════════════════════

#[test]
fn package_decl_ident() {
    assert_parses!(Rule::package_decl, "package: pkg");
    assert_parses!(Rule::package_decl, "package: RADIAL_D5");
    assert_parses!(Rule::package_decl, "package: QFN16");
}

// ═══════════════════════════════════════════════
// Designator prefix
// ═══════════════════════════════════════════════

#[test]
fn designator_prefix_test() {
    assert_parses!(Rule::designator_prefix, r#"designator_prefix: "C""#);
    assert_parses!(Rule::designator_prefix, r#"designator_prefix: "R""#);
    assert_parses!(Rule::designator_prefix, r#"designator_prefix: "U""#);
    assert_parses!(Rule::designator_prefix, r#"designator_prefix: "J""#);
    assert_parses!(Rule::designator_prefix, r#"designator_prefix: "L""#);
}

// ═══════════════════════════════════════════════
// Impl clause
// ═══════════════════════════════════════════════

#[test]
fn impl_clause_single() {
    assert_parses!(Rule::impl_clause, ": impl Capacitor");
}

#[test]
fn impl_clause_multi() {
    assert_parses!(Rule::impl_clause, ": impl Capacitor + Polarized");
}

// ═══════════════════════════════════════════════
// Full file parsing
// ═══════════════════════════════════════════════

#[test]
fn file_empty() {
    assert_parses!(Rule::file, "");
}

#[test]
fn file_with_comments() {
    assert_parses!(
        Rule::file,
        "// This is a comment
        // Another comment
        trait TwoTerminal {
            pins {
                A: Pin
                B: Pin
            }
        }"
    );
}

#[test]
fn file_use_and_design() {
    assert_parses!(
        Rule::file,
        "use power::decoupling
        use passives::{res_10k_0402, mlcc_100nF_0402}
        use stm32::STM32F103

        design MainBoard {
            inst mcu: STM32F103<pkg: LQFP64>
            decoupling(vdd: mcu.VDD_IO, gnd: GND, cap: mlcc_100nF_0402)
            decoupling(vdd: mcu.VDD_ANA, gnd: GND, cap: mlcc_100nF_0402)
            power::ldo(vin: VIN, vout: VCC_3V3, gnd: GND)
        }"
    );
}

#[test]
fn file_mod_declarations() {
    assert_parses!(
        Rule::file,
        "pub mod stm32f1
        pub mod stm32f4
        mod common"
    );
}

#[test]
fn file_trait_device_part() {
    assert_parses!(
        Rule::file,
        r#"trait TwoTerminal {
            pins {
                A: Pin
                B: Pin
            }
        }

        trait Capacitor: TwoTerminal {
            designator_prefix: "C"
            spec {
                capacitance: Farads
                voltage_rating: Voltage
            }
        }

        device MLCC<C: Farads, V: Voltage = 10V, pkg: Package = C0402>: impl Capacitor {
            package: pkg
            pins { A: 1, B: 2 }
            spec {
                capacitance: C
                voltage_rating: V
            }
        }

        type BypassCap = MLCC<C: 100nF, V: 10V, pkg: C0402>

        part mlcc_100nF_0402: MLCC<C: 100nF, V: 10V, pkg: C0402> {
            primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }
            alt { mfr: "Murata", mpn: "GRM155R61C104KA88D" }
        }

        fn decoupling<P: impl Capacitor>(
            vdd: Net,
            gnd: Net,
            cap: P
        ) {
            inst c: cap
            net vdd: c.A
            net gnd: c.B
        }

        design MainBoard {
            inst mcu: STM32F103<pkg: LQFP64>
            #[designator("C1")]
            inst c_bulk: BypassCap
            decoupling(vdd: mcu.VDD_IO, gnd: GND, cap: mlcc_100nF_0402)
        }"#
    );
}

#[test]
fn file_module_with_nested_items() {
    assert_parses!(
        Rule::file,
        r#"module power {
            pub fn decoupling<P: impl Capacitor>(vdd: Net, gnd: Net, cap: P) {
                inst c: cap
                net vdd: c.A
                net gnd: c.B
            }
        }

        module passives {
            pub type BypassCap = MLCC<C: 100nF, V: 10V, pkg: C0402>
            pub part mlcc_100nF_0402: MLCC<C: 100nF, pkg: C0402> {
                primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }
            }
        }"#
    );
}

#[test]
fn file_design_with_all_stmt_types() {
    assert_parses!(
        Rule::file,
        r#"design MainBoard {
            #[designator("U1")]
            inst mcu: STM32F103<pkg: LQFP64>
            #[allow(unconnected_pin)]
            inst c_bypass: BypassCap
            inst r_sense: Resistor<R: 100m, pkg: R2512> {
                primary { mfr: "Vishay", mpn: "WSL2512R1000FEA" }
            }
            net VDD: mcu.VDD_IO
            net GND: mcu.GND, c_bypass.B
            decoupling(vdd: mcu.VDD_IO, gnd: GND, cap: mlcc_100nF_0402)
            power::ldo(vin: VIN, vout: VCC_3V3, gnd: GND)
            voltage_divider(vin: V_REF, vout: ADC_IN, gnd: GND, R1: 10k, R2: 4.7k)
        }"#
    );
}

// ═══════════════════════════════════════════════
// parses_to! tests for structural verification
// ═══════════════════════════════════════════════

#[test]
fn parses_to_ident() {
    parses_to! {
        parser: CohdlParser,
        input: "foo",
        rule: Rule::ident,
        tokens: [
            ident(0, 3)
        ]
    };
}

#[test]
fn parses_to_eng_number_with_suffix() {
    parses_to! {
        parser: CohdlParser,
        input: "100nF",
        rule: Rule::eng_number,
        tokens: [
            eng_number(0, 5)
        ]
    };
}

#[test]
fn parses_to_eng_number_plain() {
    parses_to! {
        parser: CohdlParser,
        input: "48",
        rule: Rule::eng_number,
        tokens: [
            eng_number(0, 2)
        ]
    };
}

#[test]
fn parses_to_string_literal() {
    // string_literal is atomic, so string_inner is NOT a child token
    parses_to! {
        parser: CohdlParser,
        input: r#""hello""#,
        rule: Rule::string_literal,
        tokens: [
            string_literal(0, 7)
        ]
    };
}

#[test]
fn parses_to_boolean_true() {
    parses_to! {
        parser: CohdlParser,
        input: "true",
        rule: Rule::boolean,
        tokens: [
            boolean(0, 4)
        ]
    };
}

#[test]
fn parses_to_pin_list() {
    parses_to! {
        parser: CohdlParser,
        input: "[1, 2, 3]",
        rule: Rule::pin_list,
        tokens: [
            pin_list(0, 9, [
                integer(1, 2),
                integer(4, 5),
                integer(7, 8)
            ])
        ]
    };
}

#[test]
fn parses_to_pin_range() {
    parses_to! {
        parser: CohdlParser,
        input: "[10..17]",
        rule: Rule::pin_range,
        tokens: [
            pin_range(0, 8, [
                integer(1, 3),
                integer(5, 7)
            ])
        ]
    };
}

#[test]
fn parses_to_pin_bus_call() {
    parses_to! {
        parser: CohdlParser,
        input: "pin_bus!(PA, 10, 8)",
        rule: Rule::pin_bus_call,
        tokens: [
            pin_bus_call(0, 19, [
                ident(9, 11),
                integer(13, 15),
                integer(17, 18)
            ])
        ]
    };
}

#[test]
fn parses_to_scoped_path() {
    parses_to! {
        parser: CohdlParser,
        input: "power::decoupling",
        rule: Rule::scoped_path,
        tokens: [
            scoped_path(0, 17, [
                ident(0, 5),
                ident(7, 17)
            ])
        ]
    };
}

#[test]
fn parses_to_generic_arg() {
    parses_to! {
        parser: CohdlParser,
        input: "C: 100nF",
        rule: Rule::generic_arg,
        tokens: [
            generic_arg(0, 8, [
                ident(0, 1),
                value_expr(3, 8, [
                    expr(3, 8, [
                        comparison_expr(3, 8, [
                            additive_expr(3, 8, [
                                multiplicative_expr(3, 8, [
                                    unary_expr(3, 8, [
                                        atom_expr(3, 8, [
                                            eng_number(3, 8)
                                        ])
                                    ])
                                ])
                            ])
                        ])
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parses_to_use_group() {
    parses_to! {
        parser: CohdlParser,
        input: "{BypassCap, SmallCap}",
        rule: Rule::use_group,
        tokens: [
            use_group(0, 21, [
                ident(1, 10),
                ident(12, 20)
            ])
        ]
    };
}

#[test]
fn parses_to_use_decl_scoped() {
    parses_to! {
        parser: CohdlParser,
        input: "use power::decoupling",
        rule: Rule::use_decl,
        tokens: [
            use_decl(0, 21, [
                use_path(4, 21, [
                    ident(4, 9),
                    ident(11, 21)
                ])
            ])
        ]
    };
}

#[test]
fn parses_to_use_decl_group() {
    parses_to! {
        parser: CohdlParser,
        input: "use passives::{BypassCap, SmallCap}",
        rule: Rule::use_decl,
        tokens: [
            use_decl(0, 35, [
                use_path(4, 35, [
                    ident(4, 12),
                    use_group(14, 35, [
                        ident(15, 24),
                        ident(26, 34)
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parses_to_attribute_allow() {
    parses_to! {
        parser: CohdlParser,
        input: "#[allow(unconnected_pin)]",
        rule: Rule::attribute,
        tokens: [
            attribute(0, 25, [
                attribute_inner(2, 24, [
                    ident(2, 7),
                    attribute_args(8, 23, [
                        ident(8, 23)
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parses_to_attribute_designator() {
    // string_literal is atomic so no children
    parses_to! {
        parser: CohdlParser,
        input: r#"#[designator("U1")]"#,
        rule: Rule::attribute,
        tokens: [
            attribute(0, 19, [
                attribute_inner(2, 18, [
                    ident(2, 12),
                    attribute_args(13, 17, [
                        string_literal(13, 17)
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parses_to_mod_decl() {
    parses_to! {
        parser: CohdlParser,
        input: "mod common",
        rule: Rule::mod_decl,
        tokens: [
            mod_decl(0, 10, [
                ident(4, 10)
            ])
        ]
    };
}

#[test]
fn parses_to_avl_entry() {
    // string_literal is atomic
    parses_to! {
        parser: CohdlParser,
        input: r#"primary { mfr: "Samsung", mpn: "CL05" }"#,
        rule: Rule::avl_entry,
        tokens: [
            avl_entry(0, 39, [
                avl_kind(0, 7),
                avl_field(10, 24, [
                    ident(10, 13),
                    string_literal(15, 24)
                ]),
                avl_field(26, 37, [
                    ident(26, 29),
                    string_literal(31, 37)
                ])
            ])
        ]
    };
}

#[test]
fn parses_to_designator_prefix() {
    parses_to! {
        parser: CohdlParser,
        input: r#"designator_prefix: "C""#,
        rule: Rule::designator_prefix,
        tokens: [
            designator_prefix(0, 22, [
                string_literal(19, 22)
            ])
        ]
    };
}

#[test]
fn parses_to_package_decl() {
    parses_to! {
        parser: CohdlParser,
        input: "package: pkg",
        rule: Rule::package_decl,
        tokens: [
            package_decl(0, 12, [
                ident(9, 12)
            ])
        ]
    };
}

#[test]
fn parses_to_visibility() {
    parses_to! {
        parser: CohdlParser,
        input: "pub",
        rule: Rule::visibility,
        tokens: [
            visibility(0, 3)
        ]
    };
}

#[test]
fn parses_to_impl_clause() {
    parses_to! {
        parser: CohdlParser,
        input: ": impl Capacitor",
        rule: Rule::impl_clause,
        tokens: [
            impl_clause(0, 16, [
                trait_bound(7, 16, [
                    type_expr(7, 16, [
                        ident(7, 16)
                    ])
                ])
            ])
        ]
    };
}

#[test]
fn parses_to_impl_clause_multi() {
    parses_to! {
        parser: CohdlParser,
        input: ": impl Capacitor + Polarized",
        rule: Rule::impl_clause,
        tokens: [
            impl_clause(0, 28, [
                trait_bound(7, 28, [
                    type_expr(7, 17, [
                        ident(7, 16)
                    ]),
                    type_expr(19, 28, [
                        ident(19, 28)
                    ])
                ])
            ])
        ]
    };
}
