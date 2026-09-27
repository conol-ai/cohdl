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
//! The statement-kind pass checks concrete pins and signatures recursively;
//! lexical expression validation shares one dependency evaluator between
//! definition, actual activation, and iteration environments. Uncalled bodies
//! retain unknown generic/binder values and never specialize callees.

use crate::ast::{
    ConstTy, DeviceDef, FnDef, FnParamTy, GenericArg, GenericBound, LayoutFor, Stmt, SubdesignDef,
};
use crate::check::eval::{NameKind, Ty};
use crate::diag::{Diagnostic, Diagnostics};
use crate::resolve::{short, World};
use std::collections::{BTreeMap, BTreeSet};

/// What a net/nc reference base denotes inside a fn body.
enum Base<'a> {
    /// A `Pin` value parameter — usable bare, has no `.pin`.
    Pin,
    /// An instance of a concrete device (with its selected variant's pins).
    Concrete(&'a DeviceDef, Option<String>),
    /// RFC-032: a subdesign use site — its ports are its whole reference
    /// surface (`x.PORT`), named here for precise checking.
    Sub {
        sub_short: &'a str,
        ports: std::collections::BTreeSet<&'a str>,
    },
    /// A trait-typed parameter or generic-typed instance — its pins are
    /// abstract trait roles, not checkable without a concrete device.
    Abstract,
}

pub fn check_fn_bodies(world: &World, diags: &mut Diagnostics) {
    for f in world.fns.values() {
        check_one(world, f, false, diags);
    }
}

/// RFC-032: a subdesign body is checked AS IF it were a fn whose parameters
/// are its `Pin`-shaped ports — the same statement machinery, so an unused
/// subdesign cannot hide what an uncalled fn cannot.
pub fn check_subdesign_body(world: &World, _fq: &str, s: &SubdesignDef, diags: &mut Diagnostics) {
    let shim = FnDef {
        name: s.name.clone(),
        generics: s.generics.clone(),
        params: s
            .ports
            .iter()
            .map(|p| crate::ast::FnParam {
                name: p.name.clone(),
                ty: FnParamTy::Pin(p.span),
                span: p.span,
            })
            .collect(),
        body: s.body.clone(),
    };
    check_one(world, &shim, true, diags);
}

fn check_one(world: &World, f: &FnDef, allow_sub_use: bool, diags: &mut Diagnostics) {
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
        // RFC-032: a nested use site's ports are its reference surface.
        if let Stmt::SubdesignUse(u) = stmt {
            let base = match world.subdesigns.get(&u.ty.name.name) {
                Some(sd) => Base::Sub {
                    sub_short: short(&u.ty.name.name),
                    ports: sd.ports.iter().map(|p| p.name.name.as_str()).collect(),
                },
                None => Base::Abstract, // unresolved/wrong kind: reported elsewhere
            };
            bases.insert(u.name.name.as_str(), base);
        }
    }

    let checks = DefinitionChecks {
        world,
        f,
        allow_sub_use,
        trait_generics,
        unit_generics,
        bases,
    };
    checks.walk(&f.body, diags);

    // RFC-033 §8: uniform static declaration validation — decidable without
    // values (duplicate locals, const kinds, loop bounds, loop-body admits).
    let mut ctx = StaticCtx {
        world,
        check_names: true,
        generic_names: f.generics.iter().map(|g| g.name.name.clone()).collect(),
        names: {
            let mut m = BTreeMap::new();
            for g in &f.generics {
                match &g.bound {
                    GenericBound::Int(_) => {
                        m.insert(g.name.name.clone(), NameKind::Unknown(Ty::Int));
                    }
                    GenericBound::Unit(u) if u.unit == crate::units::UnitType::Length => {
                        m.insert(g.name.name.clone(), NameKind::Unknown(Ty::Length));
                    }
                    _ => {
                        m.insert(g.name.name.clone(), NameKind::NonValue);
                    }
                }
            }
            for p in &f.params {
                m.insert(p.name.name.clone(), NameKind::NonValue);
            }
            m
        },
        unknown_arrays: BTreeSet::new(),
        array_lens: BTreeMap::new(),
        labels: BTreeSet::new(),
        seen_locals: f
            .generics
            .iter()
            .map(|g| g.name.name.clone())
            .chain(f.params.iter().map(|p| p.name.name.clone()))
            .collect(),
    };
    check_stmts(&mut ctx, &f.body, false, diags);
}

