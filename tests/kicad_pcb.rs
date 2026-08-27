//! Native `.kicad_pcb` board emitter (`build --emit kicad_pcb`).
//!
//! The load-bearing properties: (1) zero impact — with or without the flag,
//! every other artifact's bytes and the verdict are identical; (2) byte
//! determinism with a fixed format stamp and no wall clock; (3) the RFC-026
//! back side in KiCad's own on-disk representation, pinned against worked
//! examples (the encoding pcbnew itself writes, verified live against
//! pcbnew-generated boards — the 82%-Quilter-run class of flip mistake is
//! exactly what these strings would catch); (4) nets bound by name onto
//! every physical pad copy, nc pins absent; (5) unplaced components never
//! stack at (0, 0).

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};
use std::path::PathBuf;
use std::process::Command;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The side_rotate.rs asymmetric fixture: a pad off both axes so that a
/// mirror/rotate mistake cannot cancel out.
const LIB: &str = r#"
pub trait Ic { designator_prefix: "U" }
pub pad P_R { shape: rect, size: (0.8mm, 0.3mm), layer: top_copper, plating: smd }
pub footprint FP_A {
    pad 1: P_R at (1mm, 2mm)
    pad 2: P_R at (-1mm, -2mm) rotate 90
    courtyard { shape: rect, at: (0mm, 0mm), size: (4mm, 6mm) }
}
pub device DevA { pins { A: 1 [passive], B: 2 [passive] } }
impl Ic for DevA {}
pub part PART_A: DevA { primary { mfr: "m", mpn: "a", footprint: FP_A } }
"#;

const BODY: &str = r#"
design B {
    inst u1: PART_A
    inst u2: PART_A
    net N: u1.A, u2.A
    net M: u1.B, u2.B
    layout {
        board_outline: "mechanical/outline.dxf"
        place u1 at (-5mm, 0mm)
        place u2 at (5mm, 0mm) rotate 90 side bottom
    }
}
"#;

/// A closed 30x20mm rectangle outline (centered at origin) for fixtures that
/// declare `board_outline:` — resolved FS-free.
const DXF_30X20: &str = "0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n8\nEdge.Cuts\n90\n4\n70\n1\n\
    10\n-15\n20\n-10\n10\n15\n20\n-10\n10\n15\n20\n10\n10\n-15\n20\n10\n0\nENDSEC\n";

fn emit(src: &str, dxf: Option<&str>) -> String {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    if let Some(dxf) = dxf {
        cohdl::pipeline::resolve_board_outline(&mut checked, |_| Ok(dxf.to_string()));
    }
    let _artifacts = build_artifacts(&mut checked, &LockState::default()).expect("build");
    assert!(
        !checked.diags.has_errors(),
        "fixture must build cleanly:\n{}",
        checked.diags.render(&checked.sm)
    );
    let ir = checked.ir.as_ref().unwrap();
    cohdl::emit::kicad_pcb::emit_kicad_pcb(&checked.world, ir, "board")
}

/// The footprint block whose Reference property is `refdes`.
fn fp_block(board: &str, refdes: &str) -> String {
    let needle = format!("(property \"Reference\" \"{}\"", refdes);
    board
        .split("\n\t(footprint ")
        .find(|chunk| chunk.contains(&needle))
        .unwrap_or_else(|| panic!("no footprint block for {refdes}"))
        .to_string()
}

// ---------------------------------------------------------------------------
// Determinism and format identity
// ---------------------------------------------------------------------------

#[test]
fn board_is_deterministic_with_a_fixed_format_stamp() {
    let src = format!("{LIB}{BODY}");
    let a = emit(&src, Some(DXF_30X20));
    let b = emit(&src, Some(DXF_30X20));
    assert_eq!(a, b, "same IR must emit identical bytes");
    assert!(a.starts_with("(kicad_pcb\n\t(version 20260206)\n\t(generator \"cohdl\")\n"));
    assert_eq!(
        a.matches("(version ").count(),
        1,
        "exactly one version stamp"
    );
    assert!(
        a.ends_with("\t(embedded_fonts no)\n)\n"),
        "board-level embedded_fonts closes the file"
    );
    // Deterministic uuids: RFC-4122-shaped, and stable across the two runs
    // (covered by the byte equality above); no run may leak randomness.
    assert!(a.contains("(uuid \""));
}

