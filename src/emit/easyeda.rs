//! LCEDA Pro / EasyEDA Pro netlist emitter (`.enet` JSON, format version
//! 2.0.0) — `cohdl build --emit easyeda`.
//!
//! Imported via **File → Import → Netlist** in LCEDA Pro (立创EDA) /
//! EasyEDA Pro. The document shape is pinned against the v1 compiler's
//! emitter (`legacy` branch, `crates/cohdl-codegen-lceda`), whose files the
//! importer accepted: the top level keeps v1's struct order, every JSON
//! object's keys are plain string-sorted (v1's `serde_json` BTreeMap bytes
//! — `gge1` before `gge10` before `gge2`; key order carries no meaning to a
//! JSON importer), and the indentation is the two-space pretty style.
//!
//! Semantics are the `.net` emitter's, re-projected through the SAME
//! derivations — designator natural order, `principal_value`, the resolved
//! footprint symbol's fq path, physical-pin expansion via
//! `Device::pins_for` — so the two netlists cannot disagree. Every instance
//! is a component (a pin-less mechanical part gets an empty `pinInfoMap`),
//! every connected logical pin expands to one `pinInfoMap` entry per
//! physical pad, nets bind by name, and `nc` pins are represented by their
//! guaranteed absence (the same convention the `.net` documents).

use crate::ir::DesignIr;
use crate::resolve::World;
use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::json::json_str;

pub fn emit_enet(world: &World, ir: &DesignIr) -> String {
    // (instance path, logical pin) → net name. Merged nets are disjoint, so
    // a pin resolves to at most one name.
    let mut pin_net: BTreeMap<(&str, &str), &str> = BTreeMap::new();
    for net in &ir.nets {
        for (path, pin) in &net.members {
            pin_net.insert((path.as_str(), pin.as_str()), net.name.as_str());
        }
    }

    // Unique IDs count up in designator natural order (v1 counted its own
    // instance order; the designator order is this project's stable one).
    let mut insts: Vec<_> = ir.instances.values().collect();
    insts.sort_by_key(|i| super::designator_sort_key(i.designator.as_deref().unwrap_or("")));

    let mut components: BTreeMap<String, String> = BTreeMap::new();
    for (idx, inst) in insts.iter().enumerate() {
        let unique_id = format!("gge{}", idx + 1);
        let body = component(world, inst, &unique_id, &pin_net);
        components.insert(unique_id, body);
    }

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"version\": \"2.0.0\",\n");

    // ---- components ----
    if components.is_empty() {
        out.push_str("  \"components\": {},\n");
    } else {
        out.push_str("  \"components\": {\n");
        let last = components.len() - 1;
        for (i, (id, body)) in components.iter().enumerate() {
            let _ = write!(out, "    {}: {}", json_str(id), body);
            out.push_str(if i < last { ",\n" } else { "\n" });
        }
        out.push_str("  },\n");
    }

    // ---- designRule (v1 struct order: trackPhysics, then netRule): an
    // empty global trackPhysics plus one netRule row per net, each with the
    // empty TrackPhysics binding v1 wrote.
    out.push_str("  \"designRule\": {\n");
    out.push_str("    \"trackPhysics\": {},\n");
    if ir.nets.is_empty() {
        out.push_str("    \"netRule\": {}\n");
    } else {
        out.push_str("    \"netRule\": {\n");
        let names: BTreeMap<&str, ()> = ir.nets.iter().map(|n| (n.name.as_str(), ())).collect();
        let last = names.len() - 1;
        for (i, (name, ())) in names.iter().enumerate() {
            let _ = writeln!(out, "      {}: {{", json_str(name));
            let _ = writeln!(out, "        \"net\": {},", json_str(name));
            out.push_str("        \"ruleMap\": {\n");
            out.push_str("          \"TrackPhysics\": \"\"\n");
            out.push_str("        }\n");
            out.push_str(if i < last { "      },\n" } else { "      }\n" });
        }
        out.push_str("    }\n");
    }
    out.push_str("  },\n");

    out.push_str("  \"differentialPair\": {},\n");
    out.push_str("  \"netClass\": {},\n");
    out.push_str("  \"equalLengthNetGroup\": {}\n");
    out.push_str("}\n");
    out
}

/// One component object (rendered from its opening brace; closing brace at
/// the component's four-space indent).
fn component(
    world: &World,
    inst: &crate::ir::IrInstance,
    unique_id: &str,
    pin_net: &BTreeMap<(&str, &str), &str>,
) -> String {
    let part = inst.part.as_ref().and_then(|p| world.parts.get(p));
    let footprint = part
        .and_then(|p| p.primary.footprint.as_ref())
        .map(|f| f.name.as_str())
        .unwrap_or("");

    // props, string-sorted keys. The fixed pairs mirror v1; Manufacturer /
    // Manufacturer Part appear only when the bound part carries them.
    let mut props: BTreeMap<&str, String> = BTreeMap::new();
    props.insert("Add into BOM", "yes".to_string());
    props.insert("Convert to PCB", "yes".to_string());
    props.insert(
        "Designator",
        inst.designator.as_deref().unwrap_or("?").to_string(),
    );
    props.insert(
        "DeviceName",
        crate::resolve::short(&inst.device).to_string(),
    );
    props.insert("FootprintName", footprint.to_string());
    if let Some(mfr) = part.and_then(|p| p.primary.field("mfr")) {
        props.insert("Manufacturer", mfr.value.clone());
    }
    if let Some(mpn) = part.and_then(|p| p.primary.field("mpn")) {
        props.insert("Manufacturer Part", mpn.value.clone());
    }
    props.insert("Name", super::kicad::principal_value(world, inst));
    props.insert("Unique ID", unique_id.to_string());

    // pinInfoMap: one row per physical pad of each connected logical pin,
    // keyed by pad number (string-sorted, like everything else).
    let mut pins: BTreeMap<String, String> = BTreeMap::new();
    let device = &world.devices[&inst.device];
    for dev_pin in device.pins_for(inst.variant.as_deref()) {
        let Some(net) = pin_net.get(&(inst.path.as_str(), dev_pin.name.name.as_str())) else {
            continue;
        };
        for num in &dev_pin.numbers {
            let mut row = String::new();
            row.push_str("{\n");
            let _ = writeln!(row, "          \"name\": {},", json_str(&dev_pin.name.name));
            let _ = writeln!(row, "          \"number\": {},", json_str(&num.text));
            let _ = writeln!(row, "          \"net\": {},", json_str(net));
            row.push_str("          \"props\": {\n");
            let _ = writeln!(row, "            \"Pin Number\": {}", json_str(&num.text));
            row.push_str("          }\n");
            row.push_str("        }");
            pins.insert(num.text.clone(), row);
        }
    }

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("      \"props\": {\n");
    let last = props.len() - 1;
    for (i, (k, v)) in props.iter().enumerate() {
        let _ = write!(out, "        {}: {}", json_str(k), json_str(v));
        out.push_str(if i < last { ",\n" } else { "\n" });
    }
    out.push_str("      },\n");
    if pins.is_empty() {
        out.push_str("      \"pinInfoMap\": {}\n");
    } else {
        out.push_str("      \"pinInfoMap\": {\n");
        let last = pins.len() - 1;
        for (i, (num, row)) in pins.iter().enumerate() {
            let _ = write!(out, "        {}: {}", json_str(num), row);
            out.push_str(if i < last { ",\n" } else { "\n" });
        }
        out.push_str("      }\n");
    }
    out.push_str("    }");
    out
}