struct DefinitionChecks<'a> {
    world: &'a World,
    f: &'a FnDef,
    allow_sub_use: bool,
    trait_generics: BTreeSet<&'a str>,
    unit_generics: BTreeSet<&'a str>,
    bases: BTreeMap<&'a str, Base<'a>>,
}

impl DefinitionChecks<'_> {
    fn walk(&self, stmts: &[Stmt], diags: &mut Diagnostics) {
        let checks = self;
        let Self {
            world,
            f,
            allow_sub_use,
            trait_generics,
            unit_generics,
            bases,
        } = self;
        for stmt in stmts {
            match stmt {
                Stmt::Inst(inst) => {
                    check_inst_kind(world, trait_generics, unit_generics, &inst.ty.name, diags);
                    check_variant_selection(world, inst, diags);
                    check_device_generic_args(world, inst, diags);
                }
                Stmt::Call(call) => {
                    check_call_kind(world, &call.callee, diags);
                    check_call_args(world, f, call, bases, diags);
                }
                Stmt::Net(n) => {
                    for m in &n.members {
                        check_pin_ref(world, bases, m, diags);
                    }
                }
                Stmt::Nc(nc) => {
                    for m in &nc.members {
                        check_pin_ref(world, bases, m, diags);
                    }
                }
                Stmt::Layout(_) => {} // RFC-013 arity/nets still checked at expansion
                // RFC-033 §8: uniform static validation — handled by the
                // check_stmts recursion below (duplicate locals, consts, loops).
                // RFC-032: legal in a subdesign body; rejected in a fn so an
                // UNCALLED fn cannot hide one (expansion re-checks called fns).
                // RFC-033 §8: static validation handles consts/loops via the
                // check_stmts recursion below.
                Stmt::Const(_) => {}
                Stmt::For(loop_) => checks.walk(&loop_.body, diags),
                Stmt::SubdesignUse(sub) => {
                    if !*allow_sub_use {
                        diags.push(Diagnostic::error(
                        "E1307",
                        sub.span,
                        "a `subdesign` use site needs a retained hierarchy path — a `fn` expands inline and cannot contain one (RFC-032); move it into the design or a subdesign".to_string(),
                    ));
                        continue;
                    }
                    check_sub_use_site(world, f, sub, diags);
                    for conn in &sub.conns {
                        // A bare name may be a net (resolved at expansion); only
                        // dotted references are statically checkable here.
                        if conn.value.pin.is_some() {
                            check_pin_ref(world, bases, &conn.value, diags);
                        }
                    }
                }
            }
        }
    }
}

/// RFC-033 §8: the static-validation context — name kinds for `type_check`,
/// declared arrays (length unevaluated at this stage), labels, and the
/// duplicate-local ledger (the §8 compatibility correction).
#[derive(Clone)]
struct StaticCtx<'a> {
    world: &'a World,
    check_names: bool,
    generic_names: BTreeSet<String>,
    names: BTreeMap<String, NameKind>,
    unknown_arrays: BTreeSet<String>,
    array_lens: BTreeMap<String, i64>,
    labels: BTreeSet<String>,
    seen_locals: BTreeSet<String>,
}

fn duplicate(id: &crate::ast::Ident, diags: &mut Diagnostics) {
    diags.push(Diagnostic::error(
        "E201",
        id.span,
        format!("`{}` is already defined in this scope", id.name),
    ));
}

