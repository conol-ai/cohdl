//! RFC-034 §2: supplier mappings use the same placement composition as
//! subdesigns, with an explicit top-side mapping in the instance's local frame.

use cohdl::ast::PlacementSide;
use cohdl::placement::{compose_place, compose_supplier_mapping, PlaceData};
use cohdl::span::{FileId, Span};
use cohdl::units::{UnitType, UnitValue};

const MM: i128 = 1_000_000_000_000_000;

fn place(x: i128, y: i128, rotate: u16, side: PlacementSide) -> PlaceData {
    let length = |femto| UnitValue {
        unit: UnitType::Length,
        femto,
        text: String::new(),
    };
    PlaceData {
        at: (length(x), length(y)),
        rotate,
        side,
        span: Span::new(FileId(0), 7, 19),
    }
}

#[track_caller]
fn assert_place(actual: &PlaceData, expected: (i128, i128, u16, PlacementSide)) {
    assert_eq!(actual.at.0.femto, expected.0, "x");
    assert_eq!(actual.at.1.femto, expected.1, "y");
    assert_eq!(actual.rotate, expected.2, "rotation");
    assert_eq!(actual.side, expected.3, "side");
}

#[test]
fn identity_mapping_preserves_the_instance() {
    for side in [PlacementSide::Top, PlacementSide::Bottom] {
        let instance = place(-7 * MM, 5 * MM, 217, side);
        let mapped = compose_supplier_mapping(&instance, (0, 0), 0);
        assert_place(&mapped, (-7 * MM, 5 * MM, 217, side));
        assert_eq!(mapped.span, instance.span);
    }
}

#[test]
fn top_mapping_rotates_offset_and_adds_angles() {
    let instance = place(-7 * MM, 5 * MM, 90, PlacementSide::Top);
    // Rot(90) * (2, -3) = (-3, -2).
    let mapped = compose_supplier_mapping(&instance, (2 * MM, -3 * MM), 90);
    assert_place(&mapped, (-10 * MM, 3 * MM, 180, PlacementSide::Top));
}

#[test]
fn bottom_instance_inherits_side_from_top_sided_mapping() {
    let instance = place(-7 * MM, 5 * MM, 90, PlacementSide::Bottom);
    // Mirror x first: (2, -3) -> (-2, -3), then Rot(90) -> (-3, 2).
    let mapped = compose_supplier_mapping(&instance, (2 * MM, -3 * MM), 90);
    assert_place(&mapped, (-10 * MM, 7 * MM, 0, PlacementSide::Bottom));

    // Explicit Top is load-bearing: a genuinely bottom-sided child flips back.
    let bottom_child = place(2 * MM, -3 * MM, 90, PlacementSide::Bottom);
    assert_place(
        &compose_place(&instance, &bottom_child),
        (-10 * MM, 7 * MM, 0, PlacementSide::Top),
    );
}

#[test]
fn top_non_cardinal_mapping_matches_hand_calculation() {
    let instance = place(10 * MM, 20 * MM, 37, PlacementSide::Top);
    let mapped = compose_supplier_mapping(&instance, (-MM, 2 * MM), 53);
    // Rot(37) * (-1, 2) = (-cos(37) + 2 sin(37), sin(37) + 2 cos(37)).
    // Integer fixed-point evaluation rounds those products to the values below.
    assert_place(
        &mapped,
        (
            10_404_994_536_256_804,
            22_199_086_043_246_634,
            90,
            PlacementSide::Top,
        ),
    );
}

#[test]
fn bottom_non_cardinal_mapping_reverses_angle_and_keeps_bottom() {
    let instance = place(10 * MM, 20 * MM, 37, PlacementSide::Bottom);
    let mapped = compose_supplier_mapping(&instance, (-MM, 2 * MM), 53);
    // Mirror x: (-1, 2) -> (1, 2); then apply Rot(37), and use 37 - 53 mod 360.
    assert_place(
        &mapped,
        (
            12_002_265_556_351_390,
            20_995_455_996_942_538,
            344,
            PlacementSide::Bottom,
        ),
    );
}

#[test]
fn cardinal_boundaries_are_exact_integer_swaps() {
    let cases = [
        (0, (-5 * MM, 2 * MM)),
        (90, (-10 * MM, 3 * MM)),
        (180, (-9 * MM, 8 * MM)),
        (270, (-4 * MM, 7 * MM)),
    ];
    for (angle, (x, y)) in cases {
        let instance = place(-7 * MM, 5 * MM, angle, PlacementSide::Top);
        let mapped = compose_supplier_mapping(&instance, (2 * MM, -3 * MM), 0);
        assert_place(&mapped, (x, y, angle, PlacementSide::Top));
    }
}
