//! BOM CSV emitter.
//!
//! One row per part group (grouped by MPN), quoted CSV, byte-stable, in the
//! assembly-house column layout that pairs with `tools/smt_pos.py`'s CPL
//! export — `Comment` carries the model (MPN), `Footprint` the part's
//! footprint symbol (short name, same form as the CPL's Footprint column):
//!
//! ```csv
//! Manufacturer,Comment,Designator,Footprint
//! "Samsung","CL05B104KO5NNNC","C1,C2","CHIP_0402"
//! ```

use crate::ir::DesignIr;
use crate::resolve::World;
use std::collections::BTreeMap;

pub fn emit_bom_csv(world: &World, ir: &DesignIr) -> String {
    // (MPN, manufacturer) → (refdes list, footprint). Keyed by both because
    // an MPN is not unique without its manufacturer — two parts may share an
    // MPN under different manufacturers, and keying by MPN alone dropped the
    // second manufacturer's row entirely (review F5). MPN stays the primary
    // sort key; the IPC-2581 emitter uses the identical key so the two
    // artifacts always agree.
    let mut groups: BTreeMap<(String, String), (Vec<String>, String)> = BTreeMap::new();

    for inst in ir.instances.values() {
        let refdes = inst.designator.clone().unwrap_or_else(|| "?".to_string());
        let part = inst.part.as_ref().and_then(|p| world.parts.get(p));
        let mpn = part
            .and_then(|p| p.primary.field("mpn"))
            .map(|f| f.value.clone())
            .unwrap_or_default();
        let mfr = part
            .and_then(|p| p.primary.field("mfr"))
            .map(|f| f.value.clone())
            .unwrap_or_default();
        let footprint = part
            .and_then(|p| p.primary.footprint.as_ref())
            .map(|f| crate::resolve::short(&f.name).to_string())
            .unwrap_or_default();
        let entry = groups
            .entry((mpn, mfr))
            .or_insert_with(|| (Vec::new(), footprint));
        entry.0.push(refdes);
    }

    let mut out = String::from("Manufacturer,Comment,Designator,Footprint\n");
    for ((mpn, mfr), (mut refdes, footprint)) in groups {
        refdes.sort_by_key(|d| crate::emit::designator_sort_key(d));
        out.push_str(&format!(
            "{},{},{},{}\n",
            csv_quote(&mfr),
            csv_quote(&mpn),
            csv_quote(&refdes.join(",")),
            csv_quote(&footprint),
        ));
    }
    out
}

fn csv_quote(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
