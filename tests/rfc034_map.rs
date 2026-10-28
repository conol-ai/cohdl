//! RFC-034 §3.2 supplier map and supplier CPL conformance.

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts_with_suppliers, check_files_in, BuildArtifacts, Checked};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn build(src: &str, suppliers: &[&str]) -> (Checked, BuildArtifacts) {
    let mut checked = check_files_in(
        "board",
        &[("src/main.cohdl".to_string(), src.to_string())],
        None,
    )
    .expect("design selection");
    let suppliers: Vec<String> = suppliers.iter().map(|id| (*id).to_string()).collect();
    let artifacts = build_artifacts_with_suppliers(&mut checked, &LockState::default(), &suppliers);
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    (checked, artifacts.expect("build"))
}

fn map<'a>(artifacts: &'a BuildArtifacts, supplier: &str) -> &'a str {
    &artifacts
        .supplier_maps
        .iter()
        .find(|map| map.supplier == supplier)
        .expect("supplier map")
        .json
}

const LIB: &str = r#"
pub trait Ref { designator_prefix: "U" }
pub pad Land { shape: rect, size: (1mm, 0.5mm), layer: top_copper, plating: smd }
pub pad Ring { shape: annulus, size: (1.625mm, 1.025mm), layer: top_copper, plating: smd, paste: segmented_annulus(1.625mm, 1.125mm, 0.1mm) }
pub pad RectPaste { shape: rect, size: (0.6mm, 0.4mm), layer: top_copper, plating: smd, paste: (0.5mm, 0.3mm) }
pub pad CirclePaste { shape: circle, size: (0.6mm), layer: top_copper, plating: smd, paste: circle(0.5mm) }
pub pad Through { shape: circle, size: (1.7mm), layer: through_all, plating: plated_through_hole, drill: 1mm }
pub footprint FP_A {
    pad 3: Ring at (1mm, 2mm) rotate 90
    pad 2: RectPaste at (3mm, 4mm)
    pad 1: CirclePaste at (-3mm, 2mm)
    pad 1: Land at (-10mm, 2mm)
    pad 4: Through at (6mm, 7mm)
    mount_hole 1: plated at (1mm, 2mm) diameter 0.5mm
    mount_hole 2: non_plated at (2mm, 3mm) shape: rect size: (1mm, 2mm)
}
pub footprint FP_B { pad 1: Land at (-0.5mm, 0.25mm) }
pub footprint FP_UNUSED { pad 1: Land at (99mm, 99mm) }
pub device D1 { pins { A: 1 [passive] } }
pub device D4 { pins { A: 1 [passive], B: 2 [passive], C: 3 [passive], D: 4 [passive] } }
impl Ref for D1 {}
impl Ref for D4 {}
pub part P_A: D4 { primary { mfr: "Primary", mpn: "PRIMARY", footprint: FP_A } }
pub part P_B: D1 { primary { mfr: "Primary B", mpn: "PRIMARY B", footprint: FP_B } }
pub part UNUSED: D1 { primary { mfr: "Unused", mpn: "UNUSED", footprint: FP_UNUSED } }
"#;

