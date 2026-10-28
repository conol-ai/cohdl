//! RFC-034 supplier declarations and bottom-side CPL convention.

use cohdl::fmt::format_source;
use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts_with_suppliers, check_files_in};
use cohdl::span::SourceMap;
use std::path::Path;
use std::process::Command;

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

fn e816(declaration: &str) -> cohdl::diag::Diagnostic {
    let checked = checked(&format!("{declaration}\ndesign B {{}}\n"));
    let rendered = checked.diags.render(&checked.sm);
    let errors: Vec<_> = checked
        .diags
        .iter()
        .filter(|diagnostic| diagnostic.code == "E816")
        .collect();
    assert_eq!(errors.len(), 1, "{declaration}:\n{rendered}");
    errors[0].clone()
}

#[test]
fn declaration_formats_to_one_line_and_retains_both_values() {
    for value in ["kicad", "mirrored"] {
        let src = format!("supplier acme {{\n bottom_rotation : {value},\n}}\n");
        let expected = format!("supplier acme {{ bottom_rotation: {value} }}\n");
        let once = format_source("src/main.cohdl", &src).expect("format declaration");
        assert_eq!(once, expected);
        assert_eq!(format_source("src/main.cohdl", &once).unwrap(), once);
        assert!(!checked(&format!("{expected}design B {{}}\n"))
            .diags
            .has_errors());
    }
}

#[test]
fn e816_rejects_each_invalid_declaration_semantic() {
    let cases = [
        (
            "supplier Acme { bottom_rotation: kicad }",
            "invalid supplier identifier",
        ),
        (
            "supplier a23456789012345678901234567890123 { bottom_rotation: kicad }",
            "at most 32",
        ),
        ("supplier mpn { bottom_rotation: kicad }", "reserved"),
        ("supplier mfr { bottom_rotation: kicad }", "reserved"),
        ("supplier acme { other: kicad }", "unknown key"),
        (
            "supplier acme { bottom_rotation: other }",
            "unknown bottom_rotation",
        ),
        (
            "supplier acme { bottom_rotation: kicad, bottom_rotation: mirrored }",
            "duplicate key",
        ),
        ("pub supplier acme { bottom_rotation: kicad }", "`pub`"),
    ];
    for (source, expected) in cases {
        let diagnostic = e816(source);
        assert!(
            diagnostic.message.contains(expected),
            "{source}: {}",
            diagnostic.message
        );
    }
}

#[test]
fn duplicate_declaration_spans_two_packages() {
    use cohdl::check::check_declarations_in;
    use cohdl::diag::Diagnostics;
    use cohdl::resolve::ModuleInfo;

    let mut sm = SourceMap::new();
    let mut diags = Diagnostics::new();
    let sources = [
        (
            "vendor_a/src/lib.cohdl",
            "supplier acme { bottom_rotation: kicad }",
        ),
        (
            "vendor_b/src/lib.cohdl",
            "supplier acme { bottom_rotation: mirrored }",
        ),
    ];
    let parsed = sources
        .iter()
        .map(|(name, source)| {
            let file = sm.add_file(*name, *source);
            cohdl::parse::parse(cohdl::lex::lex(file, sm.text(file), &mut diags), &mut diags)
        })
        .collect();
    let modules = ["vendor_a", "vendor_b"].map(|package| ModuleInfo {
        package: package.to_string(),
        module: package.to_string(),
    });
    let _world = check_declarations_in(parsed, &modules, &mut diags);
    diags.sort(&sm);
    let rendered = diags.render(&sm);
    assert_eq!(rendered.matches("error[E816]").count(), 1, "{rendered}");
    assert!(rendered.contains("vendor_a/src/lib.cohdl"), "{rendered}");
    assert!(rendered.contains("vendor_b/src/lib.cohdl"), "{rendered}");
}

const LIB: &str = r#"
pub trait Ref { designator_prefix: "U" }
pub pad Land { shape: rect, size: (1mm, 0.5mm), layer: top_copper, plating: smd }
pub footprint FP { pad 1: Land at (0mm, 0mm) }
pub device D { pins { A: 1 [passive] } }
impl Ref for D {}
pub part P: D {
    primary { mfr: "M", mpn: "N", footprint: FP, supplier acme "A-1" rotate 90 }
}
"#;

