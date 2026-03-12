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
use cohdl_sema::resolve;
use cohdl_sema::typeck::{type_check, EXTERNAL_INSTANCE};

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
        let a_is_main = a.file_name().unwrap().to_str().unwrap().starts_with("main");
        let b_is_main = b.file_name().unwrap().to_str().unwrap().starts_with("main");
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

/// Load a fixture where non-main `.cohdl` files should be wrapped in their
/// corresponding `module <name> { ... }` blocks (matching bare `module <name>`
/// directives in main.cohdl).
fn load_fixture_source_with_file_modules(fixture_dir: &str) -> String {
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

    files.sort_by(|a, b| {
        let a_is_main = a.file_name().unwrap().to_str().unwrap().starts_with("main");
        let b_is_main = b.file_name().unwrap().to_str().unwrap().starts_with("main");
        a_is_main.cmp(&b_is_main).then_with(|| a.cmp(b))
    });

    // Detect bare `module <name>` directives in the main file.
    let main_file = files
        .iter()
        .find(|p| p.file_name().unwrap().to_str().unwrap().starts_with("main"));
    let mut mod_decl_names: Vec<String> = Vec::new();
    if let Some(main_path) = main_file {
        let main_content = std::fs::read_to_string(main_path).unwrap();
        for line in main_content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("module ") && !trimmed.contains('{') {
                if let Some(name) = trimmed.strip_prefix("module ") {
                    let name = name.trim();
                    if !name.is_empty() {
                        mod_decl_names.push(name.to_string());
                    }
                }
            }
        }
    }

    let mut source = String::new();
    for path in &files {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("cannot read {}: {}", path.display(), e));
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let is_main = stem.starts_with("main");

        if !is_main && mod_decl_names.contains(&stem.to_string()) {
            source.push_str(&format!("module {} {{\n", stem));
            source.push_str(&content);
            source.push_str("\n}\n");
        } else if is_main {
            let mut filtered = String::new();
            for line in content.lines() {
                let trimmed = line.trim();
                let is_mod_decl = trimmed.starts_with("module ")
                    && !trimmed.contains('{')
                    && mod_decl_names
                        .iter()
                        .any(|m| trimmed == format!("module {}", m));
                if !is_mod_decl {
                    filtered.push_str(line);
                    filtered.push('\n');
                }
            }
            source.push_str(&filtered);
        } else {
            source.push_str(&content);
            source.push('\n');
        }
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

/// Synthesize the bundled `std` dependency source by wrapping the std source
/// files into `module std { pub module traits { ... } pub module passive { ... } pub module footprints { ... } }`.
fn synthesize_std_source() -> String {
    let traits_src =
        std::fs::read_to_string("std/src/traits.cohdl").expect("cannot read std/src/traits.cohdl");
    let passive_src = std::fs::read_to_string("std/src/passive.cohdl")
        .expect("cannot read std/src/passive.cohdl");
    let footprints_src = std::fs::read_to_string("std/src/footprints.cohdl")
        .expect("cannot read std/src/footprints.cohdl");

    format!(
        "module std {{\npub module traits {{\n{}\n}}\npub module footprints {{\n{}\n}}\npub module passive {{\n{}\n}}\n}}\n",
        traits_src, footprints_src, passive_src,
    )
}

/// Load a fixture that declares `[dependencies] std = "0.1.0"` — prepends the
/// synthesized std source before the fixture's own source files.
fn load_fixture_source_with_std(fixture_dir: &str) -> String {
    let std_src = synthesize_std_source();
    let user_src = load_fixture_source(fixture_dir);
    format!("{}\n{}", std_src, user_src)
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
    assert_eq!(
        ir.instances.len(),
        6,
        "expected 6 instances, got {}",
        ir.instances.len()
    );

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
    let total_expected: usize = expected_rules
        .iter()
        .map(|r| counts.get(r).unwrap_or(&0))
        .sum();
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

// ── Test 5: std_dependency ───────────────────────────────────────────────────

#[test]
fn std_dependency() {
    let src = load_fixture_source_with_std("tests/fixtures/std_dependency");
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

    // ── Instance count: c1, c2, r1 = 3
    assert_eq!(
        ir.instances.len(),
        3,
        "expected 3 instances, got {}",
        ir.instances.len()
    );

    // ── Net "VCC" connects exactly 3 instance pin-refs (c1.A, c2.A, r1.A).
    let vcc_net = ir
        .nets
        .iter()
        .find(|n| n.name == "VCC")
        .expect("VCC net not found");
    let vcc_inst_pins: Vec<_> = vcc_net
        .pins
        .iter()
        .filter(|p| p.instance_id != EXTERNAL_INSTANCE)
        .collect();
    assert_eq!(
        vcc_inst_pins.len(),
        3,
        "VCC should connect exactly 3 instance pins, got {}",
        vcc_inst_pins.len()
    );

    // ── DRC: zero errors.
    let runner = DrcRunner::default();
    let drc_diags = runner.run(ir);
    let drc_errors: Vec<_> = drc_diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        drc_errors.is_empty(),
        "expected no DRC errors, got: {:?}",
        drc_errors
    );
}

