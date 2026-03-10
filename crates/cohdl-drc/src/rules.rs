//! Built-in DRC rule implementations.
//!
//! Each struct implements [`DrcRule`] and is named after its rule ID in the spec.

use std::collections::{HashMap, HashSet};

use cohdl_sema::connectivity::{ConnectivityIR, Instance, Net};
use cohdl_sema::typeck::EXTERNAL_INSTANCE;
use cohdl_syntax::ast::Span;

use crate::{DiagnosticLevel, DrcDiagnostic, DrcRule};

/// Convenience: build a diagnostic.
fn diag(
    rule_id: &str,
    level: DiagnosticLevel,
    instance_path: &str,
    message: impl Into<String>,
) -> DrcDiagnostic {
    DrcDiagnostic {
        rule_id: rule_id.to_string(),
        level,
        span: Span { start: 0, end: 0 },
        instance_path: instance_path.to_string(),
        message: message.into(),
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Build a map from InstanceId → &Instance for fast lookup.
fn instance_map(ir: &ConnectivityIR) -> HashMap<cohdl_sema::typeck::InstanceId, &Instance> {
    ir.instances.iter().map(|i| (i.id, i)).collect()
}

/// Parse a voltage string like `"3.3V"`, `"5V"`, `"1.8V"` into an f64.
fn parse_voltage(s: &str) -> Option<f64> {
    let s = s.trim();
    let numeric = s.trim_end_matches('V').trim_end_matches('v');
    numeric.parse::<f64>().ok()
}

/// Check whether a net name conventionally indicates a GND net.
fn is_gnd_net(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower == "gnd" || lower == "vss" || lower == "gnd_analog" || lower.starts_with("gnd")
}

// ── E001: voltage_exceed ─────────────────────────────────────────────────────

/// Instance `voltage_rating` is less than the voltage annotation on the net it
/// is connected to.
pub struct VoltageExceed;

impl DrcRule for VoltageExceed {
    fn check(&self, ir: &ConnectivityIR) -> Vec<DrcDiagnostic> {
        let imap = instance_map(ir);
        let mut out = Vec::new();

        for net in &ir.nets {
            // Determine the net voltage from its name or from a connected
            // instance that carries a `voltage` generic substitution.
            let net_voltage = net_voltage_annotation(net, &imap);
            let net_v = match net_voltage {
                Some(v) => v,
                None => continue,
            };

            for pin in &net.pins {
                if pin.instance_id == EXTERNAL_INSTANCE {
                    continue;
                }
                if let Some(inst) = imap.get(&pin.instance_id) {
                    if let Some(rating_str) = inst.generic_substitutions.get("voltage_rating") {
                        if let Some(rating) = parse_voltage(rating_str) {
                            if rating < net_v {
                                out.push(diag(
                                    "E001",
                                    DiagnosticLevel::Error,
                                    &inst.hierarchical_path,
                                    format!(
                                        "voltage_rating {}V is less than net `{}` voltage {}V",
                                        rating, net.name, net_v
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
        }
        out
    }
}

/// Derive net voltage: look for instances on the net that have a `voltage`
/// generic substitution, or try to parse the net name (e.g. `"3V3"` → 3.3).
fn net_voltage_annotation(
    net: &Net,
    imap: &HashMap<cohdl_sema::typeck::InstanceId, &Instance>,
) -> Option<f64> {
    // First: check for any instance that carries a `voltage` substitution.
    for pin in &net.pins {
        if pin.instance_id == EXTERNAL_INSTANCE {
            continue;
        }
        if let Some(inst) = imap.get(&pin.instance_id) {
            if let Some(v_str) = inst.generic_substitutions.get("voltage") {
                if let Some(v) = parse_voltage(v_str) {
                    return Some(v);
                }
            }
        }
    }
    // Second: try to extract voltage from net name (e.g. "3V3" → 3.3, "5V" → 5.0).
    parse_net_name_voltage(&net.name)
}

fn parse_net_name_voltage(name: &str) -> Option<f64> {
    // "3V3" → 3.3, "5V" → 5.0, "1V8" → 1.8
    if let Some(pos) = name.find('V') {
        let before = &name[..pos];
        let after = &name[pos + 1..];
        if let Ok(int) = before.parse::<u32>() {
            if after.is_empty() {
                return Some(int as f64);
            }
            if let Ok(frac) = after.parse::<u32>() {
                let denom = 10f64.powi(after.len() as i32);
                return Some(int as f64 + frac as f64 / denom);
            }
        }
    }
    None
}

// ── E002: polarity_mismatch ──────────────────────────────────────────────────

/// A device implementing `Polarized` has its anode (`A`) pin connected to a
/// GND-annotated net.
pub struct PolarityMismatch;

impl DrcRule for PolarityMismatch {
    fn check(&self, ir: &ConnectivityIR) -> Vec<DrcDiagnostic> {
        let imap = instance_map(ir);
        let mut out = Vec::new();

        for net in &ir.nets {
            if !is_gnd_net(&net.name) {
                continue;
            }
            for pin in &net.pins {
                if pin.instance_id == EXTERNAL_INSTANCE {
                    continue;
                }
                if pin.pin != "A" {
                    continue;
                }
                if let Some(inst) = imap.get(&pin.instance_id) {
                    if is_polarized(inst) {
                        out.push(diag(
                            "E002",
                            DiagnosticLevel::Error,
                            &inst.hierarchical_path,
                            format!(
                                "polarized device `{}` has anode (A) connected to GND net `{}`",
                                inst.device, net.name
                            ),
                        ));
                    }
                }
            }
        }
        out
    }
}

/// A device is "polarized" if its generic substitutions contain
/// `impl_trait = "Polarized"` or it has an `impl_traits` entry containing
/// `"Polarized"`.
fn is_polarized(inst: &Instance) -> bool {
    inst.generic_substitutions
        .iter()
        .any(|(k, v)| (k == "impl_traits" || k == "impl_trait") && v.contains("Polarized"))
}

// ── E003: spec_not_satisfied ─────────────────────────────────────────────────

/// A device implementing a trait that requires spec fields is missing one.
pub struct SpecNotSatisfied;

impl DrcRule for SpecNotSatisfied {
    fn check(&self, ir: &ConnectivityIR) -> Vec<DrcDiagnostic> {
        let mut out = Vec::new();
        for inst in &ir.instances {
            if let Some(required) = inst.generic_substitutions.get("required_specs") {
                let required_fields: Vec<&str> = required
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                for field in required_fields {
                    if !inst.generic_substitutions.contains_key(field) {
                        out.push(diag(
                            "E003",
                            DiagnosticLevel::Error,
                            &inst.hierarchical_path,
                            format!(
                                "trait spec field `{}` missing on device `{}`",
                                field, inst.device
                            ),
                        ));
                    }
                }
            }
        }
        out
    }
}

// ── E004: trait_not_impl ─────────────────────────────────────────────────────

/// A generic argument doesn't implement the required trait.
pub struct TraitNotImpl;

impl DrcRule for TraitNotImpl {
    fn check(&self, ir: &ConnectivityIR) -> Vec<DrcDiagnostic> {
        let mut out = Vec::new();
        for inst in &ir.instances {
            if let Some(required) = inst.generic_substitutions.get("required_traits") {
                let required_traits: Vec<&str> = required
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                let impl_traits: HashSet<&str> = inst
                    .generic_substitutions
                    .get("impl_traits")
                    .map(|s| s.split(',').map(|t| t.trim()).collect())
                    .unwrap_or_default();
                for t in required_traits {
                    if !impl_traits.contains(t) {
                        out.push(diag(
                            "E004",
                            DiagnosticLevel::Error,
                            &inst.hierarchical_path,
                            format!(
                                "generic argument on `{}` does not implement required trait `{}`",
                                inst.device, t
                            ),
                        ));
                    }
                }
            }
        }
        out
    }
}

// ── E005: missing_spec_field ─────────────────────────────────────────────────

/// A trait spec field is not provided in the device instantiation.
pub struct MissingSpecField;

impl DrcRule for MissingSpecField {
    fn check(&self, ir: &ConnectivityIR) -> Vec<DrcDiagnostic> {
        let mut out = Vec::new();
        for inst in &ir.instances {
            if let Some(expected) = inst.generic_substitutions.get("expected_spec_fields") {
                let fields: Vec<&str> = expected
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                for field in fields {
                    if !inst.generic_substitutions.contains_key(field) {
                        out.push(diag(
                            "E005",
                            DiagnosticLevel::Error,
                            &inst.hierarchical_path,
                            format!(
                                "spec field `{}` not provided in device `{}`",
                                field, inst.device
                            ),
                        ));
                    }
                }
            }
        }
        out
    }
}

// ── W001: unconnected_pin ────────────────────────────────────────────────────

/// A pin on an instance has no net connection.
pub struct UnconnectedPin;

impl DrcRule for UnconnectedPin {
    fn check(&self, ir: &ConnectivityIR) -> Vec<DrcDiagnostic> {
        // Collect all (instance_id, pin) pairs that appear on any net.
        let connected: HashSet<(cohdl_sema::typeck::InstanceId, &str)> = ir
            .nets
            .iter()
            .flat_map(|n| n.pins.iter())
            .filter(|p| p.instance_id != EXTERNAL_INSTANCE)
            .map(|p| (p.instance_id, p.pin.as_str()))
            .collect();

        let mut out = Vec::new();
        for inst in &ir.instances {
            // If the instance declares expected pins via generic_substitutions,
            // check each one.
            if let Some(pins_str) = inst.generic_substitutions.get("declared_pins") {
                let declared: Vec<&str> = pins_str
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();
                for pin in declared {
                    if !connected.contains(&(inst.id, pin)) {
                        out.push(diag(
                            "W001",
                            DiagnosticLevel::Warning,
                            &inst.hierarchical_path,
                            format!("pin `{}` on `{}` is unconnected", pin, inst.name),
                        ));
                    }
                }
            }
        }
        out
    }
}

// ── W002: floating_net ───────────────────────────────────────────────────────

/// A net exists but has no instance pins connected (only external or none).
pub struct FloatingNet;

impl DrcRule for FloatingNet {
    fn check(&self, ir: &ConnectivityIR) -> Vec<DrcDiagnostic> {
        let mut out = Vec::new();
        for net in &ir.nets {
            let instance_pin_count = net
                .pins
                .iter()
                .filter(|p| p.instance_id != EXTERNAL_INSTANCE)
                .count();
            if instance_pin_count == 0 {
                out.push(diag(
                    "W002",
                    DiagnosticLevel::Error,
                    "",
                    format!(
                        "net `{}` exists but has no instance pins connected",
                        net.name
                    ),
                ));
            }
        }
        out
    }
}

// ── W003: single_driver ──────────────────────────────────────────────────────

/// A net has exactly one passive-pin connection (likely unfinished wiring).
pub struct SingleDriver;

impl DrcRule for SingleDriver {
    fn check(&self, ir: &ConnectivityIR) -> Vec<DrcDiagnostic> {
        let mut out = Vec::new();
        for net in &ir.nets {
            let instance_pins: Vec<_> = net
                .pins
                .iter()
                .filter(|p| p.instance_id != EXTERNAL_INSTANCE)
                .collect();
            if instance_pins.len() == 1 {
                let pin = &instance_pins[0];
                let imap = instance_map(ir);
                let inst_path = imap
                    .get(&pin.instance_id)
                    .map(|i| i.hierarchical_path.as_str())
                    .unwrap_or("");
                out.push(diag(
                    "W003",
                    DiagnosticLevel::Warning,
                    inst_path,
                    format!(
                        "net `{}` has only one instance pin connection ({}.{})",
                        net.name, inst_path, pin.pin
                    ),
                ));
            }
        }
        out
    }
}

// ── W004: multi_driver ───────────────────────────────────────────────────────

/// A net has multiple output-type pins connected.
pub struct MultiDriver;

impl DrcRule for MultiDriver {
    fn check(&self, ir: &ConnectivityIR) -> Vec<DrcDiagnostic> {
        let imap = instance_map(ir);
        let mut out = Vec::new();
        for net in &ir.nets {
            let mut output_pins = Vec::new();
            for pin in &net.pins {
                if pin.instance_id == EXTERNAL_INSTANCE {
                    continue;
                }
                if let Some(inst) = imap.get(&pin.instance_id) {
                    // Check if this pin is declared as an output via
                    // `output_pins` generic substitution.
                    if let Some(outputs) = inst.generic_substitutions.get("output_pins") {
                        let output_set: HashSet<&str> =
                            outputs.split(',').map(|s| s.trim()).collect();
                        if output_set.contains(pin.pin.as_str()) {
                            output_pins.push((inst, &pin.pin));
                        }
                    }
                }
            }
            if output_pins.len() > 1 {
                let drivers: Vec<String> = output_pins
                    .iter()
                    .map(|(inst, pin)| format!("{}.{}", inst.hierarchical_path, pin))
                    .collect();
                out.push(diag(
                    "W004",
                    DiagnosticLevel::Error,
                    "",
                    format!(
                        "net `{}` has multiple output drivers: {}",
                        net.name,
                        drivers.join(", ")
                    ),
                ));
            }
        }
        out
    }
}
