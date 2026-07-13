//! Codegen: KiCad `.net` netlist + BOM CSV (Layer 3).
//!
//! Emitters are byte-stable: same IR → same bytes. Every ordering is an
//! explicit sort; nothing iterates a hash map.

pub mod bom;
pub mod kicad;

use crate::diag::{Diagnostic, Diagnostics};
use crate::ir::DesignIr;
use crate::resolve::World;

/// Bind every instance to a part (provisional-syntax.md §2). Instances typed
/// by a part name are already bound; the rest bind by exact match on
/// (device, resolved spec values). No match → E801 (the BOM must not lie).
/// Ambiguous matches pick the lexicographically-smallest part name and push a
/// note (surfaced in the build output, per provisional §2).
pub fn bind_parts(
    world: &World,
    ir: &mut DesignIr,
    diags: &mut Diagnostics,
    notes: &mut Vec<String>,
) {
    // (device, sorted spec values) → part names, smallest first.
    let matches_for_instance =
        |device: &str,
         specs: &std::collections::BTreeMap<String, crate::units::UnitValue>|
         -> Vec<String> {
            let mut found = Vec::new();
            for (name, part) in &world.parts {
                if part.device.name.name != device {
                    continue;
                }
                let Some(dev) = world.devices.get(device) else {
                    continue;
                };
                let part_args = crate::check::generics::resolve_generic_args(
                    world,
                    "part",
                    &dev.generics,
                    &part.device.generic_args,
                    &Default::default(),
                    part.device.span,
                    &mut Diagnostics::new(),
                );
                // Compute the part's resolved spec values the same way instances do.
                let mut part_specs = std::collections::BTreeMap::new();
                for field in &dev.specs {
                    match &field.value {
                        crate::ast::SpecValue::Lit(v, _) => {
                            part_specs.insert(field.name.name.clone(), v.femto);
                        }
                        crate::ast::SpecValue::GenericRef(r) => {
                            if let Some(crate::check::generics::GenericValue::Unit(v)) =
                                part_args.get(&r.name)
                            {
                                part_specs.insert(field.name.name.clone(), v.femto);
                            }
                        }
                    }
                }
                let inst_specs: std::collections::BTreeMap<String, i128> =
                    specs.iter().map(|(k, v)| (k.clone(), v.femto)).collect();
                if part_specs == inst_specs {
                    found.push(name.clone());
                }
            }
            found
        };

    let paths: Vec<String> = ir.instances.keys().cloned().collect();
    for path in paths {
        let inst = &ir.instances[&path];
        if inst.part.is_some() {
            continue;
        }
        let candidates = matches_for_instance(&inst.device, &inst.specs);
        let inst = ir.instances.get_mut(&path).unwrap();
        match candidates.first() {
            Some(part) => {
                if candidates.len() > 1 {
                    notes.push(format!(
                        "`{}` matches {} parts ({}); bound to the lexicographically-smallest, `{}`",
                        path,
                        candidates.len(),
                        candidates.join(", "),
                        part
                    ));
                }
                inst.part = Some(part.clone());
            }
            None => {
                diags.push(
                    Diagnostic::error(
                        "E801",
                        inst.span,
                        format!(
                            "`{}` (device `{}`) has no part binding — the netlist/BOM would lie about what to buy",
                            path, inst.device
                        ),
                    )
                    .with_help(format!(
                        "declare a `part SomeName: {}<…>` whose spec values match this instance exactly, or instantiate an existing part by name",
                        inst.device
                    )),
                );
            }
        }
    }
}

/// Natural designator sort key: `C2` before `C10`, `C9` before `U1`.
pub(crate) fn designator_sort_key(d: &str) -> (String, u64) {
    let prefix_len = d.chars().take_while(|c| c.is_ascii_uppercase()).count();
    let num = d[prefix_len..].parse().unwrap_or(0);
    (d[..prefix_len].to_string(), num)
}
