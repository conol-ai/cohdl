//! RFC-034 bottom-side CPL convention as a build option.

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts_with_suppliers, check_files_in};
use cohdl::supplier::{BottomRotation, BottomRotations};
use std::path::Path;
use std::process::{Command, Output};

fn checked(src: &str) -> cohdl::pipeline::Checked {
    let mut checked = check_files_in(
        "board",
        &[("src/main.cohdl".to_string(), src.to_string())],
        None,
    )
    .expect("design selection");
    checked.diags.sort(&checked.sm);
    checked
}

const LIB: &str = r#"
pub trait Ref { designator_prefix: "U" }
pub pad Land { shape: rect, size: (1mm, 0.5mm), layer: top_copper, plating: smd }
pub footprint FP { pad 1: Land at (0mm, 0mm) }
pub device D { pins { A: 1 [passive] } }
impl Ref for D {}
pub part P: D {
    primary { mfr: "M", mpn: "N", footprint: FP, supplier acme "A-1" rotate 90, supplier bolt "B-1" rotate 90 }
}
"#;

const DESIGN: &str = r#"
design B {
    #[designator("U1")] inst top: P
    #[designator("U2")] inst bottom: P
    net N: top.A, bottom.A
    layout {
        place top at (10mm, 20mm) rotate 37
        place bottom at (30mm, 20mm) rotate 37 side bottom
    }
}
"#;

fn supplier_cpl(rotations: BottomRotations) -> cohdl::pipeline::BuildArtifacts {
    let mut checked = checked(&format!("{LIB}\n{DESIGN}"));
    let artifacts = build_artifacts_with_suppliers(
        &mut checked,
        &LockState::default(),
        &["acme".to_string()],
        &rotations,
    )
    .expect("build");
    assert!(!checked.diags.has_errors());
    artifacts
}

#[test]
fn mirrored_changes_only_bottom_angles_and_map_records_build_option() {
    let artifacts = supplier_cpl([("acme".to_string(), BottomRotation::Mirrored)].into());
    assert_eq!(
        artifacts.supplier_maps[0].cpl.as_deref(),
        Some(concat!(
            "Designator,Mid X,Mid Y,Layer,Rotation\n",
            "U1,10.0000mm,-20.0000mm,Top,127\n",
            "U2,30.0000mm,-20.0000mm,Bottom,53\n",
        ))
    );
    assert!(artifacts.supplier_maps[0].json.contains(
        "\"schema_version\": 2,\n  \"pad_projection\": 1,\n  \"bottom_rotation\": \"mirrored\""
    ));
    assert_eq!(
        artifacts.cpl.as_ref().unwrap().csv,
        concat!(
            "Designator,Mid X,Mid Y,Layer,Rotation\n",
            "U1,10.0000mm,-20.0000mm,Top,37\n",
            "U2,30.0000mm,-20.0000mm,Bottom,217\n",
        )
    );
}

#[test]
fn no_mapping_makes_kicad_supplier_and_neutral_cpl_byte_identical() {
    let src = format!("{LIB}\n{DESIGN}").replace(" rotate 90, supplier bolt", ", supplier bolt");
    let mut checked = checked(&src);
    for rotations in [
        BottomRotations::new(),
        [("acme".to_string(), BottomRotation::Kicad)].into(),
    ] {
        let artifacts = build_artifacts_with_suppliers(
            &mut checked,
            &LockState::default(),
            &["acme".to_string()],
            &rotations,
        )
        .expect("build");
        assert_eq!(
            artifacts.supplier_maps[0].cpl.as_deref(),
            Some(artifacts.cpl.as_ref().unwrap().csv.as_str())
        );
    }
}

