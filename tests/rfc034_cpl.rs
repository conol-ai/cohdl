//! RFC-034 §3.1 neutral CPL and E815 conformance.

use cohdl::lock::LockState;
use cohdl::pipeline::{
    build_artifacts, check_files_in, resolve_board_outline, BuildArtifacts, Checked,
};
use std::path::Path;
use std::process::Command;

const HEADER: &str = "Designator,Mid X,Mid Y,Layer,Rotation\n";
const DXF_30X20: &str = "0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n8\nEdge.Cuts\n90\n4\n70\n1\n\
    10\n-15\n20\n-10\n10\n15\n20\n-10\n10\n15\n20\n10\n10\n-15\n20\n10\n0\nENDSEC\n";

const LIB: &str = r#"
pub trait Ic { designator_prefix: "U" }
pub pad Smd { shape: rect, size: (0.8mm, 0.3mm), layer: top_copper, plating: smd }
pub pad Hole { shape: circle, size: (1.7mm), layer: through_all, plating: plated_through_hole, drill: 1.0mm }
pub pad Ring { shape: annulus, size: (1.625mm, 1.025mm), layer: top_copper, plating: smd, paste: segmented_annulus(1.625mm, 1.125mm, 0.1mm) }
pub pad BareCopper { shape: circle, size: (0.5mm), layer: top_copper, plating: smd, paste: none }
pub footprint FP_SMD { pad 1: Smd at (1mm, 2mm) }
pub footprint FP_MIXED { pad 1: Smd at (1mm, 2mm) pad 2: Hole at (-1mm, -2mm) }
pub footprint FP_HOLE { pad 1: Hole at (0mm, 0mm) }
pub footprint FP_RING { pad 1: Ring at (0mm, 0mm) }
pub footprint FP_BARE { pad 1: BareCopper at (0mm, 0mm) }
pub device D1 { pins { A: 1 [passive] } }
pub device D2 { pins { A: 1 [passive], B: 2 [passive] } }
impl Ic for D1 {}
impl Ic for D2 {}
pub part SMD: D1 { primary { mfr: "m", mpn: "smd", footprint: FP_SMD } }
pub part MIXED: D2 { primary { mfr: "m", mpn: "mixed", footprint: FP_MIXED } }
pub part HOLE: D1 { primary { mfr: "m", mpn: "hole", footprint: FP_HOLE } }
pub part RING: D1 { primary { mfr: "m", mpn: "ring", footprint: FP_RING } }
pub part BARE: D1 { primary { mfr: "m", mpn: "TouchSensor", footprint: FP_BARE } }
"#;

fn build(src: &str, dxf: Option<&str>) -> (Checked, BuildArtifacts) {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    if let Some(dxf) = dxf {
        resolve_board_outline(&mut checked, |_| Ok(dxf.to_string()));
    }
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "fixture must build cleanly:\n{}",
        checked.diags.render(&checked.sm)
    );
    (checked, artifacts.expect("build"))
}

fn cpl(artifacts: &BuildArtifacts) -> &str {
    &artifacts
        .cpl
        .as_ref()
        .expect("placed design writes CPL")
        .csv
}

fn bom_designators(bom: &str) -> std::collections::BTreeSet<&str> {
    let mut out = std::collections::BTreeSet::new();
    for line in bom.lines().skip(1) {
        let quoted = line.split('"').nth(5).expect("quoted BOM designators");
        out.extend(quoted.split(','));
    }
    out
}

#[test]
fn rows_follow_each_instances_numbered_smd_pad_projection_and_bom() {
    let src = format!(
        "{LIB}
design B {{
    #[designator(\"U2\")] inst smd: SMD
    #[designator(\"U10\")] inst mixed: MIXED
    #[designator(\"TP1\")] inst test_point: SMD
    #[designator(\"U3\")] inst bare: BARE
    #[designator(\"U4\")] inst hole: HOLE
    net A: smd.A, test_point.A, bare.A, hole.A
    net B: mixed.A, mixed.B
    layout {{
        place smd at (1mm, 2mm)
        place mixed at (3mm, 4mm)
        place test_point at (5mm, 6mm)
        place bare at (7mm, 8mm)
        place hole at (9mm, 10mm)
    }}
}}
"
    );
    let (_checked, artifacts) = build(&src, None);
    let csv = cpl(&artifacts);
    // Natural order; neither TP nor a TouchSensor value is excluded.  The
    // bare-copper SMD has no paste but remains a machine-placed part.
    assert_eq!(
        csv,
        format!(
            "{HEADER}TP1,5.0000mm,-6.0000mm,Top,0\n\
U2,1.0000mm,-2.0000mm,Top,0\n\
U3,7.0000mm,-8.0000mm,Top,0\n\
U10,3.0000mm,-4.0000mm,Top,0\n"
        )
    );
    assert!(!csv.contains("U4,"), "through-hole-only instance: {csv}");
    for line in csv.lines().skip(1) {
        let refdes = line.split(',').next().unwrap();
        assert!(
            bom_designators(&artifacts.bom).contains(refdes),
            "CPL row {refdes} must have a BOM designator: {}",
            artifacts.bom
        );
    }
}

