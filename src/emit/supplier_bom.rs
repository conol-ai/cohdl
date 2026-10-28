//! RFC-034 §3.2 per-supplier BOM CSV.
//!
//! Selection is already complete in `SupplierSelection`; this emitter only
//! groups those per-instance choices. The plain BOM has its own emitter and is
//! deliberately not involved in supplier output.

use crate::resolve::{short, World};
use crate::supplier::{SupplierSelection, SupplierSelections};
use std::collections::BTreeMap;

/// Fully rendered supplier BOM, ready for the CLI's unified artifact writer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupplierBom {
    pub supplier: String,
    pub csv: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum GroupKey {
    Bound {
        pn: String,
        mfr: String,
        mpn: String,
    },
    Unbound {
        mpn: String,
        mfr: String,
        part: String,
    },
}

impl GroupKey {
    /// Sort/group tie-break cells: supplier PN, MPN, manufacturer, then part.
    fn cells(&self) -> (&str, &str, &str, &str) {
        match self {
            Self::Bound { pn, mfr, mpn } => (pn, mpn, mfr, ""),
            Self::Unbound { mpn, mfr, part } => ("", mpn, mfr, part),
        }
    }

    fn comment(&self) -> &str {
        self.cells().1
    }

    fn pn(&self) -> &str {
        self.cells().0
    }
}

fn csv_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn field(part: Option<&crate::ast::PartDef>, name: &str) -> String {
    part.and_then(|part| part.primary.field(name))
        .map(|field| field.value.clone())
        .unwrap_or_default()
}

fn emit_one(world: &World, selection: &SupplierSelection) -> String {
    let mut groups: BTreeMap<GroupKey, (Vec<String>, String)> = BTreeMap::new();
    for instance in selection.instances.values() {
        let key = match &instance.selected {
            Some(selected) => GroupKey::Bound {
                pn: selected.part_number.clone(),
                mfr: selected.mfr.clone(),
                mpn: selected.mpn.clone(),
            },
            None => {
                let part = world.parts.get(&instance.part);
                let mpn = field(part, "mpn");
                let unknown_identity = if mpn.is_empty() {
                    instance.part.clone()
                } else {
                    String::new()
                };
                GroupKey::Unbound {
                    mpn,
                    mfr: field(part, "mfr"),
                    part: unknown_identity,
                }
            }
        };
        let entry = groups
            .entry(key)
            .or_insert_with(|| (Vec::new(), short(&instance.footprint).to_string()));
        entry.0.push(instance.designator.clone());
    }

    let mut rows: Vec<_> = groups.into_iter().collect();
    rows.sort_by(|(left, _), (right, _)| left.cells().cmp(&right.cells()));

    let mut out = format!(
        "Comment,Designator,Footprint,{} Part #\n",
        selection.supplier.to_uppercase()
    );
    for (key, (mut designators, footprint)) in rows {
        designators.sort_by_key(|designator| crate::emit::designator_sort_key(designator));
        out.push_str(&format!(
            "{},{},{},{}\n",
            csv_quote(key.comment()),
            csv_quote(&designators.join(",")),
            csv_quote(&footprint),
            csv_quote(key.pn()),
        ));
    }
    out
}

/// Render every requested supplier from the shared, per-instance selections.
pub fn emit_supplier_boms(world: &World, selections: &SupplierSelections) -> Vec<SupplierBom> {
    selections
        .selections
        .iter()
        .map(|selection| SupplierBom {
            supplier: selection.supplier.clone(),
            csv: emit_one(world, selection),
        })
        .collect()
}
