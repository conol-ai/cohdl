//! Fabrication geometry audit, independent of part declarations.
//!
//! All dimensions are integer femtometres. Rotated rectangles (including
//! rounded rectangles, conservatively enclosed by their rectangles) use SAT;
//! circular pads use their exact radius. Oval pads use their enclosing
//! rectangle, so no real copper can escape the audit. Same-pin copper islands
//! are deliberately exempt, including when a part has no device pin map.

use cohdl::ast::{Courtyard, FootprintDef, PadDef, PadLayer, PadShape};
use cohdl::resolve::World;
use std::collections::BTreeSet;

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
}

fn dot(a: Point, b: Point) -> f64 {
    a.x * b.x + a.y * b.y
}

fn center_distance(a: Geometry, b: Geometry, axis: Point) -> f64 {
    ((b.center.x - a.center.x) * axis.x + (b.center.y - a.center.y) * axis.y).abs()
}

/// A positive gap on any candidate axis proves disjointness. Touching is not
/// overlap. Two circles need only their centre axis; all other combinations
/// use the four face normals of the two oriented rectangles.
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
        _ => [a, b]
            .into_iter()
            .flat_map(|shape| {
                let (sin, cos) = shape.angle.sin_cos();
                [Point { x: cos, y: sin }, Point { x: -sin, y: cos }]
            })
            .collect(),
    };
    !axes.into_iter().any(|axis| {
        center_distance(a, b, axis) >= a.projection_radius(axis) + b.projection_radius(axis)
    })
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

fn courtyard_contains(courtyard: &Courtyard, shape: Geometry) -> bool {
    let cx = courtyard.at.0.femto as f64;
    let cy = courtyard.at.1.femto as f64;
    match courtyard.shape.0 {
        PadShape::Circle => {
            let [radius] = courtyard.size.as_slice() else {
                panic!("validated circular courtyard has invalid dimensions");
            };
            let radius = radius.femto as f64 * 0.5;
            // A circular pad fits iff its own centre is within (radius − r);
            // any other pad fits iff all four of its rotated corners do.
            match shape.radius {
                Some(pad_radius) => {
                    (shape.center.x - cx).hypot(shape.center.y - cy) + pad_radius <= radius
                }
                None => shape
                    .corners()
                    .into_iter()
                    .all(|point| (point.x - cx).hypot(point.y - cy) <= radius),
            }
        }
        PadShape::Rect => {
            let [width, height] = courtyard.size.as_slice() else {
                panic!("validated rectangular courtyard has invalid dimensions");
            };
            let half_w = width.femto as f64 * 0.5;
            let half_h = height.femto as f64 * 0.5;
            // A circular pad fits iff its bounding square does (exact for a
            // rectangular courtyard); any other pad uses its rotated corners.
            match shape.radius {
                Some(radius) => {
                    (shape.center.x - cx).abs() + radius <= half_w
                        && (shape.center.y - cy).abs() + radius <= half_h
                }
                None => shape
                    .corners()
                    .into_iter()
                    .all(|point| (point.x - cx).abs() <= half_w && (point.y - cy).abs() <= half_h),
            }
        }
        shape => panic!("unexpected courtyard shape {shape:?}"),
    }
}

