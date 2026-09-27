//! Design expansion: monomorphize and inline every fn call (RFC-006), resolve
//! every instance/net/nc, merge nets, and run the pin connection-obligation
//! exhaustiveness check (RFC-002) on the fully-assembled design.
//!
//! Body semantics are declarative: within one body, all `inst` statements are
//! processed first, then `net`/`nc`/calls in source order — so a `net` may
//! reference an instance declared later in the same body.

use crate::ast::*;
use crate::check::generics::{
    resolve_generic_args, resolve_generic_args_in, CallerEnv, GenericValue, Substitution,
};
use crate::diag::{Diagnostic, Diagnostics};
use crate::ir::{
    DesignIr, IrInstance, IrNet, LayoutDiffPair, LayoutIr, LayoutLengthMatch, LayoutNetClass,
};
use crate::resolve::World;
use crate::span::Span;
use crate::units::UnitValue;
use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};

pub fn expand_design(world: &World, design: &DesignDef, diags: &mut Diagnostics) -> DesignIr {
    let definition_failures = crate::check::eval::expression_failures(diags);
    let mut ex = Expander {
        world,
        definition_failures,
        diags,
        instances: BTreeMap::new(),
        net_decls: Vec::new(),
        nc_pins: Vec::new(),
        layout_raw: Vec::new(),
        board_outline: None,
        placements: Vec::new(),
        sub_nodes: BTreeMap::new(),
        active_subs: Vec::new(),
        abs_node_places: BTreeMap::new(),
        reported_place_conflicts: BTreeSet::new(),
        rel_places: Vec::new(),
        synth_net_conns: Vec::new(),
        phys_grounds: Vec::new(),
        phys_high_currents: Vec::new(),
        phys_impedances: Vec::new(),
        phys_bypasses: Vec::new(),
        phys_crystals: Vec::new(),
        phys_converters: Vec::new(),
        phys_bga: Vec::new(),
        active_calls: Vec::new(),
        call_counter: 0,
        anon_net_counter: 0,
        meter: crate::check::meter::Meter::new(crate::check::meter::metering_needed(world, design)),
    };
    let mut scope = Scope {
        design_name: design.name.name.clone(),
        path: design.name.name.clone(),
        is_design_body: true,
        place_ctx: PlaceCtx::Design,
        layout_owner: None,
        subst: Substitution::new(),
        bindings: BTreeMap::new(),
        local_insts: BTreeMap::new(),
        local_subs: BTreeMap::new(),
        arrays: BTreeMap::new(),
        declared_lens: BTreeMap::new(),
        failed_expressions: Vec::new(),
        unknown_arrays: BTreeSet::new(),
        consts: BTreeMap::new(),
        binders: BTreeMap::new(),
        caller_frames: Vec::new(),
        frame: Vec::new(),
    };
    ex.walk_body(&design.body, &mut scope);
    // RFC-033 §9: a tripped meter leaves no IR — assembly (and its
    // obligation checks) is skipped; the E1405 already names the site.
    if ex.meter.tripped() {
        return DesignIr {
            name: design.name.name.clone(),
            instances: Default::default(),
            subdesigns: Default::default(),
            nets: Vec::new(),
            nc_pins: Default::default(),
            layout: Default::default(),
        };
    }
    ex.assemble(design)
}

/// What a fn parameter name is bound to during expansion.
#[derive(Debug, Clone)]
enum Binding {
    /// A pin: (instance path, logical pin name).
    Pin((String, String)),
    /// An instance passed by value.
    Instance {
        path: String,
        device: String,
        /// The trait bounds it was passed under — pin access on this binding
        /// resolves through these traits' role maps only (RFC-003/007).
        via_traits: Vec<String>,
    },
}

/// What kind of body a `place`/`board_outline` statement sits in — a design's
/// own layout (absolute placements), a subdesign's internal layout (RFC-032
/// default placements, relative to the subdesign's origin), or a called fn
/// (no retained path; `place` rejected, unchanged from RFC-020).
#[derive(Clone, Copy, PartialEq, Eq)]
enum PlaceCtx {
    Design,
    Sub,
    Fn,
}

#[derive(Clone)]
struct Scope {
    design_name: String,
    path: String,
    is_design_body: bool,
    place_ctx: PlaceCtx,
    /// The coordinate frame a `place` statement's DEFAULT is recorded against:
    /// the enclosing subdesign node's retained path, or None at design level
    /// (where placements are absolute). This is deliberately NOT `path` — a
    /// loop frame appends `__for_{label}_{value}` to `path` for iteration
    /// identity, but a loop NEVER creates a new coordinate node: every
    /// iteration's placements stay relative to the same subdesign origin.
    /// Set when entering a subdesign body; inherited unchanged by `enter_frame`.
    layout_owner: Option<String>,
    subst: Substitution,
    bindings: BTreeMap<String, Binding>,
    /// local instance name → full path.
    local_insts: BTreeMap<String, String>,
    /// RFC-032: local subdesign use-site name → full node path. Node paths
    /// are retained hierarchy, never physical instances.
    local_subs: BTreeMap<String, String>,
    /// RFC-024: array-typed instance name → (declared length, decl span).
    /// An array's NAME is never itself in `local_insts` — only its elements
    /// (`NAME_0`…`NAME_{N-1}`), so a bare unindexed reference cannot resolve.
    arrays: BTreeMap<String, (i64, crate::span::Span)>,
    /// Evaluated declaration lengths, visible before instances are expanded.
    /// Separate from `arrays`, whose entries also signal materialized arrays.
    declared_lens: BTreeMap<String, i64>,
    /// RFC-033 §3: local const bindings; failed dependencies remain typed
    /// unknowns, never fabricated zero values.
    consts: BTreeMap<String, crate::check::eval::NameKind>,
    failed_expressions: Vec<Span>,
    unknown_arrays: BTreeSet<String>,
    /// RFC-033 §6: visible loop binders (name → current value). Empty until
    /// Task 8's frames; the field exists so `names()` has one shape.
    binders: BTreeMap<String, i64>,
    /// RFC-033 §6: active loop frames (label, value, binder) — innermost
    /// last. Empty until Task 8.
    frame: Vec<(String, i64, String)>,
    /// Diagnostic provenance across calls; never participates in lexical name lookup.
    caller_frames: Vec<(String, i64, String)>,
}

impl Scope {
    fn caller_env(&self) -> CallerEnv {
        CallerEnv {
            subst: self.subst.clone(),
            names: self.names(),
            array_lens: self.array_lens(),
        }
    }
    /// RFC-033: expression names and kinds. Pin/instance parameters are
    /// known names, but cannot be used as Int/Length values.
    fn names(&self) -> BTreeMap<String, crate::check::eval::NameKind> {
        let mut m = crate::check::generics::subst_names(&self.subst);
        for name in self.bindings.keys() {
            m.insert(name.clone(), crate::check::eval::NameKind::NonValue);
        }
        for (k, v) in &self.consts {
            m.insert(k.clone(), v.clone());
        }
        for (k, v) in &self.binders {
            m.insert(k.clone(), crate::check::eval::NameKind::Binder(*v));
        }
        m
    }
    /// RFC-033: visible array lengths (`.len`).
    fn array_lens(&self) -> BTreeMap<String, i64> {
        let mut lens = self.declared_lens.clone();
        lens.extend(self.arrays.iter().map(|(k, (n, _))| (k.clone(), *n)));
        lens
    }
}

/// One `net` declaration with resolved members, pre-merge.
struct NetDecl {
    /// Merge key: design-level names merge by name; fn-scoped and anonymous
    /// nets are unique per declaration.
    key: String,
    /// Candidate emitted name.
    display_name: String,
    is_design_level_name: bool,
    /// RFC-032: true for the net declaration a port-connection block
    /// synthesizes — it may join an existing named net but never prove that
    /// net exists (validated at assembly).
    synthesized: bool,
    annotation: Option<NetAnnotation>,
    members: Vec<(String, String)>,
    span: Span,
}

/// RFC-032: one expanded subdesign use site — a retained hierarchy node.
struct SubNode {
    /// The subdesign declaration's fq name (diagnostics).
    fq: String,
    /// The use-site statement's span (port-exhaustiveness diagnostics).
    use_span: Span,
    /// Declared ports: name → (obligation, declaration span).
    ports: BTreeMap<String, (Obligation, Span)>,
    /// The node's DIRECT children, snapshotted from its body's scope —
    /// the resolution table for placement reach-in paths.
    children_insts: BTreeMap<String, String>,
    children_subs: BTreeMap<String, String>,
    children_arrays: BTreeMap<String, (i64, Span)>,
}

/// The data half of a `place` statement (target resolved separately).
#[derive(Clone)]
struct PlaceData {
    at: (UnitValue, UnitValue),
    rotate: u16,
    side: PlacementSide,
    span: Span,
}

/// What a resolved placement path targets.
enum PlaceTarget {
    Inst(String),
    Node(String),
}

/// One placement recorded inside a subdesign's own `layout {}` block: a
/// DEFAULT, relative to `owner`'s origin, transformed onto the board only
/// when (and however) the owner itself ends up anchored.
struct RelPlace {
    owner: String,
    target: PlaceTarget,
    data: PlaceData,
}

struct Expander<'w, 'd> {
    world: &'w World,
    diags: &'d mut Diagnostics,
    /// Only failures established before expansion; activation errors never enter here.
    definition_failures: Vec<Span>,
    instances: BTreeMap<String, IrInstance>,
    net_decls: Vec<NetDecl>,
    nc_pins: Vec<((String, String), Span)>,
    /// RFC-013 layout constraints, collected with scope-resolved net names,
    /// validated against the final net set at assembly.
    layout_raw: Vec<RawLayout>,
    /// The board outline (pragmatic extension; see `ast::BoardOutline`).
    /// Collected once, at the design top level — a `board_outline` inside a
    /// called fn is rejected (E1006). Geometry is validated on collection.
    board_outline: Option<crate::ir::BoardOutlineIr>,
    /// Locked component placements (`place <inst> at (x, y)`), design-level
    /// only, resolved to IR paths and validated on collection (E1007).
    placements: Vec<crate::ir::LayoutPlacement>,
    /// RFC-032: every expanded subdesign use site, keyed by node path.
    sub_nodes: BTreeMap<String, SubNode>,
    /// Subdesign fq names currently being expanded (containment-cycle guard,
    /// mirroring `active_calls`; the full-cycle diagnostic is declaration-time).
    active_subs: Vec<String>,
    /// Design-level whole-unit placements of subdesign nodes (absolute).
    abs_node_places: BTreeMap<String, PlaceData>,
    /// Re-review (B): duplicate-placement conflicts already reported, keyed
    /// by the coordinate OWNER plus the resolved target — a 5000-iteration
    /// loop placing one target reports the first conflict once, while two
    /// independent targets or different owners still report each.
    reported_place_conflicts: BTreeSet<(String, String)>,
    /// Placements declared inside subdesign bodies (defaults, owner-relative).
    rel_places: Vec<RelPlace>,
    /// RFC-032 port connections written as bare net names, validated against
    /// the real declared net set at assembly: (net key, source ident).
    synth_net_conns: Vec<(String, Ident)>,
    /// RFC-027 net-target physics attributes, collected per net declaration
    /// with spans (duplicate/one-primary validation happens at assembly).
    phys_grounds: Vec<(crate::ir::QuilterGround, Span)>,
    phys_high_currents: Vec<(crate::ir::QuilterHighCurrent, Span)>,
    phys_impedances: Vec<(crate::ir::QuilterImpedance, Span)>,
    /// RFC-027 inst-target physics attributes, resolved after pass 1.
    phys_bypasses: Vec<crate::ir::QuilterBypass>,
    phys_crystals: Vec<crate::ir::QuilterCrystal>,
    phys_converters: Vec<crate::ir::QuilterConverter>,
    phys_bga: Vec<String>,
    /// fn names currently being expanded (cycle detection, RFC-006).
    active_calls: Vec<String>,
    /// RFC-033 §9: the expansion budget ledger. Inactive (legacy graphs)
    /// charges are always free.
    meter: crate::check::meter::Meter,
    /// Global (per-design) call counter — `__fn{N}_{name}` segments.
    call_counter: usize,
    anon_net_counter: usize,
}

/// A layout constraint with each net reference resolved to its candidate IR net
/// name (paired with the original identifier for precise E1001 spans).
enum RawLayout {
    NetClass {
        /// The source identifier (span for E1002) and its scope-resolved
        /// identity — fn-local classes get call-chain-scoped names exactly
        /// like fn-local nets (RFC-006), so a layout-bearing fn can be called
        /// more than once without colliding with itself.
        name: Ident,
        scoped_name: String,
        nets: Vec<(String, Ident)>,
    },
    DiffPair {
        nets: Vec<(String, Ident)>,
        differential_impedance: Option<UnitValue>,
        single_ended_impedance: Option<UnitValue>,
        frequency: Option<UnitValue>,
        span: Span,
    },
    LengthMatch {
        nets: Vec<(String, Ident)>,
        tolerance: Option<String>,
        span: Span,
    },
}

/// RFC-024: an array element's internal instance name. The RFC defines an
/// array as behaving exactly as if the author had hand-written
/// `NAME_0: Device`, `NAME_1: Device`, … — so that is literally the name used.
fn element_name(base: &str, i: i64) -> String {
    format!("{}_{}", base, i)
}

impl<'w, 'd> Expander<'w, 'd> {
    fn walk_body(&mut self, body: &[Stmt], scope: &mut Scope) {
        // Resolve the same lexical dependency graph used by definition and
        // layout validation. Only positive validated lengths can materialize.
        let locals: Vec<_> = body
            .iter()
            .filter_map(|stmt| {
                let (name, value, ty) = match stmt {
                    Stmt::Const(c) => (
                        &c.name,
                        &c.value,
                        Some(match c.ty {
                            ConstTy::Int => crate::check::eval::Ty::Int,
                            ConstTy::Length => crate::check::eval::Ty::Length,
                        }),
                    ),
                    Stmt::Inst(i) => (&i.name, &i.array_len.as_ref()?.0, None),
                    Stmt::SubdesignUse(u) => (&u.name, &u.array_len.as_ref()?.0, None),
                    _ => return None,
                };
                Some(crate::check::eval::LocalExpr {
                    name: name.clone(),
                    value: value.clone(),
                    ty,
                    span: match stmt {
                        Stmt::Const(c) => c.span,
                        _ => value.span(),
                    },
                })
            })
            .collect();
        self.bind_locals(&locals, scope);

        // Pass 1: instances AND subdesign use sites (declarative bodies —
        // nets may reference later insts and later use sites' ports).
        for stmt in body {
            if self.meter.tripped() {
                return;
            }
            if let Stmt::SubdesignUse(sub) = stmt {
                self.handle_subdesign_use(sub, scope);
            }
            if let Stmt::Inst(inst) = stmt {
                match &inst.array_len {
                    None => self.handle_inst(inst, scope),
                    // RFC-024: `inst NAME: [Device; N]` is ONE array-typed
                    // instance whose N elements are each fully real. Each
                    // element goes through the SAME `handle_inst` a hand-
                    // written `inst` does, so designator allocation (RFC-005),
                    // pin obligations (RFC-002) and trait satisfaction
                    // (RFC-003) apply to it completely unchanged.
                    Some((len_expr, span)) => {
                        // RFC-033 pass 0 already evaluated the length
                        // dependency-ordered (consts, .len, cycles); read
                        // the memoized result here.
                        let _ = span;
                        let Some(n) = scope.declared_lens.get(&inst.name.name).copied() else {
                            continue; // already diagnosed in pass 0
                        };
                        if scope.arrays.contains_key(&inst.name.name)
                            || scope.local_insts.contains_key(&inst.name.name)
                            || scope.local_subs.contains_key(&inst.name.name)
                            || scope.bindings.contains_key(&inst.name.name)
                        {
                            self.diags.push(Diagnostic::error(
                                "E201",
                                inst.name.span,
                                format!("`{}` is already defined in this scope", inst.name.name),
                            ));
                            continue;
                        }
                        scope
                            .arrays
                            .insert(inst.name.name.clone(), (n, len_expr.span()));
                        if !self.meter.ensure_capacity(
                            n as u64,
                            "`inst` array",
                            inst.span,
                            self.diags,
                        ) {
                            return;
                        }
                        for i in 0..n {
                            if self.meter.tripped() {
                                return;
                            }
                            let mut elem = inst.clone();
                            elem.name = Ident {
                                name: element_name(&inst.name.name, i),
                                span: inst.name.span,
                            };
                            elem.array_len = None;
                            self.handle_inst(&elem, scope);
                        }
                    }
                }
            }
        }
        // Pass 1.5 (RFC-027): inst-target physics attributes, resolved only
        // after EVERY instance in this body exists — a bypass may reference an
        // instance declared later in source.
        for stmt in body {
            if self.meter.tripped() {
                return;
            }
            if let Stmt::Inst(inst) = stmt {
                if !inst.phys.is_empty() {
                    self.handle_inst_phys(inst, scope);
                }
            }
        }
        // Pass 2: everything else, in source order. Port-connection blocks
        // resolve here (their pin references may name instances declared
        // anywhere in this body).
        for stmt in body {
            if self.meter.tripped() {
                return;
            }
            match stmt {
                Stmt::Inst(_) => {}
                Stmt::Net(net) => self.handle_net(net, scope),
                Stmt::Nc(nc) => self.handle_nc(nc, scope),
                Stmt::Call(call) => self.handle_call(call, scope),
                Stmt::Layout(block) => self.handle_layout(block, scope),
                Stmt::SubdesignUse(sub) => self.handle_subdesign_conns(sub, scope),
                // RFC-033: const/loop expansion lands with Tasks 7/8; the
                // statements cannot parse until Task 3, so no behavior to
                // preserve yet — the arms exist so the match stays total.
                // RFC-033: loop frames expand here (pass 2); consts inside a
                // loop body are collected by that frame's own walk_body.
                Stmt::Const(_) => {}
                Stmt::For(f) => self.handle_for(f, scope),
            }
        }
    }

