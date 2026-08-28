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
//!
//! The geometry DERIVATION here (pad plans, body graphics) is shared with
//! the `.kicad_pcb` board emitter: one derivation, two renderings — the
//! same anti-drift shape RFC-031 mandates for silkscreen. Only the
//! s-expression DIALECT differs (this file's terse one-line footprint form
//! vs the board file's multi-line form with uuids and nets).

use crate::ast::{
    MountHoleGeom, MountHolePlating, PadCorner, PadLayer, PadPaste, PadPlating, PadShape,
};
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
pub(crate) fn file_base(fq: &str) -> String {
    fq.replace("::", "-")
}

// ---------------------------------------------------------------------------
// Shared derivation: footprint attribute, body graphics, pad plans
// ---------------------------------------------------------------------------

/// KiCad component attribute. Thermal vias repeated under an SMD exposed-pad
/// number do not turn the package into a through-hole component: a PTH
/// placement makes it through-hole only when that electrical number has no
/// SMD land.
pub(crate) fn footprint_attr(world: &World, fp: &crate::ast::FootprintDef) -> &'static str {
    let smd_numbers: BTreeSet<&str> = fp
        .pads
        .iter()
        .filter(|p| {
            world
                .pads
                .get(&p.pad.name)
                .and_then(|d| d.plating.as_ref())
                .is_some_and(|(pl, _)| *pl == PadPlating::Smd)
        })
        .map(|p| p.number.text.as_str())
        .collect();
    let any_pth = fp.pads.iter().any(|p| {
        !smd_numbers.contains(p.number.text.as_str())
            && world
                .pads
                .get(&p.pad.name)
                .and_then(|d| d.plating.as_ref())
                .is_some_and(|(pl, _)| *pl == PadPlating::PlatedThroughHole)
    });
    if any_pth {
        "through_hole"
    } else {
        "smd"
    }
}

/// The layer a footprint body graphic lives on (front-side authoring frame;
/// a board emitter maps these to B.* for a flipped instance).
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum GfxLayer {
    Silk,
    Courtyard,
    EdgeCuts,
}

/// One footprint body graphic. All coordinates are exact integers at
/// 10^-16 mm (femto × 10) so courtyard corners at odd-femto halves stay
/// exact; widths are plain femto-mm. `fill` maps to this dialect's
/// `solid`/`none` and the board dialect's `yes`/`no`.
pub(crate) enum BodyGraphic {
    Line {
        from: (i128, i128),
        to: (i128, i128),
        width: i128,
        layer: GfxLayer,
    },
    Circle {
        center: (i128, i128),
        end: (i128, i128),
        width: i128,
        fill: bool,
        layer: GfxLayer,
    },
    Arc {
        start: (i128, i128),
        mid: (i128, i128),
        end: (i128, i128),
        width: i128,
        layer: GfxLayer,
    },
    Rect {
        start: (i128, i128),
        end: (i128, i128),
        width: i128,
        fill: bool,
        layer: GfxLayer,
    },
    Poly {
        points: Vec<(i128, i128)>,
        width: i128,
        fill: bool,
        layer: GfxLayer,
    },
}

const HAIRLINE: i128 = 50_000_000_000_000; // 0.05 mm in femto

