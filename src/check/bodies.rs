//! Declaration-time semantic validation of every FUNCTION body (review
//! R6-3 / R7-2), independent of whether a design ever calls it.
//!
//! Expansion (RFC-006) fully checks a fn only when it is inlined into the
//! selected design, so an UNCALLED fn body otherwise escapes checking. This
//! pass validates the statically-knowable statement properties over
//! `world.fns` up front: instance/call KINDS, structural-variant selection,
//! generic argument arity + concrete unit-literal types, call arity, and
//! net/nc pin references (including concrete-device pin existence).
//!
//! Scope, stated honestly: this is NOT yet the single unified semantic
//! checker shared with expansion the review ultimately asks for. Bound
//! satisfaction over abstract fn generics, layout-constraint arity in fn
//! bodies, duplicate-local and call-graph-cycle detection are still left to
//! expansion at call time — so an uncalled fn is checked for the forms below,
//! not for every property. Where a form IS checked here, the message mirrors
//! expansion's so a called fn reported by both collapses under dedup.

use crate::ast::{DeviceDef, FnDef, FnParamTy, GenericArg, GenericBound, Stmt};
use crate::diag::{Diagnostic, Diagnostics};
use crate::resolve::{short, World};
use std::collections::{BTreeMap, BTreeSet};

/// What a net/nc reference base denotes inside a fn body.
enum Base<'a> {
    /// A `Pin` value parameter — usable bare, has no `.pin`.
    Pin,
    /// An instance of a concrete device (with its selected variant's pins).
    Concrete(&'a DeviceDef, Option<String>),
    /// A trait-typed parameter or generic-typed instance — its pins are
    /// abstract trait roles, not checkable without a concrete device.
    Abstract,
}

pub fn check_fn_bodies(world: &World, diags: &mut Diagnostics) {
    for f in world.fns.values() {
        check_one(world, f, diags);
    }
}

fn check_one(world: &World, f: &FnDef, diags: &mut Diagnostics) {
    // A fn generic parameter is a valid instance TYPE only when it is
    // trait-bound (`T: SomeTrait`); a unit-bound generic (`V: Voltage`) is a
    // VALUE and may not be instantiated (review R7-2).
    let trait_generics: BTreeSet<&str> = f
        .generics
        .iter()
        .filter(|g| matches!(g.bound, GenericBound::Traits(_)))
        .map(|g| g.name.name.as_str())
        .collect();
    let unit_generics: BTreeSet<&str> = f
        .generics
        .iter()
        .filter(|g| matches!(g.bound, GenericBound::Unit(_)))
        .map(|g| g.name.name.as_str())
        .collect();

    // Reference bases: value params + every local instance.
    let mut bases: BTreeMap<&str, Base> = BTreeMap::new();
    for p in &f.params {
        match &p.ty {
            FnParamTy::Pin(_) => {
                bases.insert(p.name.name.as_str(), Base::Pin);
            }
            FnParamTy::Generic(_) | FnParamTy::ImplTrait(..) => {
                bases.insert(p.name.name.as_str(), Base::Abstract);
            }
        }
    }
    for stmt in &f.body {
        if let Stmt::Inst(i) = stmt {
            let base = classify_inst_base(world, &trait_generics, i);
            bases.insert(i.name.name.as_str(), base);
        }
    }

    for stmt in &f.body {
        match stmt {
            Stmt::Inst(inst) => {
                check_inst_kind(world, &trait_generics, &unit_generics, &inst.ty.name, diags);
                check_variant_selection(world, inst, diags);
                check_device_generic_args(world, f, inst, diags);
                check_named_generic_args(world, f, &inst.ty.generic_args, diags);
            }
            Stmt::Call(call) => {
                check_call_kind(world, &call.callee, diags);
                check_call_args(world, f, call, &bases, diags);
                check_named_generic_args(world, f, &call.generic_args, diags);
            }
            Stmt::Net(n) => {
                for m in &n.members {
                    check_pin_ref(world, &bases, m, diags);
                }
            }
            Stmt::Nc(nc) => {
                for m in &nc.members {
                    check_pin_ref(world, &bases, m, diags);
                }
            }
            Stmt::Layout(_) => {} // RFC-013 arity/nets still checked at expansion
        }
    }
}

