use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper: create a temp project directory with cohdl.toml and a source file.
fn setup_project(source: &str) -> TempDir {
    let dir = TempDir::new().unwrap();

    let manifest = r#"[package]
name    = "test-board"
version = "0.1.0"

[design]
root = "src/main.cohdl"
top  = "MainBoard"
"#;

    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("cohdl.toml"), manifest).unwrap();
    fs::write(dir.path().join("src/main.cohdl"), source).unwrap();

    dir
}

fn cohdl() -> Command {
    Command::cargo_bin("cohdl").unwrap()
}

// ── fmt subcommand ──────────────────────────────────────────────────────────

#[test]
fn fmt_prints_placeholder() {
    cohdl()
        .arg("fmt")
        .assert()
        .success()
        .stdout(predicate::str::contains("formatter not yet implemented"));
}

// ── build without cohdl.toml ────────────────────────────────────────────────

#[test]
fn build_fails_without_manifest() {
    let dir = TempDir::new().unwrap();
    cohdl()
        .arg("build")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not read cohdl.toml"));
}

// ── check without cohdl.toml ───────────────────────────────────────────────

#[test]
fn check_fails_without_manifest() {
    let dir = TempDir::new().unwrap();
    cohdl()
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("could not read cohdl.toml"));
}

// ── build with parse error ──────────────────────────────────────────────────

