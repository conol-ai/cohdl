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

use crate::ast::{MountHolePlating, PadLayer, PadPlating, PadShape};
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
        let drill = pad
            .drill
            .as_ref()
            .map(|(v, _)| format!(" (drill {})", geom::mm(v)))
            .unwrap_or_default();
        let _ = writeln!(
            s,
            "  (pad \"{}\" {} {} (at {} {}) (size {} {}){} (layers {}))",
            place.number.text,
            kind,
            kshape,
            geom::mm(&place.x),
            geom::mm(&place.y),
            w,
            h,
            drill,
            layers
        );
    }
    // RFC-022 mechanical locating holes — projected as KiCad's own hole pad
    // types with an empty pad number (no net): non_plated → np_thru_hole,
    // plated → an ordinary thru_hole with the pad ring sized to the drill.
    for mh in &fp.mount_holes {
        let kind = match mh.plating {
            MountHolePlating::NonPlated => "np_thru_hole",
            MountHolePlating::Plated => "thru_hole",
        };
        let d = geom::mm(&mh.diameter);
        let _ = writeln!(
            s,
            "  (pad \"\" {} circle (at {} {}) (size {} {}) (drill {}) (layers \"*.Cu\" \"*.Mask\"))",
            kind,
            geom::mm(&mh.x),
            geom::mm(&mh.y),
            d,
            d,
            d
        );
    }
    s.push_str(")\n");
    s
}
