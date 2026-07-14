//! RFC-007: generic parameter resolution and trait-bound checking.
//!
//! This is the ONE trait-bound-checking code path (DR-016): device
//! instantiations, fn calls, and `impl Trait`-typed value parameters
//! (desugared) all resolve through `resolve_generic_args`.

use crate::ast::*;
use crate::diag::{Diagnostic, Diagnostics};
use crate::resolve::World;
use crate::span::Span;
use crate::units::UnitValue;
use std::collections::BTreeMap;

/// A fully-resolved generic argument.
#[derive(Debug, Clone)]
pub enum GenericValue {
    Unit(UnitValue),
    /// A concrete device name (for trait-bound parameters).
    Device(String),
}

pub type Substitution = BTreeMap<String, GenericValue>;

/// Resolve `args` against `params`, checking bounds (RFC-007).
///
/// `env` is the enclosing substitution — a name argument may reference an
/// outer generic parameter, resolved outward-in (RFC-006). Returns `None`
/// only when arity is unusable; individual bad arguments produce diagnostics
/// and a poisoned (absent) entry so downstream checks can continue.
pub fn resolve_generic_args(
    world: &World,
    owner_desc: &str,
    params: &[GenericParam],
    args: &[GenericArg],
    env: &Substitution,
    site: Span,
    diags: &mut Diagnostics,
) -> Substitution {
    if args.len() > params.len() {
        diags.push(Diagnostic::error(
            "E401",
            site,
            format!(
                "{} takes {} generic argument{}, but {} {} given",
                owner_desc,
                params.len(),
                if params.len() == 1 { "" } else { "s" },
                args.len(),
                if args.len() == 1 { "was" } else { "were" }
            ),
        ));
    }

    let mut subst = Substitution::new();
    for (i, param) in params.iter().enumerate() {
        match args.get(i) {
            Some(arg) => {
                if let Some(value) = resolve_one(world, param, arg, env, diags) {
                    subst.insert(param.name.name.clone(), value);
                }
            }
            None => match (&param.default, &param.bound) {
                (Some((val, _)), _) => {
                    subst.insert(param.name.name.clone(), GenericValue::Unit(val.clone()));
                }
                (None, _) => {
                    diags.push(
                        Diagnostic::error(
                            "E401",
                            site,
                            format!(
                                "missing generic argument for `{}` of {} (it has no default)",
                                param.name.name, owner_desc
                            ),
                        )
                        .with_help(describe_param(param)),
                    );
                }
            },
        }
    }
    subst
}

fn describe_param(param: &GenericParam) -> String {
    match &param.bound {
        GenericBound::Unit(u) => format!(
            "`{}` expects a `{}` value (e.g. `{}`)",
            param.name.name,
            u.unit.type_name(),
            example_literal(u.unit)
        ),
        GenericBound::Traits(ts) => format!(
            "`{}` expects a device type implementing {}",
            param.name.name,
            ts.iter()
                .map(|t| format!("`{}`", t.name))
                .collect::<Vec<_>>()
                .join(" + ")
        ),
    }
}

fn example_literal(unit: crate::units::UnitType) -> String {
    use crate::units::UnitType::*;
    match unit {
        Voltage => "3.3V".into(),
        Capacitance => "100nF".into(),
        Resistance => "10kohm".into(),
        Current => "500mA".into(),
        Frequency => "16MHz".into(),
        Time => "10ms".into(),
        Inductance => "10uH".into(),
        Power => "250mW".into(),
        Temperature => "85C".into(),
        Tolerance => "1%".into(),
    }
}

