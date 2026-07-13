//! Name resolution: one flat global scope over every parsed file
//! (provisional-syntax.md §1), plus structural validation of declarations.

use crate::ast::*;
use crate::diag::{Diagnostic, Diagnostics};
use crate::units::UnitType;
use std::collections::{BTreeMap, BTreeSet};

/// Everything declared across the compilation, indexed by name.
#[derive(Debug, Default)]
pub struct World {
    pub traits: BTreeMap<String, TraitDef>,
    pub devices: BTreeMap<String, DeviceDef>,
    pub fns: BTreeMap<String, FnDef>,
    pub parts: BTreeMap<String, PartDef>,
    pub designs: BTreeMap<String, DesignDef>,
    pub impls: Vec<ImplDef>,
    /// (trait name, device name) → index into `impls`. Populated only for
    /// non-duplicate impls.
    pub impl_index: BTreeMap<(String, String), usize>,
    /// Resolved role/field maps per impl, filled by `check::impls`.
    pub resolved_impls: BTreeMap<(String, String), ResolvedImpl>,
}

/// The outcome of checking one `impl Trait for Device`: every trait-required
/// role/field resolved to a concrete device pin/spec name.
#[derive(Debug, Clone, Default)]
pub struct ResolvedImpl {
    /// trait pin role → device pin name.
    pub pin_map: BTreeMap<String, String>,
    /// trait spec field → device spec field name.
    pub spec_map: BTreeMap<String, String>,
}

impl World {
    /// All traits transitively required by `trait_name` (excluding itself),
    /// in deterministic order. Cycles are pre-rejected by `validate`.
    pub fn super_traits_transitive(&self, trait_name: &str) -> Vec<String> {
        let mut out = BTreeSet::new();
        let mut stack = vec![trait_name.to_string()];
        while let Some(t) = stack.pop() {
            if let Some(def) = self.traits.get(&t) {
                for sup in &def.super_traits {
                    if out.insert(sup.name.clone()) {
                        stack.push(sup.name.clone());
                    }
                }
            }
        }
        out.into_iter().collect()
    }

    /// Does `device` have a (checked) impl for `trait_name`?
    pub fn has_impl(&self, trait_name: &str, device: &str) -> bool {
        self.impl_index
            .contains_key(&(trait_name.to_string(), device.to_string()))
    }

    /// The designator prefix for a device: the prefix of the
    /// lexicographically-smallest implemented trait that declares one;
    /// `"U"` when none does (provisional §6).
    pub fn designator_prefix(&self, device: &str) -> String {
        let mut best: Option<(&String, &str)> = None;
        for (trait_name, _) in self.impl_index.keys().filter(|(_, d)| d == device) {
            if let Some(t) = self.traits.get(trait_name) {
                if let Some((prefix, _)) = &t.designator_prefix {
                    match best {
                        Some((bn, _)) if bn <= trait_name => {}
                        _ => best = Some((trait_name, prefix)),
                    }
                }
            }
        }
        best.map_or_else(|| "U".to_string(), |(_, p)| p.to_string())
    }

    /// All trait names `device` implements (checked impls only).
    pub fn implemented_traits(&self, device: &str) -> BTreeSet<String> {
        self.impl_index
            .keys()
            .filter(|(_, d)| d == device)
            .map(|(t, _)| t.clone())
            .collect()
    }
}

