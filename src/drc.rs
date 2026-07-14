//! RFC-004: the residual DRC engine — exactly four rules, never more.
//!
//! Every rule here is genuinely emergent from the whole connectivity graph.
//! A fifth "structural" rule request is a signal to re-run the type-system-
//! first test (RFC-004, Tooling & operations), not to add it here.
//!
//! | Code | Rule               | Severity |
//! |------|--------------------|----------|
//! | D001 | voltage-exceed     | error    |
//! | D002 | polarity-mismatch  | error    |
//! | D003 | single-driver      | warning  |
//! | D004 | multi-driver       | error    |

use crate::ast::PinRole;
use crate::diag::{Diagnostic, Diagnostics};
use crate::ir::DesignIr;
use crate::resolve::World;
use crate::units::UnitType;

pub fn run_drc(world: &World, ir: &DesignIr, diags: &mut Diagnostics) {
    for net in &ir.nets {
        // ---- D001 voltage-exceed: an instance's `voltage_rating` spec is
        // less than the voltage annotated on the net it's connected to.
        // RFC-001 comparison discipline: compare only Voltage against Voltage
        // (the annotation is grammar-guaranteed Voltage; the spec field is
        // checked here so a non-Voltage `voltage_rating` is never compared
        // across unit types).
        if let Some(net_v) = &net.voltage {
            for (path, _pin) in &net.members {
                let inst = &ir.instances[path];
                if let Some(rating) = inst.specs.get("voltage_rating") {
                    if rating.unit == UnitType::Voltage
                        && net_v.unit == UnitType::Voltage
                        && rating.femto < net_v.femto
                    {
                        diags.push(
                            Diagnostic::error(
                                "D001",
                                inst.span,
                                format!(
                                    "voltage-exceed: `{}` is rated `{}`, but net `{}` is annotated `{}`",
                                    path, rating.text, net.name, net_v.text
                                ),
                            )
                            .with_help(format!(
                                "use a part rated for at least {}, or correct the net annotation",
                                net_v.text
                            )),
                        );
                    }
                }
            }
        }

        // ---- D002 polarity-mismatch: a `Polarized` device's anode pin on a
        // GND-annotated net. RFC-016: trait identities are fully-qualified
        // paths now; D002's anchor is the trait NAMED `Polarized`, whichever
        // package declares it (matched on the path's last segment).
        if net.is_gnd {
            for (path, pin) in &net.members {
                let inst = &ir.instances[path];
                // EVERY implemented trait named `Polarized` is a candidate —
                // a project trait sharing the name must not shade the std
                // one out of the check (adversarial finding).
                let anode_hit = inst
                    .impl_traits
                    .iter()
                    .filter(|t| crate::resolve::short(t) == "Polarized")
                    .any(|polarized| {
                        world
                            .resolved_impls
                            .get(&(polarized.clone(), inst.device.clone()))
                            .and_then(|r| r.pin_map.get("Anode"))
                            == Some(pin)
                    });
                if anode_hit {
                    diags.push(
                        Diagnostic::error(
                            "D002",
                            inst.span,
                            format!(
                                "polarity-mismatch: anode pin `{}.{}` (device `{}` implements `Polarized`) is connected to GND-annotated net `{}`",
                                path, pin, crate::resolve::short(&inst.device), net.name
                            ),
                        )
                        .with_help("polarized components connect their cathode toward ground"),
                    );
                }
            }
        }

        // ---- D003 single-driver: a net whose only connected pin is a
        // driver-role pin (`output`/`power_out`) — the driver drives nothing,
        // likely unfinished wiring. Role-aware per RFC-004 (W003: "a net has
        // exactly one output-type (driver) pin connected") and RFC-008
        // ("D003/D004 … classify output/power_out as driver roles"); a lone
        // passive/input pin is not flagged (pin obligations are RFC-002's job).
        if net.members.len() == 1 {
            let (path, pin) = net.members.iter().next().unwrap();
            let role = pin_role(world, ir, path, pin);
            if role.is_driver() {
                diags.push(Diagnostic::warning(
                    "D003",
                    net.span,
                    format!(
                        "single-driver: net `{}` has only one connected pin (`{}.{}`, role `{}`) — the driver drives nothing; likely unfinished wiring",
                        net.name,
                        path,
                        pin,
                        role.name()
                    ),
                ));
            }
        }

        // ---- D004 multi-driver: two or more driver-type pins on one net.
        let drivers: Vec<String> = net
            .members
            .iter()
            .filter(|(path, pin)| pin_role(world, ir, path, pin).is_driver())
            .map(|(path, pin)| format!("`{}.{}`", path, pin))
            .collect();
        if drivers.len() >= 2 {
            diags.push(
                Diagnostic::error(
                    "D004",
                    net.span,
                    format!(
                        "multi-driver: net `{}` has {} driver-type pins: {}",
                        net.name,
                        drivers.len(),
                        drivers.join(", ")
                    ),
                )
                .with_help(
                    "driver-type pins are `output`/`power_out` — at most one may drive a net",
                ),
            );
        }
    }
}

fn pin_role(world: &World, ir: &DesignIr, path: &str, pin: &str) -> PinRole {
    let inst = &ir.instances[path];
    world.devices[&inst.device]
        .pins_for(inst.variant.as_deref())
        .iter()
        .find(|p| p.name.name == pin)
        .map(|p| p.role_or_default())
        .unwrap_or(PinRole::Passive)
}