#[test]
fn explicit_kicad_and_default_have_identical_supplier_and_neutral_cpl() {
    let without = supplier_cpl(BottomRotations::new());
    let with = supplier_cpl([("acme".to_string(), BottomRotation::Kicad)].into());
    assert_eq!(without.supplier_maps[0].cpl, with.supplier_maps[0].cpl);
    for artifacts in [&without, &with] {
        let supplier = artifacts.supplier_maps[0].cpl.as_deref().unwrap();
        let neutral = &artifacts.cpl.as_ref().unwrap().csv;
        assert_eq!(supplier.lines().count(), neutral.lines().count());
        for (supplier_row, neutral_row) in supplier.lines().zip(neutral.lines()) {
            let supplier_columns: Vec<_> = supplier_row.split(',').collect();
            let neutral_columns: Vec<_> = neutral_row.split(',').collect();
            assert_eq!(supplier_columns[0], neutral_columns[0]);
            assert_eq!(supplier_columns[3], neutral_columns[3]);
            if supplier_columns[3] == "Bottom" {
                let supplier_angle: u16 = supplier_columns[4].parse().unwrap();
                let neutral_angle: u16 = neutral_columns[4].parse().unwrap();
                let mapping_rotation = 90;
                assert_eq!(
                    supplier_angle,
                    (neutral_angle + 360 - mapping_rotation) % 360
                );
            }
        }
    }
}

#[test]
fn withdrawn_top_level_supplier_declaration_is_an_ordinary_syntax_error() {
    let checked = checked("supplier acme { bottom_rotation: mirrored }\ndesign B {}\n");
    let rendered = checked.diags.render(&checked.sm);
    assert!(checked.diags.has_errors());
    assert!(rendered.contains("error[E010]"), "{rendered}");
    let syntax_errors: Vec<_> = checked
        .diags
        .iter()
        .filter(|diagnostic| diagnostic.code == "E010")
        .collect();
    assert_eq!(syntax_errors.len(), 1, "{rendered}");
    let expected_declarations = syntax_errors[0]
        .message
        .split(", found ")
        .next()
        .expect("top-level declaration diagnostic");
    assert!(
        !expected_declarations.contains("`supplier`"),
        "{}",
        syntax_errors[0].message
    );
    assert!(!rendered.contains("E816"), "{rendered}");
    assert!(!rendered.contains("E817"), "{rendered}");
}

fn make_cli_project(root: &Path) {
    let _ = std::fs::remove_dir_all(root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        "[package]\nname = \"t\"\nversion = \"1.0.0\"\n\n[design]\ntop = \"B\"\n",
    )
    .unwrap();
    std::fs::write(root.join("src/main.cohdl"), format!("{LIB}\n{DESIGN}")).unwrap();
}

fn run_build(root: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cohdl"))
        .args(["build", root.to_str().unwrap()])
        .args(extra)
        .output()
        .unwrap()
}

fn run_cli_project(root: &Path, extra: &[&str]) -> Output {
    let mut args = vec!["--no-std"];
    args.extend_from_slice(extra);
    run_build(root, &args)
}

fn temp_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("cohdl-w10-{label}-{}", std::process::id()))
}

fn assert_json_string(haystack: &str, key: &str, expected: &str) {
    let needle = format!("\"{key}\": \"{expected}\"");
    assert!(haystack.contains(&needle), "missing {needle} in {haystack}");
}

