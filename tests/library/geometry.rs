//! Fabrication geometry audit, independent of part declarations.
//!
//! All dimensions are integer femtometres. Rotated rectangles (including
//! rounded rectangles, conservatively enclosed by their rectangles) use SAT;
//! circular pads use their exact radius, and a circle-versus-rectangle test
//! also tries the axes from the circle centre to every rectangle corner so a
//! pad that merely hugs a corner is not reported. Oval pads use their
//! enclosing rectangle, so no real copper can escape the audit.
//!
//! Same-pin copper is exempt two ways: pads sharing one pad number (split EP
//! islands), and — only when every pub part entry that binds the footprint
//! maps both numbers to the same device pin — two numbers of one pin
//! (USB-C's doubled GND, micro-B's shield tabs). A footprint no part
//! references is held to distinct numbers.

use cohdl::ast::{Courtyard, FootprintDef, PadDef, PadLayer, PadShape};
use cohdl::resolve::World;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Clone, Copy)]
struct Bounds {
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
}

impl Bounds {
    fn corners(self) -> [Point; 4] {
        [
            Point {
                x: self.left,
                y: self.top,
            },
            Point {
                x: self.right,
                y: self.top,
            },
            Point {
                x: self.right,
                y: self.bottom,
            },
            Point {
                x: self.left,
                y: self.bottom,
            },
        ]
    }
}

#[derive(Clone, Copy)]
struct Geometry {
    center: Point,
    bounds: Bounds,
    angle: f64,
    radius: Option<f64>,
}

impl Geometry {
    fn corners(self) -> [Point; 4] {
        self.bounds.corners().map(|point| {
            let (sin, cos) = self.angle.sin_cos();
            Point {
                x: self.center.x + point.x * cos - point.y * sin,
                y: self.center.y + point.x * sin + point.y * cos,
            }
        })
    }

    fn projection_radius(self, axis: Point) -> f64 {
        self.radius.unwrap_or_else(|| {
            let (sin, cos) = self.angle.sin_cos();
            let ux = Point { x: cos, y: sin };
            let uy = Point { x: -sin, y: cos };
            (self.bounds.right - self.bounds.left) * 0.5 * dot(ux, axis).abs()
                + (self.bounds.bottom - self.bounds.top) * 0.5 * dot(uy, axis).abs()
        })
    }

    fn face_normals(self) -> [Point; 2] {
        let (sin, cos) = self.angle.sin_cos();
        [Point { x: cos, y: sin }, Point { x: -sin, y: cos }]
    }
}

fn dot(a: Point, b: Point) -> f64 {
    a.x * b.x + a.y * b.y
}

fn centre_distance(a: Geometry, b: Geometry, axis: Point) -> f64 {
    ((b.center.x - a.center.x) * axis.x + (b.center.y - a.center.y) * axis.y).abs()
}

/// A positive gap on any candidate axis proves disjointness. Touching is not
/// overlap. Two circles need only their centre axis; circle versus rectangle
/// additionally tries each corner-to-centre axis, which separates a circle
/// that merely hugs a corner; rectangle versus rectangle uses the four face
/// normals of the two oriented rectangles.
fn overlaps(a: Geometry, b: Geometry) -> bool {
    let axes: Vec<Point> = match (a.radius, b.radius) {
        (Some(_), Some(_)) => {
            let dx = b.center.x - a.center.x;
            let dy = b.center.y - a.center.y;
            let length = dx.hypot(dy);
            if length == 0.0 {
                return true;
            }
            vec![Point {
                x: dx / length,
                y: dy / length,
            }]
        }
        _ => {
            let mut axes = Vec::with_capacity(6);
            for shape in [a, b] {
                axes.extend(shape.face_normals());
            }
            if let Some((rect, circle)) = pick_rect_and_circle(a, b) {
                for corner in rect.corners() {
                    let dx = circle.center.x - corner.x;
                    let dy = circle.center.y - corner.y;
                    let length = dx.hypot(dy);
                    if length == 0.0 {
                        return true;
                    }
                    axes.push(Point {
                        x: dx / length,
                        y: dy / length,
                    });
                }
            }
            axes
        }
    };
    !axes.into_iter().any(|axis| {
        centre_distance(a, b, axis) >= a.projection_radius(axis) + b.projection_radius(axis)
    })
}

