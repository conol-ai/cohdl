//! End-to-end integration tests for the cohdl compiler pipeline.
//!
//! Each test reads `.cohdl` source files from a fixture directory, runs them
//! through the full pipeline (parse → resolve → type-check → connectivity → DRC),
//! and asserts the expected outcomes.

use std::collections::HashMap;
use std::path::Path;

use cohdl_drc::{DiagnosticLevel, DrcRunner};
use cohdl_parser::parse_source_file;
use cohdl_sema::connectivity::build_connectivity;
use cohdl_sema::designator::{instance_infos_from_typed_design, DesignatorDb};
use cohdl_sema::typeck::{type_check, EXTERNAL_INSTANCE};
use cohdl_sema::resolve;

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Read all `.cohdl` files under `fixture_dir/src/` and concatenate them into a
/// single source string.  Files are sorted so that the main design file is
/// loaded last (files whose name starts with `main` are appended at the end).
fn load_fixture_source(fixture_dir: &str) -> String {
    let src_dir = Path::new(fixture_dir).join("src");
    let mut files: Vec<_> = std::fs::read_dir(&src_dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {}", src_dir.display(), e))
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("cohdl") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    // Sort: non-main files first, main last.
    files.sort_by(|a, b| {
        let a_is_main = a
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("main");
        let b_is_main = b
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("main");
        a_is_main.cmp(&b_is_main).then_with(|| a.cmp(b))
    });

    let mut source = String::new();
    for path in &files {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("cannot read {}: {}", path.display(), e));
        source.push_str(&content);
        source.push('\n');
    }
    source
}

/// Run the full pipeline (parse → resolve → type-check) and return everything
/// needed for downstream assertions.
struct PipelineOutput {
    parse_errors: Option<Vec<cohdl_parser::ParseError>>,
    sema_errors: Vec<cohdl_sema::SemaError>,
    tc_result: Option<cohdl_sema::typeck::TypeCheckResult>,
}

fn run_pipeline(src: &str) -> PipelineOutput {
    let source_file = match parse_source_file(src) {
        Ok(sf) => sf,
        Err(errors) => {
            return PipelineOutput {
                parse_errors: Some(errors),
                sema_errors: Vec::new(),
                tc_result: None,
            };
        }
    };

    let resolved = resolve(&source_file);
    let mut sema_errors = resolved.errors.clone();

    let tc_result = type_check(&source_file, &resolved);
    sema_errors.extend(tc_result.errors.clone());

    PipelineOutput {
        parse_errors: None,
        sema_errors,
        tc_result: Some(tc_result),
    }
}

// ── Test 1: stm32_minimal ────────────────────────────────────────────────────

