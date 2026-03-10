//! Unit tests for DRC rules using synthetic ConnectivityIR fixtures.

use cohdl_sema::connectivity::{ConnectivityIR, Instance, Net, PinRef};
use cohdl_sema::typeck::{InstanceId, EXTERNAL_INSTANCE};

use crate::rules::*;
use crate::user_rules::{
    RuleAppliesTo, RuleBinOp, RuleExpr, UserDefinedRule, UserDefinedRuleSet,
};
use crate::{DiagnosticLevel, DrcRule, DrcRunner};

// ── Fixture helpers ──────────────────────────────────────────────────────────

fn inst(id: u32, name: &str, device: &str, subs: &[(&str, &str)]) -> Instance {
    Instance {
        id: InstanceId(id),
        name: name.to_string(),
        hierarchical_path: format!("Board::{}", name),
        device: device.to_string(),
        mpn: None,
        generic_substitutions: subs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    }
}

fn pinref(id: u32, pin: &str) -> PinRef {
    PinRef {
        instance_id: InstanceId(id),
        pin: pin.to_string(),
    }
}

fn ext_pin(pin: &str) -> PinRef {
    PinRef {
        instance_id: EXTERNAL_INSTANCE,
        pin: pin.to_string(),
    }
}

fn net(name: &str, pins: Vec<PinRef>) -> Net {
    Net {
        name: name.to_string(),
        pins,
    }
}

fn ir(instances: Vec<Instance>, nets: Vec<Net>) -> ConnectivityIR {
    ConnectivityIR { instances, nets }
}

// ── E001: voltage_exceed ─────────────────────────────────────────────────────

#[test]
fn e001_triggers_when_rating_below_net_voltage() {
    let design = ir(
        vec![inst(
            0,
            "c1",
            "MLCC",
            &[("voltage_rating", "3.3V"), ("voltage", "5V")],
        )],
        vec![net("5V", vec![pinref(0, "A")])],
    );
    let diags = VoltageExceed.check(&design);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "E001");
    assert_eq!(diags[0].level, DiagnosticLevel::Error);
    assert!(diags[0].message.contains("voltage_rating"));
    assert_eq!(diags[0].instance_path, "Board::c1");
}

#[test]
fn e001_no_trigger_when_rating_sufficient() {
    let design = ir(
        vec![inst(
            0,
            "c1",
            "MLCC",
            &[("voltage_rating", "10V"), ("voltage", "5V")],
        )],
        vec![net("5V", vec![pinref(0, "A")])],
    );
    let diags = VoltageExceed.check(&design);
    assert!(diags.is_empty());
}

#[test]
fn e001_detects_voltage_from_net_name() {
    // Net name "3V3" implies 3.3 V; a 3 V rated part should fail.
    let design = ir(
        vec![inst(0, "c1", "MLCC", &[("voltage_rating", "3V")])],
        vec![net("3V3", vec![pinref(0, "A")])],
    );
    let diags = VoltageExceed.check(&design);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "E001");
}

// ── E002: polarity_mismatch ──────────────────────────────────────────────────

#[test]
fn e002_triggers_anode_on_gnd() {
    let design = ir(
        vec![inst(0, "d1", "LED", &[("impl_traits", "Polarized")])],
        vec![net("GND", vec![ext_pin("GND"), pinref(0, "A")])],
    );
    let diags = PolarityMismatch.check(&design);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "E002");
    assert!(diags[0].message.contains("anode"));
}

#[test]
fn e002_no_trigger_cathode_on_gnd() {
    let design = ir(
        vec![inst(0, "d1", "LED", &[("impl_traits", "Polarized")])],
        vec![net("GND", vec![ext_pin("GND"), pinref(0, "K")])],
    );
    let diags = PolarityMismatch.check(&design);
    assert!(diags.is_empty());
}

#[test]
fn e002_no_trigger_non_polarized() {
    let design = ir(
        vec![inst(0, "c1", "MLCC", &[])],
        vec![net("GND", vec![ext_pin("GND"), pinref(0, "A")])],
    );
    let diags = PolarityMismatch.check(&design);
    assert!(diags.is_empty());
}

