//! RFC-020: narrow DXF board-outline extraction.
//!
//! CoHDL parses EXACTLY one thing out of a referenced DXF file: a single
//! closed outline entity (an `LWPOLYLINE` or legacy `POLYLINE`) on a
//! designated layer. Everything else — blocks, dimensions, text, hatching,
//! other layers, other entity types — is ignored. This is deliberately NOT a
//! general DXF parser (RFC-020 Non-goals), the same narrow-contract discipline
//! RFC-018 established for pad geometry.
//!
//! The extracted outline (straight segments + circular-arc bulges) is embedded
//! directly into IPC-2581's `Profile/Polygon` and into `layout.json` — the
//! geometry that makes the emitted document Quilter-importable.
//!
//! Coordinates are the lexer's femto-mm integers (10^-15 mm), parsed EXACTLY
//! from the DXF's decimal vertex text (no float). Only an arc's CENTER — which
//! a bulge implies but does not state — is computed in `f64` and rounded to
//! femto ONCE here, then stored, so both emitters read identical integers and
//! cannot disagree (the byte-stability discipline of `emit::geom`).

/// The documented layer-name convention: the board outline is the one closed
/// polyline on this layer. Matches KiCad's board-edge layer, so a DXF exported
/// from KiCad's `Edge.Cuts` (or a mechanical CAD file tagged to match) drops in
/// directly. Documented in docs/ipc2581.md.
pub const OUTLINE_LAYER: &str = "Edge.Cuts";

const FEMTO_MM: i128 = 1_000_000_000_000_000;

/// A resolved board outline: a closed loop starting at `start`, then one
/// segment per edge (the final segment returns to `start`).
#[derive(Debug, Clone)]
pub struct Outline {
    pub start: (i128, i128),
    pub segs: Vec<Seg>,
    /// `(lo, hi)` bounding box in femto-mm (arc extent approximated by
    /// endpoints — enough for an LSP hover / sanity dimension).
    pub bbox: ((i128, i128), (i128, i128)),
}

#[derive(Debug, Clone)]
pub enum Seg {
    Line {
        to: (i128, i128),
    },
    Arc {
        to: (i128, i128),
        center: (i128, i128),
        clockwise: bool,
    },
}

impl Outline {
    /// Exact centreline extent for placement origins, in femto-mm.
    ///
    /// Unlike `bbox`, this includes cardinal points reached by an arc. The
    /// integer square root rounds the radius down to femto-mm, so a computed
    /// cardinal is less than 1 femto-mm inside the mathematical extent. Arc
    /// endpoints are retained exactly. Keep `bbox` unchanged: IPC-2581 uses
    /// its endpoint-only approximation for stable component staging.
    pub fn extent(&self) -> ((i128, i128), (i128, i128)) {
        let mut extent = (self.start, self.start);
        let mut start = self.start;
        grow(&mut extent, start);
        for seg in &self.segs {
            let (to, center, clockwise) = match seg {
                Seg::Line { to } => {
                    grow(&mut extent, *to);
                    start = *to;
                    continue;
                }
                Seg::Arc {
                    to,
                    center,
                    clockwise,
                } => (*to, *center, *clockwise),
            };
            grow(&mut extent, to);
            let radial = (start.0 - center.0, start.1 - center.1);
            let radius =
                integer_sqrt(radial.0.unsigned_abs().pow(2) + radial.1.unsigned_abs().pow(2))
                    as i128;
            for (direction, cardinal) in [
                ((1, 0), (center.0 + radius, center.1)),
                ((0, 1), (center.0, center.1 + radius)),
                ((-1, 0), (center.0 - radius, center.1)),
                ((0, -1), (center.0, center.1 - radius)),
            ] {
                if arc_reaches(
                    radial,
                    (to.0 - center.0, to.1 - center.1),
                    direction,
                    clockwise,
                ) {
                    grow(&mut extent, cardinal);
                }
            }
            start = to;
        }
        extent
    }
}

