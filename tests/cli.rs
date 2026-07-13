//! CLI-level contracts that only exist at the binary boundary: artifact
//! lifecycle on disk (RFC-013 layout.json cleanup) and diagnostic-rendering
//! parity between plain `check`, plain `build`, and `--json` (RFC-010).

use std::path::Path;
use std::process::Command;

fn cohdl() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cohdl"))
}

/// Create a one-design project directory under a fresh temp dir.
fn make_project(root: &Path, main_src: &str) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        "[package]\nname = \"t\"\n[design]\ntop = \"B\"\n",
    )
    .unwrap();
    std::fs::write(root.join("src/main.cohdl"), main_src).unwrap();
}

const WITH_LAYOUT: &str = r#"
pub trait TwoTerminal { pins { required A: pin required B: pin } }
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
impl TwoTerminal for Res {}
pub part R1: Res { primary { mfr: "m", mpn: "n", footprint: "fp" } }
design B {
    inst r1: R1
    inst r2: R1
    net X: r1.A, r2.A
    net Y: r1.B, r2.B
    layout { diff_pair(X, Y) }
}
"#;

// RFC-013 (review F5): removing layout metadata from the source must remove
// the on-disk layout artifact on the next build — a stale partner-consumed
// constraints file is unsafe.
#[test]
fn stale_layout_artifact_is_removed() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-f5-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    make_project(&tmp, WITH_LAYOUT);

    let out = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let layout_path = tmp.join("out/t-layout.json");
    assert!(layout_path.exists(), "first build emits layout.json");

    // Remove the layout block; rebuild.
    let without = WITH_LAYOUT.replace("layout { diff_pair(X, Y) }", "");
    std::fs::write(tmp.join("src/main.cohdl"), without).unwrap();
    let out = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !layout_path.exists(),
        "stale layout.json must be removed when the source no longer has layout metadata"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

const WITH_D003: &str = r#"
pub device MCU { pins { required TX: 1 [output], required GND: 2 [power_in] } }
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
pub part MCU_P: MCU { primary { mfr: "m", mpn: "n", footprint: "fp" } }
pub part R1: Res { primary { mfr: "m", mpn: "r", footprint: "fp" } }
design B {
    inst mcu: MCU_P
    inst r: Res
    net LONELY: mcu.TX
    net GND: mcu.GND, r.A, r.B
}
"#;

// RFC-010 (review F8): a successful plain `build` renders its warnings exactly
// as `check` and `--json` do — success must not hide D003.
#[test]
fn successful_plain_build_shows_warnings() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-f8-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    make_project(&tmp, WITH_D003);

    let out = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "warnings-only build succeeds:\n{}",
        stderr
    );
    assert!(
        stderr.contains("D003"),
        "plain build must render the D003 warning:\n{}",
        stderr
    );

    // And `--json` reports the same diagnostic on stdout.
    let out = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success());
    assert!(stdout.contains("\"code\": \"D003\""), "{}", stdout);
    assert!(stdout.contains("\"verdict\": \"pass\""), "{}", stdout);
    let _ = std::fs::remove_dir_all(&tmp);
}

