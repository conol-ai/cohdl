use cohdl::pipeline::check_files_in;

#[test]
fn exact_source_identity_polygon_and_unknown_hole_are_checkable() {
    let source = r#"
pub pad Land { shape: polygon, size: (2mm, 2mm), vertices: [(-1mm,-1mm),(1mm,-1mm),(0mm,1mm)], layer: top_copper, plating: smd }
pub footprint FP { pad 1: Land at (0mm,0mm) mount_hole 1: source_unspecified at (3mm,0mm) diameter 2mm }
pub device D { pins { P: 1 [passive] } }
#[source_unspecified("mpn", "mfr")]
pub part P: D { primary { mpn: "", mfr: "", footprint: FP } }
design B { inst p: P nc: p.P }
"#;
    let checked = check_files_in("board", &[("main.cohdl".into(), source.into())], None).unwrap();
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
}