    // -- layout constraints (RFC-013) ----------------------------------------

    /// Collect a `layout {}` block's constraints, resolving each net reference
    /// to its candidate IR net name within the current scope. Validation
    /// against the final net set happens in `assemble` (net existence is only
    /// knowable once every declaration in the design is processed).
    /// RFC-033 §5-§7: a labelled `for` loop expands as one FRAME per
    /// iteration — hygienic (each frame's path names the label and value),
    /// re-entrant through `walk_body` for nested loops.
    fn handle_for(&mut self, f: &ForStmt, scope: &mut Scope) {
        // Static admission first: a loop body may not declare.
        for s in &f.body {
            match s {
                Stmt::Inst(i) => self.diags.push(Diagnostic::error(
                    "E1406",
                    i.span,
                    "`inst` is not admitted inside a `for` body — declare the array outside the loop and repeat only connections, calls and placements here (RFC-033 Candidate A)".to_string(),
                )),
                Stmt::SubdesignUse(u) => {
                    if scope.place_ctx == PlaceCtx::Fn {
                        // E1307 wins (the fn-body restriction is stricter and
                        // pre-existing).
                        self.diags.push(Diagnostic::error(
                            "E1307",
                            u.span,
                            "a `subdesign` use site needs a retained hierarchy path — a `fn` expands inline and cannot contain one (RFC-032); move it into the design or a subdesign".to_string(),
                        ));
                    } else {
                        self.diags.push(Diagnostic::error(
                            "E1406",
                            u.span,
                            "a `subdesign` use site is not admitted inside a `for` body — declare it outside the loop".to_string(),
                        ));
                    }
                }
                _ => {}
            }
        }
        if !self.check_not_reserved(&f.label, "loop label")
            || !self.check_not_reserved(&f.binder, "loop variable")
        {
            return;
        }
        // Label/binder collisions with anything visible.
        for id in [&f.label, &f.binder] {
            if scope.local_insts.contains_key(&id.name)
                || scope.local_subs.contains_key(&id.name)
                || scope.arrays.contains_key(&id.name)
                || scope.bindings.contains_key(&id.name)
                || scope.consts.contains_key(&id.name)
                || scope.binders.contains_key(&id.name)
                || scope.frame.iter().any(|(l, _, _)| l == &id.name)
            {
                self.diags.push(Diagnostic::error(
                    "E201",
                    id.span,
                    format!("`{}` is already defined in this scope", id.name),
                ));
                return;
            }
        }
        let Some(lo) = self.eval_int(&f.start, scope, "a loop bound") else {
            return;
        };
        let Some(hi) = self.eval_int(&f.end, scope, "a loop bound") else {
            return;
        };
        // RFC-033 §8: static validation of the loop body ONCE per loop entry,
        // under the bound substitution (before the range decision — even an
        // empty or skipped loop hides nothing decidable).
        {
            let mut names = scope.names();
            names.insert(
                f.binder.name.clone(),
                crate::check::eval::NameKind::Unknown(crate::check::eval::Ty::Int),
            );
            let mut bases: std::collections::BTreeSet<String> =
                scope.bindings.keys().cloned().collect();
            for k in scope
                .local_insts
                .keys()
                .chain(scope.local_subs.keys())
                .chain(scope.arrays.keys())
            {
                bases.insert(k.clone());
            }
            let mut local_diags = Diagnostics::new();
            crate::check::bodies::check_loop_body_bound(
                self.world,
                &f.body,
                &names,
                &bases,
                &scope.array_lens(),
                &mut local_diags,
            );
            self.record_validation(local_diags, scope);
        }
        if lo > hi {
            self.diags.push(Diagnostic::error(
                "E1404",
                f.start.span().to(f.end.span()),
                format!(
                    "loop `{}` has a reversed range {}..{} — the end must not be below the start (a half-open range with equal bounds is empty){}",
                    f.label.name,
                    lo,
                    hi,
                    self.frame_suffix(scope)
                ),
            ));
            return;
        }
        // Frame depth is part of the budget (Task 10); iterations charged there too.
        if !self.meter.enter_frame(f.span, self.diags) {
            return;
        }
        for v in lo..hi {
            if !self.meter.enter_iteration(f.span, self.diags) {
                break;
            }
            let mut inner = self.enter_frame(&f.label, &f.binder, v, scope);
            let saved = (self.anon_net_counter, self.call_counter);
            self.anon_net_counter = 0;
            self.call_counter = 0;
            self.walk_body(&f.body, &mut inner);
            self.anon_net_counter = saved.0;
            self.call_counter = saved.1;
        }
        self.meter.leave_frame();
    }

    /// RFC-033 §6: one iteration's frame — path gains `__for_{label}_{value}`
    /// (negative values as `neg{abs}`), the binder becomes visible, and the
    /// parent's declarations ride as cloned read-only views.
    fn enter_frame(&mut self, label: &Ident, binder: &Ident, value: i64, scope: &Scope) -> Scope {
        let mut inner = Scope {
            design_name: scope.design_name.clone(),
            path: format!(
                "{}::__for_{}_{}",
                scope.path,
                label.name,
                frame_value_text(value)
            ),
            is_design_body: false,
            place_ctx: scope.place_ctx,
            layout_owner: scope.layout_owner.clone(),
            subst: scope.subst.clone(),
            bindings: scope.bindings.clone(),
            local_insts: scope.local_insts.clone(),
            local_subs: scope.local_subs.clone(),
            arrays: scope.arrays.clone(),
            declared_lens: scope.declared_lens.clone(),
            consts: scope.consts.clone(),
            failed_expressions: scope.failed_expressions.clone(),
            unknown_arrays: scope.unknown_arrays.clone(),
            binders: scope.binders.clone(),
            caller_frames: scope.caller_frames.clone(),
            frame: scope.frame.clone(),
        };
        inner.binders.insert(binder.name.clone(), value);
        inner
            .frame
            .push((label.name.clone(), value, binder.name.clone()));
        inner
    }

    /// RFC-033 §6 layout loops: consts → placements → nested loops, one frame
    /// per iteration (E1404 for reversed ranges, E1406 handled at parse).
    fn handle_layout_for(&mut self, f: &LayoutFor, scope: &mut Scope) {
        if !self.check_not_reserved(&f.label, "loop label")
            || !self.check_not_reserved(&f.binder, "loop variable")
        {
            return;
        }
        for id in [&f.label, &f.binder] {
            if scope.local_insts.contains_key(&id.name)
                || scope.local_subs.contains_key(&id.name)
                || scope.arrays.contains_key(&id.name)
                || scope.bindings.contains_key(&id.name)
                || scope.consts.contains_key(&id.name)
                || scope.binders.contains_key(&id.name)
            {
                self.diags.push(Diagnostic::error(
                    "E201",
                    id.span,
                    format!("`{}` is already defined in this scope", id.name),
                ));
                return;
            }
        }
        let Some(lo) = self.eval_int(&f.start, scope, "a loop bound") else {
            return;
        };
        let Some(hi) = self.eval_int(&f.end, scope, "a loop bound") else {
            return;
        };
        if lo > hi {
            self.diags.push(Diagnostic::error(
                "E1404",
                f.start.span().to(f.end.span()),
                format!(
                    "loop `{}` has a reversed range {}..{} — the end must not be below the start (a half-open range with equal bounds is empty){}",
                    f.label.name,
                    lo,
                    hi,
                    self.frame_suffix(scope)
                ),
            ));
            return;
        }
        if !self.meter.enter_frame(f.span, self.diags) {
            return;
        }
        for v in lo..hi {
            if !self.meter.enter_iteration(f.span, self.diags) {
                break;
            }
            let mut inner = self.enter_frame(&f.label, &f.binder, v, scope);
            self.bind_layout_constants(&f.consts, &mut inner);
            for p in &f.placements {
                if self.meter.tripped() {
                    break;
                }
                self.handle_placement(p, &inner);
            }
            for nested in &f.loops {
                if self.meter.tripped() {
                    break;
                }
                let mut inner_mut = inner.clone();
                self.handle_layout_for(nested, &mut inner_mut);
            }
        }
        self.meter.leave_frame();
    }

    fn bind_layout_constants(&mut self, consts: &[ConstStmt], scope: &mut Scope) {
        let locals: Vec<_> = consts
            .iter()
            .map(|c| crate::check::eval::LocalExpr {
                name: c.name.clone(),
                value: c.value.clone(),
                span: c.span,
                ty: Some(match c.ty {
                    ConstTy::Int => crate::check::eval::Ty::Int,
                    ConstTy::Length => crate::check::eval::Ty::Length,
                }),
            })
            .collect();
        self.bind_locals(&locals, scope);
    }

    fn bind_locals(&mut self, locals: &[crate::check::eval::LocalExpr], scope: &mut Scope) {
        let mut names = scope.names();
        let mut lens = scope.array_lens();
        let mut unknown = scope.unknown_arrays.clone();
        let mut diags = Diagnostics::new();
        let failed: Vec<_> = self
            .definition_failures
            .iter()
            .chain(&scope.failed_expressions)
            .copied()
            .collect();
        crate::check::eval::resolve_locals(
            locals,
            &mut names,
            &mut lens,
            &mut unknown,
            &failed,
            &mut diags,
        );
        self.record_validation(diags, scope);
        for local in locals {
            if local.ty.is_some() {
                if let Some(value) = names.get(&local.name.name) {
                    scope.consts.insert(local.name.name.clone(), value.clone());
                }
            }
        }
        scope.declared_lens = lens;
        scope.unknown_arrays = unknown;
    }

    fn expression_failed(&self, span: Span, scope: &Scope) -> bool {
        crate::check::eval::expression_failed(span, &self.definition_failures)
            || crate::check::eval::expression_failed(span, &scope.failed_expressions)
    }

    /// Validation happens before iteration. Remember only this activation's
    /// newly failed sites; child frames inherit them, sibling calls do not.
    fn record_validation(&mut self, local: Diagnostics, scope: &mut Scope) {
        let failed = crate::check::eval::expression_failures(&local);
        self.push_with_suffix(local, scope);
        scope.failed_expressions.extend(failed);
    }

    fn handle_layout(&mut self, block: &LayoutBlock, scope: &Scope) {
        let mut validated_scope = scope.clone();
        let mut local_diags = Diagnostics::new();
        crate::check::bodies::check_layout_bound(
            self.world,
            block,
            &scope.names(),
            &scope.array_lens(),
            &mut local_diags,
        );
        self.record_validation(local_diags, &mut validated_scope);
        let scope = &validated_scope;
        let resolve = |nets: &[Ident]| -> Vec<(String, Ident)> {
            nets.iter()
                .map(|nid| (resolve_net_name(&nid.name, scope), nid.clone()))
                .collect()
        };
        for c in &block.constraints {
            let nets = match c {
                LayoutConstraint::NetClass { nets, .. }
                | LayoutConstraint::DiffPair { nets, .. }
                | LayoutConstraint::LengthMatch { nets, .. } => nets,
            };
            if !self.meter.charge(
                1 + nets.len() as u64,
                "layout constraint",
                c.span(),
                self.diags,
            ) {
                return;
            }
            if matches!(c, LayoutConstraint::DiffPair { differential_impedance, single_ended_impedance, frequency, .. }
                if differential_impedance.is_some() || single_ended_impedance.is_some() || frequency.is_some())
                && !self
                    .meter
                    .charge(1, "diff-pair physics bracket", c.span(), self.diags)
            {
                return;
            }
            let raw = match c {
                LayoutConstraint::NetClass { name, nets, .. } => RawLayout::NetClass {
                    name: name.clone(),
                    // Class identity is scoped like net identity: raw at
                    // design level, `__fnN_name::CLASS` inside a fn call.
                    scoped_name: resolve_net_name(&name.name, scope),
                    nets: resolve(nets),
                },
                LayoutConstraint::DiffPair {
                    nets,
                    differential_impedance,
                    single_ended_impedance,
                    frequency,
                    span,
                } => RawLayout::DiffPair {
                    nets: resolve(nets),
                    differential_impedance: differential_impedance.clone(),
                    single_ended_impedance: single_ended_impedance.clone(),
                    frequency: frequency.clone(),
                    span: *span,
                },
                LayoutConstraint::LengthMatch {
                    nets,
                    tolerance,
                    span,
                } => RawLayout::LengthMatch {
                    nets: resolve(nets),
                    tolerance: tolerance.as_ref().map(|(s, _)| s.clone()),
                    span: *span,
                },
            };
            self.layout_raw.push(raw);
        }
        if let Some(outline) = &block.board_outline {
            self.handle_board_outline(outline, scope);
        }
        // RFC-033: layout consts are visible only in this layout and its
        // loops — evaluate into a local scope clone.
        let mut layout_scope = scope.clone();
        self.bind_layout_constants(&block.consts, &mut layout_scope);
        for placement in &block.placements {
            if self.meter.tripped() {
                return;
            }
            self.handle_placement(placement, &layout_scope);
        }
        // RFC-033: labelled placement loops.
        for lf in &block.loops {
            if self.meter.tripped() {
                return;
            }
            let mut s = layout_scope.clone();
            self.handle_layout_for(lf, &mut s);
        }
    }

    /// RFC-033 §6: the frame suffix appended to diagnostics raised inside a
    /// loop frame (" — in <path>, <binder> = <value>"); empty at top level.
    fn frame_suffix(&self, scope: &Scope) -> String {
        if scope.frame.is_empty() && scope.place_ctx == PlaceCtx::Design {
            return String::new();
        }
        let mut suffix = format!(" — in {}", scope.path);
        for (_, value, binder) in scope.caller_frames.iter().chain(&scope.frame) {
            suffix.push_str(&format!(", {binder} = {value}"));
        }
        for (name, value) in &scope.subst {
            match value {
                GenericValue::Int(n) => suffix.push_str(&format!(", {name} = {n}")),
                GenericValue::Unit(v) if v.unit == crate::units::UnitType::Length => {
                    suffix.push_str(&format!(", {name} = {}", v.text));
                }
                _ => {}
            }
        }
        suffix
    }

    /// Re-push diagnostics from a local batch with the frame suffix appended
    /// to each main message.
    fn push_with_suffix(&mut self, local: Diagnostics, scope: &Scope) {
        for mut d in local.drain_batch() {
            if self.expression_failed(d.primary.span, scope) {
                continue;
            }
            d.message.push_str(&self.frame_suffix(scope));
            self.diags.push(d);
        }
    }

    fn bound_generic_args(
        &mut self,
        owner: &str,
        params: &[GenericParam],
        args: &[GenericArg],
        scope: &Scope,
        site: Span,
    ) -> Substitution {
        let mut local = Diagnostics::new();
        let subst = resolve_generic_args_in(
            self.world,
            owner,
            params,
            args,
            &scope.caller_env(),
            site,
            &mut local,
        );
        for d in local.drain_batch() {
            if matches!(d.code, "E1401" | "E1402" | "E1403") {
                let mut batch = Diagnostics::new();
                batch.push(d);
                self.push_with_suffix(batch, scope);
            } else {
                // In particular, definition and activation E112 must retain
                // the identical shape established by wrong_unit_argument.
                self.diags.push(d);
            }
        }
        subst
    }