fn const_local(c: &crate::ast::ConstStmt) -> crate::check::eval::LocalExpr {
    crate::check::eval::LocalExpr {
        name: c.name.clone(),
        value: c.value.clone(),
        span: c.span,
        ty: Some(match c.ty {
            ConstTy::Int => Ty::Int,
            ConstTy::Length => Ty::Length,
        }),
    }
}

/// Reserve this lexical body's names before checking any expression. Nets
/// may have repeated declarations, but a new const/label cannot share them.
fn prepare_scope(ctx: &mut StaticCtx, stmts: &[Stmt], diags: &mut Diagnostics) {
    let mut declarations = Vec::new();
    let mut locals = Vec::new();
    for stmt in stmts {
        match stmt {
            Stmt::Inst(i) => {
                declarations.push((&i.name, false, true));
                ctx.names
                    .entry(i.name.name.clone())
                    .or_insert(NameKind::NonValue);
                if let Some((value, _)) = &i.array_len {
                    locals.push(crate::check::eval::LocalExpr {
                        name: i.name.clone(),
                        value: value.clone(),
                        span: value.span(),
                        ty: None,
                    });
                }
            }
            Stmt::SubdesignUse(u) => {
                declarations.push((&u.name, false, true));
                ctx.names
                    .entry(u.name.name.clone())
                    .or_insert(NameKind::NonValue);
                if let Some((value, _)) = &u.array_len {
                    locals.push(crate::check::eval::LocalExpr {
                        name: u.name.clone(),
                        value: value.clone(),
                        span: value.span(),
                        ty: None,
                    });
                }
            }
            Stmt::Const(c) => {
                declarations.push((&c.name, true, true));
                locals.push(const_local(c));
            }
            Stmt::For(f) => declarations.push((&f.label, true, true)),
            Stmt::Net(n) => {
                if let Some(name) = &n.name {
                    declarations.push((name, false, false));
                }
            }
            _ => {}
        }
    }
    // Diagnose only the later declaration. Legacy net names may repeat;
    // a const or label conflicts with any earlier declaration of its name.
    let mut local_names: BTreeMap<&str, (bool, bool)> = BTreeMap::new();
    for (name, new_kind, unique) in &declarations {
        let duplicate_here =
            local_names
                .get(name.name.as_str())
                .is_some_and(|(earlier_new, earlier_unique)| {
                    *new_kind || *earlier_new || (*unique && *earlier_unique)
                });
        if (*unique && ctx.seen_locals.contains(&name.name)) || duplicate_here {
            duplicate(name, diags);
        }
        let earlier = local_names.entry(name.name.as_str()).or_default();
        earlier.0 |= *new_kind;
        earlier.1 |= *unique;
    }
    ctx.seen_locals
        .extend(declarations.iter().map(|(id, _, _)| id.name.clone()));
    crate::check::eval::resolve_locals(
        &locals,
        &mut ctx.names,
        &mut ctx.array_lens,
        &mut ctx.unknown_arrays,
        &[],
        diags,
    );
}

