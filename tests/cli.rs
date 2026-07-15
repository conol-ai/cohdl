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
pub footprint TFP {}
pub part R1: Res { primary { mfr: "m", mpn: "n", footprint: TFP } }
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
pub footprint TFP {}
pub part MCU_P: MCU { primary { mfr: "m", mpn: "n", footprint: TFP } }
pub part R1: Res { primary { mfr: "m", mpn: "r", footprint: TFP } }
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

// Review-2/Review-3 (R3): command-specific flags are validated — the full
// matrix, not just the two flags the first fix covered.
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

    // R3: the rest of the matrix. Every invalid (command, flag) pair is
    // exit 2 with a message naming the flag.
    let cases: &[(&[&str], &str)] = &[
        (&["check", "--out-dir", "x"], "--out-dir"),
        (&["fmt", "--design", "B"], "--design"),
        (&["fmt", "--std", "std"], "--std"),
        (&["fmt", "--no-std"], "--no-std"),
        (&["fmt", "--out-dir", "x"], "--out-dir"),
        (&["lsp", "--json"], "lsp"),
        (&["lsp", "--design", "B"], "lsp"),
        (&["lsp", "some-path"], "lsp"),
        (&["check", "--std", "std", "--no-std"], "mutually exclusive"),
        (&["build", "--std", "std", "--no-std"], "mutually exclusive"),
    ];
    for (args, needle) in cases {
        let out = cohdl().args(*args).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(2),
            "`cohdl {}` must be rejected",
            args.join(" ")
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(needle),
            "`cohdl {}` error must mention `{}`:\n{}",
            args.join(" "),
            needle,
            stderr
        );
    }
}

// Review-3 (R3): post-check invocation failures (unwritable out dir, bad
// lock file) must render already-collected diagnostics before the error —
// same rule as selection failures. Reviewer's reproduction: warnings-only
// build with an out dir under /dev/null hid the D003 entirely.
#[test]
fn post_check_failures_preserve_collected_diagnostics() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-r3-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    make_project(&tmp, WITH_D003);

    for json in [false, true] {
        let mut args = vec![
            "build",
            tmp.to_str().unwrap(),
            "--no-std",
            "--out-dir",
            "/dev/null/x",
        ];
        if json {
            args.push("--json");
        }
        let out = cohdl().args(&args).output().unwrap();
        assert_eq!(out.status.code(), Some(2), "unwritable out dir is exit 2");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("D003"),
            "collected D003 must render before the invocation error (json={}):\n{}",
            json,
            stderr
        );
        assert!(stderr.contains("cannot create"), "{}", stderr);
        assert!(out.stdout.is_empty(), "no JSON document on exit 2");
    }

    // A corrupt design.lock is the same class: diagnostics first, then error.
    std::fs::write(tmp.join("design.lock"), "not a lock file }{").unwrap();
    let out = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "corrupt lock is exit 2");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("D003"),
        "collected D003 must render before the lock-parse error:\n{}",
        stderr
    );
    let _ = std::fs::remove_dir_all(&tmp);
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

// ---------------------------------------------------------------------------
// Fourth-review (2026-07-14) CLI regressions.

/// F4: a build WITHOUT `--emit ipc2581` must not delete a user-owned file
/// that merely shares the `<name>.xml` path — only a document CoHDL wrote
/// (identified by its completeness marker) is stale output.
#[test]
fn build_leaves_a_foreign_xml_untouched() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-r4own-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    make_project(&tmp, WITH_LAYOUT);
    let out = tmp.join("out");
    std::fs::create_dir_all(&out).unwrap();
    // A user's own file at the exact stale-artifact path.
    let foreign = out.join("t.xml");
    std::fs::write(&foreign, "<my-own-file/>\n").unwrap();
    let status = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std"])
        .status()
        .unwrap();
    assert!(status.success());
    assert_eq!(
        std::fs::read_to_string(&foreign).unwrap(),
        "<my-own-file/>\n",
        "a foreign xml must survive a no-emit build"
    );
    // `--emit` over a foreign file at the output path is REFUSED (R6-1
    // ownership) — the build fails and the foreign file is preserved, rather
    // than being clobbered.
    let out2 = cohdl()
        .args([
            "build",
            tmp.to_str().unwrap(),
            "--no-std",
            "--emit",
            "ipc2581",
        ])
        .output()
        .unwrap();
    assert!(
        !out2.status.success(),
        "--emit must refuse to overwrite a foreign xml"
    );
    assert!(String::from_utf8_lossy(&out2.stderr).contains("refusing to overwrite"));
    assert_eq!(
        std::fs::read_to_string(&foreign).unwrap(),
        "<my-own-file/>\n",
        "the foreign xml is still intact"
    );
    // On a CLEAN path, --emit writes our marked doc, and a later no-emit
    // build removes it (marker-gated stale removal).
    std::fs::remove_file(&foreign).unwrap();
    cohdl()
        .args([
            "build",
            tmp.to_str().unwrap(),
            "--no-std",
            "--emit",
            "ipc2581",
        ])
        .status()
        .unwrap();
    assert!(std::fs::read_to_string(&foreign)
        .unwrap()
        .contains("logical-complete,physical-minimal"));
    cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std"])
        .status()
        .unwrap();
    assert!(!foreign.exists(), "a marked stale doc is removed");
    let _ = std::fs::remove_dir_all(&tmp);
}

