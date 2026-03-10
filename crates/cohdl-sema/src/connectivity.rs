//! Connectivity IR: flattens [`TypedDesign`] into a union-find–merged net list.
//!
//! This module takes the output of the type-checking pass ([`TypedDesign`]) and
//! builds a [`ConnectivityIR`] where:
//!
//! - All `net` statements (including those inside `fn` bodies) are expanded into
//!   a flat net list.
//! - Nets that share a name or have direct pin-to-pin assignments are merged via
//!   a union-find data structure.
//! - Each instance is assigned a stable **hierarchical path** (e.g.
//!   `MainBoard::usb::r_dm`) suitable for designator locking.
//! - Non-existent pin references and wrong bus indices produce errors.

use std::collections::HashMap;

use crate::typeck::{InstanceId, TypedDesign, EXTERNAL_INSTANCE};
use crate::SemaError;
use cohdl_syntax::ast::Span;

// ── Public IR types ─────────────────────────────────────────────────────────

/// A reference to a specific pin on a specific instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PinRef {
    /// The instance this pin belongs to.
    pub instance_id: InstanceId,
    /// The pin name (e.g. `"A"`, `"VDD_IO"`, `"D[3]"`).
    pub pin: String,
}

/// A merged net: a named set of pin references that are electrically connected.
#[derive(Debug, Clone, PartialEq)]
pub struct Net {
    /// Canonical net name (the first name encountered during merging).
    pub name: String,
    /// All pin endpoints on this net.
    pub pins: Vec<PinRef>,
}

/// A component instance with a stable hierarchical path.
#[derive(Debug, Clone, PartialEq)]
pub struct Instance {
    /// Unique id (same as [`ComponentInstance::id`]).
    pub id: InstanceId,
    /// Local instance name (e.g. `"c1"`).
    pub name: String,
    /// Hierarchical path for designator locking (e.g. `"MainBoard::usb::r_dm"`).
    pub hierarchical_path: String,
    /// Resolved device name.
    pub device: String,
    /// Primary MPN if backed by a part.
    pub mpn: Option<String>,
    /// Alternate MPNs from the part's AVL entries.
    pub alt_mpns: Vec<String>,
    /// Generic parameter substitutions.
    pub generic_substitutions: HashMap<String, String>,
}

/// The connectivity IR: a flat list of instances and merged nets.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectivityIR {
    /// All component instances with hierarchical paths.
    pub instances: Vec<Instance>,
    /// All merged nets.
    pub nets: Vec<Net>,
}

/// Result of building the connectivity IR.
#[derive(Debug, Clone)]
pub struct ConnectivityResult {
    /// The connectivity IR (present even if there are errors, for partial results).
    pub ir: ConnectivityIR,
    /// Errors encountered during IR construction.
    pub errors: Vec<SemaError>,
}

// ── Union-Find ──────────────────────────────────────────────────────────────

/// A simple union-find (disjoint-set) data structure with path compression and
/// union by rank.
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u32>,
}

impl UnionFind {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, mut x: usize) -> usize {
        while self.parent[x] != x {
            self.parent[x] = self.parent[self.parent[x]]; // path halving
            x = self.parent[x];
        }
        x
    }

    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return;
        }
        if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb;
        } else if self.rank[ra] > self.rank[rb] {
            self.parent[rb] = ra;
        } else {
            self.parent[rb] = ra;
            self.rank[ra] += 1;
        }
    }
}

// ── Builder ─────────────────────────────────────────────────────────────────