    /// RFC-033: evaluate an Int-valued expression against the scope's
    /// visible names/arrays. A Length result is E1401.
    fn eval_int(&mut self, e: &Expr, scope: &Scope, what: &str) -> Option<i64> {
        if self.expression_failed(e.span(), scope) {
            return None;
        }
        let names = scope.names();
        let lens = scope.array_lens();
        let env = crate::check::eval::Env {
            names: &names,
            array_lens: &lens,
            unknown_arrays: &scope.unknown_arrays,
        };
        let mut local = Diagnostics::new();
        let v = crate::check::eval::eval(e, &env, &mut local);
        self.push_with_suffix(local, scope);
        match v {
            Some(crate::check::eval::Value::Int(i)) => Some(i),
            Some(crate::check::eval::Value::Length(_)) => {
                self.diags.push(Diagnostic::error(
                    "E1401",
                    e.span(),
                    format!(
                        "{} must be an Int, but `{}` is a Length{}",
                        what,
                        crate::ast::expr_text(e),
                        self.frame_suffix(scope)
                    ),
                ));
                None
            }
            None => None,
        }
    }

    /// RFC-033: evaluate a Length-valued expression. A literal keeps its own
    /// spelling; a computed value gets the canonical text. An Int result is
    /// E1401.
    fn eval_length(&mut self, e: &Expr, scope: &Scope, what: &str) -> Option<UnitValue> {
        // Legacy literal coordinates keep their position-specific E1007
        // unit check; definition validation may also reject their Expr kind.
        if let Some(v) = e.as_length_literal() {
            return Some(v.clone());
        }
        if self.expression_failed(e.span(), scope) {
            return None;
        }
        let names = scope.names();
        let lens = scope.array_lens();
        let env = crate::check::eval::Env {
            names: &names,
            array_lens: &lens,
            unknown_arrays: &scope.unknown_arrays,
        };
        let mut local = Diagnostics::new();
        let v = crate::check::eval::eval(e, &env, &mut local);
        self.push_with_suffix(local, scope);
        match v {
            Some(crate::check::eval::Value::Length(v)) => Some(v),
            Some(crate::check::eval::Value::Int(_)) => {
                self.diags.push(Diagnostic::error(
                    "E1401",
                    e.span(),
                    format!(
                        "{} is a `Length` (`mm`) value — `{}` is an Int{}",
                        what,
                        crate::ast::expr_text(e),
                        self.frame_suffix(scope)
                    ),
                ));
                None
            }
            None => None,
        }
    }

    fn indexed_local(
        &mut self,
        id: &Ident,
        index: Option<&(Expr, Span)>,
        scope: &Scope,
        unindexed_help: &str,
    ) -> Option<String> {
        // RFC-033: the index evaluates through the scope (consts, generics,
        // and — from Task 8 — loop binders); a Length result is E1401.
        let index: Option<(i64, Span)> = match index {
            None => None,
            Some((e, sp)) => Some((self.eval_int(e, scope, "an index")?, *sp)),
        };
        match (index, scope.arrays.get(&id.name).copied()) {
            (None, None) => Some(id.name.clone()),
            (None, Some(_)) => {
                self.diags.push(Diagnostic::error(
                    "E211",
                    id.span,
                    unindexed_help.to_string(),
                ));
                None
            }
            (Some((_, sp)), None) => {
                self.diags.push(Diagnostic::error(
                    "E211",
                    sp,
                    format!(
                        "`{}` is not an array-typed instance — only `inst NAME: [Device; N]` can be indexed",
                        id.name
                    ),
                ));
                None
            }
            (Some((i, sp)), Some((n, _))) => {
                if i < 0 || i >= n {
                    self.diags.push(Diagnostic::error(
                        "E202",
                        sp,
                        format!(
                            "index {} is out of bounds for `{}` — valid indices are 0..={} (length {}){}",
                            i,
                            id.name,
                            n - 1,
                            n,
                            self.frame_suffix(scope)
                        ),
                    ));
                    return None;
                }
                Some(element_name(&id.name, i))
            }
        }
    }

    /// Validate and record a locked placement (E1007/RFC-032). In the
    /// design's own layout: absolute, targeting a local instance, a
    /// subdesign node (whole-unit), or — through a dotted path — one real
    /// instance inside a subdesign (the placement reach-in). In a
    /// subdesign's internal layout: the same forms, recorded as DEFAULTS
    /// relative to that subdesign's origin. Inside a called fn: rejected,
    /// unchanged from RFC-020.
    fn handle_placement(&mut self, placement: &crate::ast::Placement, scope: &Scope) {
        // RFC-033 §9: one work item per placement, before any resolution.
        if !self
            .meter
            .charge(1, "the placement", placement.span, self.diags)
        {
            return;
        }
        use crate::units::UnitType;
        if scope.place_ctx == PlaceCtx::Fn || !self.active_calls.is_empty() {
            self.diags.push(Diagnostic::error(
                "E1007",
                placement.span,
                "`place` is only valid in a design's or subdesign's own `layout {}` block, not inside a called `fn`".to_string(),
            ));
            return;
        }
        // RFC-024: `place NAME[i]` targets one real array element — the same
        // reference form valid in every other instance position.
        let first = &placement.path[0];
        let Some(local) = self.indexed_local(
            &first.name,
            first.index.as_ref(),
            scope,
            &format!(
                "`{}` is array-typed — place one element, e.g. `place {}[0] at (…)`",
                first.name.name, first.name.name
            ),
        ) else {
            return;
        };
        let mut cur = if let Some(p) = scope.local_insts.get(&local) {
            PlaceTarget::Inst(p.clone())
        } else if let Some(p) = scope.local_subs.get(&local) {
            PlaceTarget::Node(p.clone())
        } else {
            self.diags.push(Diagnostic::error(
                "E1007",
                first.name.span,
                format!(
                    "`place` names `{}`, which is not an instance or subdesign in this scope",
                    first.name.name
                ),
            ));
            return;
        };
        // RFC-032: walk the remaining segments through subdesign nodes. The
        // FIRST segment that fails to resolve is named exactly.
        for seg in &placement.path[1..] {
            let node_path = match cur {
                PlaceTarget::Node(p) => p,
                PlaceTarget::Inst(p) => {
                    self.diags.push(Diagnostic::error(
                        "E1305",
                        seg.name.span,
                        format!(
                            "`{}` is an instance — an instance has no internals for a placement path to reach",
                            crate::resolve::short(&p)
                        ),
                    ));
                    return;
                }
            };
            let (arrays_entry, node_fq) = {
                let node = &self.sub_nodes[&node_path];
                (
                    node.children_arrays.get(&seg.name.name).copied(),
                    node.fq.clone(),
                )
            };
            let child = match (&seg.index, arrays_entry) {
                (None, None) => seg.name.name.clone(),
                (None, Some(_)) => {
                    self.diags.push(Diagnostic::error(
                        "E211",
                        seg.name.span,
                        format!(
                            "`{}` is array-typed — place one element, e.g. `{}[0]`",
                            seg.name.name, seg.name.name
                        ),
                    ));
                    return;
                }
                (Some((_, sp)), None) => {
                    self.diags.push(Diagnostic::error(
                        "E211",
                        *sp,
                        format!(
                            "`{}` is not an array-typed instance — only `inst NAME: [Device; N]` can be indexed",
                            seg.name.name
                        ),
                    ));
                    return;
                }
                (Some((e, sp)), Some((n, _))) => {
                    // RFC-033: the path-segment index evaluates in this scope.
                    let Some(i) = self.eval_int(e, scope, "a placement index") else {
                        return;
                    };
                    let _ = sp;
                    if i < 0 || i >= n {
                        self.diags.push(Diagnostic::error(
                            "E202",
                            *sp,
                            format!(
                                "index {} is out of bounds for `{}` — valid indices are 0..={} (length {}){}",
                                i, seg.name.name, n - 1, n,
                                self.frame_suffix(scope)
                            ),
                        ));
                        return;
                    }
                    element_name(&seg.name.name, i)
                }
            };
            let (child_inst, child_sub) = {
                let node = &self.sub_nodes[&node_path];
                (
                    node.children_insts.get(&child).cloned(),
                    node.children_subs.get(&child).cloned(),
                )
            };
            cur = if let Some(p) = child_inst {
                PlaceTarget::Inst(p)
            } else if let Some(p) = child_sub {
                PlaceTarget::Node(p)
            } else {
                self.diags.push(
                    Diagnostic::error(
                        "E1305",
                        seg.name.span,
                        format!(
                            "`{}` is not an instance or subdesign inside `{}` (subdesign `{}`)",
                            seg.name.name,
                            crate::resolve::short(&node_path),
                            crate::resolve::short(&node_fq)
                        ),
                    )
                    .with_help(
                        "placement reaches real internal instances and nested subdesigns only; a `fn`-expanded instance retains no stable path (RFC-032)"
                            .to_string(),
                    ),
                );
                return;
            };
        }
        let resolved_target = match &cur {
            PlaceTarget::Inst(p) | PlaceTarget::Node(p) => p.clone(),
        };
        // RFC-033: coordinates evaluate through `eval_length` (a literal keeps
        // its spelling; a computed value gets canonical text), then the
        // existing unit-type and geometry-range checks run unchanged.
        let at: (UnitValue, UnitValue) = {
            let Some(x) = self.eval_length(&placement.at.0, scope, "placement x") else {
                return;
            };
            let Some(y) = self.eval_length(&placement.at.1, scope, "placement y") else {
                return;
            };
            for (v, what) in [(&x, "x"), (&y, "y")] {
                if v.unit != UnitType::Length {
                    self.diags.push(Diagnostic::error(
                        "E1007",
                        placement.span,
                        format!(
                            "placement {} is a `Length` (`mm`) literal — `{}` is a `{}`",
                            what,
                            v.text,
                            v.unit.type_name()
                        ),
                    ));
                    return;
                }
                if !v.length_in_geom_range() {
                    self.diags.push(Diagnostic::error(
                        "E1007",
                        placement.span,
                        format!(
                            "placement {} `{}` is too large to project (review R5-5)",
                            what, v.text
                        ),
                    ));
                    return;
                }
            }
            (x, y)
        };
        // Rotation is any whole degree in 0..=359 (deviation from RFC-020's
        // closed {0, 90, 180, 270}, at the board author's direction — ledgered
        // in docs/compliance-report.md). A full turn is 0, so 360 and beyond is
        // rejected rather than silently reduced: `rotate 450` is far more likely
        // a mistake than a deliberate 90.
        let rotate: u16 = match &placement.rotate {
            None => 0,
            Some(e) => {
                let Some(n) = self.eval_int(e, scope, "a rotation") else {
                    return;
                };
                if !(0..=359).contains(&n) {
                    let mut diagnostic = Diagnostic::error(
                        "E1007", e.span(), format!(
                            "`rotate {}` is not a rotation — give a whole number of degrees in 0..=359 (counter-clockwise)", n
                        ),
                    );
                    if !scope.frame.is_empty() {
                        let context = format!(
                            "target `{resolved_target}`, computed angle {n}{}",
                            self.frame_suffix(scope)
                        );
                        // Message-only consumers must retain activation provenance.
                        diagnostic.message.push_str(&format!("; {context}"));
                        diagnostic = diagnostic.with_primary_label(context);
                    }
                    self.diags.push(diagnostic);
                    return;
                }
                n as u16
            }
        };
        let data = PlaceData {
            at: (at.0.clone(), at.1.clone()),
            rotate,
            side: placement.side,
            span: placement.span,
        };
        // The duplicate-placement conflict key: the actual coordinate owner
        // (subdesign node for defaults, the design for absolute placements)
        // plus the RESOLVED target — never the source spelling or the error
        // code/message, so distinct targets/owners can never be swallowed.
        let conflict_owner = match scope.place_ctx {
            PlaceCtx::Sub => scope
                .layout_owner
                .clone()
                .expect("PlaceCtx::Sub always carries the subdesign node as layout_owner"),
            _ => String::new(),
        };
        let dup = |ex: &mut Self, span: Span| {
            let key = (conflict_owner.clone(), resolved_target.clone());
            if !ex.reported_place_conflicts.insert(key) {
                return; // same owner + same resolved target: already reported
            }
            let mut diagnostic = Diagnostic::error(
                "E1007",
                span,
                format!("`{}` is placed more than once", placement.path_text()),
            );
            if !scope.frame.is_empty() {
                let context = format!("target `{resolved_target}`{}", ex.frame_suffix(scope));
                diagnostic.message.push_str(&format!("; {context}"));
                diagnostic = diagnostic.with_primary_label(context);
            }
            ex.diags.push(diagnostic);
        };
        match (scope.place_ctx, cur) {
            (PlaceCtx::Design, PlaceTarget::Inst(path)) => {
                if self.placements.iter().any(|p| p.path == path) {
                    dup(self, placement.path_span());
                    return;
                }
                self.placements.push(crate::ir::LayoutPlacement {
                    path,
                    at: data.at,
                    rotate: data.rotate,
                    side: data.side,
                });
            }
            (PlaceCtx::Design, PlaceTarget::Node(path)) => {
                if self.abs_node_places.contains_key(&path) {
                    dup(self, placement.path_span());
                    return;
                }
                self.abs_node_places.insert(path, data);
            }
            (PlaceCtx::Sub, target) => {
                let target_path = match &target {
                    PlaceTarget::Inst(p) | PlaceTarget::Node(p) => p.clone(),
                };
                // The frame a default is recorded against is the enclosing
                // subdesign NODE, never the loop-qualified `scope.path`: a
                // layout `for` iterates within the same coordinate owner
                // (RFC-033 §6 — `Scope::path` keeps iteration identity,
                // `Scope::layout_owner` keeps the coordinate frame).
                let owner = scope
                    .layout_owner
                    .clone()
                    .expect("PlaceCtx::Sub is only entered via handle_subdesign_use, which always sets layout_owner to the node path");
                let same = |t: &PlaceTarget| match t {
                    PlaceTarget::Inst(p) | PlaceTarget::Node(p) => *p == target_path,
                };
                if self
                    .rel_places
                    .iter()
                    .any(|r| r.owner == owner && same(&r.target))
                {
                    dup(self, placement.path_span());
                    return;
                }
                self.rel_places.push(RelPlace {
                    owner,
                    target,
                    data,
                });
            }
            (PlaceCtx::Fn, _) => unreachable!("rejected above"),
        }
    }

    /// Validate and record the board outline (RFC-020, E1006): a project-
    /// relative DXF path, declared at most once, and only in the design's own
    /// layout block — never inside a called fn (a board has one physical
    /// perimeter). The DXF is NOT read here — that happens at `cohdl build`
    /// (`pipeline::resolve_board_outline`); this only validates the reference.
    fn handle_board_outline(&mut self, outline: &crate::ast::BoardOutline, scope: &Scope) {
        if scope.place_ctx != PlaceCtx::Design || !self.active_calls.is_empty() {
            self.diags.push(Diagnostic::error(
                "E1006",
                outline.span,
                "`board_outline` is only valid in the design's own `layout {}` block — a board has one physical perimeter".to_string(),
            ));
            return;
        }
        // Path hygiene, mirroring RFC-017's #[doc] rule (review R5-9): a
        // project-relative reference only — never absolute, never `..`-escaping,
        // never a URL. The file itself is opened at build.
        let p = outline.path.trim();
        let bad = p.is_empty()
            || p.starts_with('/')
            || p.split(['/', '\\']).any(|seg| seg == "..")
            || p.contains("://")
            || (p.len() >= 2 && p.as_bytes()[1] == b':'); // drive letter
        if bad {
            self.diags.push(Diagnostic::error(
                "E1006",
                outline.path_span,
                format!(
                    "board outline path `{}` must be a project-relative file path (no absolute, `..`, or URL)",
                    outline.path
                ),
            ));
            return;
        }
        if self.board_outline.is_some() {
            self.diags.push(Diagnostic::error(
                "E1006",
                outline.span,
                "a design has at most one `board_outline`".to_string(),
            ));
            return;
        }
        self.board_outline = Some(crate::ir::BoardOutlineIr {
            path: outline.path.clone(),
            span: outline.span,
            geom: None,
        });
    }

    // -- RFC-027 physics-constraint attributes -------------------------------