fn resolve_one(
    world: &World,
    param: &GenericParam,
    arg: &GenericArg,
    env: &Substitution,
    diags: &mut Diagnostics,
) -> Option<GenericValue> {
    match (&param.bound, arg) {
        // ---- unit-bound parameter ----
        (GenericBound::Unit(u), GenericArg::Unit(val, span)) => {
            if val.unit == u.unit {
                Some(GenericValue::Unit(val.clone()))
            } else {
                diags.push(
                    Diagnostic::error(
                        "E112",
                        *span,
                        format!(
                            "generic argument for `{}` has the wrong unit type: expected `{}`, found `{}`",
                            param.name.name,
                            u.unit.type_name(),
                            val.unit.type_name()
                        ),
                    )
                    .with_primary_label(format!("`{}` is a `{}`", val.text, val.unit.type_name())),
                );
                None
            }
        }
        (GenericBound::Unit(u), GenericArg::Number(n, span)) => {
            diags.push(
                Diagnostic::error(
                    "E113",
                    *span,
                    format!(
                        "a bare number is never valid for `{}: {}` — write the unit (e.g. `{}`)",
                        param.name.name,
                        u.unit.type_name(),
                        format_args!("{}{}", n, u.unit.symbol())
                    ),
                )
                .with_help("RFC-001: no bare numbers, no default units, no coercion"),
            );
            None
        }
        (GenericBound::Unit(u), GenericArg::Name(name)) => match env.get(&name.name) {
            Some(GenericValue::Unit(val)) => {
                if val.unit == u.unit {
                    Some(GenericValue::Unit(val.clone()))
                } else {
                    diags.push(Diagnostic::error(
                        "E112",
                        name.span,
                        format!(
                            "`{}` resolves to `{}` (`{}`), but `{}` expects a `{}`",
                            name.name,
                            val.text,
                            val.unit.type_name(),
                            param.name.name,
                            u.unit.type_name()
                        ),
                    ));
                    None
                }
            }
            Some(GenericValue::Device(d)) => {
                diags.push(Diagnostic::error(
                    "E112",
                    name.span,
                    format!(
                        "`{}` resolves to device type `{}`, but `{}` expects a `{}` value",
                        name.name,
                        d,
                        param.name.name,
                        u.unit.type_name()
                    ),
                ));
                None
            }
            None => {
                if world.devices.contains_key(&name.name) || world.parts.contains_key(&name.name) {
                    diags.push(Diagnostic::error(
                        "E112",
                        name.span,
                        format!(
                            "`{}` is a device type, but `{}` expects a `{}` value",
                            name.name,
                            param.name.name,
                            u.unit.type_name()
                        ),
                    ));
                } else {
                    diags.push(Diagnostic::error(
                            "E405",
                            name.span,
                            format!(
                                "`{}` is not a generic parameter in scope here (generic arguments must be concrete by instantiation — RFC-007)",
                                name.name
                            ),
                        ));
                }
                None
            }
        },
        // ---- trait-bound parameter ----
        (GenericBound::Traits(bounds), GenericArg::Name(name)) => {
            // Outer generic parameter first (outward-in threading, RFC-006).
            let device: String = match env.get(&name.name) {
                Some(GenericValue::Device(d)) => d.clone(),
                Some(GenericValue::Unit(val)) => {
                    diags.push(Diagnostic::error(
                        "E403",
                        name.span,
                        format!(
                            "`{}` resolves to `{}` (`{}`), but `{}` expects a device type",
                            name.name,
                            val.text,
                            val.unit.type_name(),
                            param.name.name
                        ),
                    ));
                    return None;
                }
                None => {
                    if world.devices.contains_key(&name.name) {
                        name.name.clone()
                    } else if let Some(part) = world.parts.get(&name.name) {
                        part.device.name.name.clone()
                    } else {
                        diags.push(suggested(
                            world,
                            &name.name,
                            Diagnostic::error(
                                "E202",
                                name.span,
                                format!("unknown device `{}`", name.name),
                            ),
                        ));
                        return None;
                    }
                }
            };
            let required_by = format!("`{}`", param.name.name);
            check_trait_bounds(world, &device, bounds, name.span, &required_by, diags)
                .then_some(GenericValue::Device(device))
        }
        (GenericBound::Traits(_), GenericArg::Unit(val, span)) => {
            diags.push(Diagnostic::error(
                "E403",
                *span,
                format!(
                    "`{}` expects a device type, found unit literal `{}`",
                    param.name.name, val.text
                ),
            ));
            None
        }
        (GenericBound::Traits(_), GenericArg::Number(n, span)) => {
            diags.push(Diagnostic::error(
                "E403",
                *span,
                format!(
                    "`{}` expects a device type, found bare number `{}`",
                    param.name.name, n
                ),
            ));
            None
        }
    }
}

/// THE one trait-bound-checking mechanism (RFC-007 / DR-016).
///
/// Every trait bound — whether on a named generic type parameter or on an
/// `impl Trait`-typed value parameter (which is sugar for an anonymous
/// generic parameter) — is checked HERE, by looking up free-standing
/// `impl Trait for Device` statements in scope. There is deliberately no
/// second code path (the v1 bug DR-016 exists to prevent).
pub fn check_trait_bounds(
    world: &World,
    device: &str,
    bounds: &[Ident],
    site: Span,
    required_by: &str,
    diags: &mut Diagnostics,
) -> bool {
    let mut ok = true;
    for bound in bounds {
        if !world.has_impl(&bound.name, device) {
            diags.push(
                Diagnostic::error(
                    "E403",
                    site,
                    format!(
                        "`{}` does not implement `{}`, required by {}",
                        crate::resolve::short(device),
                        crate::resolve::short(&bound.name),
                        required_by
                    ),
                )
                .with_help(format!(
                    "add `impl {} for {} {{ … }}` (checked at the impl statement, RFC-003), or pass a device that has one",
                    crate::resolve::short(&bound.name),
                    crate::resolve::short(device)
                )),
            );
            ok = false;
        }
    }
    ok
}

