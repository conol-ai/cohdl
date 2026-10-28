//! RFC-034 2a: per-instance supplier selection, E814 and the real CLI boundary.

use cohdl::lock::LockState;
use cohdl::pipeline::{
    build_artifacts, build_artifacts_with_suppliers, check_files_in, BuildArtifacts, Checked,
};
use cohdl::supplier::{requested_suppliers, AvlEntryIndex};
use std::path::Path;
use std::process::{Command, Output};

fn checked(src: &str) -> Checked {
    check_files_in(
        "board",
        &[("src/main.cohdl".to_string(), src.to_string())],
        None,
    )
    .expect("design selection")
}

fn build(src: &str, suppliers: &[&str]) -> (Checked, BuildArtifacts) {
    let mut checked = checked(src);
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    let suppliers: Vec<String> = suppliers.iter().map(|id| (*id).to_string()).collect();
    let artifacts = build_artifacts_with_suppliers(&mut checked, &LockState::default(), &suppliers)
        .expect("build");
    checked.diags.sort(&checked.sm);
    (checked, artifacts)
}

const LIB: &str = r#"
pub footprint FP {}
pub footprint OTHER {}
pub device D { pins { A: 1 [passive] } }
"#;

#[test]
fn selection_is_per_instance_and_uses_primary_then_first_eligible_alt() {
    let src = format!(
        r#"{LIB}
pub part PRIMARY: D {{
    primary {{ mfr: "M", mpn: "P", footprint: FP, supplier acme "PRIMARY-PN" at (1mm, -2mm) rotate 90 }}
    alt {{ mfr: "A", mpn: "P-A", supplier acme "PRIMARY-ALT" }}
}}
pub part ALT: D {{
    primary {{ mfr: "M", mpn: "ALT-P", footprint: FP }}
    alt {{ mfr: "A", mpn: "ALT-0", supplier acme "ALT-0-PN" }}
    alt {{ mfr: "A", mpn: "ALT-1", supplier acme "ALT-1-PN" }}
}}
pub part SKIP: D {{
    primary {{ mfr: "M", mpn: "SKIP-P", footprint: FP }}
    alt {{ mfr: "A", mpn: "SKIP-0", footprint: OTHER, supplier acme "WRONG-FP" }}
    alt {{ mfr: "A", mpn: "SKIP-1", supplier acme "INHERITED-FP" }}
}}
pub part LEFT: D {{
    primary {{ mfr: "M", mpn: "SHARED", footprint: FP }}
    alt {{ mfr: "A", mpn: "LEFT-A", supplier acme "LEFT-PN" }}
}}
pub part RIGHT: D {{
    primary {{ mfr: "M", mpn: "SHARED", footprint: FP }}
    alt {{ mfr: "A", mpn: "RIGHT-A", supplier acme "RIGHT-PN" }}
}}
design Board {{
    inst primary: PRIMARY
    inst alt: ALT
    inst skip: SKIP
    inst left: LEFT
    inst right: RIGHT
    nc: primary.A
    nc: alt.A
    nc: skip.A
    nc: left.A
    nc: right.A
}}
"#
    );
    let (checked, artifacts) = build(&src, &["acme"]);
    assert!(!checked.diags.has_errors());
    let selection = &artifacts.suppliers.selections[0];
    assert_eq!(selection.instances.len(), 5);

    let primary = selection.get("Board::primary").unwrap();
    let selected = primary.selected.as_ref().unwrap();
    assert_eq!(selected.entry, AvlEntryIndex::Primary);
    assert_eq!(selected.part_number, "PRIMARY-PN");
    assert_eq!(
        selected.mapping.at,
        (1_000_000_000_000_000, -2_000_000_000_000_000)
    );
    assert_eq!(selected.mapping.rotate, 90);

    let alt = selection
        .get("Board::alt")
        .unwrap()
        .selected
        .as_ref()
        .unwrap();
    assert_eq!(alt.entry, AvlEntryIndex::Alt(0));
    assert_eq!(alt.part_number, "ALT-0-PN");
    assert_eq!(alt.mpn, "ALT-0");
    assert_eq!(alt.mapping, Default::default());

    let skip = selection
        .get("Board::skip")
        .unwrap()
        .selected
        .as_ref()
        .unwrap();
    assert_eq!(skip.entry, AvlEntryIndex::Alt(1));
    assert_eq!(skip.part_number, "INHERITED-FP");

    // The same primary MPN is not a license to select at plain-BOM group level.
    assert_eq!(
        selection
            .get("Board::left")
            .unwrap()
            .selected
            .as_ref()
            .unwrap()
            .part_number,
        "LEFT-PN"
    );
    assert_eq!(
        selection
            .get("Board::right")
            .unwrap()
            .selected
            .as_ref()
            .unwrap()
            .part_number,
        "RIGHT-PN"
    );
}