    /// Resolve one inst's physics attributes. Every referenced name must be an
    /// instance in the CURRENT scope; pin names resolve against the referenced
    /// instance's device (its selected variant). All failures are E1009 naming
    /// exactly what was not found.
    fn handle_inst_phys(&mut self, inst: &InstStmt, scope: &Scope) {
        if inst.array_len.is_some() {
            self.diags.push(Diagnostic::error(
                "E1009",
                inst.phys[0].span(),
                format!(
                    "physics attributes are not supported on the array-typed instance `{}` — attach them to plain instances",
                    inst.name.name
                ),
            ));
            return;
        }
        let Some(owner) = scope.local_insts.get(&inst.name.name).cloned() else {
            return; // the inst itself failed earlier (already reported)
        };
        // RFC-028: an instance argument may be a local inst OR a fn's
        // Instance-typed parameter — the same two forms every other instance
        // reference already resolves through.
        let resolve_inst = |ex: &mut Self,
                            id: &Ident,
                            index: Option<&(Expr, Span)>|
         -> Option<String> {
            // RFC-024: `NAME[i]` resolves through the SAME element resolver
            // `place` uses, so the two can never disagree about which element
            // an index names. An unindexed name keeps its old meaning.
            if index.is_some() || scope.arrays.contains_key(&id.name) {
                let local = ex.indexed_local(
                    id,
                    index,
                    scope,
                    &format!(
                        "`{}` is array-typed — name one element, e.g. `{}[0]`",
                        id.name, id.name
                    ),
                )?;
                if let Some(p) = scope.local_insts.get(&local) {
                    return Some(p.clone());
                }
            }
            if let Some(p) = scope.local_insts.get(&id.name) {
                return Some(p.clone());
            }
            if let Some(Binding::Instance { path, .. }) = scope.bindings.get(&id.name) {
                return Some(path.clone());
            }
            // RFC-032: a subdesign node is never a physics target — its
            // real components are behind the port boundary.
            if scope.local_subs.contains_key(&id.name) {
                ex.diags.push(Diagnostic::error(
                        "E1009",
                        id.span,
                        format!(
                            "`{}` is a subdesign — a physics attribute needs a real instance; attach it inside the subdesign that owns the part (RFC-032)",
                            id.name
                        ),
                    ));
                return None;
            }
            ex.diags.push(Diagnostic::error(
                "E1009",
                id.span,
                format!("`{}` is not an instance in this scope", id.name),
            ));
            None
        };
        // The referenced instance's device pin, by NAME -> its pad numbers.
        let pin_pads = |ex: &mut Self, path: &str, pin: &Ident| -> Option<Vec<String>> {
            // RFC-032: a `Binding::Pin` may carry a subdesign PORT — a
            // logical junction with no physical pads. A physics fact needs a
            // real device pin, so say that precisely (never index-panic on
            // the deliberately-absent node path).
            let Some(target) = ex.instances.get(path) else {
                let what = if ex.sub_nodes.contains_key(path) {
                    format!(
                        "`{}` resolves to a port of subdesign use site `{}`",
                        pin.name,
                        crate::resolve::short(path)
                    )
                } else {
                    format!("`{}` does not resolve to a physical pin", pin.name)
                };
                ex.diags.push(Diagnostic::error(
                    "E1009",
                    pin.span,
                    format!(
                        "{} — a physics attribute needs a real device pin; attach it inside the subdesign that owns the pin (RFC-032)",
                        what
                    ),
                ));
                return None;
            };
            let dev = ex.world.devices.get(&target.device)?;
            let variant = target.variant.clone();
            match dev
                .pins_for(variant.as_deref())
                .iter()
                .find(|p| p.name.name == pin.name)
            {
                Some(p) => Some(p.numbers.iter().map(|n| n.text.clone()).collect()),
                None => {
                    ex.diags.push(Diagnostic::error(
                        "E1009",
                        pin.span,
                        format!(
                            "`{}` has no pin `{}` (device `{}`)",
                            path.rsplit("::").next().unwrap_or(path),
                            pin.name,
                            crate::resolve::short(&target.device)
                        ),
                    ));
                    None
                }
            }
        };
        for pa in &inst.phys {
            let targets = match pa {
                PhysAttr::Bypass { .. } => 1,
                PhysAttr::CrystalOscillator { .. } => 3,
                PhysAttr::SwitchingConverter {
                    input_capacitor,
                    output_capacitor,
                    ..
                } => {
                    1 + u64::from(input_capacitor.is_some()) + u64::from(output_capacitor.is_some())
                }
                _ => 0,
            };
            if !self.meter.charge(
                1 + targets,
                "physics record and targets",
                pa.span(),
                self.diags,
            ) {
                return;
            }
            match pa {
                PhysAttr::Bypass {
                    inst: target,
                    index,
                    pin,
                    capacitance,
                    ..
                } => {
                    // RFC-028: `TARGET` is INST.PIN, or a bare Pin-typed fn
                    // parameter resolving through the call site's binding —
                    // the same Binding::Pin every net member already uses.
                    let (target_path, pads) = match pin {
                        Some(pin) => {
                            let Some(target_path) = resolve_inst(self, target, index.as_ref())
                            else {
                                continue;
                            };
                            let Some(pads) = pin_pads(self, &target_path, pin) else {
                                continue;
                            };
                            (target_path, pads)
                        }
                        None => match scope.bindings.get(&target.name) {
                            Some(Binding::Pin((path, pin_name))) => {
                                let path = path.clone();
                                let pin_ident = Ident {
                                    name: pin_name.clone(),
                                    span: target.span,
                                };
                                let Some(pads) = pin_pads(self, &path, &pin_ident) else {
                                    continue;
                                };
                                (path, pads)
                            }
                            _ => {
                                self.diags.push(Diagnostic::error(
                                    "E1009",
                                    target.span,
                                    format!(
                                        "`{}` is neither an `INST.PIN` reference nor a `Pin`-typed fn parameter in scope",
                                        target.name
                                    ),
                                ));
                                continue;
                            }
                        },
                    };
                    self.phys_bypasses.push(crate::ir::QuilterBypass {
                        cap_path: owner.clone(),
                        target_path,
                        pads,
                        capacitance: capacitance.clone(),
                    });
                }
                PhysAttr::CrystalOscillator {
                    parent, pin1, pin2, ..
                } => {
                    let Some(parent_path) = resolve_inst(self, parent, None) else {
                        continue;
                    };
                    let mut pads = Vec::new();
                    let mut ok = true;
                    for pin in [pin1, pin2] {
                        match pin_pads(self, &parent_path, pin) {
                            Some(nums) if nums.len() == 1 => pads.push(nums[0].clone()),
                            Some(nums) => {
                                self.diags.push(Diagnostic::error(
                                    "E1009",
                                    pin.span,
                                    format!(
                                        "`#[crystal_oscillator]` pin `{}` maps to {} pads — a crystal signal pin must map to exactly one",
                                        pin.name,
                                        nums.len()
                                    ),
                                ));
                                ok = false;
                            }
                            None => ok = false,
                        }
                    }
                    if ok {
                        self.phys_crystals.push(crate::ir::QuilterCrystal {
                            crystal_path: owner.clone(),
                            parent_path,
                            pad1: pads[0].clone(),
                            pad2: pads[1].clone(),
                        });
                    }
                }
                PhysAttr::SwitchingConverter {
                    inductor,
                    input_capacitor,
                    output_capacitor,
                    ..
                } => {
                    let Some(inductor_path) = resolve_inst(self, inductor, None) else {
                        continue;
                    };
                    let input_cap_path = match input_capacitor {
                        Some(c) => match resolve_inst(self, c, None) {
                            Some(p) => Some(p),
                            None => continue,
                        },
                        None => None,
                    };
                    let output_cap_path = match output_capacitor {
                        Some(c) => match resolve_inst(self, c, None) {
                            Some(p) => Some(p),
                            None => continue,
                        },
                        None => None,
                    };
                    self.phys_converters.push(crate::ir::QuilterConverter {
                        conv_path: owner.clone(),
                        inductor_path,
                        input_cap_path,
                        output_cap_path,
                    });
                }
                PhysAttr::BgaFanout { .. } => self.phys_bga.push(owner.clone()),
                // Net-target kinds cannot reach an inst (parse enforces).
                _ => unreachable!("net-target attribute on an inst"),
            }
        }
    }

    // -- instances -----------------------------------------------------------

    fn handle_inst(&mut self, inst: &InstStmt, scope: &mut Scope) {
        if !self.check_not_reserved(&inst.name, "instance") {
            return;
        }
        if scope.local_insts.contains_key(&inst.name.name)
            || scope.bindings.contains_key(&inst.name.name)
            || scope.local_subs.contains_key(&inst.name.name)
            || scope.arrays.contains_key(&inst.name.name)
        {
            self.diags.push(Diagnostic::error(
                "E201",
                inst.name.span,
                format!("`{}` is already defined in this scope", inst.name.name),
            ));
            return;
        }
        // RFC-033 §9: one work item per real instance, charged BEFORE any
        // materialization (a tripped meter leaves nothing behind).
        if !self.meter.charge(1, "`inst`", inst.span, self.diags) {
            return;
        }

        let ty_name = &inst.ty.name;
        let (device_name, args, part): (String, Substitution, Option<String>) = if let Some(
            part_def,
        ) =
            self.world.parts.get(&ty_name.name)
        {
            if !inst.ty.generic_args.is_empty() {
                self.diags.push(Diagnostic::error(
                    "E401",
                    inst.ty.span,
                    format!(
                        "part `{}` is already fully bound — it takes no generic arguments",
                        ty_name.name
                    ),
                ));
            }
            let Some(dev) = self.world.devices.get(&part_def.device.name.name) else {
                return; // already reported by check_parts
            };
            let args = resolve_generic_args(
                self.world,
                &format!("device `{}`", dev.name.name),
                &dev.generics,
                &part_def.device.generic_args,
                &Substitution::new(),
                part_def.device.span,
                &mut Diagnostics::new(), // already reported by check_parts
            );
            // A part is fully bound — its variant comes from the part
            // declaration, never from the instantiation site.
            if let Some(sel) = &inst.ty.variant {
                self.diags.push(Diagnostic::error(
                    "E905",
                    sel.span,
                    format!(
                        "part `{}` already selects its variant — remove the `[{}]` selector",
                        part_def.name.name, sel.name
                    ),
                ));
            }
            (
                // RFC-016: map keys are fq paths — the part's device ref was
                // rewritten to the device's fq key; the part's own key is the
                // reference text that just matched.
                part_def.device.name.name.clone(),
                args,
                Some(ty_name.name.clone()),
            )
        } else if let Some(dev) = self.world.devices.get(&ty_name.name) {
            let args = self.bound_generic_args(
                &format!("device `{}`", dev.name.name),
                &dev.generics,
                &inst.ty.generic_args,
                scope,
                inst.ty.span,
            );
            (ty_name.name.clone(), args, None)
        } else if scope.subst.contains_key(&ty_name.name) {
            self.diags.push(Diagnostic::error(
                    "E205",
                    ty_name.span,
                    format!(
                        "cannot instantiate generic parameter `{}` — `inst` requires a concrete device or part",
                        ty_name.name
                    ),
                ));
            return;
        } else if self.world.traits.contains_key(&ty_name.name) {
            self.diags.push(Diagnostic::error(
                "E205",
                ty_name.span,
                format!(
                    "`{}` is a trait — `inst` requires a concrete device or part",
                    ty_name.name
                ),
            ));
            return;
        } else {
            let mut d = Diagnostic::error(
                "E202",
                ty_name.span,
                format!("unknown device or part `{}`", ty_name.name),
            );
            if let Some(sugg) = self.world.suggest(&ty_name.name) {
                d = d.with_help(format!("did you mean `{}`?", sugg));
            }
            self.diags.push(d);
            return;
        };

        let device = &self.world.devices[&device_name];

        // RFC-008: resolve the variant selection. Parts carry their own
        // selector (validated by check_parts); direct device instantiations
        // validate here (E903 undeclared / E904 omitted / E905 spurious).
        let variant: Option<String> = if part.is_some() {
            self.world.parts[ty_name.name.as_str()]
                .device
                .variant
                .as_ref()
                .map(|v| v.name.clone())
        } else {
            let valid_set = || {
                device
                    .variants
                    .iter()
                    .map(|v| v.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            match (&inst.ty.variant, device.has_variants()) {
                (Some(sel), true) => {
                    if device.variants.iter().any(|v| v.name == sel.name) {
                        Some(sel.name.clone())
                    } else {
                        self.diags.push(
                            Diagnostic::error(
                                "E903",
                                sel.span,
                                format!(
                                    "device `{}` declares no variant named `{}`",
                                    device_name, sel.name
                                ),
                            )
                            .with_help(format!("valid variants are: {}", valid_set())),
                        );
                        return;
                    }
                }
                (None, true) => {
                    self.diags.push(
                        Diagnostic::error(
                            "E904",
                            inst.ty.span,
                            format!(
                                "device `{}` declares variants — select one with a `[VARIANT]` suffix (no implicit default)",
                                device_name
                            ),
                        )
                        .with_help(format!("valid variants are: {}", valid_set())),
                    );
                    return;
                }
                (Some(sel), false) => {
                    self.diags.push(Diagnostic::error(
                        "E905",
                        sel.span,
                        format!(
                            "device `{}` has no `variants {{ }}` block — remove the `[{}]` selector",
                            device_name, sel.name
                        ),
                    ));
                    return;
                }
                (None, false) => None,
            }
        };

        // Concrete spec values via substitution, over the variant-merged
        // spec fields (base `spec {}` + `spec[VARIANT]` overrides, RFC-008).
        let mut specs = BTreeMap::new();
        for field in device.spec_fields_for(variant.as_deref()) {
            match &field.value {
                SpecValue::Lit(v, _) => {
                    specs.insert(field.name.name.clone(), v.clone());
                }
                SpecValue::GenericRef(r) => {
                    if let Some(GenericValue::Unit(v)) = args.get(&r.name) {
                        specs.insert(field.name.name.clone(), v.clone());
                    }
                    // Missing → the arg error was already reported.
                }
            }
        }

        // #[designator("U7")] override (RFC-005).
        let mut designator_override = None;
        for attr in &inst.attrs {
            if attr.name.name == "designator" {
                match attr.args.as_slice() {
                    [(text, span)] => {
                        if is_valid_designator(text) {
                            designator_override = Some((text.clone(), *span));
                        } else {
                            self.diags.push(Diagnostic::error(
                                "E804",
                                *span,
                                format!(
                                    "`{}` is not a valid designator — expected an uppercase prefix followed by a number, e.g. `U7`",
                                    text
                                ),
                            ));
                        }
                    }
                    _ => {
                        self.diags.push(Diagnostic::error(
                            "E804",
                            attr.span,
                            "`#[designator(…)]` takes exactly one string, e.g. `#[designator(\"U7\")]`",
                        ));
                    }
                }
            } else {
                self.diags.push(Diagnostic::error(
                    "E010",
                    attr.span,
                    format!(
                        "unrecognized attribute `{}` (only `#[designator(\"…\")]` and `#[intent(\"…\")]` are supported)",
                        attr.name.name
                    ),
                ));
            }
        }

        let path = format!("{}::{}", scope.path, inst.name.name);
        scope
            .local_insts
            .insert(inst.name.name.clone(), path.clone());
        self.instances.insert(
            path.clone(),
            IrInstance {
                path,
                device: device_name.clone(),
                variant,
                specs,
                part,
                designator_override,
                designator: None,
                placement_hint: inst.placement_hint.as_ref().map(|(s, _)| s.clone()),
                impl_traits: self.world.implemented_traits(&device_name),
                span: inst.span,
            },
        );
    }

    // -- pin references ------------------------------------------------------

    /// Resolve a pin reference to (instance path, logical pin name).
    /// RFC-024: an array element's internal identity — exactly as if the
    /// author had hand-written `NAME_0: Device`, `NAME_1: Device`, … The
    /// source-facing spelling stays `NAME[i]`.
    /// RFC-033: evaluate a selector's expressions to concrete indices
    /// (Range inclusive semantics: start..=end with optional step).
    fn array_bounds(
        &mut self,
        base: &Ident,
        sel: &IndexSel,
        n: i64,
        scope: &Scope,
        charge_members: bool,
    ) -> Option<Vec<i64>> {
        let out_of_bounds = |ex: &mut Self, i: i64| {
            ex.diags.push(Diagnostic::error(
                "E202",
                sel.span(),
                format!(
                    "index {} is out of bounds for `{}` — valid indices are 0..={} (length {}){}",
                    i,
                    base.name,
                    n - 1,
                    n,
                    ex.frame_suffix(scope)
                ),
            ));
        };
        let indices = match sel {
            IndexSel::Range {
                start, end, step, ..
            } => {
                let start = self.eval_int(start, scope, "a range start")?;
                let end = self.eval_int(end, scope, "a range end")?;
                let step = match step {
                    None => 1,
                    Some(e) => self.eval_int(e, scope, "a stride")?,
                };
                if step <= 0 {
                    self.diags.push(Diagnostic::error(
                        "E211",
                        sel.span(),
                        format!("array range stride `{step}` must be 1 or more"),
                    ));
                    return None;
                }
                if start > end {
                    Vec::new()
                } else {
                    // Count and validate a range without constructing it. i128
                    // handles the distance between any two i64 endpoints.
                    if start < 0 || start >= n {
                        out_of_bounds(self, start);
                        return None;
                    }
                    let count = (i128::from(end) - i128::from(start)) / i128::from(step) + 1;
                    let last = i128::from(start) + (count - 1) * i128::from(step);
                    if last >= i128::from(n) {
                        let first_bad = i128::from(start)
                            + ((i128::from(n) - i128::from(start) - 1) / i128::from(step) + 1)
                                * i128::from(step);
                        out_of_bounds(self, first_bad as i64);
                        return None;
                    }
                    if charge_members
                        && !self
                            .meter
                            .charge(count as u64, "net members", sel.span(), self.diags)
                    {
                        return None;
                    }
                    return Some(
                        std::iter::successors(Some(start), |i| {
                            i.checked_add(step).filter(|next| *next <= end)
                        })
                        .collect(),
                    );
                }
            }
            IndexSel::Single(e, _) => vec![self.eval_int(e, scope, "an index")?],
            IndexSel::List(items, _) => {
                if charge_members
                    && !self.meter.ensure_capacity(
                        items.len() as u64,
                        "net members",
                        sel.span(),
                        self.diags,
                    )
                {
                    return None;
                }
                let mut out = Vec::with_capacity(items.len());
                for e in items {
                    out.push(self.eval_int(e, scope, "an index")?);
                }
                out
            }
        };
        if indices.is_empty() {
            self.diags.push(Diagnostic::error(
                "E211",
                sel.span(),
                format!("`{}[…]` selects no elements", base.name),
            ));
            return None;
        }
        for i in &indices {
            if *i < 0 || *i >= n {
                out_of_bounds(self, *i);
                return None;
            }
        }
        if charge_members
            && !self
                .meter
                .charge(indices.len() as u64, "net members", sel.span(), self.diags)
        {
            return None;
        }
        Some(indices)
    }

    /// RFC-024: expand a possibly-indexed net member into flat, ordinary
    /// single-instance references — the exact list an author would have
    /// hand-written. Every index must name a real declared instance; the
    /// FIRST one that doesn't is reported (E202, the same unresolved-name
    /// class RFC-016 established) and the member contributes nothing, so one
    /// mistyped range yields one diagnostic rather than one per index.
    fn expand_member(&mut self, m: &PinRef, scope: &Scope) -> Vec<PinRef> {
        // Only the fan-out SUGAR (range/list) expands here; a `Single` index
        // is a real reference and is resolved by `resolve_pin_ref` itself.
        let Some(sel @ (IndexSel::Range { .. } | IndexSel::List(..))) = &m.index else {
            return if self.meter.charge(1, "net member", m.span, self.diags) {
                vec![m.clone()]
            } else {
                Vec::new()
            };
        };
        let Some((n, _)) = scope.arrays.get(&m.base.name).copied() else {
            self.diags.push(Diagnostic::error(
                "E211",
                sel.span(),
                format!(
                    "`{}` is not an array-typed instance — only `inst NAME: [Device; N]` can be indexed",
                    m.base.name
                ),
            ));
            return Vec::new();
        };
        let Some(idx) = self.array_bounds(&m.base, sel, n, scope, true) else {
            return Vec::new();
        };
        idx.into_iter()
            .map(|i| PinRef {
                base: m.base.clone(),
                index: Some(IndexSel::Single(Expr::int(i, sel.span()), sel.span())),
                pin: m.pin.clone(),
                span: m.span,
            })
            .collect()
    }

    /// RFC-024: resolve one reference's selector before looking up its base.
    /// Pin references and instance arguments must share the same element
    /// identity and validation. Net-member fan-out is expanded by `handle_net`
    /// before reaching this single-element resolver.
    fn resolve_array_ref<'r>(&mut self, r: &'r PinRef, scope: &Scope) -> Option<Cow<'r, PinRef>> {
        let array = scope.arrays.get(&r.base.name).copied();
        match (&r.index, array) {
            (None, None) => Some(Cow::Borrowed(r)),
            (None, Some(_)) => {
                self.diags.push(Diagnostic::error(
                    "E211",
                    r.base.span,
                    format!(
                        "`{}` is array-typed — reference one element, e.g. `{}[0]`",
                        r.base.name, r.base.name
                    ),
                ));
                None
            }
            (Some(sel), None) => {
                self.diags.push(Diagnostic::error(
                    "E211",
                    sel.span(),
                    format!(
                        "`{}` is not an array-typed instance — only `inst NAME: [Device; N]` can be indexed",
                        r.base.name
                    ),
                ));
                None
            }
            (Some(sel), Some((n, _))) => {
                let IndexSel::Single(e, _) = sel else {
                    self.diags.push(Diagnostic::error(
                        "E211",
                        sel.span(),
                        format!(
                            "a range or index list is only valid in a net's member list — `{}` needs a single index here",
                            r.base.name
                        ),
                    ));
                    return None;
                };
                // RFC-033: a computed single index evaluates in this scope.
                let i = self.eval_int(e, scope, "an index")?;
                self.array_bounds(&r.base, sel, n, scope, false)?;
                Some(Cow::Owned(PinRef {
                    base: Ident {
                        name: element_name(&r.base.name, i),
                        span: r.base.span,
                    },
                    index: None,
                    pin: r.pin.clone(),
                    span: r.span,
                }))
            }
        }
    }

