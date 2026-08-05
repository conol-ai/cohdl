//! Regression coverage for KiCad's 0.5 native chamfer-ratio ceiling.

use cohdl::lock::LockState;
use cohdl::pipeline::{build_artifacts, check_files_in};

fn project(src: &str) -> String {
    let files = vec![("src/main.cohdl".to_string(), src.to_string())];
    let mut checked = check_files_in("board", &files, None).expect("selection");
    let artifacts = build_artifacts(&mut checked, &LockState::default());
    checked.diags.sort(&checked.sm);
    assert!(
        artifacts.is_some() && !checked.diags.has_errors(),
        "{}",
        checked.diags.render(&checked.sm)
    );
    cohdl::emit::kicad_mod::emit_kicad_mods(&checked.world, checked.ir.as_ref().unwrap())[0]
        .2
        .clone()
}

const SOURCE: &str = r#"
pub pad P_LargeImplicit {
    shape: rect
    size: (0.675mm, 0.275mm)
    layer: top_copper
    plating: smd
    chamfer: (bottom_right, 0.201mm)
    mask_expansion: 0.05mm
}
pub pad P_LargeExplicit {
    shape: rect
    size: (0.8mm, 0.6mm)
    layer: top_copper
    plating: smd
    chamfer: (top_left, 0.4mm)
    paste: (0.4mm, 0.15mm)
}
pub pad P_LargeNone {
    shape: rect
    size: (0.8mm, 0.6mm)
    layer: bottom_copper
    plating: smd
    chamfer: (top_right, 0.4mm)
    paste: none
}
pub pad P_Native {
    shape: rect
    size: (0.8mm, 0.6mm)
    layer: top_copper
    plating: smd
    chamfer: (bottom_left, 0.3mm)
}
pub pad P_LargeBottomLeft {
    shape: rect
    size: (0.8mm, 0.6mm)
    layer: top_copper
    plating: smd
    chamfer: (bottom_left, 0.4mm)
}
pub footprint FP {
    pad 1: P_LargeImplicit at (1mm, -2mm) rotate 90
    pad 2: P_LargeExplicit at (2mm, -2mm) rotate 270
    pad 3: P_LargeNone at (3mm, -2mm)
    pad 4: P_Native at (4mm, -2mm)
    pad 5: P_LargeBottomLeft at (5mm, -2mm)
}
pub device D {
    pins {
        A: 1 [passive]
        B: 2 [passive]
        C: 3 [passive]
        D: 4 [passive]
        E: 5 [passive]
    }
}
pub part P: D { primary { mfr: "m", mpn: "n", footprint: FP } }
design B { inst p: P nc: p.A, p.B, p.C, p.D, p.E }
"#;

#[test]
fn oversized_chamfer_uses_exact_custom_polygon_and_keeps_pad_controls() {
    let content = project(SOURCE);

    assert!(content.contains(
        "(pad \"1\" smd custom (at 1 -2 90) (size 0.074 0.074) (layers \"F.Cu\" \"F.Paste\" \"F.Mask\") (solder_mask_margin 0.05) (options (clearance outline) (anchor rect)) (primitives (gr_poly (pts (xy -0.3375 -0.1375) (xy 0.3375 -0.1375) (xy 0.3375 -0.0635) (xy 0.1365 0.1375) (xy -0.3375 0.1375)) (width 0) (fill yes))))"
    ), "{content}");

    assert!(content.contains(
        "(pad \"2\" smd custom (at 2 -2 270) (size 0.2 0.2) (layers \"F.Cu\" \"F.Mask\") (options (clearance outline) (anchor rect)) (primitives (gr_poly (pts (xy 0 -0.3) (xy 0.4 -0.3) (xy 0.4 0.3) (xy -0.4 0.3) (xy -0.4 0.1)) (width 0) (fill yes))))"
    ), "{content}");
    assert!(
        content.contains("(pad \"\" smd rect (at 2 -2 270) (size 0.4 0.15) (layers \"F.Paste\"))"),
        "{content}"
    );

    assert!(content.contains(
        "(pad \"3\" smd custom (at 3 -2) (size 0.2 0.2) (layers \"B.Cu\" \"B.Mask\") (options (clearance outline) (anchor rect)) (primitives (gr_poly (pts (xy -0.4 -0.3) (xy 0 -0.3) (xy 0.4 0.1) (xy 0.4 0.3) (xy -0.4 0.3)) (width 0) (fill yes))))"
    ), "{content}");
    assert!(content.contains(
        "(pad \"5\" smd custom (at 5 -2) (size 0.2 0.2) (layers \"F.Cu\" \"F.Paste\" \"F.Mask\") (options (clearance outline) (anchor rect)) (primitives (gr_poly (pts (xy -0.4 -0.3) (xy 0.4 -0.3) (xy 0.4 0.3) (xy 0 0.3) (xy -0.4 -0.1)) (width 0) (fill yes))))"
    ), "{content}");
    assert_eq!(content.matches("B.Paste").count(), 0, "{content}");
    assert_eq!(content.matches("(pad \"1\"").count(), 1, "{content}");
}

#[test]
fn exactly_half_minimum_dimension_stays_native_and_deterministic() {
    let first = project(SOURCE);
    assert!(first.contains(
        "(pad \"4\" smd roundrect (at 4 -2) (size 0.8 0.6) (layers \"F.Cu\" \"F.Paste\" \"F.Mask\") (roundrect_rratio 0) (chamfer_ratio 0.5) (chamfer bottom_left))"
    ), "{first}");
    assert_eq!(first, project(SOURCE));
}
