//! RFC-034 2b: supplier BOM contents and their on-disk lifecycle.

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts_with_suppliers, check_files_in, BuildArtifacts, Checked};
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
    build_with_lock(src, suppliers, &LockState::default())
}

fn build_with_lock(src: &str, suppliers: &[&str], lock: &LockState) -> (Checked, BuildArtifacts) {
    let mut checked = checked(src);
    assert!(
        !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    let suppliers: Vec<String> = suppliers.iter().map(|id| (*id).to_string()).collect();
    let artifacts = build_artifacts_with_suppliers(&mut checked, lock, &suppliers).expect("build");
    (checked, artifacts)
}

const LIB: &str = r#"
pub trait Ref { designator_prefix: "R" }
pub device D { pins { A: 1 [passive] } }
impl Ref for D {}
pub footprint FP {}
pub footprint LONG_FP {}
"#;

#[test]
fn rfc_alt_binding_uses_selected_manufacturer_mpn_and_comment() {
    let src = r#"pub footprint CHIP_0402 {}
pub device MLCC { pins { A: 1 [passive] } }
pub part C_100n_16V_X7R_0402: MLCC {
    primary { mfr: "Yageo", mpn: "CC0402KRX7R7BB104", footprint: CHIP_0402 }
    alt { mfr: "Samsung", mpn: "CL05B104KO5NNNC", supplier acme "A-200157" }
    alt { mfr: "Murata", mpn: "GRM155R71C104KA88D" }
}
design Board { inst c: C_100n_16V_X7R_0402 nc: c.A }
"#;
    let (checked, artifacts) = build(src, &["acme"]);
    assert_eq!(
        artifacts.supplier_boms[0].csv,
        "Comment,Designator,Footprint,ACME Part #\n\
         \"CL05B104KO5NNNC\",\"U1\",\"CHIP_0402\",\"A-200157\"\n"
    );
    let enet = cohdl::emit::easyeda::emit_enet_with_supplier(
        &checked.world,
        checked.ir.as_ref().unwrap(),
        artifacts.suppliers.selections.first(),
    );
    let component = enet
        .split("\"components\": {")
        .nth(1)
        .unwrap()
        .split("\"designRule\"")
        .next()
        .unwrap();
    for property in [
        "\"Manufacturer\": \"Samsung\"",
        "\"Manufacturer Part\": \"CL05B104KO5NNNC\"",
        "\"Supplier\": \"ACME\"",
        "\"Supplier Part\": \"A-200157\"",
    ] {
        assert!(component.contains(property), "missing {property}");
    }
    assert!(!component.contains("Yageo"));
    assert!(!component.contains("CC0402KRX7R7BB104"));
}

#[test]
fn grouping_sorting_escaping_and_plain_bom_are_deterministic() {
    let src = format!(
        r#"{LIB}
pub part BOUND: D {{
    primary {{ mfr: "Primary", mpn: "PRIMARY", footprint: FP }}
    alt {{ mfr: "Selected", mpn: "M,2", supplier acme "Z,PN", supplier bolt "BOLT-2" }}
}}
pub part SHARED_A: D {{ primary {{ mfr: "A", mpn: "SAME", footprint: FP }} }}
pub part SHARED_B: D {{ primary {{ mfr: "B", mpn: "SAME", footprint: LONG_FP }} }}
#[source_unspecified("mpn")]
pub part UNKNOWN: D {{ primary {{ mfr: "Unknown", mpn: "", footprint: FP }} }}
#[source_unspecified("mpn")]
pub part UNKNOWN_OTHER: D {{ primary {{ mfr: "Unknown", mpn: "", footprint: FP }} }}
#[source_unspecified("mpn")]
pub part UNKNOWN_ALT: D {{
    primary {{ mfr: "Unknown", mpn: "", footprint: FP, supplier acme "EMPTY" }}
}}
design Board {{
    inst z: BOUND
    inst a: BOUND
    inst shared_a: SHARED_A
    inst shared_b: SHARED_B
    inst unknown: UNKNOWN
    inst other: UNKNOWN_OTHER
    inst empty: UNKNOWN_ALT
    nc: z.A
    nc: a.A
    nc: shared_a.A
    nc: shared_b.A
    nc: unknown.A
    nc: other.A
    nc: empty.A
}}
"#
    );
    let lock = LockState {
        designators: [
            ("Board::a", "R2"),
            ("Board::z", "R10"),
            ("Board::shared_a", "R1"),
            ("Board::shared_b", "R3"),
            ("Board::unknown", "R5"),
            ("Board::other", "R4"),
            ("Board::empty", "R6"),
        ]
        .into_iter()
        .map(|(path, designator)| (path.to_string(), designator.to_string()))
        .collect(),
        tombstones: Default::default(),
    };
    let (_, plain) = build_with_lock(&src, &[], &lock);
    let (_, supplied) = build_with_lock(&src, &["bolt", "acme", "acme"], &lock);
    assert!(plain.supplier_boms.is_empty());
    assert_eq!(plain.bom, supplied.bom);
    assert_eq!(supplied.supplier_boms.len(), 2);
    assert_eq!(
        supplied.supplier_boms[1].csv,
        "Comment,Designator,Footprint,ACME Part #\n\
         \"\",\"R5\",\"FP\",\"\"\n\
         \"\",\"R4\",\"FP\",\"\"\n\
         \"SAME\",\"R1\",\"FP\",\"\"\n\
         \"SAME\",\"R3\",\"LONG_FP\",\"\"\n\
         \"\",\"R6\",\"FP\",\"EMPTY\"\n\
         \"M,2\",\"R2,R10\",\"FP\",\"Z,PN\"\n"
    );
    assert_eq!(
        supplied.supplier_boms[0].csv,
        "Comment,Designator,Footprint,BOLT Part #\n\
         \"\",\"R5\",\"FP\",\"\"\n\
         \"\",\"R6\",\"FP\",\"\"\n\
         \"\",\"R4\",\"FP\",\"\"\n\
         \"SAME\",\"R1\",\"FP\",\"\"\n\
         \"SAME\",\"R3\",\"LONG_FP\",\"\"\n\
         \"M,2\",\"R2,R10\",\"FP\",\"BOLT-2\"\n"
    );
}

#[test]
fn csv_quote_doubles_embedded_quotes() {
    use cohdl::supplier::{
        InstanceSelection, SelectedSupplierEntry, SupplierSelection, SupplierSelections,
    };
    let selection = SupplierSelection {
        supplier: "acme".into(),
        instances: [(
            "Board::p".into(),
            InstanceSelection {
                instance: "Board::p".into(),
                designator: "R2".into(),
                part: "P".into(),
                footprint: "pkg::F\"P".into(),
                selected: Some(SelectedSupplierEntry {
                    entry: cohdl::supplier::AvlEntryIndex::Primary,
                    mfr: "M".into(),
                    mpn: "M\"2".into(),
                    part_number: "PN\"2".into(),
                    mapping: Default::default(),
                }),
            },
        )]
        .into_iter()
        .collect(),
    };
    let selections = SupplierSelections {
        selections: vec![selection],
    };
    let boms = cohdl::emit::supplier_bom::emit_supplier_boms(
        &cohdl::resolve::World::default(),
        &selections,
    );
    assert_eq!(
        boms[0].csv,
        "Comment,Designator,Footprint,ACME Part #\n\"M\"\"2\",\"R2\",\"F\"\"P\",\"PN\"\"2\"\n"
    );
}

fn make_project(root: &Path, source: &str) {
    let _ = std::fs::remove_dir_all(root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        "[package]\nname = \"t\"\n\n[design]\ntop = \"Board\"\n",
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

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn supplier_files_follow_manifest_stale_foreign_and_json_contracts() {
    let tmp = std::env::temp_dir().join(format!("cohdl-rfc034-bom-cli-{}", std::process::id()));
    let src = format!(
        r#"{LIB}
pub part P: D {{
    primary {{ mfr: "M", mpn: "N", footprint: FP, supplier acme "A", supplier bolt "B" }}
}}
design Board {{ inst p: P nc: p.A }}
"#
    );
    make_project(&tmp, &src);
    let acme = tmp.join("out/t-acme-bom.csv");
    let bolt = tmp.join("out/t-bolt-bom.csv");

    let plain = run(&tmp, &["--logical-only"]);
    assert_success(&plain);
    let plain_bom = std::fs::read(tmp.join("out/t-bom.csv")).unwrap();
    assert!(!String::from_utf8(plain.stdout)
        .unwrap()
        .contains("\"suppliers\""));

    let both = run(
        &tmp,
        &[
            "--logical-only",
            "--supplier",
            "bolt",
            "--supplier",
            "acme",
            "--supplier",
            "acme",
        ],
    );
    assert_success(&both);
    assert_eq!(plain_bom, std::fs::read(tmp.join("out/t-bom.csv")).unwrap());
    assert_eq!(
        std::fs::read_to_string(&acme).unwrap(),
        "Comment,Designator,Footprint,ACME Part #\n\"N\",\"R1\",\"FP\",\"A\"\n"
    );
    assert!(bolt.exists());
    let stdout = String::from_utf8(both.stdout).unwrap();
    assert_eq!(stdout.matches("\"suppliers\"").count(), 1, "{stdout}");
    assert_eq!(stdout.matches("\"bom\":").count(), 3, "{stdout}");
    assert!(stdout.contains("\"acme\": {"), "{stdout}");
    assert!(stdout.contains("\"bolt\": {"), "{stdout}");
    assert!(stdout.contains("t-acme-bom.csv"), "{stdout}");
    assert!(stdout.contains("t-bolt-bom.csv"), "{stdout}");
    assert!(stdout.contains("\"map\""), "{stdout}");
    assert!(stdout.contains("t-acme-map.json"), "{stdout}");
    let manifest = std::fs::read_to_string(tmp.join("out/.cohdl-manifest")).unwrap();
    assert!(manifest.lines().any(|line| line == "out/t-acme-bom.csv"));
    assert!(manifest.lines().any(|line| line == "out/t-bolt-bom.csv"));

    let physical = run(&tmp, &["--supplier", "acme"]);
    assert_success(&physical);
    assert_eq!(
        std::fs::read_to_string(&acme).unwrap(),
        "Comment,Designator,Footprint,ACME Part #\n\"N\",\"R1\",\"FP\",\"A\"\n"
    );
    assert!(String::from_utf8(physical.stdout)
        .unwrap()
        .contains("t-acme-bom.csv"));

    let switched = run(&tmp, &["--logical-only", "--supplier", "acme"]);
    assert_success(&switched);
    assert!(!bolt.exists(), "revoked supplier's stale BOM is removed");
    assert!(acme.exists());
    let stdout = String::from_utf8(switched.stdout).unwrap();
    assert!(stdout.contains("\"acme\": {"), "{stdout}");
    assert!(!stdout.contains("\"bolt\": {"), "{stdout}");
    let manifest = std::fs::read_to_string(tmp.join("out/.cohdl-manifest")).unwrap();
    assert!(!manifest.contains("t-bolt-bom.csv"), "{manifest}");

    std::fs::write(&bolt, "foreign\n").unwrap();
    let refused = run(&tmp, &["--logical-only", "--supplier", "bolt"]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("refusing to overwrite"));
    assert_eq!(std::fs::read_to_string(&bolt).unwrap(), "foreign\n");
    let manifest = std::fs::read_to_string(tmp.join("out/.cohdl-manifest")).unwrap();
    assert!(!manifest.contains("t-bolt-bom.csv"), "{manifest}");

    let removed = run(&tmp, &["--logical-only"]);
    assert_success(&removed);
    assert!(!acme.exists());
    assert!(!String::from_utf8(removed.stdout)
        .unwrap()
        .contains("\"suppliers\""));
    let _ = std::fs::remove_dir_all(&tmp);
}