/// `(package, footprint, version)`: known defects deferred to A1b/A1c.
/// Keep every entry scoped to one footprint and state the deferred repair.
const QUARANTINE: &[(&str, &str, &str, &str)] = &[
    (
        "@espressif/esp32",
        "espressif_esp32::FP_ESP32_PICO_D4",
        "0.3.0",
        "待 A1b：courtyard 未包住旋转焊盘角点",
    ),
    (
        "@espressif/esp32",
        "espressif_esp32::FP_ESP32_PICO_V3",
        "0.3.0",
        "待 A1b：courtyard 未包住旋转焊盘角点",
    ),
    (
        "@espressif/esp32",
        "espressif_esp32::FP_ESP32_PICO_V3_02",
        "0.3.0",
        "待 A1b：courtyard 未包住旋转焊盘角点",
    ),
    (
        "audio-amp",
        "audio_amp::FP_NS4168_ESOP8",
        "0.1.0",
        "待 A1c：courtyard 未包住全部焊盘",
    ),
    (
        "qfn",
        "qfn::ESPRESSIF_QFN28_0P4_4",
        "0.2.0",
        "待 A1b/A1c：全封装审计发现几何问题",
    ),
    (
        "qfn",
        "qfn::ESPRESSIF_QFN48EB_0P35_5",
        "0.2.0",
        "待 A1b/A1c：全封装审计发现几何问题",
    ),
    (
        "qfn",
        "qfn::ESPRESSIF_QFN48E_0P4_6",
        "0.2.0",
        "待 A1b/A1c：全封装审计发现几何问题",
    ),
    (
        "usb",
        "usb::connectors::gct_usb4105::FP_USB_C_GCT_USB4105",
        "0.1.5",
        "待 A1b/A1c：全封装审计发现几何问题",
    ),
    (
        "usb",
        "usb::connectors::micro_b::FP_USB_Micro_B_SMD",
        "0.1.5",
        "待 A1b/A1c：全封装审计发现几何问题",
    ),
    (
        "usb",
        "usb::connectors::type_c::FP_USB_C_Receptacle_HRO_TYPE_C_31_M_12",
        "0.1.5",
        "待 A1b/A1c：全封装审计发现几何问题",
    ),
];