// ── Test 5b: designator lock file lifecycle ──────────────────────────────────

#[test]
fn designator_lock_lifecycle() {
    let src = load_fixture_source("tests/fixtures/stm32_minimal");
    let output = run_pipeline(&src);
    assert!(output.parse_errors.is_none());
    assert!(output.sema_errors.is_empty());

    let tc = output.tc_result.as_ref().unwrap();
    let design = tc
        .designs
        .iter()
        .find(|d| d.name == "MainBoard")
        .expect("MainBoard design not found");
    let conn = build_connectivity(design, &tc.device_pins);
    let ir = &conn.ir;

    // ── Phase 1: fresh assignment ────────────────────────────────────────
    let infos = instance_infos_from_typed_design(design, ir, &tc.trait_prefixes);
    let mut db = DesignatorDb::new();
    let (assignments, errors) = db.assign(&infos);
    assert!(errors.is_empty(), "designator errors: {:?}", errors);

    // Verify prefixes are correctly applied.
    let cap_desigs: Vec<&str> = assignments
        .iter()
        .filter(|(_, d)| d.starts_with('C'))
        .map(|(_, d)| d.as_str())
        .collect();
    assert_eq!(cap_desigs.len(), 4, "expected 4 capacitor designators");

    let u_desigs: Vec<&str> = assignments
        .iter()
        .filter(|(_, d)| d.starts_with('U'))
        .map(|(_, d)| d.as_str())
        .collect();
    assert_eq!(u_desigs.len(), 1, "expected 1 IC designator");

    let j_desigs: Vec<&str> = assignments
        .iter()
        .filter(|(_, d)| d.starts_with('J'))
        .map(|(_, d)| d.as_str())
        .collect();
    assert_eq!(j_desigs.len(), 1, "expected 1 connector designator");

    // ── Phase 2: stable reassignment ─────────────────────────────────────
    let mut db2 = db.clone();
    let (assignments2, errors2) = db2.assign(&infos);
    assert!(errors2.is_empty());
    assert_eq!(assignments, assignments2, "reassignment should be stable");

    // ── Phase 3: save/load round-trip ────────────────────────────────────
    let lock_path = std::env::temp_dir().join("cohdl_e2e_designator.lock");
    let _ = std::fs::remove_file(&lock_path);
    db.save(&lock_path).unwrap();

    let mut db3 = DesignatorDb::load(&lock_path).unwrap();
    let (assignments3, errors3) = db3.assign(&infos);
    assert!(errors3.is_empty());
    assert_eq!(
        assignments, assignments3,
        "assignment after load should match"
    );

    // ── Phase 4: removal triggers tombstone ──────────────────────────────
    // Simulate removing the first capacitor instance.
    let cap_path = assignments
        .iter()
        .find(|(_, d)| *d == "C1")
        .map(|(p, _)| p.clone())
        .unwrap();
    db3.tombstone_removed(std::slice::from_ref(&cap_path));

    // C1's designator should now be in tombstones.
    assert!(db3.tombstones().contains_key(&cap_path));
    assert!(!db3.designators().contains_key(&cap_path));

    // Adding a new capacitor should skip C1 (tombstoned).
    let mut reduced_infos: Vec<_> = infos
        .iter()
        .filter(|i| i.hierarchical_path != cap_path)
        .cloned()
        .collect();
    reduced_infos.push(cohdl_sema::designator::InstanceInfo {
        hierarchical_path: "MainBoard::c_new".to_string(),
        designator_override: None,
        prefix: Some("C".to_string()),
    });
    let (assignments4, errors4) = db3.assign(&reduced_infos);
    assert!(errors4.is_empty());
    assert!(
        !assignments4.values().any(|d| d == "C1"),
        "C1 should be tombstoned and never reused; got {:?}",
        assignments4
    );
    let new_desig = &assignments4["MainBoard::c_new"];
    assert!(
        new_desig.starts_with('C'),
        "new cap should get C prefix; got {}",
        new_desig
    );

    // ── Phase 5: apply_designators populates the IR ──────────────────────
    let mut ir_clone = ir.clone();
    ir_clone.apply_designators(&assignments);
    for inst in &ir_clone.instances {
        assert!(
            inst.designator.is_some(),
            "instance {} should have designator",
            inst.name
        );
    }

    // Clean up.
    let _ = std::fs::remove_file(&lock_path);
}

