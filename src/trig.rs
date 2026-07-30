//! Deterministic fixed-point trigonometry for integer-degree rotations.
//!
//! Placement rotation used to be a closed four-value set (RFC-020), so the
//! emitters could rotate a pad offset with exact integer swaps and no
//! trigonometry at all. Arbitrary integer degrees need real sin/cos, and the
//! Constitution's hard constraint — same source, same netlist **bytes** — rules
//! out `f64::sin`: it resolves to the platform libm, whose final bit is not
//! guaranteed identical across macOS, glibc and musl. A rotated pad coordinate
//! is emitted geometry, so a one-bit difference is a different artifact.
//!
//! So the sine of every whole degree is a checked-in integer constant
//! (`tools/gen_trig.py`), and every rotation is integer arithmetic over those
//! constants. Deterministic by construction, not by luck.
//!
//! Note `emit::silk` and `emit::kicad_mod` do use `f64` trig to tessellate arcs
//! and circles into polylines. That is deliberately not followed here: those
//! call sites divide a rounded result by a radius and land on a femtometre
//! grid many orders coarser than the error, whereas a rotation scales a
//! coordinate that is already at full magnitude.
//!
//! ## Scale
//!
//! Values are scaled by `2^64`, chosen over a power of ten so that reducing the
//! 256-bit product back to 128 bits is a shift rather than a 256/128 division.
//! `sin(90°)` is therefore exactly `2^64` and `cos(90°)` exactly `0`, which is
//! what makes [`rotate`] agree with the old exact-swap implementation to the
//! bit at 0/90/180/270 — every board authored before arbitrary angles existed
//! emits identical bytes.

/// Fixed-point scale: one unit = `1 / 2^64`.
pub const SCALE: i128 = 1 << 64;

