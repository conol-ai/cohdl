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

// Every shipped library package and example stays canonical.
#[test]
fn fmt_check_gate_std_and_examples() {
    fn packages_under(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        if dir.join("cohdl.toml").is_file() {
            out.push(dir.to_path_buf());
            return;
        }
        let mut children: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.is_dir())
            .collect();
        children.sort();
        for child in children {
            packages_under(&child, out);
        }
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut packages = Vec::new();
    packages_under(&root.join("lib"), &mut packages);
    packages_under(&root.join("examples"), &mut packages);
    for path in packages {
        let out = cohdl()
            .args(["fmt", path.to_str().unwrap(), "--check"])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "`{}` is not canonical:\n{}",
            path.display(),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

#[test]
fn shipped_libraries_check_standalone() {
    fn packages_under(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        if dir.join("cohdl.toml").is_file() {
            out.push(dir.to_path_buf());
            return;
        }
        let mut children: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .filter(|path| path.is_dir())
            .collect();
        children.sort();
        for child in children {
            packages_under(&child, out);
        }
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut packages = Vec::new();
    packages_under(&root.join("lib"), &mut packages);
    for path in packages {
        // `std` cannot be loaded as a user project because its package name is
        // reserved. Every other package below checks against it, exercising
        // the core prelude as part of this gate.
        if path == root.join("lib/std") {
            continue;
        }
        let out = cohdl()
            .args(["check", path.to_str().unwrap()])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "`{}` does not check standalone:\n{}{}",
            path.display(),
            String::from_utf8_lossy(&out.stdout),
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
    let ex = root.join("examples/rpi-pico2");
    let out = cohdl()
        .args(["check", ex.to_str().unwrap(), "--check"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "--check rejected on check");
    assert!(String::from_utf8_lossy(&out.stderr).contains("fmt"));

    let out = cohdl()
        .args(["fmt", root.join("lib/std").to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2), "--json rejected on fmt");

    // R3: the rest of the matrix. Every invalid (command, flag) pair is
    // exit 2 with a message naming the flag.
    let cases: &[(&[&str], &str)] = &[
        (&["check", "--out-dir", "x"], "--out-dir"),
        (&["fmt", "--design", "B"], "--design"),
        (&["fmt", "--std", "lib/std"], "--std"),
        (&["fmt", "--no-std"], "--no-std"),
        (&["fmt", "--out-dir", "x"], "--out-dir"),
        (&["lsp", "--json"], "lsp"),
        (&["lsp", "--design", "B"], "lsp"),
        (&["lsp", "some-path"], "lsp"),
        (
            &["check", "--std", "lib/std", "--no-std"],
            "mutually exclusive",
        ),
        (
            &["build", "--std", "lib/std", "--no-std"],
            "mutually exclusive",
        ),
        (&["search", "stm32", "--check"], "--check"),
        (&["search", "stm32", "--design", "B"], "--design"),
        (&["search", "stm32", "--std", "lib/std"], "--std"),
        (&["search", "stm32", "--no-std"], "--no-std"),
        (&["search", "stm32", "--out-dir", "x"], "--out-dir"),
        (&["search", "stm32", "--emit", "ipc2581"], "--emit"),
        (&["search", "stm32", "--dep", "std"], "--dep"),
        (&["search", "stm32", "--out", "result.json"], "--out"),
        (&["search", "stm32", "--publish"], "--publish"),
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

#[test]
fn search_query_invocation_contract_is_validated_before_network() {
    let cases: &[(&[&str], &str)] = &[
        (&["search"], "needs a query"),
        (&["search", "stm32", "usb"], "exactly one query"),
        (&["search", "ab"], "at least 3 characters"),
        (&["search", "stm\n32"], "control characters"),
    ];
    for (args, needle) in cases {
        let out = cohdl()
            .env("COHDL_REGISTRY", "http://127.0.0.1:9")
            .args(*args)
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(2),
            "`cohdl {}` is an invocation error",
            args.join(" ")
        );
        assert!(out.stdout.is_empty(), "invocation errors emit no JSON");
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains(needle),
            "`cohdl {}` must mention `{needle}`:\n{stderr}",
            args.join(" ")
        );
    }

    let too_long = "x".repeat(129);
    let out = cohdl()
        .env("COHDL_REGISTRY", "http://127.0.0.1:9")
        .args(["search", too_long.as_str()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("at most 128 UTF-8 bytes"));

    // The minimum counts Unicode scalar values, while the maximum counts the
    // actual UTF-8 bytes sent on the wire.
    let out = cohdl()
        .env("COHDL_REGISTRY", "http://127.0.0.1:9")
        .args(["search", "电源"])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(2),
        "two Unicode scalars are too short"
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("at least 3 characters"));

    let out = cohdl()
        .env("COHDL_REGISTRY", "http://127.0.0.1:9")
        .args(["search", "--", "-12V"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1), "`--` admits a hyphen-led query");
    assert!(String::from_utf8_lossy(&out.stderr).contains("E1204"));
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
        .contains("logical-complete,placement-staged,unrouted"));
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

// ---------------------------------------------------------------------------
// Seventh-review (2026-07-15) CLI regressions.

// R7-1: a symlinked output directory must not let a build escape the project.
#[cfg(unix)]
#[test]
fn build_refuses_symlinked_out_dir() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-r71a-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    make_project(&tmp, PAD_PROJECT);
    let victim = std::env::temp_dir().join(format!("cohdl-victim-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&victim);
    std::fs::create_dir_all(&victim).unwrap();
    std::os::unix::fs::symlink(&victim, tmp.join("out")).unwrap();
    let out = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "symlinked out/ must be refused");
    assert!(String::from_utf8_lossy(&out.stderr).contains("symlink"));
    assert_eq!(
        std::fs::read_dir(&victim).unwrap().count(),
        0,
        "nothing written into the victim dir"
    );
    let _ = std::fs::remove_dir_all(&tmp);
    let _ = std::fs::remove_dir_all(&victim);
}

// R7-1: a foreign regular file at a generated NET/BOM path (no marker, no
// manifest) is refused, not silently replaced.
#[test]
fn build_refuses_foreign_net_and_bom() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-r71b-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    make_project(&tmp, PAD_PROJECT);
    std::fs::create_dir_all(tmp.join("out")).unwrap();
    std::fs::write(tmp.join("out/t.net"), "FOREIGN NET\n").unwrap();
    let out = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "foreign net must be refused");
    assert!(String::from_utf8_lossy(&out.stderr).contains("refusing to overwrite"));
    assert_eq!(
        std::fs::read_to_string(tmp.join("out/t.net")).unwrap(),
        "FOREIGN NET\n"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

// R7-1: a clean build writes a deterministic manifest, and a second build
// (manifest present) overwrites its own files without complaint.
#[test]
fn build_manifest_enables_reownership() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-r71c-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    make_project(&tmp, PAD_PROJECT);
    assert!(cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std"])
        .status()
        .unwrap()
        .success());
    assert!(tmp.join("out/.cohdl-manifest").exists(), "manifest written");
    // Second build succeeds (ownership via the manifest).
    assert!(cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std"])
        .status()
        .unwrap()
        .success());
    let _ = std::fs::remove_dir_all(&tmp);
}

// ---------------------------------------------------------------------------
// docs/apidocs.md: the `cohdl docs` verb at the binary boundary.

/// A project with a vendored std under `deps/` (the RFC-029 family layout),
/// so `docs` resolves its locked dependency set without a global registry.
fn make_docs_project(root: &Path, lib_src: &str) {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (_, std_manifest) = cohdl::project::peek_manifest(&repo.join("lib/std")).unwrap();
    let std_version = std_manifest.version.expect("std pins a version");
    let vendored = root.join("deps/std").join(&std_version);
    std::fs::create_dir_all(vendored.join("src")).unwrap();
    std::fs::copy(repo.join("lib/std/cohdl.toml"), vendored.join("cohdl.toml")).unwrap();
    for entry in std::fs::read_dir(repo.join("lib/std/src")).unwrap() {
        let p = entry.unwrap().path();
        if p.extension().is_some_and(|e| e == "cohdl") {
            std::fs::copy(&p, vendored.join("src").join(p.file_name().unwrap())).unwrap();
        }
    }
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        format!(
            "[package]\nname = \"t\"\nversion = \"0.1.0\"\nlicense = \"MIT\"\n\n[dependencies]\nstd = \"{std_version}\"\n"
        ),
    )
    .unwrap();
    std::fs::write(root.join("src/lib.cohdl"), lib_src).unwrap();
}

const DOCS_LIB: &str = r#"
pub device Res { pins { A: 1 [passive], B: 2 [passive] } }
impl TwoTerminal for Res {}
pub footprint TFP {}
pub part R1: Res { primary { mfr: "m", mpn: "n", footprint: TFP } }
"#;

#[test]
fn docs_command_emits_json_to_stdout_and_file() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-docs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    make_docs_project(&tmp, DOCS_LIB);

    // stdout mode: the document (and only the document) on stdout.
    let out = cohdl()
        .args(["docs", tmp.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.starts_with("{\n  \"schema_version\": 1,\n"),
        "document leads with the schema version:\n{}",
        &stdout[..stdout.len().min(200)]
    );
    assert!(stdout.contains("\"fq\": \"t::R1\""));
    assert!(stdout.ends_with("}\n"));

    // --out FILE mode: identical bytes on disk, nothing on stdout.
    let out_path = tmp.join("api.json");
    let out2 = cohdl()
        .args([
            "docs",
            tmp.to_str().unwrap(),
            "--out",
            out_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out2.status.success(),
        "{}",
        String::from_utf8_lossy(&out2.stderr)
    );
    assert!(out2.stdout.is_empty(), "--out keeps stdout clean");
    assert_eq!(
        std::fs::read_to_string(&out_path).unwrap(),
        stdout,
        "file and stdout bytes are identical"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn docs_command_refuses_errors_and_bad_flags() {
    let tmp = std::env::temp_dir().join(format!("cohdl-cli-docs2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    // `nosuch` is unresolvable — the package does not check.
    make_docs_project(
        &tmp,
        "pub part P1: nosuch::Dev { primary { mpn: \"x\" } }\n",
    );

    let out = cohdl()
        .args(["docs", tmp.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "check errors → exit 1, no document"
    );
    assert!(out.stdout.is_empty(), "no partial document on errors");
    assert!(!String::from_utf8_lossy(&out.stderr).is_empty());

    // Flag matrix: --json is not valid with docs (the output IS JSON).
    let out = cohdl()
        .args(["docs", tmp.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("`--json` is not valid with `docs`"));

    // And --out/--publish are docs-only.
    let out = cohdl()
        .args(["check", tmp.to_str().unwrap(), "--out", "x.json"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("`--out` is not valid with `check`"));
    let _ = std::fs::remove_dir_all(&tmp);
}