fn pick_rect_and_circle(a: Geometry, b: Geometry) -> Option<(Geometry, Geometry)> {
    match (a.radius, b.radius) {
        (None, Some(_)) => Some((a, b)),
        (Some(_), None) => Some((b, a)),
        _ => None,
    }
}

fn geometry(place: &cohdl::ast::PadPlace, pad: &PadDef) -> Geometry {
    let [width, height] = dimensions(pad);
    Geometry {
        center: Point {
            x: place.x.femto as f64,
            y: place.y.femto as f64,
        },
        bounds: Bounds {
            left: -(width as f64) * 0.5,
            right: width as f64 * 0.5,
            top: -(height as f64) * 0.5,
            bottom: height as f64 * 0.5,
        },
        angle: (place.rotate as f64).to_radians(),
        radius: (pad.shape.map(|(shape, _)| shape) == Some(PadShape::Circle))
            .then_some(width as f64 * 0.5),
    }
}

fn dimensions(pad: &PadDef) -> [i128; 2] {
    match pad.size.as_slice() {
        [diameter] => [diameter.femto, diameter.femto],
        [width, height] => [width.femto, height.femto],
        _ => panic!("validated pad geometry has invalid dimensions"),
    }
}

fn mm(femto: f64) -> f64 {
    femto / 1e15
}

/// The largest distance (femto) by which `shape` escapes `courtyard`; `None`
/// when the shape is contained. Rectangular courtyards measure each corner's
/// overshoot along x and y; a circular pad measures centre-plus-radius.
fn courtyard_excess(courtyard: &Courtyard, shape: Geometry) -> Option<f64> {
    let cx = courtyard.at.0.femto as f64;
    let cy = courtyard.at.1.femto as f64;
    let excess = match courtyard.shape.0 {
        PadShape::Circle => {
            let [radius] = courtyard.size.as_slice() else {
                panic!("validated circular courtyard has invalid dimensions");
            };
            let radius = radius.femto as f64 * 0.5;
            match shape.radius {
                Some(pad_radius) => {
                    (shape.center.x - cx).hypot(shape.center.y - cy) + pad_radius - radius
                }
                None => shape
                    .corners()
                    .into_iter()
                    .map(|point| (point.x - cx).hypot(point.y - cy) - radius)
                    .fold(f64::NEG_INFINITY, f64::max),
            }
        }
        PadShape::Rect => {
            let [width, height] = courtyard.size.as_slice() else {
                panic!("validated rectangular courtyard has invalid dimensions");
            };
            let half_w = width.femto as f64 * 0.5;
            let half_h = height.femto as f64 * 0.5;
            match shape.radius {
                Some(radius) => {
                    let over_x = (shape.center.x - cx).abs() + radius - half_w;
                    let over_y = (shape.center.y - cy).abs() + radius - half_h;
                    over_x.max(over_y)
                }
                None => shape
                    .corners()
                    .into_iter()
                    .map(|point| ((point.x - cx).abs() - half_w).max((point.y - cy).abs() - half_h))
                    .fold(f64::NEG_INFINITY, f64::max),
            }
        }
        shape => panic!("unexpected courtyard shape {shape:?}"),
    };
    (excess > 0.0).then_some(excess)
}

/// `(package, footprint, version)`: defects this audit exposes and defers to
/// A1b/A1c. Every reason states the failing check, the pads involved and the
/// measured excess — this table is the dispatch list for those cards.
const QUARANTINE: &[(&str, &str, &str, &str)] = &[];

/// `(package, footprint, body width, body height, source)` in millimetres:
/// courtyards that A1b/A1c redrew from the pad envelope. Each must also
/// contain the package body, centred on the footprint origin, plus the
/// 0.25 mm IPC-7351B nominal-density clearance. Body sizes are maxima where
/// the source gives a tolerance.
const REPAIRED_COURTYARD_BODIES: &[(&str, &str, f64, f64, &str)] = &[
    (
        "qfn",
        "ESPRESSIF_QFN28_0P4_4",
        4.0,
        4.0,
        "source decal QFN28_0P4_4",
    ),
    (
        "qfn",
        "ESPRESSIF_QFN48EB_0P35_5",
        5.0,
        5.0,
        "source decal QFN48EB_0P35_5",
    ),
    (
        "qfn",
        "ESPRESSIF_QFN48E_0P4_6",
        6.0,
        6.0,
        "source decal QFN48E_0P4_6",
    ),
    (
        "@espressif/esp32",
        "FP_ESP32_PICO_D4",
        7.0,
        7.0,
        "source decal QFN48_0P5_7",
    ),
    (
        "@espressif/esp32",
        "FP_ESP32_PICO_V3",
        7.0,
        7.0,
        "source decal QFN48_0P5_7",
    ),
    (
        "@espressif/esp32",
        "FP_ESP32_PICO_V3_02",
        7.0,
        7.0,
        "source decal QFN48_0P5_7",
    ),
    (
        "audio-amp",
        "FP_NS4168_ESOP8",
        4.0,
        5.1,
        "NS4168 V1.2 §11.1, E and D max",
    ),
];