#[test]
fn build_parse_error_exits_1() {
    let dir = setup_project("this is not valid cohdl syntax }{}{");
    cohdl()
        .args(["build", "--color", "never"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error[PARSE]"));
}

// ── check with sema error ───────────────────────────────────────────────────

#[test]
fn check_sema_error_undefined_symbol() {
    let src = r#"
        design MainBoard {
            inst c: NonExistent
        }
    "#;
    let dir = setup_project(src);
    cohdl()
        .args(["check", "--color", "never"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("undefined symbol"));
}

// ── build with valid source produces output files ───────────────────────────

#[test]
fn build_valid_source_produces_netlist() {
    let src = r#"
        trait TwoTerminal {
            pins { A: Pin, B: Pin }
        }
        device MLCC<C: Farads, V: Voltage = 10V>: impl TwoTerminal {
            pins { A: 1, B: 2 }
            spec { capacitance: C, voltage_rating: V }
        }
        design MainBoard {
            inst c1: MLCC<C: 100nF>
        }
    "#;
    let dir = setup_project(src);
    cohdl()
        .args(["build", "--color", "never"])
        .current_dir(dir.path())
        .assert()
        .success();

    // Default --out-dir is "out"
    assert!(dir.path().join("out/test-board.net").exists());
    assert!(dir.path().join("out/test-board-bom.csv").exists());
    assert!(dir.path().join("out/test-board-bom-avl.csv").exists());
}

// ── build with module declarations resolves multi-file projects ─────────────

#[test]
fn build_multi_file_with_module_decl() {
    let dir = TempDir::new().unwrap();

    let manifest = r#"[package]
name    = "test-board"
version = "0.1.0"

[design]
root = "src/main.cohdl"
top  = "MainBoard"
"#;

    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("cohdl.toml"), manifest).unwrap();

    // Module file: defines the device
    fs::write(
        dir.path().join("src/parts.cohdl"),
        r#"
        trait TwoTerminal {
            pins { A: Pin, B: Pin }
        }
        device MLCC<C: Farads, V: Voltage = 10V>: impl TwoTerminal {
            pins { A: 1, B: 2 }
            spec { capacitance: C, voltage_rating: V }
        }
        "#,
    )
    .unwrap();

    // Root file: references the module
    fs::write(
        dir.path().join("src/main.cohdl"),
        r#"
        module parts

        design MainBoard {
            inst c1: MLCC<C: 100nF>
        }
        "#,
    )
    .unwrap();

    cohdl()
        .args(["build", "--color", "never"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("out/test-board.net").exists());
}

#[test]
fn build_missing_module_file_exits_1() {
    let dir = setup_project(
        r#"
        module nonexistent

        design MainBoard {}
        "#,
    );
    cohdl()
        .args(["build", "--color", "never"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "could not read module `nonexistent`",
        ));
}

// ── build with custom out-dir ───────────────────────────────────────────────

#[test]
fn build_custom_out_dir() {
    let src = r#"
        trait TwoTerminal {
            pins { A: Pin, B: Pin }
        }
        device MLCC<C: Farads, V: Voltage = 10V>: impl TwoTerminal {
            pins { A: 1, B: 2 }
            spec { capacitance: C, voltage_rating: V }
        }
        design MainBoard {
            inst c1: MLCC<C: 100nF>
        }
    "#;
    let dir = setup_project(src);
    cohdl()
        .args(["build", "--out-dir", "build-output", "--color", "never"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("build-output/test-board.net").exists());
}

// ── build with --emit netlist only ──────────────────────────────────────────

#[test]
fn build_emit_netlist_only() {
    let src = r#"
        trait TwoTerminal {
            pins { A: Pin, B: Pin }
        }
        device MLCC<C: Farads, V: Voltage = 10V>: impl TwoTerminal {
            pins { A: 1, B: 2 }
            spec { capacitance: C, voltage_rating: V }
        }
        design MainBoard {
            inst c1: MLCC<C: 100nF>
        }
    "#;
    let dir = setup_project(src);
    cohdl()
        .args(["build", "--emit", "netlist", "--color", "never"])
        .current_dir(dir.path())
        .assert()
        .success();

    assert!(dir.path().join("out/test-board.net").exists());
    assert!(!dir.path().join("out/test-board-bom.csv").exists());
    assert!(!dir.path().join("out/test-board-bom-avl.csv").exists());
}

// ── build with missing design name ──────────────────────────────────────────

#[test]
fn build_missing_design_exits_1() {
    let src = r#"
        trait TwoTerminal {
            pins { A: Pin, B: Pin }
        }
        device MLCC<C: Farads>: impl TwoTerminal {
            pins { A: 1, B: 2 }
            spec { capacitance: C }
        }
        design OtherBoard {
            inst c1: MLCC<C: 100nF>
        }
    "#;
    let dir = setup_project(src);
    cohdl()
        .args(["build", "--color", "never"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("design `MainBoard` not found"));
}

// ── --design flag overrides top ─────────────────────────────────────────────

#[test]
fn build_design_override() {
    let src = r#"
        trait TwoTerminal {
            pins { A: Pin, B: Pin }
        }
        device MLCC<C: Farads, V: Voltage = 10V>: impl TwoTerminal {
            pins { A: 1, B: 2 }
            spec { capacitance: C, voltage_rating: V }
        }
        design AltBoard {
            inst c1: MLCC<C: 100nF>
        }
    "#;
    let dir = setup_project(src);
    cohdl()
        .args(["build", "--design", "AltBoard", "--color", "never"])
        .current_dir(dir.path())
        .assert()
        .success();
}

// ── color flag values accepted ──────────────────────────────────────────────

#[test]
fn color_flag_always() {
    cohdl()
        .args(["--color", "always", "fmt"])
        .assert()
        .success();
}

#[test]
fn color_flag_never() {
    cohdl().args(["--color", "never", "fmt"]).assert().success();
}

// ── init subcommand ─────────────────────────────────────────────────────────

#[test]
fn init_creates_project_files() {
    let dir = TempDir::new().unwrap();
    cohdl()
        .args(["init", "my-board"])
        .current_dir(dir.path())
        .assert()
        .success()
        .stderr(predicate::str::contains("Created").and(predicate::str::contains("my-board")));

    let manifest = fs::read_to_string(dir.path().join("cohdl.toml")).unwrap();
    assert!(manifest.contains("my-board"));
    assert!(manifest.contains("[package]"));
    assert!(manifest.contains("[design]"));
    assert!(dir.path().join("src/main.cohdl").exists());
}

#[test]
fn init_defaults_name_to_dir() {
    let dir = TempDir::new().unwrap();
    let project_dir = dir.path().join("cool-project");
    fs::create_dir_all(&project_dir).unwrap();

    cohdl()
        .arg("init")
        .current_dir(&project_dir)
        .assert()
        .success()
        .stderr(predicate::str::contains("cool-project"));

    let manifest = fs::read_to_string(project_dir.join("cohdl.toml")).unwrap();
    assert!(manifest.contains("cool-project"));
}

#[test]
fn init_fails_if_manifest_exists() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("cohdl.toml"), "existing").unwrap();

    cohdl()
        .args(["init", "test"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("cohdl.toml already exists"));
}

#[test]
fn init_then_check_works() {
    let dir = TempDir::new().unwrap();
    cohdl()
        .args(["init", "test-board"])
        .current_dir(dir.path())
        .assert()
        .success();

    // The initialized project should be valid enough for `check` to succeed
    cohdl()
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .success();
}

// ── diagnostic output contains source span ──────────────────────────────────

#[test]
fn diagnostic_shows_source_location() {
    let src = r#"
        design MainBoard {
            inst c: Bogus
        }
    "#;
    let dir = setup_project(src);
    cohdl()
        .args(["check", "--color", "never"])
        .current_dir(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("-->").and(predicate::str::contains("src/main.cohdl")));
}
