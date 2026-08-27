//! KiCad legacy `.net` (S-expression, "version D") emitter.
//!
//! Faithful and lossless (Constitution): every instance becomes a `comp`,
//! every merged net a `net`, every physical pin of a connected logical pin a
//! `node`. Pins marked `nc` join no net — that is KiCad's own no-connect
//! convention for legacy netlists (see docs/provisional-syntax.md §5 note in
//! DR-012: the emitter represents nc pins by their guaranteed absence).

use crate::ir::DesignIr;
use crate::resolve::World;
use std::fmt::Write as _;

pub fn emit_kicad_net(world: &World, ir: &DesignIr) -> String {
    let mut out = String::new();
    out.push_str("(export (version D)\n");

    // ---- components, sorted by designator (natural order) ----
    let mut insts: Vec<_> = ir.instances.values().collect();
    insts.sort_by_key(|i| super::designator_sort_key(i.designator.as_deref().unwrap_or("")));

    out.push_str("  (components\n");
    for (idx, inst) in insts.iter().enumerate() {
        let refdes = inst.designator.as_deref().unwrap_or("?");
        let value = principal_value(world, inst);
        let part = inst.part.as_ref().and_then(|p| world.parts.get(p));
        // RFC-017: the footprint field emits the resolved footprint SYMBOL's
        // fully-qualified path (real geometry projection is RFC-018's).
        let footprint = part
            .and_then(|p| p.primary.footprint.as_ref())
            .map(|f| f.name.as_str())
            .unwrap_or("");
        let mpn = part
            .and_then(|p| p.primary.field("mpn"))
            .map(|f| f.value.as_str());
        let mfr = part
            .and_then(|p| p.primary.field("mfr"))
            .map(|f| f.value.as_str());

        let _ = writeln!(
            out,
            "    (comp (ref {}) (value {})",
            quote(refdes),
            quote(&value)
        );
        let _ = writeln!(out, "      (footprint {})", quote(footprint));
        let _ = writeln!(
            out,
            "      (libsource (lib \"cohdl\") (part {}) (description \"\"))",
            quote(crate::resolve::short(&inst.device))
        );
        if mpn.is_some() || mfr.is_some() {
            out.push_str("      (fields\n");
            if let Some(mpn) = mpn {
                let _ = writeln!(out, "        (field (name \"MPN\") {})", quote(mpn));
            }
            if let Some(mfr) = mfr {
                let _ = writeln!(out, "        (field (name \"MFR\") {})", quote(mfr));
            }
            out.push_str("      )\n");
        }
        out.push_str("      (sheetpath (names \"/\") (tstamps \"/\"))\n");
        let _ = writeln!(out, "      (tstamp \"{:08X}\")", idx);
        out.push_str("    )\n");
    }
    out.push_str("  )\n");

    // ---- nets, sorted by name, codes sequential from 1 ----
    out.push_str("  (nets\n");
    for (code, net) in ir.nets.iter().enumerate() {
        let _ = writeln!(
            out,
            "    (net (code {}) (name {})",
            code + 1,
            quote(&net.name)
        );
        // One node per PHYSICAL pin number of each connected logical pin —
        // pads in layout are physical.
        let mut nodes: Vec<(String, String)> = Vec::new();
        for (path, pin) in &net.members {
            let inst = &ir.instances[path];
            let refdes = inst.designator.as_deref().unwrap_or("?").to_string();
            let device = &world.devices[&inst.device];
            // RFC-008: physical pin numbers come from the selected variant.
            if let Some(dev_pin) = device
                .pins_for(inst.variant.as_deref())
                .iter()
                .find(|p| p.name.name == *pin)
            {
                for num in &dev_pin.numbers {
                    nodes.push((refdes.clone(), num.text.clone()));
                }
            }
        }
        nodes.sort_by(|a, b| {
            (super::designator_sort_key(&a.0), pin_sort_key(&a.1))
                .cmp(&(super::designator_sort_key(&b.0), pin_sort_key(&b.1)))
        });
        for (refdes, pin) in nodes {
            let _ = writeln!(
                out,
                "      (node (ref {}) (pin {}))",
                quote(&refdes),
                quote(&pin)
            );
        }
        out.push_str("    )\n");
    }
    out.push_str("  )\n");
    out.push_str(")\n");
    out
}

/// The "value" field shown in KiCad: the domain-primary spec when the device
/// has one (capacitance for capacitors, …), else the device name.
pub(crate) fn principal_value(world: &World, inst: &crate::ir::IrInstance) -> String {
    const PRINCIPAL: [&str; 4] = ["capacitance", "resistance", "inductance", "frequency"];
    for field in PRINCIPAL {
        if let Some(v) = inst.specs.get(field) {
            return v.text.clone();
        }
    }
    let _ = world;
    crate::resolve::short(&inst.device).to_string()
}

pub(crate) fn pin_sort_key(pin: &str) -> (u64, String) {
    match pin.parse::<u64>() {
        Ok(n) => (n, String::new()),
        Err(_) => (u64::MAX, pin.to_string()),
    }
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}
