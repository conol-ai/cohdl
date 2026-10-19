//! Independent oracles: Abracon Rev A PDF p4 and RP2350 guide Figures 3-4.
use cohdl::ast::PinRole;
use cohdl::pipeline::{build_artifacts, check_files_in_with_deps};

fn project() -> cohdl::project::Project {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    cohdl::project::load_project_with_deps(
        &root.join("lib/passive"),
        &[("std".into(), root.join("lib/std"))],
    )
    .unwrap()
}

#[test]
fn winding_mark_has_distinct_physical_terminal_identity() {
    let p = project();
    let c = check_files_in_with_deps("passive", &["std".into()], &p.files, None).unwrap();
    assert!(!c.diags.has_errors(), "{}", c.diags.render(&c.sm));
    let pins = c.world.devices["passive::AOTA_B201610S3R3"].pins_for(None);
    let actual = pins
        .iter()
        .map(|p| format!("{}={}", p.name.name, p.numbers[0].text))
        .collect::<Vec<_>>();
    assert_eq!(actual, ["DOT=1", "UNDOTTED=2"]);
    assert!(pins.iter().all(|p| p.role_or_default() == PinRole::Passive));
}

#[test]
fn land_matches_abracon_instead_of_same_body_sunlord_land() {
    let p = project();
    let c = check_files_in_with_deps("passive", &["std".into()], &p.files, None).unwrap();
    let fp = &c.world.footprints["passive::FP_AOTA_B201610S3R3"];
    let actual = fp
        .pads
        .iter()
        .map(|p| format!("{}={},{}", p.number.text, p.x.text, p.y.text))
        .collect::<Vec<_>>();
    assert_eq!(actual, ["1=1mm,0mm", "2=-1mm,0mm"]);
    let pad = &c.world.pads["passive::P_AOTA_B201610S3R3"];
    assert_eq!(
        pad.size.iter().map(|x| x.text.as_str()).collect::<Vec<_>>(),
        ["1mm", "1.6mm"]
    );
    let mut diags = cohdl::diag::Diagnostics::new();
    cohdl::check::footprints::check_pad_consistency(&c.world, &mut diags);
    assert!(!diags.has_errors(), "{}", diags.render(&c.sm));
}

#[test]
fn exact_part_keeps_dot_to_dvdd_in_emitted_netlist() {
    let mut p = project();
    p.files.push(("src/aota_smoke.cohdl".into(), "design AotaSmoke {\ninst l: passive::IND_3U3_AOTA_B201610S3R3_101_T\nnet DVDD: l.DOT\nnet VREG_LX: l.UNDOTTED\n}\n".into()));
    let mut c = check_files_in_with_deps("passive", &["std".into()], &p.files, None).unwrap();
    let out = build_artifacts(&mut c, &cohdl::lock::LockState::default());
    assert!(!c.diags.has_errors(), "{}", c.diags.render(&c.sm));
    let out = out.unwrap();
    assert!(out.bom.contains("AOTA-B201610S3R3-101-T"));
    assert!(out.netlist.contains("(value \"3.3uH\")"));
    assert!(out
        .netlist
        .contains("(name \"DVDD\")\n      (node (ref \"L1\") (pin \"1\"))"));
    assert!(out
        .netlist
        .contains("(name \"VREG_LX\")\n      (node (ref \"L1\") (pin \"2\"))"));
    let symbol = "passive::IND_3U3_AOTA_B201610S3R3_101_T";
    assert!(c.world.parts[symbol].alts.is_empty());
    assert!(c.world.docs[symbol].contains(&"docs/abracon-aota-application.md".into()));
    assert!(c.world.intents[symbol].contains("DOT connects to DVDD"));
}