#[test]
fn invalid_options_exit_two_before_checking_and_write_nothing() {
    let root = temp_root("usage");
    make_cli_project(&root);
    // The invalid source proves option validation precedes dependency loading/checking.
    std::fs::write(root.join("src/main.cohdl"), "this is not CoHDL\n").unwrap();
    let cases: &[&[&str]] = &[
        &["--supplier", "acme", "--bottom-rotation", "acme=other"],
        &[
            "--supplier",
            "acme",
            "--bottom-rotation",
            "acme=kicad",
            "--bottom-rotation",
            "acme=mirrored",
        ],
        &["--supplier", "acme", "--bottom-rotation", "bolt=mirrored"],
        &["--supplier", "acme", "--bottom-rotation", "acme"],
    ];
    for args in cases {
        let output = run_cli_project(&root, args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(!root.join("out").exists(), "{args:?}");
        assert!(!root.join("design.lock").exists(), "{args:?}");
        assert!(
            !String::from_utf8_lossy(&output.stderr).contains("E010"),
            "{args:?}"
        );
    }
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn logical_only_records_option_without_writing_cpl() {
    let root = temp_root("logical");
    make_cli_project(&root);
    let output = run_cli_project(
        &root,
        &[
            "--supplier",
            "acme",
            "--bottom-rotation",
            "acme=mirrored",
            "--logical-only",
            "--json",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_json_string(&stdout, "bottom_rotation", "mirrored");
    assert!(!stdout.contains("\"cpl\""), "{stdout}");
    assert!(!root.join("out/t-cpl.csv").exists());
    assert!(!root.join("out/t-acme-cpl.csv").exists());
    assert!(std::fs::read_to_string(root.join("out/t-acme-map.json"))
        .unwrap()
        .contains("\"bottom_rotation\": \"mirrored\""));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn two_suppliers_have_independent_cpl_map_and_json_conventions() {
    let root = temp_root("two");
    make_cli_project(&root);
    let output = run_cli_project(
        &root,
        &[
            "--supplier",
            "acme",
            "--supplier",
            "bolt",
            "--bottom-rotation",
            "acme=mirrored",
            "--json",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let acme = stdout
        .split("\"acme\": {")
        .nth(1)
        .unwrap()
        .split("\"bolt\": {")
        .next()
        .unwrap();
    let bolt = stdout.split("\"bolt\": {").nth(1).unwrap();
    assert!(acme.contains("\"bottom_rotation\": \"mirrored\""), "{acme}");
    assert!(bolt.contains("\"bottom_rotation\": \"kicad\""), "{bolt}");
    assert!(std::fs::read_to_string(root.join("out/t-acme-map.json"))
        .unwrap()
        .contains("\"bottom_rotation\": \"mirrored\""));
    assert!(std::fs::read_to_string(root.join("out/t-bolt-map.json"))
        .unwrap()
        .contains("\"bottom_rotation\": \"kicad\""));
    let acme_cpl = std::fs::read_to_string(root.join("out/t-acme-cpl.csv")).unwrap();
    let bolt_cpl = std::fs::read_to_string(root.join("out/t-bolt-cpl.csv")).unwrap();
    let neutral = std::fs::read_to_string(root.join("out/t-cpl.csv")).unwrap();
    assert!(acme_cpl.contains("U2,30.0000mm,-20.0000mm,Bottom,53\n"));
    assert!(bolt_cpl.contains("U2,30.0000mm,-20.0000mm,Bottom,127\n"));
    assert!(neutral.contains("U2,30.0000mm,-20.0000mm,Bottom,217\n"));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn human_output_names_each_written_supplier_cpl_convention() {
    let root = temp_root("human");
    make_cli_project(&root);
    let output = run_cli_project(
        &root,
        &[
            "--supplier",
            "acme",
            "--supplier",
            "bolt",
            "--bottom-rotation",
            "acme=mirrored",
        ],
    );
    assert!(output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("t-acme-cpl.csv (bottom_rotation: mirrored)"),
        "{stderr}"
    );
    assert!(
        stderr.contains("t-bolt-cpl.csv (bottom_rotation: kicad)"),
        "{stderr}"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn preview_compiler_output_matches_independent_reference_byte_for_byte() {
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rfc034/bottom-preview");
    let out = fixture.join(format!("out-w10-{}", std::process::id()));
    let output = run_build(
        &fixture,
        &[
            "--supplier",
            "acme",
            "--bottom-rotation",
            "acme=mirrored",
            "--out-dir",
            out.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual = std::fs::read(out.join("bottom_preview-acme-cpl.csv")).unwrap();
    let reference = std::fs::read(fixture.join("handoff/cpl-acme-reference.csv")).unwrap();
    assert_eq!(actual, reference);
    let _ = std::fs::remove_dir_all(out);
    let _ = std::fs::remove_file(fixture.join("design.lock"));
    let _ = std::fs::remove_file(fixture.join("cohdl.lock"));
}
