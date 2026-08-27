//! LCEDA Pro `.enet` netlist emitter (`build --emit easyeda`).
//!
//! The load-bearing properties: (1) zero impact — with or without the flag,
//! every other artifact's bytes and the verdict are identical; (2) byte
//! determinism; (3) agreement with the `.net` emitter — both netlists come
//! from the same derivations, and these tests cross-check the strings; (4)
//! `nc` pins are represented by their guaranteed absence; (5) the document
//! shape is the v1 emitter's proven one (top-level struct order,
//! string-sorted object keys).

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};
use std::path::PathBuf;
use std::process::Command;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Two netted parts (one multi-pad logical pin, one `nc` pin each) plus a
/// pin-less mechanical part: every pinInfoMap case in one fixture.
const FIXTURE: &str = r#"
pub trait Ic { designator_prefix: "U" }
pub trait Hole { designator_prefix: "H" }
pub pad P { shape: rect, size: (0.8mm, 0.3mm), layer: top_copper, plating: smd }
pub footprint F {
    pad 1: P at (1mm, 2mm)
    pad 2: P at (-1mm, -2mm)
    pad 3: P at (0mm, 3mm)
    pad 4: P at (0mm, -3mm)
}
pub footprint FH {
}
pub device D { pins { A: 1 [passive], GND: 2, 3 [passive], X: 4 [passive] } }
impl Ic for D {}
pub device MH { }
impl Hole for MH {}
pub part PT: D { primary { mfr: "m", mpn: "x", footprint: F } }
pub part PH: MH { primary { mfr: "m", mpn: "h", footprint: FH } }

design B {
    inst u1: PT
    inst u2: PT
    inst h1: PH
    net N: u1.A, u2.A
    net G: u1.GND, u2.GND
    nc: u1.X, u2.X
}
"#;

fn checked_fixture(src: &str) -> cohdl::pipeline::Checked {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    let _artifacts = build_artifacts(&mut checked, &LockState::default()).expect("build");
    assert!(
        !checked.diags.has_errors(),
        "fixture must build cleanly:\n{}",
        checked.diags.render(&checked.sm)
    );
    checked
}

fn emit(src: &str) -> String {
    let checked = checked_fixture(src);
    let ir = checked.ir.as_ref().unwrap();
    cohdl::emit::easyeda::emit_enet(&checked.world, ir)
}

#[test]
fn shape_is_the_v1_document() {
    let enet = emit(FIXTURE);
    // Top level opens with the version and the components map, and keeps the
    // v1 struct order for the remaining keys.
    assert!(enet.starts_with("{\n  \"version\": \"2.0.0\",\n  \"components\": {\n"));
    let pos = |needle: &str| enet.find(needle).unwrap_or_else(|| panic!("no {needle}"));
    assert!(pos("\"designRule\"") < pos("\"differentialPair\""));
    assert!(pos("\"differentialPair\"") < pos("\"netClass\""));
    assert!(pos("\"netClass\"") < pos("\"equalLengthNetGroup\""));
    // designRule: trackPhysics before netRule (v1 struct order), one netRule
    // row per net with the empty TrackPhysics binding.
    assert!(pos("\"trackPhysics\": {}") < pos("\"netRule\""));
    assert!(enet.contains(
        "\"net\": \"N\",\n        \"ruleMap\": {\n          \"TrackPhysics\": \"\"\n        }"
    ));
    assert!(enet.contains("\"net\": \"G\","));

    // Unique IDs count up in designator natural order: H1, U1, U2.
    assert!(enet.contains("\"Designator\": \"H1\",\n        \"DeviceName\": \"MH\""));
    let gge1 = component_block(&enet, "gge1");
    assert!(gge1.contains("\"Designator\": \"H1\""));
    assert!(
        gge1.contains("\"pinInfoMap\": {}"),
        "a pin-less mechanical part still converts to PCB, with no pins"
    );
    let gge2 = component_block(&enet, "gge2");
    assert!(gge2.contains("\"Designator\": \"U1\""));

    // The fixed v1 props plus the part-derived ones, string-sorted.
    assert!(gge2.contains(
        "\"props\": {\n        \"Add into BOM\": \"yes\",\n        \"Convert to PCB\": \"yes\",\n        \"Designator\": \"U1\",\n        \"DeviceName\": \"D\",\n        \"FootprintName\": \"board::F\",\n        \"Manufacturer\": \"m\",\n        \"Manufacturer Part\": \"x\",\n        \"Name\": \"D\",\n        \"Unique ID\": \"gge2\"\n      }"
    ));
    // A pin row binds its net by name and carries the v1 props shape.
    assert!(gge2.contains(
        "\"1\": {\n          \"name\": \"A\",\n          \"number\": \"1\",\n          \"net\": \"N\",\n          \"props\": {\n            \"Pin Number\": \"1\"\n          }\n        }"
    ));
}

