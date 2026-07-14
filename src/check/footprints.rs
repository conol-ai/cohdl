//! RFC-018: the footprint/device pad-consistency check — the check RFC-017
//! deferred, now real because footprint pad lists are real structured data.
//!
//! Runs at `cohdl build`, at the point every instance's part binding is
//! known (the same point MPN completeness is checked — RFC-003's
//! precedent; `cohdl check` does not require footprint content). Every AVL
//! entry's footprint is checked — an alt-sourced part must fit the same
//! land pattern contract, or the mismatch stays latent until a fab swaps
//! sources. A footprint with a fully EMPTY body is RFC-017's stage-one
//! placeholder and is exempt — the check applies the moment real content
//! exists (a courtyard-only footprint is real content whose pad set is ∅,
//! not a placeholder).

use crate::ast::FootprintDef;
use crate::diag::{Diagnostic, Diagnostics};
use crate::ir::DesignIr;
use crate::resolve::{short, World};
use std::collections::BTreeSet;

/// A footprint with no body content at all — RFC-017's stage-one
/// placeholder shape, exempt from the pad-consistency check and from
/// geometry projection.
pub fn is_placeholder(fp: &FootprintDef) -> bool {
    fp.pads.is_empty() && fp.courtyard.is_none() && fp.silkscreen_ref.is_none()
}

pub fn check_pad_consistency(world: &World, ir: &DesignIr, diags: &mut Diagnostics) {
    // One report per (part, footprint) pair — every instance of the same
    // part would repeat the identical mismatch.
    let mut reported: BTreeSet<(String, String)> = BTreeSet::new();
    for inst in ir.instances.values() {
        let Some(part_name) = &inst.part else {
            continue;
        };
        let Some(part) = world.parts.get(part_name) else {
            continue;
        };
        let Some(device) = world.devices.get(&inst.device) else {
            continue;
        };
        let entries = std::iter::once(&part.primary).chain(part.alts.iter());
        for entry in entries {
            let Some(fp_ref) = &entry.footprint else {
                continue;
            };
            let Some(fp) = world.footprints.get(&fp_ref.name) else {
                continue; // unresolved: already reported at declaration check
            };
            if is_placeholder(fp) {
                continue; // stage-one placeholder (RFC-017 migration)
            }
            if !reported.insert((part_name.clone(), fp_ref.name.clone())) {
                continue;
            }
            let device_pins: BTreeSet<&str> = device
                .pins_for(inst.variant.as_deref())
                .iter()
                .flat_map(|p| p.numbers.iter().map(|n| n.text.as_str()))
                .collect();
            let fp_pads: BTreeSet<&str> = fp.pads.iter().map(|p| p.number.text.as_str()).collect();
            if device_pins == fp_pads {
                continue;
            }
            let missing: Vec<&str> = device_pins.difference(&fp_pads).copied().collect();
            let extra: Vec<&str> = fp_pads.difference(&device_pins).copied().collect();
            let mut parts_msg = Vec::new();
            if !missing.is_empty() {
                parts_msg.push(format!(
                    "missing pad{} {}",
                    if missing.len() == 1 { "" } else { "s" },
                    missing
                        .iter()
                        .map(|n| format!("`{}`", n))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            if !extra.is_empty() {
                parts_msg.push(format!(
                    "extra pad{} {}",
                    if extra.len() == 1 { "" } else { "s" },
                    extra
                        .iter()
                        .map(|n| format!("`{}`", n))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
            diags.push(
                Diagnostic::error(
                    "E807",
                    fp_ref.span,
                    format!(
                        "footprint `{}` does not match device `{}`'s pins: {}",
                        short(&fp_ref.name),
                        short(&inst.device),
                        parts_msg.join("; ")
                    ),
                )
                .with_secondary(
                    fp.name.span,
                    "the footprint's pads are declared here".to_string(),
                )
                .with_help(
                    "every device pin number needs exactly one `pad N: …` placement (RFC-018)",
                ),
            );
        }
    }
}