/// Definition and bound validation share the same lexical graph. Child
/// constants/binders never leak; only the definition's label ledger returns.
fn check_stmts(ctx: &mut StaticCtx, stmts: &[Stmt], in_loop: bool, diags: &mut Diagnostics) {
    prepare_scope(ctx, stmts, diags);
    for stmt in stmts {
        match stmt {
            Stmt::Inst(i) => {
                if in_loop {
                    diags.push(Diagnostic::error("E1406", i.span,
                        "`inst` is not admitted inside a `for` body — declare the array outside the loop and repeat only connections, calls and placements here (RFC-033 Candidate A)"));
                }
                check_named_generic_args(
                    ctx,
                    &i.ty.generic_args,
                    ctx.world
                        .devices
                        .get(&i.ty.name.name)
                        .map(|d| d.generics.as_slice()),
                    diags,
                );
            }
            Stmt::SubdesignUse(u) => {
                if in_loop {
                    diags.push(Diagnostic::error("E1406", u.span,
                        "a `subdesign` use site is not admitted inside a `for` body — declare it outside the loop"));
                }
                check_named_generic_args(
                    ctx,
                    &u.ty.generic_args,
                    ctx.world
                        .subdesigns
                        .get(&u.ty.name.name)
                        .map(|d| d.generics.as_slice()),
                    diags,
                );
                for conn in &u.conns {
                    check_selector_static(ctx, &conn.value, diags);
                }
            }
            Stmt::Const(_) => {} // prepared dependency-first above
            Stmt::For(f) => {
                if !ctx.labels.insert(f.label.name.clone()) {
                    duplicate(&f.label, diags);
                }
                if ctx.seen_locals.contains(&f.binder.name) {
                    duplicate(&f.binder, diags);
                }
                static_type_check(ctx, &f.start, Ty::Int, "a loop bound", diags);
                static_type_check(ctx, &f.end, Ty::Int, "a loop bound", diags);
                let mut inner = ctx.clone();
                inner
                    .names
                    .insert(f.binder.name.clone(), NameKind::Unknown(Ty::Int));
                inner.seen_locals.insert(f.binder.name.clone());
                check_stmts(&mut inner, &f.body, true, diags);
                ctx.labels = inner.labels;
            }
            Stmt::Call(call) => {
                check_named_generic_args(
                    ctx,
                    &call.generic_args,
                    ctx.world
                        .fns
                        .get(&call.callee.name)
                        .map(|d| d.generics.as_slice()),
                    diags,
                );
                for arg in &call.args {
                    check_selector_static(ctx, arg, diags);
                }
            }
            Stmt::Net(n) => {
                // Syntactic admission (named loop nets, E1406): definition-time
                // only — `ctx.check_names` is true in the definition pass
                // (check_one) and false in the per-activation bound_context
                // pass, so a called helper/subdesign reports exactly once at
                // its definition site and is not re-reported per expansion.
                if in_loop && ctx.check_names {
                    if let Some(name) = &n.name {
                        diags.push(
                            Diagnostic::error(
                                "E1406",
                                name.span,
                                format!(
                                    "a named net `{name}` is not admitted inside a `for` body — declare the named net outside the loop and connect it here with an anonymous `net _` through the shared pins/ports (RFC-033 named loop nets)",
                                    name = name.name
                                ),
                            )
                            .with_help(
                                "move `net NAME: …` outside the loop and use `net _: OUTER.PIN, …` inside the loop to join it",
                            ),
                        );
                    }
                }
                for m in &n.members {
                    check_selector_static(ctx, m, diags);
                }
            }
            Stmt::Nc(nc) => {
                for m in &nc.members {
                    check_selector_static(ctx, m, diags);
                }
            }
            Stmt::Layout(block) => {
                let mut inner = ctx.clone();
                let consts: Vec<_> = block.consts.iter().cloned().map(Stmt::Const).collect();
                prepare_scope(&mut inner, &consts, diags);
                for p in &block.placements {
                    check_placement_static(&inner, p, diags);
                }
                check_layout_for_static(&mut inner, &block.loops, diags);
                ctx.labels = inner.labels;
            }
        }
    }
}

fn check_layout_for_static(ctx: &mut StaticCtx, loops: &[LayoutFor], diags: &mut Diagnostics) {
    // All labels are visible to collision checks even when declared later.
    for f in loops {
        if ctx.seen_locals.contains(&f.label.name) {
            duplicate(&f.label, diags);
        }
    }
    ctx.seen_locals
        .extend(loops.iter().map(|f| f.label.name.clone()));
    for f in loops {
        if !ctx.labels.insert(f.label.name.clone()) {
            duplicate(&f.label, diags);
        }
        if ctx.seen_locals.contains(&f.binder.name) {
            duplicate(&f.binder, diags);
        }
        static_type_check(ctx, &f.start, Ty::Int, "a loop bound", diags);
        static_type_check(ctx, &f.end, Ty::Int, "a loop bound", diags);
        let mut inner = ctx.clone();
        inner
            .names
            .insert(f.binder.name.clone(), NameKind::Unknown(Ty::Int));
        inner.seen_locals.insert(f.binder.name.clone());
        let consts: Vec<_> = f.consts.iter().cloned().map(Stmt::Const).collect();
        prepare_scope(&mut inner, &consts, diags);
        for p in &f.placements {
            check_placement_static(&inner, p, diags);
        }
        check_layout_for_static(&mut inner, &f.loops, diags);
        ctx.labels = inner.labels;
    }
}