const COURTYARD_CLEARANCE_MM: f64 = 0.25;

/// The largest distance (femto) by which the body plus clearance escapes the
/// footprint's courtyard; `None` when it is contained.
fn body_excess(footprint: &FootprintDef, width_mm: f64, height_mm: f64) -> Option<f64> {
    let courtyard = footprint.courtyard.as_ref()?;
    let half_w = (width_mm * 0.5 + COURTYARD_CLEARANCE_MM) * 1e15;
    let half_h = (height_mm * 0.5 + COURTYARD_CLEARANCE_MM) * 1e15;
    let body = Geometry {
        center: Point { x: 0.0, y: 0.0 },
        bounds: Bounds {
            left: -half_w,
            right: half_w,
            top: -half_h,
            bottom: half_h,
        },
        angle: 0.0,
        radius: None,
    };
    courtyard_excess(courtyard, body)
}

/// One pub part entry that binds a footprint: (part, device, variant).
type Binding<'a> = (&'a str, &'a str, Option<&'a str>);

fn bindings_for<'a>(world: &'a World, fp_name: &str) -> Vec<Binding<'a>> {
    world
        .parts
        .iter()
        .filter(|(name, _)| world.symbols.get(*name).is_some_and(|symbol| symbol.is_pub))
        .filter_map(|(name, part)| {
            let binds = std::iter::once(&part.primary)
                .chain(part.alts.iter())
                .any(|entry| {
                    entry
                        .footprint
                        .as_ref()
                        .is_some_and(|fp| fp.name == fp_name)
                });
            binds.then(|| {
                (
                    name.as_str(),
                    part.device.name.name.as_str(),
                    part.device.variant.as_ref().map(|v| v.name.as_str()),
                )
            })
        })
        .collect()
}

/// Two pad numbers share a pin only when every binding of the footprint maps
/// them to one device pin. Unbound numbers (mount-hole-style pads aside) are
/// their own pin. With no binding at all, distinct numbers are distinct pins.
fn same_pin(world: &World, bindings: &[Binding], a: &str, b: &str) -> bool {
    if bindings.is_empty() {
        return false;
    }
    bindings.iter().all(|(_, device, variant)| {
        let pin_of: BTreeMap<&str, &str> = world.devices[*device]
            .pins_for(*variant)
            .iter()
            .flat_map(|pin| {
                pin.numbers
                    .iter()
                    .map(move |number| (number.text.as_str(), pin.name.name.as_str()))
            })
            .collect();
        pin_of.get(a).copied().unwrap_or(a) == pin_of.get(b).copied().unwrap_or(b)
    })
}

fn own_declarations<'a>(
    world: &'a World,
    package: &'a str,
) -> impl Iterator<Item = (&'a String, &'a FootprintDef)> {
    let prefix = format!("{}::", cohdl::pipeline::package_root(package));
    world
        .footprints
        .iter()
        .filter(move |(name, _)| name.starts_with(&prefix))
}