// ---------------------------------------------------------------------------
// Placement encoding — top side
// ---------------------------------------------------------------------------

#[test]
fn top_side_placement_passes_coordinates_through_verbatim() {
    let board = emit(&format!("{LIB}{BODY}"), Some(DXF_30X20));
    let u1 = fp_block(&board, "U1");
    // CoHDL's authoring frame IS KiCad's board frame: no y negation, no
    // offset; a 0 rotation omits the angle atom entirely.
    assert!(u1.contains("\t\t(layer \"F.Cu\")\n"), "{u1}");
    assert!(u1.contains("\t\t(at -5 0)\n"), "{u1}");
    // Pad-local geometry stays authored; RFC-025 pad rotation rides the
    // 3-argument at form.
    assert!(u1.contains("\t\t\t(at 1 2)\n"), "{u1}");
    assert!(u1.contains("\t\t\t(at -1 -2 90)\n"), "{u1}");
    assert!(
        u1.contains("(layers \"F.Cu\" \"F.Mask\" \"F.Paste\")"),
        "{u1}"
    );
    assert!(u1.contains("(property \"Reference\" \"U1\""), "{u1}");
    assert!(
        !u1.contains("justify mirror"),
        "top side never mirrors: {u1}"
    );
}

// ---------------------------------------------------------------------------
// RFC-026 back side — KiCad's own on-disk representation
// ---------------------------------------------------------------------------

/// The worked example, cross-checked against the IPC-2581 emitter's model
/// (tests/side_rotate.rs) and pcbnew's own files:
///
/// u2: place (5, 0) rotate 90 side bottom.
/// - Footprint angle = authored 90 + the flip's folded-in 180 = 270,
///   normalized to pcbnew's (-180, 180] as -90.
/// - Pad 1, local (1, 2), no local rotation: stored y-negated (1, -2) with
///   angle 270 (= -90 mod 360; pad angles normalize to [0, 360)).
///   Semantically: absolute = (5,0) + Rot(-90)·(1,-2) = (5,0)+(2,1) = (7,1),
///   which equals the IPC model's mirror-x-then-rotate (7,-1) in its y-up
///   frame — the LEFT_RIGHT flip, never TOP_BOTTOM.
/// - Pad 2, local (-1, -2) rotate 90: a reflection REVERSES pad-local
///   rotation: angle = (-90 - 90) mod 360 = 180, position (-1, 2).
#[test]
fn back_side_is_kicads_own_on_disk_representation() {
    let board = emit(&format!("{LIB}{BODY}"), Some(DXF_30X20));
    let u2 = fp_block(&board, "U2");
    assert!(u2.contains("\t\t(layer \"B.Cu\")\n"), "{u2}");
    assert!(u2.contains("\t\t(at 5 0 -90)\n"), "{u2}");
    assert!(u2.contains("\t\t\t(at 1 -2 270)\n"), "{u2}");
    assert!(u2.contains("\t\t\t(at -1 2 180)\n"), "{u2}");
    // Every layer flips to its back counterpart.
    assert!(
        u2.contains("(layers \"B.Cu\" \"B.Mask\" \"B.Paste\")"),
        "{u2}"
    );
    assert!(u2.contains("(layer \"B.SilkS\")"), "{u2}");
    assert!(u2.contains("(layer \"B.CrtYd\")"), "{u2}");
    // Texts mirror their justification on the back.
    assert!(u2.contains("(justify mirror)"), "{u2}");
    // The courtyard's stored y also negates (symmetric here, so the corners
    // simply survive) — pinned via the rect being present at all.
    assert!(u2.contains("(fp_rect"), "{u2}");
}

