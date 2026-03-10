//! KiCad legacy netlist emitter (`.net` XML-based format, KiCad 5/6).
//!
//! Generates a netlist string from a [`ConnectivityIR`] that can be imported
//! into KiCad via **File → Import Netlist**.

use std::collections::HashMap;
use std::fmt::Write;

use cohdl_sema::connectivity::{ConnectivityIR, Instance, PinRef};
use cohdl_sema::typeck::EXTERNAL_INSTANCE;

// ── Footprint mapping ────────────────────────────────────────────────────────

/// Default footprint lookup table mapping cohdl package names to KiCad
/// footprint library identifiers.
fn default_footprint_table() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("C0402", "Capacitor_SMD:C_0402_1005Metric"),
        ("C0603", "Capacitor_SMD:C_0603_1608Metric"),
        ("R0402", "Resistor_SMD:R_0402_1005Metric"),
        ("R0603", "Resistor_SMD:R_0603_1608Metric"),
        ("LQFP48", "Package_QFP:LQFP-48_7x7mm_P0.5mm"),
        ("LQFP64", "Package_QFP:LQFP-64_10x10mm_P0.5mm"),
        ("SOT-23", "Package_TO_SOT_SMD:SOT-23"),
    ])
}

/// Configuration for the KiCad netlist emitter.
pub struct KicadNetlistConfig {
    /// Maps cohdl package/device names to KiCad footprint strings.
    pub footprint_table: HashMap<String, String>,
}

