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
