//! RFC-018: the footprint/device pad-consistency check — the check RFC-017
//! deferred, now real because footprint pad lists are real structured data.
//!
//! Declaration-complete (review R5-4): the check walks `world.parts`, not
//! the instantiated IR, so a bad part is caught whether or not a design
//! happens to instantiate it — the correctness of a declaration-only
//! library must not depend on a consumer exercising every exported part.
//! It runs at BUILD (RFC-018 pins the pad/device comparison to `cohdl build`;
//! review R6-5), but declaration-complete — it walks `world.parts`, not the
//! instantiated IR, so an unused part is still checked. Every AVL entry's footprint is checked — an
//! alt-sourced part must fit the same land pattern, or the mismatch stays
//! latent until a fab swaps sources. A footprint with a fully EMPTY body is
//! RFC-017's stage-one placeholder and is exempt (a courtyard-only
//! footprint is real content whose pad set is ∅, not a placeholder). The
//! permanence of that exemption is a documented deviation from RFC-018's
//! exact-match rule (docs/compliance-report.md, review R5-4).

use crate::ast::FootprintDef;
use crate::diag::{Diagnostic, Diagnostics};
use crate::resolve::{short, World};
use std::collections::BTreeSet;

/// A footprint with no body content at all — RFC-017's stage-one
/// placeholder shape, exempt from the pad-consistency check and from
/// geometry projection.
pub fn is_placeholder(fp: &FootprintDef) -> bool {
    fp.pads.is_empty()
        && fp.mount_holes.is_empty()
        && fp.courtyard.is_none()
        && fp.silkscreen_ref.is_none()
}

pub fn check_pad_consistency(world: &World, diags: &mut Diagnostics) {
    // One report per (part, footprint) pair.
    let mut reported: BTreeSet<(&str, &str)> = BTreeSet::new();
    for (part_name, part) in &world.parts {
        let Some(device) = world.devices.get(&part.device.name.name) else {
            continue; // unresolved device: reported at declaration check
        };
        // A part pins its own structural variant (RFC-008).
        let variant = part.device.variant.as_ref().map(|v| v.name.as_str());
        // Skip the pad comparison unless the variant selection is structurally
        // VALID (review R6-5/R7-8): `pins_for` returns an empty set for any
        // ill-formed selection, which would fabricate a spurious "extra pad"
        // E807 on top of the real E903 (unknown variant) / E904 (missing
        // selector) / E905 (selector on a non-variant device).
        let variant_ok = match (device.has_variants(), variant) {
            (true, Some(v)) => device.variants.iter().any(|dv| dv.name == v),
            (true, None) => false, // E904: a variant device needs a selector
            (false, None) => true, // ordinary device, no selector: fine
            (false, Some(_)) => false, // E905: selector on a non-variant device
        };
        if !variant_ok {
            continue;
        }
        for entry in std::iter::once(&part.primary).chain(part.alts.iter()) {
            let Some(fp_ref) = &entry.footprint else {
                continue;
            };
            let Some(fp) = world.footprints.get(&fp_ref.name) else {
                continue; // unresolved footprint: reported at declaration check
            };
            if is_placeholder(fp) {
                continue; // stage-one placeholder (RFC-017 migration)
            }
            if !reported.insert((part_name.as_str(), fp_ref.name.as_str())) {
                continue;
            }
            let device_pins: BTreeSet<&str> = device
                .pins_for(variant)
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
                        short(&part.device.name.name),
                        parts_msg.join("; ")
                    ),
                )
                .with_secondary(
                    fp.name.span,
                    "the footprint's pads are declared here".to_string(),
                )
                .with_help(
                    "the footprint's distinct electrical pad-number set must exactly match the device pin-number set; one number may have multiple physical placements",
                ),
            );
        }
    }
}
