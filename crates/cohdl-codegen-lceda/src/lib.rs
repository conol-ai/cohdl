//! LCEDA Pro (立创EDA) netlist emitter (`.enet` JSON format).
//!
//! Generates a netlist string from a [`ConnectivityIR`] that can be imported
//! into LCEDA Pro via **File → Import → Netlist**.

use std::collections::HashMap;

use cohdl_sema::connectivity::{ConnectivityIR, Instance};
use cohdl_sema::typeck::{ResolvedFootprint, EXTERNAL_INSTANCE};
use serde::Serialize;

// ── JSON schema types ───────────────────────────────────────────────────────

#[derive(Serialize)]
struct EnetFile {
    version: &'static str,
    components: serde_json::Map<String, serde_json::Value>,
    #[serde(rename = "designRule")]
    design_rule: DesignRule,
    #[serde(rename = "differentialPair")]
    differential_pair: serde_json::Map<String, serde_json::Value>,
    #[serde(rename = "netClass")]
    net_class: serde_json::Map<String, serde_json::Value>,
    #[serde(rename = "equalLengthNetGroup")]
    equal_length_net_group: serde_json::Map<String, serde_json::Value>,
}

#[derive(Serialize)]
struct DesignRule {
    #[serde(rename = "trackPhysics")]
    track_physics: serde_json::Map<String, serde_json::Value>,
    #[serde(rename = "netRule")]
    net_rule: serde_json::Map<String, serde_json::Value>,
}

#[derive(Serialize)]
struct ComponentEntry {
    props: serde_json::Map<String, serde_json::Value>,
    #[serde(rename = "pinInfoMap")]
    pin_info_map: serde_json::Map<String, serde_json::Value>,
}

#[derive(Serialize)]
struct PinInfo {
    name: String,
    number: String,
    net: String,
    props: serde_json::Map<String, serde_json::Value>,
}

// ── Emitter ─────────────────────────────────────────────────────────────────

/// Emit an LCEDA Pro netlist (`.enet` JSON format) from a [`ConnectivityIR`].
pub fn emit_lceda_netlist(ir: &ConnectivityIR) -> String {
    // Build reverse index: instance_id → vec of (pin_name, net_name).
    let mut inst_pins: HashMap<u32, Vec<(&str, &str)>> = HashMap::new();
    for net in &ir.nets {
        for pin_ref in &net.pins {
            if pin_ref.instance_id == EXTERNAL_INSTANCE {
                continue;
            }
            inst_pins
                .entry(pin_ref.instance_id.0)
                .or_default()
                .push((&pin_ref.pin, &net.name));
        }
    }

    let mut components = serde_json::Map::new();

    for (idx, inst) in ir.instances.iter().enumerate() {
        let unique_id = format!("gge{}", idx + 1);
        let comp = build_component(inst, &unique_id, inst_pins.get(&inst.id.0));
        components.insert(unique_id, serde_json::to_value(comp).unwrap());
    }

    // Collect all unique net names for designRule.netRule.
    let mut net_rule = serde_json::Map::new();
    for net in &ir.nets {
        let has_internal_pin = net.pins.iter().any(|p| p.instance_id != EXTERNAL_INSTANCE);
        if !has_internal_pin {
            continue;
        }
        let mut rule_map = serde_json::Map::new();
        rule_map.insert(
            "TrackPhysics".into(),
            serde_json::Value::String(String::new()),
        );

        let mut entry = serde_json::Map::new();
        entry.insert("net".into(), serde_json::Value::String(net.name.clone()));
        entry.insert("ruleMap".into(), serde_json::Value::Object(rule_map));
        net_rule.insert(net.name.clone(), serde_json::Value::Object(entry));
    }

    let file = EnetFile {
        version: "2.0.0",
        components,
        design_rule: DesignRule {
            track_physics: serde_json::Map::new(),
            net_rule,
        },
        differential_pair: serde_json::Map::new(),
        net_class: serde_json::Map::new(),
        equal_length_net_group: serde_json::Map::new(),
    };

    serde_json::to_string_pretty(&file).unwrap()
}