#[test]
fn map_is_kicad_numbered_pad_projection_with_canonical_frame() {
    let src = format!(
        r#"{LIB}
design B {{ inst a: P_A inst b: P_B net N: a.A, a.B, a.C, a.D, b.A }}
"#
    );
    let (_checked, artifacts) = build(&src, &["acme"]);
    let map = map(&artifacts, "acme");
    assert!(map.starts_with("{\n  \"schema_version\": 1,\n  \"pad_projection\": 1,\n"));
    assert!(
        map.contains("\"footprints\": {\n    \"board::FP_A\": {"),
        "{map}"
    );
    assert!(map.contains("\"package\": \"board\""));
    assert!(map.contains("\"version\": \"0.0.0+unversioned\""));
    assert!(map.contains("[[\"1\",\"-10mm\",\"2mm\"],[\"1\",\"-3mm\",\"2mm\"],[\"2\",\"3mm\",\"4mm\"],[\"3\",\"1mm\",\"1.3375mm\"],[\"4\",\"6mm\",\"7mm\"]]"), "{map}");
    assert!(map.contains("[[\"1\",\"-0.5mm\",\"0.25mm\"]]"));
    assert!(!map.contains("FP_UNUSED"));
    assert!(!map.contains("99mm"));
    assert!(
        map.contains("\"pn\": null"),
        "unbound instances carry identity mapping: {map}"
    );
    assert!(map.contains("\"entry\": \"primary\""));
    assert!(map.contains("\"at\": [\"0mm\", \"0mm\"]"));
    assert_eq!(artifacts.supplier_maps[0].cpl, None);

    let canonical = "[[\"1\",\"-10mm\",\"2mm\"],[\"1\",\"-3mm\",\"2mm\"],[\"2\",\"3mm\",\"4mm\"],[\"3\",\"1mm\",\"1.3375mm\"],[\"4\",\"6mm\",\"7mm\"]]";
    assert!(map.contains(&format!(
        "\"frame\": \"{}\"",
        cohdl::hash::sha256_hex(canonical.as_bytes())
    )));
    let board = cohdl::emit::kicad_pcb::emit_kicad_pcb(
        &_checked.world,
        _checked.ir.as_ref().unwrap(),
        "board",
    )
    .unwrap();
    assert!(
        board.contains("(at 1 1.3375 90)"),
        "map and board share annulus anchor: {board}"
    );
    assert_eq!(
        board.matches("(pad \"\" smd").count(),
        6,
        "paste apertures are deliberately absent from map"
    );
}

#[test]
fn rfc_capacitor_alt_selection_is_the_same_in_bom_and_map() {
    let src = format!(
        r#"{LIB}
pub footprint CHIP_0402 {{ pad 1: Land at (-0.5mm, 0mm) pad 2: Land at (0.5mm, 0mm) }}
pub device MLCC {{ pins {{ A: 1 [passive], B: 2 [passive] }} }}
pub part C_100n_16V_X7R_0402: MLCC {{
    primary {{ mfr: "Yageo", mpn: "CC0402KRX7R7BB104", footprint: CHIP_0402 }}
    alt {{ mfr: "Samsung", mpn: "CL05B104KO5NNNC", supplier acme "A-200157" at (0.25mm, -0.5mm) rotate 90 }}
    alt {{ mfr: "Murata", mpn: "GRM155R71C104KA88D" }}
}}
design Board {{ inst c: C_100n_16V_X7R_0402 nc: c.A nc: c.B }}
"#
    );
    let (_checked, artifacts) = build(&src, &["acme"]);
    assert_eq!(artifacts.supplier_boms[0].csv, "Comment,Designator,Footprint,ACME Part #\n\"CL05B104KO5NNNC\",\"U1\",\"CHIP_0402\",\"A-200157\"\n");
    let map = map(&artifacts, "acme");
    assert!(map.contains("\"part\": \"board::C_100n_16V_X7R_0402\""));
    assert!(map.contains("\"footprint\": \"board::CHIP_0402\""));
    assert!(map.contains("\"entry\": 0"));
    assert!(map.contains("\"mfr\": \"Samsung\""));
    assert!(map.contains("\"mpn\": \"CL05B104KO5NNNC\""));
    assert!(map.contains("\"pn\": \"A-200157\""));
    assert!(map.contains("\"at\": [\"0.25mm\", \"-0.5mm\"]"));
    assert!(map.contains("\"rotate\": 90"));
}