/// Every drawn body graphic of a footprint, in the emission order both
/// dialects share: courtyard, window (board cutout), silkscreen. RFC-031
/// markers are already expanded to primitives by `emit::silk::graphics`.
pub(crate) fn body_graphics(world: &World, fp: &crate::ast::FootprintDef) -> Vec<BodyGraphic> {
    let mut out = Vec::new();
    let x10 = |v: &crate::units::UnitValue| v.femto * 10;
    if let Some(c) = &fp.courtyard {
        // KiCad courtyards are drawn shapes on F.CrtYd; a rect courtyard
        // maps to fp_rect around the center point; circle to fp_circle.
        match (c.shape.0, c.size.as_slice()) {
            (PadShape::Rect | PadShape::Oval, [w, h]) => out.push(BodyGraphic::Rect {
                start: (x10(&c.at.0) - w.femto * 5, x10(&c.at.1) - h.femto * 5),
                end: (x10(&c.at.0) + w.femto * 5, x10(&c.at.1) + h.femto * 5),
                width: HAIRLINE,
                fill: false,
                layer: GfxLayer::Courtyard,
            }),
            (PadShape::Circle, [d]) => out.push(BodyGraphic::Circle {
                center: (x10(&c.at.0), x10(&c.at.1)),
                end: (x10(&c.at.0) + d.femto * 5, x10(&c.at.1)),
                width: HAIRLINE,
                fill: false,
                layer: GfxLayer::Courtyard,
            }),
            _ => {} // arity errors already reported at declaration check
        }
    }
    // A `window` is a board CUTOUT, so it belongs on Edge.Cuts — the same layer
    // and the same closed outline KiCad's own reverse-mount LED footprints draw
    // for their light aperture. Corners stay square: a fabricator's router bit
    // rounds them to its own radius, which is not ours to invent.
    if let Some(w) = &fp.window {
        match (w.shape.0, w.size.as_slice()) {
            (PadShape::Rect | PadShape::Oval, [ww, wh]) => out.push(BodyGraphic::Rect {
                start: (x10(&w.at.0) - ww.femto * 5, x10(&w.at.1) - wh.femto * 5),
                end: (x10(&w.at.0) + ww.femto * 5, x10(&w.at.1) + wh.femto * 5),
                width: HAIRLINE,
                fill: false,
                layer: GfxLayer::EdgeCuts,
            }),
            (PadShape::Circle, [d]) => out.push(BodyGraphic::Circle {
                center: (x10(&w.at.0), x10(&w.at.1)),
                end: (x10(&w.at.0) + d.femto * 5, x10(&w.at.1)),
                width: HAIRLINE,
                fill: false,
                layer: GfxLayer::EdgeCuts,
            }),
            _ => {} // arity errors already reported at declaration check
        }
    }
    // RFC-031: silkscreen graphics. Markers are already expanded to
    // primitives by `emit::silk`, so this is a straight one-to-one
    // projection. A bottom-side placement mirrors these along with
    // everything else, exactly as pad geometry does.
    for g in crate::emit::silk::graphics(world, fp) {
        use crate::ast::{SilkFill, SilkGraphic};
        let filled = |f: SilkFill| matches!(f, SilkFill::Solid);
        match g {
            SilkGraphic::Line { from, to, width } => out.push(BodyGraphic::Line {
                from: (x10(&from.0), x10(&from.1)),
                to: (x10(&to.0), x10(&to.1)),
                width: width.femto,
                layer: GfxLayer::Silk,
            }),
            SilkGraphic::Circle {
                at,
                radius,
                width,
                fill: f,
            } => out.push(BodyGraphic::Circle {
                // KiCad states a circle by centre + a point ON it.
                center: (x10(&at.0), x10(&at.1)),
                end: ((at.0.femto + radius.femto) * 10, x10(&at.1)),
                width: width.femto,
                fill: filled(f),
                layer: GfxLayer::Silk,
            }),
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
                let pt = |deg: f64| -> (i128, i128) {
                    let r = radius.femto as f64;
                    let (s, c) = deg.to_radians().sin_cos();
                    (
                        (at.0.femto + (r * c).round() as i128) * 10,
                        (at.1.femto + (r * s).round() as i128) * 10,
                    )
                };
                out.push(BodyGraphic::Arc {
                    start: pt(start_angle as f64),
                    mid: pt((start_angle as f64 + end_angle as f64) / 2.0),
                    end: pt(end_angle as f64),
                    width: width.femto,
                    layer: GfxLayer::Silk,
                });
            }
            SilkGraphic::Polygon { points, fill: f } => out.push(BodyGraphic::Poly {
                points: points.iter().map(|(x, y)| (x10(x), x10(y))).collect(),
                width: HAIRLINE,
                fill: filled(f),
                layer: GfxLayer::Silk,
            }),
        }
    }
    out
}

/// A pad's KiCad layer membership, footprint-local. The board emitter swaps
/// front and back for a flipped instance; `CuMask` (`*.Cu`/`*.Mask`) spans
/// both faces and never swaps.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum LayerSet {
    CuMask,
    Front,
    FrontNoPaste,
    Back,
    BackNoPaste,
    PasteFront,
    PasteBack,
}