#[test]
fn back_side_with_zero_rotation_folds_the_flip_into_180() {
    // R = 0, bottom: footprint angle 180; a pad at local (1, 2) stores
    // (1, -2) at angle 180 — absolutely Rot(180)·(1,-2) = (-1, 2): the
    // empirical LEFT_RIGHT check from the flip-trap ledger.
    let src = format!("{LIB}{BODY}").replace(
        "place u2 at (5mm, 0mm) rotate 90 side bottom",
        "place u2 at (5mm, 0mm) side bottom",
    );
    let board = emit(&src, Some(DXF_30X20));
    let u2 = fp_block(&board, "U2");
    assert!(u2.contains("\t\t(at 5 0 180)\n"), "{u2}");
    assert!(u2.contains("\t\t\t(at 1 -2 180)\n"), "{u2}");
    // Reversed pad-local rotation: (180 - 90) mod 360 = 90.
    assert!(u2.contains("\t\t\t(at -1 2 90)\n"), "{u2}");
}

// ---------------------------------------------------------------------------
// Nets
// ---------------------------------------------------------------------------

#[test]
fn nets_bind_by_name_on_every_pad_copy_and_nc_is_absent() {
    let src = r#"
pub pad P_S { shape: rect, size: (0.5mm, 0.5mm), layer: top_copper, plating: smd }
pub footprint FP_EP {
    pad 1: P_S at (-1mm, 0mm)
    pad 2: P_S at (1mm, 0mm)
    pad 3: P_S at (0mm, 0mm)
    pad 3: P_S at (0mm, 1mm)
    pad 3: P_S at (0mm, -1mm)
}
pub device DevE { pins { A: 1 [passive], B: 2 [passive], EP: 3 [passive] } }
pub part PART_E: DevE { primary { mfr: "m", mpn: "e", footprint: FP_EP } }

design B {
    inst u1: PART_E
    inst u2: PART_E
    net SIG: u1.A, u2.A
    net GNDISH: u1.EP, u2.EP
    nc: u1.B, u2.B
}
"#;
    let board = emit(src, None);
    let u1 = fp_block(&board, "U1");
    // The exposed pad's three same-numbered copies EVERY one carry the net —
    // a partial binding reads as copper shorted to an unnamed net.
    assert_eq!(
        u1.matches("(net \"GNDISH\")").count(),
        3,
        "all three pad-3 copies must carry the net: {u1}"
    );
    assert_eq!(u1.matches("(net \"SIG\")").count(), 1, "{u1}");
    // nc pins are represented by ABSENCE: pad 2 has no net clause at all.
    let pad2 = u1
        .split("\n\t\t(pad ")
        .find(|p| p.starts_with("\"2\""))
        .expect("pad 2 present");
    assert!(!pad2.contains("(net "), "nc pad must carry no net: {pad2}");
}

// ---------------------------------------------------------------------------
// Staging: unplaced components never stack at the origin
// ---------------------------------------------------------------------------

#[test]
fn unplaced_without_outline_stage_on_the_grid() {
    let src = format!(
        "{LIB}\ndesign B {{\n    inst u1: PART_A\n    inst u2: PART_A\n    net N: u1.A, u2.A\n    net M: u1.B, u2.B\n}}\n"
    );
    let board = emit(&src, None);
    let u1 = fp_block(&board, "U1");
    let u2 = fp_block(&board, "U2");
    // The plain staging grid the retired pcbnew script used: 12mm pitch from
    // (40, 40), designator order.
    assert!(u1.contains("\t\t(at 40 40)\n"), "{u1}");
    assert!(u2.contains("\t\t(at 52 40)\n"), "{u2}");
}

#[test]
fn unplaced_with_outline_stage_on_the_shared_shelf_outside_it() {
    let src = format!(
        "{LIB}\ndesign B {{\n    inst u1: PART_A\n    inst u2: PART_A\n    net N: u1.A, u2.A\n    net M: u1.B, u2.B\n    layout {{\n        board_outline: \"mechanical/outline.dxf\"\n    }}\n}}\n"
    );
    let board = emit(&src, Some(DXF_30X20));
    // The same shelf the IPC-2581 document stages: strictly outside the
    // outline's +x edge (15mm) — never inside, never at (0, 0).
    for refdes in ["U1", "U2"] {
        let block = fp_block(&board, refdes);
        let at = block
            .lines()
            .find(|l| l.starts_with("\t\t(at "))
            .expect("placement");
        let x: f64 = at
            .trim()
            .trim_start_matches("(at ")
            .split_whitespace()
            .next()
            .unwrap()
            .parse()
            .unwrap();
        assert!(x > 15.0, "{refdes} must stage outside the outline: {at}");
    }
}