#[test]
fn supplier_cpl_composes_top_bottom_right_and_oblique_angles() {
    let src = format!(
        r#"{LIB}
pub part MAPPED: D1 {{
    primary {{ mfr: "P", mpn: "P", footprint: FP_B }}
    alt {{ mfr: "A", mpn: "A", supplier acme "A1" at (2mm, 1mm) rotate 90 }}
}}
design B {{
    #[designator("U1")] inst top0: MAPPED
    #[designator("U2")] inst top90: MAPPED
    #[designator("U3")] inst bottom0: MAPPED
    #[designator("U4")] inst bottom90: MAPPED
    #[designator("U5")] inst top37: MAPPED
    #[designator("U6")] inst bottom37: MAPPED
    net N: top0.A, top90.A, bottom0.A, bottom90.A, top37.A, bottom37.A
    layout {{
        place top0 at (10mm, 20mm)
        place top90 at (20mm, 20mm) rotate 90
        place bottom0 at (30mm, 20mm) side bottom
        place bottom90 at (40mm, 20mm) rotate 90 side bottom
        place top37 at (50mm, 20mm) rotate 37
        place bottom37 at (60mm, 20mm) rotate 37 side bottom
    }}
}}
"#
    );
    let (_checked, artifacts) = build(&src, &["acme", "unbound"]);
    assert_eq!(
        artifacts.supplier_maps[0].cpl.as_deref(),
        Some(concat!(
            "Designator,Mid X,Mid Y,Layer,Rotation\n",
            "U1,12.0000mm,-21.0000mm,Top,90\n",
            "U2,21.0000mm,-18.0000mm,Top,180\n",
            "U3,28.0000mm,-21.0000mm,Bottom,90\n",
            "U4,41.0000mm,-22.0000mm,Bottom,180\n",
            "U5,52.1991mm,-19.5950mm,Top,127\n",
            "U6,59.0045mm,-22.0023mm,Bottom,127\n",
        ))
    );
    assert_eq!(
        artifacts.supplier_maps[1].cpl.as_deref(),
        artifacts.cpl.as_ref().map(|cpl| cpl.csv.as_str())
    );
    let unbound = map(&artifacts, "unbound");
    assert!(unbound.contains("\"pn\": null"));
    assert!(unbound.contains("\"entry\": \"primary\""));
    assert!(unbound.contains("\"mfr\": \"P\""));
    assert!(unbound.contains("\"at\": [\"0mm\", \"0mm\"]"));
}

fn make_project(root: &Path, source: &str) {
    let _ = std::fs::remove_dir_all(root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        "[package]\nname = \"t\"\nversion = \"1.2.3\"\n\n[design]\ntop = \"B\"\n",
    )
    .unwrap();
    std::fs::write(root.join("src/main.cohdl"), source).unwrap();
}

fn run(root: &Path, extra: &[&str]) -> Output {
    let mut args = vec!["build", root.to_str().unwrap(), "--no-std", "--json"];
    args.extend_from_slice(extra);
    Command::new(env!("CARGO_BIN_EXE_cohdl"))
        .args(args)
        .output()
        .unwrap()
}

fn success(output: &Output) -> &str {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::str::from_utf8(&output.stdout).unwrap()
}

