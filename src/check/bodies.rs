//! Declaration-time semantic validation of every FUNCTION body (review
//! R6-3), independent of whether a design ever calls it.
//!
//! Expansion (RFC-006) fully checks a fn only when it is inlined into the
//! selected design, so an UNCALLED fn body previously received a clean
//! verdict even with a wrong-kind instantiation, a wrong-kind call, an
//! unresolved generic argument, or a net/nc reference to a nonexistent
//! local. This pass validates those forms over `world.fns` up front, so the
//! CLI/LSP verdict is sound regardless of reachability.
//!
//! Messages mirror expansion's exactly, so the same diagnostic reported by
//! both this pass and expansion (for a fn that IS called) collapses under
//! `Diagnostics::sort`'s exact-duplicate dedup rather than doubling.

use crate::ast::{FnParamTy, GenericArg, Stmt};
use crate::diag::{Diagnostic, Diagnostics};
use crate::resolve::World;
use std::collections::BTreeSet;

pub fn check_fn_bodies(world: &World, diags: &mut Diagnostics) {
    for f in world.fns.values() {
        // In-scope names: the fn's generic parameters (valid as instance
        // TYPES — a generic-typed inst), and its value parameters + all
        // local instances (valid as net/nc reference BASES).
        let generics: BTreeSet<&str> = f.generics.iter().map(|g| g.name.name.as_str()).collect();
        let mut bases: BTreeSet<&str> = f.params.iter().map(|p| p.name.name.as_str()).collect();
        for stmt in &f.body {
            if let Stmt::Inst(i) = stmt {
                bases.insert(i.name.name.as_str());
            }
        }
        // Trait-bound value params are also valid instance-like bases.
        for p in &f.params {
            if let FnParamTy::Generic(_) | FnParamTy::ImplTrait(..) = p.ty {
                bases.insert(p.name.name.as_str());
            }
        }

        for stmt in &f.body {
            match stmt {
                Stmt::Inst(inst) => {
                    check_inst_kind(world, &generics, &inst.ty.name, diags);
                    check_generic_args(world, &generics, &inst.ty.generic_args, diags);
                }
                Stmt::Call(call) => {
                    check_call_kind(world, &call.callee, diags);
                    check_generic_args(world, &generics, &call.generic_args, diags);
                }
                Stmt::Net(n) => {
                    for m in &n.members {
                        check_base(world, &generics, &bases, &m.base, diags);
                    }
                }
                Stmt::Nc(nc) => {
                    for m in &nc.members {
                        check_base(world, &generics, &bases, &m.base, diags);
                    }
                }
                Stmt::Layout(_) => {} // RFC-013 constraints — checked at expansion
            }
        }
    }
}

/// An `inst` type must be a device, a part, or one of the fn's own generic
/// parameters — not a trait, fn, or pad (mirrors expansion's E205 messages).
fn check_inst_kind(
    world: &World,
    generics: &BTreeSet<&str>,
    ty: &crate::ast::Ident,
    diags: &mut Diagnostics,
) {
    let n = &ty.name;
    if generics.contains(n.as_str()) || world.devices.contains_key(n) || world.parts.contains_key(n)
    {
        return;
    }
    if world.traits.contains_key(n) {
        diags.push(Diagnostic::error(
            "E205",
            ty.span,
            format!(
                "`{}` is a trait — `inst` requires a concrete device or part",
                n
            ),
        ));
    } else if world.fns.contains_key(n) || world.pads.contains_key(n) {
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
    // Unresolved (in no map): already reported at the rewrite pass (E202).
}

/// A call target must be a fn — not a device or part (mirrors expansion's
/// E205 message for the device/part case).
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
    }
    // Unresolved: already reported at the rewrite pass (E504).
}

/// A named generic argument must resolve to SOMETHING — a fn generic
/// parameter in scope, or a declared symbol. Full bound-checking is a
/// call-time concern (abstract fn generics have no concrete value here), but
/// a name that resolves to nothing is unknown regardless (review R6-3).
fn check_generic_args(
    world: &World,
    generics: &BTreeSet<&str>,
    args: &[GenericArg],
    diags: &mut Diagnostics,
) {
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

/// A net/nc reference base must be a fn parameter or a local instance
/// (mirrors expansion's E202 "unknown instance or parameter" message).
fn check_base(
    world: &World,
    generics: &BTreeSet<&str>,
    bases: &BTreeSet<&str>,
    base: &crate::ast::Ident,
    diags: &mut Diagnostics,
) {
    if bases.contains(base.name.as_str()) || generics.contains(base.name.as_str()) {
        return;
    }
    // A bare name may also be an instance passed as a call argument elsewhere;
    // but as a net/nc base it must be a local. Unknown otherwise.
    let _ = world;
    diags.push(Diagnostic::error(
        "E202",
        base.span,
        format!(
            "unknown instance or parameter `{}` in this scope",
            base.name
        ),
    ));
}
