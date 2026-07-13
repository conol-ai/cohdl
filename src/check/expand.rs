//! Design expansion: monomorphize and inline every fn call (RFC-006), resolve
//! every instance/net/nc, merge nets, and run the pin connection-obligation
//! exhaustiveness check (RFC-002) on the fully-assembled design.
//!
//! Body semantics are declarative: within one body, all `inst` statements are
//! processed first, then `net`/`nc`/calls in source order — so a `net` may
//! reference an instance declared later in the same body.

use crate::ast::*;
use crate::check::generics::{resolve_generic_args, GenericValue, Substitution};
use crate::diag::{Diagnostic, Diagnostics};
use crate::ir::{DesignIr, IrInstance, IrNet};
use crate::resolve::World;
use crate::span::Span;
use crate::units::UnitValue;
use std::collections::{BTreeMap, BTreeSet};

pub fn expand_design(world: &World, design: &DesignDef, diags: &mut Diagnostics) -> DesignIr {
    let mut ex = Expander {
        world,
        diags,
        instances: BTreeMap::new(),
        net_decls: Vec::new(),
        nc_pins: Vec::new(),
        active_calls: Vec::new(),
        call_counter: 0,
        anon_net_counter: 0,
    };
    let mut scope = Scope {
        design_name: design.name.name.clone(),
        path: design.name.name.clone(),
        is_design_body: true,
        subst: Substitution::new(),
        bindings: BTreeMap::new(),
        local_insts: BTreeMap::new(),
    };
    ex.walk_body(&design.body, &mut scope);
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

struct Scope {
    design_name: String,
    path: String,
    is_design_body: bool,
    subst: Substitution,
    bindings: BTreeMap<String, Binding>,
    /// local instance name → full path.
    local_insts: BTreeMap<String, String>,
}

/// One `net` declaration with resolved members, pre-merge.
struct NetDecl {
    /// Merge key: design-level names merge by name; fn-scoped and anonymous
    /// nets are unique per declaration.
    key: String,
    /// Candidate emitted name.
    display_name: String,
    is_design_level_name: bool,
    annotation: Option<NetAnnotation>,
    members: Vec<(String, String)>,
    span: Span,
}

struct Expander<'w, 'd> {
    world: &'w World,
    diags: &'d mut Diagnostics,
    instances: BTreeMap<String, IrInstance>,
    net_decls: Vec<NetDecl>,
    nc_pins: Vec<((String, String), Span)>,
    /// fn names currently being expanded (cycle detection, RFC-006).
    active_calls: Vec<String>,
    /// Global (per-design) call counter — `__fn{N}_{name}` segments.
    call_counter: usize,
    anon_net_counter: usize,
}

impl<'w, 'd> Expander<'w, 'd> {
    fn walk_body(&mut self, body: &[Stmt], scope: &mut Scope) {
        // Pass 1: instances (declarative bodies — nets may reference later insts).
        for stmt in body {
            if let Stmt::Inst(inst) = stmt {
                self.handle_inst(inst, scope);
            }
        }
        // Pass 2: everything else, in source order.
        for stmt in body {
            match stmt {
                Stmt::Inst(_) => {}
                Stmt::Net(net) => self.handle_net(net, scope),
                Stmt::Nc(nc) => self.handle_nc(nc, scope),
                Stmt::Call(call) => self.handle_call(call, scope),
            }
        }
    }

    // -- instances -----------------------------------------------------------

    fn handle_inst(&mut self, inst: &InstStmt, scope: &mut Scope) {
        if scope.local_insts.contains_key(&inst.name.name)
            || scope.bindings.contains_key(&inst.name.name)
        {
            self.diags.push(Diagnostic::error(
                "E201",
                inst.name.span,
                format!("`{}` is already defined in this scope", inst.name.name),
            ));
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
            (
                dev.name.name.clone(),
                args,
                Some(part_def.name.name.clone()),
            )
        } else if let Some(dev) = self.world.devices.get(&ty_name.name) {
            let args = resolve_generic_args(
                self.world,
                &format!("device `{}`", dev.name.name),
                &dev.generics,
                &inst.ty.generic_args,
                &scope.subst,
                inst.ty.span,
                self.diags,
            );
            (dev.name.name.clone(), args, None)
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
            self.diags.push(Diagnostic::error(
                "E202",
                ty_name.span,
                format!("unknown device or part `{}`", ty_name.name),
            ));
            return;
        };

        let device = &self.world.devices[&device_name];

        // Concrete spec values via substitution.
        let mut specs = BTreeMap::new();
        for field in &device.specs {
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
                        "unrecognized attribute `{}` (only `#[designator(\"…\")]` is supported)",
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
                specs,
                part,
                designator_override,
                designator: None,
                impl_traits: self.world.implemented_traits(&device_name),
                span: inst.span,
            },
        );
    }

    // -- pin references ------------------------------------------------------

    /// Resolve a pin reference to (instance path, logical pin name).
    fn resolve_pin_ref(&mut self, r: &PinRef, scope: &Scope) -> Option<(String, String)> {
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
        // Base: a local instance?
        if let Some(path) = scope.local_insts.get(&r.base.name) {
            let inst = &self.instances[path];
            let device = &self.world.devices[&inst.device];
            let Some(pin) = &r.pin else {
                self.diags.push(Diagnostic::error(
                    "E602",
                    r.span,
                    format!(
                        "`{}` is an instance — reference one of its pins (e.g. `{}.{}`)",
                        r.base.name,
                        r.base.name,
                        device
                            .pins
                            .first()
                            .map(|p| p.name.name.as_str())
                            .unwrap_or("PIN")
                    ),
                ));
                return None;
            };
            if device.pins.iter().any(|p| p.name.name == pin.name) {
                return Some((path.clone(), pin.name.clone()));
            }
            self.diags.push(
                Diagnostic::error(
                    "E203",
                    pin.span,
                    format!(
                        "device `{}` (instance `{}`) has no pin named `{}`",
                        inst.device, r.base.name, pin.name
                    ),
                )
                .with_help(format!(
                    "its pins are: {}",
                    device
                        .pins
                        .iter()
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

    fn handle_net(&mut self, net: &NetStmt, scope: &mut Scope) {
        let mut members = Vec::new();
        for m in &net.members {
            if let Some(resolved) = self.resolve_pin_ref(m, scope) {
                members.push(resolved);
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
        self.net_decls.push(NetDecl {
            key,
            display_name,
            is_design_level_name,
            annotation: net.annotation.clone(),
            members,
            span: net.span,
        });
    }

    fn handle_nc(&mut self, nc: &NcStmt, scope: &mut Scope) {
        for m in &nc.members {
            if let Some(resolved) = self.resolve_pin_ref(m, scope) {
                self.nc_pins.push((resolved, m.span));
            }
        }
    }

    // -- calls (RFC-006) -----------------------------------------------------

    fn handle_call(&mut self, call: &CallStmt, scope: &mut Scope) {
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
                Diagnostic::error(
                    "E504",
                    call.callee.span,
                    format!("unknown fn `{}`", call.callee.name),
                )
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
                    call.callee.name,
                    chain.join(" → ")
                ),
            ));
            return;
        }

        // Named generic parameters come from the turbofish, resolved in the
        // CALLER's substitution (outward-in threading, RFC-006).
        let subst = resolve_generic_args(
            self.world,
            &format!("fn `{}`", fndef.name.name),
            &fndef.generics,
            &call.generic_args,
            &scope.subst,
            call.span,
            self.diags,
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
                    // (RFC-007): check the bounds directly against the
                    // argument's concrete device.
                    match self.resolve_instance_arg(arg, scope) {
                        Some((path, device)) => {
                            let mut ok = true;
                            for bt in bound_traits {
                                if !self.world.has_impl(&bt.name, &device) {
                                    self.diags.push(
                                        Diagnostic::error(
                                            "E403",
                                            arg.span,
                                            format!(
                                                "`{}` does not implement `{}`, required by parameter `{}` of fn `{}`",
                                                device, bt.name, param.name.name, fndef.name.name
                                            ),
                                        )
                                        .with_help(format!(
                                            "add `impl {} for {} {{ … }}`, or pass a device that has one",
                                            bt.name, device
                                        )),
                                    );
                                    ok = false;
                                }
                            }
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
        self.call_counter += 1;
        let mut inner = Scope {
            design_name: scope.design_name.clone(),
            path: format!("{}::{}", scope.path, seg),
            is_design_body: false,
            subst,
            bindings,
            local_insts: BTreeMap::new(),
        };
        self.active_calls.push(call.callee.name.clone());
        // Clone the body to release the borrow on `self.world`.
        let body = fndef.body.clone();
        self.walk_body(&body, &mut inner);
        self.active_calls.pop();
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

        let mut nets = Vec::new();
        for decl_idxs in groups.values() {
            let decls: Vec<&NetDecl> = decl_idxs.iter().map(|&i| &self.net_decls[i]).collect();
            let members: BTreeSet<(String, String)> = decls
                .iter()
                .flat_map(|d| d.members.iter().cloned())
                .collect();
            if members.is_empty() {
                // Every member failed to resolve — errors already reported;
                // (a zero-member net is unrepresentable in the grammar).
                continue;
            }
            // Name: smallest design-level name, else smallest scoped name.
            let name = decls
                .iter()
                .filter(|d| d.is_design_level_name)
                .map(|d| d.display_name.clone())
                .min()
                .or_else(|| decls.iter().map(|d| d.display_name.clone()).min())
                .unwrap();
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

        let nc_pins: BTreeSet<(String, String)> =
            self.nc_pins.iter().map(|(p, _)| p.clone()).collect();

        let ir = DesignIr {
            name: design.name.name.clone(),
            instances: self.instances,
            nets,
            nc_pins,
        };

        // RFC-002: pin connection-obligation exhaustiveness, once, at final
        // design assembly, after all inlining/monomorphization.
        check_pin_obligations(self.world, &ir, self.diags);
        ir
    }
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
        for pin in &device.pins {
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
                            pin.name.name, inst.device
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