fn grow(bounds: &mut ((i128, i128), (i128, i128)), point: (i128, i128)) {
    bounds.0 .0 = bounds.0 .0.min(point.0);
    bounds.0 .1 = bounds.0 .1.min(point.1);
    bounds.1 .0 = bounds.1 .0.max(point.0);
    bounds.1 .1 = bounds.1 .1.max(point.1);
}

fn cross(a: (i128, i128), b: (i128, i128)) -> i128 {
    a.0 * b.1 - a.1 * b.0
}

/// Does the directed arc pass this cardinal direction? Endpoints count as
/// reached, including the zero-sweep limit of two nearly coincident vertices.
fn arc_reaches(
    start: (i128, i128),
    end: (i128, i128),
    cardinal: (i128, i128),
    clockwise: bool,
) -> bool {
    // Reflect a clockwise arc into CCW; all angular comparisons then use one
    // convention without angles or floating point.
    let directed_cross = |a, b| if clockwise { -cross(a, b) } else { cross(a, b) };
    if start == end {
        return true; // DXF coincident endpoints denote a full circle.
    }
    let sweep = directed_cross(start, end);
    let after_start = directed_cross(start, cardinal);
    let before_end = directed_cross(cardinal, end);
    if sweep > 0 {
        // Less than 180 degrees: the cardinal must be inside both half-planes.
        after_start >= 0 && before_end >= 0
    } else if sweep < 0 {
        // More than 180 degrees: exclude the short, untravelled end-to-start arc.
        after_start >= 0 || before_end >= 0
    } else if start.0 * end.0 + start.1 * end.1 < 0 {
        after_start >= 0 // Exact semicircle.
    } else {
        after_start == 0 && start.0 * cardinal.0 + start.1 * cardinal.1 >= 0
    }
}

