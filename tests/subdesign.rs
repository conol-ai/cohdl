//! RFC-032 typed logical composition (`subdesign`).
//!
//! The load-bearing properties: (1) ports merge nets across the boundary —
//! the container never becomes a component, designator, or BOM row; (2) two
//! uses are hygienic and deterministic, with retained `Design::use::inst`
//! paths feeding RFC-005 unchanged; (3) generics reuse RFC-007, arrays reuse
//! RFC-024; (4) the internal `layout {}` is a default the outer design
//! transforms as one unit (the RFC-025/026 pad math, verified against its
//! own worked example) and overrides per instance; (5) the port boundary is
//! the only electrical surface — placement is the sole reach-in; (6) errors
//! carry the E13xx contract.

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in, check_files_in_with_deps};

/// A reusable regulator-shaped subdesign plus enough library to bind parts.
const LIB: &str = r#"
pub trait Ic { designator_prefix: "U" }
pub trait Cap { designator_prefix: "C" }
pub pad P { shape: rect, size: (0.8mm, 0.3mm), layer: top_copper, plating: smd }
pub footprint F {
    pad 1: P at (1mm, 2mm)
    pad 2: P at (-1mm, -2mm)
}
pub device Reg { pins { VIN: 1 [power_in], VOUT: 2 [power_out] } }
impl Ic for Reg {}
pub part REG: Reg { primary { mfr: "m", mpn: "reg", footprint: F } }
pub device C2T<C: Capacitance> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C }
}
impl Cap for C2T {}
pub part C100N: C2T<100nF> { primary { mfr: "m", mpn: "c100n", footprint: F } }
pub part C1U: C2T<1uF> { primary { mfr: "m", mpn: "c1u", footprint: F } }

pub subdesign Vreg<Cin: Capacitance> {
    ports {
        required VIN: Pin
        required VOUT: Pin
        optional SENSE: Pin
    }
    inst reg: REG
    inst c_in: C2T<Cin>
    net _: VIN, reg.VIN, c_in.A
    net OUT: VOUT, reg.VOUT, c_in.B
    net SNS: SENSE
    layout {
        place reg at (1mm, 2mm)
        place c_in at (-1mm, -2mm) rotate 90
    }
}
"#;

fn checked(src: &str) -> cohdl::pipeline::Checked {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    let _ = build_artifacts(&mut checked, &LockState::default());
    checked
}

fn checked_ok(src: &str) -> cohdl::pipeline::Checked {
    let c = checked(src);
    assert!(
        !c.diags.has_errors(),
        "fixture must build cleanly:\n{}",
        c.diags.render(&c.sm)
    );
    c
}

fn errors_of(src: &str) -> String {
    let c = checked(src);
    c.diags.render(&c.sm)
}

// ---------------------------------------------------------------------------
// Ports: connectivity across the boundary
// ---------------------------------------------------------------------------

#[test]
fn ports_merge_internal_and_external_nets_and_the_container_vanishes() {
    let src = format!(
        "{LIB}
design B {{
    inst c_bulk: C1U
    subdesign vr: Vreg<100nF> {{
        VIN: vbat
        VOUT: c_bulk.A
    }}
    net vbat [5V]: c_bulk.B
}}
"
    );
    let c = checked_ok(&src);
    let ir = c.ir.as_ref().unwrap();
    // Retained paths, ordinary designators, no node instance.
    assert!(ir.instances.contains_key("B::vr::reg"));
    assert!(ir.instances.contains_key("B::vr::c_in"));
    assert!(
        !ir.instances.contains_key("B::vr"),
        "the node is not an instance"
    );
    // The external design-level name wins the merged class.
    let vbat = ir.nets.iter().find(|n| n.name == "vbat").expect("vbat");
    assert!(vbat.members.contains(&("B::vr::reg".into(), "VIN".into())));
    assert!(vbat.members.contains(&("B::vr::c_in".into(), "A".into())));
    assert!(vbat.members.contains(&("B::c_bulk".into(), "B".into())));
    // No phantom (node, port) member survives into the IR.
    for net in &ir.nets {
        for (p, _) in &net.members {
            assert!(ir.instances.contains_key(p), "phantom member `{}`", p);
        }
    }
    // The internal net exposed through VOUT merged with the external pin;
    // its internal name survives (no design-level name in the class).
    let out = ir
        .nets
        .iter()
        .find(|n| n.members.contains(&("B::c_bulk".into(), "A".into())))
        .expect("merged output net");
    assert_eq!(out.name, "vr::OUT");
    assert!(out.members.contains(&("B::vr::reg".into(), "VOUT".into())));
    // The netlist carries only real instances.
    let net = cohdl::emit::kicad::emit_kicad_net(&c.world, ir);
    assert!(
        !net.contains("Vreg"),
        "container leaked into the netlist:\n{net}"
    );
    // Designators: reg (Ic->U), caps (Cap->C) — all real, none for the node.
    let bom = &build_artifacts(&mut checked_ok(&src), &LockState::default())
        .expect("artifacts")
        .bom;
    assert!(bom.contains("reg") && bom.contains("c100n") && bom.contains("c1u"));
}