fn check_selector_static(ctx: &StaticCtx, member: &crate::ast::PinRef, diags: &mut Diagnostics) {
    use crate::ast::IndexSel;
    match &member.index {
        Some(IndexSel::Single(e, _)) => static_type_check(ctx, e, Ty::Int, "an index", diags),
        Some(IndexSel::List(items, _)) => {
            for e in items {
                static_type_check(ctx, e, Ty::Int, "an index", diags);
            }
        }
        Some(IndexSel::Range {
            start, end, step, ..
        }) => {
            static_type_check(ctx, start, Ty::Int, "a range bound", diags);
            static_type_check(ctx, end, Ty::Int, "a range bound", diags);
            if let Some(step) = step {
                static_type_check(ctx, step, Ty::Int, "a stride", diags);
            }
        }
        None => {}
    }
}

/// Type-check one expression with ANY resulting kind (generic arguments may
/// be Int or Length) — still surfaces invariant E1402/E1403.
fn static_type_check_any(
    ctx: &StaticCtx,
    e: &crate::ast::Expr,
    what: &str,
    diags: &mut Diagnostics,
) -> crate::check::eval::Ty {
    use crate::check::eval::Env;
    let env = Env {
        names: &ctx.names,
        array_lens: &ctx.array_lens,
        unknown_arrays: &ctx.unknown_arrays,
    };
    let t = crate::check::eval::type_check(e, &env, diags);
    match t {
        Some(t) => t,
        None => {
            let _ = what;
            Ty::Int
        }
    }
}

/// Type-check one expression statically; `type_check` also evaluates concrete
/// subexpressions, surfacing invariant E1402/E1403 without values for the
/// rest.
fn static_type_check(
    ctx: &StaticCtx,
    e: &crate::ast::Expr,
    want: Ty,
    what: &str,
    diags: &mut Diagnostics,
) {
    use crate::check::eval::Env;
    let env = Env {
        names: &ctx.names,
        array_lens: &ctx.array_lens,
        unknown_arrays: &ctx.unknown_arrays,
    };
    match crate::check::eval::type_check(e, &env, diags) {
        Some(t) if t == want => {}
        Some(_) => {
            diags.push(Diagnostic::error(
                "E1401",
                e.span(),
                format!(
                    "{} must be an {}, but `{}` is a {}",
                    what,
                    match want {
                        Ty::Int => "Int",
                        Ty::Length => "Length",
                    },
                    crate::ast::expr_text(e),
                    match crate::check::eval::type_check(e, &env, &mut Diagnostics::new()) {
                        Some(Ty::Int) => "Int",
                        _ => "Length",
                    }
                ),
            ));
        }
        None => {}
    }
}

/// Statically type-check a placement's expressions (at → Length, rotate →
/// Int, segment indexes → Int).
fn check_placement_static(ctx: &StaticCtx, p: &crate::ast::Placement, diags: &mut Diagnostics) {
    static_type_check(ctx, &p.at.0, Ty::Length, "placement x", diags);
    static_type_check(ctx, &p.at.1, Ty::Length, "placement y", diags);
    if let Some(r) = &p.rotate {
        static_type_check(ctx, r, Ty::Int, "a rotation", diags);
    }
    for seg in &p.path {
        if let Some((e, _)) = &seg.index {
            static_type_check(ctx, e, Ty::Int, "an index", diags);
        }
    }
}

/// RFC-033 §8: every design body gets the same static validation a fn body
/// does (allowing subdesign use sites).
pub fn check_design_bodies(world: &World, diags: &mut Diagnostics) {
    for design in world.designs.values() {
        let shim = FnDef {
            name: design.name.clone(),
            generics: Vec::new(),
            params: Vec::new(),
            body: design.body.clone(),
        };
        check_one(world, &shim, true, diags);
    }
}