pub(crate) enum DrillPlan {
    Round(i128),
    Slot(i128, i128),
}

/// What sits inside the pad beyond the flat land: KiCad custom-pad
/// primitives, with their exact point scales.
pub(crate) enum PlanBody {
    /// An ordinary land — no `options`/`primitives`.
    Flat,
    /// RFC-018 annulus: anchor circle + an unfilled ring stroke.
    AnnulusRing { mid_radius: i128, stroke: i128 },
    /// Oversize chamfer as the exact five-vertex polygon; points are stored
    /// at TWICE their femto value (render via [`geom::half_mm_femto`]).
    ChamferPoly { points: Vec<(i128, i128)> },
    /// A segmented-annulus paste sector polygon; plain femto points.
    PastePoly { points: Vec<(i128, i128)> },
}

/// One KiCad pad to emit — an electrical land, an unnumbered paste
/// aperture, or a mount hole — with every field both dialects need.
/// Coordinates and sizes are exact femto-mm integers.
pub(crate) struct PadPlan {
    /// The authored pad number text; empty for apertures and mount holes
    /// (KiCad's own "no net, not electrical" convention).
    pub number: String,
    pub kind: &'static str,   // "smd" | "thru_hole" | "np_thru_hole"
    pub kshape: &'static str, // "rect" | "roundrect" | "circle" | "oval" | "custom"
    pub x: i128,
    pub y: i128,
    /// RFC-025 pad-local rotation, whole degrees.
    pub rotate: u16,
    pub size: (i128, i128),
    pub drill: Option<DrillPlan>,
    pub layers: LayerSet,
    /// Native chamfer: (pre-rendered ratio, corner). The corner is kept
    /// symbolic — a flipped board instance swaps it vertically.
    pub chamfer: Option<(String, PadCorner)>,
    /// Pre-rendered `roundrect_rratio` for `corner_radius`.
    pub corner_radius: Option<String>,
    pub mask_expansion: Option<i128>,
    pub body: PlanBody,
}

/// KiCad clamps its native `chamfer_ratio` to 0.5.  A larger authored cut is
/// therefore represented as a custom pad whose copper primitive is the exact
/// five-vertex land polygon.  Coordinates are local to the pad, so placement
/// rotation and front/back layer selection continue to be handled by the pad
/// itself.  Each coordinate is stored as twice its actual femto-mm value and
/// rendered with [`geom::half_mm_femto`] to retain half-femto vertices.
fn custom_chamfer_points(corner: PadCorner, w: i128, h: i128, cut: i128) -> Vec<(i128, i128)> {
    let left = -w;
    let right = w;
    let top = -h;
    let bottom = h;
    let twice_cut = cut.saturating_mul(2);
    let points = match corner {
        PadCorner::TopLeft => [
            (left + twice_cut, top),
            (right, top),
            (right, bottom),
            (left, bottom),
            (left, top + twice_cut),
        ],
        PadCorner::TopRight => [
            (left, top),
            (right - twice_cut, top),
            (right, top + twice_cut),
            (right, bottom),
            (left, bottom),
        ],
        PadCorner::BottomRight => [
            (left, top),
            (right, top),
            (right, bottom - twice_cut),
            (right - twice_cut, bottom),
            (left, bottom),
        ],
        PadCorner::BottomLeft => [
            (left, top),
            (right, top),
            (right, bottom),
            (left + twice_cut, bottom),
            (left, bottom - twice_cut),
        ],
    };
    points.to_vec()
}

fn segmented_annulus_points(
    outer: i128,
    inner: i128,
    plan: crate::resolve::SegmentedAnnulusSectorPlan,
) -> Vec<(i128, i128)> {
    let outer_r = outer as f64 / 2.0;
    let inner_r = inner as f64 / 2.0;
    let conservative_inner = inner_r / (plan.step_angle / 2.0).cos();
    let mut points = Vec::with_capacity(plan.vertices);
    for i in 0..=plan.segments {
        let angle = plan.start_angle + plan.step_angle * i as f64;
        points.push((outer_r * angle.cos(), outer_r * angle.sin()));
    }
    for i in (0..=plan.segments).rev() {
        let angle = plan.start_angle + plan.step_angle * i as f64;
        points.push((
            conservative_inner * angle.cos(),
            conservative_inner * angle.sin(),
        ));
    }
    points
        .into_iter()
        .map(|(x, y)| (x.round() as i128, y.round() as i128))
        .collect()
}

