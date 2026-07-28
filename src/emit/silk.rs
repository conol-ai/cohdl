//! RFC-031: silkscreen graphics, resolved to primitives.
//!
//! The two semantic markers (`pin_1_marker`, `polarity_marker`) are sugar —
//! the RFC is explicit that they "expand to real, checked primitive shapes".
//! That expansion lives here, once, so the `.kicad_mod` and IPC-2581 emitters
//! project the SAME geometry and cannot drift apart.
//!
//! Expansion needs a pad's position AND its size (a marker sits clear of the
//! copper it points at), so it takes the resolved `World` alongside the
//! footprint.

use crate::ast::{
    FootprintDef, PadShape, Pin1Shape, PolarityShape, SilkFill, SilkGraphic, SilkItem,
};
use crate::resolve::World;
use crate::units::UnitValue;

/// Conventional marker geometry. The RFC fixes the pin-1 dot radius (0.2mm)
/// and the standoff (0.3mm); the rest are conventional values chosen here and
/// recorded in docs/compliance-report.md.
/// One millimetre in the lexer's femto-units (10^-15 of the unit's base).
const MM: i128 = 1_000_000_000_000_000;
const STANDOFF: i128 = 3 * MM / 10; // 0.3mm — RFC-031's stated clearance
const DOT_RADIUS: i128 = 2 * MM / 10; // 0.2mm — RFC-031
const DOT_STROKE: i128 = MM / 10; // 0.1mm (a solid dot still needs a stroke)
const TRI_SIDE: i128 = 8 * MM / 10; // 0.8mm equilateral triangle
const BAND_STROKE: i128 = 3 * MM / 10; // 0.3mm — a cathode band is a wide stroke
const ARROW_LEN: i128 = 9 * MM / 10; // 0.9mm

/// A computed femto-mm value as a `Length`, with `text` rendered by the same
/// canonical formatter the emitters use — so a generated coordinate prints
/// exactly like an authored one.
fn femto(v: i128) -> UnitValue {
    UnitValue {
        unit: crate::units::UnitType::Length,
        femto: v,
        text: crate::emit::geom::mm_femto(v),
    }
}

/// A pad placement's centre and half-extents, in femto-mm.
struct PadBox {
    x: i128,
    y: i128,
    hw: i128,
    hh: i128,
}

fn pad_box(world: &World, fp: &FootprintDef, number: &str) -> Option<PadBox> {
    let place = fp.pads.iter().find(|p| p.number.text == number)?;
    let def = world.pads.get(&place.pad.name);
    let (mut hw, mut hh) = (0i128, 0i128);
    if let Some(def) = def {
        match (def.shape.map(|(s, _)| s), def.size.as_slice()) {
            (Some(PadShape::Circle), [d]) => {
                hw = d.femto / 2;
                hh = d.femto / 2;
            }
            (_, [w, h]) => {
                hw = w.femto / 2;
                hh = h.femto / 2;
            }
            _ => {}
        }
        // RFC-025: a rotated placement swaps which axis the pad is long on.
        if place.rotate == 90 || place.rotate == 270 {
            std::mem::swap(&mut hw, &mut hh);
        }
    }
    Some(PadBox {
        x: place.x.femto,
        y: place.y.femto,
        hw,
        hh,
    })
}

/// The outward direction for a marker on `pad`: away from the footprint's pad
/// centroid, snapped to the dominant axis. Snapping keeps the mark square to
/// the package the way a hand-drawn one would be, and makes the result
/// deterministic — no floating-point angle anywhere.
fn outward(fp: &FootprintDef, world: &World, pad: &PadBox) -> (i128, i128) {
    let n = fp.pads.len().max(1) as i128;
    let (mut cx, mut cy) = (0i128, 0i128);
    for p in &fp.pads {
        cx += p.x.femto;
        cy += p.y.femto;
    }
    let _ = world;
    let (dx, dy) = (pad.x - cx / n, pad.y - cy / n);
    if dx.abs() >= dy.abs() {
        (if dx < 0 { -1 } else { 1 }, 0)
    } else {
        (0, if dy < 0 { -1 } else { 1 })
    }
}

