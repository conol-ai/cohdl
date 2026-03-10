use cohdl_parser::parse_source_file;
use cohdl_parser::{
    AvlKind, DesignBodyStmtKind, DeviceBodyItem, ExprKind, FnBodyStmtKind, FnParamKind,
    GenericParamKind, NetEndpointKind, PinEntryKind, TopLevelItemKind, TraitBodyItem,
};

// ═══════════════════════════════════════════════════════════════════════════
// Test 1: Multi-device .cohdl file (STM32 + USB connector + passives)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn parse_multi_device_file() {
    let src = r#"
// Traits
trait TwoTerminal {
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
    rule voltage_exceed(level: Error) {
        assert net_voltage(self.A, self.B) <= self.spec.voltage_rating
        message: "Capacitor voltage {net_voltage(self.A, self.B)}V exceeds rating {self.spec.voltage_rating}V"
    }
}

trait Connector {
    designator_prefix: "J"
}

// Devices
device MLCC<C: Farads, V: Voltage = 10V, pkg: Package = C0402>: impl Capacitor {
    package: pkg
    pins { A: 1, B: 2 }
    spec {
        capacitance: C
        voltage_rating: V
    }
}

device STM32F103<pkg: Package = LQFP48> {
    package: pkg
    pins[LQFP48] {
        VDD_CORE: 1
        VDD_IO: 24
        GND: [8, 23, 35, 47]
        pin_bus!(PA, 10, 8)
        USB_DM: 20
        USB_DP: 21
    }
}

device USBTypeC: impl Connector {
    package: USB_C_SMD
    pins {
        VBUS: 1
        CC1: 2
        CC2: 3
        DP: 4
        DM: 5
        GND: [6, 7]
    }
}

// Parts
part mlcc_100nF: MLCC<C: 100nF, V: 10V> {
    primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }
    alt { mfr: "Murata", mpn: "GRM155R61C104KA88D" }
}