#[test]
fn annulus_is_smd_but_an_unnumbered_paste_aperture_is_not() {
    let ring = format!(
        "{LIB}
design B {{ inst ring: RING  net A: ring.A  layout {{ place ring at (1mm, 2mm) }} }}
"
    );
    let (_checked, artifacts) = build(&ring, None);
    assert_eq!(
        cpl(&artifacts),
        format!("{HEADER}U1,1.0000mm,-2.0000mm,Top,0\n")
    );

    let aperture_only = LIB.replace(
        "pad 1: BareCopper at (0mm, 0mm)",
        "pad 1: Hole at (0mm, 0mm)",
    );
    let src = format!(
        "{aperture_only}
design B {{ inst bare: BARE  net A: bare.A  layout {{ place bare at (1mm, 2mm) }} }}
"
    );
    let (_checked, artifacts) = build(&src, None);
    assert_eq!(cpl(&artifacts), HEADER);
}

#[test]
fn zero_rows_still_writes_header_and_a_place_free_design_has_no_cpl() {
    let placed = format!(
        "{LIB}
design B {{ inst hole: HOLE  net A: hole.A  layout {{ place hole at (1mm, 2mm) }} }}
"
    );
    let (_checked, artifacts) = build(&placed, None);
    assert_eq!(cpl(&artifacts), HEADER);

    let unplaced = placed.replace("  layout { place hole at (1mm, 2mm) }", "");
    let (_checked, artifacts) = build(&unplaced, None);
    assert!(artifacts.cpl.is_none());
}

#[test]
fn outline_origin_rotation_bottom_and_subdesign_composition_match_board_convention() {
    let src = format!(
        "{LIB}
design B {{
    inst top: SMD
    inst bottom: SMD
    net A: top.A, bottom.A
    layout {{
        board_outline: \"mechanical/outline.dxf\"
        place top at (-15mm, -10mm) rotate 359
        place bottom at (15mm, 10mm) rotate 37 side bottom
    }}
}}
"
    );
    let (checked, artifacts) = build(&src, Some(DXF_30X20));
    assert_eq!(
        cpl(&artifacts),
        format!(
            "{HEADER}U1,30.0000mm,20.0000mm,Bottom,217\n\
U2,0.0000mm,0.0000mm,Top,359\n"
        )
    );
    let board = cohdl::emit::kicad_pcb::emit_kicad_pcb(
        &checked.world,
        checked.ir.as_ref().unwrap(),
        "board",
    )
    .unwrap();
    assert!(board.contains("(at -15 -10 -1)"), "{board}");
    assert!(board.contains("(at 15 10 -143)"), "{board}");

    let subdesign = format!(
        "{LIB}
pub subdesign Block {{
    ports {{ optional A: Pin }}
    inst inner: SMD
    net _: A, inner.A
    layout {{ place inner at (1mm, 2mm) }}
}}
design B {{
    subdesign block: Block {{}}
    layout {{ place block at (5mm, 0mm) rotate 90 side bottom }}
}}
"
    );
    let (_checked, artifacts) = build(&subdesign, None);
    assert_eq!(
        cpl(&artifacts),
        format!("{HEADER}U1,7.0000mm,-1.0000mm,Bottom,270\n")
    );
}