/// Every primitive a footprint's `silkscreen` block draws, markers expanded.
/// Unresolvable markers (a pad number that does not exist) contribute nothing;
/// they are reported as E812 at declaration, not here.
pub fn graphics(world: &World, fp: &FootprintDef) -> Vec<SilkGraphic> {
    let Some(block) = &fp.silkscreen else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for item in &block.items {
        match item {
            SilkItem::Graphic(g, _) => out.push(g.clone()),
            SilkItem::Pin1Marker { pad, shape, .. } => {
                let Some(pb) = pad_box(world, fp, &pad.text) else {
                    continue;
                };
                let (ux, uy) = outward(fp, world, &pb);
                // Stand off from the pad's EDGE, not its centre: the RFC's
                // 0.3mm is "a conventional pin-1-marker clearance", and a mark
                // 0.3mm from the pad CENTRE would sit on the copper it marks.
                let edge = if ux != 0 { pb.hw } else { pb.hh };
                let d = edge + STANDOFF;
                let (cx, cy) = (pb.x + ux * (d + DOT_RADIUS), pb.y + uy * (d + DOT_RADIUS));
                match shape {
                    Pin1Shape::Dot => out.push(SilkGraphic::Circle {
                        at: (femto(cx), femto(cy)),
                        radius: femto(DOT_RADIUS),
                        width: femto(DOT_STROKE),
                        fill: SilkFill::Solid,
                    }),
                    Pin1Shape::Triangle => {
                        // Equilateral, apex pointing back at the pad.
                        let h = TRI_SIDE * 866 / 1000; // side * sin(60°)
                        let base = pb.x + ux * (d + h);
                        let basey = pb.y + uy * (d + h);
                        let (apex_x, apex_y) = (pb.x + ux * d, pb.y + uy * d);
                        let (px, py) = (-uy, ux); // perpendicular
                        out.push(SilkGraphic::Polygon {
                            points: vec![
                                (femto(apex_x), femto(apex_y)),
                                (
                                    femto(base + px * TRI_SIDE / 2),
                                    femto(basey + py * TRI_SIDE / 2),
                                ),
                                (
                                    femto(base - px * TRI_SIDE / 2),
                                    femto(basey - py * TRI_SIDE / 2),
                                ),
                            ],
                            fill: SilkFill::Solid,
                        });
                    }
                }
            }
            SilkItem::PolarityMarker {
                cathode_pad, shape, ..
            } => {
                let Some(pb) = pad_box(world, fp, &cathode_pad.text) else {
                    continue;
                };
                // Direction from the OTHER terminals to the cathode — the
                // part's own axis, which is what the mark must be square to.
                let (ux, uy) = outward(fp, world, &pb);
                let edge = if ux != 0 { pb.hw } else { pb.hh };
                let across = if ux != 0 { pb.hh } else { pb.hw };
                let (px, py) = (-uy, ux);
                match shape {
                    PolarityShape::Band => {
                        // A wide stroke just outboard of the cathode land,
                        // perpendicular to the terminal axis.
                        let d = edge + STANDOFF;
                        let (bx, by) = (pb.x + ux * d, pb.y + uy * d);
                        out.push(SilkGraphic::Line {
                            from: (femto(bx + px * across), femto(by + py * across)),
                            to: (femto(bx - px * across), femto(by - py * across)),
                            width: femto(BAND_STROKE),
                        });
                    }
                    PolarityShape::Arrow => {
                        // Filled triangle pointing AWAY from the cathode, i.e.
                        // along conventional current flow, anode -> cathode.
                        let d = edge + STANDOFF;
                        let (tipx, tipy) =
                            (pb.x + ux * (d + ARROW_LEN), pb.y + uy * (d + ARROW_LEN));
                        let (bx, by) = (pb.x + ux * d, pb.y + uy * d);
                        out.push(SilkGraphic::Polygon {
                            points: vec![
                                (femto(tipx), femto(tipy)),
                                (femto(bx + px * across), femto(by + py * across)),
                                (femto(bx - px * across), femto(by - py * across)),
                            ],
                            fill: SilkFill::Solid,
                        });
                    }
                }
            }
        }
    }
    out
}

