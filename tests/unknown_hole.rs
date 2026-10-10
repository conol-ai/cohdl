#[path = "common/unknown_hole.rs"]
mod fixture;

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};

#[test]
fn unknown_hole_plating_is_logical_geometry_and_typed_physical_error() {
    let mut c = check_files_in(
        "board",
        &[("src/main.cohdl".into(), fixture::SOURCE.into())],
        None,
    )
    .unwrap();
    assert!(
        build_artifacts(&mut c, &LockState::default()).is_some(),
        "{}",
        c.diags.render(&c.sm)
    );
    let hole = &c.world.footprints.values().next().unwrap().mount_holes[0];
    assert_eq!(hole.plating.name(), "source_unspecified");
    let ir = c.ir.as_ref().unwrap();
    for error in [
        cohdl::emit::kicad_mod::emit_kicad_mods(&c.world, ir).unwrap_err(),
        cohdl::emit::ipc2581::emit_ipc2581(&c.world, ir, "board").unwrap_err(),
        cohdl::emit::kicad_pcb::emit_kicad_pcb(&c.world, ir, "board").unwrap_err(),
    ] {
        assert!(error
            .to_string()
            .contains("source-unspecified hole plating"));
    }
    let formatted = cohdl::fmt::format_source("src/main.cohdl", fixture::SOURCE).unwrap();
    assert!(formatted.contains("mount_hole 1: source_unspecified"));
}

#[test]
fn actual_cli_physical_export_fails_before_writes_and_logical_mode_succeeds() {
    let dir = std::env::temp_dir().join(format!("cohdl-unknown-hole-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("cohdl.toml"),
        "[package]\nname = \"unknown-hole\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/main.cohdl"), fixture::SOURCE).unwrap();
    let run = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_cohdl"))
            .args(args)
            .arg(&dir)
            .output()
            .unwrap()
    };
    let physical = run(&["build", "--no-std", "--json"]);
    assert!(!physical.status.success());
    assert!(String::from_utf8_lossy(&physical.stderr).contains("source-unspecified hole plating"));
    assert!(!dir.join("out").exists());
    assert!(!dir.join("design.lock").exists());
    let logical = run(&["build", "--no-std", "--json", "--logical-only"]);
    assert!(
        logical.status.success(),
        "{}",
        String::from_utf8_lossy(&logical.stderr)
    );
    assert!(dir.join("out/unknown-hole.net").is_file());
    assert!(dir.join("out/unknown-hole-bom.csv").is_file());
    assert!(!dir.join("out/footprints").exists());
    assert!(!dir.join("out/unknown-hole.xml").exists());
    let before = std::fs::read(dir.join("out/.cohdl-manifest")).unwrap();
    let net_before = std::fs::read(dir.join("out/unknown-hole.net")).unwrap();
    let bom_before = std::fs::read(dir.join("out/unknown-hole-bom.csv")).unwrap();
    let lock_before = std::fs::read(dir.join("design.lock")).unwrap();
    assert!(!run(&["build", "--no-std"]).status.success());
    assert_eq!(
        net_before,
        std::fs::read(dir.join("out/unknown-hole.net")).unwrap()
    );
    assert_eq!(
        bom_before,
        std::fs::read(dir.join("out/unknown-hole-bom.csv")).unwrap()
    );
    assert_eq!(lock_before, std::fs::read(dir.join("design.lock")).unwrap());
    assert!(!dir.join("out/footprints").exists());
    for format in ["ipc2581", "kicad_pcb", "easyeda"] {
        assert!(
            !run(&["build", "--no-std", "--logical-only", "--emit", format])
                .status
                .success()
        );
        assert!(!run(&["build", "--no-std", "--emit", format])
            .status
            .success());
        assert_eq!(
            net_before,
            std::fs::read(dir.join("out/unknown-hole.net")).unwrap()
        );
        assert_eq!(lock_before, std::fs::read(dir.join("design.lock")).unwrap());
    }
    for command in ["check", "fmt", "update", "lsp"] {
        assert!(!run(&[command, "--logical-only"]).status.success());
    }
    assert_eq!(
        before,
        std::fs::read(dir.join("out/.cohdl-manifest")).unwrap()
    );
    std::fs::write(
        dir.join("src/main.cohdl"),
        fixture::SOURCE.replace("source_unspecified at", "non_plated at"),
    )
    .unwrap();
    let known = run(&["build", "--no-std", "--emit", "ipc2581"]);
    assert!(
        known.status.success(),
        "{}",
        String::from_utf8_lossy(&known.stderr)
    );
    assert!(dir.join("out/footprints").is_dir());
    assert!(dir.join("out/unknown-hole.xml").is_file());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn only_referenced_primary_and_alternate_holes_block_physical_export() {
    let known = fixture::SOURCE.replace("source_unspecified at", "non_plated at");
    let unused = format!("{known}\npub footprint Unknown {{ pad 1: SurfaceCopper at (0mm,0mm) mount_hole 1: source_unspecified at (0mm,0mm) diameter 2mm }}\n");
    let alternate = unused.replace(
        "footprint: Screw }",
        "footprint: Screw } alt { mpn: \"ALT\", mfr: \"Maker\", footprint: Unknown }",
    );
    for (source, blocked) in [(unused, false), (alternate, true)] {
        let mut c = check_files_in("board", &[("src/main.cohdl".into(), source)], None).unwrap();
        assert!(
            build_artifacts(&mut c, &LockState::default()).is_some(),
            "{}",
            c.diags.render(&c.sm)
        );
        let ir = c.ir.as_ref().unwrap();
        assert_eq!(
            cohdl::emit::kicad_mod::emit_kicad_mods(&c.world, ir).is_err(),
            blocked
        );
        assert_eq!(
            cohdl::emit::ipc2581::emit_ipc2581(&c.world, ir, "board").is_err(),
            blocked
        );
        assert_eq!(
            cohdl::emit::kicad_pcb::emit_kicad_pcb(&c.world, ir, "board").is_err(),
            blocked
        );
    }
}