#[test]
fn explicit_same_footprint_bare_and_qualified_alts_are_eligible() {
    for footprint in ["FP", "board::FP"] {
        let src = format!(
            r#"{LIB}
pub part P: D {{
    primary {{ mfr: "Yageo", mpn: "Y1", footprint: FP }}
    alt {{ mfr: "Samsung", mpn: "S1", footprint: {footprint}, supplier acme "A-200157" }}
}}
design Board {{
    inst p: P
    nc: p.A
}}
"#
        );
        let (checked, artifacts) = build(&src, &["acme"]);
        let selection = &artifacts.suppliers.selections[0];
        let selected = selection
            .get("Board::p")
            .unwrap()
            .selected
            .as_ref()
            .unwrap_or_else(|| panic!("{footprint} was rejected"));
        assert_eq!(selected.entry, AvlEntryIndex::Alt(0));
        assert_eq!(selected.part_number, "A-200157");
        assert_eq!(selected.mfr, "Samsung");
        assert_eq!(selected.mpn, "S1");
        assert!(!checked.diags.render(&checked.sm).contains("E814"));
    }

    let src = format!(
        r#"{LIB}
pub part P: D {{
    primary {{ mfr: "Yageo", mpn: "Y1", footprint: FP }}
    alt {{ mfr: "Samsung", mpn: "S1", footprint: OTHER, supplier acme "A-200157" }}
}}
design Board {{
    inst p: P
    nc: p.A
}}
"#
    );
    let (checked, artifacts) = build(&src, &["acme"]);
    assert!(artifacts.suppliers.selections[0]
        .get("Board::p")
        .unwrap()
        .selected
        .is_none());
    let rendered = checked.diags.render(&checked.sm);
    assert_eq!(rendered.matches("E814").count(), 1, "{rendered}");
    assert!(
        rendered.contains("effective footprint differs"),
        "{rendered}"
    );
}

#[test]
fn ordinary_bom_retains_stable_path_order_for_equal_natural_sort_keys() {
    let src = format!(
        r#"{LIB}
pub part P: D {{ primary {{ mfr: "M", mpn: "N", footprint: FP }} }}
design Board {{
    inst a: P
    inst b: P
    nc: a.A
    nc: b.A
}}
"#
    );
    let prior = LockState {
        designators: [
            ("Board::a".to_string(), "J1B".to_string()),
            ("Board::b".to_string(), "J1A".to_string()),
        ]
        .into_iter()
        .collect(),
        tombstones: Default::default(),
    };
    let mut checked = checked(&src);
    let artifacts = build_artifacts(&mut checked, &prior).expect("ordinary build");
    assert!(artifacts.suppliers.is_empty());
    assert_eq!(
        artifacts.bom,
        "Manufacturer,Comment,Designator,Footprint\n\"M\",\"N\",\"J1B,J1A\",\"FP\"\n"
    );
}

const NO_FOOTPRINT_PRIMARY: &str = r#"
pub device D { pins { A: 1 [passive] } }
pub part PRIMARY: D {
    primary { mfr: "M", mpn: "P", supplier acme "PRIMARY-PN" }
}
design Board {
    inst primary: PRIMARY
    nc: primary.A
}
"#;