#[test]
fn two_uses_are_hygienic_with_distinct_paths_and_designators() {
    let src = format!(
        "{LIB}
design B {{
    subdesign a: Vreg<100nF> {{ VIN: rail, VOUT: mid }}
    subdesign b: Vreg<1uF> {{ VIN: mid, VOUT: out }}
    inst load: C1U
    net rail [5V]: load.A
    net mid: load.B
    net out: a.SENSE
}}
"
    );
    let c = checked_ok(&src);
    let ir = c.ir.as_ref().unwrap();
    for p in ["B::a::reg", "B::a::c_in", "B::b::reg", "B::b::c_in"] {
        assert!(ir.instances.contains_key(p), "missing {}", p);
    }
    // RFC-007 monomorphization: each use's c_in carries ITS argument.
    assert_eq!(
        ir.instances["B::a::c_in"].specs["capacitance"].femto,
        100_000_000, // 100nF in femtofarads
    );
    assert_eq!(
        ir.instances["B::b::c_in"].specs["capacitance"].femto,
        1_000_000_000,
    );
    // Distinct designators via the ordinary allocator.
    let d: Vec<_> = ir
        .instances
        .values()
        .filter_map(|i| i.designator.clone())
        .collect();
    let unique: std::collections::BTreeSet<_> = d.iter().collect();
    assert_eq!(
        d.len(),
        unique.len(),
        "designators must be injective: {d:?}"
    );
}

#[test]
fn deterministic_bytes() {
    let src = format!(
        "{LIB}
design B {{
    subdesign a: Vreg<100nF> {{ VIN: rail, VOUT: out }}
    inst load: C1U
    net rail [5V]: load.A
    net out: load.B
}}
"
    );
    let a = checked_ok(&src);
    let b = checked_ok(&src);
    let net_a = cohdl::emit::kicad::emit_kicad_net(&a.world, a.ir.as_ref().unwrap());
    let net_b = cohdl::emit::kicad::emit_kicad_net(&b.world, b.ir.as_ref().unwrap());
    assert_eq!(net_a, net_b);
}

// ---------------------------------------------------------------------------
// Port-boundary errors
// ---------------------------------------------------------------------------

#[test]
fn required_port_unconnected_is_e1302() {
    let src = format!(
        "{LIB}
design B {{
    subdesign vr: Vreg<100nF> {{ VIN: rail }}
    inst load: C1U
    net rail [5V]: load.A
    net x: load.B
}}
"
    );
    let e = errors_of(&src);
    assert!(e.contains("E1302"), "{e}");
    assert!(e.contains("required port `VOUT`"), "{e}");
    // The optional SENSE port dangles silently.
    assert!(!e.contains("SENSE"), "{e}");
}

#[test]
fn unknown_port_is_e1301_naming_the_boundary() {
    let src = format!(
        "{LIB}
design B {{
    subdesign vr: Vreg<100nF> {{ VIN: rail, VOUT: out }}
    inst load: C1U
    net rail [5V]: load.A
    net out: load.B
    net y: vr.reg
}}
"
    );
    let e = errors_of(&src);
    assert!(e.contains("E1301"), "{e}");
    assert!(e.contains("no port named `reg`"), "{e}");
    assert!(e.contains("only `place` may reach in"), "{e}");
}