// ── E003: spec_not_satisfied ─────────────────────────────────────────────────

#[test]
fn e003_triggers_missing_spec() {
    let design = ir(
        vec![inst(
            0,
            "c1",
            "MLCC",
            &[("required_specs", "capacitance,voltage_rating")],
        )],
        vec![],
    );
    let diags = SpecNotSatisfied.check(&design);
    assert_eq!(diags.len(), 2);
    assert!(diags.iter().all(|d| d.rule_id == "E003"));
}

#[test]
fn e003_no_trigger_when_specs_present() {
    let design = ir(
        vec![inst(
            0,
            "c1",
            "MLCC",
            &[("required_specs", "capacitance"), ("capacitance", "100nF")],
        )],
        vec![],
    );
    let diags = SpecNotSatisfied.check(&design);
    assert!(diags.is_empty());
}

// ── E004: trait_not_impl ─────────────────────────────────────────────────────

#[test]
fn e004_triggers_missing_trait() {
    let design = ir(
        vec![inst(
            0,
            "x",
            "Generic",
            &[
                ("required_traits", "Capacitor"),
                ("impl_traits", "Resistor"),
            ],
        )],
        vec![],
    );
    let diags = TraitNotImpl.check(&design);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "E004");
    assert!(diags[0].message.contains("Capacitor"));
}

#[test]
fn e004_no_trigger_when_trait_present() {
    let design = ir(
        vec![inst(
            0,
            "x",
            "Generic",
            &[
                ("required_traits", "Capacitor"),
                ("impl_traits", "Capacitor"),
            ],
        )],
        vec![],
    );
    let diags = TraitNotImpl.check(&design);
    assert!(diags.is_empty());
}

// ── E005: missing_spec_field ─────────────────────────────────────────────────

#[test]
fn e005_triggers_missing_field() {
    let design = ir(
        vec![inst(
            0,
            "c1",
            "MLCC",
            &[("expected_spec_fields", "capacitance,tolerance")],
        )],
        vec![],
    );
    let diags = MissingSpecField.check(&design);
    assert_eq!(diags.len(), 2);
    assert!(diags.iter().all(|d| d.rule_id == "E005"));
}

#[test]
fn e005_no_trigger_when_fields_present() {
    let design = ir(
        vec![inst(
            0,
            "c1",
            "MLCC",
            &[
                ("expected_spec_fields", "capacitance"),
                ("capacitance", "100nF"),
            ],
        )],
        vec![],
    );
    let diags = MissingSpecField.check(&design);
    assert!(diags.is_empty());
}

// ── W001: unconnected_pin ────────────────────────────────────────────────────

#[test]
fn w001_triggers_on_unconnected_pin() {
    let design = ir(
        vec![inst(0, "c1", "MLCC", &[("declared_pins", "A,B")])],
        vec![net("VDD", vec![pinref(0, "A")])],
    );
    let diags = UnconnectedPin.check(&design);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "W001");
    assert_eq!(diags[0].level, DiagnosticLevel::Warning);
    assert!(diags[0].message.contains("B"));
}

#[test]
fn w001_no_trigger_all_connected() {
    let design = ir(
        vec![inst(0, "c1", "MLCC", &[("declared_pins", "A,B")])],
        vec![
            net("VDD", vec![pinref(0, "A")]),
            net("GND", vec![pinref(0, "B")]),
        ],
    );
    let diags = UnconnectedPin.check(&design);
    assert!(diags.is_empty());
}

// ── W002: floating_net ───────────────────────────────────────────────────────

#[test]
fn w002_triggers_on_floating_net() {
    let design = ir(vec![], vec![net("ORPHAN", vec![ext_pin("ORPHAN")])]);
    let diags = FloatingNet.check(&design);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "W002");
    assert_eq!(diags[0].level, DiagnosticLevel::Error);
}

#[test]
fn w002_no_trigger_with_instance_pin() {
    let design = ir(
        vec![inst(0, "c1", "MLCC", &[])],
        vec![net("VDD", vec![ext_pin("VDD"), pinref(0, "A")])],
    );
    let diags = FloatingNet.check(&design);
    assert!(diags.is_empty());
}