/// The component object rendered for `id` (up to the next component or the
/// end of the components map).
fn component_block(enet: &str, id: &str) -> String {
    let start = enet
        .find(&format!("\"{}\": {{", id))
        .unwrap_or_else(|| panic!("no component {id}"));
    let rest = &enet[start..];
    let end = rest[1..]
        .find("\n    \"gge")
        .map(|i| i + 1)
        .unwrap_or_else(|| rest.find("\n  },").expect("components map end"));
    rest[..end].to_string()
}

#[test]
fn multi_pad_pins_flatten_and_nc_is_absent() {
    let enet = emit(FIXTURE);
    // GND spans pads 2 and 3: one row per physical pad, per instance —
    // 4 pin rows plus the netRule row make 5 mentions of net G.
    assert_eq!(enet.matches("\"name\": \"GND\"").count(), 4);
    assert_eq!(enet.matches("\"net\": \"G\"").count(), 5);
    // The nc pin (X, pad 4) is represented by its guaranteed absence.
    assert!(!enet.contains("\"name\": \"X\""));
    assert!(!enet.contains("\"Pin Number\": \"4\""));
}

#[test]
fn agrees_with_the_kicad_netlist() {
    let checked = checked_fixture(FIXTURE);
    let ir = checked.ir.as_ref().unwrap();
    let enet = cohdl::emit::easyeda::emit_enet(&checked.world, ir);
    let net = cohdl::emit::kicad::emit_kicad_net(&checked.world, ir);
    // Same footprint symbol string, same principal value, same net names,
    // same designators — the derivations are shared, and the bytes agree.
    assert!(
        net.contains("(footprint \"board::F\"") && enet.contains("\"FootprintName\": \"board::F\"")
    );
    assert!(net.contains("(value \"D\")") && enet.contains("\"Name\": \"D\""));
    for net_name in ["N", "G"] {
        assert!(net.contains(&format!("(name \"{net_name}\")")));
        assert!(enet.contains(&format!("\"net\": \"{net_name}\"")));
    }
    for refdes in ["H1", "U1", "U2"] {
        assert!(net.contains(&format!("(ref \"{refdes}\")")));
        assert!(enet.contains(&format!("\"Designator\": \"{refdes}\"")));
    }
}

#[test]
fn emission_is_deterministic() {
    assert_eq!(emit(FIXTURE), emit(FIXTURE), "same source, same bytes");
}