fn build_component(
    inst: &Instance,
    unique_id: &str,
    pins: Option<&Vec<(&str, &str)>>,
) -> ComponentEntry {
    let value = inst
        .generic_substitutions
        .get("value")
        .cloned()
        .unwrap_or_else(|| inst.device.clone());

    let mut props = serde_json::Map::new();

    let set = |m: &mut serde_json::Map<String, serde_json::Value>, k: &str, v: &str| {
        m.insert(k.into(), serde_json::Value::String(v.into()));
    };

    set(&mut props, "Add into BOM", "yes");
    set(&mut props, "Convert to PCB", "yes");
    set(&mut props, "Designator", &inst.name);
    set(&mut props, "Name", &value);
    set(&mut props, "Unique ID", unique_id);
    set(&mut props, "DeviceName", &inst.device);
    let footprint_name = resolve_footprint_for_lceda(&inst.footprint_override, &inst.device);
    set(&mut props, "FootprintName", &footprint_name);

    // Populate MPN-related fields if available.
    if let Some(mpn) = &inst.mpn {
        set(&mut props, "Manufacturer Part", mpn);
    }

    // Supplier Part from generic_substitutions (e.g. LCSC number).
    if let Some(lcsc) = inst.generic_substitutions.get("lcsc") {
        set(&mut props, "Supplier Part", lcsc);
        set(&mut props, "Supplier", "LCSC");
    }

    // Build pinInfoMap using actual device pin numbers when available.
    let mut pin_info_map = serde_json::Map::new();

    if let Some(pins) = pins {
        // Deduplicate and maintain order: a pin name may appear only once per instance.
        let mut seen: Vec<(&str, &str)> = Vec::new();
        for &(pin_name, net_name) in pins {
            if !seen.iter().any(|(p, _)| *p == pin_name) {
                seen.push((pin_name, net_name));
            }
        }

        let mut fallback_counter = 1u32;
        for (pin_name, net_name) in &seen {
            // Look up actual physical pin numbers from the device definition.
            let physical_numbers: Vec<String> = if let Some(nums) = inst.pin_numbers.get(*pin_name)
            {
                nums.clone()
            } else {
                // Fallback: assign sequential numbers.
                let num = fallback_counter.to_string();
                fallback_counter += 1;
                vec![num]
            };

            for num in &physical_numbers {
                let pin = PinInfo {
                    name: pin_name.to_string(),
                    number: num.clone(),
                    net: net_name.to_string(),
                    props: {
                        let mut m = serde_json::Map::new();
                        m.insert("Pin Number".into(), serde_json::Value::String(num.clone()));
                        m
                    },
                };
                pin_info_map.insert(num.clone(), serde_json::to_value(pin).unwrap());
            }
        }
    }

    ComponentEntry {
        props,
        pin_info_map,
    }
}

