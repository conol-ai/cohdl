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
use crate::ir::{
    DesignIr, IrInstance, IrNet, LayoutDiffPair, LayoutIr, LayoutLengthMatch, LayoutNetClass,
};
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
        layout_raw: Vec::new(),
        board_outline: None,
        placements: Vec::new(),
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
    };
    let mut scope = Scope {
        design_name: design.name.name.clone(),
        path: design.name.name.clone(),
        is_design_body: true,
        subst: Substitution::new(),
        bindings: BTreeMap::new(),
        local_insts: BTreeMap::new(),
        arrays: BTreeMap::new(),
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
    /// RFC-024: array-typed instance name → (declared length, decl span).
    /// An array's NAME is never itself in `local_insts` — only its elements
    /// (`NAME_0`…`NAME_{N-1}`), so a bare unindexed reference cannot resolve.
    arrays: BTreeMap<String, (i64, crate::span::Span)>,
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
        // Pass 1: instances (declarative bodies — nets may reference later insts).
        for stmt in body {
            if let Stmt::Inst(inst) = stmt {
                match inst.array_len {
                    None => self.handle_inst(inst, scope),
                    // RFC-024: `inst NAME: [Device; N]` is ONE array-typed
                    // instance whose N elements are each fully real. Each
                    // element goes through the SAME `handle_inst` a hand-
                    // written `inst` does, so designator allocation (RFC-005),
                    // pin obligations (RFC-002) and trait satisfaction
                    // (RFC-003) apply to it completely unchanged.
                    Some((n, span)) => {
                        if scope.arrays.contains_key(&inst.name.name)
                            || scope.local_insts.contains_key(&inst.name.name)
                        {
                            self.diags.push(Diagnostic::error(
                                "E201",
                                inst.name.span,
                                format!("`{}` is already defined in this scope", inst.name.name),
                            ));
                            continue;
                        }
                        scope.arrays.insert(inst.name.name.clone(), (n, span));
                        for i in 0..n {
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
            if let Stmt::Inst(inst) = stmt {
                if !inst.phys.is_empty() {
                    self.handle_inst_phys(inst, scope);
                }
            }
        }
        // Pass 2: everything else, in source order.
        for stmt in body {
            match stmt {
                Stmt::Inst(_) => {}
                Stmt::Net(net) => self.handle_net(net, scope),
                Stmt::Nc(nc) => self.handle_nc(nc, scope),
                Stmt::Call(call) => self.handle_call(call, scope),
                Stmt::Layout(block) => self.handle_layout(block, scope),
            }
        }
    }

    // -- layout constraints (RFC-013) ----------------------------------------

    /// Collect a `layout {}` block's constraints, resolving each net reference
    /// to its candidate IR net name within the current scope. Validation
    /// against the final net set happens in `assemble` (net existence is only
    /// knowable once every declaration in the design is processed).
    fn handle_layout(&mut self, block: &LayoutBlock, scope: &Scope) {
        let resolve = |nets: &[Ident]| -> Vec<(String, Ident)> {
            nets.iter()
                .map(|nid| (resolve_net_name(&nid.name, scope), nid.clone()))
                .collect()
        };
        for c in &block.constraints {
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
            self.handle_board_outline(outline);
        }
        for placement in &block.placements {
            self.handle_placement(placement, scope);
        }
    }

    /// Validate and record a locked component placement (E1007): design-level
    /// only, the instance must exist, and the coordinates must be `Length`
    /// values in geometry range.
    fn handle_placement(&mut self, placement: &crate::ast::Placement, scope: &Scope) {
        use crate::units::UnitType;
        if !self.active_calls.is_empty() {
            self.diags.push(Diagnostic::error(
                "E1007",
                placement.span,
                "`place` is only valid in the design's own `layout {}` block, not inside a called `fn`".to_string(),
            ));
            return;
        }
        // RFC-024: `place NAME[i]` targets one real array element — the same
        // reference form valid in every other instance position.
        let local = match (
            placement.index,
            scope.arrays.get(&placement.inst.name).copied(),
        ) {
            (None, None) => placement.inst.name.clone(),
            (None, Some(_)) => {
                self.diags.push(Diagnostic::error(
                    "E211",
                    placement.inst.span,
                    format!(
                        "`{}` is array-typed — place one element, e.g. `place {}[0] at (…)`",
                        placement.inst.name, placement.inst.name
                    ),
                ));
                return;
            }
            (Some((_, sp)), None) => {
                self.diags.push(Diagnostic::error(
                    "E211",
                    sp,
                    format!(
                        "`{}` is not an array-typed instance — only `inst NAME: [Device; N]` can be indexed",
                        placement.inst.name
                    ),
                ));
                return;
            }
            (Some((i, sp)), Some((n, _))) => {
                if i < 0 || i >= n {
                    self.diags.push(Diagnostic::error(
                        "E202",
                        sp,
                        format!(
                            "index {} is out of bounds for `{}` — valid indices are 0..={} (length {})",
                            i,
                            placement.inst.name,
                            n - 1,
                            n
                        ),
                    ));
                    return;
                }
                element_name(&placement.inst.name, i)
            }
        };
        let Some(path) = scope.local_insts.get(&local).cloned() else {
            self.diags.push(Diagnostic::error(
                "E1007",
                placement.inst.span,
                format!(
                    "`place` names `{}`, which is not an instance in this design",
                    placement.inst.name
                ),
            ));
            return;
        };
        for (v, what) in [(&placement.at.0, "x"), (&placement.at.1, "y")] {
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
        // RFC-020: rotation is a closed set {0, 90, 180, 270}.
        if !matches!(placement.rotate, 0 | 90 | 180 | 270) {
            let shown = if placement.rotate == u16::MAX {
                "that value".to_string()
            } else {
                placement.rotate.to_string()
            };
            self.diags.push(Diagnostic::error(
                "E1007",
                placement.span,
                format!(
                    "`rotate {}` is not one of the allowed rotations {{0, 90, 180, 270}}",
                    shown
                ),
            ));
            return;
        }
        if self.placements.iter().any(|p| p.path == path) {
            self.diags.push(Diagnostic::error(
                "E1007",
                placement.inst.span,
                format!("`{}` is placed more than once", placement.inst.name),
            ));
            return;
        }
        self.placements.push(crate::ir::LayoutPlacement {
            path,
            at: (placement.at.0.clone(), placement.at.1.clone()),
            rotate: placement.rotate,
            side: placement.side,
        });
    }

    /// Validate and record the board outline (RFC-020, E1006): a project-
    /// relative DXF path, declared at most once, and only in the design's own
    /// layout block — never inside a called fn (a board has one physical
    /// perimeter). The DXF is NOT read here — that happens at `cohdl build`
    /// (`pipeline::resolve_board_outline`); this only validates the reference.
    fn handle_board_outline(&mut self, outline: &crate::ast::BoardOutline) {
        if !self.active_calls.is_empty() {
            self.diags.push(Diagnostic::error(
                "E1006",
                outline.span,
                "`board_outline` is only valid in the design's own `layout {}` block, not inside a called `fn`".to_string(),
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
        let resolve_inst = |ex: &mut Self, id: &Ident| -> Option<String> {
            if let Some(p) = scope.local_insts.get(&id.name) {
                return Some(p.clone());
            }
            if let Some(Binding::Instance { path, .. }) = scope.bindings.get(&id.name) {
                return Some(path.clone());
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
            let target = &ex.instances[path];
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
            match pa {
                PhysAttr::Bypass {
                    inst: target,
                    pin,
                    capacitance,
                    ..
                } => {
                    // RFC-028: `TARGET` is INST.PIN, or a bare Pin-typed fn
                    // parameter resolving through the call site's binding —
                    // the same Binding::Pin every net member already uses.
                    let (target_path, pads) = match pin {
                        Some(pin) => {
                            let Some(target_path) = resolve_inst(self, target) else {
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
                    let Some(parent_path) = resolve_inst(self, parent) else {
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
                    let Some(inductor_path) = resolve_inst(self, inductor) else {
                        continue;
                    };
                    let input_cap_path = match input_capacitor {
                        Some(c) => match resolve_inst(self, c) {
                            Some(p) => Some(p),
                            None => continue,
                        },
                        None => None,
                    };
                    let output_cap_path = match output_capacitor {
                        Some(c) => match resolve_inst(self, c) {
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
            let args = resolve_generic_args(
                self.world,
                &format!("device `{}`", dev.name.name),
                &dev.generics,
                &inst.ty.generic_args,
                &scope.subst,
                inst.ty.span,
                self.diags,
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
    fn array_bounds(&mut self, base: &Ident, sel: &IndexSel, n: i64) -> Option<Vec<i64>> {
        let idx = sel.indices();
        if idx.is_empty() {
            self.diags.push(Diagnostic::error(
                "E211",
                sel.span(),
                format!("`{}[…]` selects no elements", base.name),
            ));
            return None;
        }
        for i in &idx {
            if *i < 0 || *i >= n {
                self.diags.push(Diagnostic::error(
                    "E202",
                    sel.span(),
                    format!(
                        "index {} is out of bounds for `{}` — valid indices are 0..={} (length {})",
                        i,
                        base.name,
                        n - 1,
                        n
                    ),
                ));
                return None;
            }
        }
        Some(idx)
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
            return vec![m.clone()];
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
        let Some(idx) = self.array_bounds(&m.base, sel, n) else {
            return Vec::new();
        };
        idx.into_iter()
            .map(|i| PinRef {
                base: m.base.clone(),
                index: Some(IndexSel::Single(i, sel.span())),
                pin: m.pin.clone(),
                span: m.span,
            })
            .collect()
    }

    fn resolve_pin_ref(&mut self, r: &PinRef, scope: &Scope) -> Option<(String, String)> {
        // RFC-024: resolve an array-typed reference to its one real element.
        // `NAME[i]` is valid in EVERY position an ordinary instance reference
        // is; only the range/list fan-out sugar is net-member-only, and that
        // has already been expanded to `Single`s by `handle_net`.
        let array = scope.arrays.get(&r.base.name).copied();
        let owned;
        let r = match (&r.index, array) {
            (None, None) => r,
            (None, Some(_)) => {
                self.diags.push(Diagnostic::error(
                    "E211",
                    r.base.span,
                    format!(
                        "`{}` is array-typed — reference one element, e.g. `{}[0]`",
                        r.base.name, r.base.name
                    ),
                ));
                return None;
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
                return None;
            }
            (Some(sel), Some((n, _))) => {
                let IndexSel::Single(i, _) = sel else {
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
                self.array_bounds(&r.base, sel, n)?;
                owned = PinRef {
                    base: Ident {
                        name: element_name(&r.base.name, *i),
                        span: r.base.span,
                    },
                    index: None,
                    pin: r.pin.clone(),
                    span: r.span,
                };
                &owned
            }
        };
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
            for expanded in self.expand_member(m, scope) {
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
        self.call_counter += 1;
        let mut inner = Scope {
            design_name: scope.design_name.clone(),
            path: format!("{}::{}", scope.path, seg),
            is_design_body: false,
            subst,
            bindings,
            local_insts: BTreeMap::new(),
            arrays: BTreeMap::new(),
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
        // RFC-013: every declared net name (including aliases merged into a
        // differently-named group) maps to that group's final name, so a layout
        // reference validates against *declared identity*, not the post-merge
        // name it may have lost to the naming race.
        let mut declared_to_merged: BTreeMap<String, String> = BTreeMap::new();
        for decl_idxs in groups.values() {
            let decls: Vec<&NetDecl> = decl_idxs.iter().map(|&i| &self.net_decls[i]).collect();
            // Name: smallest design-level name, else smallest scoped name.
            let name = decls
                .iter()
                .filter(|d| d.is_design_level_name)
                .map(|d| d.display_name.clone())
                .min()
                .or_else(|| decls.iter().map(|d| d.display_name.clone()).min())
                .unwrap();
            for d in &decls {
                declared_to_merged.insert(d.display_name.clone(), name.clone());
            }
            let members: BTreeSet<(String, String)> = decls
                .iter()
                .flat_map(|d| d.members.iter().cloned())
                .collect();
            if members.is_empty() {
                // Every member failed to resolve — errors already reported;
                // (a zero-member net is unrepresentable in the grammar).
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

        let nc_pins: BTreeSet<(String, String)> =
            self.nc_pins.iter().map(|(p, _)| p.clone()).collect();

        // RFC-013: validate layout constraints against declared-net identity
        // and build the (connectivity-independent) layout IR.
        let mut layout = build_layout_ir(&self.layout_raw, &declared_to_merged, self.diags);
        // The board outline + locked placements (validated on collection) ride
        // the same IR.
        layout.board_outline = self.board_outline;
        layout.placements = self.placements;

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
