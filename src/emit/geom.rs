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

/// Render `n / 10^scale` as a minimal decimal string.
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
    use super::render;

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
}