/// Every silkscreen primitive reduced to a closed polygon ring in FOOTPRINT
/// coordinates (femto-mm), for consumers whose geometry model is polygons.
///
/// IPC-2581 is one: its `Features` element accepts a `StandardShape`, and
/// `Line`/`Arc` belong to the `Simple` group which cannot appear there — but
/// `Contour` can carry an arbitrary polygon. Reducing a stroke to its own
/// outline is faithful rather than lossy: a plotted silkscreen stroke IS a
/// rectangle of the stroke's width with round caps, and the caps are the only
/// thing dropped.
pub fn polygons(world: &World, fp: &FootprintDef) -> Vec<Vec<(i128, i128)>> {
    let mut out = Vec::new();
    for g in graphics(world, fp) {
        match g {
            SilkGraphic::Line { from, to, width } => {
                out.push(stroke_quad(
                    (from.0.femto, from.1.femto),
                    (to.0.femto, to.1.femto),
                    width.femto,
                ));
            }
            SilkGraphic::Polygon { points, .. } => {
                out.push(points.iter().map(|(x, y)| (x.femto, y.femto)).collect());
            }
            SilkGraphic::Circle { at, radius, .. } => {
                // A regular 24-gon: enough that a 0.2mm dot reads as round,
                // and deterministic (one rounding, at construction).
                let (cx, cy, r) = (at.0.femto, at.1.femto, radius.femto as f64);
                out.push(
                    (0..24)
                        .map(|i| {
                            let a = std::f64::consts::TAU * i as f64 / 24.0;
                            (
                                cx + (r * a.cos()).round() as i128,
                                cy + (r * a.sin()).round() as i128,
                            )
                        })
                        .collect(),
                );
            }
            SilkGraphic::Arc {
                at,
                radius,
                start_angle,
                end_angle,
                width,
            } => {
                // Chain of stroke quads along the arc — same reduction as a
                // line, applied to a segmented centreline.
                let (cx, cy, r) = (at.0.femto, at.1.femto, radius.femto as f64);
                let steps = 16;
                let pt = |t: f64| {
                    let a =
                        (start_angle as f64 + (end_angle - start_angle) as f64 * t).to_radians();
                    (
                        cx + (r * a.cos()).round() as i128,
                        cy + (r * a.sin()).round() as i128,
                    )
                };
                for i in 0..steps {
                    let a = pt(i as f64 / steps as f64);
                    let b = pt((i + 1) as f64 / steps as f64);
                    out.push(stroke_quad(a, b, width.femto));
                }
            }
        }
    }
    out
}

/// The four corners of a stroke of `width` from `a` to `b`.
fn stroke_quad(a: (i128, i128), b: (i128, i128), width: i128) -> Vec<(i128, i128)> {
    let (dx, dy) = ((b.0 - a.0) as f64, (b.1 - a.1) as f64);
    let len = (dx * dx + dy * dy).sqrt();
    if len == 0.0 {
        return vec![a, a, a];
    }
    let h = width as f64 / 2.0;
    let (nx, ny) = (
        (-dy / len * h).round() as i128,
        (dx / len * h).round() as i128,
    );
    vec![
        (a.0 + nx, a.1 + ny),
        (b.0 + nx, b.1 + ny),
        (b.0 - nx, b.1 - ny),
        (a.0 - nx, a.1 - ny),
    ]
}