/// Resolve a `ResolvedFootprint` to an LCEDA footprint string.
fn resolve_footprint_for_lceda(fp: &Option<ResolvedFootprint>, device: &str) -> String {
    match fp {
        Some(ResolvedFootprint::String(s)) => s.clone(),
        Some(ResolvedFootprint::Alias { mappings, .. }) => mappings
            .get("lceda")
            .or_else(|| mappings.get("default"))
            .cloned()
            .unwrap_or_else(|| device.to_string()),
        Some(ResolvedFootprint::InlineMap(map)) => map
            .get("lceda")
            .or_else(|| map.get("default"))
            .cloned()
            .unwrap_or_else(|| device.to_string()),
        Some(ResolvedFootprint::NoFootprint) => String::new(),
        None => device.to_string(),
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use cohdl_sema::connectivity::{ConnectivityIR, Instance, Net, PinRef};
    use cohdl_sema::typeck::{InstanceId, EXTERNAL_INSTANCE};

    fn fixture_ir() -> ConnectivityIR {
        ConnectivityIR {
            instances: vec![
                Instance {
                    id: InstanceId(0),
                    name: "U1".into(),
                    hierarchical_path: "Board::U1".into(),
                    device: "LQFP48".into(),
                    mpn: Some("STM32F103C8T6".into()),
                    alt_mpns: vec![],
                    generic_substitutions: {
                        let mut m = HashMap::new();
                        m.insert("value".to_string(), "STM32F103".to_string());
                        m
                    },
                    footprint_override: None,
                    pin_numbers: HashMap::new(),
                },
                Instance {
                    id: InstanceId(1),
                    name: "C1".into(),
                    hierarchical_path: "Board::C1".into(),
                    device: "C0402".into(),
                    mpn: Some("CL05B104KO5NNNC".into()),
                    alt_mpns: vec![],
                    generic_substitutions: {
                        let mut m = HashMap::new();
                        m.insert("value".to_string(), "100nF".to_string());
                        m.insert("lcsc".to_string(), "C1525".to_string());
                        m
                    },
                    footprint_override: None,
                    pin_numbers: HashMap::new(),
                },
            ],
            nets: vec![
                Net {
                    name: "VDD".into(),
                    pins: vec![
                        PinRef {
                            instance_id: EXTERNAL_INSTANCE,
                            pin: "VDD".into(),
                        },
                        PinRef {
                            instance_id: InstanceId(0),
                            pin: "VDD".into(),
                        },
                        PinRef {
                            instance_id: InstanceId(1),
                            pin: "A".into(),
                        },
                    ],
                },
                Net {
                    name: "GND".into(),
                    pins: vec![
                        PinRef {
                            instance_id: InstanceId(0),
                            pin: "GND".into(),
                        },
                        PinRef {
                            instance_id: InstanceId(1),
                            pin: "B".into(),
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn output_is_valid_json() {
        let output = emit_lceda_netlist(&fixture_ir());
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn version_is_2_0_0() {
        let output = emit_lceda_netlist(&fixture_ir());
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["version"], "2.0.0");
    }

    #[test]
    fn components_are_present() {
        let output = emit_lceda_netlist(&fixture_ir());
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let comps = parsed["components"].as_object().unwrap();
        assert_eq!(comps.len(), 2);
        assert!(comps.contains_key("gge1"));
        assert!(comps.contains_key("gge2"));
    }

    #[test]
    fn component_has_designator() {
        let output = emit_lceda_netlist(&fixture_ir());
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["components"]["gge1"]["props"]["Designator"], "U1");
        assert_eq!(parsed["components"]["gge2"]["props"]["Designator"], "C1");
    }

    #[test]
    fn component_has_mpn() {
        let output = emit_lceda_netlist(&fixture_ir());
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            parsed["components"]["gge1"]["props"]["Manufacturer Part"],
            "STM32F103C8T6"
        );
    }

    #[test]
    fn component_has_lcsc_supplier() {
        let output = emit_lceda_netlist(&fixture_ir());
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            parsed["components"]["gge2"]["props"]["Supplier Part"],
            "C1525"
        );
        assert_eq!(parsed["components"]["gge2"]["props"]["Supplier"], "LCSC");
    }

    #[test]
    fn pin_info_map_has_correct_nets() {
        let output = emit_lceda_netlist(&fixture_ir());
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        // C1 should have 2 pins: A→VDD, B→GND
        let c1_pins = parsed["components"]["gge2"]["pinInfoMap"]
            .as_object()
            .unwrap();
        assert_eq!(c1_pins.len(), 2);

        // Find pin with name "A" and check net
        let pin1 = &c1_pins["1"];
        assert_eq!(pin1["name"], "A");
        assert_eq!(pin1["net"], "VDD");

        let pin2 = &c1_pins["2"];
        assert_eq!(pin2["name"], "B");
        assert_eq!(pin2["net"], "GND");
    }

    #[test]
    fn external_pins_excluded_from_components() {
        let output = emit_lceda_netlist(&fixture_ir());
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let comps = parsed["components"].as_object().unwrap();
        // No component should have EXTERNAL_INSTANCE's id
        for (_, comp) in comps {
            assert_ne!(
                comp["props"]["Unique ID"].as_str().unwrap(),
                format!("gge{}", u32::MAX)
            );
        }
    }

    #[test]
    fn net_rules_are_populated() {
        let output = emit_lceda_netlist(&fixture_ir());
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        let net_rule = parsed["designRule"]["netRule"].as_object().unwrap();
        assert!(net_rule.contains_key("VDD"));
        assert!(net_rule.contains_key("GND"));
    }

    #[test]
    fn empty_ir_produces_valid_output() {
        let ir = ConnectivityIR {
            instances: vec![],
            nets: vec![],
        };
        let output = emit_lceda_netlist(&ir);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["version"], "2.0.0");
        assert_eq!(parsed["components"].as_object().unwrap().len(), 0);
    }

    #[test]
    fn footprint_override_used_when_present() {
        let ir = ConnectivityIR {
            instances: vec![Instance {
                id: InstanceId(0),
                name: "U1".into(),
                hierarchical_path: "Board::U1".into(),
                device: "LQFP48".into(),
                mpn: None,
                alt_mpns: vec![],
                generic_substitutions: HashMap::new(),
                footprint_override: Some(ResolvedFootprint::String("LQFP-48_Custom".into())),
                pin_numbers: HashMap::new(),
            }],
            nets: vec![],
        };
        let output = emit_lceda_netlist(&ir);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            parsed["components"]["gge1"]["props"]["FootprintName"],
            "LQFP-48_Custom"
        );
    }

    #[test]
    fn footprint_falls_back_to_device_when_no_override() {
        let ir = ConnectivityIR {
            instances: vec![Instance {
                id: InstanceId(0),
                name: "U1".into(),
                hierarchical_path: "Board::U1".into(),
                device: "LQFP48".into(),
                mpn: None,
                alt_mpns: vec![],
                generic_substitutions: HashMap::new(),
                footprint_override: None,
                pin_numbers: HashMap::new(),
            }],
            nets: vec![],
        };
        let output = emit_lceda_netlist(&ir);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            parsed["components"]["gge1"]["props"]["FootprintName"],
            "LQFP48"
        );
    }
}
