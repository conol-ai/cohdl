//! RFC-034 §3.2 supplier footprint-frame map and placement list.
//!
//! The map consumes shared per-instance selection and the exact KiCad pad
//! projection.  Its `frame` hashes only the canonical pads JSON, never the
//! presentation whitespace of the surrounding map document.

use crate::emit::json::json_str;
use crate::ir::DesignIr;
use crate::resolve::World;
use crate::supplier::{AvlEntryIndex, SupplierSelection, SupplierSelections};
use std::collections::BTreeSet;
use std::fmt::Write as _;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupplierMap {
    pub supplier: String,
    pub json: String,
    pub cpl: Option<String>,
}

fn root(fq: &str) -> &str {
    fq.split("::").next().unwrap_or(fq)
}

fn entry_json(index: AvlEntryIndex) -> String {
    match index {
        AvlEntryIndex::Primary => "\"primary\"".to_string(),
        AvlEntryIndex::Alt(index) => index.to_string(),
    }
}

fn render_footprints(world: &World, footprints: &BTreeSet<String>) -> String {
    let mut out = String::from("{");
    for (index, fq) in footprints.iter().enumerate() {
        let Some(footprint) = world.footprints.get(fq) else {
            continue;
        };
        if index > 0 {
            out.push(',');
        }
        let root = root(fq);
        let package = world
            .package_names
            .get(root)
            .map(String::as_str)
            .unwrap_or(root);
        let _ = write!(
            out,
            "\n    {}: {{\n      \"package\": {},\n      \"version\": {},",
            json_str(fq),
            json_str(package),
            world.package_versions.get(package).map_or_else(
                || json_str("0.0.0+unversioned"),
                |version| json_str(version)
            ),
        );
        let mut pads: Vec<_> = crate::emit::kicad_mod::pad_plans(world, footprint)
            .into_iter()
            .filter(|pad| !pad.number.is_empty())
            .map(|pad| (pad.number, pad.x, pad.y))
            .collect();
        // RFC sorting is by number and coordinate value, not decimal text.
        // Keep the source-of-truth femto integers for the ordering while
        // preserving their canonical mm spellings in the frame JSON.
        pads.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
                .then_with(|| left.2.cmp(&right.2))
        });
        let canonical = format!(
            "[{}]",
            pads.iter()
                .map(|(number, x, y)| {
                    format!(
                        "[{},{},{}]",
                        json_str(number),
                        json_str(&format!("{}mm", crate::emit::geom::mm_femto(*x))),
                        json_str(&format!("{}mm", crate::emit::geom::mm_femto(*y))),
                    )
                })
                .collect::<Vec<_>>()
                .join(",")
        );
        let _ = write!(
            out,
            "\n      \"pads\": {},\n      \"frame\": {}\n    }}",
            canonical,
            json_str(&crate::hash::sha256_hex(canonical.as_bytes())),
        );
    }
    out.push_str("\n  }");
    out
}

fn emit_one(world: &World, ir: &DesignIr, selection: &SupplierSelection) -> SupplierMap {
    let mut footprints = BTreeSet::new();
    let mut designators = Vec::new();
    for instance in selection.instances.values() {
        let Some(footprint) = world.footprints.get(&instance.footprint) else {
            continue;
        };
        if footprint
            .mount_holes
            .iter()
            .any(|hole| hole.plating == crate::ast::MountHolePlating::SourceUnspecified)
        {
            // A physical-only unknown is retained by --logical-only without
            // invoking its checked physical projection invariant.
            continue;
        }
        if !crate::emit::kicad_mod::pad_plans(world, footprint)
            .iter()
            .any(|pad| !pad.number.is_empty() && pad.kind == "smd")
        {
            continue;
        }
        footprints.insert(instance.footprint.clone());
        designators.push(instance);
    }
    designators.sort_by(|left, right| {
        crate::emit::designator_sort_key(&left.designator)
            .cmp(&crate::emit::designator_sort_key(&right.designator))
            .then_with(|| left.instance.cmp(&right.instance))
    });

    let mut json =
        String::from("{\n  \"schema_version\": 1,\n  \"pad_projection\": 1,\n  \"footprints\": ");
    json.push_str(&render_footprints(world, &footprints));
    json.push_str(",\n  \"designators\": {");
    for instance in designators {
        let _ = write!(json, "\n    {}: {{", json_str(&instance.designator));
        let _ = write!(
            json,
            "\n      \"part\": {},\n      \"footprint\": {},",
            json_str(&instance.part),
            json_str(&instance.footprint),
        );
        let part = world.parts.get(&instance.part);
        let (entry, mfr, mpn) = instance.selected.as_ref().map_or_else(
            || {
                let entry = part.map(|part| &part.primary);
                (
                    "\"primary\"".to_string(),
                    entry
                        .and_then(|entry| entry.field("mfr"))
                        .map_or("", |field| field.value.as_str()),
                    entry
                        .and_then(|entry| entry.field("mpn"))
                        .map_or("", |field| field.value.as_str()),
                )
            },
            |selected| {
                (
                    entry_json(selected.entry),
                    selected.mfr.as_str(),
                    selected.mpn.as_str(),
                )
            },
        );
        let mapping = instance
            .selected
            .as_ref()
            .map(|selected| selected.mapping)
            .unwrap_or_default();
        {
            let _ = write!(
                json,
                "\n      \"entry\": {},\n      \"mfr\": {},\n      \"mpn\": {},\n      \"pn\": {},\n      \"at\": [{}, {}],\n      \"rotate\": {}",
                entry,
                json_str(mfr),
                json_str(mpn),
                instance.selected.as_ref().map_or_else(|| "null".to_string(), |selected| json_str(&selected.part_number)),
                json_str(&format!("{}mm", crate::emit::geom::mm_femto(mapping.at.0))),
                json_str(&format!("{}mm", crate::emit::geom::mm_femto(mapping.at.1))),
                mapping.rotate,
            );
        }
        json.push_str("\n    },");
    }
    if json.ends_with(',') {
        json.pop();
    }
    json.push_str("\n  }\n}\n");
    SupplierMap {
        supplier: selection.supplier.clone(),
        json,
        cpl: crate::emit::cpl::emit_supplier_cpl(world, ir, selection),
    }
}

pub fn emit_supplier_maps(
    world: &World,
    ir: &DesignIr,
    selections: &SupplierSelections,
) -> Vec<SupplierMap> {
    selections
        .selections
        .iter()
        .map(|selection| emit_one(world, ir, selection))
        .collect()
}