/// Build the [`ConnectivityIR`] from a [`TypedDesign`].
///
/// `device_pins` provides the set of valid pin names for each device name.
/// This is used to validate pin references.
pub fn build_connectivity(
    design: &TypedDesign,
    device_pins: &HashMap<String, Vec<String>>,
) -> ConnectivityResult {
    let mut errors = Vec::new();

    // ── 1. Build instances with hierarchical paths ──────────────────────

    let instances: Vec<Instance> = design
        .instances
        .iter()
        .map(|ci| Instance {
            id: ci.id,
            name: ci.name.clone(),
            hierarchical_path: format!("{}::{}", design.name, ci.name),
            device: ci.device.clone(),
            mpn: ci.mpn.clone(),
            alt_mpns: ci.alt_mpns.clone(),
            generic_substitutions: ci.generic_substitutions.clone(),
        })
        .collect();

    // Map from InstanceId → device name for pin validation.
    let instance_device: HashMap<InstanceId, &str> = design
        .instances
        .iter()
        .map(|ci| (ci.id, ci.device.as_str()))
        .collect();

    // ── 2. Collect all pin refs and validate them ───────────────────────

    // We give each unique PinRef a numeric index for the union-find.
    // We also track net-name → first index, so we can merge by name.
    let mut pin_index: Vec<PinRef> = Vec::new();
    let mut pin_to_idx: HashMap<PinRef, usize> = HashMap::new();
    let mut net_name_to_first: HashMap<String, usize> = HashMap::new();

    // Helper: get-or-insert a PinRef, returning its index.
    let intern_pin = |pr: PinRef,
                      pin_index: &mut Vec<PinRef>,
                      pin_to_idx: &mut HashMap<PinRef, usize>|
     -> usize {
        if let Some(&idx) = pin_to_idx.get(&pr) {
            idx
        } else {
            let idx = pin_index.len();
            pin_to_idx.insert(pr.clone(), idx);
            pin_index.push(pr);
            idx
        }
    };

    // We'll build union-find edges as we go.
    let mut edges: Vec<(usize, usize)> = Vec::new();

    for net in &design.nets {
        let mut ep_indices = Vec::new();

        for (inst_id, pin_name) in &net.endpoints {
            // Validate pin reference (skip external nets).
            if *inst_id != EXTERNAL_INSTANCE {
                if let Some(device_name) = instance_device.get(inst_id) {
                    if let Some(valid_pins) = device_pins.get(*device_name) {
                        // Check for bus index: e.g. "D[3]" → base name "D"
                        let (base_pin, bus_idx) = parse_bus_pin(pin_name);
                        if !valid_pins.iter().any(|p| {
                            let (vbase, _) = parse_bus_pin(p);
                            vbase == base_pin
                        }) {
                            errors.push(SemaError::new(
                                format!(
                                    "pin `{}` does not exist on device `{}`",
                                    pin_name, device_name
                                ),
                                Span { start: 0, end: 0 },
                            ));
                            continue;
                        }
                        // If bus index given, verify it's within declared range.
                        if let Some(idx) = bus_idx {
                            let max_idx = valid_pins
                                .iter()
                                .filter_map(|p| {
                                    let (vb, vi) = parse_bus_pin(p);
                                    if vb == base_pin {
                                        vi
                                    } else {
                                        None
                                    }
                                })
                                .max();
                            if let Some(max) = max_idx {
                                if idx > max {
                                    errors.push(SemaError::new(
                                        format!(
                                            "bus index {} out of range for pin `{}` on device `{}` (max {})",
                                            idx, base_pin, device_name, max
                                        ),
                                        Span { start: 0, end: 0 },
                                    ));
                                    continue;
                                }
                            }
                        }
                    }
                }
            }

            let pr = PinRef {
                instance_id: *inst_id,
                pin: pin_name.clone(),
            };
            let idx = intern_pin(pr, &mut pin_index, &mut pin_to_idx);
            ep_indices.push(idx);
        }

        // All endpoints in this net statement are connected to each other.
        // Chain them: 0-1, 1-2, 2-3, ...
        for w in ep_indices.windows(2) {
            edges.push((w[0], w[1]));
        }

        // Also merge by net name: if two net statements share the same name,
        // they belong to the same logical net.
        if !ep_indices.is_empty() {
            let first_idx = ep_indices[0];
            if let Some(&prev_idx) = net_name_to_first.get(&net.name) {
                edges.push((prev_idx, first_idx));
            } else {
                net_name_to_first.insert(net.name.clone(), first_idx);
            }
        }
    }

    // ── 3. Run union-find to merge nets ─────────────────────────────────

    let n = pin_index.len();
    let mut uf = UnionFind::new(n);
    for (a, b) in &edges {
        uf.union(*a, *b);
    }

    // ── 4. Group pins by their root representative ──────────────────────

    let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
    for i in 0..n {
        let root = uf.find(i);
        groups.entry(root).or_default().push(i);
    }

    // Pick the canonical name for each group: use the net name from the
    // first net statement that contributed a pin in this group.
    // Build a map: pin-index → net-name, for the first endpoint of each net.
    let mut idx_to_net_name: HashMap<usize, String> = HashMap::new();
    for (name, &first_idx) in &net_name_to_first {
        idx_to_net_name.insert(first_idx, name.clone());
    }

    let mut nets: Vec<Net> = Vec::new();
    let mut roots: Vec<usize> = groups.keys().copied().collect();
    roots.sort(); // deterministic output order

    for root in roots {
        let members = &groups[&root];

        // Find canonical net name: check each member for a known net name.
        let name = members
            .iter()
            .find_map(|&idx| {
                let r = uf.find(idx);
                // Check if this specific index was the first for some net name.
                idx_to_net_name.get(&idx).cloned().or_else(|| {
                    // Fallback: find any net name whose first index has same root.
                    net_name_to_first.iter().find_map(|(name, &first)| {
                        if uf.find(first) == r {
                            Some(name.clone())
                        } else {
                            None
                        }
                    })
                })
            })
            .unwrap_or_else(|| format!("__net_{}", root));

        let pins: Vec<PinRef> = members.iter().map(|&idx| pin_index[idx].clone()).collect();

        nets.push(Net { name, pins });
    }

    ConnectivityResult {
        ir: ConnectivityIR { instances, nets },
        errors,
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Parse a pin name that may have a bus index, e.g. `"D[3]"` → `("D", Some(3))`.
/// Plain names like `"A"` return `("A", None)`.
fn parse_bus_pin(pin: &str) -> (&str, Option<u32>) {
    if let Some(bracket) = pin.find('[') {
        let base = &pin[..bracket];
        let rest = &pin[bracket + 1..];
        if let Some(end) = rest.find(']') {
            if let Ok(idx) = rest[..end].parse::<u32>() {
                return (base, Some(idx));
            }
        }
        (base, None)
    } else {
        (pin, None)
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typeck::{ComponentInstance, InstanceId, TypedDesign, TypedNet, EXTERNAL_INSTANCE};
    use std::collections::HashMap;

    /// Build a simple device_pins map for testing.
    fn device_pins_map(entries: &[(&str, &[&str])]) -> HashMap<String, Vec<String>> {
        entries
            .iter()
            .map(|(dev, pins)| {
                (
                    dev.to_string(),
                    pins.iter().map(|p| p.to_string()).collect(),
                )
            })
            .collect()
    }

    fn make_instance(id: u32, name: &str, device: &str) -> ComponentInstance {
        ComponentInstance {
            id: InstanceId(id),
            name: name.to_string(),
            device: device.to_string(),
            mpn: None,
            alt_mpns: Vec::new(),
            generic_substitutions: HashMap::new(),
            designator_override: None,
            impl_traits: Vec::new(),
        }
    }

    // ── Net merging correctness ─────────────────────────────────────────

    #[test]
    fn single_net_statement_connects_all_endpoints() {
        // net VDD: mcu.VDD_IO, c1.A
        let design = TypedDesign {
            name: "Board".into(),
            instances: vec![
                make_instance(0, "mcu", "STM32"),
                make_instance(1, "c1", "MLCC"),
            ],
            nets: vec![TypedNet {
                name: "VDD".into(),
                endpoints: vec![
                    (EXTERNAL_INSTANCE, "VDD".into()),
                    (InstanceId(0), "VDD_IO".into()),
                    (InstanceId(1), "A".into()),
                ],
            }],
        };
        let dp = device_pins_map(&[("STM32", &["VDD_IO", "GND"]), ("MLCC", &["A", "B"])]);
        let result = build_connectivity(&design, &dp);

        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        assert_eq!(result.ir.nets.len(), 1);
        assert_eq!(result.ir.nets[0].name, "VDD");
        assert_eq!(result.ir.nets[0].pins.len(), 3);
    }

    #[test]
    fn nets_with_same_name_are_merged() {
        // Two separate net statements both named "GND" should merge.
        let design = TypedDesign {
            name: "Board".into(),
            instances: vec![
                make_instance(0, "mcu", "STM32"),
                make_instance(1, "c1", "MLCC"),
                make_instance(2, "c2", "MLCC"),
            ],
            nets: vec![
                TypedNet {
                    name: "GND".into(),
                    endpoints: vec![
                        (EXTERNAL_INSTANCE, "GND".into()),
                        (InstanceId(0), "GND".into()),
                        (InstanceId(1), "B".into()),
                    ],
                },
                TypedNet {
                    name: "GND".into(),
                    endpoints: vec![
                        (EXTERNAL_INSTANCE, "GND".into()),
                        (InstanceId(2), "B".into()),
                    ],
                },
            ],
        };
        let dp = device_pins_map(&[("STM32", &["VDD_IO", "GND"]), ("MLCC", &["A", "B"])]);
        let result = build_connectivity(&design, &dp);

        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);

        // All GND-related pins should be in one net.
        let gnd_net = result.ir.nets.iter().find(|n| n.name == "GND").unwrap();
        // Unique pin refs: EXTERNAL/GND, 0/GND, 1/B, 2/B = 4 pins
        assert_eq!(gnd_net.pins.len(), 4);
    }

    #[test]
    fn disjoint_nets_stay_separate() {
        let design = TypedDesign {
            name: "Board".into(),
            instances: vec![
                make_instance(0, "mcu", "STM32"),
                make_instance(1, "c1", "MLCC"),
            ],
            nets: vec![
                TypedNet {
                    name: "VDD".into(),
                    endpoints: vec![
                        (EXTERNAL_INSTANCE, "VDD".into()),
                        (InstanceId(0), "VDD_IO".into()),
                    ],
                },
                TypedNet {
                    name: "GND".into(),
                    endpoints: vec![
                        (EXTERNAL_INSTANCE, "GND".into()),
                        (InstanceId(1), "B".into()),
                    ],
                },
            ],
        };
        let dp = device_pins_map(&[("STM32", &["VDD_IO", "GND"]), ("MLCC", &["A", "B"])]);
        let result = build_connectivity(&design, &dp);

        assert!(result.errors.is_empty());
        assert_eq!(result.ir.nets.len(), 2);
    }

    #[test]
    fn shared_pin_merges_nets() {
        // If the same pin appears in two different net statements, they merge.
        let design = TypedDesign {
            name: "Board".into(),
            instances: vec![
                make_instance(0, "r1", "RES"),
                make_instance(1, "r2", "RES"),
                make_instance(2, "r3", "RES"),
            ],
            nets: vec![
                TypedNet {
                    name: "N1".into(),
                    endpoints: vec![(InstanceId(0), "B".into()), (InstanceId(1), "A".into())],
                },
                TypedNet {
                    name: "N2".into(),
                    endpoints: vec![
                        (InstanceId(1), "A".into()), // shared pin with N1
                        (InstanceId(2), "A".into()),
                    ],
                },
            ],
        };
        let dp = device_pins_map(&[("RES", &["A", "B"])]);
        let result = build_connectivity(&design, &dp);

        assert!(result.errors.is_empty());
        // N1 and N2 share pin r2.A, so they should merge into one net.
        assert_eq!(result.ir.nets.len(), 1);
        assert_eq!(result.ir.nets[0].pins.len(), 3);
    }

    // ── Error detection: non-existent pin ───────────────────────────────

    #[test]
    fn error_on_nonexistent_pin() {
        let design = TypedDesign {
            name: "Board".into(),
            instances: vec![make_instance(0, "c1", "MLCC")],
            nets: vec![TypedNet {
                name: "VDD".into(),
                endpoints: vec![
                    (EXTERNAL_INSTANCE, "VDD".into()),
                    (InstanceId(0), "NONEXISTENT".into()),
                ],
            }],
        };
        let dp = device_pins_map(&[("MLCC", &["A", "B"])]);
        let result = build_connectivity(&design, &dp);

        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].message.contains("does not exist"));
        assert!(result.errors[0].message.contains("NONEXISTENT"));
        assert!(result.errors[0].message.contains("MLCC"));
    }

    // ── Error detection: wrong bus index ────────────────────────────────

    #[test]
    fn error_on_wrong_bus_index() {
        let design = TypedDesign {
            name: "Board".into(),
            instances: vec![make_instance(0, "mcu", "MCU")],
            nets: vec![TypedNet {
                name: "DATA".into(),
                endpoints: vec![
                    (EXTERNAL_INSTANCE, "DATA".into()),
                    (InstanceId(0), "D[99]".into()),
                ],
            }],
        };
        // Declare bus pins D[0]..D[7]
        let dp = device_pins_map(&[(
            "MCU",
            &[
                "D[0]", "D[1]", "D[2]", "D[3]", "D[4]", "D[5]", "D[6]", "D[7]",
            ],
        )]);
        let result = build_connectivity(&design, &dp);

        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0]
            .message
            .contains("bus index 99 out of range"));
    }

    // ── Hierarchical instance paths ─────────────────────────────────────

    #[test]
    fn hierarchical_paths_are_correct() {
        let design = TypedDesign {
            name: "MainBoard".into(),
            instances: vec![
                make_instance(0, "mcu", "STM32"),
                make_instance(1, "r_dm", "RES"),
            ],
            nets: vec![],
        };
        let dp = device_pins_map(&[("STM32", &["VDD"]), ("RES", &["A", "B"])]);
        let result = build_connectivity(&design, &dp);

        assert!(result.errors.is_empty());
        assert_eq!(result.ir.instances[0].hierarchical_path, "MainBoard::mcu");
        assert_eq!(result.ir.instances[1].hierarchical_path, "MainBoard::r_dm");
    }

    // ── Empty design ────────────────────────────────────────────────────

    #[test]
    fn empty_design_produces_empty_ir() {
        let design = TypedDesign {
            name: "Empty".into(),
            instances: vec![],
            nets: vec![],
        };
        let dp = HashMap::new();
        let result = build_connectivity(&design, &dp);

        assert!(result.errors.is_empty());
        assert!(result.ir.instances.is_empty());
        assert!(result.ir.nets.is_empty());
    }

    // ── External-only net ───────────────────────────────────────────────

    #[test]
    fn external_only_net_is_preserved() {
        let design = TypedDesign {
            name: "Board".into(),
            instances: vec![],
            nets: vec![TypedNet {
                name: "VDD".into(),
                endpoints: vec![(EXTERNAL_INSTANCE, "VDD".into())],
            }],
        };
        let dp = HashMap::new();
        let result = build_connectivity(&design, &dp);

        assert!(result.errors.is_empty());
        assert_eq!(result.ir.nets.len(), 1);
        assert_eq!(result.ir.nets[0].pins.len(), 1);
        assert_eq!(result.ir.nets[0].pins[0].instance_id, EXTERNAL_INSTANCE);
    }

    // ── parse_bus_pin helper ────────────────────────────────────────────

    #[test]
    fn parse_bus_pin_plain() {
        assert_eq!(parse_bus_pin("A"), ("A", None));
        assert_eq!(parse_bus_pin("VDD_IO"), ("VDD_IO", None));
    }

    #[test]
    fn parse_bus_pin_indexed() {
        assert_eq!(parse_bus_pin("D[0]"), ("D", Some(0)));
        assert_eq!(parse_bus_pin("D[15]"), ("D", Some(15)));
    }

    // ── Instance fields are preserved ───────────────────────────────────

    #[test]
    fn instance_fields_preserved() {
        let mut subs = HashMap::new();
        subs.insert("C".to_string(), "100nF".to_string());
        let design = TypedDesign {
            name: "Board".into(),
            instances: vec![ComponentInstance {
                id: InstanceId(0),
                name: "c1".into(),
                device: "MLCC".into(),
                mpn: Some("CL05B104KO5NNNC".into()),
                alt_mpns: Vec::new(),
                generic_substitutions: subs.clone(),
                designator_override: None,
                impl_traits: Vec::new(),
            }],
            nets: vec![],
        };
        let dp = device_pins_map(&[("MLCC", &["A", "B"])]);
        let result = build_connectivity(&design, &dp);

        assert!(result.errors.is_empty());
        let inst = &result.ir.instances[0];
        assert_eq!(inst.device, "MLCC");
        assert_eq!(inst.mpn, Some("CL05B104KO5NNNC".into()));
        assert_eq!(inst.generic_substitutions, subs);
    }

    // ── Multiple errors collected ───────────────────────────────────────

    #[test]
    fn multiple_pin_errors_collected() {
        let design = TypedDesign {
            name: "Board".into(),
            instances: vec![make_instance(0, "c1", "MLCC")],
            nets: vec![
                TypedNet {
                    name: "N1".into(),
                    endpoints: vec![(InstanceId(0), "X".into())],
                },
                TypedNet {
                    name: "N2".into(),
                    endpoints: vec![(InstanceId(0), "Y".into())],
                },
            ],
        };
        let dp = device_pins_map(&[("MLCC", &["A", "B"])]);
        let result = build_connectivity(&design, &dp);

        assert_eq!(result.errors.len(), 2);
    }
}
