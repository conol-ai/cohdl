//! BOM (Bill of Materials) emitters.
//!
//! Two output modes:
//! - **Simple BOM** (`emit_simple_bom`): CSV with `RefDes,MPN,Qty` — groups
//!   identical MPNs and combines reference designators.
//! - **AVL BOM** (`emit_avl_bom`): CSV with `RefDes,Primary MPN,Alt 1,Alt 2,...`
//!   — one row per instance, includes all alternate MPNs from `part` declarations.

use std::collections::BTreeMap;
use std::fmt::Write;

use cohdl_sema::connectivity::ConnectivityIR;

const UNSPECIFIED: &str = "<UNSPECIFIED>";

/// Emit a simple BOM as CSV: `RefDes,MPN,Qty`.
///
/// Instances sharing the same MPN are grouped into a single row with combined
/// reference designators (comma-separated) and a quantity count. Instances
/// without a bound part appear with MPN `"<UNSPECIFIED>"`.
pub fn emit_simple_bom(ir: &ConnectivityIR) -> String {
    // Group by MPN → sorted list of RefDes.
    let mut groups: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

    for inst in &ir.instances {
        let mpn = inst.mpn.as_deref().unwrap_or(UNSPECIFIED);
        groups.entry(mpn).or_default().push(&inst.name);
    }

    let mut out = String::new();
    writeln!(out, "RefDes,MPN,Qty").unwrap();

    for (mpn, mut refs) in groups {
        refs.sort();
        let qty = refs.len();
        let refdes = refs.join(",");
        writeln!(out, "\"{}\",\"{}\",{}", refdes, mpn, qty).unwrap();
    }

    out
}