fn integer_sqrt(n: u128) -> u128 {
    let mut lo = 0u128;
    let mut hi = n.min(u128::from(u64::MAX)) + 1;
    while lo + 1 < hi {
        let mid = lo + (hi - lo) / 2;
        if mid <= n / mid {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// Why an outline could not be extracted — each becomes a distinct E1006
/// sub-case message (RFC-020 Gradeability).
#[derive(Debug)]
pub enum DxfError {
    /// The file is not readable as DXF group-code/value pairs at all.
    Unparseable(String),
    /// No closed polyline on the designated layer.
    NoEntity,
    /// A polyline on the layer exists but is not flagged closed.
    NotClosed,
    /// Fewer than three vertices — cannot bound an area.
    TooFew,
}

impl DxfError {
    pub fn message(&self, layer: &str) -> String {
        match self {
            DxfError::Unparseable(why) => {
                format!("board-outline DXF is not valid DXF: {}", why)
            }
            DxfError::NoEntity => format!(
                "board-outline DXF has no closed polyline on layer `{}` (RFC-020: \
                 the outline is one closed LWPOLYLINE/POLYLINE on that layer)",
                layer
            ),
            DxfError::NotClosed => format!(
                "the board-outline polyline on layer `{}` is not closed — a board \
                 outline must form one closed loop",
                layer
            ),
            DxfError::TooFew => "the board-outline polyline has fewer than 3 vertices".to_string(),
        }
    }
}

/// One raw DXF polyline entity gathered during scanning.
struct RawPoly {
    layer: String,
    closed: bool,
    /// `(x_femto, y_femto, bulge)` per vertex.
    verts: Vec<(i128, i128, f64)>,
}

/// Extract the closed outline on `layer`. Pure — the file's bytes come from
/// the caller (the CLI reads the FS; tests pass a literal), so this stays
/// testable and FS-free.
pub fn extract_outline(text: &str, layer: &str) -> Result<Outline, DxfError> {
    let pairs = tokenize(text)?;
    let polys = scan_polylines(&pairs)?;
    // The one closed polyline on the designated layer.
    let mut chosen: Option<&RawPoly> = None;
    let mut saw_layer = false;
    for p in &polys {
        if p.layer == layer {
            saw_layer = true;
            if p.closed {
                chosen = Some(p);
                break;
            }
        }
    }
    let poly = match chosen {
        Some(p) => p,
        None if saw_layer => return Err(DxfError::NotClosed),
        None => return Err(DxfError::NoEntity),
    };
    if poly.verts.len() < 3 {
        return Err(DxfError::TooFew);
    }

    let start = (poly.verts[0].0, poly.verts[0].1);
    let mut segs = Vec::new();
    let mut bbox = (start, start);
    // One segment per edge; edge i goes from vertex i to vertex i+1, using the
    // bulge stored ON vertex i (DXF convention). The closing edge (last->first)
    // uses the last vertex's bulge.
    let n = poly.verts.len();
    for i in 0..n {
        let (x1, y1, bulge) = poly.verts[i];
        let (x2, y2, _) = poly.verts[(i + 1) % n];
        let to = (x2, y2);
        grow(&mut bbox, to);
        if bulge == 0.0 {
            segs.push(Seg::Line { to });
        } else {
            let center = arc_center((x1, y1), to, bulge);
            segs.push(Seg::Arc {
                to,
                center,
                clockwise: bulge < 0.0,
            });
        }
    }
    Ok(Outline { start, segs, bbox })
}

/// Group-code/value pairs: DXF is line-oriented, alternating an integer code
/// line and a value line.
fn tokenize(text: &str) -> Result<Vec<(i32, String)>, DxfError> {
    let mut out = Vec::new();
    let mut lines = text.lines();
    while let Some(code) = lines.next() {
        let code = code.trim();
        if code.is_empty() {
            continue;
        }
        let value = match lines.next() {
            Some(v) => v.trim().to_string(),
            None => {
                return Err(DxfError::Unparseable(format!(
                    "group code `{}` has no value line",
                    code
                )))
            }
        };
        let code: i32 = code.parse().map_err(|_| {
            DxfError::Unparseable(format!("`{}` is not a numeric group code", code))
        })?;
        out.push((code, value));
    }
    if out.is_empty() {
        return Err(DxfError::Unparseable("empty file".to_string()));
    }
    Ok(out)
}

/// Gather every LWPOLYLINE and legacy POLYLINE entity (we only need their
/// layer, closed flag, and vertices — nothing else in the file is read).
fn scan_polylines(pairs: &[(i32, String)]) -> Result<Vec<RawPoly>, DxfError> {
    let mut polys = Vec::new();
    let mut i = 0;
    while i < pairs.len() {
        let (code, val) = &pairs[i];
        if *code == 0 && val == "LWPOLYLINE" {
            i = scan_lwpolyline(pairs, i + 1, &mut polys)?;
        } else if *code == 0 && val == "POLYLINE" {
            i = scan_polyline(pairs, i + 1, &mut polys)?;
        } else {
            i += 1;
        }
    }
    Ok(polys)
}

/// LWPOLYLINE: attributes until the next `0/...`. Layer 8, flags 70 (bit 1 =
/// closed), and repeated 10(x)/20(y)/42(bulge). A new 10 starts a new vertex.
fn scan_lwpolyline(
    pairs: &[(i32, String)],
    mut i: usize,
    out: &mut Vec<RawPoly>,
) -> Result<usize, DxfError> {
    let mut layer = String::new();
    let mut closed = false;
    let mut verts: Vec<(i128, i128, f64)> = Vec::new();
    while i < pairs.len() && pairs[i].0 != 0 {
        let (code, val) = &pairs[i];
        match code {
            8 => layer = val.clone(),
            70 => {
                closed = val
                    .trim()
                    .parse::<i32>()
                    .map(|f| f & 1 == 1)
                    .unwrap_or(false);
            }
            10 => {
                let x = femto(val)?;
                verts.push((x, 0, 0.0));
            }
            20 => {
                if let Some(v) = verts.last_mut() {
                    v.1 = femto(val)?;
                }
            }
            42 => {
                if let Some(v) = verts.last_mut() {
                    v.2 = val.trim().parse::<f64>().unwrap_or(0.0);
                }
            }
            _ => {}
        }
        i += 1;
    }
    out.push(RawPoly {
        layer,
        closed,
        verts,
    });
    Ok(i)
}

/// Legacy POLYLINE: a header (layer 8, flags 70) then a run of `0/VERTEX`
/// entities (each 10/20, optional 42), terminated by `0/SEQEND`.
fn scan_polyline(
    pairs: &[(i32, String)],
    mut i: usize,
    out: &mut Vec<RawPoly>,
) -> Result<usize, DxfError> {
    let mut layer = String::new();
    let mut closed = false;
    // header, up to the first sub-entity
    while i < pairs.len() && pairs[i].0 != 0 {
        let (code, val) = &pairs[i];
        match code {
            8 => layer = val.clone(),
            70 => {
                closed = val
                    .trim()
                    .parse::<i32>()
                    .map(|f| f & 1 == 1)
                    .unwrap_or(false);
            }
            _ => {}
        }
        i += 1;
    }
    let mut verts: Vec<(i128, i128, f64)> = Vec::new();
    while i < pairs.len() {
        let (code, val) = &pairs[i];
        if *code == 0 && val == "VERTEX" {
            let (x, y, b, ni) = scan_vertex(pairs, i + 1)?;
            verts.push((x, y, b));
            i = ni;
        } else if *code == 0 && val == "SEQEND" {
            i += 1;
            break;
        } else if *code == 0 {
            break;
        } else {
            i += 1;
        }
    }
    out.push(RawPoly {
        layer,
        closed,
        verts,
    });
    Ok(i)
}

fn scan_vertex(
    pairs: &[(i32, String)],
    mut i: usize,
) -> Result<(i128, i128, f64, usize), DxfError> {
    let (mut x, mut y, mut b) = (0i128, 0i128, 0.0f64);
    while i < pairs.len() && pairs[i].0 != 0 {
        let (code, val) = &pairs[i];
        match code {
            10 => x = femto(val)?,
            20 => y = femto(val)?,
            42 => b = val.trim().parse::<f64>().unwrap_or(0.0),
            _ => {}
        }
        i += 1;
    }
    Ok((x, y, b, i))
}

/// Parse a DXF decimal coordinate to femto-mm EXACTLY (no float): the vertex
/// coordinates carry the outline's real dimensions and must be lossless.
fn femto(s: &str) -> Result<i128, DxfError> {
    let s = s.trim();
    let neg = s.starts_with('-');
    let body = s.trim_start_matches(['+', '-']);
    if body.is_empty() || body.contains(['e', 'E']) {
        // Scientific notation is legal DXF but unusual for board coords; reject
        // rather than silently mis-scale.
        return Err(DxfError::Unparseable(format!("coordinate `{}`", s)));
    }
    let (int, frac) = body.split_once('.').unwrap_or((body, ""));
    let int_v: i128 = if int.is_empty() {
        0
    } else {
        int.parse()
            .map_err(|_| DxfError::Unparseable(format!("coordinate `{}`", s)))?
    };
    let mut f = frac.to_string();
    f.truncate(15);
    while f.len() < 15 {
        f.push('0');
    }
    let frac_v: i128 = if f.is_empty() {
        0
    } else {
        f.parse()
            .map_err(|_| DxfError::Unparseable(format!("coordinate `{}`", s)))?
    };
    let v = int_v * FEMTO_MM + frac_v;
    Ok(if neg { -v } else { v })
}

/// The center a bulge implies. A circular arc through two rational endpoints
/// with a rational bulge has a rational center, but the femto-scaled exact form
/// overflows i128; f64 (rounded to femto once, here) is exact enough for a
/// mechanical board outline and — crucially — computed in ONE place so both
/// emitters read the same integer. `center = mid + perp(chord) * (1-b²)/(4b)`.
fn arc_center(p1: (i128, i128), p2: (i128, i128), bulge: f64) -> (i128, i128) {
    let mm = |v: i128| v as f64 / FEMTO_MM as f64;
    let (x1, y1) = (mm(p1.0), mm(p1.1));
    let (x2, y2) = (mm(p2.0), mm(p2.1));
    let (mx, my) = ((x1 + x2) / 2.0, (y1 + y2) / 2.0);
    let (dx, dy) = (x2 - x1, y2 - y1);
    let k = (1.0 - bulge * bulge) / (4.0 * bulge);
    let cx = mx - dy * k;
    let cy = my + dx * k;
    let fm = |v: f64| (v * FEMTO_MM as f64).round() as i128;
    (fm(cx), fm(cy))
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECT: &str = "0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n8\nEdge.Cuts\n90\n4\n70\n1\n\
        10\n0.0\n20\n0.0\n10\n51.0\n20\n0.0\n10\n51.0\n20\n21.0\n10\n0.0\n20\n21.0\n0\nENDSEC\n";

    #[test]
    fn extracts_closed_rect() {
        let o = extract_outline(RECT, "Edge.Cuts").unwrap();
        assert_eq!(o.start, (0, 0));
        assert_eq!(o.segs.len(), 4); // 3 explicit edges + closing edge
        assert_eq!(o.bbox, ((0, 0), (51 * FEMTO_MM, 21 * FEMTO_MM)));
        assert!(matches!(o.segs[3], Seg::Line { to } if to == (0, 0)));
    }

    #[test]
    fn open_polyline_is_not_closed() {
        let open = RECT.replace("70\n1", "70\n0");
        assert!(matches!(
            extract_outline(&open, "Edge.Cuts"),
            Err(DxfError::NotClosed)
        ));
    }

    #[test]
    fn wrong_layer_is_no_entity() {
        assert!(matches!(
            extract_outline(RECT, "Other"),
            Err(DxfError::NoEntity)
        ));
    }

    #[test]
    fn arc_center_of_quarter_circle() {
        // P1=(1,0) -> P2=(0,1), CCW quarter (bulge=tan(22.5°)); center=(0,0).
        let f = FEMTO_MM;
        let c = arc_center((f, 0), (0, f), (22.5f64).to_radians().tan());
        // within 1 nm
        assert!(
            c.0.abs() < 1_000_000 && c.1.abs() < 1_000_000,
            "center {:?}",
            c
        );
    }

    #[test]
    fn garbage_is_unparseable() {
        assert!(extract_outline("not a dxf at all", "").is_err());
    }

    #[test]
    fn integer_square_root_rounds_down() {
        assert_eq!(integer_sqrt(0), 0);
        assert_eq!(integer_sqrt(1), 1);
        assert_eq!(integer_sqrt(99), 9);
        assert_eq!(integer_sqrt(100), 10);
        assert_eq!(integer_sqrt(u128::MAX), u128::from(u64::MAX));
    }

    #[test]
    fn arc_cardinals_follow_direction_and_sweep() {
        let east = (1, 0);
        let north = (0, 1);
        let west = (-1, 0);
        let south = (0, -1);
        for (start, end, clockwise, expected) in [
            (east, north, false, &[east, north][..]),
            (north, east, true, &[east, north][..]),
            (north, east, false, &[east, north, west, south][..]),
            (east, north, true, &[east, north, west, south][..]),
            (east, east, false, &[east, north, west, south][..]),
            (east, east, true, &[east, north, west, south][..]),
        ] {
            for cardinal in [east, north, west, south] {
                assert_eq!(
                    arc_reaches(start, end, cardinal, clockwise),
                    expected.contains(&cardinal),
                    "start={start:?}, end={end:?}, clockwise={clockwise}, cardinal={cardinal:?}"
                );
            }
        }
        // Short arcs between adjacent cardinals reach only their endpoints.
        for clockwise in [false, true] {
            let (start, end) = if clockwise {
                (north, east)
            } else {
                (east, north)
            };
            for cardinal in [west, south] {
                assert!(!arc_reaches(start, end, cardinal, clockwise));
            }
        }
        // Exact semicircles include only the pair of cardinals on their side.
        assert!(arc_reaches(east, west, north, false));
        assert!(!arc_reaches(east, west, south, false));
        assert!(arc_reaches(east, west, south, true));
        assert!(!arc_reaches(east, west, north, true));
    }
}
