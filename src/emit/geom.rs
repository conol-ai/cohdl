//! Exact-integer mm geometry shared by the `.kicad_mod` and IPC-2581
//! emitters (RFC-018). All arithmetic happens on the lexer's femto-scaled
//! integers (10^-15 mm) — no floats, no re-parsing of source text, no
//! precision cliff — and rendering is canonical (minimal decimal, trailing
//! zeros trimmed, never `-0`), so the two emitters cannot disagree on the
//! same geometry and two spellings of one value (`1mm`, `1.0mm`) project
//! identically.

use crate::units::UnitValue;

/// The largest `Length` magnitude (in femto-mm) the geometry emitters accept.
/// Corner arithmetic multiplies a femto value by 10 and adds/subtracts
/// another ×5, so an unbounded `i128` femto could overflow and panic
/// (review R5-5). `10^30` femto = `10^15` mm = a thousand billion kilometres
/// — beyond any conceivable footprint, yet 8 orders of magnitude below the
/// overflow point, so the arithmetic below can never wrap. A `Length` past
/// this bound is a clean E805/E806 at validation, BEFORE any artifact write
/// (see `crate::units::UnitValue::length_in_geom_range`).
pub const MAX_GEOM_FEMTO: i128 = 1_000_000_000_000_000_000_000_000_000_000;

/// A Length literal as a minimal decimal mm string (`-0.5mm` → `-0.5`,
/// `1.0mm` → `1`).
pub fn mm(v: &UnitValue) -> String {
    render(v.femto, 15)
}

/// A raw femto-mm integer as a minimal decimal mm string — for values the
/// emitter *computes* (e.g. component staging positions) rather than reads
/// from a `Length` literal. Same canonical rendering as [`mm`], so computed
/// and literal geometry project identically.
pub fn mm_femto(femto: i128) -> String {
    render(femto, 15)
}

/// Half of a raw femto-mm integer, rendered without rounding.  Pad polygon
/// vertices routinely sit at `size / 2`; multiplying the numerator by five
/// and rendering at 10^-16 mm preserves an odd final femto exactly.
pub fn half_mm_femto(femto: i128) -> String {
    render(femto.saturating_mul(5), 16)
}

/// A `Length` literal's y-coordinate NEGATED — for emitters whose target frame
/// has the opposite y-orientation from CoHDL's (IPC-2581 is +y-up; CoHDL/KiCad
/// author +y-down). Same canonical rendering as [`mm`].
pub fn mm_y(v: &UnitValue) -> String {
    render(-v.femto, 15)
}

/// A computed femto y-coordinate NEGATED (the raw-integer counterpart of
/// [`mm_y`]).
pub fn mm_femto_y(femto: i128) -> String {
    render(-femto, 15)
}

/// `center - size/2`, exact: computed at 10^-16 mm so halving an odd femto
/// count loses nothing. Saturating arithmetic is defense-in-depth — validation
/// rejects any `Length` past `MAX_GEOM_FEMTO`, so a real input never saturates.
pub fn corner_lo(center: &UnitValue, size: &UnitValue) -> String {
    render(
        center
            .femto
            .saturating_mul(10)
            .saturating_sub(size.femto.saturating_mul(5)),
        16,
    )
}

/// `center + size/2`, exact (also the circle-radius offset point).
pub fn corner_hi(center: &UnitValue, size: &UnitValue) -> String {
    render(
        center
            .femto
            .saturating_mul(10)
            .saturating_add(size.femto.saturating_mul(5)),
        16,
    )
}

/// `-(center - size/2)` — [`corner_lo`] with the y-axis negated (IPC-2581
/// +y-up projection of a CoHDL +y-down courtyard corner).
pub fn corner_lo_y(center: &UnitValue, size: &UnitValue) -> String {
    render(
        -(center
            .femto
            .saturating_mul(10)
            .saturating_sub(size.femto.saturating_mul(5))),
        16,
    )
}

/// `-(center + size/2)` — [`corner_hi`] with the y-axis negated.
pub fn corner_hi_y(center: &UnitValue, size: &UnitValue) -> String {
    render(
        -(center
            .femto
            .saturating_mul(10)
            .saturating_add(size.femto.saturating_mul(5))),
        16,
    )
}

/// Render `n / 10^scale` as a minimal decimal string.
/// RFC-027: a femto-unit value rendered at an arbitrary decimal scale — the
/// Quilter CSVs carry mA (scale 12), nF (6), ohm (15), and GHz (24). Same
/// canonical trailing-zero-trimmed rendering as every geometry emitter.
pub fn scaled(femto: i128, scale: u32) -> String {
    render(femto, scale)
}

/// Render a nonnegative rational geometry ratio without floating point.
/// KiCad stores chamfer size as `cut / min(width, height)`; both operands are
/// exact femto-mm integers, so long division keeps output deterministic and
/// avoids an architecture-dependent `f64` round trip.  Fifteen fractional
/// digits match CoHDL's geometry resolution.
pub fn ratio(numerator: i128, denominator: i128) -> String {
    debug_assert!(numerator >= 0 && denominator > 0);
    let n = numerator.max(0) as u128;
    let d = denominator.max(1) as u128;
    let int = n / d;
    let mut rem = n % d;
    let mut s = int.to_string();
    if rem == 0 {
        return s;
    }
    s.push('.');
    for _ in 0..15 {
        rem *= 10;
        s.push(char::from(b'0' + (rem / d) as u8));
        rem %= d;
        if rem == 0 {
            break;
        }
    }
    while s.ends_with('0') {
        s.pop();
    }
    s
}

fn render(n: i128, scale: u32) -> String {
    let neg = n < 0;
    let n = n.unsigned_abs();
    let base = 10u128.pow(scale);
    let int = n / base;
    let frac = n % base;
    let mut s = String::new();
    if neg && (int > 0 || frac > 0) {
        s.push('-'); // a zero value renders `0`, never `-0`
    }
    s.push_str(&int.to_string());
    if frac > 0 {
        let mut f = format!("{:0width$}", frac, width = scale as usize);
        while f.ends_with('0') {
            f.pop();
        }
        s.push('.');
        s.push_str(&f);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::{ratio, render};

    #[test]
    fn canonical_rendering() {
        assert_eq!(render(0, 15), "0");
        assert_eq!(render(-0, 15), "0");
        assert_eq!(render(1_000_000_000_000_000, 15), "1");
        assert_eq!(render(-500_000_000_000_000, 15), "-0.5");
        assert_eq!(render(1_900, 15), "0.0000000000019");
        // Odd femto halved at 10^-16 keeps the last digit.
        assert_eq!(render(15, 16), "0.0000000000000015");
    }

    #[test]
    fn ratio_rendering_is_exact_and_bounded() {
        assert_eq!(ratio(1, 2), "0.5");
        assert_eq!(ratio(1, 3), "0.333333333333333");
        assert_eq!(ratio(201, 275), "0.73090909090909");
        assert_eq!(ratio(1, 1), "1");
    }
}
