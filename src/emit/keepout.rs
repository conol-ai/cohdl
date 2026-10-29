//! Provisional rectangular, all-copper keepouts. One zone serialization is
//! shared by standalone footprints and embedded board footprints. Unlike pad
//! coordinates, KiCad stores embedded zone polygons in ABSOLUTE board space.

use crate::ast::{Courtyard, PadShape};
use crate::emit::geom;
use crate::units::UnitType;
use std::fmt::Write as _;

pub(crate) fn validate(k: &Courtyard) -> Result<(), &'static str> {
    if k.shape.0 != PadShape::Rect || k.size.len() != 2 {
        return Err("keepout requires a rectangle with two dimensions");
    }
    if k.size
        .iter()
        .chain([&k.at.0, &k.at.1])
        .any(|v| v.unit != UnitType::Length || !v.length_in_geom_range())
    {
        return Err("keepout coordinates and dimensions must be in-range Length literals");
    }
    if k.size.iter().any(|v| v.femto <= 0) {
        return Err("keepout dimensions must be positive");
    }
    Ok(())
}

/// 10^-16 mm coordinates retain exact half-femtometre rectangle edges.
pub(crate) fn points(k: &Courtyard) -> [(i128, i128); 4] {
    let (x, y) = (k.at.0.femto * 10, k.at.1.femto * 10);
    let (w, h) = (k.size[0].femto * 5, k.size[1].femto * 5);
    [
        (x - w, y - h),
        (x + w, y - h),
        (x + w, y + h),
        (x - w, y + h),
    ]
}

pub(crate) fn zone(s: &mut String, indent: &str, pts: &[(i128, i128)], uuid: Option<&str>) {
    let _ = writeln!(s, "{indent}(zone");
    let _ = writeln!(s, "{indent}  (layers \"*.Cu\")");
    if let Some(uuid) = uuid {
        let _ = writeln!(s, "{indent}  (uuid \"{uuid}\")");
    }
    let _ = writeln!(s, "{indent}  (name \"antenna keepout\")");
    let _ = writeln!(s, "{indent}  (hatch edge 0.5)");
    let _ = writeln!(s, "{indent}  (connect_pads (clearance 0))");
    let _ = writeln!(s, "{indent}  (min_thickness 0.25)");
    let _ = writeln!(s, "{indent}  (keepout (tracks not_allowed) (vias not_allowed) (pads not_allowed) (copperpour not_allowed) (footprints not_allowed))");
    let _ = writeln!(
        s,
        "{indent}  (fill (thermal_gap 0.5) (thermal_bridge_width 0.5))"
    );
    let _ = write!(s, "{indent}  (polygon (pts");
    for &(x, y) in pts {
        let _ = write!(s, " (xy {} {})", geom::render(x, 16), geom::render(y, 16));
    }
    let _ = writeln!(s, "))\n{indent})");
}