#[test]
fn unknown_holes_keep_normal_geometry_validation() {
    for bad in [
        fixture::SOURCE.replace("diameter 2mm", "diameter 0mm"),
        fixture::SOURCE.replace("diameter 2mm", "diameter 2V"),
        fixture::SOURCE.replace("diameter 2mm", "diameter -2mm"),
        fixture::SOURCE.replace("source_unspecified at", "unknown at"),
    ] {
        let c = check_files_in("board", &[("src/main.cohdl".into(), bad)], None).unwrap();
        assert!(c.diags.has_errors());
    }
}

#[test]
fn logical_mode_removes_owned_physical_outputs_but_keeps_layout() {
    let dir = std::env::temp_dir().join(format!("cohdl-logical-owned-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("cohdl.toml"),
        "[package]\nname = \"logical-owned\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    let known = fixture::SOURCE
        .replace("source_unspecified at", "non_plated at")
        .replace(
            "nc: screw.CONTACT",
            "#[ground(primary)] net GND: screw.CONTACT layout { place screw at (0mm,0mm) }",
        );
    std::fs::write(dir.join("src/main.cohdl"), &known).unwrap();
    let run = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_cohdl"))
            .args(args)
            .arg(&dir)
            .output()
            .unwrap()
    };
    let physical = run(&[
        "build",
        "--no-std",
        "--emit",
        "ipc2581",
        "--emit",
        "kicad_pcb",
    ]);
    assert!(
        physical.status.success(),
        "{}",
        String::from_utf8_lossy(&physical.stderr)
    );
    assert!(dir.join("out/ground_nets.csv").is_file());
    std::fs::write(
        dir.join("src/main.cohdl"),
        known.replace("non_plated at", "source_unspecified at"),
    )
    .unwrap();
    let logical = run(&["build", "--no-std", "--logical-only", "--json"]);
    assert!(
        logical.status.success(),
        "{}",
        String::from_utf8_lossy(&logical.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&logical.stdout).unwrap();
    assert!(json["build"]["quilter"].is_null(), "{json}");
    assert!(dir.join("out/logical-owned-layout.json").is_file());
    assert!(!dir.join("out/ground_nets.csv").exists());
    assert!(!dir.join("out/logical-owned.xml").exists());
    assert!(!dir.join("out/logical-owned.kicad_pcb").exists());
    assert!(!dir
        .join("out/footprints/logical_owned-Screw.kicad_mod")
        .exists());
    std::fs::remove_dir_all(dir).unwrap();
}