/// `sin(d°) * SCALE` for `d` in `0..=90`, rounded half away from zero. The
/// other three quadrants come from symmetry in [`sin_cos`], so the cardinals
/// stay exact. Regenerate with `python3 tools/gen_trig.py --in-place`.
///
/// `rustfmt::skip` because the layout is the generator's: left alone, rustfmt
/// re-aligns the trailing comments and indents the END marker, which would make
/// the file differ from what `gen_trig.py` writes and break the marker match.
#[rustfmt::skip]
static SIN_Q1: [i128; 91] = [
    // BEGIN GENERATED TABLE (tools/gen_trig.py)
    0, // sin(0 deg)
    321940075018930070, // sin(1 deg)
    643782083972305837, // sin(2 deg)
    965427990666446557, // sin(3 deg)
    1286779818642319329, // sin(4 deg)
    1607739681020117646, // sin(5 deg)
    1928209810316553226, // sin(6 deg)
    2248092588225778524, // sin(7 deg)
    2567290575354868340, // sin(8 deg)
    2885706540904802817, // sin(9 deg)
    3203243492287910726, // sin(10 deg)
    3519804704672751252, // sin(11 deg)
    3835293750447434651, // sin(12 deg)
    4149614528592406941, // sin(13 deg)
    4462671293953751414, // sin(14 deg)
    4774368686408090022, // sin(15 deg)
    5084611759910200732, // sin(16 deg)
    5393306011414502663, // sin(17 deg)
    5700357409661599243, // sin(18 deg)
    6005672423821110721, // sin(19 deg)
    6309158051982071155, // sin(20 deg)
    6610721849482211423, // sin(21 deg)
    6910271957067498877, // sin(22 deg)
    7207717128873355989, // sin(23 deg)
    7502966760219034614, // sin(24 deg)
    7795930915206679458, // sin(25 deg)
    8086520354116673787, // sin(26 deg)
    8374646560590922514, // sin(27 deg)
    8660221768595792345, // sin(28 deg)
    8943158989156495841, // sin(29 deg)
    9223372036854775808, // sin(30 deg)
    9500775556081818606, // sin(31 deg)
    9775285047038399446, // sin(32 deg)
    10046816891474339802, // sin(33 deg)
    10315288378159436446, // sin(34 deg)
    10580617728078103408, // sin(35 deg)
    10842724119340052334, // sin(36 deg)
    11101527711799423208, // sin(37 deg)
    11356949671374866210, // sin(38 deg)
    11608912194063166579, // sin(39 deg)
    11857338529639097693, // sin(40 deg)
    12102153005034283173, // sin(41 deg)
    12343281047387946570, // sin(42 deg)
    12580649206762527161, // sin(43 deg)
    12814185178517242456, // sin(44 deg)
    13043817825332782212, // sin(45 deg)
    13269477198880425016, // sin(46 deg)
    13491094561128976817, // sin(47 deg)
    13708602405283041112, // sin(48 deg)
    13921934476346242775, // sin(49 deg)
    14131025791303141777, // sin(50 deg)
    14335812658913689198, // sin(51 deg)
    14536232699114195931, // sin(52 deg)
    14732224862018904379, // sin(53 deg)
    14923729446516375051, // sin(54 deg)
    15110688118455023452, // sin(55 deg)
    15293043928412267748, // sin(56 deg)
    15470741329041874563, // sin(57 deg)
    15643726191994218747, // sin(58 deg)
    15811945824404303021, // sin(59 deg)
    15975348984942515102, // sin(60 deg)
    16133885899423233090, // sin(61 deg)
    16287508275966524584, // sin(62 deg)
    16436169319708321120, // sin(63 deg)
    16579823747054587077, // sin(64 deg)
    16718427799475141098, // sin(65 deg)
    16851939256832928277, // sin(66 deg)
    16980317450244682903, // sin(67 deg)
    17103523274469064271, // sin(68 deg)
    17221519199818492015, // sin(69 deg)
    17334269283591052503, // sin(70 deg)
    17441739181018994027, // sin(71 deg)
    17543896155730475763, // sin(72 deg)
    17640709089721383757, // sin(73 deg)
    17732148492834176429, // sin(74 deg)
    17818186511740872234, // sin(75 deg)
    17898796938427443188, // sin(76 deg)
    17973955218177029824, // sin(77 deg)
    18043638457049545813, // sin(78 deg)
    18107825428855393894, // sin(79 deg)
    18166496581621168848, // sin(80 deg)
    18219634043545378001, // sin(81 deg)
    18267221628442365087, // sin(82 deg)
    18309244840672779197, // sin(83 deg)
    18345690879559086948, // sin(84 deg)
    18376548643284782866, // sin(85 deg)
    18401808732276110234, // sin(86 deg)
    18421463452065262316, // sin(87 deg)
    18435506815634191792, // sin(88 deg)
    18443934545238314447, // sin(89 deg)
    18446744073709551616, // sin(90 deg)
    // END GENERATED TABLE
];

/// `(sin, cos)` of `deg` degrees, each scaled by [`SCALE`].
///
/// Exact at the four cardinal angles: `(0, SCALE)`, `(SCALE, 0)`,
/// `(0, -SCALE)`, `(-SCALE, 0)`.
pub fn sin_cos(deg: u16) -> (i128, i128) {
    let d = (deg % 360) as usize;
    match d {
        0..=89 => (SIN_Q1[d], SIN_Q1[90 - d]),
        90..=179 => (SIN_Q1[180 - d], -SIN_Q1[d - 90]),
        180..=269 => (-SIN_Q1[d - 180], -SIN_Q1[270 - d]),
        _ => (-SIN_Q1[360 - d], SIN_Q1[d - 270]),
    }
}

const MASK: u128 = u64::MAX as u128;

/// Full 128x128 -> 256-bit unsigned product, as `(high, low)`.
fn umul(a: u128, b: u128) -> (u128, u128) {
    let (a_lo, a_hi) = (a & MASK, a >> 64);
    let (b_lo, b_hi) = (b & MASK, b >> 64);
    let ll = a_lo * b_lo;
    let lh = a_lo * b_hi;
    let hl = a_hi * b_lo;
    let hh = a_hi * b_hi;
    let mid = (ll >> 64) + (lh & MASK) + (hl & MASK);
    let lo = (ll & MASK) | (mid << 64);
    let hi = hh + (lh >> 64) + (hl >> 64) + (mid >> 64);
    (hi, lo)
}