impl Default for KicadNetlistConfig {
    fn default() -> Self {
        Self {
            footprint_table: default_footprint_table()
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}

// ── Emitter ──────────────────────────────────────────────────────────────────

/// Emit a KiCad legacy netlist (`.net` XML format) from a [`ConnectivityIR`].
///
/// Uses the default footprint table. For custom mappings, use
/// [`emit_kicad_netlist_with_config`].
pub fn emit_kicad_netlist(ir: &ConnectivityIR) -> String {
    emit_kicad_netlist_with_config(ir, &KicadNetlistConfig::default())
}

/// Emit a KiCad legacy netlist with a custom configuration.
pub fn emit_kicad_netlist_with_config(ir: &ConnectivityIR, config: &KicadNetlistConfig) -> String {
    let mut out = String::new();

    // Build a lookup from InstanceId → Instance for resolving pin refs.
    let instance_by_id: HashMap<u32, &Instance> =
        ir.instances.iter().map(|inst| (inst.id.0, inst)).collect();

    // ── Header
    writeln!(out, "(export (version D)").unwrap();

    // ── Components
    writeln!(out, "  (components").unwrap();
    for inst in &ir.instances {
        let footprint = resolve_footprint(&inst.device, config);
        let value = inst
            .generic_substitutions
            .get("value")
            .cloned()
            .unwrap_or_else(|| inst.device.clone());

        writeln!(out, "    (comp (ref \"{}\") (value \"{}\")", inst.name, xml_escape(&value)).unwrap();
        writeln!(out, "      (footprint \"{}\")", xml_escape(&footprint)).unwrap();
        if let Some(mpn) = &inst.mpn {
            writeln!(out, "      (fields (field (name \"MPN\") \"{}\"))", xml_escape(mpn)).unwrap();
        }
        writeln!(out, "    )").unwrap();
    }
    writeln!(out, "  )").unwrap();

    // ── Nets
    writeln!(out, "  (nets").unwrap();
    let mut code = 1u32;
    for net in &ir.nets {
        // Collect only non-external pins.
        let nodes: Vec<&PinRef> = net
            .pins
            .iter()
            .filter(|p| p.instance_id != EXTERNAL_INSTANCE)
            .collect();

        // Skip nets that have no real component pins.
        if nodes.is_empty() {
            continue;
        }

        write!(out, "    (net (code {}) (name \"{}\")", code, xml_escape(&net.name)).unwrap();

        for pin in &nodes {
            if let Some(inst) = instance_by_id.get(&pin.instance_id.0) {
                write!(out, "\n      (node (ref \"{}\") (pin \"{}\"))", inst.name, xml_escape(&pin.pin)).unwrap();
            }
        }

        writeln!(out, "\n    )").unwrap();
        code += 1;
    }
    writeln!(out, "  )").unwrap();

    // ── Close export
    writeln!(out, ")").unwrap();

    out
}

/// Resolve a cohdl device name to a KiCad footprint string.
fn resolve_footprint(device: &str, config: &KicadNetlistConfig) -> String {
    config
        .footprint_table
        .get(device)
        .cloned()
        .unwrap_or_else(|| format!("Unknown:{}", device))
}

/// Minimal XML/S-expression escaping for string values.
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use cohdl_sema::connectivity::{ConnectivityIR, Instance, Net, PinRef};
    use cohdl_sema::typeck::{InstanceId, EXTERNAL_INSTANCE};
    use std::collections::HashMap;

    /// Build a fixture IR representing a simple board with an MCU, a resistor,
    /// and a capacitor connected via two nets.
    fn fixture_ir() -> ConnectivityIR {
        ConnectivityIR {
            instances: vec![
                Instance {
                    id: InstanceId(0),
                    name: "U1".into(),
                    hierarchical_path: "Board::U1".into(),
                    device: "LQFP48".into(),
                    mpn: Some("STM32F103C8T6".into()),
                    generic_substitutions: {
                        let mut m = HashMap::new();
                        m.insert("value".to_string(), "STM32F103".to_string());
                        m
                    },
                },
                Instance {
                    id: InstanceId(1),
                    name: "R1".into(),
                    hierarchical_path: "Board::R1".into(),
                    device: "R0402".into(),
                    mpn: None,
                    generic_substitutions: {
                        let mut m = HashMap::new();
                        m.insert("value".to_string(), "10k".to_string());
                        m
                    },
                },
                Instance {
                    id: InstanceId(2),
                    name: "C1".into(),
                    hierarchical_path: "Board::C1".into(),
                    device: "C0402".into(),
                    mpn: None,
                    generic_substitutions: {
                        let mut m = HashMap::new();
                        m.insert("value".to_string(), "100nF".to_string());
                        m
                    },
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
                            pin: "VDD_IO".into(),
                        },
                        PinRef {
                            instance_id: InstanceId(1),
                            pin: "A".into(),
                        },
                        PinRef {
                            instance_id: InstanceId(2),
                            pin: "A".into(),
                        },
                    ],
                },
                Net {
                    name: "GND".into(),
                    pins: vec![
                        PinRef {
                            instance_id: EXTERNAL_INSTANCE,
                            pin: "GND".into(),
                        },
                        PinRef {
                            instance_id: InstanceId(1),
                            pin: "B".into(),
                        },
                        PinRef {
                            instance_id: InstanceId(2),
                            pin: "B".into(),
                        },
                    ],
                },
                Net {
                    name: "PA0".into(),
                    pins: vec![
                        PinRef {
                            instance_id: InstanceId(0),
                            pin: "PA0".into(),
                        },
                        PinRef {
                            instance_id: InstanceId(1),
                            pin: "A".into(),
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn netlist_has_export_header() {
        let netlist = emit_kicad_netlist(&fixture_ir());
        assert!(netlist.starts_with("(export (version D)"));
    }

    #[test]
    fn netlist_contains_components() {
        let netlist = emit_kicad_netlist(&fixture_ir());
        assert!(netlist.contains("(components"));
        assert!(netlist.contains(r#"(comp (ref "U1") (value "STM32F103")"#));
        assert!(netlist.contains(r#"(comp (ref "R1") (value "10k")"#));
        assert!(netlist.contains(r#"(comp (ref "C1") (value "100nF")"#));
    }

    #[test]
    fn netlist_contains_footprints() {
        let netlist = emit_kicad_netlist(&fixture_ir());
        assert!(netlist.contains(r#"(footprint "Package_QFP:LQFP-48_7x7mm_P0.5mm")"#));
        assert!(netlist.contains(r#"(footprint "Resistor_SMD:R_0402_1005Metric")"#));
        assert!(netlist.contains(r#"(footprint "Capacitor_SMD:C_0402_1005Metric")"#));
    }

    #[test]
    fn netlist_contains_mpn_field() {
        let netlist = emit_kicad_netlist(&fixture_ir());
        assert!(netlist.contains(r#"(field (name "MPN") "STM32F103C8T6")"#));
        // R1 has no MPN – should not emit a fields block for it.
        // Count occurrences of "MPN" – should be exactly 1.
        assert_eq!(netlist.matches("MPN").count(), 1);
    }

    #[test]
    fn netlist_contains_nets() {
        let netlist = emit_kicad_netlist(&fixture_ir());
        assert!(netlist.contains("(nets"));
        assert!(netlist.contains(r#"(name "VDD")"#));
        assert!(netlist.contains(r#"(name "GND")"#));
        assert!(netlist.contains(r#"(name "PA0")"#));
    }

    #[test]
    fn netlist_net_nodes_reference_components() {
        let netlist = emit_kicad_netlist(&fixture_ir());
        // VDD net should reference U1.VDD_IO, R1.A, C1.A
        assert!(netlist.contains(r#"(node (ref "U1") (pin "VDD_IO"))"#));
        assert!(netlist.contains(r#"(node (ref "C1") (pin "A"))"#));
    }

    #[test]
    fn netlist_excludes_external_pins_from_nodes() {
        let netlist = emit_kicad_netlist(&fixture_ir());
        // EXTERNAL_INSTANCE id is u32::MAX – should not appear as a node ref.
        assert!(!netlist.contains(&format!("ref \"{}\"", u32::MAX)));
    }

    #[test]
    fn netlist_net_codes_are_sequential() {
        let netlist = emit_kicad_netlist(&fixture_ir());
        assert!(netlist.contains("(net (code 1)"));
        assert!(netlist.contains("(net (code 2)"));
        assert!(netlist.contains("(net (code 3)"));
        assert!(!netlist.contains("(net (code 0)"));
    }

    #[test]
    fn empty_ir_produces_valid_netlist() {
        let ir = ConnectivityIR {
            instances: vec![],
            nets: vec![],
        };
        let netlist = emit_kicad_netlist(&ir);
        assert!(netlist.contains("(export (version D)"));
        assert!(netlist.contains("(components"));
        assert!(netlist.contains("(nets"));
    }

    #[test]
    fn unknown_device_gets_fallback_footprint() {
        let ir = ConnectivityIR {
            instances: vec![Instance {
                id: InstanceId(0),
                name: "X1".into(),
                hierarchical_path: "Board::X1".into(),
                device: "MY_CUSTOM_PKG".into(),
                mpn: None,
                generic_substitutions: HashMap::new(),
            }],
            nets: vec![],
        };
        let netlist = emit_kicad_netlist(&ir);
        assert!(netlist.contains(r#"(footprint "Unknown:MY_CUSTOM_PKG")"#));
    }

    #[test]
    fn custom_footprint_config() {
        let ir = ConnectivityIR {
            instances: vec![Instance {
                id: InstanceId(0),
                name: "U1".into(),
                hierarchical_path: "Board::U1".into(),
                device: "QFN32".into(),
                mpn: None,
                generic_substitutions: HashMap::new(),
            }],
            nets: vec![],
        };
        let mut config = KicadNetlistConfig::default();
        config
            .footprint_table
            .insert("QFN32".into(), "Package_DFN_QFN:QFN-32_5x5mm".into());
        let netlist = emit_kicad_netlist_with_config(&ir, &config);
        assert!(netlist.contains(r#"(footprint "Package_DFN_QFN:QFN-32_5x5mm")"#));
    }

    #[test]
    fn xml_special_chars_are_escaped() {
        let ir = ConnectivityIR {
            instances: vec![Instance {
                id: InstanceId(0),
                name: "U1".into(),
                hierarchical_path: "Board::U1".into(),
                device: "R0402".into(),
                mpn: Some("Part<1>&\"2\"".into()),
                generic_substitutions: {
                    let mut m = HashMap::new();
                    m.insert("value".to_string(), "R<10>".to_string());
                    m
                },
            }],
            nets: vec![],
        };
        let netlist = emit_kicad_netlist(&ir);
        assert!(netlist.contains("R&lt;10&gt;"));
        assert!(netlist.contains("Part&lt;1&gt;&amp;&quot;2&quot;"));
    }
}