/// Every KiCad pad a footprint emits, in the shared order: each electrical
/// pad in source placement order (immediately followed by its paste-override
/// aperture pads, RFC-018), then RFC-022/023 mount holes.
pub(crate) fn pad_plans(world: &World, fp: &crate::ast::FootprintDef) -> Vec<PadPlan> {
    let mut out = Vec::new();
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
        let custom_chamfer = pad.chamfer.as_ref().is_some_and(|(_, cut, _)| {
            let min = pad.size.iter().map(|v| v.femto).min().unwrap_or(1);
            cut.femto.saturating_mul(2) > min
        });
        let kshape = match shape {
            PadShape::Annulus => "custom",
            PadShape::Rect if custom_chamfer => "custom",
            PadShape::Rect if pad.chamfer.is_some() || pad.corner_radius.is_some() => "roundrect",
            PadShape::Rect => "rect",
            PadShape::Circle => "circle",
            PadShape::Oval => "oval",
        };
        let size = match pad.size.as_slice() {
            [d] => (d.femto, d.femto),
            [w, h] => (w.femto, h.femto),
            _ => continue, // arity errors already reported
        };
        let layers = match (plating, pad.layer.map(|(l, _)| l), pad.paste.is_some()) {
            (PadPlating::PlatedThroughHole, _, _) => LayerSet::CuMask,
            (PadPlating::Smd, Some(PadLayer::BottomCopper), false) => LayerSet::Back,
            (PadPlating::Smd, Some(PadLayer::BottomCopper), true) => LayerSet::BackNoPaste,
            (PadPlating::Smd, _, false) => LayerSet::Front,
            (PadPlating::Smd, _, true) => LayerSet::FrontNoPaste,
        };
        // A slot uses KiCad's own oval-drill form — the same one RFC-023's
        // non-circular mount holes already emit.
        let drill = pad.drill.as_ref().map(|(d, _)| match d {
            crate::ast::PadDrill::Round(v) => DrillPlan::Round(v.femto),
            crate::ast::PadDrill::Slot(w, l) => DrillPlan::Slot(w.femto, l.femto),
        });
        let chamfer = pad
            .chamfer
            .as_ref()
            .filter(|_| !custom_chamfer)
            .map(|(corner, cut, _)| {
                let min = pad.size.iter().map(|v| v.femto).min().unwrap_or(1).max(1);
                (geom::ratio(cut.femto, min), *corner)
            });
        let corner_radius = pad.corner_radius.as_ref().map(|(radius, _)| {
            let min = pad.size.iter().map(|v| v.femto).min().unwrap_or(1).max(1);
            geom::ratio(radius.femto, min)
        });
        let mask_expansion = pad.mask_expansion.as_ref().map(|(m, _)| m.femto);
        if matches!(shape, PadShape::Annulus) {
            let [outer, inner] = pad.size.as_slice() else {
                continue;
            };
            out.push(PadPlan {
                number: place.number.text.clone(),
                kind: "smd",
                kshape: "custom",
                x: place.x.femto,
                y: place.y.femto,
                rotate: place.rotate,
                size: (0, 0),
                drill: None,
                layers,
                chamfer: None,
                corner_radius: None,
                mask_expansion,
                body: PlanBody::AnnulusRing {
                    mid_radius: (outer.femto + inner.femto) / 4,
                    stroke: (outer.femto - inner.femto) / 2,
                },
            });
        } else if custom_chamfer {
            // The anchor is a centred square wholly contained by the authored
            // polygon.  Its union with the filled primitive therefore adds no
            // copper, while satisfying KiCad's required custom-pad anchor.
            let (corner, cut, _) = pad.chamfer.as_ref().expect("custom chamfer");
            let [pw, ph] = pad.size.as_slice() else {
                continue; // arity errors already reported
            };
            let anchor = pw.femto.min(ph.femto) - cut.femto;
            out.push(PadPlan {
                number: place.number.text.clone(),
                kind,
                kshape,
                x: place.x.femto,
                y: place.y.femto,
                rotate: place.rotate,
                size: (anchor, anchor),
                drill: None,
                layers,
                chamfer: None,
                corner_radius: None,
                mask_expansion,
                body: PlanBody::ChamferPoly {
                    points: custom_chamfer_points(*corner, pw.femto, ph.femto, cut.femto),
                },
            });
        } else {
            out.push(PadPlan {
                number: place.number.text.clone(),
                kind,
                kshape,
                x: place.x.femto,
                y: place.y.femto,
                rotate: place.rotate,
                size,
                drill,
                layers,
                chamfer,
                corner_radius,
                mask_expansion,
                body: PlanBody::Flat,
            });
        }
        let paste_layer = if matches!(pad.layer, Some((PadLayer::BottomCopper, _))) {
            LayerSet::PasteBack
        } else {
            LayerSet::PasteFront
        };
        if let Some((PadPaste::Circle(diameter), _)) = &pad.paste {
            out.push(PadPlan {
                number: String::new(),
                kind: "smd",
                kshape: "circle",
                x: place.x.femto,
                y: place.y.femto,
                rotate: place.rotate,
                size: (diameter.femto, diameter.femto),
                drill: None,
                layers: paste_layer,
                chamfer: None,
                corner_radius: None,
                mask_expansion: None,
                body: PlanBody::Flat,
            });
        } else if let Some((PadPaste::Rect(pw, ph), _)) = &pad.paste {
            out.push(PadPlan {
                number: String::new(),
                kind: "smd",
                kshape: "rect",
                x: place.x.femto,
                y: place.y.femto,
                rotate: place.rotate,
                size: (pw.femto, ph.femto),
                drill: None,
                layers: paste_layer,
                chamfer: None,
                corner_radius: None,
                mask_expansion: None,
                body: PlanBody::Flat,
            });
        } else if let Some((PadPaste::SegmentedAnnulus(values), _)) = &pad.paste {
            let [outer, inner, gap] = values.as_ref();
            let Some(plan) =
                crate::resolve::segmented_annulus_plan(outer.femto, inner.femto, gap.femto)
            else {
                continue;
            };
            for sector in plan.sectors {
                out.push(PadPlan {
                    number: String::new(),
                    kind: "smd",
                    kshape: "custom",
                    x: place.x.femto,
                    y: place.y.femto,
                    rotate: place.rotate,
                    size: (0, 0),
                    drill: None,
                    layers: paste_layer,
                    chamfer: None,
                    corner_radius: None,
                    mask_expansion: None,
                    body: PlanBody::PastePoly {
                        points: segmented_annulus_points(outer.femto, inner.femto, sector),
                    },
                });
            }
        }
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
        let (size, drill) = match &mh.geom {
            MountHoleGeom::Diameter(d) => ((d.femto, d.femto), DrillPlan::Round(d.femto)),
            MountHoleGeom::Size(dims, _) => {
                // Checked to be exactly (w, h) before emit (E810).
                let w = dims.first().map(|v| v.femto).unwrap_or(0);
                let h = dims.get(1).map(|v| v.femto).unwrap_or(0);
                ((w, h), DrillPlan::Slot(w, h))
            }
        };
        out.push(PadPlan {
            number: String::new(),
            kind,
            kshape: mh.shape_or_default().name(),
            x: mh.x.femto,
            y: mh.y.femto,
            rotate: 0,
            size,
            drill: Some(drill),
            layers: LayerSet::CuMask,
            chamfer: None,
            corner_radius: None,
            mask_expansion: None,
            body: PlanBody::Flat,
        });
    }
    out
}