#[test]
fn stm32_minimal() {
    let src = load_fixture_source("tests/fixtures/stm32_minimal");
    let output = run_pipeline(&src);

    // No parse errors.
    assert!(
        output.parse_errors.is_none(),
        "unexpected parse errors: {:?}",
        output.parse_errors
    );

    // No sema errors.
    assert!(
        output.sema_errors.is_empty(),
        "unexpected sema errors: {:?}",
        output.sema_errors
    );

    let tc = output.tc_result.as_ref().unwrap();

    // Find the MainBoard design.
    let design = tc
        .designs
        .iter()
        .find(|d| d.name == "MainBoard")
        .expect("MainBoard design not found");

    // Build connectivity IR.
    let conn = build_connectivity(design, &tc.device_pins);
    assert!(
        conn.errors.is_empty(),
        "connectivity errors: {:?}",
        conn.errors
    );
    let ir = &conn.ir;

    // ── Instance count: 1 MCU + 1 connector + 4 caps = 6
    assert_eq!(ir.instances.len(), 6, "expected 6 instances, got {}", ir.instances.len());

    // ── Net "USB_DM" connects exactly 2 instance pin-refs.
    let usb_dm_net = ir
        .nets
        .iter()
        .find(|n| n.name == "USB_DM")
        .expect("USB_DM net not found");
    let usb_dm_inst_pins: Vec<_> = usb_dm_net
        .pins
        .iter()
        .filter(|p| p.instance_id != EXTERNAL_INSTANCE)
        .collect();
    assert_eq!(
        usb_dm_inst_pins.len(),
        2,
        "USB_DM should connect exactly 2 instance pins, got {}",
        usb_dm_inst_pins.len()
    );

    // ── Net "GND" connects at minimum 6 instance pin-refs.
    let gnd_net = ir
        .nets
        .iter()
        .find(|n| n.name == "GND")
        .expect("GND net not found");
    let gnd_inst_pins: Vec<_> = gnd_net
        .pins
        .iter()
        .filter(|p| p.instance_id != EXTERNAL_INSTANCE)
        .collect();
    assert!(
        gnd_inst_pins.len() >= 6,
        "GND should connect at least 6 instance pins, got {}",
        gnd_inst_pins.len()
    );

    // ── DRC: zero errors, zero warnings.
    let runner = DrcRunner::default();
    let drc_diags = runner.run(ir);
    let drc_errors: Vec<_> = drc_diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    let drc_warnings: Vec<_> = drc_diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Warning)
        .collect();
    assert!(
        drc_errors.is_empty(),
        "expected no DRC errors, got: {:?}",
        drc_errors
    );
    assert!(
        drc_warnings.is_empty(),
        "expected no DRC warnings, got: {:?}",
        drc_warnings
    );

    // ── Designators: U1, J1, C1–C4.
    let infos = instance_infos_from_typed_design(design, ir, &tc.trait_prefixes);
    let mut db = DesignatorDb::new();
    let (designators, desig_errors) = db.assign(&infos);
    assert!(
        desig_errors.is_empty(),
        "designator errors: {:?}",
        desig_errors
    );

    // Collect assigned designators for verification.
    let desig_values: Vec<&str> = designators.values().map(|s| s.as_str()).collect();

    assert!(
        desig_values.contains(&"U1"),
        "expected U1 designator; got {:?}",
        desig_values
    );
    assert!(
        desig_values.contains(&"J1"),
        "expected J1 designator; got {:?}",
        desig_values
    );
    for c in &["C1", "C2", "C3", "C4"] {
        assert!(
            desig_values.contains(c),
            "expected {} designator; got {:?}",
            c,
            desig_values
        );
    }
}

// ── Test 2: drc_violations ───────────────────────────────────────────────────

#[test]
fn drc_violations() {
    let src = load_fixture_source("tests/fixtures/drc_violations");
    let output = run_pipeline(&src);

    // No parse errors.
    assert!(
        output.parse_errors.is_none(),
        "unexpected parse errors: {:?}",
        output.parse_errors
    );

    let tc = output.tc_result.as_ref().unwrap();

    // Find the DrcTest design.
    let design = tc
        .designs
        .iter()
        .find(|d| d.name == "DrcTest")
        .expect("DrcTest design not found");

    // Build connectivity IR.
    let conn = build_connectivity(design, &tc.device_pins);
    let ir = &conn.ir;

    // Run DRC.
    let runner = DrcRunner::default();
    let drc_diags = runner.run(ir);

    // Count occurrences of each diagnostic rule.
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for diag in &drc_diags {
        *counts.entry(diag.rule_id.as_str()).or_insert(0) += 1;
    }

    // E001 voltage_exceed — exactly 1
    assert_eq!(
        counts.get("E001").copied().unwrap_or(0),
        1,
        "expected exactly 1 E001, got {:?}; all diags: {:?}",
        counts.get("E001"),
        drc_diags
    );

    // E002 polarity_mismatch — exactly 1
    assert_eq!(
        counts.get("E002").copied().unwrap_or(0),
        1,
        "expected exactly 1 E002, got {:?}; all diags: {:?}",
        counts.get("E002"),
        drc_diags
    );

    // W001 unconnected_pin — exactly 1
    assert_eq!(
        counts.get("W001").copied().unwrap_or(0),
        1,
        "expected exactly 1 W001, got {:?}; all diags: {:?}",
        counts.get("W001"),
        drc_diags
    );

    // W002 floating_net — exactly 1
    assert_eq!(
        counts.get("W002").copied().unwrap_or(0),
        1,
        "expected exactly 1 W002, got {:?}; all diags: {:?}",
        counts.get("W002"),
        drc_diags
    );

    // No other diagnostics.
    let expected_rules: Vec<&str> = vec!["E001", "E002", "W001", "W002"];
    let total_expected: usize = expected_rules.iter().map(|r| counts.get(r).unwrap_or(&0)).sum();
    assert_eq!(
        drc_diags.len(),
        total_expected,
        "unexpected extra diagnostics; all diags: {:?}",
        drc_diags
    );
}

