//! RFC-018: `.kicad_mod` projection of footprints with authored content.
//!
//! One file per footprint that (a) is referenced by a bound part in the
//! built design — through the part's primary OR alt AVL entries, since a
//! fab may build either source — and (b) carries real body content
//! (RFC-017 stage-one placeholders emit nothing — there is no geometry to
//! project; a courtyard-only footprint IS projected, its authored geometry
//! must not silently vanish).
//!
//! The emitted subset targets KiCad's s-expression footprint format
//! (version-stamped, generator "cohdl"): per-pad shape/size/position/
//! layers/drill, an optional courtyard rectangle, and an optional
//! silkscreen reference-designator anchor. Deterministic: footprints in
//! fq-name order, pads in placement (source) order; all dimensions render
//! canonically from the lexer's exact femto integers (emit::geom — KiCad's
//! native unit is mm, no conversion, no floats).

use crate::ast::{MountHoleGeom, MountHolePlating, PadLayer, PadPlating, PadShape};
use crate::check::footprints::is_placeholder;
use crate::emit::geom;
use crate::ir::DesignIr;
use crate::resolve::World;
use std::collections::BTreeSet;
use std::fmt::Write as _;

/// The content-bearing footprints used by the design: (fq name, file base
/// name, rendered `.kicad_mod` content), in fq order.
pub fn emit_kicad_mods(world: &World, ir: &DesignIr) -> Vec<(String, String, String)> {
    let mut used: BTreeSet<&str> = BTreeSet::new();
    for inst in ir.instances.values() {
        if let Some(part) = inst.part.as_ref().and_then(|p| world.parts.get(p)) {
            for entry in std::iter::once(&part.primary).chain(part.alts.iter()) {
                if let Some(fp) = &entry.footprint {
                    used.insert(&fp.name);
                }
            }
        }
    }
    let mut out = Vec::new();
    for fq in used {
        let Some(fp) = world.footprints.get(fq) else {
            continue;
        };
        if is_placeholder(fp) {
            continue; // stage-one placeholder — nothing to project
        }
        out.push((fq.to_string(), file_base(fq), render(world, fq, fp)));
    }
    out
}

/// The file base name: the fq path with `::` → `-`. Injective by
/// construction — `-` cannot appear in identifiers or module names, so two
/// distinct fq paths can never collide (`__` could: `a__b::c` vs
/// `a::b__c`).
fn file_base(fq: &str) -> String {
    fq.replace("::", "-")
}