// ── W003: single_driver ──────────────────────────────────────────────────────

#[test]
fn w003_triggers_single_instance_pin() {
    let design = ir(
        vec![inst(0, "c1", "MLCC", &[])],
        vec![net("LONELY", vec![pinref(0, "A")])],
    );
    let diags = SingleDriver.check(&design);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "W003");
    assert_eq!(diags[0].level, DiagnosticLevel::Warning);
}

#[test]
fn w003_no_trigger_two_instance_pins() {
    let design = ir(
        vec![inst(0, "c1", "MLCC", &[]), inst(1, "r1", "RES", &[])],
        vec![net("N1", vec![pinref(0, "A"), pinref(1, "A")])],
    );
    let diags = SingleDriver.check(&design);
    assert!(diags.is_empty());
}

// ── W004: multi_driver ───────────────────────────────────────────────────────

#[test]
fn w004_triggers_multiple_outputs() {
    let design = ir(
        vec![
            inst(0, "u1", "BUF", &[("output_pins", "Y")]),
            inst(1, "u2", "BUF", &[("output_pins", "Y")]),
        ],
        vec![net("BUS", vec![pinref(0, "Y"), pinref(1, "Y")])],
    );
    let diags = MultiDriver.check(&design);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "W004");
    assert_eq!(diags[0].level, DiagnosticLevel::Error);
    assert!(diags[0].message.contains("multiple output"));
}

#[test]
fn w004_no_trigger_single_output() {
    let design = ir(
        vec![
            inst(0, "u1", "BUF", &[("output_pins", "Y")]),
            inst(1, "r1", "RES", &[]),
        ],
        vec![net("SIG", vec![pinref(0, "Y"), pinref(1, "A")])],
    );
    let diags = MultiDriver.check(&design);
    assert!(diags.is_empty());
}

// ── DrcRunner integration ────────────────────────────────────────────────────

#[test]
fn runner_collects_diagnostics_from_all_rules() {
    let design = ir(
        vec![inst(0, "d1", "LED", &[("impl_traits", "Polarized")])],
        vec![net("GND", vec![ext_pin("GND"), pinref(0, "A")])],
    );
    let runner = DrcRunner::new();
    let diags = runner.run(&design);
    assert!(diags.iter().any(|d| d.rule_id == "E002"));
}

#[test]
fn runner_allow_suppresses_diagnostic() {
    let design = ir(
        vec![inst(0, "d1", "LED", &[("impl_traits", "Polarized")])],
        vec![net("GND", vec![ext_pin("GND"), pinref(0, "A")])],
    );
    let mut runner = DrcRunner::new();
    runner.allow("Board::d1", "E002");
    let diags = runner.run(&design);
    assert!(!diags
        .iter()
        .any(|d| d.rule_id == "E002" && d.instance_path == "Board::d1"));
}

#[test]
fn runner_empty_design_no_diagnostics() {
    let design = ir(vec![], vec![]);
    let runner = DrcRunner::new();
    let diags = runner.run(&design);
    assert!(diags.is_empty());
}

// ── User-defined rules: expression evaluation ──────────────────────────────

#[test]
fn user_rule_spec_field_float_literal() {
    // Simple assertion: self.spec.voltage_rating >= 5.0
    let rule = UserDefinedRule {
        name: "min_voltage".into(),
        level: DiagnosticLevel::Warning,
        assertion: RuleExpr::Binary {
            op: RuleBinOp::Ge,
            lhs: Box::new(RuleExpr::SpecField("voltage_rating".into())),
            rhs: Box::new(RuleExpr::Float(5.0)),
        },
        message_template: "voltage_rating {voltage_rating} is below 5V".into(),
        applies_to: RuleAppliesTo::Trait("Capacitor".into()),
    };

    let mut rules = UserDefinedRuleSet::new();
    rules.add(rule);

    // Instance with rating 3.3V — should trigger.
    let design = ir(
        vec![inst(
            0,
            "c1",
            "MLCC",
            &[
                ("voltage_rating", "3.3V"),
                ("impl_traits", "Capacitor"),
            ],
        )],
        vec![],
    );
    let diags = rules.check(&design);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "min_voltage");
    assert_eq!(diags[0].level, DiagnosticLevel::Warning);
    assert!(diags[0].message.contains("3.3V"));
}

