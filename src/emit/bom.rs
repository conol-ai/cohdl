//! BOM CSV emitter.
//!
//! One row per part group (grouped by MPN), quoted CSV, byte-stable:
//!
//! ```csv
//! RefDes,Value,MPN,Manufacturer,Qty
//! "C1,C2","100nF","CL05B104KO5NNNC","Samsung",2
//! ```

use crate::ir::DesignIr;
use crate::resolve::World;
use std::collections::BTreeMap;

pub fn emit_bom_csv(world: &World, ir: &DesignIr) -> String {
    // MPN → (refdes list, value, manufacturer).
    let mut groups: BTreeMap<String, (Vec<String>, String, String)> = BTreeMap::new();

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
        let value = principal_value(inst);
        let entry = groups
            .entry(mpn)
            .or_insert_with(|| (Vec::new(), value.clone(), mfr));
        entry.0.push(refdes);
    }

    let mut out = String::from("RefDes,Value,MPN,Manufacturer,Qty\n");
    for (mpn, (mut refdes, value, mfr)) in groups {
        refdes.sort_by_key(|d| crate::emit::designator_sort_key(d));
        out.push_str(&format!(
            "{},{},{},{},{}\n",
            csv_quote(&refdes.join(",")),
            csv_quote(&value),
            csv_quote(&mpn),
            csv_quote(&mfr),
            refdes.len()
        ));
    }
    out
}

fn principal_value(inst: &crate::ir::IrInstance) -> String {
    const PRINCIPAL: [&str; 4] = ["capacitance", "resistance", "inductance", "frequency"];
    for field in PRINCIPAL {
        if let Some(v) = inst.specs.get(field) {
            return v.text.clone();
        }
    }
    inst.device.clone()
}

fn csv_quote(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
