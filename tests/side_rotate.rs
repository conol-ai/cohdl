//! RFC-025 (rotated pad placements) + RFC-026 (back-side placement)
//! conformance. Both are one optional clause on an existing statement, closed
//! set, default-preserving — so the load-bearing tests are byte-compat (no
//! clause → outputs unchanged in every byte) and the emitter projections.

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};

fn check(src: &str) -> (cohdl::pipeline::Checked, String) {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    checked.diags.sort(&checked.sm);
    let rendered = checked.diags.render(&checked.sm);
    (checked, rendered)
}

fn build(src: &str) -> (cohdl::pipeline::Checked, cohdl::pipeline::BuildArtifacts) {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    assert!(
        !checked.diags.has_errors(),
        "clean build expected:\n{}",
        checked.diags.render(&checked.sm)
    );
    (checked, artifacts.expect("build"))
}

/// An ASYMMETRIC footprint (pad off both axes) so mirror/rotate mistakes
/// cannot cancel out, bound into a minimal buildable design.
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

fn full() -> String {
    format!("{LIB}{BODY}")
}

// ---------------------------------------------------------------------------
// RFC-025: pad rotation.

#[test]
fn pad_rotate_parses_and_emits_kicad_angle() {
    let (checked, _r) = check(&full());
    let fp = &checked.world.footprints["board::FP_A"];
    assert_eq!(fp.pads[0].rotate, 0);
    assert_eq!(fp.pads[1].rotate, 90);
    let (checked, _a) = build(&full());
    let ir = checked.ir.as_ref().unwrap();
    let mods = cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, ir);
    let content = &mods[0].2;
    // Unrotated pad: the 2-argument form, byte-identical to pre-RFC-025.
    assert!(
        content.contains("(pad \"1\" smd rect (at 1 2) (size 0.8 0.3)"),
        "{content}"
    );
    // Rotated pad: KiCad's own 3-argument (at x y angle), size UNCHANGED —
    // the declared rotation is preserved losslessly, never a silent w/h swap.
    assert!(
        content.contains("(pad \"2\" smd rect (at -1 -2 90) (size 0.8 0.3)"),
        "{content}"
    );
}

#[test]
fn pad_rotate_invalid_is_e811() {
    let src = full().replace("rotate 90\n", "rotate 45\n");
    let (_c, r) = check(&src);
    assert!(r.contains("E811"), "{r}");
    assert!(r.contains("{0, 90, 180, 270}"), "must name the set:\n{r}");
}

#[test]
fn pad_rotate_round_trips_through_fmt() {
    use cohdl::fmt::format_source;
    let src = "pub footprint F {\n    pad 1: P at (1mm, 2mm)\n    pad 2: P at (-1mm, -2mm) rotate 90\n    pad 3: P at (0mm, 0mm) rotate 180\n}\n";
    let once = format_source("f.cohdl", src).unwrap();
    assert!(once.contains("pad 1: P at (1mm, 2mm)\n"), "{once}");
    assert!(once.contains("pad 2: P at (-1mm, -2mm) rotate 90"), "{once}");
    // 180 is geometrically a no-op on a rect but the author's stated fact is
    // preserved verbatim (the RFC: CoHDL does not second-guess it).
    assert!(once.contains("pad 3: P at (0mm, 0mm) rotate 180"), "{once}");
    let twice = format_source("f.cohdl", &once).unwrap();
    assert_eq!(once, twice, "fmt not idempotent:\n{once}");
}

// ---------------------------------------------------------------------------
// RFC-026: back-side placement.

#[test]
fn side_parses_layout_json_and_byte_compat() {
    let (_c, a) = build(&full());
    let layout = a.layout.expect("layout.json present");
    // Bottom placement carries the side; the top placement's JSON object is
    // byte-identical to its pre-RFC-026 form (no "side" key at all).
    assert!(layout.contains("\"side\": \"bottom\""), "{layout}");
    assert!(
        layout.contains("{ \"instance\": \"B::u1\", \"at\": [-5, 0], \"rotate\": 0 }"),
        "top-side placement must stay byte-identical:\n{layout}"
    );
}

#[test]
fn side_invalid_is_e1008() {
    let src = full().replace("side bottom", "side left");
    let (_c, r) = check(&src);
    assert!(r.contains("E1008"), "{r}");
    assert!(r.contains("top, bottom"), "must name the set:\n{r}");
}

#[test]
fn side_and_rotate_accept_either_order_and_fmt_canonicalizes() {
    use cohdl::fmt::format_source;
    let a = full();
    let b = full().replace("rotate 90 side bottom", "side bottom rotate 90");
    let (_c1, r1) = check(&a);
    let (_c2, r2) = check(&b);
    assert!(!r1.contains("error"), "{r1}");
    assert!(!r2.contains("error"), "{r2}");
    // fmt canonicalizes to rotate-then-side; default `top` is never written.
    let once = format_source("m.cohdl", &b).unwrap();
    assert!(
        once.contains("place u2 at (5mm, 0mm) rotate 90 side bottom"),
        "{once}"
    );
    assert!(!once.contains("side top"), "{once}");
    let twice = format_source("m.cohdl", &once).unwrap();
    assert_eq!(once, twice);
}

#[test]
fn ipc_carries_side_and_mirrored_pads() {
    let (checked, _a) = build(&full());
    let ir = checked.ir.as_ref().unwrap();
    let xml = cohdl::emit::ipc2581::emit_ipc2581(&checked.world, ir, "board");
    // Component: bottom side rides layerRef + the Xform mirror attribute.
    assert!(
        xml.contains("layerRef=\"B.Cu\" mountType=\"SMT\""),
        "{xml}"
    );
    assert!(xml.contains("mirror=\"true\""), "{xml}");
    // u1 (top) keeps the pre-RFC-026 layerRef.
    assert!(xml.contains("layerRef=\"F.Cu\" mountType=\"SMT\""), "{xml}");
    // Bottom SMD copper lands under a B.Cu LayerFeature (and paste on
    // B.Paste), while u1's stays on F.Cu.
    assert!(xml.contains("<LayerFeature layerRef=\"B.Cu\">"), "{xml}");
    assert!(xml.contains("<LayerFeature layerRef=\"B.Paste\">"), "{xml}");
    // Pad-offset math: u2 at (5, 0) rotate 90 side bottom; pad 1 local (1, 2).
    // Mirror x: (-1, 2); IPC y-up local: (-1, -2); rotate 90 CCW: (2, -1);
    // absolute: (5+2, -0-1) = (7, -1).
    assert!(
        xml.contains("<Location x=\"7\" y=\"-1\"/>"),
        "mirror-then-rotate pad math:\n{xml}"
    );
}

#[test]
fn ipc_with_side_and_pad_rotate_is_schema_valid() {
    let (checked, _a) = build(&full());
    let ir = checked.ir.as_ref().unwrap();
    let xml = cohdl::emit::ipc2581::emit_ipc2581(&checked.world, ir, "board");
    let schema = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/schema/IPC-2581B1.xsd");
    let tmp = std::env::temp_dir().join("cohdl_side_rotate_ipc.xml");
    std::fs::write(&tmp, &xml).unwrap();
    let out = std::process::Command::new("xmllint")
        .args(["--noout", "--schema"])
        .arg(&schema)
        .arg(&tmp)
        .output();
    if let Ok(o) = out {
        assert!(
            o.status.success(),
            "IPC-2581 with side/pad-rotate fails schema validation:\n{}",
            String::from_utf8_lossy(&o.stderr)
        );
    }
}