/// Emit an AVL BOM as CSV: `RefDes,Primary MPN,Alt 1,Alt 2,...`.
///
/// Each instance gets its own row. The number of `Alt N` columns equals the
/// maximum number of alternate MPNs across all instances. Instances without a
/// bound part show `"<UNSPECIFIED>"` as the primary MPN with no alternates.
pub fn emit_avl_bom(ir: &ConnectivityIR) -> String {
    let max_alts = ir
        .instances
        .iter()
        .map(|inst| inst.alt_mpns.len())
        .max()
        .unwrap_or(0);

    let mut out = String::new();

    // Header
    write!(out, "RefDes,Primary MPN").unwrap();
    for i in 1..=max_alts {
        write!(out, ",Alt {}", i).unwrap();
    }
    writeln!(out).unwrap();

    // Sort instances by name for deterministic output.
    let mut sorted: Vec<_> = ir.instances.iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));

    for inst in sorted {
        let primary = inst.mpn.as_deref().unwrap_or(UNSPECIFIED);
        write!(out, "\"{}\",\"{}\"", inst.name, primary).unwrap();
        for i in 0..max_alts {
            let alt = inst.alt_mpns.get(i).map(|s| s.as_str()).unwrap_or("");
            write!(out, ",\"{}\"", alt).unwrap();
        }
        writeln!(out).unwrap();
    }

    out
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use cohdl_sema::connectivity::{ConnectivityIR, Instance};
    use cohdl_sema::typeck::InstanceId;
    use std::collections::HashMap;

    fn make_inst(id: u32, name: &str, mpn: Option<&str>, alt_mpns: &[&str]) -> Instance {
        Instance {
            id: InstanceId(id),
            name: name.into(),
            hierarchical_path: format!("Board::{}", name),
            device: "DEV".into(),
            mpn: mpn.map(|s| s.into()),
            alt_mpns: alt_mpns.iter().map(|s| s.to_string()).collect(),
            generic_substitutions: HashMap::new(),
        }
    }

    // ── Simple BOM tests ─────────────────────────────────────────────────

    #[test]
    fn simple_bom_groups_identical_mpns() {
        let ir = ConnectivityIR {
            instances: vec![
                make_inst(0, "C1", Some("CL05B104KO5NNNC"), &[]),
                make_inst(1, "C2", Some("CL05B104KO5NNNC"), &[]),
                make_inst(2, "R1", Some("RC0402FR-0710KL"), &[]),
            ],
            nets: vec![],
        };

        let csv = emit_simple_bom(&ir);
        let expected = "\
RefDes,MPN,Qty
\"C1,C2\",\"CL05B104KO5NNNC\",2
\"R1\",\"RC0402FR-0710KL\",1
";
        assert_eq!(csv, expected);
    }

    #[test]
    fn simple_bom_unspecified_mpn() {
        let ir = ConnectivityIR {
            instances: vec![
                make_inst(0, "U1", None, &[]),
                make_inst(1, "C1", Some("CL05B104KO5NNNC"), &[]),
            ],
            nets: vec![],
        };

        let csv = emit_simple_bom(&ir);
        let expected = "\
RefDes,MPN,Qty
\"U1\",\"<UNSPECIFIED>\",1
\"C1\",\"CL05B104KO5NNNC\",1
";
        assert_eq!(csv, expected);
    }

    #[test]
    fn simple_bom_empty_ir() {
        let ir = ConnectivityIR {
            instances: vec![],
            nets: vec![],
        };
        let csv = emit_simple_bom(&ir);
        assert_eq!(csv, "RefDes,MPN,Qty\n");
    }

    // ── AVL BOM tests ────────────────────────────────────────────────────

    #[test]
    fn avl_bom_includes_alternates() {
        let ir = ConnectivityIR {
            instances: vec![
                make_inst(0, "C1", Some("CL05B104KO5NNNC"), &["GRM155R71C104KA88D"]),
                make_inst(1, "R1", Some("RC0402FR-0710KL"), &[]),
            ],
            nets: vec![],
        };

        let csv = emit_avl_bom(&ir);
        let expected = "\
RefDes,Primary MPN,Alt 1
\"C1\",\"CL05B104KO5NNNC\",\"GRM155R71C104KA88D\"
\"R1\",\"RC0402FR-0710KL\",\"\"
";
        assert_eq!(csv, expected);
    }

    #[test]
    fn avl_bom_multiple_alternates() {
        let ir = ConnectivityIR {
            instances: vec![make_inst(
                0,
                "C1",
                Some("CL05B104KO5NNNC"),
                &["GRM155R71C104KA88D", "08055C104KAT2A"],
            )],
            nets: vec![],
        };

        let csv = emit_avl_bom(&ir);
        let expected = "\
RefDes,Primary MPN,Alt 1,Alt 2
\"C1\",\"CL05B104KO5NNNC\",\"GRM155R71C104KA88D\",\"08055C104KAT2A\"
";
        assert_eq!(csv, expected);
    }

    #[test]
    fn avl_bom_unspecified_mpn() {
        let ir = ConnectivityIR {
            instances: vec![make_inst(0, "U1", None, &[])],
            nets: vec![],
        };

        let csv = emit_avl_bom(&ir);
        let expected = "\
RefDes,Primary MPN
\"U1\",\"<UNSPECIFIED>\"
";
        assert_eq!(csv, expected);
    }

    #[test]
    fn avl_bom_empty_ir() {
        let ir = ConnectivityIR {
            instances: vec![],
            nets: vec![],
        };
        let csv = emit_avl_bom(&ir);
        assert_eq!(csv, "RefDes,Primary MPN\n");
    }

    #[test]
    fn avl_bom_sorted_by_refdes() {
        let ir = ConnectivityIR {
            instances: vec![
                make_inst(0, "R1", Some("RC0402FR-0710KL"), &[]),
                make_inst(1, "C1", Some("CL05B104KO5NNNC"), &[]),
            ],
            nets: vec![],
        };

        let csv = emit_avl_bom(&ir);
        // C1 should come before R1 in sorted output.
        let lines: Vec<&str> = csv.lines().collect();
        assert!(lines[1].starts_with("\"C1\""));
        assert!(lines[2].starts_with("\"R1\""));
    }
}