/// F4: a manifest package name that is not a safe basename (path separators,
/// `..`) must be rejected — otherwise artifact writes/deletes escape `out/`.
#[test]
fn traversal_package_name_is_rejected() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-r4esc-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(tmp.join("src")).unwrap();
    std::fs::write(
        tmp.join("cohdl.toml"),
        "[package]\nname = \"../escaped\"\n[design]\ntop = \"B\"\n",
    )
    .unwrap();
    std::fs::write(
        tmp.join("src/main.cohdl"),
        "pub device D { pins { A: 1 [passive] } }\ndesign B { inst a: D  net N: a.A }\n",
    )
    .unwrap();
    let out = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "traversal name must be exit 2");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("not a valid identifier") || err.contains("not a safe output basename"),
        "must reject the traversal name:\n{}",
        err
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

/// F12.5: `--emit` given twice must be rejected, not silently last-one-wins
/// (which made `--emit bogus --emit ipc2581` succeed but the reverse fail).
#[test]
fn duplicate_emit_flag_is_rejected() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-r4emit-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    make_project(&tmp, WITH_LAYOUT);
    let out = cohdl()
        .args([
            "build",
            tmp.to_str().unwrap(),
            "--no-std",
            "--emit",
            "ipc2581",
            "--emit",
            "ipc2581",
        ])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(2),
        "duplicate --emit must be exit 2"
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("more than once"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// Fifth-review (2026-07-15) CLI regression.

const PAD_PROJECT: &str = "\
pub pad P { shape: rect, size: (0.6mm, 0.7mm), layer: top_copper, plating: smd }
pub footprint FP {
    pad 1: P at (-0.5mm, 0mm)
    pad 2: P at (0.5mm, 0mm)
}
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
pub part R1: Res { primary { mfr: \"m\", mpn: \"n\", footprint: FP } }
design B { inst r1: R1  inst r2: R1  net N: r1.A, r2.A  net M: r1.B, r2.B }
";

// R5-6: an ordinary build clears its own `.kicad_mod` projections but must
// preserve a foreign `.kicad_mod` (a file format is not proof of ownership).
#[test]
fn build_preserves_foreign_kicad_mod() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-r56-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    make_project(&tmp, PAD_PROJECT);
    let fpdir = tmp.join("out/footprints");
    std::fs::create_dir_all(&fpdir).unwrap();
    let foreign = fpdir.join("foreign.kicad_mod");
    std::fs::write(&foreign, "(footprint \"mine\" (generator \"kicad\"))\n").unwrap();
    let status = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std"])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(
        foreign.exists(),
        "a foreign .kicad_mod must survive a build"
    );
    assert_eq!(
        std::fs::read_to_string(&foreign).unwrap(),
        "(footprint \"mine\" (generator \"kicad\"))\n"
    );
    // Our own projections were written.
    assert!(fpdir.join("board-FP.kicad_mod").exists() || fpdir.join("t-FP.kicad_mod").exists());
    let _ = std::fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// Sixth-review (2026-07-15) CLI regressions.

// R6-1: a foreign file at the EXACT generated `.kicad_mod` destination is
// refused, not overwritten; a symlink at the destination is refused, so the
// build cannot escape out/ to mutate the target.
#[test]
fn build_refuses_exact_name_foreign_and_symlink() {
    // Exact-name foreign file.
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-r61a-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    make_project(&tmp, PAD_PROJECT);
    let fpdir = tmp.join("out/footprints");
    std::fs::create_dir_all(&fpdir).unwrap();
    let exact = fpdir.join("t-FP.kicad_mod"); // package "t", footprint FP
    std::fs::write(&exact, "(footprint MINE (generator kicad))\n").unwrap();
    let out = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std"])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "must refuse to overwrite foreign exact-name file"
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("refusing to overwrite"));
    assert_eq!(
        std::fs::read_to_string(&exact).unwrap(),
        "(footprint MINE (generator kicad))\n",
        "foreign file preserved"
    );
    let _ = std::fs::remove_dir_all(&tmp);

    // Symlink at the destination → refused, target untouched.
    #[cfg(unix)]
    {
        let tmp = std::env::temp_dir().join(format!("cohdl-cli-r61b-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        make_project(&tmp, PAD_PROJECT);
        let fpdir = tmp.join("out/footprints");
        std::fs::create_dir_all(&fpdir).unwrap();
        let victim = tmp.join("victim.txt");
        std::fs::write(&victim, "VICTIM").unwrap();
        std::os::unix::fs::symlink("../../victim.txt", fpdir.join("t-FP.kicad_mod")).unwrap();
        let out = cohdl()
            .args(["build", tmp.to_str().unwrap(), "--no-std"])
            .output()
            .unwrap();
        assert!(!out.status.success(), "must refuse a symlink destination");
        assert!(String::from_utf8_lossy(&out.stderr).contains("symlink"));
        assert_eq!(
            std::fs::read_to_string(&victim).unwrap(),
            "VICTIM",
            "symlink target must be untouched"
        );
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