#[test]
fn unknown_net_name_in_port_block_is_e1303() {
    let src = format!(
        "{LIB}
design B {{
    subdesign vr: Vreg<100nF> {{ VIN: no_such_net, VOUT: out }}
    inst load: C1U
    net rail [5V]: load.A, vr.SENSE
    net out: load.B
}}
"
    );
    let e = errors_of(&src);
    assert!(e.contains("E1303"), "{e}");
    assert!(e.contains("no_such_net"), "{e}");
}

#[test]
fn nc_on_a_port_is_e1306() {
    let src = format!(
        "{LIB}
design B {{
    subdesign vr: Vreg<100nF> {{ VIN: rail, VOUT: out }}
    inst load: C1U
    net rail [5V]: load.A
    net out: load.B
    nc: vr.SENSE
}}
"
    );
    let e = errors_of(&src);
    assert!(e.contains("E1306"), "{e}");
}

#[test]
fn use_site_in_fn_body_is_e1307_even_uncalled() {
    let src = format!(
        "{LIB}
fn helper(p: Pin) {{
    subdesign vr: Vreg<100nF> {{ VIN: p, VOUT: p }}
}}
design B {{
    inst load: C1U
    net rail [5V]: load.A
    net out: load.B
}}
"
    );
    let e = errors_of(&src);
    assert!(e.contains("E1307"), "{e}");
}

#[test]
fn recursive_containment_is_e1304_naming_the_cycle() {
    let src = format!(
        "{LIB}
pub subdesign A2 {{
    ports {{ required P: Pin }}
    subdesign inner: B2 {{ P: P }}
}}
pub subdesign B2 {{
    ports {{ required P: Pin }}
    subdesign inner: A2 {{ P: P }}
}}
design B {{
    inst load: C1U
    net rail [5V]: load.A
    net out: load.B
}}
"
    );
    let e = errors_of(&src);
    assert!(e.contains("E1304"), "{e}");
    assert!(
        e.contains("`A2` → `B2` → `A2`") || e.contains("`B2` → `A2` → `B2`"),
        "{e}"
    );
}

// ---------------------------------------------------------------------------
// Arrays (RFC-024 reused verbatim)
// ---------------------------------------------------------------------------

#[test]
fn array_typed_use_sites_index_everywhere_and_bound_check() {
    let src = format!(
        "{LIB}
design B {{
    subdesign vrs: [Vreg<100nF>; 2]
    inst load: C1U
    inst load2: C1U
    net rail [5V]: load.A, load2.A, vrs[0].VIN, vrs[1].VIN
    net o0: load.B, vrs[0].VOUT
    net o1: load2.B, vrs[1].VOUT
}}
"
    );
    let c = checked_ok(&src);
    let ir = c.ir.as_ref().unwrap();
    for p in ["B::vrs_0::reg", "B::vrs_1::reg"] {
        assert!(ir.instances.contains_key(p), "missing {}", p);
    }
    // Out of bounds reuses RFC-024's E202.
    let bad = format!(
        "{LIB}
design B {{
    subdesign vrs: [Vreg<100nF>; 2]
    inst load: C1U
    net rail [5V]: load.A, vrs[2].VIN, vrs[0].VIN, vrs[1].VIN
    net o0: load.B, vrs[0].VOUT
    net o1: vrs[1].VOUT
}}
"
    );
    let e = errors_of(&bad);
    assert!(e.contains("E202") && e.contains("out of bounds"), "{e}");
}

// ---------------------------------------------------------------------------
// Layout: whole-unit transform, override precedence, reach-in
// ---------------------------------------------------------------------------

/// One placement row by path.
fn place_of<'a>(ir: &'a cohdl::ir::DesignIr, path: &str) -> &'a cohdl::ir::LayoutPlacement {
    ir.layout
        .placements
        .iter()
        .find(|p| p.path == path)
        .unwrap_or_else(|| panic!("no placement for {path}"))
}