fn render(world: &World, fq: &str, fp: &crate::ast::FootprintDef) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "(footprint \"{}\"", fq);
    s.push_str("  (version 20240108)\n");
    s.push_str("  (generator \"cohdl\")\n");
    s.push_str("  (layer \"F.Cu\")\n");
    // Attribute: smd when every resolved pad is smd; through_hole if any PTH.
    let any_pth = fp.pads.iter().any(|p| {
        world
            .pads
            .get(&p.pad.name)
            .and_then(|d| d.plating.as_ref())
            .is_some_and(|(pl, _)| *pl == PadPlating::PlatedThroughHole)
    });
    let _ = writeln!(
        s,
        "  (attr {})",
        if any_pth { "through_hole" } else { "smd" }
    );
    if let Some((x, y, _)) = &fp.silkscreen_ref {
        let _ = writeln!(
            s,
            "  (fp_text reference \"REF**\" (at {} {}) (layer \"F.SilkS\"))",
            geom::mm(x),
            geom::mm(y)
        );
    }
    if let Some(c) = &fp.courtyard {
        // KiCad courtyards are drawn shapes on F.CrtYd; a rect courtyard
        // maps to fp_rect around the center point; circle to fp_circle.
        match (c.shape.0, c.size.as_slice()) {
            (PadShape::Rect | PadShape::Oval, [w, h]) => {
                let _ = writeln!(
                    s,
                    "  (fp_rect (start {} {}) (end {} {}) (layer \"F.CrtYd\") (stroke (width 0.05) (type solid)) (fill none))",
                    geom::corner_lo(&c.at.0, w),
                    geom::corner_lo(&c.at.1, h),
                    geom::corner_hi(&c.at.0, w),
                    geom::corner_hi(&c.at.1, h)
                );
            }
            (PadShape::Circle, [d]) => {
                let _ = writeln!(
                    s,
                    "  (fp_circle (center {} {}) (end {} {}) (layer \"F.CrtYd\") (stroke (width 0.05) (type solid)) (fill none))",
                    geom::mm(&c.at.0),
                    geom::mm(&c.at.1),
                    geom::corner_hi(&c.at.0, d),
                    geom::mm(&c.at.1)
                );
            }
            _ => {} // arity errors already reported at declaration check
        }
    }
    // A `window` is a board CUTOUT, so it belongs on Edge.Cuts — the same layer
    // and the same closed outline KiCad's own reverse-mount LED footprints draw
    // for their light aperture. Corners stay square: a fabricator's router bit
    // rounds them to its own radius, which is not ours to invent.
    if let Some(w) = &fp.window {
        match (w.shape.0, w.size.as_slice()) {
            (PadShape::Rect | PadShape::Oval, [ww, wh]) => {
                let _ = writeln!(
                    s,
                    "  (fp_rect (start {} {}) (end {} {}) (layer \"Edge.Cuts\") (stroke (width 0.05) (type solid)) (fill none))",
                    geom::corner_lo(&w.at.0, ww),
                    geom::corner_lo(&w.at.1, wh),
                    geom::corner_hi(&w.at.0, ww),
                    geom::corner_hi(&w.at.1, wh)
                );
            }
            (PadShape::Circle, [d]) => {
                let _ = writeln!(
                    s,
                    "  (fp_circle (center {} {}) (end {} {}) (layer \"Edge.Cuts\") (stroke (width 0.05) (type solid)) (fill none))",
                    geom::mm(&w.at.0),
                    geom::mm(&w.at.1),
                    geom::corner_hi(&w.at.0, d),
                    geom::mm(&w.at.1)
                );
            }
            _ => {} // arity errors already reported at declaration check
        }
    }
    // RFC-031: silkscreen graphics onto KiCad's own native graphic items on
    // F.SilkS. Markers are already expanded to primitives by `emit::silk`, so
    // this is a straight one-to-one projection. A bottom-side placement
    // mirrors these along with everything else, exactly as pad geometry does.
    for g in crate::emit::silk::graphics(world, fp) {
        use crate::ast::{SilkFill, SilkGraphic};
        let fill = |f: SilkFill| match f {
            SilkFill::Solid => "solid",
            SilkFill::None => "none",
        };
        match g {
            SilkGraphic::Line { from, to, width } => {
                let _ = writeln!(
                    s,
                    "  (fp_line (start {} {}) (end {} {}) (layer \"F.SilkS\") (stroke (width {}) (type solid)))",
                    geom::mm(&from.0),
                    geom::mm(&from.1),
                    geom::mm(&to.0),
                    geom::mm(&to.1),
                    geom::mm(&width)
                );
            }
            SilkGraphic::Circle {
                at,
                radius,
                width,
                fill: f,
            } => {
                // KiCad states a circle by centre + a point ON it.
                let _ = writeln!(
                    s,
                    "  (fp_circle (center {} {}) (end {} {}) (layer \"F.SilkS\") (stroke (width {}) (type solid)) (fill {}))",
                    geom::mm(&at.0),
                    geom::mm(&at.1),
                    geom::mm_femto(at.0.femto + radius.femto),
                    geom::mm(&at.1),
                    geom::mm(&width),
                    fill(f)
                );
            }
            SilkGraphic::Arc {
                at,
                radius,
                start_angle,
                end_angle,
                width,
            } => {
                // KiCad states an arc by start/mid/end points, so the three are
                // computed here from centre+radius+angles. Integer-degree input
                // (RFC-031) keeps this the only float arithmetic in the path,
                // rounded once to femto — the same discipline RFC-020's arc
                // centres already follow.
                let pt = |deg: f64| -> (String, String) {
                    let r = radius.femto as f64;
                    let (s, c) = deg.to_radians().sin_cos();
                    (
                        geom::mm_femto(at.0.femto + (r * c).round() as i128),
                        geom::mm_femto(at.1.femto + (r * s).round() as i128),
                    )
                };
                let (sx, sy) = pt(start_angle as f64);
                let (mx, my) = pt((start_angle as f64 + end_angle as f64) / 2.0);
                let (ex, ey) = pt(end_angle as f64);
                let _ = writeln!(
                    s,
                    "  (fp_arc (start {} {}) (mid {} {}) (end {} {}) (layer \"F.SilkS\") (stroke (width {}) (type solid)))",
                    sx, sy, mx, my, ex, ey,
                    geom::mm(&width)
                );
            }
            SilkGraphic::Polygon { points, fill: f } => {
                let pts: Vec<String> = points
                    .iter()
                    .map(|(x, y)| format!("(xy {} {})", geom::mm(x), geom::mm(y)))
                    .collect();
                let _ = writeln!(
                    s,
                    "  (fp_poly (pts {}) (layer \"F.SilkS\") (stroke (width 0.05) (type solid)) (fill {}))",
                    pts.join(" "),
                    fill(f)
                );
            }
        }
    }
    for place in &fp.pads {
        let Some(pad) = world.pads.get(&place.pad.name) else {
            continue; // unresolved: reported at declaration check
        };
        let (Some((shape, _)), Some((plating, _))) = (&pad.shape, &pad.plating) else {
            continue;
        };
        let kind = match plating {
            PadPlating::Smd => "smd",
            PadPlating::PlatedThroughHole => "thru_hole",
        };
        let kshape = match shape {
            PadShape::Rect => "rect",
            PadShape::Circle => "circle",
            PadShape::Oval => "oval",
        };
        let (w, h) = match pad.size.as_slice() {
            [d] => (geom::mm(d), geom::mm(d)),
            [w, h] => (geom::mm(w), geom::mm(h)),
            _ => continue, // arity errors already reported
        };
        let layers = match (plating, pad.layer.map(|(l, _)| l)) {
            (PadPlating::PlatedThroughHole, _) => "\"*.Cu\" \"*.Mask\"",
            (PadPlating::Smd, Some(PadLayer::BottomCopper)) => "\"B.Cu\" \"B.Paste\" \"B.Mask\"",
            _ => "\"F.Cu\" \"F.Paste\" \"F.Mask\"",
        };
        // A slot uses KiCad's own oval-drill form — the same one RFC-023's
        // non-circular mount holes already emit.
        let drill = pad
            .drill
            .as_ref()
            .map(|(d, _)| match d {
                crate::ast::PadDrill::Round(v) => format!(" (drill {})", geom::mm(v)),
                crate::ast::PadDrill::Slot(w, l) => {
                    format!(" (drill oval {} {})", geom::mm(w), geom::mm(l))
                }
            })
            .unwrap_or_default();
        // RFC-025: a rotated placement uses KiCad's own 3-argument
        // `(at x y angle)` pad form, size UNCHANGED — the declared rotation is
        // preserved losslessly rather than silently swapping w/h.
        let angle = if place.rotate != 0 {
            format!(" {}", place.rotate)
        } else {
            String::new()
        };
        let _ = writeln!(
            s,
            "  (pad \"{}\" {} {} (at {} {}{}) (size {} {}){} (layers {}))",
            place.number.text,
            kind,
            kshape,
            geom::mm(&place.x),
            geom::mm(&place.y),
            angle,
            w,
            h,
            drill,
            layers
        );
    }
    // RFC-022 mechanical locating holes — projected as KiCad's own hole pad
    // types with an empty pad number (no net): non_plated → np_thru_hole,
    // plated → an ordinary thru_hole with the pad ring sized to the drill.
    // RFC-023: a rect/oval hole reuses this same path — KiCad's hole pads are
    // not restricted to round shapes. Its DRILL vocabulary is, though (round or
    // oval only), so a non-circular hole gets an oval drill spanning (w, h):
    // the manufacturable slot that seats a rectangular leg.
    for mh in &fp.mount_holes {
        let kind = match mh.plating {
            MountHolePlating::NonPlated => "np_thru_hole",
            MountHolePlating::Plated => "thru_hole",
        };
        let (w, h, drill) = match &mh.geom {
            MountHoleGeom::Diameter(d) => {
                let d = geom::mm(d);
                (d.clone(), d.clone(), format!("(drill {})", d))
            }
            MountHoleGeom::Size(dims, _) => {
                // Checked to be exactly (w, h) before emit (E810).
                let w = dims.first().map(geom::mm).unwrap_or_else(|| "0".into());
                let h = dims.get(1).map(geom::mm).unwrap_or_else(|| "0".into());
                let drill = format!("(drill oval {} {})", w, h);
                (w, h, drill)
            }
        };
        let _ = writeln!(
            s,
            "  (pad \"\" {} {} (at {} {}) (size {} {}) {} (layers \"*.Cu\" \"*.Mask\"))",
            kind,
            mh.shape_or_default().name(),
            geom::mm(&mh.x),
            geom::mm(&mh.y),
            w,
            h,
            drill
        );
    }
    s.push_str(")\n");
    s
}