pub fn audit_package(package: &str, version: &str, world: &World) {
    let declared: BTreeSet<_> = own_declarations(world, package)
        .map(|(name, _)| name.rsplit("::").next().unwrap_or(name))
        .collect();
    let stale: Vec<_> = REPAIRED_COURTYARD_BODIES
        .iter()
        .filter(|(p, f, ..)| *p == package && !declared.contains(f))
        .map(|(_, f, ..)| *f)
        .collect();
    assert!(
        stale.is_empty(),
        "repaired-courtyard entries for `{package}` name no footprint: {stale:?}"
    );
    let mut unexpected = Vec::new();
    let mut used = BTreeSet::new();
    for (name, footprint) in own_declarations(world, package) {
        let mut failures = Vec::new();
        let courtyard = footprint.courtyard.as_ref();
        for place in &footprint.pads {
            let pad = &world.pads[&place.pad.name];
            let shape = geometry(place, pad);
            if let Some(courtyard) = courtyard {
                if let Some(excess) = courtyard_excess(courtyard, shape) {
                    failures.push(format!(
                        "courtyard overflow: pad {} corner exceeds the {} courtyard by {:.3} mm",
                        place.number.text,
                        courtyard_shape_name(courtyard),
                        mm(excess)
                    ));
                }
            }
        }
        let short = name.rsplit("::").next().unwrap_or(name);
        for (_, _, width, height, source) in REPAIRED_COURTYARD_BODIES
            .iter()
            .filter(|(p, f, ..)| *p == package && *f == short)
        {
            if footprint.courtyard.is_none() {
                failures.push(format!("repaired courtyard missing ({source})"));
            } else if let Some(excess) = body_excess(footprint, *width, *height) {
                failures.push(format!(
                    "courtyard does not contain the {width} x {height} mm body ({source}) plus {COURTYARD_CLEARANCE_MM} mm: short by {:.3} mm",
                    mm(excess)
                ));
            }
        }
        let mut bindings: Option<Vec<Binding>> = None;
        for left in 0..footprint.pads.len() {
            for right in (left + 1)..footprint.pads.len() {
                let a = &footprint.pads[left];
                let b = &footprint.pads[right];
                if a.number.text == b.number.text {
                    continue;
                }
                let a_pad = &world.pads[&a.pad.name];
                let b_pad = &world.pads[&b.pad.name];
                let a_layer = a_pad.layer.map(|(layer, _)| layer);
                let b_layer = b_pad.layer.map(|(layer, _)| layer);
                let shared_layer = a_layer == b_layer
                    || a_layer == Some(PadLayer::ThroughAll)
                    || b_layer == Some(PadLayer::ThroughAll);
                if !shared_layer || !overlaps(geometry(a, a_pad), geometry(b, b_pad)) {
                    continue;
                }
                let bindings = bindings.get_or_insert_with(|| bindings_for(world, name));
                if same_pin(
                    world,
                    bindings,
                    a.number.text.as_str(),
                    b.number.text.as_str(),
                ) {
                    continue;
                }
                failures.push(format!(
                    "pad overlap: pads {} and {} overlap",
                    a.number.text, b.number.text
                ));
            }
        }
        if !failures.is_empty() {
            let key = (package, name.as_str(), version);
            if let Some((_, _, _, reason)) =
                QUARANTINE.iter().find(|(p, f, v, _)| (*p, *f, *v) == key)
            {
                used.insert(key);
                eprintln!("A1a quarantined {package} {name} {version}: {reason}");
            } else {
                unexpected.extend(
                    failures
                        .into_iter()
                        .map(|failure| format!("{package} {name} {version}: {failure}")),
                );
            }
        }
    }
    assert!(
        unexpected.is_empty(),
        "unquarantined footprint defects:\n{}",
        unexpected.join("\n")
    );
    let listed: BTreeSet<_> = QUARANTINE
        .iter()
        .filter(|(p, _, _, _)| *p == package)
        .map(|(p, f, v, _)| (*p, *f, *v))
        .collect();
    assert!(
        used == listed,
        "quarantine entries for `{package}` no longer match the audit: unused {:?}, missing {:?}",
        listed.difference(&used).collect::<Vec<_>>(),
        used.difference(&listed).collect::<Vec<_>>()
    );
}

fn courtyard_shape_name(courtyard: &Courtyard) -> &'static str {
    match courtyard.shape.0 {
        PadShape::Circle => "circular",
        PadShape::Rect => "rectangular",
        shape => panic!("unexpected courtyard shape {shape:?}"),
    }
}

pub fn assert_quarantine_is_current() {
    let expected: BTreeSet<_> = QUARANTINE
        .iter()
        .map(|(package, footprint, version, _)| (*package, *footprint, *version))
        .collect();
    assert_eq!(
        expected.len(),
        QUARANTINE.len(),
        "duplicate quarantine keys"
    );
}