#[test]
fn whole_unit_placement_transforms_the_default_layout() {
    let src = format!(
        "{LIB}
design B {{
    subdesign vr: Vreg<100nF> {{ VIN: rail, VOUT: out }}
    inst load: C1U
    net rail [5V]: load.A
    net out: load.B
    layout {{
        place vr at (10mm, 0mm)
    }}
}}
"
    );
    let c = checked_ok(&src);
    let ir = c.ir.as_ref().unwrap();
    // reg default (1, 2) translates to (11, 2); c_in (-1, -2) r90 to (9, -2).
    let reg = place_of(ir, "B::vr::reg");
    assert_eq!(
        (reg.at.0.femto, reg.at.1.femto),
        (11_000_000_000_000_000, 2_000_000_000_000_000)
    );
    assert_eq!((reg.rotate, reg.side), (0, cohdl::ast::PlacementSide::Top));
    let c_in = place_of(ir, "B::vr::c_in");
    assert_eq!(
        (c_in.at.0.femto, c_in.at.1.femto),
        (9_000_000_000_000_000, -2_000_000_000_000_000)
    );
    assert_eq!(c_in.rotate, 90);
}

#[test]
fn rotated_and_flipped_anchor_uses_the_rfc026_pad_math() {
    // The kicad_pcb worked example, applied to instances: anchor (5, 0)
    // rotate 90 side bottom, local (1, 2) → absolute (7, 1), and the local
    // rotation REVERSES under the reflection: 90 − 90 = 0.
    let src = format!(
        "{LIB}
design B {{
    subdesign vr: Vreg<100nF> {{ VIN: rail, VOUT: out }}
    inst load: C1U
    net rail [5V]: load.A
    net out: load.B
    layout {{
        place vr at (5mm, 0mm) rotate 90 side bottom
    }}
}}
"
    );
    let c = checked_ok(&src);
    let ir = c.ir.as_ref().unwrap();
    let reg = place_of(ir, "B::vr::reg"); // local (1, 2) r0
    assert_eq!(
        (reg.at.0.femto, reg.at.1.femto),
        (7_000_000_000_000_000, 1_000_000_000_000_000)
    );
    assert_eq!(reg.rotate, 90); // 90 − 0
    assert_eq!(reg.side, cohdl::ast::PlacementSide::Bottom);
    let c_in = place_of(ir, "B::vr::c_in"); // local (-1, -2) r90
                                            // mirror x: (1, -2); Rot(90): (-2, -1) → absolute (3, -1).
    assert_eq!(
        (c_in.at.0.femto, c_in.at.1.femto),
        (3_000_000_000_000_000, -1_000_000_000_000_000)
    );
    assert_eq!(c_in.rotate, 0); // 90 − 90, reversed by the reflection
    assert_eq!(c_in.side, cohdl::ast::PlacementSide::Bottom);
}

#[test]
fn explicit_reach_in_override_beats_the_default_for_one_instantiation() {
    let src = format!(
        "{LIB}
design B {{
    subdesign a: Vreg<100nF> {{ VIN: rail, VOUT: mid }}
    subdesign b: Vreg<100nF> {{ VIN: mid, VOUT: out }}
    inst load: C1U
    net rail [5V]: load.A
    net mid: load.B
    net out: a.SENSE
    layout {{
        place a at (10mm, 0mm)
        place b at (30mm, 0mm)
        place a.c_in at (12mm, 7mm) rotate 180
    }}
}}
"
    );
    let c = checked_ok(&src);
    let ir = c.ir.as_ref().unwrap();
    // a.c_in: the explicit override, verbatim.
    let a_cin = place_of(ir, "B::a::c_in");
    assert_eq!(
        (a_cin.at.0.femto, a_cin.at.1.femto, a_cin.rotate),
        (12_000_000_000_000_000, 7_000_000_000_000_000, 180)
    );
    // b.c_in: the untouched default, transformed by b's anchor only.
    let b_cin = place_of(ir, "B::b::c_in");
    assert_eq!(
        (b_cin.at.0.femto, b_cin.at.1.femto, b_cin.rotate),
        (29_000_000_000_000_000, -2_000_000_000_000_000, 90)
    );
    // a.reg keeps its transformed default alongside the sibling override.
    let a_reg = place_of(ir, "B::a::reg");
    assert_eq!(a_reg.at.0.femto, 11_000_000_000_000_000);
}