// ---------------------------------------------------------------------------
// Board outline
// ---------------------------------------------------------------------------

#[test]
fn outline_draws_edge_cuts_lines() {
    let board = emit(&format!("{LIB}{BODY}"), Some(DXF_30X20));
    assert_eq!(
        board.matches("\t(gr_line\n").count(),
        4,
        "a rectangle outline is four gr_lines"
    );
    assert!(board.contains("\t\t(layer \"Edge.Cuts\")\n"));
    assert!(
        board.contains("\t\t(stroke\n\t\t\t(width 0.1)\n\t\t\t(type default)\n\t\t)\n"),
        "0.1mm default-stroke Edge.Cuts, the established convention"
    );
    // No outline → no board-level graphics at all.
    let bare = emit(
        &format!(
            "{LIB}\ndesign B {{\n    inst u1: PART_A\n    inst u2: PART_A\n    net N: u1.A, u2.A\n    net M: u1.B, u2.B\n}}\n"
        ),
        None,
    );
    assert!(!bare.contains("(gr_line"), "no invented outline");
}

// ---------------------------------------------------------------------------
// Example boards: real designs emit loadable, deterministic boards
// ---------------------------------------------------------------------------

/// Build a repo example with the exact libraries in its manifest (the
/// tests/ipc2581.rs harness shape) and emit its board.
fn example_board(dir: &str) -> String {
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
    let proj_dir = proj.dir.clone();
    cohdl::pipeline::resolve_board_outline(&mut checked, |p| {
        std::fs::read_to_string(proj_dir.join(p)).map_err(|e| e.to_string())
    });
    let _ = build_artifacts(&mut checked, &LockState::default()).expect("build");
    let ir = checked.ir.as_ref().unwrap();
    cohdl::emit::kicad_pcb::emit_kicad_pcb(&checked.world, ir, &proj.name)
}

#[test]
fn example_boards_are_deterministic_and_complete() {
    let pico = example_board("examples/rpi-pico2");
    assert_eq!(pico, example_board("examples/rpi-pico2"), "byte-stable");
    assert_eq!(
        pico.matches("\n\t(footprint ").count(),
        52,
        "every instance embeds a footprint"
    );
    // The example's real DXF outline has both lines and arcs.
    assert!(pico.contains("\t(gr_arc\n"), "outline arcs project");
    assert!(pico.contains("\t(gr_line\n"));
    // Placed parts carry their authored positions; the MCU is placed.
    assert!(pico.contains("(property \"Reference\" \"U1\""));

    let sf32 = example_board("examples/sf32-miniboard");
    assert_eq!(
        sf32.matches("\n\t(footprint ").count(),
        49,
        "every instance embeds a footprint"
    );
}

// ---------------------------------------------------------------------------
// CLI: flag wiring, zero impact, ownership
// ---------------------------------------------------------------------------

fn cohdl() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cohdl"))
}

const CLI_FIXTURE: &str = r#"
pub pad P { shape: rect, size: (0.8mm, 0.3mm), layer: top_copper, plating: smd }
pub footprint F {
    pad 1: P at (1mm, 2mm)
    pad 2: P at (-1mm, -2mm)
}
pub device D { pins { A: 1 [passive], B: 2 [passive] } }
pub part PT: D { primary { mfr: "m", mpn: "x", footprint: F } }

design B {
    inst u1: PT
    inst u2: PT
    net N: u1.A, u2.A
    net M: u1.B, u2.B
}
"#;