/// RFC-033 §8: statically validate a loop body ONCE per loop entry under the
/// activation's known names (consts, binders, Int/Length generics bound by
/// the call). Values are known here, so an activation-bound `1 / N` with
/// `N = 0` reports E1403 — while the UNCALLEd definition check never sees a
/// value (no specialization of skipped activations).
pub fn check_loop_body_bound(
    world: &World,
    body: &[Stmt],
    names: &BTreeMap<String, NameKind>,
    bases: &BTreeSet<String>,
    array_lens: &BTreeMap<String, i64>,
    diags: &mut Diagnostics,
) {
    let mut ctx = bound_context(world, names, bases, array_lens);
    check_stmts(&mut ctx, body, true, diags);
}

fn bound_context<'a>(
    world: &'a World,
    names: &BTreeMap<String, NameKind>,
    bases: &BTreeSet<String>,
    array_lens: &BTreeMap<String, i64>,
) -> StaticCtx<'a> {
    StaticCtx {
        world,
        check_names: false,
        generic_names: BTreeSet::new(),
        names: names.clone(),
        unknown_arrays: BTreeSet::new(),
        array_lens: array_lens.clone(),
        labels: BTreeSet::new(),
        seen_locals: names.keys().chain(bases.iter()).cloned().collect(),
    }
}

pub fn check_layout_bound(
    world: &World,
    block: &crate::ast::LayoutBlock,
    names: &BTreeMap<String, NameKind>,
    array_lens: &BTreeMap<String, i64>,
    diags: &mut Diagnostics,
) {
    let mut ctx = bound_context(world, names, &BTreeSet::new(), array_lens);
    check_stmts(&mut ctx, &[Stmt::Layout(block.clone())], false, diags);
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
fn check_device_generic_args(world: &World, inst: &crate::ast::InstStmt, diags: &mut Diagnostics) {
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
                diags.push(crate::check::generics::wrong_unit_argument(
                    param, u.unit, v.unit, &v.text, *span,
                ));
            }
        }
    }
}