/// `a * b / SCALE`, rounded half away from zero, through a 256-bit
/// intermediate.
///
/// The intermediate is what lets the geometry bound stay exactly where
/// `emit::geom::MAX_GEOM_FEMTO` documents it. A `Length` at that bound is
/// ~2^100 femto; multiplied by a `2^64` trig value the product needs 164 bits,
/// so computing it in `i128` would wrap. Widening the multiply — rather than
/// shrinking the accepted coordinate range or the trig precision — keeps both
/// at full strength.
fn mul_scale_round(a: i128, b: i128) -> i128 {
    let neg = (a < 0) != (b < 0);
    let (hi, lo) = umul(a.unsigned_abs(), b.unsigned_abs());
    // += 2^63 (round half up on the magnitude), carrying into `hi`
    let (lo, carry) = lo.overflowing_add(1u128 << 63);
    let hi = hi + u128::from(carry);
    // `hi < 2^64` for every in-range input, so this cannot truncate.
    debug_assert!(hi >> 64 == 0, "rotation product exceeded 192 bits");
    let mag = (hi << 64) | (lo >> 64);
    let v = mag as i128;
    if neg {
        -v
    } else {
        v
    }
}

/// Rotate `(x, y)` counter-clockwise by `deg` whole degrees.
///
/// Counter-clockwise in the IPC-2581 frame (+y up), matching the convention the
/// closed-set implementation this replaces already used — `rotate(x, y, 90)` is
/// `(-y, x)`, to the bit.
pub fn rotate(x: i128, y: i128, deg: u16) -> (i128, i128) {
    match deg % 360 {
        // Kept as exact integer arithmetic. `sin_cos` is exact here too, so
        // this arm is redundant for correctness — it is here so the common
        // case carries no rounding step at all, and so the equivalence is
        // asserted by a test rather than assumed.
        0 => (x, y),
        90 => (-y, x),
        180 => (-x, -y),
        270 => (y, -x),
        d => {
            let (s, c) = sin_cos(d);
            (
                mul_scale_round(x, c) - mul_scale_round(y, s),
                mul_scale_round(x, s) + mul_scale_round(y, c),
            )
        }
    }
}

