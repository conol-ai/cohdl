//! Unit tests for DRC rules using synthetic ConnectivityIR fixtures.

use cohdl_sema::connectivity::{ConnectivityIR, Instance, Net, PinRef};
use cohdl_sema::typeck::{InstanceId, EXTERNAL_INSTANCE};

use crate::rules::*;
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