#[test]
fn user_rule_no_trigger_when_assertion_holds() {
    let rule = UserDefinedRule {
        name: "min_voltage".into(),
        level: DiagnosticLevel::Warning,
        assertion: RuleExpr::Binary {
            op: RuleBinOp::Ge,
            lhs: Box::new(RuleExpr::SpecField("voltage_rating".into())),
            rhs: Box::new(RuleExpr::Float(5.0)),
        },
        message_template: "too low".into(),
        applies_to: RuleAppliesTo::Trait("Capacitor".into()),
    };

    let mut rules = UserDefinedRuleSet::new();
    rules.add(rule);

    let design = ir(
        vec![inst(
            0,
            "c1",
            "MLCC",
            &[
                ("voltage_rating", "10V"),
                ("impl_traits", "Capacitor"),
            ],
        )],
        vec![],
    );
    let diags = rules.check(&design);
    assert!(diags.is_empty());
}

#[test]
fn user_rule_does_not_apply_to_unrelated_device() {
    let rule = UserDefinedRule {
        name: "cap_check".into(),
        level: DiagnosticLevel::Warning,
        assertion: RuleExpr::Float(0.0), // Would always be falsy, but shouldn't apply.
        message_template: "should not fire".into(),
        applies_to: RuleAppliesTo::Trait("Capacitor".into()),
    };

    let mut rules = UserDefinedRuleSet::new();
    rules.add(rule);

    // Instance is a Resistor, not a Capacitor.
    let design = ir(
        vec![inst(0, "r1", "RES", &[("impl_traits", "Resistor")])],
        vec![],
    );
    let diags = rules.check(&design);
    assert!(diags.is_empty());
}

// ── User-defined rules: voltage_derating from Capacitor trait ──────────────

/// Build the `voltage_derating` rule as it would appear in a `Capacitor` trait:
///
/// ```hdl
/// rule voltage_derating(level: Warning) {
///     assert self.spec.voltage_rating * 0.8 >= net_voltage(self.A, self.B)
///     message: "net voltage {net_voltage}V exceeds 80% derating of {voltage_rating}"
/// }
/// ```
fn capacitor_voltage_derating_rule() -> UserDefinedRule {
    UserDefinedRule {
        name: "voltage_derating".into(),
        level: DiagnosticLevel::Warning,
        assertion: RuleExpr::Binary {
            op: RuleBinOp::Ge,
            lhs: Box::new(RuleExpr::Binary {
                op: RuleBinOp::Mul,
                lhs: Box::new(RuleExpr::SpecField("voltage_rating".into())),
                rhs: Box::new(RuleExpr::Float(0.8)),
            }),
            rhs: Box::new(RuleExpr::NetVoltage {
                pin_a: "A".into(),
                pin_b: "B".into(),
            }),
        },
        message_template:
            "net voltage {net_voltage}V exceeds 80% derating of {voltage_rating}".into(),
        applies_to: RuleAppliesTo::Trait("Capacitor".into()),
    }
}

#[test]
fn voltage_derating_triggers_when_close_to_rating() {
    // Capacitor rated 10V on a 9V net — 9 > 10*0.8 = 8, so should trigger.
    let mut rules = UserDefinedRuleSet::new();
    rules.add(capacitor_voltage_derating_rule());

    let design = ir(
        vec![inst(
            0,
            "c1",
            "MLCC",
            &[
                ("voltage_rating", "10V"),
                ("impl_traits", "Capacitor"),
            ],
        )],
        vec![net("9V", vec![pinref(0, "A"), ext_pin("9V")])],
    );

    let diags = rules.check(&design);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "voltage_derating");
    assert_eq!(diags[0].level, DiagnosticLevel::Warning);
    assert_eq!(diags[0].instance_path, "Board::c1");
    assert!(diags[0].message.contains("9"));
    assert!(diags[0].message.contains("derating"));
}