/// Half-extents of an axis-aligned box rotated by `deg`, as the half-extents of
/// the axis-aligned box that BOUNDS the result: `(hw|cos| + hh|sin|,
/// hw|sin| + hh|cos|)`.
///
/// Reduces to the identity at 0/180 and to a `w`/`h` swap at 90/270 — exactly
/// what the closed-set code did — and is conservative in between, which is the
/// direction a clearance calculation must err in.
pub fn bound_half_extents(hw: i128, hh: i128, deg: u16) -> (i128, i128) {
    match deg % 360 {
        0 | 180 => (hw, hh),
        90 | 270 => (hh, hw),
        d => {
            let (s, c) = sin_cos(d);
            let (s, c) = (s.abs(), c.abs());
            (
                mul_scale_round(hw, c) + mul_scale_round(hh, s),
                mul_scale_round(hw, s) + mul_scale_round(hh, c),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cardinals_are_exact() {
        assert_eq!(sin_cos(0), (0, SCALE));
        assert_eq!(sin_cos(90), (SCALE, 0));
        assert_eq!(sin_cos(180), (0, -SCALE));
        assert_eq!(sin_cos(270), (-SCALE, 0));
    }

    /// The fast path and the table path must agree to the BIT at the cardinals,
    /// or every board authored under the closed set shifts.
    #[test]
    fn table_path_reproduces_the_exact_swaps() {
        for (x, y) in [(0i128, 0i128), (1, 0), (0, 1), (12_345, -67_890), (-7, 3)] {
            for deg in [0u16, 90, 180, 270] {
                let (s, c) = sin_cos(deg);
                let via_table = (
                    mul_scale_round(x, c) - mul_scale_round(y, s),
                    mul_scale_round(x, s) + mul_scale_round(y, c),
                );
                assert_eq!(via_table, rotate(x, y, deg), "({x},{y}) rotate {deg}");
            }
        }
    }

    #[test]
    fn quadrant_symmetry() {
        for d in 0..360u16 {
            let (s, c) = sin_cos(d);
            // sin^2 + cos^2 == 1, within rounding of the scaled representation
            let unit = mul_scale_round(s, s) + mul_scale_round(c, c);
            assert!((unit - SCALE).abs() <= 4, "deg {d}: {unit} vs {SCALE}");
        }
    }

    #[test]
    fn known_angles() {
        // sin 30 = 1/2 exactly at this scale
        assert_eq!(sin_cos(30).0, SCALE / 2);
        // sin 45 == cos 45
        let (s, c) = sin_cos(45);
        assert_eq!(s, c);
        // 45 deg is sqrt(2)/2 = 0.70710678...
        assert_eq!(s / (SCALE / 100_000_000), 70_710_678);
    }

    #[test]
    fn four_quarter_turns_return_to_start_exactly() {
        let (x0, y0) = (4_000_000_000_000i128, -1_500_000_000_000i128);
        let (mut x, mut y) = (x0, y0);
        for _ in 0..4 {
            (x, y) = rotate(x, y, 90);
        }
        assert_eq!((x, y), (x0, y0));
        assert_eq!(rotate(x0, y0, 0), (x0, y0));
        assert_eq!(rotate(x0, y0, 360), (x0, y0));
    }

    /// `d` then `360 - d` must come back to where it started, within the
    /// femtometre or two the two roundings can cost.
    #[test]
    fn a_rotation_and_its_inverse_cancel() {
        let (x0, y0) = (37_250_000_000_000i128, -8_400_000_000_000i128);
        for d in 1..360u16 {
            let (x, y) = rotate(x0, y0, d);
            let (x, y) = rotate(x, y, 360 - d);
            assert!(
                (x - x0).abs() <= 4 && (y - y0).abs() <= 4,
                "deg {d}: ({x},{y}) vs ({x0},{y0})"
            );
        }
    }

    #[test]
    fn rotation_preserves_length_closely() {
        // 50mm in femto-mm — a real board coordinate
        for (x, y) in [
            (50_000_000_000_000i128, 0i128),
            (0, 50_000_000_000_000),
            (33_000_000_000_000, -19_000_000_000_000),
        ] {
            let want = x * x + y * y;
            // each coordinate carries at most ~1fm of rounding, so the squared
            // radius can move by ~2*r per axis — bound it generously in those
            // terms rather than as a ratio, which at this magnitude rounds to 0.
            let tol = 8 * (x.abs() + y.abs());
            for d in 0..360u16 {
                let (rx, ry) = rotate(x, y, d);
                let got = rx * rx + ry * ry;
                assert!(
                    (got - want).abs() <= tol,
                    "deg {d}: {got} vs {want} (tol {tol})"
                );
            }
        }
    }

    #[test]
    fn no_overflow_at_the_geometry_bound() {
        let m = crate::emit::geom::MAX_GEOM_FEMTO;
        for d in [1u16, 45, 89, 137, 271, 359] {
            let (rx, ry) = rotate(m, -m, d);
            // a rotation cannot grow the magnitude past sqrt(2) x the input
            assert!(rx.abs() <= 2 * m && ry.abs() <= 2 * m, "deg {d}");
        }
    }

    #[test]
    fn bound_half_extents_matches_the_closed_set() {
        assert_eq!(bound_half_extents(300, 100, 0), (300, 100));
        assert_eq!(bound_half_extents(300, 100, 180), (300, 100));
        assert_eq!(bound_half_extents(300, 100, 90), (100, 300));
        assert_eq!(bound_half_extents(300, 100, 270), (100, 300));
        // 45 deg: both extents become (300+100)*sqrt(2)/2 = 282.8 -> 283
        assert_eq!(bound_half_extents(300, 100, 45), (283, 283));
        // and it is never smaller than the unrotated box's shorter side
        for d in 0..360u16 {
            let (hw, hh) = bound_half_extents(300, 100, d);
            assert!(hw >= 100 && hh >= 100, "deg {d}: {hw},{hh}");
        }
    }
}
