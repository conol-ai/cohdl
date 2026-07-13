//! RFC-003: trait satisfaction, checked exhaustively at each free-standing
//! `impl Trait for Device` statement.
//!
//! Satisfaction resolves by matching the trait's required pin-role/spec-field
//! names against the device's own already-declared names; an explicit in-body
//! mapping is used only where names differ. Empty body is the common case.
//! Sub-trait bounds require a *separate* satisfying impl to exist in scope
//! (E302) — each impl checks only its own trait's direct requirements.

use crate::ast::*;
use crate::diag::{Diagnostic, Diagnostics};
use crate::resolve::{ResolvedImpl, World};
use crate::units::UnitType;
use std::collections::BTreeMap;

pub fn check_impls(world: &mut World, diags: &mut Diagnostics) {
    let mut resolved = BTreeMap::new();
    for (key, &idx) in &world.impl_index {
        let im = &world.impls[idx];
        let tr = &world.traits[&key.0];
        let dev = &world.devices[&key.1];
        if let Some(r) = check_one(world, im, tr, dev, diags) {
            resolved.insert(key.clone(), r);
        }
    }
    world.resolved_impls = resolved;
}

fn check_one(
    world: &World,
    im: &ImplDef,
    tr: &TraitDef,
    dev: &DeviceDef,
    diags: &mut Diagnostics,
) -> Option<ResolvedImpl> {
    let mut ok = true;
    let impl_desc = format!("impl `{}` for `{}`", tr.name.name, dev.name.name);

    // --- sub-trait bounds: a separate satisfying impl must exist (E302) ---
    for sup in &tr.super_traits {
        if world.traits.contains_key(&sup.name) && !world.has_impl(&sup.name, &dev.name.name) {
            diags.push(
                Diagnostic::error(
                    "E302",
                    im.span,
                    format!(
                        "{} requires `impl {} for {}`, which was not found in scope",
                        impl_desc, sup.name, dev.name.name
                    ),
                )
                .with_secondary(
                    sup.span,
                    format!(
                        "`{}: {}` — the sub-trait bound is declared here",
                        tr.name.name, sup.name
                    ),
                )
                .with_help(format!(
                    "add a free-standing `impl {} for {} {{ … }}` anywhere in scope",
                    sup.name, dev.name.name
                )),
            );
            ok = false;
        }
    }

    // RFC-008: with variants, satisfaction must hold for EVERY variant — an
    // instance of any variant must satisfy the trait. Each "view" is one
    // variant's pin set + merged spec fields (or the single bare view).
    let views: Vec<(Option<&str>, &[DevicePin], Vec<&DeviceSpecField>)> = if dev.has_variants() {
        dev.variants
            .iter()
            .map(|v| {
                (
                    Some(v.name.as_str()),
                    dev.pins_for(Some(&v.name)),
                    dev.spec_fields_for(Some(&v.name)),
                )
            })
            .collect()
    } else {
        vec![(None, dev.pins_for(None), dev.spec_fields_for(None))]
    };
    let in_view = |label: Option<&str>| match label {
        Some(v) => format!(" (variant `{}`)", v),
        None => String::new(),
    };

    // --- mapping entries must name things that exist (E304/E305) ---
    for entry in &im.pin_map {
        if !tr.pins.iter().any(|p| p.name.name == entry.role.name) {
            diags.push(Diagnostic::error(
                "E304",
                entry.role.span,
                format!(
                    "trait `{}` does not require a pin role named `{}`",
                    tr.name.name, entry.role.name
                ),
            ));
            ok = false;
        }
        if let Some((label, ..)) = views
            .iter()
            .find(|(_, pins, _)| !pins.iter().any(|p| p.name.name == entry.target.name))
        {
            diags.push(Diagnostic::error(
                "E305",
                entry.target.span,
                format!(
                    "device `{}`{} has no pin named `{}`",
                    dev.name.name,
                    in_view(*label),
                    entry.target.name
                ),
            ));
            ok = false;
        }
    }
    for entry in &im.spec_map {
        if !tr.specs.iter().any(|s| s.name.name == entry.role.name) {
            diags.push(Diagnostic::error(
                "E304",
                entry.role.span,
                format!(
                    "trait `{}` does not require a spec field named `{}`",
                    tr.name.name, entry.role.name
                ),
            ));
            ok = false;
        }
        if let Some((label, ..)) = views
            .iter()
            .find(|(_, _, fields)| !fields.iter().any(|s| s.name.name == entry.target.name))
        {
            diags.push(Diagnostic::error(
                "E305",
                entry.target.span,
                format!(
                    "device `{}`{} has no spec field named `{}`",
                    dev.name.name,
                    in_view(*label),
                    entry.target.name
                ),
            ));
            ok = false;
        }
    }

    let mut result = ResolvedImpl::default();

    // --- pin roles: explicit mapping, else by-name, in every view (E301) ---
    for role in &tr.pins {
        let mapped = im
            .pin_map
            .iter()
            .find(|e| e.role.name == role.name.name)
            .map(|e| (&e.target, e.span));
        let (target_name, ref_span) = match mapped {
            Some((t, s)) => (t.name.clone(), s),
            None => (role.name.name.clone(), im.span),
        };
        let mut role_ok = true;
        for (label, pins, _) in &views {
            match pins.iter().find(|p| p.name.name == target_name) {
                Some(dev_pin) => {
                    if dev_pin.obligation != role.obligation {
                        diags.push(
                            Diagnostic::error(
                                "E301",
                                ref_span,
                                format!(
                                    "{} is unsatisfied: pin role `{}` is `{}` on the trait, but device pin `{}`{} is `{}`",
                                    impl_desc,
                                    role.name.name,
                                    role.obligation.keyword(),
                                    target_name,
                                    in_view(*label),
                                    dev_pin.obligation.keyword()
                                ),
                            )
                            .with_secondary(dev_pin.span, "the device pin is declared here")
                            .with_secondary(role.span, "the trait requires it here"),
                        );
                        role_ok = false;
                        break;
                    }
                }
                None => {
                    diags.push(
                        Diagnostic::error(
                            "E301",
                            im.span,
                            format!(
                                "{} is unsatisfied: trait `{}` requires pin role `{}`, and device `{}`{} has no pin with that name",
                                impl_desc,
                                tr.name.name,
                                role.name.name,
                                dev.name.name,
                                in_view(*label)
                            ),
                        )
                        .with_secondary(role.span, "required here")
                        .with_help(format!(
                            "add a mapping in the impl body: `pins {{ {}: <one of {}> }}`",
                            role.name.name,
                            pins.iter()
                                .map(|p| p.name.name.clone())
                                .collect::<Vec<_>>()
                                .join("/")
                        )),
                    );
                    role_ok = false;
                    break;
                }
            }
        }
        if role_ok {
            result
                .pin_map
                .insert(role.name.name.clone(), target_name.clone());
        } else {
            ok = false;
        }
    }

    // --- spec fields: explicit mapping, else by-name; unit types must match
    // in every view ---
    for field in &tr.specs {
        let mapped = im
            .spec_map
            .iter()
            .find(|e| e.role.name == field.name.name)
            .map(|e| (&e.target, e.span));
        let (target_name, ref_span) = match mapped {
            Some((t, s)) => (t.name.clone(), s),
            None => (field.name.name.clone(), im.span),
        };
        let mut field_ok = true;
        for (label, _, fields) in &views {
            match fields.iter().find(|s| s.name.name == target_name) {
                Some(dev_field) => match device_spec_unit(dev, dev_field) {
                    Some(unit) if unit == field.ty.unit => {}
                    Some(unit) => {
                        diags.push(
                            Diagnostic::error(
                                "E301",
                                ref_span,
                                format!(
                                    "{} is unsatisfied: spec field `{}` must be `{}`, but device field `{}`{} is `{}`",
                                    impl_desc,
                                    field.name.name,
                                    field.ty.unit.type_name(),
                                    target_name,
                                    in_view(*label),
                                    unit.type_name()
                                ),
                            )
                            .with_secondary(dev_field.span, "the device field is declared here")
                            .with_secondary(field.span, "the trait requires it here"),
                        );
                        field_ok = false;
                        break;
                    }
                    None => {
                        // Invalid generic ref — already reported by resolve.
                        field_ok = false;
                        break;
                    }
                },
                None => {
                    diags.push(
                        Diagnostic::error(
                            "E301",
                            im.span,
                            format!(
                                "{} is unsatisfied: trait `{}` requires spec field `{}` (`{}`), and device `{}`{} has no spec field with that name",
                                impl_desc,
                                tr.name.name,
                                field.name.name,
                                field.ty.unit.type_name(),
                                dev.name.name,
                                in_view(*label)
                            ),
                        )
                        .with_secondary(field.span, "required here")
                        .with_help(if fields.is_empty() {
                            format!("device `{}` declares no spec fields at all", dev.name.name)
                        } else {
                            format!(
                                "add a mapping in the impl body: `spec {{ {}: <one of {}> }}`",
                                field.name.name,
                                fields
                                    .iter()
                                    .map(|s| s.name.name.clone())
                                    .collect::<Vec<_>>()
                                    .join("/")
                            )
                        }),
                    );
                    field_ok = false;
                    break;
                }
            }
        }
        if field_ok {
            result
                .spec_map
                .insert(field.name.name.clone(), target_name.clone());
        } else {
            ok = false;
        }
    }

    ok.then_some(result)
}

/// The unit type of a device spec field: a literal's own unit, or the unit
/// bound of the generic parameter it references ("matching generic parameter
/// bound for generic devices" — RFC-003).
fn device_spec_unit(dev: &DeviceDef, field: &DeviceSpecField) -> Option<UnitType> {
    match &field.value {
        SpecValue::Lit(v, _) => Some(v.unit),
        SpecValue::GenericRef(r) => {
            let g = dev.generics.iter().find(|g| g.name.name == r.name)?;
            match &g.bound {
                GenericBound::Unit(u) => Some(u.unit),
                GenericBound::Traits(_) => None,
            }
        }
    }
}