/// The one-line `(xy …)` list for a chamfer polygon (half-scale points).
pub(crate) fn chamfer_xy_list(points: &[(i128, i128)]) -> String {
    points
        .iter()
        .map(|(x, y)| {
            format!(
                "(xy {} {})",
                geom::half_mm_femto(*x),
                geom::half_mm_femto(*y)
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The one-line `(xy …)` list for a paste-sector polygon (femto points).
pub(crate) fn paste_xy_list(points: &[(i128, i128)]) -> String {
    points
        .iter()
        .map(|(x, y)| format!("(xy {} {})", geom::mm_femto(*x), geom::mm_femto(*y)))
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// The .kicad_mod dialect rendering
// ---------------------------------------------------------------------------

fn mod_layer(l: GfxLayer) -> &'static str {
    match l {
        GfxLayer::Silk => "F.SilkS",
        GfxLayer::Courtyard => "F.CrtYd",
        GfxLayer::EdgeCuts => "Edge.Cuts",
    }
}

fn mod_layers(set: LayerSet) -> &'static str {
    match set {
        LayerSet::CuMask => "\"*.Cu\" \"*.Mask\"",
        LayerSet::Front => "\"F.Cu\" \"F.Paste\" \"F.Mask\"",
        LayerSet::FrontNoPaste => "\"F.Cu\" \"F.Mask\"",
        LayerSet::Back => "\"B.Cu\" \"B.Paste\" \"B.Mask\"",
        LayerSet::BackNoPaste => "\"B.Cu\" \"B.Mask\"",
        LayerSet::PasteFront => "\"F.Paste\"",
        LayerSet::PasteBack => "\"B.Paste\"",
    }
}

fn render(world: &World, fq: &str, fp: &crate::ast::FootprintDef) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "(footprint \"{}\"", fq);
    s.push_str("  (version 20240108)\n");
    s.push_str("  (generator \"cohdl\")\n");
    s.push_str("  (layer \"F.Cu\")\n");
    let _ = writeln!(s, "  (attr {})", footprint_attr(world, fp));
    if let Some((x, y, _)) = &fp.silkscreen_ref {
        let _ = writeln!(
            s,
            "  (fp_text reference \"REF**\" (at {} {}) (layer \"F.SilkS\"))",
            geom::mm(x),
            geom::mm(y)
        );
    }
    let g16 = |v: i128| geom::render(v, 16);
    for g in body_graphics(world, fp) {
        let fill = |f: bool| if f { "solid" } else { "none" };
        match g {
            BodyGraphic::Line {
                from,
                to,
                width,
                layer,
            } => {
                let _ = writeln!(
                    s,
                    "  (fp_line (start {} {}) (end {} {}) (layer \"{}\") (stroke (width {}) (type solid)))",
                    g16(from.0),
                    g16(from.1),
                    g16(to.0),
                    g16(to.1),
                    mod_layer(layer),
                    geom::mm_femto(width)
                );
            }
            BodyGraphic::Circle {
                center,
                end,
                width,
                fill: f,
                layer,
            } => {
                let _ = writeln!(
                    s,
                    "  (fp_circle (center {} {}) (end {} {}) (layer \"{}\") (stroke (width {}) (type solid)) (fill {}))",
                    g16(center.0),
                    g16(center.1),
                    g16(end.0),
                    g16(end.1),
                    mod_layer(layer),
                    geom::mm_femto(width),
                    fill(f)
                );
            }
            BodyGraphic::Arc {
                start,
                mid,
                end,
                width,
                layer,
            } => {
                let _ = writeln!(
                    s,
                    "  (fp_arc (start {} {}) (mid {} {}) (end {} {}) (layer \"{}\") (stroke (width {}) (type solid)))",
                    g16(start.0),
                    g16(start.1),
                    g16(mid.0),
                    g16(mid.1),
                    g16(end.0),
                    g16(end.1),
                    mod_layer(layer),
                    geom::mm_femto(width)
                );
            }
            BodyGraphic::Rect {
                start,
                end,
                width,
                fill: f,
                layer,
            } => {
                let _ = writeln!(
                    s,
                    "  (fp_rect (start {} {}) (end {} {}) (layer \"{}\") (stroke (width {}) (type solid)) (fill {}))",
                    g16(start.0),
                    g16(start.1),
                    g16(end.0),
                    g16(end.1),
                    mod_layer(layer),
                    geom::mm_femto(width),
                    fill(f)
                );
            }
            BodyGraphic::Poly {
                points,
                width,
                fill: f,
                layer,
            } => {
                let pts: Vec<String> = points
                    .iter()
                    .map(|(x, y)| format!("(xy {} {})", g16(*x), g16(*y)))
                    .collect();
                let _ = writeln!(
                    s,
                    "  (fp_poly (pts {}) (layer \"{}\") (stroke (width {}) (type solid)) (fill {}))",
                    pts.join(" "),
                    mod_layer(layer),
                    geom::mm_femto(width),
                    fill(f)
                );
            }
        }
    }
    for p in pad_plans(world, fp) {
        // RFC-025: a rotated placement uses KiCad's own 3-argument
        // `(at x y angle)` pad form, size UNCHANGED — the declared rotation is
        // preserved losslessly rather than silently swapping w/h.
        let angle = if p.rotate != 0 {
            format!(" {}", p.rotate)
        } else {
            String::new()
        };
        let drill = p
            .drill
            .as_ref()
            .map(|d| match d {
                DrillPlan::Round(v) => format!(" (drill {})", geom::mm_femto(*v)),
                DrillPlan::Slot(w, l) => {
                    format!(
                        " (drill oval {} {})",
                        geom::mm_femto(*w),
                        geom::mm_femto(*l)
                    )
                }
            })
            .unwrap_or_default();
        let chamfer = p
            .chamfer
            .as_ref()
            .map(|(ratio, corner)| {
                format!(
                    " (roundrect_rratio 0) (chamfer_ratio {}) (chamfer {})",
                    ratio,
                    corner.name()
                )
            })
            .unwrap_or_default();
        let corner_radius = p
            .corner_radius
            .as_ref()
            .map(|ratio| format!(" (roundrect_rratio {})", ratio))
            .unwrap_or_default();
        let mask_expansion = p
            .mask_expansion
            .map(|m| format!(" (solder_mask_margin {})", geom::mm_femto(m)))
            .unwrap_or_default();
        match &p.body {
            PlanBody::AnnulusRing { mid_radius, stroke } => {
                let _ = writeln!(
                    s,
                    "  (pad \"{}\" smd custom (at {} {}{}) (size 0 0) (layers {}){} (options (clearance outline) (anchor circle)) (primitives (gr_circle (center 0 0) (end {} 0) (width {}) (fill none))))",
                    p.number,
                    geom::mm_femto(p.x),
                    geom::mm_femto(p.y),
                    angle,
                    mod_layers(p.layers),
                    mask_expansion,
                    geom::mm_femto(*mid_radius),
                    geom::mm_femto(*stroke)
                );
            }
            PlanBody::ChamferPoly { points } => {
                let _ = writeln!(
                    s,
                    "  (pad \"{}\" {} {} (at {} {}{}) (size {} {}) (layers {}){} (options (clearance outline) (anchor rect)) (primitives (gr_poly (pts {}) (width 0) (fill yes))))",
                    p.number,
                    p.kind,
                    p.kshape,
                    geom::mm_femto(p.x),
                    geom::mm_femto(p.y),
                    angle,
                    geom::mm_femto(p.size.0),
                    geom::mm_femto(p.size.1),
                    mod_layers(p.layers),
                    mask_expansion,
                    chamfer_xy_list(points)
                );
            }
            PlanBody::PastePoly { points } => {
                let _ = writeln!(
                    s,
                    "  (pad \"{}\" {} {} (at {} {}{}) (size 0 0) (layers {}) (options (clearance outline) (anchor circle)) (primitives (gr_poly (pts {}) (width 0) (fill yes))))",
                    p.number,
                    p.kind,
                    p.kshape,
                    geom::mm_femto(p.x),
                    geom::mm_femto(p.y),
                    angle,
                    mod_layers(p.layers),
                    paste_xy_list(points)
                );
            }
            PlanBody::Flat => {
                let _ = writeln!(
                    s,
                    "  (pad \"{}\" {} {} (at {} {}{}) (size {} {}){} (layers {}){}{}{})",
                    p.number,
                    p.kind,
                    p.kshape,
                    geom::mm_femto(p.x),
                    geom::mm_femto(p.y),
                    angle,
                    geom::mm_femto(p.size.0),
                    geom::mm_femto(p.size.1),
                    drill,
                    mod_layers(p.layers),
                    chamfer,
                    corner_radius,
                    mask_expansion
                );
            }
        }
    }
    s.push_str(")\n");
    s
}