#[test]
fn voltage_derating_no_trigger_within_margin() {
    // Capacitor rated 10V on a 5V net — 5 <= 10*0.8 = 8, so should NOT trigger.
    let mut rules = UserDefinedRuleSet::new();
    rules.add(capacitor_voltage_derating_rule());

    let design = ir(
        vec![inst(
            0,
            "c1",
            "MLCC",
            &[
                ("voltage_rating", "10V"),
                ("impl_traits", "Capacitor"),
            ],
        )],
        vec![net("5V", vec![pinref(0, "A"), ext_pin("5V")])],
    );

    let diags = rules.check(&design);
    assert!(diags.is_empty());
}

// ── User-defined rules: device override of trait rule ──────────────────────

/// Build a stricter `voltage_derating` rule for an `Electrolytic` device:
/// 50% derating instead of 80%.
///
/// ```hdl
/// device Electrolytic: impl Capacitor {
///     rule voltage_derating(level: Warning) {
///         assert self.spec.voltage_rating * 0.5 >= net_voltage(self.A, self.B)
///         message: "net voltage {net_voltage}V exceeds 50% derating of {voltage_rating}"
///     }
/// }
/// ```
fn electrolytic_voltage_derating_override() -> UserDefinedRule {
    UserDefinedRule {
        name: "voltage_derating".into(),
        level: DiagnosticLevel::Warning,
        assertion: RuleExpr::Binary {
            op: RuleBinOp::Ge,
            lhs: Box::new(RuleExpr::Binary {
                op: RuleBinOp::Mul,
                lhs: Box::new(RuleExpr::SpecField("voltage_rating".into())),
                rhs: Box::new(RuleExpr::Float(0.5)),
            }),
            rhs: Box::new(RuleExpr::NetVoltage {
                pin_a: "A".into(),
                pin_b: "B".into(),
            }),
        },
        message_template:
            "net voltage {net_voltage}V exceeds 50% derating of {voltage_rating}".into(),
        applies_to: RuleAppliesTo::Device("Electrolytic".into()),
    }
}

#[test]
fn device_override_uses_stricter_rule() {
    // Both the trait rule (80%) and the device rule (50%) are registered.
    // For an Electrolytic instance, the device rule should override the trait rule.
    // Rated 10V on a 6V net:
    //   - 80% rule: 10*0.8 = 8 >= 6 → passes
    //   - 50% rule: 10*0.5 = 5 >= 6 → FAILS → should trigger
    let mut rules = UserDefinedRuleSet::new();
    rules.add(capacitor_voltage_derating_rule());
    rules.add(electrolytic_voltage_derating_override());

    let design = ir(
        vec![inst(
            0,
            "c1",
            "Electrolytic",
            &[
                ("voltage_rating", "10V"),
                ("impl_traits", "Capacitor"),
            ],
        )],
        vec![net("6V", vec![pinref(0, "A"), ext_pin("6V")])],
    );

    let diags = rules.check(&design);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "voltage_derating");
    // Confirm it's the device's 50% message, not the trait's 80%.
    assert!(diags[0].message.contains("50%"));
}

#[test]
fn trait_rule_still_applies_to_non_overriding_device() {
    // An MLCC (non-electrolytic) should still use the trait's 80% derating.
    // Rated 10V on a 9V net: 10*0.8 = 8 < 9 → triggers.
    let mut rules = UserDefinedRuleSet::new();
    rules.add(capacitor_voltage_derating_rule());
    rules.add(electrolytic_voltage_derating_override());

    let design = ir(
        vec![inst(
            0,
            "c1",
            "MLCC",
            &[
                ("voltage_rating", "10V"),
                ("impl_traits", "Capacitor"),
            ],
        )],
        vec![net("9V", vec![pinref(0, "A"), ext_pin("9V")])],
    );

    let diags = rules.check(&design);
    assert_eq!(diags.len(), 1);
    assert!(diags[0].message.contains("80%"));
}