fn make_project(root: &std::path::Path) {
    let _ = std::fs::remove_dir_all(root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("cohdl.toml"),
        "[package]\nname = \"t\"\n\n[design]\ntop = \"B\"\n",
    )
    .unwrap();
    std::fs::write(root.join("src/main.cohdl"), CLI_FIXTURE).unwrap();
}

#[test]
fn cli_board_only_with_flag_and_zero_impact_on_everything_else() {
    let tmp = std::env::temp_dir().join(format!("cohdl-kpcb-zero-{}", std::process::id()));
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
    let board_path = tmp.join("out/t.kicad_pcb");

    run(&[]);
    assert!(!board_path.exists(), "no flag, no board");
    let netlist = std::fs::read_to_string(tmp.join("out/t.net")).unwrap();
    let bom = std::fs::read_to_string(tmp.join("out/t-bom.csv")).unwrap();
    let lock = std::fs::read_to_string(tmp.join("design.lock")).unwrap();

    run(&["--emit", "kicad_pcb"]);
    assert!(board_path.exists(), "flagged build writes the board");
    let board = std::fs::read_to_string(&board_path).unwrap();
    assert!(board.starts_with("(kicad_pcb\n\t(version 20260206)\n"));
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

    // Dropping the flag sweeps the stale board (it is manifest-owned).
    run(&[]);
    assert!(!board_path.exists(), "stale board swept without the flag");
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn cli_emits_both_formats_in_one_build() {
    let tmp = std::env::temp_dir().join(format!("cohdl-kpcb-both-{}", std::process::id()));
    make_project(&tmp);
    let out = cohdl()
        .args([
            "build",
            tmp.to_str().unwrap(),
            "--no-std",
            "--emit",
            "ipc2581",
            "--emit",
            "kicad_pcb",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(tmp.join("out/t.xml").exists(), "IPC document written");
    assert!(tmp.join("out/t.kicad_pcb").exists(), "board written");
    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn cli_flag_matrix_names_both_formats() {
    let tmp = std::env::temp_dir().join(format!("cohdl-kpcb-matrix-{}", std::process::id()));
    make_project(&tmp);
    // Command compatibility outranks value validity.
    let out = cohdl()
        .args([
            "check",
            tmp.to_str().unwrap(),
            "--no-std",
            "--emit",
            "kicad_pcb",
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
    // The same value twice is still the duplicate error (F12.5) even though
    // distinct values are now legal together.
    let out = cohdl()
        .args([
            "build",
            tmp.to_str().unwrap(),
            "--no-std",
            "--emit",
            "kicad_pcb",
            "--emit",
            "kicad_pcb",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("more than once"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = std::fs::remove_dir_all(&tmp);
}

/// A routed board is protected exactly while it is FOREIGN (not in the
/// build manifest): the emitter refuses to overwrite it and the sweep never
/// touches it. (Once cohdl owns the path, rebuilds rewrite it — route on a
/// copy outside out/, per docs/kicad_pcb.md.)
#[test]
fn cli_foreign_board_file_is_never_clobbered() {
    let tmp = std::env::temp_dir().join(format!("cohdl-kpcb-foreign-{}", std::process::id()));
    make_project(&tmp);
    std::fs::create_dir_all(tmp.join("out")).unwrap();
    std::fs::write(tmp.join("out/t.kicad_pcb"), "ROUTED BOARD — precious\n").unwrap();

    // A build without the flag leaves the foreign file alone.
    let out = cohdl()
        .args(["build", tmp.to_str().unwrap(), "--no-std"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        std::fs::read_to_string(tmp.join("out/t.kicad_pcb")).unwrap(),
        "ROUTED BOARD — precious\n"
    );

    // A flagged build REFUSES rather than overwrite what it did not write.
    let out = cohdl()
        .args([
            "build",
            tmp.to_str().unwrap(),
            "--no-std",
            "--emit",
            "kicad_pcb",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success(), "must refuse to clobber");
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("refusing to overwrite"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(tmp.join("out/t.kicad_pcb")).unwrap(),
        "ROUTED BOARD — precious\n"
    );
    let _ = std::fs::remove_dir_all(&tmp);
}