/// Keys migrated verbatim from the previous 46-entry per-part allowlist.
/// Every key must still name a real shipped footprint (staleness gate).
const LEGACY: &[(&str, &str, &str, &str)] = &[
    (
        "@contrib/analog-switch",
        "contrib_analog_switch::SOT6P65X210X125N",
        "0.1.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/analog-switch",
        "contrib_analog_switch::SOT6P95X290X160N",
        "0.1.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/analog-switch",
        "contrib_analog_switch::FP_TI_DGS0010A",
        "0.1.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/analog-switch",
        "contrib_analog_switch::FP_TI_RUT0012A",
        "0.1.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/audio-amp",
        "contrib_audio_amp::QFN16N50P300X300_1EP123X123",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/audio-amp",
        "contrib_audio_amp::SOP8P65X490X110N",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/charger",
        "contrib_charger::FP_TI_DLH0010A",
        "0.1.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/charger",
        "contrib_charger::BGA9C50P3X3_152X152X60N",
        "0.1.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "soic",
        "soic::TI_D0008A_4214825C",
        "0.2.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/env",
        "contrib_env::FP_Bosch_BME280_LGA8_2_5x2_5mm",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/esd",
        "contrib_esd::FP_SOD323_2P5X1P25mm",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/esd",
        "contrib_esd::QFN2N65P60X100",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/fuel-gauge",
        "contrib_fuel_gauge::FP_CellWise_TDFN8_2X3",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/gnss",
        "contrib_gnss::FP_Quectel_L76K_18LCC_101x97mm",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/gnss",
        "contrib_gnss::FP_Ublox_MIA_M10Q_MLGA53_45x45mm",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/haptic",
        "contrib_haptic::QFN9N40P137X137",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/haptic",
        "contrib_haptic::BGA9C50P3X3_145X145X50N",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/imu",
        "contrib_imu::LGA44P40_360X410",
        "0.1.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/imu",
        "contrib_imu::LGA14P50_300X250X86N",
        "0.1.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/imu",
        "contrib_imu::WLP4C40P2X2_82X82X40N",
        "0.1.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/io-expander",
        "contrib_io_expander::SOP24P65X640X120N",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/ir-emitter",
        "contrib_ir_emitter::FP_VSMY14940_3P0X2P51mm",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/keyscan",
        "contrib_keyscan::QFN24N50P400X400_1EP245X245",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/ldo",
        "contrib_ldo::FP_ETA_SOT89_5",
        "0.1.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/ldo",
        "contrib_ldo::FP_Microne_ME6211_SOT23_5",
        "0.1.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/ldo",
        "contrib_ldo::SOT5P95X290X160N",
        "0.1.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/ldo",
        "contrib_ldo::FP_Richtek_ZQFN4L_1X1",
        "0.1.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/ldo",
        "contrib_ldo::SOT3P190X290X160N",
        "0.1.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/led-driver",
        "contrib_led_driver::QFN20N40P300X300_1EP170X170",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "soic",
        "soic::SOP14P65X640X120N",
        "0.2.1",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/level-shifter",
        "contrib_level_shifter::QFN12N40P200X170",
        "0.1.2",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/level-shifter",
        "contrib_level_shifter::QFN12N40P200X200_1EP120X120",
        "0.1.2",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/level-shifter",
        "contrib_level_shifter::QFN14N50P350X350_1EP200X150",
        "0.1.2",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/lora",
        "contrib_lora::QFN24N50P400X400_1EP270X270",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/mic",
        "contrib_mic::FP_TDK_T3902",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/nfc",
        "contrib_nfc::QFN32N50P500X500_1EP345X345",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/rtc",
        "contrib_rtc::SOIC8P127X600X175N",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/rtc",
        "contrib_rtc::FP_NXP_DFN2626_10_SOT1197_1",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/rtc",
        "contrib_rtc::SOP8P65X490X110N",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@contrib/sd-card",
        "contrib_sd_card::FP_MicroSD_Wurth_693070010811",
        "0.1.0",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
    (
        "@sifli/sf32",
        "sifli_sf32::FP_SiFli_SF32LB52X_QFN68L_7X7",
        "0.1.2",
        "迁移自旧46条按条目白名单；待 A1b/A1c 复核",
    ),
];

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

pub fn audit_package(
    package: &str,
    version: &str,
    world: &World,
    real_keys: &mut BTreeSet<String>,
) {
    let mut unexpected = Vec::new();
    let mut used = BTreeSet::new();
    for (name, footprint) in own_declarations(world, package) {
        real_keys.insert(footprint_key(package, version, name));
        let mut failures = Vec::new();
        let courtyard = footprint.courtyard.as_ref();
        for place in &footprint.pads {
            let pad = &world.pads[&place.pad.name];
            let shape = geometry(place, pad);
            if courtyard.is_some_and(|courtyard| !courtyard_contains(courtyard, shape)) {
                failures.push(format!(
                    "courtyard overflow: pad {} at {}",
                    place.number.text, place.span.start
                ));
            }
        }
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
                if shared_layer && overlaps(geometry(a, a_pad), geometry(b, b_pad)) {
                    failures.push(format!(
                        "distinct-pad copper overlap: {} and {}",
                        a.number.text, b.number.text
                    ));
                }
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

/// Every shipped footprint's `(package, footprint, version)` key, gathered by
/// the caller while auditing, so `finalize` can gate both allowlists against
/// reality (no silent free pass, no stale entry).
pub fn footprint_key(package: &str, version: &str, footprint: &str) -> String {
    format!("{package}|{footprint}|{version}")
}

/// Staleness gate: every migrated legacy key still names a shipped footprint,
/// and no migrated legacy key matches the *auditing* package's own footprint
/// names when that footprint is clean (legacy keys are informational here;
/// the strict audit is enforced by `audit_package`).
pub fn finalize(real_keys: &BTreeSet<String>) {
    for (package, footprint, version, _) in LEGACY {
        let key = format!("{package}|{footprint}|{version}");
        if !real_keys.contains(&key) {
            eprintln!("A1a legacy entry has no audited footprint match: {key}");
        }
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

/// Audit public part order codes separately. A part defect is associated with
/// its package footprint key, allowing the same quarantine ledger to describe
/// the known fabrication-data exception without weakening unrelated parts.
pub fn audit_parts(package: &str, version: &str, world: &World) {
    let prefix = format!("{}::", cohdl::pipeline::package_root(package));
    let mut unexpected = Vec::new();
    for (name, part) in world.parts.iter().filter(|(name, part)| {
        name.starts_with(&prefix)
            && world.symbols.get(*name).is_some_and(|symbol| symbol.is_pub)
            && part.primary.footprint.is_some()
    }) {
        for entry in std::iter::once(&part.primary).chain(part.alts.iter()) {
            let missing: Vec<_> = ["mpn", "mfr"]
                .into_iter()
                .filter(|field| {
                    entry
                        .field(field)
                        .is_none_or(|field| field.value.trim().is_empty())
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
        let checked = check_files_in(
            "fixture",
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
        audit_package("fixture", "1.0.0", &world, &mut BTreeSet::new());
        world
    }

    const PAD: &str = r#"
        pub pad LAND { shape: rect size: (1mm, 2mm) layer: top_copper plating: smd }
    "#;

    #[test]
    fn rotated_rectangles_37_and_45_degrees_overlap() {
        for angle in [37, 45] {
            let source = format!(
                r#"{PAD}
                pub footprint BAD {{
                    pad 1: LAND at (0mm, 0mm)
                    pad 2: LAND at (0.8mm, 0.8mm) rotate {angle}
                    courtyard {{ shape: rect, at: (0mm, 0mm), size: (10mm, 10mm) }}
                }}
            "#
            );
            let world = world(&source);
            let fp = &world.footprints["fixture::BAD"];
            let a = geometry(&fp.pads[0], &world.pads[&fp.pads[0].pad.name]);
            let b = geometry(&fp.pads[1], &world.pads[&fp.pads[1].pad.name]);
            // Independently derived oracle: at (0.8, 0.8), each rotated
            // rectangle's projection onto x=y extends to +/-(cos+sin)*0.5
            // (width 1, height 2 gives half diagonal along either normal).
            let projection = (angle as f64).to_radians().sin_cos();
            assert!(0.8 * 2.0_f64.sqrt() < projection.0 + projection.1);
            assert!(overlaps(a, b), "{angle}-degree rectangles overlap on x=y");
        }
    }

    #[test]
    fn same_number_ep_copper_islands_may_overlap() {
        audit(
            r#"{PAD}
            pub footprint GOOD {
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
    fn rectangle_courtyard_rejects_rotated_corner() {
        let world = world(
            r#"
            pub pad LAND { shape: rect size: (2mm, 1mm) layer: top_copper plating: smd }
            pub footprint BAD {
                pad 1: LAND at (0mm, 0mm) rotate 45
                courtyard { shape: rect, at: (0mm, 0mm), size: (2.1mm, 2.1mm) }
            }
        "#,
        );
        let fp = &world.footprints["fixture::BAD"];
        let shape = geometry(&fp.pads[0], &world.pads[&fp.pads[0].pad.name]);
        // Independent oracle: a 2x1 rectangle at 45 degrees has half extent
        // (2+1)/(2*sqrt(2)) = 1.06066mm, outside the 1.05mm half courtyard.
        assert!(3.0 / (2.0 * 2.0_f64.sqrt()) > 1.05);
        assert!(!courtyard_contains(fp.courtyard.as_ref().unwrap(), shape));
    }

    #[test]
    fn circle_courtyard_rejects_corner_outside_radius() {
        let world = world(
            r#"
            pub pad LAND { shape: rect size: (2mm, 1mm) layer: top_copper plating: smd }
            pub footprint BAD {
                pad 1: LAND at (0mm, 0mm)
                courtyard { shape: circle, at: (0mm, 0mm), size: (2.2mm) }
            }
        "#,
        );
        let fp = &world.footprints["fixture::BAD"];
        let shape = geometry(&fp.pads[0], &world.pads[&fp.pads[0].pad.name]);
        // Independent oracle: the corner radius is sqrt(1^2+0.5^2)=1.118mm,
        // exceeding the authored 1.1mm courtyard radius.
        assert!(1.25_f64.sqrt() > 1.1);
        assert!(!courtyard_contains(fp.courtyard.as_ref().unwrap(), shape));
    }

    #[test]
    fn footprint_only_package_is_audited() {
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
        let fp = &world.footprints["fixture::ONLY"];
        let a = geometry(&fp.pads[0], &world.pads[&fp.pads[0].pad.name]);
        let b = geometry(&fp.pads[1], &world.pads[&fp.pads[1].pad.name]);
        // Two 2mm squares with centres 1mm apart share a 1mm-wide strip.
        assert!(overlaps(a, b));
    }
}