/// Validate every `part` declaration (provisional §2): the device exists, the
/// generic arguments are fully concrete and bound-correct, `primary` carries
/// `mpn` + `footprint`, every `alt` carries `mpn`.
pub fn check_parts(world: &World, diags: &mut Diagnostics) {
    for part in world.parts.values() {
        let Some(device) = world.devices.get(&part.device.name.name) else {
            let d = if world.traits.contains_key(&part.device.name.name) {
                Diagnostic::error(
                    "E205",
                    part.device.name.span,
                    format!(
                        "`{}` is a trait — a part binds a concrete device",
                        part.device.name.name
                    ),
                )
            } else {
                suggested(
                    world,
                    &part.device.name.name,
                    Diagnostic::error(
                        "E202",
                        part.device.name.span,
                        format!("unknown device `{}`", part.device.name.name),
                    ),
                )
            };
            diags.push(d);
            continue;
        };
        // RFC-008: a part is a purchasable component — it must pin down the
        // variant exactly like it pins down every generic argument.
        let valid_set = || {
            device
                .variants
                .iter()
                .map(|v| v.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        };
        match (&part.device.variant, device.has_variants()) {
            (Some(sel), true) => {
                if !device.variants.iter().any(|v| v.name == sel.name) {
                    diags.push(
                        Diagnostic::error(
                            "E903",
                            sel.span,
                            format!(
                                "device `{}` declares no variant named `{}`",
                                device.name.name, sel.name
                            ),
                        )
                        .with_help(format!("valid variants are: {}", valid_set())),
                    );
                }
            }
            (None, true) => {
                diags.push(
                    Diagnostic::error(
                        "E904",
                        part.device.span,
                        format!(
                            "part `{}` binds device `{}`, which declares variants — select one with a `[VARIANT]` suffix",
                            part.name.name, device.name.name
                        ),
                    )
                    .with_help(format!("valid variants are: {}", valid_set())),
                );
            }
            (Some(sel), false) => {
                diags.push(Diagnostic::error(
                    "E905",
                    sel.span,
                    format!(
                        "device `{}` has no `variants {{ }}` block — remove the `[{}]` selector",
                        device.name.name, sel.name
                    ),
                ));
            }
            (None, false) => {}
        }

        // No open parameters allowed: every arg must be a literal (an empty
        // env means any Name argument fails as non-concrete).
        for arg in &part.device.generic_args {
            if let GenericArg::Name(n) = arg {
                diags.push(
                    Diagnostic::error(
                        "E802",
                        n.span,
                        format!(
                            "part `{}` must bind a fully-concrete device — `{}` is not a literal",
                            part.name.name, n.name
                        ),
                    )
                    .with_help("a part is a purchasable component: every generic argument must be a unit literal"),
                );
            }
        }
        let _ = resolve_generic_args(
            world,
            &format!("device `{}`", device.name.name),
            &device.generics,
            &part.device.generic_args,
            &Substitution::new(),
            part.device.span,
            diags,
        );
        // AVL discipline.
        for (entry, is_primary) in
            std::iter::once((&part.primary, true)).chain(part.alts.iter().map(|a| (a, false)))
        {
            if entry.field("mpn").is_none() {
                diags.push(Diagnostic::error(
                    "E802",
                    entry.span,
                    format!(
                        "{} entry of part `{}` is missing `mpn` — MPN binding is non-optional the moment a part is declared",
                        if is_primary { "`primary`" } else { "`alt`" },
                        part.name.name
                    ),
                ));
            }
            if is_primary && entry.footprint.is_none() {
                diags.push(
                    Diagnostic::error(
                        "E802",
                        entry.span,
                        format!(
                            "`primary` entry of part `{}` is missing `footprint`",
                            part.name.name
                        ),
                    )
                    .with_help(
                        "reference a footprint symbol, e.g. `footprint: C_0402_1005Metric` (RFC-017)",
                    ),
                );
            }
            // RFC-017: the footprint reference must resolve to a FOOTPRINT
            // symbol — unknown and wrong-kind get the standard RFC-016
            // diagnostics.
            for fp in entry.footprint.iter() {
                if world.footprints.contains_key(&fp.name) {
                    continue;
                }
                let d = if world.devices.contains_key(&fp.name)
                    || world.parts.contains_key(&fp.name)
                    || world.traits.contains_key(&fp.name)
                    || world.fns.contains_key(&fp.name)
                {
                    let kind = world
                        .symbols
                        .get(&fp.name)
                        .map(|s| s.kind)
                        .unwrap_or("name");
                    Diagnostic::error(
                        "E205",
                        fp.span,
                        format!(
                            "`{}` is a {}, not a footprint — `footprint:` references a `footprint` declaration",
                            fp.name, kind
                        ),
                    )
                } else {
                    suggested(
                        world,
                        &fp.name,
                        Diagnostic::error(
                            "E202",
                            fp.span,
                            format!("unknown footprint `{}`", fp.name),
                        ),
                    )
                };
                diags.push(d);
            }
            for field in &entry.fields {
                if !matches!(field.name.name.as_str(), "mpn" | "mfr") {
                    diags.push(Diagnostic::error(
                        "E802",
                        field.span,
                        format!(
                            "unknown AVL field `{}` (expected `mpn`, `mfr`, or `footprint`)",
                            field.name.name
                        ),
                    ));
                }
            }
        }
    }
}

/// Attach RFC-016's closest-match help to an unknown-name diagnostic.
pub(crate) fn suggested(
    world: &World,
    name: &str,
    d: crate::diag::Diagnostic,
) -> crate::diag::Diagnostic {
    match world.suggest(name) {
        Some(s) => d.with_help(format!("did you mean `{}`?", s)),
        None => d,
    }
}