    fn resolve_pin_ref(&mut self, r: &PinRef, scope: &Scope) -> Option<(String, String)> {
        let r = self.resolve_array_ref(r, scope)?;
        // Base: a fn parameter binding?
        if let Some(binding) = scope.bindings.get(&r.base.name) {
            return match (binding, &r.pin) {
                (Binding::Pin(target), None) => Some(target.clone()),
                (Binding::Pin(_), Some(pin)) => {
                    self.diags.push(Diagnostic::error(
                        "E602",
                        pin.span,
                        format!(
                            "`{}` is a `Pin` parameter — it is already a pin and has no `.{}`",
                            r.base.name, pin.name
                        ),
                    ));
                    None
                }
                (Binding::Instance { .. }, None) => {
                    self.diags.push(Diagnostic::error(
                        "E602",
                        r.span,
                        format!(
                            "`{}` is an instance — reference one of its pins (e.g. `{}.A`)",
                            r.base.name, r.base.name
                        ),
                    ));
                    None
                }
                (
                    Binding::Instance {
                        path,
                        device,
                        via_traits,
                    },
                    Some(pin),
                ) => {
                    // Trait-bound access: the pin name is a trait role,
                    // resolved through the concrete device's impls.
                    let device = device.clone();
                    let path = path.clone();
                    let via = via_traits.clone();
                    self.resolve_trait_role(&path, &device, &via, pin)
                }
            };
        }
        // Base: a subdesign use site? Its PORTS are its whole electrical
        // surface (RFC-032) — internal nets and instances stay behind the
        // boundary; only `place` may reach in.
        if let Some(node_path) = scope.local_subs.get(&r.base.name) {
            let node = &self.sub_nodes[node_path];
            let Some(pin) = &r.pin else {
                self.diags.push(Diagnostic::error(
                    "E1303",
                    r.span,
                    format!(
                        "`{}` is a subdesign — reference one of its ports (e.g. `{}.{}`)",
                        r.base.name,
                        r.base.name,
                        node.ports
                            .keys()
                            .next()
                            .map(|p| p.as_str())
                            .unwrap_or("PORT")
                    ),
                ));
                return None;
            };
            if node.ports.contains_key(&pin.name) {
                return Some((node_path.clone(), pin.name.clone()));
            }
            self.diags.push(
                Diagnostic::error(
                    "E1301",
                    pin.span,
                    format!(
                        "subdesign `{}` (use site `{}`) has no port named `{}`",
                        crate::resolve::short(&node.fq),
                        r.base.name,
                        pin.name
                    ),
                )
                .with_help(format!(
                    "its ports are: {} — internal nets and instances are behind the port boundary; only `place` may reach in (RFC-032)",
                    node.ports.keys().cloned().collect::<Vec<_>>().join(", ")
                )),
            );
            return None;
        }
        // Base: a local instance?
        if let Some(path) = scope.local_insts.get(&r.base.name) {
            let inst = &self.instances[path];
            let device = &self.world.devices[&inst.device];
            // RFC-008: the instance's pin layout is its selected variant's.
            let pins = device.pins_for(inst.variant.as_deref());
            let Some(pin) = &r.pin else {
                self.diags.push(Diagnostic::error(
                    "E602",
                    r.span,
                    format!(
                        "`{}` is an instance — reference one of its pins (e.g. `{}.{}`)",
                        r.base.name,
                        r.base.name,
                        pins.first().map(|p| p.name.name.as_str()).unwrap_or("PIN")
                    ),
                ));
                return None;
            };
            if pins.iter().any(|p| p.name.name == pin.name) {
                return Some((path.clone(), pin.name.clone()));
            }
            self.diags.push(
                Diagnostic::error(
                    "E203",
                    pin.span,
                    format!(
                        "device `{}` (instance `{}`) has no pin named `{}`",
                        crate::resolve::short(&inst.device),
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
            return None;
        }
        self.diags.push(Diagnostic::error(
            "E202",
            r.base.span,
            format!(
                "unknown instance or parameter `{}` in this scope",
                r.base.name
            ),
        ));
        None
    }

    /// Resolve `target.ROLE` where `target` was passed under trait bounds:
    /// find the (unique) bound trait — or transitive super-trait — declaring
    /// the role, then map role → device pin through the checked impl.
    fn resolve_trait_role(
        &mut self,
        path: &str,
        device: &str,
        via_traits: &[String],
        role: &Ident,
    ) -> Option<(String, String)> {
        let mut all_traits: BTreeSet<String> = BTreeSet::new();
        for t in via_traits {
            all_traits.insert(t.clone());
            for s in self.world.super_traits_transitive(t) {
                all_traits.insert(s);
            }
        }
        for trait_name in &all_traits {
            let Some(tr) = self.world.traits.get(trait_name) else {
                continue;
            };
            if tr.pins.iter().any(|p| p.name.name == role.name) {
                if let Some(resolved) = self
                    .world
                    .resolved_impls
                    .get(&(trait_name.clone(), device.to_string()))
                {
                    if let Some(dev_pin) = resolved.pin_map.get(&role.name) {
                        return Some((path.to_string(), dev_pin.clone()));
                    }
                }
                // The impl exists but failed checking — already reported.
                return None;
            }
        }
        self.diags.push(
            Diagnostic::error(
                "E203",
                role.span,
                format!(
                    "no pin role `{}` in the trait bound{} {}",
                    role.name,
                    if via_traits.len() == 1 { "" } else { "s" },
                    via_traits
                        .iter()
                        .map(|t| format!("`{}`", t))
                        .collect::<Vec<_>>()
                        .join(" + ")
                ),
            )
            .with_help(
                "a trait-bound parameter's pins are known only through its trait bounds (RFC-007)",
            ),
        );
        None
    }

    // -- nets / nc -----------------------------------------------------------

    /// Names beginning with `__` are reserved for compiler-generated
    /// expansion names (`__fn{N}_…`, `__net{N}`) — RFC-006's collision-free
    /// naming guarantee holds only if user names can never enter that
    /// namespace.
    fn check_not_reserved(&mut self, name: &Ident, what: &str) -> bool {
        if name.name.starts_with("__") {
            self.diags.push(
                Diagnostic::error(
                    "E206",
                    name.span,
                    format!(
                        "{} names beginning with `__` are reserved for compiler-generated names",
                        what
                    ),
                )
                .with_help("pick a name that does not start with `__`"),
            );
            return false;
        }
        true
    }

    fn handle_net(&mut self, net: &NetStmt, scope: &mut Scope) {
        // RFC-033 §9: one work item per statement + one per member, before
        // any resolution push.
        if !self.meter.charge(1, "`net`", net.span, self.diags) {
            return;
        }
        if let Some(name) = &net.name {
            if !self.check_not_reserved(name, "net") {
                return;
            }
        }
        let mut members = Vec::new();
        for m in &net.members {
            // RFC-024: a range/stride/list member expands to the flat PinRef
            // list first; everything downstream is byte-identical to the
            // hand-written form.
            let expanded_members = self.expand_member(m, scope);
            if self.meter.tripped() {
                return;
            }
            for expanded in expanded_members {
                if let Some(resolved) = self.resolve_pin_ref(&expanded, scope) {
                    members.push(resolved);
                }
            }
        }
        let (key, display_name, is_design_level_name) = match (&net.name, scope.is_design_body) {
            (Some(name), true) => (format!("named:{}", name.name), name.name.clone(), true),
            (Some(name), false) => {
                let scoped = format!("{}::{}", scope.path, name.name);
                let display = scoped
                    .strip_prefix(&format!("{}::", scope.design_name))
                    .unwrap_or(&scoped)
                    .to_string();
                (format!("scoped:{}", scoped), display, false)
            }
            (None, _) => {
                let n = self.anon_net_counter;
                self.anon_net_counter += 1;
                let scoped = format!("{}::__net{}", scope.path, n);
                let display = scoped
                    .strip_prefix(&format!("{}::", scope.design_name))
                    .unwrap_or(&scoped)
                    .to_string();
                (format!("scoped:{}", scoped), display, false)
            }
        };
        // RFC-027: record this declaration's physics attributes against the
        // net's emitted display name (dup/one-primary checks at assembly).
        for pa in &net.phys {
            if !self
                .meter
                .charge(1, "net physics record", pa.span(), self.diags)
            {
                return;
            }
            match pa {
                PhysAttr::Ground {
                    primary,
                    region_pour,
                    ..
                } => self.phys_grounds.push((
                    crate::ir::QuilterGround {
                        net: display_name.clone(),
                        primary: *primary,
                        region_pour: *region_pour,
                    },
                    pa.span(),
                )),
                PhysAttr::HighCurrent {
                    current,
                    power_pour,
                    ..
                } => self.phys_high_currents.push((
                    crate::ir::QuilterHighCurrent {
                        net: display_name.clone(),
                        current: current.clone(),
                        power_pour: *power_pour,
                    },
                    pa.span(),
                )),
                PhysAttr::Impedance {
                    impedance,
                    frequency,
                    ..
                } => self.phys_impedances.push((
                    crate::ir::QuilterImpedance {
                        net: display_name.clone(),
                        impedance: impedance.clone(),
                        frequency: frequency.clone(),
                    },
                    pa.span(),
                )),
                _ => unreachable!("inst-target attribute on a net"),
            }
        }
        self.net_decls.push(NetDecl {
            key,
            display_name,
            is_design_level_name,
            synthesized: false,
            annotation: net.annotation.clone(),
            members,
            span: net.span,
        });
    }

    fn handle_nc(&mut self, nc: &NcStmt, scope: &mut Scope) {
        // RFC-033 §9: one work item per statement + one per member.
        if !self.meter.charge(1, "`nc`", nc.span, self.diags) {
            return;
        }
        if !self
            .meter
            .charge(nc.members.len() as u64, "nc members", nc.span, self.diags)
        {
            return;
        }
        for m in &nc.members {
            // Fan-out sugar stays scoped to NET member lists (E211 contract,
            // tests/inst_array.rs) — `nc` takes single elements only.
            if let Some(resolved) = self.resolve_pin_ref(m, scope) {
                // RFC-032: a port is a connection surface, not a device pin —
                // `nc` has no meaning for it (an optional port is simply left
                // unconnected).
                if self.sub_nodes.contains_key(&resolved.0) {
                    self.diags.push(Diagnostic::error(
                        "E1306",
                        m.span,
                        format!(
                            "a subdesign port cannot be marked `nc` — leave the optional port `{}` unconnected instead",
                            m
                        ),
                    ));
                    continue;
                }
                self.nc_pins.push((resolved, m.span));
            }
        }
    }

    // -- calls (RFC-006) -----------------------------------------------------

    fn handle_call(&mut self, call: &CallStmt, scope: &mut Scope) {
        if self.expression_failed(call.span, scope) {
            return;
        }
        let Some(fndef) = self.world.fns.get(&call.callee.name) else {
            let d = if self.world.devices.contains_key(&call.callee.name)
                || self.world.parts.contains_key(&call.callee.name)
            {
                Diagnostic::error(
                    "E205",
                    call.callee.span,
                    format!(
                        "`{}` is a device/part — instantiate it with `inst name: {}`",
                        call.callee.name, call.callee.name
                    ),
                )
            } else {
                let mut d = Diagnostic::error(
                    "E504",
                    call.callee.span,
                    format!("unknown fn `{}`", call.callee.name),
                );
                if let Some(sugg) = self.world.suggest(&call.callee.name) {
                    d = d.with_help(format!("did you mean `{}`?", sugg));
                }
                d
            };
            self.diags.push(d);
            return;
        };

        // Cycle detection BEFORE any expansion (RFC-006: never leave partial
        // half-expanded state behind).
        if self.active_calls.contains(&call.callee.name) {
            let mut chain: Vec<&str> = self
                .active_calls
                .iter()
                .skip_while(|n| **n != call.callee.name)
                .map(|s| s.as_str())
                .collect();
            chain.push(&call.callee.name);
            self.diags.push(Diagnostic::error(
                "E501",
                call.span,
                format!(
                    "recursive fn call: `{}` is already being expanded in this call chain: {}",
                    crate::resolve::short(&call.callee.name),
                    chain
                        .iter()
                        .map(|s| crate::resolve::short(s))
                        .collect::<Vec<_>>()
                        .join(" → ")
                ),
            ));
            return;
        }

        if fndef
            .generics
            .iter()
            .any(|p| self.expression_failed(p.span, scope))
        {
            return;
        }

        // Named generic parameters come from the turbofish, resolved in the
        // CALLER's lexical environment (outward-in threading, RFC-006/033).
        let subst = self.bound_generic_args(
            &format!("fn `{}`", fndef.name.name),
            &fndef.generics,
            &call.generic_args,
            scope,
            call.span,
        );

        // Bind value parameters.
        if call.args.len() != fndef.params.len() {
            self.diags.push(Diagnostic::error(
                "E502",
                call.span,
                format!(
                    "fn `{}` takes {} argument{}, but {} {} given",
                    fndef.name.name,
                    fndef.params.len(),
                    if fndef.params.len() == 1 { "" } else { "s" },
                    call.args.len(),
                    if call.args.len() == 1 { "was" } else { "were" }
                ),
            ));
            return;
        }

        let mut bindings = BTreeMap::new();
        let mut bind_failed = false;
        for (param, arg) in fndef.params.iter().zip(&call.args) {
            match &param.ty {
                FnParamTy::Pin(_) => match self.resolve_pin_ref(arg, scope) {
                    Some(target) => {
                        bindings.insert(param.name.name.clone(), Binding::Pin(target));
                    }
                    None => bind_failed = true,
                },
                FnParamTy::Generic(gname) => {
                    match self.resolve_instance_arg(arg, scope) {
                        Some((path, device)) => {
                            // The instance's device must be exactly the type
                            // bound to the generic parameter.
                            match subst.get(&gname.name) {
                                Some(GenericValue::Device(d)) if *d == device => {
                                    let via = match fndef
                                        .generics
                                        .iter()
                                        .find(|g| g.name.name == gname.name)
                                        .map(|g| &g.bound)
                                    {
                                        Some(GenericBound::Traits(ts)) => {
                                            ts.iter().map(|t| t.name.clone()).collect()
                                        }
                                        _ => Vec::new(),
                                    };
                                    bindings.insert(
                                        param.name.name.clone(),
                                        Binding::Instance {
                                            path,
                                            device,
                                            via_traits: via,
                                        },
                                    );
                                }
                                Some(GenericValue::Device(d)) => {
                                    self.diags.push(Diagnostic::error(
                                        "E503",
                                        arg.span,
                                        format!(
                                            "`{}` expects an instance of `{}` (the argument for `{}`), found an instance of `{}`",
                                            param.name.name, d, gname.name, device
                                        ),
                                    ));
                                    bind_failed = true;
                                }
                                _ => {
                                    // Turbofish arg missing/failed — reported.
                                    bind_failed = true;
                                }
                            }
                        }
                        None => bind_failed = true,
                    }
                }
                FnParamTy::ImplTrait(bound_traits, _) => {
                    // Sugar for an anonymous trait-bound generic parameter
                    // (RFC-007/DR-016): routed through the ONE trait-bound-
                    // checking mechanism, exactly like a named parameter.
                    match self.resolve_instance_arg(arg, scope) {
                        Some((path, device)) => {
                            let required_by = format!(
                                "parameter `{}` of fn `{}`",
                                param.name.name, fndef.name.name
                            );
                            let ok = crate::check::generics::check_trait_bounds(
                                self.world,
                                &device,
                                bound_traits,
                                arg.span,
                                &required_by,
                                self.diags,
                            );
                            if ok {
                                bindings.insert(
                                    param.name.name.clone(),
                                    Binding::Instance {
                                        path,
                                        device,
                                        via_traits: bound_traits
                                            .iter()
                                            .map(|t| t.name.clone())
                                            .collect(),
                                    },
                                );
                            } else {
                                bind_failed = true;
                            }
                        }
                        None => bind_failed = true,
                    }
                }
            }
        }
        if bind_failed {
            return;
        }

        let seg = format!("__fn{}_{}", self.call_counter, fndef.name.name);
        // RFC-033 §9: one work item per entered call, before the body walk.
        if !self.meter.charge(1, "the call", call.span, self.diags) {
            return;
        }
        self.call_counter += 1;
        let mut inner = Scope {
            design_name: scope.design_name.clone(),
            path: format!("{}::{}", scope.path, seg),
            is_design_body: false,
            place_ctx: PlaceCtx::Fn,
            layout_owner: None, // `place` is rejected in Fn contexts outright
            subst,
            bindings,
            local_insts: BTreeMap::new(),
            local_subs: BTreeMap::new(),
            arrays: BTreeMap::new(),
            declared_lens: BTreeMap::new(),
            failed_expressions: Vec::new(),
            unknown_arrays: BTreeSet::new(),
            consts: BTreeMap::new(),
            binders: BTreeMap::new(),
            frame: Vec::new(),
            caller_frames: scope
                .caller_frames
                .iter()
                .chain(&scope.frame)
                .cloned()
                .collect(),
        };
        self.active_calls.push(call.callee.name.clone());
        // Clone the body to release the borrow on `self.world`.
        let body = fndef.body.clone();
        self.walk_body(&body, &mut inner);
        self.active_calls.pop();
    }

    // -- subdesign use sites (RFC-032) ---------------------------------------

    /// Expand one `subdesign local: Name<…>` use site into its retained node
    /// (or, array-typed, its N nodes) — pass 1, so nets anywhere in the body
    /// can reference `local.PORT`.
    fn handle_subdesign_use(&mut self, stmt: &SubdesignUseStmt, scope: &mut Scope) {
        // A rejected length must not reach even the node/port budget preflight.
        if stmt.array_len.is_some() && !scope.declared_lens.contains_key(&stmt.name.name) {
            return;
        }
        if !self.check_not_reserved(&stmt.name, "subdesign use-site") {
            return;
        }
        if scope.place_ctx == PlaceCtx::Fn {
            self.diags.push(Diagnostic::error(
                "E1307",
                stmt.span,
                "a `subdesign` use site needs a retained hierarchy path — a `fn` expands inline and cannot contain one (RFC-032); move it into the design or a subdesign".to_string(),
            ));
            return;
        }
        if scope.local_insts.contains_key(&stmt.name.name)
            || scope.bindings.contains_key(&stmt.name.name)
            || scope.local_subs.contains_key(&stmt.name.name)
            || scope.arrays.contains_key(&stmt.name.name)
        {
            self.diags.push(Diagnostic::error(
                "E201",
                stmt.name.span,
                format!("`{}` is already defined in this scope", stmt.name.name),
            ));
            return;
        }
        let ty_name = &stmt.ty.name;
        let Some(sd) = self.world.subdesigns.get(&ty_name.name) else {
            // Unresolved names were already reported at the rewrite pass;
            // a name of the WRONG KIND is reported here, precisely.
            if let Some(sym) = self.world.symbols.get(&ty_name.name) {
                self.diags.push(Diagnostic::error(
                    "E205",
                    ty_name.span,
                    format!(
                        "`{}` is a {} — a `subdesign` use site requires a subdesign",
                        ty_name.name, sym.kind
                    ),
                ));
            }
            return;
        };
        if let Some(sel) = &stmt.ty.variant {
            self.diags.push(Diagnostic::error(
                "E1303",
                sel.span,
                format!(
                    "a subdesign has no variants — remove the `[{}]` selector",
                    sel.name
                ),
            ));
            return;
        }
        // Containment-cycle guard (the full-cycle diagnostic is declaration-
        // time in check::subdesigns; this stops runaway expansion the same
        // way `active_calls` does for fns, RFC-006 discipline).
        if self.active_subs.contains(&ty_name.name) {
            let mut chain: Vec<&str> = self
                .active_subs
                .iter()
                .skip_while(|n| **n != ty_name.name)
                .map(|s| s.as_str())
                .collect();
            chain.push(&ty_name.name);
            self.diags.push(Diagnostic::error(
                "E1304",
                stmt.span,
                format!(
                    "recursive subdesign containment: {}",
                    chain
                        .iter()
                        .map(|s| format!("`{}`", crate::resolve::short(s)))
                        .collect::<Vec<_>>()
                        .join(" → ")
                ),
            ));
            return;
        }
        if self.expression_failed(stmt.ty.span, scope)
            || sd
                .generics
                .iter()
                .any(|p| self.expression_failed(p.span, scope))
        {
            return;
        }
        // RFC-007 generics, reused verbatim.
        let subst = self.bound_generic_args(
            &format!("subdesign `{}`", crate::resolve::short(&ty_name.name)),
            &sd.generics,
            &stmt.ty.generic_args,
            scope,
            stmt.ty.span,
        );
        let node_work = 1 + sd.ports.len() as u64;
        if !self
            .meter
            .ensure_capacity(node_work, "subdesign node and ports", stmt.span, self.diags)
        {
            return;
        }
        let ports: BTreeMap<String, (Obligation, Span)> = sd
            .ports
            .iter()
            .map(|p| (p.name.name.clone(), (p.obligation, p.span)))
            .collect();
        let element_count = match &stmt.array_len {
            None => 1,
            Some((len_expr, _)) => {
                let Some(n) = scope.declared_lens.get(&stmt.name.name).copied() else {
                    return; // rejected (or unknown) in the shared dependency pass
                };
                scope
                    .arrays
                    .insert(stmt.name.name.clone(), (n, len_expr.span()));
                n
            }
        };
        let body = sd.body.clone();
        let fq = ty_name.name.clone();
        // Empty bodies have no intervening source sites. Preflight their
        // entire array before bulk allocation; nonempty bodies stay interleaved
        // per node so an earlier body failure retains its original site.
        if body.is_empty()
            && !self.meter.ensure_capacity(
                (element_count as u64).saturating_mul(node_work),
                "subdesign nodes and ports",
                stmt.span,
                self.diags,
            )
        {
            return;
        }
        for i in 0..element_count {
            if !self
                .meter
                .charge(node_work, "subdesign node and ports", stmt.span, self.diags)
            {
                return;
            }
            let elem = if stmt.array_len.is_some() {
                element_name(&stmt.name.name, i)
            } else {
                stmt.name.name.clone()
            };
            // RFC-024 discipline: each element is fully real, so its
            // generated name takes the SAME duplicate check a hand-written
            // declaration would (physical arrays get this via handle_inst).
            if scope.local_insts.contains_key(&elem)
                || scope.bindings.contains_key(&elem)
                || scope.local_subs.contains_key(&elem)
                || scope.arrays.contains_key(&elem)
            {
                self.diags.push(Diagnostic::error(
                    "E201",
                    stmt.name.span,
                    format!("`{}` is already defined in this scope", elem),
                ));
                continue;
            }
            let node_path = format!("{}::{}", scope.path, elem);
            scope.local_subs.insert(elem, node_path.clone());
            // Inside the body, every port is a pin OF THE NODE — the same
            // `Binding::Pin` a fn's `Pin` parameter uses, so all existing
            // reference machinery applies unchanged. The (node, port) members
            // this creates are phantoms: they merge nets (a port is an
            // equivalence-class join) and are stripped before the IR leaves
            // assembly — a node is never a manufacturable instance.
            let bindings: BTreeMap<String, Binding> = ports
                .keys()
                .map(|p| (p.clone(), Binding::Pin((node_path.clone(), p.clone()))))
                .collect();
            let mut inner = Scope {
                design_name: scope.design_name.clone(),
                path: node_path.clone(),
                is_design_body: false,
                place_ctx: PlaceCtx::Sub,
                layout_owner: Some(node_path.clone()),
                subst: subst.clone(),
                bindings,
                local_insts: BTreeMap::new(),
                local_subs: BTreeMap::new(),
                arrays: BTreeMap::new(),
                declared_lens: BTreeMap::new(),
                failed_expressions: Vec::new(),
                unknown_arrays: BTreeSet::new(),
                consts: BTreeMap::new(),
                binders: BTreeMap::new(),
                frame: Vec::new(),
                caller_frames: scope
                    .caller_frames
                    .iter()
                    .chain(&scope.frame)
                    .cloned()
                    .collect(),
            };
            self.active_subs.push(fq.clone());
            // The node exists BEFORE its body walks: an in-body reference
            // that lands on the node path (a port used as a physics target,
            // say) must identify it as a subdesign node, not fall through to
            // a generic "not physical" shape. Children fill in after.
            self.sub_nodes.insert(
                node_path.clone(),
                SubNode {
                    fq: fq.clone(),
                    use_span: stmt.span,
                    ports: ports.clone(),
                    children_insts: BTreeMap::new(),
                    children_subs: BTreeMap::new(),
                    children_arrays: BTreeMap::new(),
                },
            );
            self.walk_body(&body, &mut inner);
            self.active_subs.pop();
            let node = self
                .sub_nodes
                .get_mut(&node_path)
                .expect("stub inserted above");
            node.children_insts = inner.local_insts;
            node.children_subs = inner.local_subs;
            node.children_arrays = inner.arrays;
        }
    }

    /// Resolve a use site's inline port-connection block — pass 2 (its pin
    /// references may name instances declared later in the body). Each entry
    /// synthesizes one net declaration joining the port's phantom member with
    /// the target, and the ordinary shared-member merge does the rest.
    fn handle_subdesign_conns(&mut self, stmt: &SubdesignUseStmt, scope: &mut Scope) {
        if stmt.conns.is_empty() {
            return;
        }
        let Some(node_path) = scope.local_subs.get(&stmt.name.name).cloned() else {
            return; // the use site itself failed earlier (already reported)
        };
        let ports = self.sub_nodes[&node_path].ports.clone();
        let sub_short = crate::resolve::short(&self.sub_nodes[&node_path].fq).to_string();
        let mut seen: BTreeMap<&str, Span> = BTreeMap::new();
        for conn in &stmt.conns {
            // One authored entry plus its one scalar pin/net target. The
            // synthesized joining net below is deliberately not charged.
            if !self
                .meter
                .charge(2, "port connection", conn.span, self.diags)
            {
                return;
            }
            if let Some(prev) = seen.insert(conn.port.name.as_str(), conn.span) {
                self.diags.push(
                    Diagnostic::error(
                        "E1301",
                        conn.port.span,
                        format!("port `{}` is connected more than once", conn.port.name),
                    )
                    .with_secondary(prev, "first connected here".to_string()),
                );
                continue;
            }
            if !ports.contains_key(&conn.port.name) {
                self.diags.push(
                    Diagnostic::error(
                        "E1301",
                        conn.port.span,
                        format!(
                            "subdesign `{}` (use site `{}`) has no port named `{}`",
                            sub_short, stmt.name.name, conn.port.name
                        ),
                    )
                    .with_help(format!(
                        "its ports are: {}",
                        ports.keys().cloned().collect::<Vec<_>>().join(", ")
                    )),
                );
                continue;
            }
            let phantom = (node_path.clone(), conn.port.name.clone());
            let v = &conn.value;
            let is_bare_unknown = v.pin.is_none()
                && v.index.is_none()
                && !scope.bindings.contains_key(&v.base.name)
                && !scope.local_insts.contains_key(&v.base.name)
                && !scope.local_subs.contains_key(&v.base.name)
                && !scope.arrays.contains_key(&v.base.name);
            if is_bare_unknown {
                // A net name in the enclosing scope: join the port to that
                // named net's equivalence class by KEY (a later `net NAME:`
                // declaration merges by the same key). Whether the name is
                // ever really declared is validated at assembly.
                let key = if scope.is_design_body {
                    format!("named:{}", v.base.name)
                } else {
                    format!("scoped:{}::{}", scope.path, v.base.name)
                };
                self.synth_net_conns.push((key.clone(), v.base.clone()));
                let display_name = resolve_net_name(&v.base.name, scope);
                self.net_decls.push(NetDecl {
                    key,
                    display_name,
                    is_design_level_name: scope.is_design_body,
                    synthesized: true,
                    annotation: None,
                    members: vec![phantom],
                    span: conn.span,
                });
            } else {
                let Some(resolved) = self.resolve_pin_ref(v, scope) else {
                    continue;
                };
                let n = self.anon_net_counter;
                self.anon_net_counter += 1;
                let scoped = format!("{}::__net{}", scope.path, n);
                let display = scoped
                    .strip_prefix(&format!("{}::", scope.design_name))
                    .unwrap_or(&scoped)
                    .to_string();
                self.net_decls.push(NetDecl {
                    key: format!("scoped:{}", scoped),
                    display_name: display,
                    is_design_level_name: false,
                    synthesized: true,
                    annotation: None,
                    members: vec![phantom, resolved],
                    span: conn.span,
                });
            }
        }
    }

    /// Resolve a call argument that must be an instance (for a generic /
    /// `impl Trait` parameter) to (path, device name).
    fn resolve_instance_arg(&mut self, arg: &PinRef, scope: &Scope) -> Option<(String, String)> {
        if arg.pin.is_some() {
            self.diags.push(Diagnostic::error(
                "E503",
                arg.span,
                format!("expected an instance, found pin reference `{}`", arg),
            ));
            return None;
        }
        let arg = self.resolve_array_ref(arg, scope)?;
        if let Some(binding) = scope.bindings.get(&arg.base.name) {
            return match binding {
                Binding::Instance { path, device, .. } => Some((path.clone(), device.clone())),
                Binding::Pin(_) => {
                    self.diags.push(Diagnostic::error(
                        "E503",
                        arg.span,
                        format!(
                            "expected an instance, but `{}` is a `Pin` parameter",
                            arg.base.name
                        ),
                    ));
                    None
                }
            };
        }
        if let Some(path) = scope.local_insts.get(&arg.base.name) {
            let device = self.instances[path].device.clone();
            return Some((path.clone(), device));
        }
        self.diags.push(Diagnostic::error(
            "E202",
            arg.base.span,
            format!("unknown instance `{}` in this scope", arg.base.name),
        ));
        None
    }

    // -- assembly: merge nets, check exhaustiveness ---------------------------

    fn assemble(self, design: &DesignDef) -> DesignIr {
        // Union-find over net declarations: same design-level name → merged;
        // shared pin → merged.
        let n = self.net_decls.len();
        let mut parent: Vec<usize> = (0..n).collect();
        fn find(parent: &mut Vec<usize>, i: usize) -> usize {
            if parent[i] != i {
                let root = find(parent, parent[i]);
                parent[i] = root;
            }
            parent[i]
        }
        fn union(parent: &mut Vec<usize>, a: usize, b: usize) {
            let (ra, rb) = (find(parent, a), find(parent, b));
            if ra != rb {
                // Attach the larger index to the smaller for determinism.
                let (lo, hi) = if ra < rb { (ra, rb) } else { (rb, ra) };
                parent[hi] = lo;
            }
        }

        let mut by_key: BTreeMap<&str, usize> = BTreeMap::new();
        for (i, decl) in self.net_decls.iter().enumerate() {
            match by_key.get(decl.key.as_str()) {
                Some(&first) => union(&mut parent, first, i),
                None => {
                    by_key.insert(&decl.key, i);
                }
            }
        }
        let mut by_pin: BTreeMap<(String, String), usize> = BTreeMap::new();
        for (i, decl) in self.net_decls.iter().enumerate() {
            for m in &decl.members {
                match by_pin.get(m) {
                    Some(&first) => union(&mut parent, first, i),
                    None => {
                        by_pin.insert(m.clone(), i);
                    }
                }
            }
        }

        // Group declarations by root.
        let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for i in 0..n {
            let root = find(&mut parent, i);
            groups.entry(root).or_default().push(i);
        }

        // RFC-032: use-site port exhaustiveness — every REQUIRED port must
        // reach the world OUTSIDE its node (a member of its merged
        // equivalence class whose path is neither the node nor inside it).
        // Checked before phantom members are stripped below.
        for (node_path, node) in &self.sub_nodes {
            let local = crate::resolve::short(node_path);
            let inside = format!("{}::", node_path);
            for (port, (obligation, _)) in &node.ports {
                if *obligation != Obligation::Required {
                    continue;
                }
                let key = (node_path.clone(), port.clone());
                let connected = groups.values().any(|decl_idxs| {
                    decl_idxs
                        .iter()
                        .any(|&i| self.net_decls[i].members.contains(&key))
                        && decl_idxs
                            .iter()
                            .flat_map(|&i| &self.net_decls[i].members)
                            .any(|(p, _)| p != node_path && !p.starts_with(&inside))
                });
                if !connected {
                    self.diags.push(Diagnostic::error(
                        "E1302",
                        node.use_span,
                        format!(
                            "required port `{}` of `{}` (subdesign `{}`) is not connected — connect it in the use site's port block or a `net`",
                            port,
                            local,
                            crate::resolve::short(&node.fq)
                        ),
                    ));
                }
            }
        }

        let mut subdesigns: BTreeMap<String, crate::ir::IrSubdesign> = self
            .sub_nodes
            .iter()
            .map(|(path, node)| {
                let parent = path.rsplit_once("::").map(|(p, _)| p);
                (
                    path.clone(),
                    crate::ir::IrSubdesign {
                        definition: node.fq.clone(),
                        parent: parent
                            .filter(|p| self.sub_nodes.contains_key(*p))
                            .map(str::to_string),
                        span: node.use_span,
                        local_placements: Vec::new(),
                        ports: node
                            .ports
                            .iter()
                            .map(|(name, (obligation, _))| {
                                (
                                    name.clone(),
                                    crate::ir::IrSubdesignPort {
                                        obligation: *obligation,
                                        net: None,
                                        connected: false,
                                    },
                                )
                            })
                            .collect(),
                    },
                )
            })
            .collect();
        let mut nets = Vec::new();
        // RFC-013: every declared net name (including aliases merged into a
        // differently-named group) maps to that group's final name, so a layout
        // reference validates against *declared identity*, not the post-merge
        // name it may have lost to the naming race.
        let mut declared_to_merged: BTreeMap<String, String> = BTreeMap::new();
        for decl_idxs in groups.values() {
            let decls: Vec<&NetDecl> = decl_idxs.iter().map(|&i| &self.net_decls[i]).collect();
            // Name: smallest design-level name, else smallest scoped name.
            // RFC-032: declarations synthesized by port-connection blocks
            // never compete — an author-written name always beats a
            // compiler-generated `__netN` join.
            let name = decls
                .iter()
                .filter(|d| d.is_design_level_name && !d.synthesized)
                .map(|d| d.display_name.clone())
                .min()
                .or_else(|| {
                    decls
                        .iter()
                        .filter(|d| !d.synthesized)
                        .map(|d| d.display_name.clone())
                        .min()
                })
                .or_else(|| decls.iter().map(|d| d.display_name.clone()).min())
                .unwrap();
            for d in &decls {
                declared_to_merged.insert(d.display_name.clone(), name.clone());
            }
            // Retain boundary-to-net identity BEFORE stripping port phantoms.
            // This also preserves empty/pass-through logical subdesigns.
            for (path, port) in decls.iter().flat_map(|d| &d.members) {
                let Some(node) = subdesigns.get_mut(path) else {
                    continue;
                };
                let Some(port) = node.ports.get_mut(port) else {
                    continue;
                };
                let inside = format!("{path}::");
                port.net = Some(name.clone());
                port.connected = decls
                    .iter()
                    .flat_map(|d| &d.members)
                    .any(|(p, _)| p != path && !p.starts_with(&inside));
            }
            let members: BTreeSet<(String, String)> = decls
                .iter()
                .flat_map(|d| d.members.iter().cloned())
                // RFC-032: port phantoms did their job (merging the classes);
                // a subdesign node is never a manufacturable member.
                .filter(|(p, _)| !self.sub_nodes.contains_key(p))
                .collect();
            if members.is_empty() {
                // Every member failed to resolve (errors already reported), or
                // the class held only subdesign ports and dissolves with them.
                continue;
            }
            let span = decls
                .iter()
                .map(|d| d.span)
                .min_by_key(|s| (s.file, s.start))
                .unwrap();

            // Annotations: conflicting values are contradictory (E603).
            let mut voltage: Option<(UnitValue, Span)> = None;
            let mut gnd: Option<Span> = None;
            for d in &decls {
                match &d.annotation {
                    Some(NetAnnotation::Voltage(v, s)) => match &voltage {
                        Some((prev, prev_span)) if prev.femto != v.femto => {
                            self.diags.push(
                                Diagnostic::error(
                                    "E603",
                                    *s,
                                    format!(
                                        "net `{}` has contradictory voltage annotations: `{}` and `{}`",
                                        name, prev.text, v.text
                                    ),
                                )
                                .with_secondary(*prev_span, "the other annotation is here"),
                            );
                        }
                        Some(_) => {}
                        None => voltage = Some((v.clone(), *s)),
                    },
                    Some(NetAnnotation::Gnd(s)) => gnd = Some(*s),
                    None => {}
                }
            }
            if let (Some((v, vs)), Some(gs)) = (&voltage, &gnd) {
                self.diags.push(
                    Diagnostic::error(
                        "E603",
                        *vs,
                        format!(
                            "net `{}` is annotated both `[gnd]` and `[{}]` — a net cannot be both",
                            name, v.text
                        ),
                    )
                    .with_secondary(*gs, "the `[gnd]` annotation is here"),
                );
            }

            nets.push(IrNet {
                name,
                voltage: voltage.map(|(v, _)| v),
                is_gnd: gnd.is_some(),
                members,
                span,
            });
        }
        nets.sort_by(|a, b| a.name.cmp(&b.name));

        // RFC-032: a port connection written as a bare name must name a net
        // that really exists (a synthesized declaration may JOIN a named
        // net's class, never prove it).
        {
            let real_keys: BTreeSet<&str> = self
                .net_decls
                .iter()
                .filter(|d| !d.synthesized)
                .map(|d| d.key.as_str())
                .collect();
            let mut reported: BTreeSet<(&str, u32, u32)> = BTreeSet::new();
            for (key, ident) in &self.synth_net_conns {
                if !real_keys.contains(key.as_str())
                    && reported.insert((key.as_str(), ident.span.file.0, ident.span.start))
                {
                    self.diags.push(Diagnostic::error(
                        "E1303",
                        ident.span,
                        format!(
                            "`{}` is not a declared net, an instance pin, or a port in this scope — a port connects to a net name or an `INST.PIN`",
                            ident.name
                        ),
                    ));
                }
            }
        }

        let nc_pins: BTreeSet<(String, String)> =
            self.nc_pins.iter().map(|(p, _)| p.clone()).collect();

        // RFC-013: validate layout constraints against declared-net identity
        // and build the (connectivity-independent) layout IR.
        let mut layout = build_layout_ir(&self.layout_raw, &declared_to_merged, self.diags);
        // The board outline + locked placements (validated on collection) ride
        // the same IR.
        layout.board_outline = self.board_outline;
        layout.placements = self.placements;

        // RFC-032: transform subdesign default layouts onto the board.
        //
        // 1. Anchor every node: a design-level whole-unit `place` (absolute)
        //    wins; else the node's placement in an ancestor's layout composes
        //    with that ancestor's own anchor — the entry from the OUTERMOST
        //    layout wins ("explicit beats the subdesign's own default"). An
        //    unanchored node contributes no default positions at all: its
        //    internals stay unplaced (staged) unless individually overridden.
        // 2. Give every internally-placed real instance its composed default,
        //    unless an explicit (design-level, possibly reached-in) placement
        //    exists for it. Explicit placements keep their declaration order
        //    (byte-stability for pre-RFC-032 designs); composed defaults
        //    append after, in path order.
        {
            let mut node_paths: Vec<String> = self.sub_nodes.keys().cloned().collect();
            node_paths.sort_by_key(|p| (p.matches("::").count(), p.clone()));
            let explicit: BTreeSet<&str> =
                layout.placements.iter().map(|p| p.path.as_str()).collect();
            let defaults: Vec<_> =
                default_placements(&node_paths, &self.rel_places, self.abs_node_places)
                    .into_iter()
                    .filter(|p| !explicit.contains(p.path.as_str()))
                    .collect();
            layout.placements.extend(defaults);

            // Preserve each subdesign's authored layout for tooling even if
            // it has no board anchor. The SAME precedence/composition pass
            // runs from that scope's identity frame; outer overrides never
            // enter it, and these rows never enter the manufacturing layout.
            for (path, node) in &mut subdesigns {
                let origin = PlaceData {
                    at: (length_value(0), length_value(0)),
                    rotate: 0,
                    side: PlacementSide::Top,
                    span: node.span,
                };
                node.local_placements = default_placements(
                    &node_paths,
                    &self.rel_places,
                    BTreeMap::from([(path.clone(), origin)]),
                );
            }
        }

        // RFC-027: validate + adopt the physics-constraint facts. At most one
        // primary ground per design; at most one attribute of each kind per
        // (merged) net — two source declarations of one net may not both carry
        // the same kind.
        {
            let mut primary_span: Option<Span> = None;
            let mut seen_ground: BTreeMap<String, Span> = BTreeMap::new();
            for (g, span) in &self.phys_grounds {
                if let Some(prev) = seen_ground.insert(g.net.clone(), *span) {
                    self.diags.push(
                        Diagnostic::error(
                            "E1009",
                            *span,
                            format!("net `{}` carries `#[ground]` more than once", g.net),
                        )
                        .with_secondary(prev, "first written here".to_string()),
                    );
                    continue;
                }
                if g.primary {
                    if let Some(prev) = primary_span {
                        self.diags.push(
                            Diagnostic::error(
                                "E1009",
                                *span,
                                "a design has at most one `#[ground(primary)]` net".to_string(),
                            )
                            .with_secondary(prev, "the first primary ground".to_string()),
                        );
                        continue;
                    }
                    primary_span = Some(*span);
                }
                layout.grounds.push(g.clone());
            }
            let mut seen: BTreeMap<(&str, String), Span> = BTreeMap::new();
            for (h, span) in &self.phys_high_currents {
                if let Some(prev) = seen.insert(("hc", h.net.clone()), *span) {
                    self.diags.push(
                        Diagnostic::error(
                            "E1009",
                            *span,
                            format!("net `{}` carries `#[high_current]` more than once", h.net),
                        )
                        .with_secondary(prev, "first written here".to_string()),
                    );
                    continue;
                }
                layout.high_currents.push(h.clone());
            }
            for (i, span) in &self.phys_impedances {
                if let Some(prev) = seen.insert(("imp", i.net.clone()), *span) {
                    self.diags.push(
                        Diagnostic::error(
                            "E1009",
                            *span,
                            format!("net `{}` carries `#[impedance]` more than once", i.net),
                        )
                        .with_secondary(prev, "first written here".to_string()),
                    );
                    continue;
                }
                layout.impedances.push(i.clone());
            }
            layout.bypasses = self.phys_bypasses;
            layout.crystals = self.phys_crystals;
            layout.converters = self.phys_converters;
            layout.bga_fanouts = self.phys_bga;
        }

        let ir = DesignIr {
            name: design.name.name.clone(),
            instances: self.instances,
            subdesigns,
            nets,
            nc_pins,
            layout,
        };

        // RFC-002: pin connection-obligation exhaustiveness, once, at final
        // design assembly, after all inlining/monomorphization.
        check_pin_obligations(self.world, &ir, self.diags);
        ir
    }
}

/// Resolve defaults reachable from the supplied coordinate-frame anchors.
/// Node paths must be sorted by depth then path. A shallower anchored owner
/// wins over an inner default; an unanchored owner cannot shadow one.
fn default_placements(
    node_paths: &[String],
    relative: &[RelPlace],
    mut anchors: BTreeMap<String, PlaceData>,
) -> Vec<crate::ir::LayoutPlacement> {
    let depth = |p: &str| p.matches("::").count();
    // Each frame visits only a node's own candidates, rather than scanning
    // all relative placements for every node (notably with array use sites).
    let mut node_defaults: BTreeMap<&str, Vec<(usize, &RelPlace)>> = BTreeMap::new();
    for (i, r) in relative.iter().enumerate() {
        if let PlaceTarget::Node(path) = &r.target {
            node_defaults.entry(path).or_default().push((i, r));
        }
    }
    for np in node_paths {
        if anchors.contains_key(np) {
            continue;
        }
        let Some(candidates) = node_defaults.get(np.as_str()) else {
            continue;
        };
        let best = candidates
            .iter()
            .filter(|(_, r)| anchors.contains_key(&r.owner))
            .min_by_key(|(i, r)| (depth(&r.owner), *i));
        if let Some((_, r)) = best {
            anchors.insert(np.clone(), compose_place(&anchors[&r.owner], &r.data));
        }
    }
    let mut best_inst: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for (i, r) in relative.iter().enumerate() {
        if let PlaceTarget::Inst(p) = &r.target {
            if !anchors.contains_key(&r.owner) {
                continue;
            }
            let key = (depth(&r.owner), i);
            let slot = best_inst.entry(p.clone()).or_insert(key);
            if key < *slot {
                *slot = key;
            }
        }
    }
    best_inst
        .into_iter()
        .map(|(path, (_, i))| {
            let r = &relative[i];
            let d = compose_place(&anchors[&r.owner], &r.data);
            crate::ir::LayoutPlacement {
                path,
                at: d.at,
                rotate: d.rotate,
                side: d.side,
            }
        })
        .collect()
}

/// RFC-032: transform a child placement, relative to a subdesign's origin,
/// through the subdesign's own anchor placement — the identical geometric
/// operation RFC-025/026 already define for a pad inside a placed footprint:
/// rotate about the anchor; on the back side, mirror x BEFORE rotating, flip
/// the side, and REVERSE the child's own rotation (a reflection).
///
/// The authoring frame is +y-down (KiCad's board frame); `trig::rotate` is
/// counter-clockwise in the +y-up IPC frame, so the board-frame rotation is
/// its inverse angle. Exact fixed-point arithmetic throughout — placement
/// coordinates are byte-stability-critical (never `f64::sin`/`cos`).
fn compose_place(parent: &PlaceData, child: &PlaceData) -> PlaceData {
    use crate::ast::PlacementSide;
    let bottom = parent.side == PlacementSide::Bottom;
    let (dx, dy) = (child.at.0.femto, child.at.1.femto);
    let dx = if bottom { -dx } else { dx };
    let inv = ((360 - (parent.rotate as u32 % 360)) % 360) as u16;
    let (rx, ry) = crate::trig::rotate(dx, dy, inv);
    let x = parent.at.0.femto + rx;
    let y = parent.at.1.femto + ry;
    let rotate = if bottom {
        ((parent.rotate as u32 + 360 - child.rotate as u32) % 360) as u16
    } else {
        ((parent.rotate as u32 + child.rotate as u32) % 360) as u16
    };
    let side = match (bottom, child.side) {
        (false, s) => s,
        (true, PlacementSide::Top) => PlacementSide::Bottom,
        (true, PlacementSide::Bottom) => PlacementSide::Top,
    };
    PlaceData {
        at: (length_value(x), length_value(y)),
        rotate,
        side,
        span: child.span,
    }
}

/// A computed `Length` value: exact femto-mm integer, canonical mm text.
fn length_value(femto: i128) -> UnitValue {
    UnitValue {
        unit: crate::units::UnitType::Length,
        femto,
        text: format!("{}mm", crate::emit::geom::mm_femto(femto)),
    }
}

/// The IR net name a source net name resolves to in `scope` — identical to the
/// naming rule `handle_net` applies to a named net (RFC-013).
fn resolve_net_name(name: &str, scope: &Scope) -> String {
    if scope.is_design_body {
        name.to_string()
    } else {
        let scoped = format!("{}::{}", scope.path, name);
        scoped
            .strip_prefix(&format!("{}::", scope.design_name))
            .unwrap_or(&scoped)
            .to_string()
    }
}

/// RFC-013: validate layout constraints against their own closed vocabulary and
/// build the resolved layout IR. Every check here is structural (net existence,
/// arity, name uniqueness) — none touches the connectivity graph's emergent
/// properties, and none can alter the netlist. Net references are validated
/// against declared identity (`declared_to_merged` maps every declared net name
/// to its final merged name) and rewritten to the merged name the `.net` file
/// uses, so `layout.json` stays consistent with the netlist.
fn build_layout_ir(
    raw: &[RawLayout],
    declared_to_merged: &BTreeMap<String, String>,
    diags: &mut Diagnostics,
) -> LayoutIr {
    let mut layout = LayoutIr::default();
    let mut seen_classes: BTreeSet<String> = BTreeSet::new();
    for c in raw {
        match c {
            RawLayout::NetClass {
                name,
                scoped_name,
                nets,
            } => {
                let (mapped, _) = map_layout_nets(nets, declared_to_merged, diags);
                if !seen_classes.insert(scoped_name.clone()) {
                    // Report the SOURCE name (with its scope for fn-local
                    // classes) — never the raw mangled identity.
                    let msg = match scoped_name.strip_suffix(&format!("::{}", name.name)) {
                        Some(prefix) => format!(
                            "duplicate `net_class` name `{}` (in `{}`)",
                            name.name, prefix
                        ),
                        None => format!("duplicate `net_class` name `{}`", name.name),
                    };
                    diags.push(Diagnostic::error("E1002", name.span, msg));
                }
                layout.net_classes.push(LayoutNetClass {
                    name: scoped_name.clone(),
                    nets: dedup_in_order(mapped),
                });
            }
            RawLayout::DiffPair {
                nets,
                differential_impedance,
                single_ended_impedance,
                frequency,
                span,
            } => {
                let (mapped, all_known) = map_layout_nets(nets, declared_to_merged, diags);
                if nets.len() != 2 {
                    diags.push(Diagnostic::error(
                        "E1003",
                        *span,
                        format!(
                            "`diff_pair` must name exactly two nets, found {}",
                            nets.len()
                        ),
                    ));
                } else if all_known && mapped[0] == mapped[1] {
                    // Two source names may be aliases of one electrical net
                    // (shared pin merge) — a pair needs two DISTINCT nets.
                    diags.push(Diagnostic::error(
                        "E1003",
                        *span,
                        format!(
                            "`diff_pair` must name two distinct nets — `{}` and `{}` resolve to the same net `{}`",
                            nets[0].1.name, nets[1].1.name, mapped[0]
                        ),
                    ));
                } else {
                    layout.diff_pairs.push(LayoutDiffPair {
                        p: mapped[0].clone(),
                        n: mapped[1].clone(),
                        differential_impedance: differential_impedance.clone(),
                        single_ended_impedance: single_ended_impedance.clone(),
                        frequency: frequency.clone(),
                    });
                }
            }
            RawLayout::LengthMatch {
                nets,
                tolerance,
                span,
            } => {
                let (mapped, all_known) = map_layout_nets(nets, declared_to_merged, diags);
                let distinct: BTreeSet<&String> = mapped.iter().collect();
                if nets.len() < 2 {
                    diags.push(Diagnostic::error(
                        "E1004",
                        *span,
                        format!(
                            "`length_match` must name at least two nets, found {}",
                            nets.len()
                        ),
                    ));
                } else if all_known && distinct.len() < 2 {
                    diags.push(Diagnostic::error(
                        "E1004",
                        *span,
                        format!(
                            "`length_match` must name at least two distinct nets — all references resolve to the same net `{}`",
                            mapped[0]
                        ),
                    ));
                } else {
                    layout.length_matches.push(LayoutLengthMatch {
                        // Aliases of one merged net collapse to a single
                        // entry, first-occurrence order (the artifact
                        // advertises distinct nets).
                        nets: dedup_in_order(mapped),
                        tolerance: tolerance.clone(),
                    });
                }
            }
        }
    }
    layout
}

/// Resolve each layout net reference to its merged IR net name, emitting E1001
/// for any reference to a net that was never declared (in the applicable
/// design/fn scope). The second return is `true` only when every reference
/// resolved — distinctness checks are skipped otherwise (no error cascades).
fn map_layout_nets(
    nets: &[(String, Ident)],
    declared_to_merged: &BTreeMap<String, String>,
    diags: &mut Diagnostics,
) -> (Vec<String>, bool) {
    let mut all_known = true;
    let mapped = nets
        .iter()
        .map(|(resolved, orig)| match declared_to_merged.get(resolved) {
            Some(merged) => merged.clone(),
            None => {
                all_known = false;
                diags.push(Diagnostic::error(
                    "E1001",
                    orig.span,
                    format!(
                        "unknown net `{}` in a layout constraint — no such net is declared in this design",
                        orig.name
                    ),
                ));
                resolved.clone()
            }
        })
        .collect();
    (mapped, all_known)
}

/// Deduplicate, preserving first-occurrence order (never sort — the artifact's
/// determinism comes from source order).
/// RFC-033 §6: a frame value's path spelling — non-negative in decimal,
/// negative as `neg{abs}` (`__` names stay reserved).
fn frame_value_text(v: i64) -> String {
    if v >= 0 {
        v.to_string()
    } else {
        format!("neg{}", v.unsigned_abs())
    }
}

fn dedup_in_order(nets: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    nets.into_iter()
        .filter(|n| seen.insert(n.clone()))
        .collect()
}

/// RFC-002 exhaustiveness: every `required` pin of every instance appears in
/// exactly one of {some net, the nc set}.
fn check_pin_obligations(world: &World, ir: &DesignIr, diags: &mut Diagnostics) {
    let mut connected: BTreeMap<&(String, String), &str> = BTreeMap::new();
    for net in &ir.nets {
        for m in &net.members {
            connected.entry(m).or_insert(net.name.as_str());
        }
    }
    for inst in ir.instances.values() {
        let device = &world.devices[&inst.device];
        for pin in device.pins_for(inst.variant.as_deref()) {
            let key = (inst.path.clone(), pin.name.name.clone());
            let in_net = connected.get(&key).copied();
            let in_nc = ir.nc_pins.contains(&key);
            match (pin.obligation, in_net, in_nc) {
                (Obligation::Required, None, false) => {
                    diags.push(
                        Diagnostic::error(
                            "E701",
                            inst.span,
                            format!(
                                "required pin `{}.{}` is unresolved: add it to a `net` or explicitly mark it `nc`",
                                inst.path, pin.name.name
                            ),
                        )
                        .with_secondary(pin.span, format!(
                            "`{}` is declared `required` on device `{}` here",
                            pin.name.name, crate::resolve::short(&inst.device)
                        )),
                    );
                }
                (_, Some(net_name), true) => {
                    diags.push(Diagnostic::error(
                        "E702",
                        inst.span,
                        format!(
                            "pin `{}.{}` is contradictory: it appears in net `{}` AND in an `nc` declaration — a pin cannot be both connected and explicitly not-connected",
                            inst.path, pin.name.name, net_name
                        ),
                    ));
                }
                _ => {}
            }
        }
    }
}

fn is_valid_designator(s: &str) -> bool {
    let prefix_len = s.chars().take_while(|c| c.is_ascii_uppercase()).count();
    prefix_len > 0
        && s.len() > prefix_len
        && s[prefix_len..].chars().all(|c| c.is_ascii_digit())
        && !s[prefix_len..].starts_with('0')
}

#[cfg(test)]
mod budget_tests {
    use super::*;
    use crate::check::meter::MAX_WORK_ITEMS;

    const LIB: &str = "pub device D { pins { A: 1 [passive], B: 2 [passive] } }";

    // Exercise the real expansion handlers close to the production limit,
    // without allocating a million objects to prove each semantic charge.
    fn metered(src: &str, initial_work: u64) -> (u64, usize, String) {
        let checked = crate::pipeline::check_files_in(
            "board",
            &[("main.cohdl".into(), format!("{LIB} {src}"))],
            None,
        )
        .unwrap();
        assert!(
            !checked.diags.has_errors(),
            "{}",
            checked.diags.render(&checked.sm)
        );
        let design = checked.world.designs.values().next().unwrap();
        let mut diags = Diagnostics::new();
        let mut ex = Expander {
            world: &checked.world,
            definition_failures: Vec::new(),
            diags: &mut diags,
            instances: BTreeMap::new(),
            net_decls: Vec::new(),
            nc_pins: Vec::new(),
            layout_raw: Vec::new(),
            board_outline: None,
            placements: Vec::new(),
            sub_nodes: BTreeMap::new(),
            active_subs: Vec::new(),
            abs_node_places: BTreeMap::new(),
            reported_place_conflicts: BTreeSet::new(),
            rel_places: Vec::new(),
            synth_net_conns: Vec::new(),
            phys_grounds: Vec::new(),
            phys_high_currents: Vec::new(),
            phys_impedances: Vec::new(),
            phys_bypasses: Vec::new(),
            phys_crystals: Vec::new(),
            phys_converters: Vec::new(),
            phys_bga: Vec::new(),
            active_calls: Vec::new(),
            call_counter: 0,
            anon_net_counter: 0,
            meter: crate::check::meter::Meter::new(true),
        };
        let mut scope = Scope {
            design_name: design.name.name.clone(),
            path: design.name.name.clone(),
            is_design_body: true,
            place_ctx: PlaceCtx::Design,
            layout_owner: None,
            subst: Substitution::new(),
            bindings: BTreeMap::new(),
            local_insts: BTreeMap::new(),
            local_subs: BTreeMap::new(),
            arrays: BTreeMap::new(),
            declared_lens: BTreeMap::new(),
            failed_expressions: Vec::new(),
            unknown_arrays: BTreeSet::new(),
            consts: BTreeMap::new(),
            binders: BTreeMap::new(),
            caller_frames: Vec::new(),
            frame: Vec::new(),
        };

        ex.meter.work = initial_work;
        ex.walk_body(&design.body, &mut scope);
        let result = (ex.meter.work, ex.sub_nodes.len());
        (result.0, result.1, diags.render(&checked.sm))
    }

    #[test]
    fn logical_nodes_and_optional_ports_are_charged_before_materialization() {
        let source = "pub subdesign S { ports { optional P: Pin optional Q: Pin } }
            design B { subdesign a: S subdesign b: S }";
        let (work, nodes, diagnostics) = metered(source, MAX_WORK_ITEMS - 5);
        assert_eq!(nodes, 1, "{diagnostics}");
        assert_eq!(work, MAX_WORK_ITEMS - 2);
        assert_eq!(
            diagnostics.matches("error[E1405]").count(),
            1,
            "{diagnostics}"
        );
        let (_, nodes, diagnostics) = metered(
            "pub subdesign S {} design B { subdesign a: S subdesign b: S }",
            MAX_WORK_ITEMS - 1,
        );
        assert_eq!(nodes, 1, "{diagnostics}");
        assert_eq!(
            diagnostics.matches("error[E1405]").count(),
            1,
            "{diagnostics}"
        );
        let (work, nodes, diagnostics) = metered(
            "pub subdesign S { ports { optional P: Pin optional Q: Pin } }
            design B { subdesign a: S }",
            MAX_WORK_ITEMS - 3,
        );
        assert_eq!((work, nodes), (MAX_WORK_ITEMS, 1));
        assert!(diagnostics.is_empty(), "{diagnostics}");
    }

    #[test]
    fn array_preflight_preserves_first_failing_body_site() {
        let (work, nodes, diagnostics) = metered(
            "pub subdesign S { ports { optional P: Pin } net inside: P }
            design B { subdesign a: [S; 3] }",
            MAX_WORK_ITEMS - 2,
        );
        assert_eq!((work, nodes), (MAX_WORK_ITEMS, 1));
        assert_eq!(
            diagnostics.matches("error[E1405]").count(),
            1,
            "{diagnostics}"
        );
        assert!(diagnostics.contains("`net` would exceed"), "{diagnostics}");
    }

    #[test]
    fn list_fanout_charges_repeated_members_before_materialization() {
        let (work, _, diagnostics) = metered(
            "design B { inst a: [D; 1] net _: a[0, 0, 0].A nc: a[0].B }",
            MAX_WORK_ITEMS - 4,
        );
        assert_eq!(work, MAX_WORK_ITEMS - 2);
        assert_eq!(
            diagnostics.matches("error[E1405]").count(),
            1,
            "{diagnostics}"
        );
        assert!(
            diagnostics.contains("net members would exceed"),
            "{diagnostics}"
        );
    }

    #[test]
    fn authored_port_connections_charge_entry_and_target_without_synthesized_net() {
        let (work, _, diagnostics) = metered(
            "pub subdesign S { ports { optional P: Pin optional Q: Pin } }
            design B { inst a: D subdesign s: S { P: a.A Q: named } net named: a.B }",
            0,
        );
        assert!(diagnostics.is_empty(), "{diagnostics}");
        // Instance + node and two ports + two (entry + target) + net and member.
        assert_eq!(work, 1 + 3 + 4 + 2);
    }

    #[test]
    fn layout_constraints_charge_declaration_references_and_physics_bracket() {
        let (work, _, diagnostics) = metered(
            "design B { inst a: D net P: a.A net N: a.B
            layout { net_class C { P, N } length_match(P, N)
                diff_pair(P, N) [differential_impedance: 100ohm] } }",
            0,
        );
        assert!(diagnostics.is_empty(), "{diagnostics}");
        // One instance, two net/member pairs, three constraints with two refs,
        // and one resolved diff-pair physics bracket.
        assert_eq!(work, 1 + 4 + 9 + 1);
    }

    #[test]
    fn physics_records_and_authored_targets_are_charged() {
        let (work, _, diagnostics) = metered(
            "design B { #[bga_fanout] inst a: D
            #[bypass(a.A, 100nF)] inst b: D
            #[ground(primary)] net G: a.A, a.B, b.A, b.B }",
            0,
        );
        assert!(diagnostics.is_empty(), "{diagnostics}");
        // Two instances; bare BGA record; bypass record and explicit pin target;
        // net statement and four members; ground record (no explicit targets).
        assert_eq!(work, 2 + 1 + 2 + 5 + 1);
    }
}