#[test]
fn e814_is_per_instantiated_part_with_natural_designators_and_fp_reason() {
    let src = format!(
        r#"{LIB}
pub part USED: D {{
    primary {{ mfr: "M", mpn: "U", footprint: FP }}
    alt {{ mfr: "A", mpn: "U-A", footprint: OTHER, supplier acme "SKIP-ME" }}
}}
pub part UNUSED: D {{ primary {{ mfr: "M", mpn: "UNUSED", footprint: FP }} }}
design Board {{
    #[designator("U10")]
    inst ten: USED
    #[designator("U2")]
    inst two: USED
    nc: ten.A
    nc: two.A
}}
"#
    );
    let (checked, artifacts) = build(&src, &["acme"]);
    let rendered = checked.diags.render(&checked.sm);
    assert_eq!(rendered.matches("E814").count(), 1, "{rendered}");
    assert!(rendered.contains("part `USED`"), "{rendered}");
    assert!(rendered.contains("U2, U10"), "{rendered}");
    assert!(
        rendered.contains("effective footprint differs"),
        "{rendered}"
    );
    assert!(!rendered.contains("part `UNUSED`"), "{rendered}");
    assert_eq!(artifacts.suppliers.selections[0].instances.len(), 2);
    assert!(artifacts.suppliers.selections[0]
        .instances
        .values()
        .all(|instance| instance.selected.is_none()));
}

#[test]
fn wholly_unbound_id_gets_one_summary_and_excludes_unused_parts() {
    let src = format!(
        r#"{LIB}
pub part USED_A: D {{
    primary {{ mfr: "M", mpn: "A", footprint: FP }}
    alt {{ mfr: "M", mpn: "A-ALT", supplier bolt "B" }}
}}
pub part USED_B: D {{ primary {{ mfr: "M", mpn: "B", footprint: FP, supplier acme "A" }} }}
pub part UNUSED: D {{ primary {{ mfr: "M", mpn: "U", footprint: FP, supplier typo "UNUSED" }} }}
design Board {{
    inst a: USED_A
    inst b: USED_B
    nc: a.A
    nc: b.A
}}
"#
    );
    let (checked, _) = build(&src, &["missing"]);
    let rendered = checked.diags.render(&checked.sm);
    assert_eq!(rendered.matches("E814").count(), 1, "{rendered}");
    assert!(
        rendered.contains("supplier `missing` has no bindings"),
        "{rendered}"
    );
    assert!(
        rendered.contains("bound supplier IDs: acme, bolt"),
        "{rendered}"
    );
    assert!(!rendered.contains("typo"), "{rendered}");
    assert!(!rendered.contains("part `UNUSED`"), "{rendered}");
}

#[test]
fn requested_ids_keep_first_occurrence_order_and_no_supplier_is_legacy_behavior() {
    assert_eq!(
        requested_suppliers(&["bolt".into(), "acme".into(), "bolt".into()]),
        vec!["bolt".to_string(), "acme".to_string()]
    );
    let src = format!(
        r#"{LIB}
pub part P: D {{ primary {{ mfr: "M", mpn: "N", footprint: FP, supplier acme "A" }} }}
design Board {{
    inst p: P
    nc: p.A
}}
"#
    );
    let mut plain = checked(&src);
    let plain_artifacts = build_artifacts(&mut plain, &LockState::default()).unwrap();
    let (empty, empty_artifacts) = build(&src, &[]);
    assert!(plain_artifacts.suppliers.is_empty());
    assert!(empty_artifacts.suppliers.is_empty());
    assert_eq!(plain_artifacts.netlist, empty_artifacts.netlist);
    assert_eq!(plain_artifacts.bom, empty_artifacts.bom);
    assert_eq!(
        plain_artifacts.cpl.map(|cpl| cpl.csv),
        empty_artifacts.cpl.map(|cpl| cpl.csv)
    );
    assert_eq!(plain.diags.render(&plain.sm), empty.diags.render(&empty.sm));

    let (_, artifacts) = build(&src, &["missing", "acme", "missing", "acme"]);
    assert_eq!(artifacts.suppliers.selections.len(), 2);
    assert_eq!(artifacts.suppliers.selections[0].supplier, "missing");
    assert_eq!(artifacts.suppliers.selections[1].supplier, "acme");
}

#[test]
fn two_unbound_parts_each_get_one_warning_with_their_own_designators() {
    let src = format!(
        r#"{LIB}
pub part P: D {{ primary {{ mfr: "M", mpn: "P", footprint: FP, supplier acme "A" }} }}
pub part U: D {{ primary {{ mfr: "M", mpn: "U", footprint: FP }} }}
pub part V: D {{ primary {{ mfr: "M", mpn: "V", footprint: FP }} }}
design Board {{
    inst p: P
    inst u: U
    inst v: V
    nc: p.A
    nc: u.A
    nc: v.A
}}
"#
    );
    let (checked, _) = build(&src, &["acme"]);
    let rendered = checked.diags.render(&checked.sm);
    assert_eq!(rendered.matches("E814").count(), 2, "{rendered}");
    assert!(rendered.contains("part `U`"), "{rendered}");
    assert!(rendered.contains("part `V`"), "{rendered}");
    assert!(!rendered.contains("part `P` has no eligible"), "{rendered}");
}