#[test]
fn document_parses_as_json() {
    // python3's parser is the neutral referee (the xmllint pattern from the
    // RFC-015 schema gate: authoritative in CI, skipped locally if absent).
    use std::io::Write as _;
    use std::process::Stdio;
    let mut child = match Command::new("python3")
        .args(["-m", "json.tool"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => {
            eprintln!(
                "warning: python3 not found — JSON well-formedness not verified here (CI verifies)"
            );
            return;
        }
    };
    child
        .stdin
        .take()
        .unwrap()
        .write_all(emit(FIXTURE).as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "json.tool rejected the document:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    // Positive marker: the re-serialized document still opens with version.
    assert!(String::from_utf8_lossy(&out.stdout).contains("\"version\": \"2.0.0\""));
}

fn example_enet(dir: &str) -> String {
    let root = manifest();
    let project_dir = root.join(dir);
    let (_, project_manifest) = cohdl::project::peek_manifest(&project_dir).unwrap();
    let mut dep_names: Vec<String> = project_manifest
        .deps_raw
        .unwrap_or_default()
        .into_iter()
        .map(|(name, _, _)| name)
        .collect();
    dep_names.sort();
    if let Some(pos) = dep_names.iter().position(|name| name == "std") {
        let std = dep_names.remove(pos);
        dep_names.insert(0, std);
    }
    let deps: Vec<(String, PathBuf)> = dep_names
        .iter()
        .map(|name| (name.clone(), root.join("lib").join(name)))
        .collect();
    let proj = cohdl::project::load_project_with_deps(&project_dir, &deps).unwrap();
    let mut checked = cohdl::pipeline::check_files_in_with_deps(
        &proj.name,
        &dep_names,
        &proj.files,
        proj.top.as_deref(),
    )
    .unwrap();
    assert!(!checked.diags.has_errors());
    let _ = build_artifacts(&mut checked, &LockState::default()).expect("build");
    let ir = checked.ir.as_ref().unwrap();
    cohdl::emit::easyeda::emit_enet(&checked.world, ir)
}

#[test]
fn example_boards_are_deterministic_and_complete() {
    let pico = example_enet("examples/rpi-pico2");
    assert_eq!(pico, example_enet("examples/rpi-pico2"), "byte-stable");
    // Every instance is a component; every net has its rule row (the same
    // counts the .kicad_pcb and explorer tests pin for this example).
    assert_eq!(pico.matches("\"Unique ID\": ").count(), 52);
    assert_eq!(pico.matches("\"ruleMap\"").count(), 67);
    assert!(pico.contains("\"net\": \"GND\""));

    let sf32 = example_enet("examples/sf32-miniboard");
    assert_eq!(sf32.matches("\"Unique ID\": ").count(), 49);
}

// ---- CLI-level behavior --------------------------------------------------

fn cohdl() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cohdl"))
}

fn make_project(root: &std::path::Path) {
    let _ = std::fs::remove_dir_all(root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        "[package]\nname = \"t\"\n\n[design]\ntop = \"B\"\n",
    )
    .unwrap();
    std::fs::write(root.join("src/main.cohdl"), FIXTURE).unwrap();
}

#[test]
fn cli_enet_only_with_flag_and_zero_impact_on_everything_else() {
    let tmp = std::env::temp_dir().join(format!("cohdl-enet-zero-{}", std::process::id()));
    make_project(&tmp);
    let run = |extra: &[&str]| {
        let mut args = vec!["build", tmp.to_str().unwrap(), "--no-std"];
        args.extend_from_slice(extra);
        let out = cohdl().args(&args).output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    let enet_path = tmp.join("out/t.enet");

    run(&[]);
    assert!(!enet_path.exists(), "no flag, no enet");
    let netlist = std::fs::read_to_string(tmp.join("out/t.net")).unwrap();
    let bom = std::fs::read_to_string(tmp.join("out/t-bom.csv")).unwrap();
    let lock = std::fs::read_to_string(tmp.join("design.lock")).unwrap();

    run(&["--emit", "easyeda"]);
    assert!(enet_path.exists(), "flagged build writes the enet");
    let enet = std::fs::read_to_string(&enet_path).unwrap();
    assert!(enet.starts_with("{\n  \"version\": \"2.0.0\",\n"));
    // Zero impact: every other artifact byte-identical.
    assert_eq!(
        netlist,
        std::fs::read_to_string(tmp.join("out/t.net")).unwrap()
    );
    assert_eq!(
        bom,
        std::fs::read_to_string(tmp.join("out/t-bom.csv")).unwrap()
    );
    assert_eq!(
        lock,
        std::fs::read_to_string(tmp.join("design.lock")).unwrap()
    );

    // Dropping the flag sweeps the stale enet (it is manifest-owned).
    run(&[]);
    assert!(!enet_path.exists(), "stale enet swept without the flag");
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn cli_flag_matrix_includes_easyeda() {
    let tmp = std::env::temp_dir().join(format!("cohdl-enet-matrix-{}", std::process::id()));
    make_project(&tmp);
    // Command compatibility outranks value validity.
    let out = cohdl()
        .args([
            "check",
            tmp.to_str().unwrap(),
            "--no-std",
            "--emit",
            "easyeda",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("not valid"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    // Unknown value enumerates the full valid set.
    let out = cohdl()
        .args([
            "build",
            tmp.to_str().unwrap(),
            "--no-std",
            "--emit",
            "bogus",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("valid: easyeda, ipc2581, kicad_pcb"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The same value twice is still the duplicate error (F12.5).
    let out = cohdl()
        .args([
            "build",
            tmp.to_str().unwrap(),
            "--no-std",
            "--emit",
            "easyeda",
            "--emit",
            "easyeda",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("more than once"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    // All three formats compose in one build.
    let out = cohdl()
        .args([
            "build",
            tmp.to_str().unwrap(),
            "--no-std",
            "--emit",
            "easyeda",
            "--emit",
            "kicad_pcb",
            "--emit",
            "ipc2581",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(tmp.join("out/t.enet").exists());
    assert!(tmp.join("out/t.kicad_pcb").exists());
    assert!(tmp.join("out/t.xml").exists());
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn cli_json_names_the_enet_artifact() {
    let tmp = std::env::temp_dir().join(format!("cohdl-enet-json-{}", std::process::id()));
    make_project(&tmp);
    let out = cohdl()
        .args([
            "build",
            tmp.to_str().unwrap(),
            "--no-std",
            "--emit",
            "easyeda",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"easyeda\": "), "{stdout}");
    assert!(stdout.contains("t.enet"), "{stdout}");
    let _ = std::fs::remove_dir_all(&tmp);
}