fn supplier_cpl(declaration: &str) -> (cohdl::pipeline::Checked, cohdl::pipeline::BuildArtifacts) {
    let src = format!(
        r#"{declaration}
{LIB}
design B {{
    #[designator("U1")] inst top: P
    #[designator("U2")] inst bottom: P
    net N: top.A, bottom.A
    layout {{
        place top at (10mm, 20mm) rotate 37
        place bottom at (30mm, 20mm) rotate 37 side bottom
    }}
}}
"#
    );
    let mut checked = checked(&src);
    let artifacts =
        build_artifacts_with_suppliers(&mut checked, &LockState::default(), &["acme".to_string()])
            .expect("build");
    checked.diags.sort(&checked.sm);
    (checked, artifacts)
}

#[test]
fn mirrored_changes_only_bottom_angles_and_map_declares_actual_convention() {
    let (_checked, artifacts) = supplier_cpl("supplier acme { bottom_rotation: mirrored }");
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
fn explicit_kicad_and_no_declaration_have_identical_cpl() {
    let (undeclared, without) = supplier_cpl("");
    let (declared, with) = supplier_cpl("supplier acme { bottom_rotation: kicad }");
    assert_eq!(without.supplier_maps[0].cpl, with.supplier_maps[0].cpl);
    assert!(!undeclared.diags.iter().any(|d| d.code == "E817"));
    assert!(!declared.diags.iter().any(|d| d.code == "E817"));
}

fn run_build(root: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_cohdl"))
        .args([
            "build",
            root.to_str().unwrap(),
            "--no-std",
            "--supplier",
            "acme",
            "--json",
        ])
        .output()
        .unwrap()
}

fn make_cli_project(root: &Path, source: &str) {
    let _ = std::fs::remove_dir_all(root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        "[package]\nname = \"t\"\nversion = \"1.0.0\"\n\n[design]\ntop = \"ZTop\"\n",
    )
    .unwrap();
    std::fs::write(root.join("src/main.cohdl"), source).unwrap();
}

#[test]
fn e817_only_accompanies_an_actual_supplier_cpl() {
    let root = std::env::temp_dir().join(format!("cohdl-w9-e817-cpl-{}", std::process::id()));
    make_cli_project(
        &root,
        &format!("{LIB}\ndesign ZTop {{ inst p: P nc: p.A }}\n"),
    );
    let unplaced = run_build(&root);
    assert!(unplaced.status.success());
    let stdout = String::from_utf8(unplaced.stdout).unwrap();
    assert!(!stdout.contains("E817"), "{stdout}");
    assert!(!root.join("out/t-acme-cpl.csv").exists());

    std::fs::write(
        root.join("src/main.cohdl"),
        format!("{LIB}\ndesign ZTop {{ inst p: P nc: p.A layout {{ place p at (5mm, 6mm) }} }}\n"),
    )
    .unwrap();
    let placed = run_build(&root);
    assert!(placed.status.success());
    let stdout = String::from_utf8(placed.stdout).unwrap();
    assert_eq!(stdout.matches("\"code\": \"E817\"").count(), 1, "{stdout}");
    assert!(root.join("out/t-acme-cpl.csv").exists());
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn e817_span_is_the_selected_top_design_not_the_first_design_name() {
    let root = std::env::temp_dir().join(format!("cohdl-w9-e817-top-{}", std::process::id()));
    make_cli_project(
        &root,
        &format!(
            "{LIB}\ndesign AFirst {{ inst p: P nc: p.A }}\ndesign ZTop {{ inst p: P nc: p.A layout {{ place p at (5mm, 6mm) }} }}\n"
        ),
    );
    let output = run_build(&root);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.matches("\"code\": \"E817\"").count(), 1, "{stdout}");
    let ztop_line = std::fs::read_to_string(root.join("src/main.cohdl"))
        .unwrap()
        .lines()
        .position(|line| line.starts_with("design ZTop"))
        .unwrap()
        + 1;
    assert!(
        stdout.contains(&format!("\"start_line\": {ztop_line}")),
        "{stdout}"
    );
    assert!(!stdout.contains("\"start_line\": 10"), "{stdout}");
    let _ = std::fs::remove_dir_all(root);
}