fn make_project(root: &Path, source: &str) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        "[package]\nname = \"t\"\n[design]\ntop = \"Board\"\n",
    )
    .unwrap();
    std::fs::write(root.join("src/main.cohdl"), source).unwrap();
}

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cohdl"))
        .args(args)
        .arg(root)
        .arg("--no-std")
        .output()
        .unwrap()
}

#[test]
fn real_cli_accepts_repeatable_supplier_logical_only_and_renders_human_and_json_e814() {
    let tmp = std::env::temp_dir().join(format!("cohdl-rfc034-select-cli-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    let src = format!(
        r#"{LIB}
pub part P: D {{ primary {{ mfr: "M", mpn: "N", footprint: FP, supplier acme "A" }} }}
pub part U: D {{ primary {{ mfr: "M", mpn: "U", footprint: FP }} }}
design Board {{
    inst p: P
    inst u: U
    nc: p.A
    nc: u.A
}}
"#
    );
    make_project(&tmp, &src);

    let human = run(
        &tmp,
        &[
            "build",
            "--logical-only",
            "--supplier",
            "acme",
            "--supplier",
            "acme",
        ],
    );
    assert!(
        human.status.success(),
        "{}",
        String::from_utf8_lossy(&human.stderr)
    );
    let stderr = String::from_utf8(human.stderr).unwrap();
    assert_eq!(stderr.matches("E814").count(), 1, "{stderr}");
    assert!(stderr.contains("part `U`"), "{stderr}");
    assert!(!tmp.join("out/t-cpl.csv").exists());
    assert!(tmp.join("out/t.net").exists());
    assert!(tmp.join("out/t-bom.csv").exists());

    let json = run(
        &tmp,
        &["build", "--logical-only", "--supplier", "acme", "--json"],
    );
    assert!(
        json.status.success(),
        "{}",
        String::from_utf8_lossy(&json.stderr)
    );
    let stdout = String::from_utf8(json.stdout).unwrap();
    assert!(stdout.contains("\"verdict\": \"pass\""), "{stdout}");
    assert!(stdout.contains("\"code\": \"E814\""), "{stdout}");
    assert!(stdout.contains("\"severity\": \"warning\""), "{stdout}");
    assert!(stdout.contains("part `U`"), "{stdout}");

    let no_supplier = run(&tmp, &["build", "--logical-only", "--json"]);
    assert!(no_supplier.status.success());
    assert!(!String::from_utf8(no_supplier.stdout)
        .unwrap()
        .contains("E814"));
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn real_cli_logical_only_supplier_keeps_no_footprint_primary_e802() {
    let tmp = std::env::temp_dir().join(format!("cohdl-rfc034-select-nofp-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    make_project(&tmp, NO_FOOTPRINT_PRIMARY);
    let output = run(&tmp, &["build", "--logical-only", "--supplier", "acme"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("E802"), "{stderr}");
    assert!(stderr.contains("missing `footprint`"), "{stderr}");
    assert!(!stderr.contains("E814"), "{stderr}");
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn real_cli_rejects_supplier_outside_build_and_bad_or_missing_ids() {
    let tmp =
        std::env::temp_dir().join(format!("cohdl-rfc034-select-usage-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    make_project(&tmp, "design Board {}\n");
    for (args, expected) in [
        (
            &["check", "--supplier", "acme"][..],
            "`--supplier` is not valid with `check`",
        ),
        (
            &["build", "--supplier", "Acme"][..],
            "invalid supplier ID `Acme`",
        ),
    ] {
        let output = run(&tmp, args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let missing = Command::new(env!("CARGO_BIN_EXE_cohdl"))
        .current_dir(&tmp)
        .args(["build", "--supplier"])
        .output()
        .unwrap();
    assert_eq!(missing.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&missing.stderr).contains("--supplier needs a value"));
    let _ = std::fs::remove_dir_all(&tmp);
}