/// Audit public part order codes: every entry must carry a non-empty MPN and
/// manufacturer. A defect is charged to the footprint's quarantine key.
pub fn audit_parts(package: &str, version: &str, world: &World) {
    let prefix = format!("{}::", cohdl::pipeline::package_root(package));
    let mut unexpected = Vec::new();
    for (name, part) in world.parts.iter().filter(|(name, part)| {
        name.starts_with(&prefix)
            && world.symbols.get(*name).is_some_and(|symbol| symbol.is_pub)
            && part.primary.footprint.is_some()
    }) {
        for (entry, is_primary) in std::iter::once((&part.primary, true))
            .chain(part.alts.iter().map(|entry| (entry, false)))
        {
            let missing: Vec<_> = ["mpn", "mfr"]
                .into_iter()
                .filter(|field_name| {
                    entry.field(field_name).is_none_or(|field| {
                        field.value.trim().is_empty()
                            && !(is_primary && part.allows_source_empty(field_name))
                    })
                })
                .collect();
            if missing.is_empty() {
                continue;
            }
            let footprint = entry
                .footprint
                .as_ref()
                .or(part.primary.footprint.as_ref())
                .expect("primary footprint exists");
            let key = (package, footprint.name.as_str(), version);
            if QUARANTINE.iter().any(|(p, f, v, _)| (*p, *f, *v) == key) {
                continue;
            }
            unexpected.push(format!("{package} {name}: empty {}", missing.join("/")));
        }
    }
    assert!(
        unexpected.is_empty(),
        "unquarantined part metadata defects:\n{}",
        unexpected.join("\n")
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use cohdl::pipeline::check_files_in;

    fn world(source: &str) -> World {
        world_in("fixture", source)
    }

    fn world_in(package: &str, source: &str) -> World {
        let checked = check_files_in(
            package,
            &[("src/lib.cohdl".to_string(), source.to_string())],
            None,
        )
        .expect("fixture selection");
        assert!(
            !checked.diags.has_errors(),
            "{}",
            checked.diags.render(&checked.sm)
        );
        checked.world
    }

    fn audit(source: &str) -> World {
        let world = world(source);
        audit_package("fixture", "1.0.0", &world);
        audit_parts("fixture", "1.0.0", &world);
        world
    }

    fn audit_panics(source: &str) -> String {
        let result = std::panic::catch_unwind(|| audit(source));
        match result {
            Ok(_) => panic!("audit accepted a defective fixture"),
            Err(payload) => payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default(),
        }
    }

    const PAD: &str = r#"
        pub pad LAND { shape: rect size: (1mm, 2mm) layer: top_copper plating: smd }
    "#;

    const PART: &str = r#"
        pub device DEV { pins { required GND: 1, 2 [passive] } }
        pub part P: DEV { primary { mfr: "m", mpn: "p", footprint: FP } }
    "#;

    #[test]
    fn audit_honours_only_explicit_source_empty_metadata() {
        let source = r#"
            pub footprint FP {}
            pub device DEV { pins { required GND: 1 [passive] } }
            #[source_unspecified("mpn")]
            pub part P: DEV { primary { mfr: "m", mpn: "", footprint: FP } }
        "#;
        let mut world = audit(source);
        // The checker rejects an empty MPN without the attribute, so the
        // audit-level negative strips it from an already-checked world.
        world
            .parts
            .get_mut("fixture::P")
            .expect("fixture part")
            .source_unspecified = None;
        let message = std::panic::catch_unwind(|| audit_parts("fixture", "1.0.0", &world))
            .expect_err("audit accepted an empty MPN without source_unspecified");
        let message = message
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_default();
        assert!(message.contains("fixture::P: empty mpn"), "{message}");
    }

    #[test]
    fn audit_rejects_rotated_overlap_at_37_and_45_degrees() {
        for angle in [37, 45] {
            let source = format!(
                r#"{PAD}
                pub footprint FP {{
                    pad 1: LAND at (0mm, 0mm)
                    pad 2: LAND at (0.8mm, 0.8mm) rotate {angle}
                    courtyard {{ shape: rect, at: (0mm, 0mm), size: (10mm, 10mm) }}
                }}
            "#
            );
            let message = audit_panics(&source);
            // Independent oracle: at (0.8, 0.8) the projection of each pad on
            // x=y is (cos a + sin a) * 1 >= 1.5 * 0.8*sqrt(2)... explicitly,
            // 0.8*sqrt(2) = 1.131 < 1.524 (37°) and 1.414 (45°), so the
            // projections meet on x=y whatever the other axes do.
            let (sin, cos) = (angle as f64).to_radians().sin_cos();
            assert!(0.8 * 2.0_f64.sqrt() < (cos + sin) * 1.0);
            assert!(
                message.contains("pad overlap") && message.contains("1 and 2"),
                "audit must name the overlapping pads, got: {message}"
            );
        }
    }

    #[test]
    fn audit_rejects_rectangular_courtyard_overflow() {
        let message = audit_panics(
            r#"
            pub pad LAND { shape: rect size: (2mm, 1mm) layer: top_copper plating: smd }
            pub footprint FP {
                pad 1: LAND at (0mm, 0mm) rotate 45
                courtyard { shape: rect, at: (0mm, 0mm), size: (2.1mm, 2.1mm) }
            }
        "#,
        );
        // Independent oracle: a 2x1 rectangle at 45 degrees has half extent
        // (2+1)/(2*sqrt(2)) = 1.0607 mm, exceeding the 1.05 mm half courtyard
        // by 0.011 mm — the message must quote that measured excess.
        assert!(3.0 / (2.0 * 2.0_f64.sqrt()) - 1.05 > 0.0);
        assert!(
            message.contains("courtyard overflow")
                && message.contains("rectangular")
                && message.contains("0.011 mm"),
            "audit must quote the rectangular courtyard excess, got: {message}"
        );
    }

    #[test]
    fn audit_rejects_circular_courtyard_overflow() {
        let message = audit_panics(
            r#"
            pub pad LAND { shape: rect size: (2mm, 1mm) layer: top_copper plating: smd }
            pub footprint FP {
                pad 1: LAND at (0mm, 0mm)
                courtyard { shape: circle, at: (0mm, 0mm), size: (2.2mm) }
            }
        "#,
        );
        // Independent oracle: the corner distance is sqrt(1^2+0.5^2) =
        // 1.1180 mm against the 1.1 mm courtyard radius: 0.018 mm over.
        assert!(1.25_f64.sqrt() - 1.1 > 0.0);
        assert!(
            message.contains("courtyard overflow")
                && message.contains("circular")
                && message.contains("0.018 mm"),
            "audit must quote the circular courtyard excess, got: {message}"
        );
    }

    #[test]
    fn audit_rejects_defects_in_a_footprint_only_package() {
        let message = audit_panics(
            r#"
            pub pad LAND { shape: rect size: (2mm, 2mm) layer: top_copper plating: smd }
            pub footprint ONLY {
                pad 1: LAND at (0mm, 0mm)
                pad 2: LAND at (1mm, 0mm)
                courtyard { shape: rect, at: (0mm, 0mm), size: (10mm, 10mm) }
            }
        "#,
        );
        let world = world(
            r#"
            pub pad LAND { shape: rect size: (2mm, 2mm) layer: top_copper plating: smd }
            pub footprint ONLY {
                pad 1: LAND at (0mm, 0mm)
                pad 2: LAND at (1mm, 0mm)
                courtyard { shape: rect, at: (0mm, 0mm), size: (10mm, 10mm) }
            }
        "#,
        );
        assert!(
            world.parts.is_empty(),
            "oracle requires a footprint-only fixture"
        );
        // Independent oracle: two 2 mm squares whose centres are 1 mm apart
        // share a 1 mm-wide strip of copper.
        assert!(
            message.contains("pad overlap") && message.contains("1 and 2"),
            "audit must flag a footprint-only package, got: {message}"
        );
    }

    #[test]
    fn audit_allows_same_pin_numbers_to_overlap() {
        // Device maps both pad numbers to one GND pin — the stacked GND lands
        // of a USB-C receptacle are the shipped case.
        audit(
            r#"
            pub device DEV { pins { required GND: 1, 2 [passive] } }
            pub pad LAND { shape: rect size: (2mm, 2mm) layer: top_copper plating: smd }
            pub footprint FP {
                pad 1: LAND at (0mm, 0mm)
                pad 2: LAND at (0.5mm, 0.5mm) rotate 37
                courtyard { shape: rect, at: (0mm, 0mm), size: (10mm, 10mm) }
            }
            pub part P: DEV { primary { mfr: "m", mpn: "p", footprint: FP } }
        "#,
        );
    }

    #[test]
    fn audit_rejects_overlap_when_numbers_map_to_distinct_pins() {
        let message = audit_panics(
            r#"
            pub device DEV { pins { required A: 1 [passive], required B: 2 [passive] } }
            pub pad LAND { shape: rect size: (2mm, 2mm) layer: top_copper plating: smd }
            pub footprint FP {
                pad 1: LAND at (0mm, 0mm)
                pad 2: LAND at (0.5mm, 0.5mm) rotate 37
                courtyard { shape: rect, at: (0mm, 0mm), size: (10mm, 10mm) }
            }
            pub part P: DEV { primary { mfr: "m", mpn: "p", footprint: FP } }
        "#,
        );
        assert!(
            message.contains("pad overlap"),
            "distinct pins must not be bridged by copper, got: {message}"
        );
    }

    #[test]
    fn audit_allows_same_number_ep_copper_islands() {
        audit(
            r#"{PAD}
            pub footprint FP {
                pad 0: LAND at (0mm, 0mm)
                pad 0: LAND at (0.5mm, 0mm) rotate 37
                courtyard { shape: rect, at: (0mm, 0mm), size: (10mm, 10mm) }
            }
        "#
            .replace("{PAD}", PAD)
            .as_str(),
        );
    }

    #[test]
    fn circle_hugging_a_rectangle_corner_without_contact_is_separate() {
        let world = world(
            r#"
            pub pad RECT_LAND { shape: rect size: (1mm, 2mm) layer: top_copper plating: smd }
            pub pad ROUND_LAND { shape: circle size: (1mm) layer: top_copper plating: smd }
            pub footprint FP {
                pad 1: RECT_LAND at (0mm, 0mm)
                pad 2: ROUND_LAND at (0.9mm, 1.4mm)
                courtyard { shape: rect, at: (0mm, 0mm), size: (10mm, 10mm) }
            }
        "#,
        );
        let fp = &world.footprints["fixture::FP"];
        let rect = geometry(&fp.pads[0], &world.pads[&fp.pads[0].pad.name]);
        let circle = geometry(&fp.pads[1], &world.pads[&fp.pads[1].pad.name]);
        // Independent oracle: the nearest rectangle corner is (0.5, 1.0);
        // the circle centre is sqrt(0.4^2 + 0.4^2) = 0.566 mm away against a
        // 0.5 mm radius — a 0.066 mm gap, no contact. Face normals alone
        // would wrongly report overlap (0.9 < 0.5 + 0.5 on x).
        assert!((0.4_f64 * 0.4 + 0.4 * 0.4).sqrt() > 0.5);
        assert!(!overlaps(rect, circle));
    }

    #[test]
    fn audit_accepts_a_clean_footprint_bound_to_a_part() {
        audit(
            format!(
                r#"{PAD}
                {PART}
                pub footprint FP {{
                    pad 1: LAND at (0mm, -0.5mm)
                    pad 2: LAND at (0mm, 0.5mm)
                    courtyard {{ shape: rect, at: (0mm, 0mm), size: (4mm, 4mm) }}
                }}
            "#
            )
            .replace("{PAD}", PAD)
            .replace("{PART}", PART)
            .as_str(),
        );
    }

    #[test]
    fn repaired_courtyard_must_contain_the_body() {
        let source = |height: &str| {
            format!(
                r#"
                pub pad LEAD {{ shape: rect size: (1.5mm, 0.55mm) layer: top_copper plating: smd }}
                pub footprint FP_NS4168_ESOP8 {{
                    pad 1: LEAD at (-3.1mm, -1.905mm)
                    pad 2: LEAD at (3.1mm, 1.905mm)
                    courtyard {{ shape: rect, at: (0mm, 0mm), size: (8.2mm, {height}) }}
                }}
                "#
            )
        };
        let contained = world_in("audio-amp", &source("5.6mm"));
        audit_package("audio-amp", "1.0.0", &contained);
        let short = world_in("audio-amp", &source("4.86mm"));
        let message = std::panic::catch_unwind(|| audit_package("audio-amp", "1.0.0", &short))
            .expect_err("a courtyard narrower than the body passed");
        let message = message
            .downcast_ref::<String>()
            .cloned()
            .unwrap_or_default();
        assert!(
            message.contains("FP_NS4168_ESOP8")
                && message.contains("4 x 5.1 mm body")
                && message.contains("short by 0.370 mm"),
            "{message}"
        );
    }
}