#[test]
fn unanchored_node_contributes_no_placements() {
    let src = format!(
        "{LIB}
design B {{
    subdesign vr: Vreg<100nF> {{ VIN: rail, VOUT: out }}
    inst load: C1U
    net rail [5V]: load.A
    net out: load.B
}}
"
    );
    let c = checked_ok(&src);
    let ir = c.ir.as_ref().unwrap();
    assert!(
        ir.layout.placements.is_empty(),
        "an unplaced use site stages its internals: {:?}",
        ir.layout
            .placements
            .iter()
            .map(|p| &p.path)
            .collect::<Vec<_>>()
    );
}

#[test]
fn reach_in_path_failure_names_the_exact_segment() {
    let src = format!(
        "{LIB}
design B {{
    subdesign vr: Vreg<100nF> {{ VIN: rail, VOUT: out }}
    inst load: C1U
    net rail [5V]: load.A
    net out: load.B
    layout {{
        place vr.nope at (1mm, 1mm)
    }}
}}
"
    );
    let e = errors_of(&src);
    assert!(e.contains("E1305"), "{e}");
    assert!(
        e.contains("`nope` is not an instance or subdesign inside `vr`"),
        "{e}"
    );
}

// ---------------------------------------------------------------------------
// Nesting
// ---------------------------------------------------------------------------

#[test]
fn nested_subdesigns_compose_paths_ports_and_layout() {
    let src = format!(
        "{LIB}
pub subdesign Duo {{
    ports {{
        required IN: Pin
        required OUT: Pin
    }}
    subdesign first: Vreg<100nF> {{ VIN: IN }}
    subdesign second: Vreg<1uF> {{ VOUT: OUT }}
    net link: first.VOUT, second.VIN
    layout {{
        place first at (0mm, 0mm)
        place second at (10mm, 0mm) rotate 90
    }}
}}
design B {{
    subdesign duo: Duo {{ IN: rail, OUT: out }}
    inst load: C1U
    net rail [5V]: load.A
    net out: load.B
    layout {{
        place duo at (100mm, 50mm)
    }}
}}
"
    );
    let c = checked_ok(&src);
    let ir = c.ir.as_ref().unwrap();
    // Two-level retained paths.
    assert!(ir.instances.contains_key("B::duo::first::reg"));
    assert!(ir.instances.contains_key("B::duo::second::c_in"));
    // The inner link net merges through two port boundaries; the merged
    // class takes the smallest scoped name (provisional §5's deterministic
    // rule — first's own internal OUT wins over `duo::link`).
    let link = ir
        .nets
        .iter()
        .find(|n| {
            n.members
                .contains(&("B::duo::second::reg".into(), "VIN".into()))
        })
        .expect("link");
    assert_eq!(link.name, "duo::first::OUT");
    assert!(link
        .members
        .contains(&("B::duo::first::reg".into(), "VOUT".into())));
    // Layout composes: duo (100, 50) ∘ first (0, 0) ∘ reg (1, 2) = (101, 52);
    // duo ∘ second (10, 0) r90 ∘ reg (1, 2) = (100+10, 50) + Rot90(1,2)=(2,-1)
    // → (112, 49), rotate 90.
    let first_reg = place_of(ir, "B::duo::first::reg");
    assert_eq!(
        (first_reg.at.0.femto, first_reg.at.1.femto),
        (101_000_000_000_000_000, 52_000_000_000_000_000)
    );
    let second_reg = place_of(ir, "B::duo::second::reg");
    assert_eq!(
        (second_reg.at.0.femto, second_reg.at.1.femto),
        (112_000_000_000_000_000, 49_000_000_000_000_000)
    );
    assert_eq!(second_reg.rotate, 90);
}

// ---------------------------------------------------------------------------
// fmt round-trip (RFC-009: canonical is a fixed point)
// ---------------------------------------------------------------------------