// ── Test 6: stm32_core (generics + function calls) ──────────────────────────

#[test]
fn stm32_core() {
    let std_src = synthesize_std_source();
    let user_src = load_fixture_source_with_file_modules("tests/fixtures/stm32_core");
    let src = format!("{}\n{}", std_src, user_src);
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

    // ── Instance count: 1 MCU + 4 decoupling caps + 1 resistor + 1 cap = 7
    assert_eq!(
        ir.instances.len(),
        7,
        "expected 7 instances, got {}",
        ir.instances.len()
    );

    // ── Verify 4 function-expanded MLCC decoupling instances exist.
    let fn_mlcc_count = ir
        .instances
        .iter()
        .filter(|i| i.name.starts_with("__fn") && i.device == "std::passive::MLCC")
        .count();
    assert_eq!(
        fn_mlcc_count, 4,
        "expected 4 function-expanded MLCC instances, got {}",
        fn_mlcc_count
    );

    // ── Net "GND" connects MCU VSS/VSSA + c_rst + 4 decoupling caps.
    let gnd_net = ir
        .nets
        .iter()
        .find(|n| n.name == "GND")
        .expect("GND net not found");
    assert!(
        gnd_net.pins.len() >= 7,
        "GND should connect at least 7 pins, got {}",
        gnd_net.pins.len()
    );

    // ── DRC: zero errors.
    let runner = DrcRunner::default();
    let drc_diags = runner.run(ir);
    let drc_errors: Vec<_> = drc_diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        drc_errors.is_empty(),
        "expected no DRC errors, got: {:?}",
        drc_errors
    );
}

// ── Test: conol-pin (Plaud NotePin S clone) ─────────────────────────────────

#[test]
fn conol_pin() {
    let std_src = synthesize_std_source();
    let user_src = load_fixture_source_with_file_modules("tests/fixtures/conol-pin");
    let src = format!("{}\n{}", std_src, user_src);
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

    // ── Verify we have a substantial number of instances.
    // Single-SoC (ESP32-S3) design has fewer parts than dual-SoC.
    assert!(
        ir.instances.len() >= 40,
        "expected at least 40 instances, got {}",
        ir.instances.len()
    );

    // ── Verify function-expanded decoupling caps exist.
    let fn_mlcc_count = ir
        .instances
        .iter()
        .filter(|i| i.name.starts_with("__fn") && i.device == "std::passive::MLCC")
        .count();
    assert!(
        fn_mlcc_count >= 14,
        "expected at least 14 function-expanded MLCC instances, got {}",
        fn_mlcc_count
    );

    // ── Key nets exist.
    let net_names: Vec<&str> = ir.nets.iter().map(|n| n.name.as_str()).collect();
    for expected in &[
        "GND", "VDD_3V3", "VDD_1V8", "VBUS", "VSYS", "VBAT", "SDIO_CLK", "PDM_CLK", "RF_ANT",
        "BTN_REC", "BTN_MARK", "LED_DRV",
    ] {
        assert!(
            net_names.contains(expected),
            "expected net '{}' not found in {:?}",
            expected,
            net_names
        );
    }

    // ── GND net should have many pins (all grounds merged).
    let gnd_net = ir.nets.iter().find(|n| n.name == "GND").unwrap();
    assert!(
        gnd_net.pins.len() >= 30,
        "GND should connect at least 30 pins, got {}",
        gnd_net.pins.len()
    );

    // ── DRC: zero errors (conol-pin).
    let runner = DrcRunner::default();
    let drc_diags = runner.run(ir);
    let drc_errors: Vec<_> = drc_diags
        .iter()
        .filter(|d| d.level == DiagnosticLevel::Error)
        .collect();
    assert!(
        drc_errors.is_empty(),
        "expected no DRC errors, got: {:?}",
        drc_errors
    );
}