/// The device (+ selected variant) an instance denotes, if concrete.
fn classify_inst_base<'a>(
    world: &'a World,
    trait_generics: &BTreeSet<&str>,
    inst: &crate::ast::InstStmt,
) -> Base<'a> {
    let ty = &inst.ty.name.name;
    if trait_generics.contains(ty.as_str()) {
        return Base::Abstract;
    }
    if let Some(dev) = world.devices.get(ty) {
        return Base::Concrete(dev, inst.ty.variant.as_ref().map(|v| v.name.clone()));
    }
    if let Some(part) = world.parts.get(ty) {
        if let Some(dev) = world.devices.get(&part.device.name.name) {
            let variant = part.device.variant.as_ref().map(|v| v.name.clone());
            return Base::Concrete(dev, variant);
        }
    }
    Base::Abstract // unresolved / wrong-kind: reported elsewhere
}

fn check_inst_kind(
    world: &World,
    trait_generics: &BTreeSet<&str>,
    unit_generics: &BTreeSet<&str>,
    ty: &crate::ast::Ident,
    diags: &mut Diagnostics,
) {
    let n = &ty.name;
    if trait_generics.contains(n.as_str())
        || world.devices.contains_key(n)
        || world.parts.contains_key(n)
    {
        return;
    }
    if unit_generics.contains(n.as_str()) {
        diags.push(Diagnostic::error(
            "E205",
            ty.span,
            format!(
                "`{}` is a unit-typed generic value, not a device or part — `inst` requires a concrete device or part",
                n
            ),
        ));
    } else if world.traits.contains_key(n) {
        diags.push(Diagnostic::error(
            "E205",
            ty.span,
            format!(
                "`{}` is a trait — `inst` requires a concrete device or part",
                n
            ),
        ));
    } else if world.fns.contains_key(n)
        || world.pads.contains_key(n)
        || world.footprints.contains_key(n)
    {
        let kind = world.symbols.get(n).map(|s| s.kind).unwrap_or("name");
        diags.push(Diagnostic::error(
            "E205",
            ty.span,
            format!(
                "`{}` is a {}, not a device or part — `inst` requires a concrete device or part",
                n, kind
            ),
        ));
    }
    // Unresolved: already reported at the rewrite pass (E202).
}