pub fn build_world(files: Vec<SourceFile>, diags: &mut Diagnostics) -> World {
    let mut world = World::default();
    let mut seen: BTreeMap<String, (&'static str, crate::span::Span)> = BTreeMap::new();

    for file in files {
        for item in file.items {
            let kind_str = item.kind.kind_str();
            if let Some(name) = item.kind.name() {
                if let Some((prev_kind, prev_span)) = seen.get(&name.name) {
                    diags.push(
                        Diagnostic::error(
                            "E201",
                            name.span,
                            format!(
                                "duplicate declaration of `{}` (all top-level names share one flat scope)",
                                name.name
                            ),
                        )
                        .with_secondary(*prev_span, format!("earlier declared here as a {}", prev_kind)),
                    );
                    continue;
                }
                seen.insert(name.name.clone(), (kind_str, name.span));
            }
            match item.kind {
                ItemKind::Trait(t) => {
                    world.traits.insert(t.name.name.clone(), t);
                }
                ItemKind::Device(d) => {
                    world.devices.insert(d.name.name.clone(), d);
                }
                ItemKind::Fn(f) => {
                    world.fns.insert(f.name.name.clone(), f);
                }
                ItemKind::Part(p) => {
                    world.parts.insert(p.name.name.clone(), p);
                }
                ItemKind::Design(d) => {
                    world.designs.insert(d.name.name.clone(), d);
                }
                ItemKind::Impl(i) => {
                    world.impls.push(i);
                }
            }
        }
    }

    validate(&mut world, diags);
    world
}

fn validate(world: &mut World, diags: &mut Diagnostics) {
    validate_traits(world, diags);
    validate_devices(world, diags);
    validate_fns(world, diags);
    index_impls(world, diags);
    // Parts are validated in check::generics (they need generic-arg checking).
}

fn validate_traits(world: &World, diags: &mut Diagnostics) {
    for tr in world.traits.values() {
        for sup in &tr.super_traits {
            if !world.traits.contains_key(&sup.name) {
                let d = if world.devices.contains_key(&sup.name)
                    || world.parts.contains_key(&sup.name)
                    || world.fns.contains_key(&sup.name)
                {
                    Diagnostic::error(
                        "E205",
                        sup.span,
                        format!(
                            "`{}` is not a trait (sub-trait bounds must name traits)",
                            sup.name
                        ),
                    )
                } else {
                    Diagnostic::error("E202", sup.span, format!("unknown trait `{}`", sup.name))
                };
                diags.push(d);
            }
        }
        // Duplicate pin roles / spec fields within the trait.
        check_dup_names(
            tr.pins.iter().map(|p| &p.name),
            "pin role",
            &tr.name.name,
            diags,
        );
        check_dup_names(
            tr.specs.iter().map(|s| &s.name),
            "spec field",
            &tr.name.name,
            diags,
        );
    }

    // Cyclic sub-trait bounds (E306): DFS from each trait.
    for start in world.traits.keys() {
        let mut path: Vec<String> = Vec::new();
        if let Some(cycle) = find_trait_cycle(world, start, &mut path, &mut BTreeSet::new()) {
            // Report only when `start` is the lexicographically-smallest
            // member so each cycle is reported exactly once.
            if cycle.iter().min() == Some(start) {
                let tr = &world.traits[start];
                diags.push(Diagnostic::error(
                    "E306",
                    tr.name.span,
                    format!("cyclic sub-trait bounds: {}", cycle.join(" → ")),
                ));
            }
        }
    }
}

fn find_trait_cycle(
    world: &World,
    current: &str,
    path: &mut Vec<String>,
    visited: &mut BTreeSet<String>,
) -> Option<Vec<String>> {
    if let Some(pos) = path.iter().position(|p| p == current) {
        let mut cycle = path[pos..].to_vec();
        cycle.push(current.to_string());
        return Some(cycle);
    }
    if !visited.insert(current.to_string()) {
        return None;
    }
    path.push(current.to_string());
    if let Some(def) = world.traits.get(current) {
        for sup in &def.super_traits {
            if let Some(c) = find_trait_cycle(world, &sup.name, path, visited) {
                path.pop();
                return Some(c);
            }
        }
    }
    path.pop();
    None
}

fn validate_devices(world: &World, diags: &mut Diagnostics) {
    for dev in world.devices.values() {
        check_dup_names(
            dev.generics.iter().map(|g| &g.name),
            "generic parameter",
            &dev.name.name,
            diags,
        );
        check_dup_names(
            dev.pins.iter().map(|p| &p.name),
            "pin",
            &dev.name.name,
            diags,
        );
        check_dup_names(
            dev.specs.iter().map(|s| &s.name),
            "spec field",
            &dev.name.name,
            diags,
        );
        validate_generic_params(world, &dev.generics, diags);
        // Spec fields referencing generic params must name a unit-bound param.
        for field in &dev.specs {
            if let SpecValue::GenericRef(r) = &field.value {
                match dev.generics.iter().find(|g| g.name.name == r.name) {
                    None => diags.push(
                        Diagnostic::error(
                            "E202",
                            r.span,
                            format!(
                                "`{}` is not a generic parameter of device `{}`",
                                r.name, dev.name.name
                            ),
                        )
                        .with_help(
                            "device spec values are unit literals or the device's own generic parameters",
                        ),
                    ),
                    Some(g) => {
                        if let GenericBound::Traits(_) = &g.bound {
                            diags.push(Diagnostic::error(
                                "E205",
                                r.span,
                                format!(
                                    "generic parameter `{}` is trait-bound and cannot be used as a spec value (spec fields are unit-typed)",
                                    r.name
                                ),
                            ));
                        }
                    }
                }
            }
        }
        // Duplicate physical pin numbers across the device.
        let mut nums: BTreeMap<&str, crate::span::Span> = BTreeMap::new();
        for pin in &dev.pins {
            for n in &pin.numbers {
                if let Some(prev) = nums.insert(n.text.as_str(), n.span) {
                    diags.push(
                        Diagnostic::error(
                            "E201",
                            n.span,
                            format!(
                                "physical pin number `{}` is used twice on device `{}`",
                                n.text, dev.name.name
                            ),
                        )
                        .with_secondary(prev, "first used here"),
                    );
                }
            }
        }
    }
}

fn validate_fns(world: &World, diags: &mut Diagnostics) {
    for f in world.fns.values() {
        check_dup_names(
            f.generics.iter().map(|g| &g.name),
            "generic parameter",
            &f.name.name,
            diags,
        );
        check_dup_names(
            f.params.iter().map(|p| &p.name),
            "parameter",
            &f.name.name,
            diags,
        );
        validate_generic_params(world, &f.generics, diags);
        for p in &f.params {
            match &p.ty {
                FnParamTy::Pin(_) => {}
                FnParamTy::Generic(g) => {
                    match f.generics.iter().find(|gp| gp.name.name == g.name) {
                        None => diags.push(Diagnostic::error(
                            "E202",
                            g.span,
                            format!(
                            "`{}` is not `Pin`, `impl Trait`, or a generic parameter of fn `{}`",
                            g.name, f.name.name
                        ),
                        )),
                        Some(gp) => {
                            if let GenericBound::Unit(u) = &gp.bound {
                                diags.push(Diagnostic::error(
                                    "E205",
                                    g.span,
                                    format!(
                                        "generic parameter `{}` is unit-bound (`{}`) — only trait-bound parameters can type a value parameter",
                                        g.name,
                                        u.unit.type_name()
                                    ),
                                ));
                            }
                        }
                    }
                }
                FnParamTy::ImplTrait(traits, _) => {
                    for t in traits {
                        check_trait_ref(world, t, diags);
                    }
                }
            }
        }
    }
}

fn validate_generic_params(world: &World, generics: &[GenericParam], diags: &mut Diagnostics) {
    for g in generics {
        match &g.bound {
            GenericBound::Unit(u) => {
                if let Some((val, span)) = &g.default {
                    if val.unit != u.unit {
                        diags.push(
                            Diagnostic::error(
                                "E110",
                                *span,
                                format!(
                                    "default for `{}` has the wrong unit type: expected `{}`, found `{}`",
                                    g.name.name,
                                    u.unit.type_name(),
                                    val.unit.type_name()
                                ),
                            )
                            .with_primary_label(format!("this literal is `{}`", val.unit.type_name())),
                        );
                    }
                }
            }
            GenericBound::Traits(traits) => {
                for t in traits {
                    check_trait_ref(world, t, diags);
                }
                if let Some((_, span)) = &g.default {
                    diags.push(Diagnostic::error(
                        "E406",
                        *span,
                        format!(
                            "generic parameter `{}` is trait-bound — defaults are only valid on unit-type parameters (RFC-007)",
                            g.name.name
                        ),
                    ));
                }
            }
        }
    }
}

fn check_trait_ref(world: &World, t: &Ident, diags: &mut Diagnostics) {
    if world.traits.contains_key(&t.name) {
        return;
    }
    // A unit type name in trait-bound position parses as a unit bound, so
    // reaching here with a unit name is impossible; distinguish device/part.
    let diag = if world.devices.contains_key(&t.name) || world.parts.contains_key(&t.name) {
        Diagnostic::error(
            "E205",
            t.span,
            format!(
                "`{}` is not a trait (trait bounds must name traits)",
                t.name
            ),
        )
    } else if UnitType::from_type_name(&t.name).is_some() {
        // e.g. `impl Voltage + Capacitor` — nonsensical mix.
        Diagnostic::error(
            "E205",
            t.span,
            format!("`{}` is a unit type, not a trait", t.name),
        )
    } else {
        Diagnostic::error("E202", t.span, format!("unknown trait `{}`", t.name))
    };
    diags.push(diag);
}

fn index_impls(world: &mut World, diags: &mut Diagnostics) {
    for (idx, im) in world.impls.iter().enumerate() {
        let trait_ok = world.traits.contains_key(&im.trait_name.name);
        let device_ok = world.devices.contains_key(&im.device_name.name);
        if !trait_ok {
            let d = if world.devices.contains_key(&im.trait_name.name) {
                Diagnostic::error(
                    "E205",
                    im.trait_name.span,
                    format!("`{}` is a device, not a trait", im.trait_name.name),
                )
            } else {
                Diagnostic::error(
                    "E202",
                    im.trait_name.span,
                    format!("unknown trait `{}`", im.trait_name.name),
                )
            };
            diags.push(d);
        }
        if !device_ok {
            let d = if world.traits.contains_key(&im.device_name.name) {
                Diagnostic::error(
                    "E205",
                    im.device_name.span,
                    format!(
                        "`{}` is a trait — `impl Trait for Device` implements a trait *for a device*",
                        im.device_name.name
                    ),
                )
            } else {
                Diagnostic::error(
                    "E202",
                    im.device_name.span,
                    format!("unknown device `{}`", im.device_name.name),
                )
            };
            diags.push(d);
        }
        if !(trait_ok && device_ok) {
            continue;
        }
        let key = (im.trait_name.name.clone(), im.device_name.name.clone());
        if let Some(&prev_idx) = world.impl_index.get(&key) {
            diags.push(
                Diagnostic::error(
                    "E303",
                    im.span,
                    format!(
                        "duplicate `impl {} for {}`",
                        im.trait_name.name, im.device_name.name
                    ),
                )
                .with_secondary(world.impls[prev_idx].span, "the earlier impl is here"),
            );
            continue;
        }
        world.impl_index.insert(key, idx);
    }
}

fn check_dup_names<'i>(
    names: impl Iterator<Item = &'i Ident>,
    what: &str,
    owner: &str,
    diags: &mut Diagnostics,
) {
    let mut seen: BTreeMap<&str, crate::span::Span> = BTreeMap::new();
    for n in names {
        if let Some(prev) = seen.insert(n.name.as_str(), n.span) {
            diags.push(
                Diagnostic::error(
                    "E201",
                    n.span,
                    format!("duplicate {} `{}` on `{}`", what, n.name, owner),
                )
                .with_secondary(prev, "first declared here"),
            );
        }
    }
}