// ── Test 3: generic_passives ─────────────────────────────────────────────────

#[test]
fn generic_passives() {
    let src = load_fixture_source("tests/fixtures/generic_passives");
    let output = run_pipeline(&src);

    // No parse errors.
    assert!(
        output.parse_errors.is_none(),
        "unexpected parse errors: {:?}",
        output.parse_errors
    );

    // No sema errors.
    assert!(
        output.sema_errors.is_empty(),
        "unexpected sema errors: {:?}",
        output.sema_errors
    );

    let tc = output.tc_result.as_ref().unwrap();

    // Find the TestBoard design.
    let design = tc
        .designs
        .iter()
        .find(|d| d.name == "TestBoard")
        .expect("TestBoard design not found");

    // Build connectivity IR.
    let conn = build_connectivity(design, &tc.device_pins);
    assert!(
        conn.errors.is_empty(),
        "connectivity errors: {:?}",
        conn.errors
    );
    let ir = &conn.ir;

    // ── Instance count: c1, r1, r_top, r_bot, c_dec = 5.
    assert_eq!(
        ir.instances.len(),
        5,
        "expected 5 instances, got {}",
        ir.instances.len()
    );

    // ── Net "VOUT" connects exactly 3 instance pin-refs.
    let vout_net = ir
        .nets
        .iter()
        .find(|n| n.name == "VOUT")
        .expect("VOUT net not found");
    let vout_inst_pins: Vec<_> = vout_net
        .pins
        .iter()
        .filter(|p| p.instance_id != EXTERNAL_INSTANCE)
        .collect();
    assert_eq!(
        vout_inst_pins.len(),
        3,
        "VOUT should connect exactly 3 instance pins, got {}",
        vout_inst_pins.len()
    );

    // ── DRC: zero errors, zero warnings.
    let runner = DrcRunner::default();
    let drc_diags = runner.run(ir);
    assert!(
        drc_diags.is_empty(),
        "expected no DRC diagnostics, got: {:?}",
        drc_diags
    );
}

// ── Test 4: multi_module ─────────────────────────────────────────────────────

#[test]
fn multi_module() {
    let src = load_fixture_source("tests/fixtures/multi_module");
    let output = run_pipeline(&src);

    // No parse errors.
    assert!(
        output.parse_errors.is_none(),
        "unexpected parse errors: {:?}",
        output.parse_errors
    );

    // ── Exactly 2 sema errors, both about private access.
    let private_errors: Vec<_> = output
        .sema_errors
        .iter()
        .filter(|e| e.message.contains("private"))
        .collect();
    assert_eq!(
        private_errors.len(),
        2,
        "expected exactly 2 private_access sema errors, got {} (all sema errors: {:?})",
        private_errors.len(),
        output.sema_errors
    );

    // Verify the two errors reference the correct private items.
    let msgs: Vec<&str> = private_errors.iter().map(|e| e.message.as_str()).collect();
    assert!(
        msgs.iter().any(|m| m.contains("_calibrate")),
        "expected a private_access error for _calibrate; got: {:?}",
        msgs
    );
    assert!(
        msgs.iter().any(|m| m.contains("_internal_helper")),
        "expected a private_access error for _internal_helper; got: {:?}",
        msgs
    );

    // ── No DRC errors on the valid portion of the design.
    let tc = output.tc_result.as_ref().unwrap();
    let design = tc
        .designs
        .iter()
        .find(|d| d.name == "Board")
        .expect("Board design not found");

    let conn = build_connectivity(design, &tc.device_pins);
    let ir = &conn.ir;

    let runner = DrcRunner::default();
    let drc_diags = runner.run(ir);
    let drc_errors: Vec<_> = drc_diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        drc_errors.is_empty(),
        "expected no DRC errors on valid portion, got: {:?}",
        drc_errors
    );
}