/// A concrete variant-bearing device instantiated with no selector is E904
/// (mirrors expansion). Wrong/spurious selectors are E903/E905 at expansion;
/// the missing-selector case is the one reachable in an uncalled fn body.
fn check_variant_selection(world: &World, inst: &crate::ast::InstStmt, diags: &mut Diagnostics) {
    let Some(dev) = world.devices.get(&inst.ty.name.name) else {
        return; // part or non-device: variant fixed / handled elsewhere
    };
    if dev.has_variants() && inst.ty.variant.is_none() {
        diags.push(
            Diagnostic::error(
                "E904",
                inst.ty.span,
                format!(
                    "device `{}` declares variants — select one with a `[VARIANT]` suffix (no implicit default)",
                    short(&inst.ty.name.name)
                ),
            )
            .with_help(format!(
                "valid variants are: {}",
                dev.variants
                    .iter()
                    .map(|v| v.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        );
    }
}

/// Generic arguments on a DEVICE instance: arity (too many → E401) and a
/// unit-literal argument whose type mismatches its unit-bound parameter
/// (E112). A `Name` argument referencing a fn generic is a valid
/// passthrough; full bound checking is deferred to call-time substitution.
fn check_device_generic_args(
    world: &World,
    _f: &FnDef,
    inst: &crate::ast::InstStmt,
    diags: &mut Diagnostics,
) {
    let Some(dev) = world.devices.get(&inst.ty.name.name) else {
        return;
    };
    let args = &inst.ty.generic_args;
    if args.len() > dev.generics.len() {
        diags.push(Diagnostic::error(
            "E401",
            inst.ty.span,
            format!(
                "device `{}` takes {} generic argument{}, but {} were given",
                short(&inst.ty.name.name),
                dev.generics.len(),
                if dev.generics.len() == 1 { "" } else { "s" },
                args.len()
            ),
        ));
    }
    for (param, arg) in dev.generics.iter().zip(args) {
        if let (GenericBound::Unit(u), GenericArg::Unit(v, span)) = (&param.bound, arg) {
            if v.unit != u.unit {
                diags.push(Diagnostic::error(
                    "E112",
                    *span,
                    format!(
                        "generic argument for `{}` has the wrong unit type: expected `{}`, found `{}`",
                        param.name.name,
                        u.unit.type_name(),
                        v.unit.type_name()
                    ),
                ));
            }
        }
    }
}

fn check_call_kind(world: &World, callee: &crate::ast::Ident, diags: &mut Diagnostics) {
    let n = &callee.name;
    if world.fns.contains_key(n) {
        return;
    }
    if world.devices.contains_key(n) || world.parts.contains_key(n) {
        diags.push(Diagnostic::error(
            "E205",
            callee.span,
            format!(
                "`{}` is a device/part — instantiate it with `inst name: {}`",
                n, n
            ),
        ));
    } else if world.traits.contains_key(n)
        || world.pads.contains_key(n)
        || world.footprints.contains_key(n)
    {
        let kind = world.symbols.get(n).map(|s| s.kind).unwrap_or("name");
        diags.push(Diagnostic::error(
            "E205",
            callee.span,
            format!("`{}` is a {}, not a callable fn", n, kind),
        ));
    }
    // Unresolved: already reported at the rewrite pass (E504).
}

/// Call value-argument count (vs the fn's `Pin` parameters) and each
/// argument's reference base (E502 arity, mirrors expansion; unknown bases
/// reuse the pin-ref path).
fn check_call_args(
    world: &World,
    _f: &FnDef,
    call: &crate::ast::CallStmt,
    bases: &BTreeMap<&str, Base>,
    diags: &mut Diagnostics,
) {
    if let Some(callee) = world.fns.get(&call.callee.name) {
        if call.args.len() != callee.params.len() {
            diags.push(Diagnostic::error(
                "E502",
                call.span,
                format!(
                    "fn `{}` takes {} argument{}, but {} were given",
                    short(&call.callee.name),
                    callee.params.len(),
                    if callee.params.len() == 1 { "" } else { "s" },
                    call.args.len()
                ),
            ));
        }
    }
    for arg in &call.args {
        check_pin_ref(world, bases, arg, diags);
    }
}

/// A named generic argument (turbofish) must resolve to a fn generic in
/// scope or a declared symbol (review R6-3).
fn check_named_generic_args(
    world: &World,
    f: &FnDef,
    args: &[GenericArg],
    diags: &mut Diagnostics,
) {
    let generics: BTreeSet<&str> = f.generics.iter().map(|g| g.name.name.as_str()).collect();
    for a in args {
        if let GenericArg::Name(id) = a {
            if generics.contains(id.name.as_str()) || world.symbols.contains_key(&id.name) {
                continue;
            }
            let mut d = Diagnostic::error(
                "E202",
                id.span,
                format!("cannot find `{}` in this scope", id.name),
            );
            if let Some(sugg) = world.suggest(&id.name) {
                d = d.with_help(format!("did you mean `{}`?", sugg));
            }
            diags.push(d);
        }
    }
}

/// A net/nc pin reference: its base must be a known local, and — for a
/// concrete device instance — the named pin must exist (mirrors expansion's
/// E202/E602/E203).
fn check_pin_ref(
    world: &World,
    bases: &BTreeMap<&str, Base>,
    r: &crate::ast::PinRef,
    diags: &mut Diagnostics,
) {
    match bases.get(r.base.name.as_str()) {
        None => {
            diags.push(Diagnostic::error(
                "E202",
                r.base.span,
                format!(
                    "unknown instance or parameter `{}` in this scope",
                    r.base.name
                ),
            ));
        }
        Some(Base::Pin) => {
            if let Some(pin) = &r.pin {
                diags.push(Diagnostic::error(
                    "E602",
                    pin.span,
                    format!(
                        "`{}` is a `Pin` parameter — it is already a pin and has no `.{}`",
                        r.base.name, pin.name
                    ),
                ));
            }
        }
        Some(Base::Concrete(dev, variant)) => {
            let pins = dev.pins_for(variant.as_deref());
            match &r.pin {
                None => {
                    diags.push(Diagnostic::error(
                        "E602",
                        r.span,
                        format!(
                            "`{}` is an instance — reference one of its pins (e.g. `{}.{}`)",
                            r.base.name,
                            r.base.name,
                            pins.first().map(|p| p.name.name.as_str()).unwrap_or("PIN")
                        ),
                    ));
                }
                Some(pin) if !pins.iter().any(|p| p.name.name == pin.name) => {
                    let _ = world;
                    diags.push(
                        Diagnostic::error(
                            "E203",
                            pin.span,
                            format!(
                                "device `{}` (instance `{}`) has no pin named `{}`",
                                short(&dev.name.name),
                                r.base.name,
                                pin.name
                            ),
                        )
                        .with_help(format!(
                            "its pins are: {}",
                            pins.iter()
                                .map(|p| p.name.name.clone())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )),
                    );
                }
                Some(_) => {}
            }
        }
        Some(Base::Abstract) => {} // trait-role access — abstract, checked at call time
    }
}