#[test]
fn device_override_no_trigger_within_stricter_margin() {
    // Electrolytic rated 10V on a 4V net: 10*0.5 = 5 >= 4 → passes.
    let mut rules = UserDefinedRuleSet::new();
    rules.add(capacitor_voltage_derating_rule());
    rules.add(electrolytic_voltage_derating_override());

    let design = ir(
        vec![inst(
            0,
            "c1",
            "Electrolytic",
            &[
                ("voltage_rating", "10V"),
                ("impl_traits", "Capacitor"),
            ],
        )],
        vec![net("4V", vec![pinref(0, "A"), ext_pin("4V")])],
    );

    let diags = rules.check(&design);
    assert!(diags.is_empty());
}

// ── Integration: user-defined rules in DrcRunner ───────────────────────────

#[test]
fn runner_includes_user_defined_diagnostics() {
    let mut rules = UserDefinedRuleSet::new();
    rules.add(capacitor_voltage_derating_rule());

    let mut runner = DrcRunner::new();
    runner.add_user_rules(rules);

    let design = ir(
        vec![inst(
            0,
            "c1",
            "MLCC",
            &[
                ("voltage_rating", "10V"),
                ("impl_traits", "Capacitor"),
            ],
        )],
        vec![net("9V", vec![pinref(0, "A"), ext_pin("9V")])],
    );

    let diags = runner.run(&design);
    assert!(diags.iter().any(|d| d.rule_id == "voltage_derating"));
}

#[test]
fn runner_suppresses_user_defined_diagnostic() {
    let mut rules = UserDefinedRuleSet::new();
    rules.add(capacitor_voltage_derating_rule());

    let mut runner = DrcRunner::new();
    runner.add_user_rules(rules);
    runner.allow("Board::c1", "voltage_derating");

    let design = ir(
        vec![inst(
            0,
            "c1",
            "MLCC",
            &[
                ("voltage_rating", "10V"),
                ("impl_traits", "Capacitor"),
            ],
        )],
        vec![net("9V", vec![pinref(0, "A"), ext_pin("9V")])],
    );

    let diags = runner.run(&design);
    assert!(!diags.iter().any(|d| d.rule_id == "voltage_derating"));
}

// ── Expression IR: arithmetic operations ───────────────────────────────────

#[test]
fn user_rule_mul_and_comparison() {
    // Assert: self.spec.capacitance * 1000.0 >= 100.0
    // i.e. capacitance must be at least 100mF = 0.1F when multiplied by 1000.
    // 100nF = 1e-7 → 1e-7 * 1000 = 1e-4 < 100 → should trigger.
    let rule = UserDefinedRule {
        name: "min_cap".into(),
        level: DiagnosticLevel::Error,
        assertion: RuleExpr::Binary {
            op: RuleBinOp::Ge,
            lhs: Box::new(RuleExpr::Binary {
                op: RuleBinOp::Mul,
                lhs: Box::new(RuleExpr::SpecField("capacitance".into())),
                rhs: Box::new(RuleExpr::Float(1000.0)),
            }),
            rhs: Box::new(RuleExpr::Float(100.0)),
        },
        message_template: "capacitance too low".into(),
        applies_to: RuleAppliesTo::Device("MLCC".into()),
    };

    let mut rules = UserDefinedRuleSet::new();
    rules.add(rule);

    let design = ir(
        vec![inst(0, "c1", "MLCC", &[("capacitance", "100nF")])],
        vec![],
    );

    let diags = rules.check(&design);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "min_cap");
    assert_eq!(diags[0].level, DiagnosticLevel::Error);
}

#[test]
fn user_rule_net_voltage_with_instance_voltage_annotation() {
    // Instance carries voltage=5V, so net_voltage should resolve to 5.0.
    let mut rules = UserDefinedRuleSet::new();
    rules.add(capacitor_voltage_derating_rule());

    // c1 rated 6V; voltage source instance on same net annotated 5V.
    // 6*0.8 = 4.8 < 5 → triggers.
    let design = ir(
        vec![
            inst(
                0,
                "c1",
                "MLCC",
                &[
                    ("voltage_rating", "6V"),
                    ("impl_traits", "Capacitor"),
                ],
            ),
            inst(1, "vreg", "LDO", &[("voltage", "5V")]),
        ],
        vec![net(
            "VDD",
            vec![pinref(0, "A"), pinref(1, "OUT")],
        )],
    );

    let diags = rules.check(&design);
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].rule_id, "voltage_derating");
}