#[test]
fn fmt_canonical_form_is_a_fixed_point_and_keeps_every_construct() {
    let src = format!(
        "{LIB}
design B {{
    subdesign vr: Vreg<100nF> {{
        VIN: rail
        VOUT: out
    }}
    subdesign vrs: [Vreg<1uF>; 2]
    inst load: C1U
    net rail [5V]: load.A, vrs[0].VIN, vrs[1].VIN
    net out: load.B, vrs[0].VOUT, vrs[1].VOUT
    layout {{
        place vr at (10mm, 0mm)
        place vr.c_in at (12mm, 7mm) rotate 180 side bottom
    }}
}}
"
    );
    let once = cohdl::fmt::format_source("main.cohdl", &src).expect("fmt");
    let twice = cohdl::fmt::format_source("main.cohdl", &once).expect("fmt twice");
    assert_eq!(once, twice, "canonical form must be a fixed point");
    for needle in [
        "subdesign Vreg<Cin: Capacitance> {",
        "ports {",
        "required VIN: Pin",
        "optional SENSE: Pin",
        "subdesign vr: Vreg<100nF> {",
        "VIN: rail",
        "subdesign vrs: [Vreg<1uF>; 2]",
        "place vr.c_in at (12mm, 7mm) rotate 180 side bottom",
        "place vr at (10mm, 0mm)",
    ] {
        assert!(
            once.contains(needle),
            "canonical form lost `{needle}`:\n{once}"
        );
    }
}

// ---------------------------------------------------------------------------
// Cross-package citizenship (RFC-016/029/030 machinery, unchanged)
// ---------------------------------------------------------------------------

#[test]
fn pub_subdesign_imports_across_packages_and_private_is_e209() {
    let lib_pkg = r#"
pub trait Ic { designator_prefix: "U" }
pub pad P { shape: rect, size: (0.8mm, 0.3mm), layer: top_copper, plating: smd }
pub footprint F {
    pad 1: P at (1mm, 2mm)
    pad 2: P at (-1mm, -2mm)
}
pub device Reg { pins { VIN: 1 [power_in], VOUT: 2 [power_out] } }
impl Ic for Reg {}
pub part REG: Reg { primary { mfr: "m", mpn: "reg", footprint: F } }

pub subdesign Exported {
    ports {
        required VIN: Pin
        required VOUT: Pin
    }
    inst reg: REG
    net _: VIN, reg.VIN
    net _: VOUT, reg.VOUT
}

subdesign Hidden {
    ports { required VIN: Pin }
    inst reg: REG
    net _: VIN, reg.VIN
    nc: reg.VOUT
}
"#;
    let main_ok = r#"
use mylib::Exported;
use mylib::REG;

design B {
    subdesign vr: Exported { VIN: rail, VOUT: out }
    inst src_reg: REG
    net rail [5V]: src_reg.VOUT
    net out: src_reg.VIN
}
"#;
    let files = vec![
        ("mylib/vreg.cohdl".to_string(), lib_pkg.to_string()),
        ("src/main.cohdl".to_string(), main_ok.to_string()),
    ];
    let mut c =
        check_files_in_with_deps("board", &["mylib".to_string()], &files, None).expect("pipeline");
    let _ = build_artifacts(&mut c, &LockState::default());
    assert!(
        !c.diags.has_errors(),
        "cross-package use must check cleanly:\n{}",
        c.diags.render(&c.sm)
    );
    let ir = c.ir.as_ref().unwrap();
    assert!(ir.instances.contains_key("B::vr::reg"));

    // A non-pub subdesign stays package-private (E209, RFC-016 unchanged).
    let main_bad = r#"
use mylib::Hidden;
use mylib::REG;

design B {
    subdesign vr: Hidden { VIN: rail }
    inst src_reg: REG
    net rail [5V]: src_reg.VOUT
    nc: src_reg.VIN
}
"#;
    let files = vec![
        ("mylib/vreg.cohdl".to_string(), lib_pkg.to_string()),
        ("src/main.cohdl".to_string(), main_bad.to_string()),
    ];
    let mut c =
        check_files_in_with_deps("board", &["mylib".to_string()], &files, None).expect("pipeline");
    let _ = build_artifacts(&mut c, &LockState::default());
    let e = c.diags.render(&c.sm);
    assert!(e.contains("E209"), "{e}");
    assert!(e.contains("subdesign"), "the E209 help names the kind: {e}");
}