/// RFC-032: the statically-knowable properties of a NESTED use site — target
/// kind, variant selector, generic arity, argument kinds and concrete
/// values/bounds, and connection port keys. A use site inside an unused
/// subdesign otherwise escapes all of this until a consumer instantiates
/// the enclosing one. Judgments come from THE bound checker (`resolve_one`,
/// DR-016) so messages mirror expansion's exactly and a used enclosing
/// subdesign reported by both collapses under dedup.
fn check_sub_use_site(
    world: &World,
    f: &FnDef,
    stmt: &crate::ast::SubdesignUseStmt,
    diags: &mut Diagnostics,
) {
    let name = &stmt.ty.name;
    let Some(sd) = world.subdesigns.get(&name.name) else {
        if let Some(sym) = world.symbols.get(&name.name) {
            diags.push(Diagnostic::error(
                "E205",
                name.span,
                format!(
                    "`{}` is a {} — a `subdesign` use site requires a subdesign",
                    name.name, sym.kind
                ),
            ));
        }
        return;
    };
    if let Some(sel) = &stmt.ty.variant {
        diags.push(Diagnostic::error(
            "E1303",
            sel.span,
            format!(
                "a subdesign has no variants — remove the `[{}]` selector",
                sel.name
            ),
        ));
    }
    let args = &stmt.ty.generic_args;
    if args.len() > sd.generics.len() {
        diags.push(Diagnostic::error(
            "E401",
            stmt.ty.span,
            format!(
                "subdesign `{}` takes {} generic argument{}, but {} {} given",
                short(&name.name),
                sd.generics.len(),
                if sd.generics.len() == 1 { "" } else { "s" },
                args.len(),
                if args.len() == 1 { "was" } else { "were" }
            ),
        ));
    }
    let enclosing: BTreeSet<&str> = f.generics.iter().map(|g| g.name.name.as_str()).collect();
    for (i, param) in sd.generics.iter().enumerate() {
        match args.get(i) {
            Some(arg) => {
                // Deferred to expansion: a name referencing an ENCLOSING
                // generic (its value exists only per use, RFC-006) or an
                // unknown name (check_named_generic_args' E202 above owns
                // that report). Everything else — unit literals, bare
                // numbers, concrete device/part names — is judged now with
                // an empty substitution, which for these argument shapes
                // behaves exactly as expansion's env does.
                let deferred = matches!(arg, GenericArg::Expr(_))
                    || matches!(arg, GenericArg::Name(id)
                    if enclosing.contains(id.name.as_str())
                        || !world.symbols.contains_key(&id.name));
                if !deferred {
                    let _ = crate::check::generics::resolve_one(
                        world,
                        param,
                        arg,
                        &crate::check::generics::Substitution::new(),
                        diags,
                    );
                }
            }
            None => {
                if param.default.is_none() {
                    diags.push(
                        Diagnostic::error(
                            "E401",
                            stmt.ty.span,
                            format!(
                                "missing generic argument for `{}` of subdesign `{}` (it has no default)",
                                param.name.name,
                                short(&name.name)
                            ),
                        )
                        .with_help(crate::check::generics::describe_param(param)),
                    );
                }
            }
        }
    }
    let ports: BTreeSet<&str> = sd.ports.iter().map(|p| p.name.name.as_str()).collect();
    let mut seen: BTreeMap<&str, crate::span::Span> = BTreeMap::new();
    for conn in &stmt.conns {
        if let Some(prev) = seen.insert(conn.port.name.as_str(), conn.span) {
            diags.push(
                Diagnostic::error(
                    "E1301",
                    conn.port.span,
                    format!("port `{}` is connected more than once", conn.port.name),
                )
                .with_secondary(prev, "first connected here".to_string()),
            );
            continue;
        }
        if !ports.contains(conn.port.name.as_str()) {
            diags.push(
                Diagnostic::error(
                    "E1301",
                    conn.port.span,
                    format!(
                        "subdesign `{}` (use site `{}`) has no port named `{}`",
                        short(&name.name),
                        stmt.name.name,
                        conn.port.name
                    ),
                )
                .with_help(format!(
                    "its ports are: {}",
                    ports.iter().cloned().collect::<Vec<_>>().join(", ")
                )),
            );
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

/// Call value-argument count and reference kinds, using the callee's parameter
/// types. A whole instance is valid for an instance parameter; only a `Pin`
/// parameter requires a pin reference. Bounds and selectors are checked at
/// expansion, as for net/nc references in this pass.
fn check_call_args(
    world: &World,
    _f: &FnDef,
    call: &crate::ast::CallStmt,
    bases: &BTreeMap<&str, Base>,
    diags: &mut Diagnostics,
) {
    let callee = world.fns.get(&call.callee.name);
    if let Some(callee) = callee {
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
    for (i, arg) in call.args.iter().enumerate() {
        match callee.and_then(|f| f.params.get(i)).map(|p| &p.ty) {
            Some(FnParamTy::Generic(_) | FnParamTy::ImplTrait(..)) => {
                if arg.pin.is_some() {
                    diags.push(Diagnostic::error(
                        "E503",
                        arg.span,
                        format!("expected an instance, found pin reference `{}`", arg),
                    ));
                } else {
                    match bases.get(arg.base.name.as_str()) {
                        Some(Base::Concrete(..) | Base::Abstract) => {}
                        Some(Base::Pin) => diags.push(Diagnostic::error(
                            "E503",
                            arg.span,
                            format!(
                                "expected an instance, but `{}` is a `Pin` parameter",
                                arg.base.name
                            ),
                        )),
                        None | Some(Base::Sub { .. }) => diags.push(Diagnostic::error(
                            "E202",
                            arg.base.span,
                            format!("unknown instance `{}` in this scope", arg.base.name),
                        )),
                    }
                }
            }
            _ => check_pin_ref(world, bases, arg, diags),
        }
    }
}

/// A named generic argument (turbofish) must resolve to a fn generic in
/// scope or a declared symbol (review R6-3).
fn check_named_generic_args(
    ctx: &StaticCtx,
    args: &[GenericArg],
    params: Option<&[crate::ast::GenericParam]>,
    diags: &mut Diagnostics,
) {
    for (i, arg) in args.iter().enumerate() {
        let expected = params
            .and_then(|p| p.get(i))
            .and_then(|param| match &param.bound {
                GenericBound::Int(_) => Some(Ty::Int),
                GenericBound::Unit(unit) if unit.unit == crate::units::UnitType::Length => {
                    Some(Ty::Length)
                }
                _ => None,
            });
        match arg {
            GenericArg::Name(id) => {
                if let Some(expected) = expected {
                    if ctx.names.contains_key(&id.name) {
                        static_type_check(
                            ctx,
                            &crate::ast::Expr::Name(id.clone()),
                            expected,
                            "a generic argument",
                            diags,
                        );
                    }
                }
                if !ctx.check_names
                    || ctx.generic_names.contains(&id.name)
                    || ctx
                        .names
                        .get(&id.name)
                        .is_some_and(|kind| !matches!(kind, NameKind::NonValue))
                    || ctx.world.symbols.contains_key(&id.name)
                {
                    continue;
                }
                let mut d = Diagnostic::error(
                    "E202",
                    id.span,
                    format!("cannot find `{}` in this scope", id.name),
                );
                if let Some(suggestion) = ctx.world.suggest(&id.name) {
                    d = d.with_help(format!("did you mean `{suggestion}`?"));
                }
                diags.push(d);
            }
            GenericArg::Expr(e) => {
                if let Some(expected) = expected {
                    static_type_check(ctx, e, expected, "a generic argument", diags);
                } else {
                    let ty = static_type_check_any(ctx, e, "a generic argument", diags);
                    if let Some(param) = params.and_then(|p| p.get(i)) {
                        if let GenericBound::Unit(unit) = &param.bound {
                            if ty == Ty::Length {
                                diags.push(crate::check::generics::wrong_length_expression(
                                    param, unit.unit, e,
                                ));
                            }
                        }
                    }
                }
            }
            GenericArg::Unit(value, span) if expected == Some(Ty::Int) => {
                static_type_check(
                    ctx,
                    &crate::ast::Expr::Length(value.clone(), *span),
                    Ty::Int,
                    "a generic argument",
                    diags,
                );
            }
            GenericArg::Number(n, span)
                if params
                    .and_then(|p| p.get(i))
                    .is_some_and(|p| matches!(p.bound, GenericBound::Int(_))) =>
            {
                let _ = crate::check::generics::checked_int(n, *span, diags);
            }
            _ => {}
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
        Some(Base::Sub { sub_short, ports }) => match &r.pin {
            None => {
                diags.push(Diagnostic::error(
                    "E1303",
                    r.span,
                    format!(
                        "`{}` is a subdesign — reference one of its ports (e.g. `{}.{}`)",
                        r.base.name,
                        r.base.name,
                        ports.iter().next().copied().unwrap_or("PORT")
                    ),
                ));
            }
            Some(pin) if !ports.contains(pin.name.as_str()) => {
                diags.push(
                    Diagnostic::error(
                        "E1301",
                        pin.span,
                        format!(
                            "subdesign `{}` (use site `{}`) has no port named `{}`",
                            sub_short, r.base.name, pin.name
                        ),
                    )
                    .with_help(format!(
                        "its ports are: {} — internal nets and instances are behind the port boundary; only `place` may reach in (RFC-032)",
                        ports.iter().copied().collect::<Vec<_>>().join(", ")
                    )),
                );
            }
            Some(_) => {}
        },
        Some(Base::Abstract) => {} // trait-role access — abstract, checked at call time
    }
}
