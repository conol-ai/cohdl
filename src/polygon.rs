use crate::ast::{PadDef, PadShape};
use crate::units::UnitType;

type Point = (i128, i128);

// Bound products and the signed-area sum well inside i128, including all
// pairwise edge comparisons. These limits are local to polygon geometry.
const MAX_COORD: i128 = 100_000_000_000_000_000;
const MAX_VERTICES: usize = 4096;
const ANCHOR_HALF: i128 = 500_000_000;

fn cross(a: Point, b: Point, c: Point) -> i128 {
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

fn on_segment(a: Point, b: Point, p: Point) -> bool {
    cross(a, b, p) == 0
        && (a.0.min(b.0)..=a.0.max(b.0)).contains(&p.0)
        && (a.1.min(b.1)..=a.1.max(b.1)).contains(&p.1)
}

fn intersects(a: Point, b: Point, c: Point, d: Point) -> bool {
    let ab_c = cross(a, b, c);
    let ab_d = cross(a, b, d);
    let cd_a = cross(c, d, a);
    let cd_b = cross(c, d, b);
    (ab_c.signum() * ab_d.signum() < 0 && cd_a.signum() * cd_b.signum() < 0)
        || on_segment(a, b, c)
        || on_segment(a, b, d)
        || on_segment(c, d, a)
        || on_segment(c, d, b)
}

fn contains_anchor(points: &[Point]) -> bool {
    let mut winding = 0_i32;
    let square = [
        (-ANCHOR_HALF, -ANCHOR_HALF),
        (ANCHOR_HALF, -ANCHOR_HALF),
        (ANCHOR_HALF, ANCHOR_HALF),
        (-ANCHOR_HALF, ANCHOR_HALF),
    ];
    for i in 0..points.len() {
        let (a, b) = (points[i], points[(i + 1) % points.len()]);
        if a.0.abs() <= ANCHOR_HALF && a.1.abs() <= ANCHOR_HALF {
            return false;
        }
        if (0..4).any(|j| intersects(a, b, square[j], square[(j + 1) % 4])) {
            return false;
        }
        if a.1 <= 0 && b.1 > 0 && cross(a, b, (0, 0)) > 0 {
            winding += 1;
        }
        if a.1 > 0 && b.1 <= 0 && cross(a, b, (0, 0)) < 0 {
            winding -= 1;
        }
    }
    winding != 0
}

pub(crate) fn validate(pad: &PadDef) -> Result<(), &'static str> {
    let polygon = matches!(pad.shape, Some((PadShape::Polygon, _)));
    if !polygon {
        return if pad.vertices.is_some() {
            Err("`vertices` requires `shape: polygon`")
        } else {
            Ok(())
        };
    }
    let Some((vertices, _)) = &pad.vertices else {
        return Err("polygon requires `vertices`");
    };
    if vertices.len() > MAX_VERTICES {
        return Err("polygon exceeds the 4096-vertex limit");
    }
    if vertices
        .iter()
        .flat_map(|(x, y)| [x, y])
        .any(|v| v.unit != UnitType::Length || v.femto < -MAX_COORD || v.femto > MAX_COORD)
    {
        return Err("polygon vertices must be finite Length literals within +/-100mm");
    }
    if pad.chamfer.is_some() || pad.corner_radius.is_some() {
        return Err("polygon cannot have a chamfer or corner radius");
    }
    if pad
        .mask_expansion
        .as_ref()
        .is_some_and(|(v, _)| v.femto != 0)
    {
        return Err("polygon with nonzero mask expansion is unsupported; exact polygon offsets are required");
    }
    let mut points: Vec<Point> = vertices.iter().map(|(x, y)| (x.femto, y.femto)).collect();
    points.dedup();
    if points.len() > 1 && points.first() == points.last() {
        points.pop();
    }
    validate_geometry(&points)
}

fn validate_geometry(points: &[Point]) -> Result<(), &'static str> {
    let distinct: std::collections::BTreeSet<_> = points.iter().collect();
    if distinct.len() < 3 {
        return Err("polygon needs at least three distinct vertices");
    }
    if distinct.len() != points.len() {
        return Err("polygon has a nonconsecutive repeated vertex (self-intersection)");
    }
    let n = points.len();
    let area: i128 = (0..n)
        .map(|i| {
            let (a, b) = (points[i], points[(i + 1) % n]);
            a.0 * b.1 - a.1 * b.0
        })
        .sum();
    if area == 0 {
        return Err("polygon has zero area");
    }
    for i in 0..n {
        let (a, b, c) = (points[i], points[(i + 1) % n], points[(i + 2) % n]);
        let dot = (b.0 - a.0) * (c.0 - b.0) + (b.1 - a.1) * (c.1 - b.1);
        if cross(a, b, c) == 0 && dot < 0 {
            return Err("polygon contains overlapping adjacent edges");
        }
        for j in i + 2..n {
            if i == 0 && j == n - 1 {
                continue;
            }
            if intersects(a, b, points[j], points[(j + 1) % n]) {
                return Err("polygon has self-intersecting edges");
            }
        }
    }
    if !contains_anchor(points) {
        return Err("polygon cannot contain KiCad's 1nm anchor at the declared pad center; exact custom-pad export is unsupported");
    }
    Ok(())
}