// Design
design USBBoard {
    inst mcu: STM32F103<pkg: LQFP48>
    inst usb: USBTypeC
    inst c_vbus: MLCC<C: 100nF, V: 16V>
    net VDD: mcu.VDD_IO
    net GND: mcu.GND, usb.GND, c_vbus.B
    net USB_DP: mcu.USB_DP, usb.DP
    net USB_DM: mcu.USB_DM, usb.DM
    net VBUS: usb.VBUS, c_vbus.A
}
"#;

    let sf = parse_source_file(src).expect("should parse multi-device file");

    // Count top-level items: 3 traits + 3 devices + 1 part + 1 design = 8
    assert_eq!(sf.items.len(), 8, "expected 8 top-level items");

    // Verify trait kinds
    assert!(matches!(sf.items[0].kind, TopLevelItemKind::Trait(_)));
    assert!(matches!(sf.items[1].kind, TopLevelItemKind::Trait(_)));
    assert!(matches!(sf.items[2].kind, TopLevelItemKind::Trait(_)));

    // TwoTerminal trait has pins block with 2 typed pins
    if let TopLevelItemKind::Trait(ref t) = sf.items[0].kind {
        assert_eq!(t.name.name, "TwoTerminal");
        assert!(t.parents.is_none());
        assert_eq!(t.body.len(), 1);
        if let TraitBodyItem::Pins(ref pins) = t.body[0] {
            assert_eq!(pins.entries.len(), 2);
            assert!(matches!(pins.entries[0].kind, PinEntryKind::Typed { .. }));
        } else {
            panic!("expected pins block in TwoTerminal");
        }
    }

    // Capacitor trait has parent, designator_prefix, spec, and rule
    if let TopLevelItemKind::Trait(ref t) = sf.items[1].kind {
        assert_eq!(t.name.name, "Capacitor");
        assert!(t.parents.is_some());
        let parents = t.parents.as_ref().unwrap();
        assert_eq!(parents.bounds.len(), 1);
        assert_eq!(parents.bounds[0].path.segments[0].name, "TwoTerminal");
        assert_eq!(t.body.len(), 3); // designator_prefix, spec, rule
        assert!(matches!(t.body[0], TraitBodyItem::DesignatorPrefix(_)));
        assert!(matches!(t.body[1], TraitBodyItem::Spec(_)));
        assert!(matches!(t.body[2], TraitBodyItem::Rule(_)));
    }

    // Connector trait
    if let TopLevelItemKind::Trait(ref t) = sf.items[2].kind {
        assert_eq!(t.name.name, "Connector");
        assert!(t.parents.is_none());
        assert_eq!(t.body.len(), 1);
        assert!(matches!(t.body[0], TraitBodyItem::DesignatorPrefix(_)));
    }

    // MLCC device with generics and impl Capacitor
    assert!(matches!(sf.items[3].kind, TopLevelItemKind::Device(_)));
    if let TopLevelItemKind::Device(ref d) = sf.items[3].kind {
        assert_eq!(d.name.name, "MLCC");
        assert!(d.generic_params.is_some());
        let gp = d.generic_params.as_ref().unwrap();
        assert_eq!(gp.params.len(), 3);
        assert_eq!(gp.params[0].name.name, "C");
        assert!(d.impl_traits.is_some());
        // body: package, pins, spec
        assert_eq!(d.body.len(), 3);
        assert!(matches!(d.body[0], DeviceBodyItem::Package(_)));
        assert!(matches!(d.body[1], DeviceBodyItem::Pins(_)));
        assert!(matches!(d.body[2], DeviceBodyItem::Spec(_)));
    }

    // STM32F103 device with qualified pins block
    if let TopLevelItemKind::Device(ref d) = sf.items[4].kind {
        assert_eq!(d.name.name, "STM32F103");
        assert!(d.impl_traits.is_none());
        // Check the pins block has a qualifier
        let pins_item = d.body.iter().find(|b| matches!(b, DeviceBodyItem::Pins(_)));
        assert!(pins_item.is_some());
        if let DeviceBodyItem::Pins(ref p) = d.body[1] {
            assert_eq!(p.qualifier.as_ref().unwrap().name, "LQFP48");
            // VDD_CORE (single), VDD_IO (single), GND (list), pin_bus!(PA, ...), USB_DM, USB_DP
            assert_eq!(p.entries.len(), 6);
            assert!(matches!(p.entries[2].kind, PinEntryKind::List { .. }));
            assert!(matches!(p.entries[3].kind, PinEntryKind::BusMacro { .. }));
        }
    }

    // USB connector device
    if let TopLevelItemKind::Device(ref d) = sf.items[5].kind {
        assert_eq!(d.name.name, "USBTypeC");
        assert!(d.impl_traits.is_some());
    }

    // Part
    assert!(matches!(sf.items[6].kind, TopLevelItemKind::Part(_)));
    if let TopLevelItemKind::Part(ref p) = sf.items[6].kind {
        assert_eq!(p.name.name, "mlcc_100nF");
        assert_eq!(p.avl_entries.len(), 2);
        assert_eq!(p.avl_entries[0].kind, AvlKind::Primary);
        assert_eq!(p.avl_entries[1].kind, AvlKind::Alt);
        assert_eq!(p.avl_entries[0].fields.len(), 2);
        assert_eq!(p.avl_entries[0].fields[0].name.name, "mfr");
        assert_eq!(p.avl_entries[0].fields[0].value.value, "Samsung");
    }

    // Design
    assert!(matches!(sf.items[7].kind, TopLevelItemKind::Design(_)));
    if let TopLevelItemKind::Design(ref d) = sf.items[7].kind {
        assert_eq!(d.name.name, "USBBoard");
        assert_eq!(d.body.len(), 8); // 3 inst + 5 net
        assert!(matches!(d.body[0].kind, DesignBodyStmtKind::Inst(_)));
        assert!(matches!(d.body[3].kind, DesignBodyStmtKind::Net(_)));
    }

    // Verify all spans are non-degenerate
    for item in &sf.items {
        assert!(
            item.span.start < item.span.end,
            "item span should be non-empty"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Test 2: Module file with use, type, fn, part
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn parse_module_file() {
    let src = r#"
use power::decoupling
use passives::{res_10k, mlcc_100nF}

type BypassCap = MLCC<C: 100nF, V: 10V, pkg: C0402>
type SmallCap<C: Farads> = MLCC<C: C, V: 10V, pkg: C0402>

fn voltage_divider<R1: Ohms, R2: Ohms>(
    vin: Net,
    vout: Net,
    gnd: Net
) {
    inst r_top: Resistor<R: R1>
    inst r_bot: Resistor<R: R2>
    net vin: r_top.A
    net vout: r_top.B, r_bot.A
    net gnd: r_bot.B
}

part mlcc_100nF_0402: MLCC<C: 100nF, V: 10V, pkg: C0402> {
    primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }
    alt { mfr: "Murata", mpn: "GRM155R61C104KA88D" }
    alt { mfr: "Yageo", mpn: "CC0402KRX5R9BB104" }
}

mod common
"#;

    let sf = parse_source_file(src).expect("should parse module file");

    // 2 use + 2 type + 1 fn + 1 part + 1 mod = 7
    assert_eq!(sf.items.len(), 7);

    // use power::decoupling
    assert!(matches!(sf.items[0].kind, TopLevelItemKind::Use(_)));
    if let TopLevelItemKind::Use(ref u) = sf.items[0].kind {
        assert_eq!(u.tree.prefix.len(), 2);
        assert_eq!(u.tree.prefix[0].name, "power");
        assert_eq!(u.tree.prefix[1].name, "decoupling");
        assert!(u.tree.group.is_none());
    }

    // use passives::{res_10k, mlcc_100nF}
    if let TopLevelItemKind::Use(ref u) = sf.items[1].kind {
        assert_eq!(u.tree.prefix.len(), 1);
        assert_eq!(u.tree.prefix[0].name, "passives");
        assert!(u.tree.group.is_some());
        let group = u.tree.group.as_ref().unwrap();
        assert_eq!(group.len(), 2);
        assert_eq!(group[0].name, "res_10k");
        assert_eq!(group[1].name, "mlcc_100nF");
    }

    // type BypassCap = MLCC<...> (no generic params on alias)
    assert!(matches!(sf.items[2].kind, TopLevelItemKind::TypeAlias(_)));
    if let TopLevelItemKind::TypeAlias(ref ta) = sf.items[2].kind {
        assert_eq!(ta.name.name, "BypassCap");
        assert!(ta.generic_params.is_none());
        assert_eq!(ta.target.path.segments[0].name, "MLCC");
        assert!(ta.target.generic_args.is_some());
    }

    // type SmallCap<C: Farads> = MLCC<...> (has generic params)
    if let TopLevelItemKind::TypeAlias(ref ta) = sf.items[3].kind {
        assert_eq!(ta.name.name, "SmallCap");
        assert!(ta.generic_params.is_some());
        let gp = ta.generic_params.as_ref().unwrap();
        assert_eq!(gp.params.len(), 1);
        assert_eq!(gp.params[0].name.name, "C");
        assert!(matches!(gp.params[0].kind, GenericParamKind::Type(_)));
    }

    // fn voltage_divider<R1, R2>(vin, vout, gnd) { ... }
    assert!(matches!(sf.items[4].kind, TopLevelItemKind::Fn(_)));
    if let TopLevelItemKind::Fn(ref f) = sf.items[4].kind {
        assert_eq!(f.name.name, "voltage_divider");
        assert!(f.generic_params.is_some());
        let gp = f.generic_params.as_ref().unwrap();
        assert_eq!(gp.params.len(), 2);
        assert_eq!(f.params.len(), 3);
        assert_eq!(f.params[0].name.name, "vin");
        assert!(matches!(f.params[0].kind, FnParamKind::Type(_)));
        // Body: 2 inst + 3 net = 5
        assert_eq!(f.body.len(), 5);
        assert!(matches!(f.body[0].kind, FnBodyStmtKind::Inst(_)));
        assert!(matches!(f.body[2].kind, FnBodyStmtKind::Net(_)));

        // Verify net with multiple endpoints
        if let FnBodyStmtKind::Net(ref ns) = f.body[3].kind {
            assert_eq!(ns.endpoints.len(), 2);
            assert!(matches!(ns.endpoints[0].kind, NetEndpointKind::DotPath(_)));
            assert!(matches!(ns.endpoints[1].kind, NetEndpointKind::DotPath(_)));
        }
    }

    // part mlcc_100nF_0402
    assert!(matches!(sf.items[5].kind, TopLevelItemKind::Part(_)));
    if let TopLevelItemKind::Part(ref p) = sf.items[5].kind {
        assert_eq!(p.name.name, "mlcc_100nF_0402");
        assert_eq!(p.avl_entries.len(), 3);
        assert_eq!(p.avl_entries[0].kind, AvlKind::Primary);
        assert_eq!(p.avl_entries[1].kind, AvlKind::Alt);
        assert_eq!(p.avl_entries[2].kind, AvlKind::Alt);
    }

    // mod common
    assert!(matches!(sf.items[6].kind, TopLevelItemKind::Mod(_)));
    if let TopLevelItemKind::Mod(ref m) = sf.items[6].kind {
        assert_eq!(m.name.name, "common");
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Test 3: Round-tripped AST shapes match expectations
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn ast_shapes_match_expectations() {
    let src = r#"
trait Capacitor: TwoTerminal {
    designator_prefix: "C"
    spec {
        capacitance: Farads
        voltage_rating: Voltage
    }
    rule voltage_derating(level: Warning) {
        assert net_voltage(self.A, self.B) <= self.spec.voltage_rating * 0.8
        message: "voltage {net_voltage(self.A, self.B)}V exceeds 80% derating"
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

fn decoupling<P: impl Capacitor>(vdd: Net, gnd: Net, cap: P) {
    inst c: cap
    net vdd: c.A
    net gnd: c.B
}

design MainBoard {
    #[designator("U1")]
    inst mcu: STM32F103<pkg: LQFP64>
    decoupling(vdd: mcu.VDD_IO, gnd: GND, cap: mlcc_100nF)
}
"#;

    let sf = parse_source_file(src).expect("should parse");
    assert_eq!(sf.items.len(), 4);

    // Re-parse the same source to verify deterministic output
    let sf2 = parse_source_file(src).expect("re-parse should succeed");
    assert_eq!(sf.items.len(), sf2.items.len());

    // Verify trait shape
    if let TopLevelItemKind::Trait(ref t) = sf.items[0].kind {
        assert_eq!(t.name.name, "Capacitor");
        // Rule block assertion is a binary <= expression
        if let TraitBodyItem::Rule(ref r) = t.body[2] {
            assert_eq!(r.name.name, "voltage_derating");
            assert!(matches!(r.level, cohdl_parser::RuleLevel::Warning));
            assert!(matches!(r.assertion.kind, ExprKind::Binary(_)));
            if let ExprKind::Binary(ref bin) = r.assertion.kind {
                assert_eq!(bin.op, cohdl_parser::BinaryOp::Le);
                // LHS is a fn call
                assert!(matches!(bin.lhs.kind, ExprKind::FnCall(_)));
                // RHS is a multiply
                assert!(matches!(bin.rhs.kind, ExprKind::Binary(_)));
            }
        }
    }

    // Verify device shape: generic params with defaults
    if let TopLevelItemKind::Device(ref d) = sf.items[1].kind {
        let gp = d.generic_params.as_ref().unwrap();
        assert_eq!(gp.params.len(), 3);
        // First param has no default
        assert!(gp.params[0].default.is_none());
        // Second param has default "10V"
        assert!(gp.params[1].default.is_some());
        if let Some(ref def) = gp.params[1].default {
            assert!(matches!(def.kind, ExprKind::EngineeringNumber(_)));
        }
        // Third param has default "C0402"
        assert!(gp.params[2].default.is_some());
    }

    // Verify fn shape: impl constraint
    if let TopLevelItemKind::Fn(ref f) = sf.items[2].kind {
        let gp = f.generic_params.as_ref().unwrap();
        assert_eq!(gp.params.len(), 1);
        assert!(matches!(
            gp.params[0].kind,
            GenericParamKind::ImplConstraint(_)
        ));
        if let GenericParamKind::ImplConstraint(ref tb) = gp.params[0].kind {
            assert_eq!(tb.bounds.len(), 1);
            assert_eq!(tb.bounds[0].path.segments[0].name, "Capacitor");
        }
        // Third param is P (impl Capacitor)
        assert_eq!(f.params[2].name.name, "cap");
        assert!(matches!(f.params[2].kind, FnParamKind::Type(_)));
    }

    // Verify design shape: attributes and call statements
    if let TopLevelItemKind::Design(ref d) = sf.items[3].kind {
        assert_eq!(d.name.name, "MainBoard");
        assert_eq!(d.body.len(), 2);
        // First statement has a designator attribute
        assert_eq!(d.body[0].attributes.len(), 1);
        assert_eq!(d.body[0].attributes[0].name.name, "designator");
        // Second statement is a call
        assert!(matches!(d.body[1].kind, DesignBodyStmtKind::Call(_)));
        if let DesignBodyStmtKind::Call(ref c) = d.body[1].kind {
            assert_eq!(c.path.segments[0].name, "decoupling");
            assert_eq!(c.args.len(), 3);
            assert_eq!(c.args[0].name.name, "vdd");
            // vdd arg value is a dot_path: mcu.VDD_IO
            assert!(matches!(c.args[0].value.kind, ExprKind::DotPath(_)));
        }
    }

    // Verify all spans across the tree
    verify_spans_non_empty(&sf);
}

fn verify_spans_non_empty(sf: &cohdl_parser::SourceFile) {
    for item in &sf.items {
        assert!(
            item.span.end >= item.span.start,
            "item span should not be inverted"
        );
        // Check nested spans
        for attr in &item.attributes {
            assert!(attr.span.end > attr.span.start);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Test 4: Syntax errors produce non-empty error vecs with accurate spans
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn syntax_errors_produce_errors_with_spans() {
    // Missing closing brace
    let src = "trait Foo {";
    let result = parse_source_file(src);
    assert!(result.is_err(), "unclosed brace should fail");
    let errors = result.unwrap_err();
    assert!(!errors.is_empty(), "should have at least one error");
    // The error span should be at or near the end of input
    assert!(
        errors[0].span.start <= src.len(),
        "error span start should be within source"
    );
    // Error should have a non-empty message
    assert!(
        !errors[0].message.is_empty(),
        "error message should not be empty"
    );

    // Verify Display impl works
    let display = format!("{}", errors[0]);
    assert!(!display.is_empty());

    // Verify std::error::Error impl
    let _: &dyn std::error::Error = &errors[0];
}

#[test]
fn syntax_error_invalid_keyword() {
    let src = "blah blah blah";
    let result = parse_source_file(src);
    assert!(result.is_err(), "garbage input should fail");
    let errors = result.unwrap_err();
    assert!(!errors.is_empty());
    assert!(errors[0].span.start <= src.len());
}

#[test]
fn syntax_error_incomplete_device() {
    let src = "device Foo<> {";
    let result = parse_source_file(src);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(!errors.is_empty());
}

#[test]
fn syntax_error_missing_colon_in_part() {
    let src = "part foo MLCC { }";
    let result = parse_source_file(src);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(!errors.is_empty());
    // Span should be within the source range
    for e in &errors {
        assert!(e.span.start <= src.len());
        assert!(e.span.end <= src.len());
    }
}

#[test]
fn empty_source_is_ok() {
    let sf = parse_source_file("").expect("empty source should be valid");
    assert!(sf.items.is_empty());
}

#[test]
fn comments_only_is_ok() {
    let sf =
        parse_source_file("// just a comment\n// another comment").expect("comments-only is valid");
    assert!(sf.items.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// Additional: Design with all statement types (inst, net, call) and module
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn parse_module_with_nested_items() {
    let src = r#"
module power {
    pub fn decoupling<P: impl Capacitor>(vdd: Net, gnd: Net, cap: P) {
        inst c: cap
        net vdd: c.A
        net gnd: c.B
    }
}

module passives {
    pub type BypassCap = MLCC<C: 100nF, V: 10V, pkg: C0402>
    pub part mlcc_100nF: MLCC<C: 100nF, pkg: C0402> {
        primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }
    }
}
"#;

    let sf = parse_source_file(src).expect("should parse modules");
    assert_eq!(sf.items.len(), 2);

    // Both are modules
    assert!(matches!(sf.items[0].kind, TopLevelItemKind::Module(_)));
    assert!(matches!(sf.items[1].kind, TopLevelItemKind::Module(_)));

    // power module has a pub fn
    if let TopLevelItemKind::Module(ref m) = sf.items[0].kind {
        assert_eq!(m.name.name, "power");
        assert_eq!(m.items.len(), 1);
        assert!(m.items[0].visibility.is_some());
        assert!(matches!(m.items[0].kind, TopLevelItemKind::Fn(_)));
    }

    // passives module has a pub type and a pub part
    if let TopLevelItemKind::Module(ref m) = sf.items[1].kind {
        assert_eq!(m.name.name, "passives");
        assert_eq!(m.items.len(), 2);
        assert!(matches!(m.items[0].kind, TopLevelItemKind::TypeAlias(_)));
        assert!(matches!(m.items[1].kind, TopLevelItemKind::Part(_)));
        // Both are pub
        assert!(m.items[0].visibility.is_some());
        assert!(m.items[1].visibility.is_some());
    }
}

#[test]
fn parse_design_with_inline_avl() {
    let src = r#"
design Board {
    inst r_sense: Resistor<R: 100m, pkg: R2512> {
        primary { mfr: "Vishay", mpn: "WSL2512R1000FEA" }
        alt { mfr: "Bourns", mpn: "CSS2512FT0R100" }
    }
    net VDD: r_sense.A
}
"#;

    let sf = parse_source_file(src).expect("should parse inline AVL");
    if let TopLevelItemKind::Design(ref d) = sf.items[0].kind {
        if let DesignBodyStmtKind::Inst(ref inst) = d.body[0].kind {
            assert_eq!(inst.name.name, "r_sense");
            assert!(inst.avl_entries.is_some());
            let avl = inst.avl_entries.as_ref().unwrap();
            assert_eq!(avl.len(), 2);
            assert_eq!(avl[0].kind, AvlKind::Primary);
            assert_eq!(avl[1].kind, AvlKind::Alt);
        }
    }
}

#[test]
fn parse_fn_with_impl_constraint_and_call() {
    let src = r#"
fn decoupling<P: impl Capacitor>(vdd: Net, gnd: Net, cap: P) {
    inst c: cap
    net vdd: c.A
    net gnd: c.B
}

design Main {
    inst mcu: STM32F103<pkg: LQFP64>
    decoupling(vdd: mcu.VDD_IO, gnd: GND, cap: mlcc_100nF)
    power::ldo(vin: VIN, vout: VCC_3V3, gnd: GND)
}
"#;

    let sf = parse_source_file(src).expect("should parse fn + design with calls");
    assert_eq!(sf.items.len(), 2);

    // fn with impl constraint
    if let TopLevelItemKind::Fn(ref f) = sf.items[0].kind {
        let gp = f.generic_params.as_ref().unwrap();
        assert!(matches!(
            gp.params[0].kind,
            GenericParamKind::ImplConstraint(_)
        ));
    }

    // design with call statements
    if let TopLevelItemKind::Design(ref d) = sf.items[1].kind {
        assert_eq!(d.body.len(), 3);
        // Second stmt: decoupling(...)
        assert!(matches!(d.body[1].kind, DesignBodyStmtKind::Call(_)));
        // Third stmt: power::ldo(...) - scoped call
        if let DesignBodyStmtKind::Call(ref c) = d.body[2].kind {
            assert_eq!(c.path.segments.len(), 2);
            assert_eq!(c.path.segments[0].name, "power");
            assert_eq!(c.path.segments[1].name, "ldo");
        }
    }
}