// RFC-010/RFC-011 (review F9, documented contract): invocation-level failures
// (design selection, nothing to build) are E000-class — exit 2, prose on
// stderr, and NO JSON document on stdout. Machine consumers distinguish them
// by exit code.
#[test]
fn invocation_failures_are_exit_2_with_no_json() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-f9-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    // Two designs, no [design] top -> selection failure.
    std::fs::create_dir_all(tmp.join("src")).unwrap();
    std::fs::write(tmp.join("cohdl.toml"), "[package]\nname = \"t\"\n").unwrap();
    std::fs::write(
        tmp.join("src/main.cohdl"),
        "pub device D { pins { A: 1 [passive] } }\ndesign A1 { inst d: D\nnet N: d.A }\ndesign A2 { inst d: D\nnet N: d.A }",
    )
    .unwrap();

    let out = cohdl()
        .args(["check", tmp.to_str().unwrap(), "--no-std", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "invocation failure is exit 2");
    assert!(out.stdout.is_empty(), "no JSON document on stdout");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("designs"),
        "prose explanation on stderr"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

// The std library and examples stay canonical (fmt --check as a CI gate).
#[test]
fn fmt_check_gate_std_and_examples() {
    for dir in ["std", "examples/sensor-node/src", "examples/rpi-pico2/src"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(dir);
        let out = cohdl()
            .args(["fmt", path.to_str().unwrap(), "--check"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "`{}` is not canonical:\n{}",
            dir,
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

// Review-2: invocation failures must never discard collected source
// diagnostics — they render to stderr before the exit-2 error.
#[test]
fn selection_failure_preserves_collected_diagnostics() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-sel-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(tmp.join("src")).unwrap();
    std::fs::write(tmp.join("cohdl.toml"), "[package]\nname = \"t\"\n").unwrap();
    // A lex error (10kΩ) AND two designs with no top: the E101 must not be
    // swallowed by the selection failure.
    std::fs::write(
        tmp.join("src/main.cohdl"),
        "pub device D { spec { r: 10kΩ } pins { A: 1 [passive] } }\ndesign A1 { inst d: D\nnet N: d.A }\ndesign A2 { inst d: D\nnet N: d.A }",
    )
    .unwrap();
    for json in [false, true] {
        let mut args = vec!["check", tmp.to_str().unwrap(), "--no-std"];
        if json {
            args.push("--json");
        }
        let out = cohdl().args(&args).output().unwrap();
        assert_eq!(out.status.code(), Some(2), "selection failure is exit 2");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("E101"),
            "collected diagnostics must render before the invocation error (json={}):\n{}",
            json,
            stderr
        );
        assert!(stderr.contains("designs"), "{}", stderr);
        assert!(out.stdout.is_empty(), "no JSON document on exit 2");
    }
    let _ = std::fs::remove_dir_all(&tmp);
}

// Review-2: `build --json` on a declarations-only project (nothing to build)
// is also the exit-2/no-JSON class — previously untested.
#[test]
fn nothing_to_build_is_exit_2_with_no_json() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-ntb-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(tmp.join("src")).unwrap();
    std::fs::write(tmp.join("cohdl.toml"), "[package]\nname = \"t\"\n").unwrap();
    std::fs::write(
        tmp.join("src/main.cohdl"),
        "pub device D { pins { A: 1 [passive] } }",
    )
    .unwrap();
    let out = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std", "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("nothing to build"));
    let _ = std::fs::remove_dir_all(&tmp);
}

// Review-2: command-specific flags are validated.
#[test]
fn command_specific_flags_are_validated() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ex = root.join("examples/sensor-node");
    let out = cohdl()
        .args(["check", ex.to_str().unwrap(), "--check"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "--check rejected on check");
    assert!(String::from_utf8_lossy(&out.stderr).contains("fmt"));

    let out = cohdl()
        .args(["fmt", root.join("std").to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "--json rejected on fmt");
}

// Review-2: `10kΩ` end-to-end — the lexer emits ONE targeted E101; the
// documented recovery fallout is the generic follow-on where the consumed
// token leaves a hole (here E401: the generic argument list lost its entry).
// This pins the full-pipeline diagnostic set so recovery-policy changes are
// deliberate.
#[test]
fn unicode_omega_full_pipeline_recovery() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-omega-{}.cohdl", std::process::id()));
    std::fs::write(
        &tmp,
        "pub device R<X: Resistance> {\n    pins { A: 1 [passive] }\n    spec { r: X }\n}\ndesign B {\n    inst r: R<10kΩ>\n    net N: r.A\n}",
    )
    .unwrap();
    let out = cohdl()
        .args(["check", tmp.to_str().unwrap(), "--no-std", "--json"])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout.matches("\"code\": \"E101\"").count(),
        1,
        "exactly one targeted E101:\n{}",
        stdout
    );
    assert!(
        !stdout.contains("\"code\": \"E103\"") && !stdout.contains("\"code\": \"E107\""),
        "no unknown-suffix/stray-glyph cascade:\n{}",
        stdout
    );
    assert!(
        stdout.contains("10kohm"),
        "rewrite help present:\n{}",
        stdout
    );
    // Documented recovery follow-on (generic arity, from the consumed token).
    assert!(stdout.contains("\"code\": \"E401\""), "{}", stdout);
    let _ = std::fs::remove_file(&tmp);
}