#[test]
fn map_golden_board_physical_cpl_and_cli_lifecycle() {
    let root = std::env::temp_dir().join(format!("cohdl-rfc034-map-cli-{}", std::process::id()));
    let source = format!(
        r#"{LIB}
pub part P: D1 {{ primary {{ mfr: "M", mpn: "N", footprint: FP_B, supplier acme "A" at (1mm, 2mm) rotate 90 }} }}
design B {{ inst p: P nc: p.A layout {{ place p at (5mm, 6mm) rotate 90 }} }}
"#
    );
    make_project(&root, &source);
    let out = run(&root, &["--supplier", "acme", "--emit", "kicad_pcb"]);
    let stdout = success(&out);
    assert!(stdout.contains("\"map\": \""));
    assert!(stdout.contains("t-acme-cpl.csv"));
    assert!(stdout.contains("t-acme-map.json"));
    let map_path = root.join("out/t-acme-map.json");
    let cpl_path = root.join("out/t-acme-cpl.csv");
    let map_text = std::fs::read_to_string(&map_path).unwrap();
    assert!(map_text.contains("\"version\": \"1.2.3\""));
    assert_eq!(
        std::fs::read_to_string(&cpl_path).unwrap(),
        "Designator,Mid X,Mid Y,Layer,Rotation\nU1,7.0000mm,-5.0000mm,Top,180\n"
    );
    assert!(root.join("out/t-cpl.csv").metadata().is_ok());
    let manifest = std::fs::read_to_string(root.join("out/.cohdl-manifest")).unwrap();
    for name in ["t-acme-map.json", "t-acme-cpl.csv", "t-cpl.csv"] {
        assert!(
            manifest.lines().any(|line| line == format!("out/{name}")),
            "{manifest}"
        );
    }
    let validation = Command::new("python3").arg("-c").arg("import json, sys; sys.path.insert(0, sys.argv[1]); import smt_pos_map; doc=json.load(open(sys.argv[2])); smt_pos_map.validate_map(doc)")
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tools")).arg(&map_path).output().unwrap();
    assert!(
        validation.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&validation.stdout),
        String::from_utf8_lossy(&validation.stderr)
    );

    let logical = run(&root, &["--supplier", "acme", "--logical-only"]);
    let stdout = success(&logical);
    assert!(map_path.exists());
    assert!(
        !cpl_path.exists(),
        "logical-only leaves map but removes supplier CPL"
    );
    assert!(stdout.contains("t-acme-map.json"));
    assert!(!stdout.contains("\"cpl\": \""));
    let manifest = std::fs::read_to_string(root.join("out/.cohdl-manifest")).unwrap();
    assert!(manifest.contains("t-acme-map.json"));
    assert!(!manifest.contains("t-acme-cpl.csv"));
    assert!(!manifest.contains("t-cpl.csv"));

    let removed = run(&root, &["--logical-only"]);
    success(&removed);
    assert!(!map_path.exists(), "revoked supplier map is stale");
    let manifest = std::fs::read_to_string(root.join("out/.cohdl-manifest")).unwrap();
    assert!(!manifest.lines().any(|line| line == "out/t-acme-map.json"));
    let foreign = root.join("out/t-acme-map.json");
    std::fs::write(&foreign, "foreign\n").unwrap();
    let refused = run(&root, &["--supplier", "acme", "--logical-only"]);
    assert!(
        !refused.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&refused.stdout),
        String::from_utf8_lossy(&refused.stderr)
    );
    assert!(String::from_utf8_lossy(&refused.stderr).contains("refusing to overwrite"));
    assert_eq!(std::fs::read_to_string(&foreign).unwrap(), "foreign\n");

    assert!(!String::from_utf8(removed.stdout)
        .unwrap()
        .contains("\"suppliers\""));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn golden_map_is_current_and_board_backed() {
    let root = std::env::temp_dir().join(format!("cohdl-rfc034-map-golden-{}", std::process::id()));
    let source = format!(
        r#"{LIB}
pub part P: D1 {{ primary {{ mfr: "M", mpn: "N", footprint: FP_B, supplier acme "A" at (1mm, 2mm) rotate 90 }} }}
design B {{ inst p: P nc: p.A layout {{ place p at (5mm, 6mm) rotate 90 }} }}
"#
    );
    make_project(&root, &source);
    let out = run(&root, &["--supplier", "acme", "--emit", "kicad_pcb"]);
    success(&out);
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rfc034");
    assert_eq!(
        std::fs::read_to_string(root.join("out/t-acme-map.json")).unwrap(),
        std::fs::read_to_string(fixture.join("map-golden.json")).unwrap()
    );
    assert_eq!(
        std::fs::read_to_string(root.join("out/t.kicad_pcb")).unwrap(),
        std::fs::read_to_string(fixture.join("map-golden.kicad_pcb")).unwrap()
    );
    let _ = std::fs::remove_dir_all(&root);
}