#[test]
fn e815_lists_only_unplaced_machine_placed_instances_in_natural_order() {
    let src = format!(
        "{LIB}
design B {{
    #[designator(\"U10\")] inst ten: SMD
    #[designator(\"U2\")] inst two: SMD
    #[designator(\"U3\")] inst three: HOLE
    inst placed: SMD
    net A: ten.A, two.A, three.A, placed.A
    layout {{ place placed at (1mm, 2mm) }}
}}
"
    );
    let (mut checked, artifacts) = build(&src, None);
    let cpl = artifacts.cpl.as_ref().unwrap();
    cohdl::emit::cpl::report_unplaced(&mut checked.diags, cpl);
    let rendered = checked.diags.render(&checked.sm);
    assert_eq!(rendered.matches("E815").count(), 1, "{rendered}");
    assert!(
        rendered.contains("2 unplaced machine-placed instances: U2, U10"),
        "{rendered}"
    );
    assert!(!rendered.contains("U3"), "THT is not machine placed");
    assert_eq!(cpl.csv.matches('\n').count(), 2);
    let board = cohdl::emit::kicad_pcb::emit_kicad_pcb(
        &checked.world,
        checked.ir.as_ref().unwrap(),
        "board",
    )
    .unwrap();
    assert!(
        board.contains("(at 40 40"),
        "fixture proves staging exists: {board}"
    );
    assert!(!cpl.csv.contains("40.0000mm"));
}

fn make_project(root: &Path, src: &str) {
    let _ = std::fs::remove_dir_all(root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        "[package]\nname = \"t\"\n\n[design]\ntop = \"B\"\n",
    )
    .unwrap();
    std::fs::write(root.join("src/main.cohdl"), src).unwrap();
}

fn run(root: &Path, extra: &[&str]) -> std::process::Output {
    let mut args = vec!["build", root.to_str().unwrap(), "--no-std"];
    args.extend_from_slice(extra);
    Command::new(env!("CARGO_BIN_EXE_cohdl"))
        .args(&args)
        .output()
        .unwrap()
}

#[test]
fn physical_cli_writes_owned_cpl_json_and_e815_but_logical_only_does_not() {
    let tmp = std::env::temp_dir().join(format!("cohdl-rfc034-cpl-{}", std::process::id()));
    let src = format!(
        "{LIB}
design B {{
    inst placed: SMD
    inst missing: SMD
    net A: placed.A, missing.A
    layout {{ place placed at (1.23405mm, -1.23405mm) }}
}}
"
    );
    make_project(&tmp, &src);
    let cpl_path = tmp.join("out/t-cpl.csv");

    let logical = run(&tmp, &["--logical-only", "--json"]);
    assert!(
        logical.status.success(),
        "{}",
        String::from_utf8_lossy(&logical.stderr)
    );
    let logical_stdout = String::from_utf8(logical.stdout).unwrap();
    assert!(!logical_stdout.contains("\"cpl\""), "{logical_stdout}");
    assert!(!logical_stdout.contains("E815"), "{logical_stdout}");
    assert!(!cpl_path.exists());

    let output = run(&tmp, &["--json"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"cpl\": \""), "{stdout}");
    assert!(stdout.contains("t-cpl.csv"), "{stdout}");
    assert_eq!(stdout.matches("\"code\": \"E815\"").count(), 1, "{stdout}");
    assert!(
        stdout.contains("1 unplaced machine-placed instance: U1"),
        "{stdout}"
    );
    assert_eq!(
        std::fs::read_to_string(&cpl_path).unwrap(),
        format!("{HEADER}U2,1.2341mm,1.2341mm,Top,0\n")
    );
    let manifest = std::fs::read_to_string(tmp.join("out/.cohdl-manifest")).unwrap();
    assert!(
        manifest.lines().any(|line| line == "out/t-cpl.csv"),
        "{manifest}"
    );

    let before_net = std::fs::read(tmp.join("out/t.net")).unwrap();
    let before_bom = std::fs::read(tmp.join("out/t-bom.csv")).unwrap();
    make_project(
        &tmp,
        &src.replace(
            "    layout { place placed at (1.23405mm, -1.23405mm) }\n",
            "",
        ),
    );
    let no_place = run(&tmp, &["--json"]);
    assert!(no_place.status.success());
    assert!(!cpl_path.exists(), "stale owned CPL is removed");
    assert!(!String::from_utf8(no_place.stdout).unwrap().contains("E815"));
    assert_eq!(before_net, std::fs::read(tmp.join("out/t.net")).unwrap());
    assert_eq!(
        before_bom,
        std::fs::read(tmp.join("out/t-bom.csv")).unwrap()
    );

    std::fs::write(tmp.join("src/main.cohdl"), &src).unwrap();
    std::fs::write(&cpl_path, "foreign\n").unwrap();
    let refused = run(&tmp, &[]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("refusing"));
    assert_eq!(std::fs::read_to_string(cpl_path).unwrap(), "foreign\n");
}
