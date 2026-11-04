import sys
import unittest
from decimal import Decimal
from pathlib import Path

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))
import realign_footprint as rf

SOURCE = """pub pad P { shape: rect, size: (0.8mm, 0.3mm), layer: top_copper, plating: smd }
pub footprint F {
    pad 1: P at (-2.5mm, -1.27mm)
    pad 2: P at (2.5mm, 1.27mm) rotate 90
    mount_hole 1: non_plated at (1mm, 0mm) diameter 1.02mm
    silkscreen {
        circle at (-3mm, -2mm) radius 0.2mm width 0.1mm fill solid
        polygon [(-2.6mm, -2.47mm), (-2.84mm, -2.8mm), (-2.36mm, -2.8mm)]
        pin_1_marker near pad 1 shape dot
    }
    // a comment keeps (1mm, 2mm)
    courtyard { shape: rect, at: (0.5mm, 0mm), size: (7.4mm, 5.4mm) }
    silkscreen_ref { at: (0mm, -3.4mm) }
}

pub footprint G {
    pad 1: P at (1mm, 1mm)
}
"""

# Hand-computed: R = 270 means p' = Rot(-270) p = Rot(90) p = (y, -x),
# pad angle' = (angle - 270) mod 360, and the courtyard swaps its size.
MOVED_270 = """pub footprint F {
    pad 1: P at (-1.27mm, 2.5mm) rotate 90
    pad 2: P at (1.27mm, -2.5mm) rotate 180
    mount_hole 1: non_plated at (0mm, -1mm) diameter 1.02mm
    silkscreen {
        circle at (-2mm, 3mm) radius 0.2mm width 0.1mm fill solid
        polygon [(-2.47mm, 2.6mm), (-2.8mm, 2.84mm), (-2.8mm, 2.36mm)]
        pin_1_marker near pad 1 shape dot
    }
    // a comment keeps (1mm, 2mm)
    courtyard { shape: rect, at: (0mm, -0.5mm), size: (5.4mm, 7.4mm) }
    silkscreen_ref { at: (-3.4mm, 0mm) }
}
"""


def block(text, name):
    start = text.index("pub footprint %s {" % name)
    return text[start:text.index("\n}\n", start) + 3]


class RealignFootprintTest(unittest.TestCase):
    def test_hand_computed_270(self):
        moved = rf.transform(SOURCE, "F", 270)
        self.assertEqual(block(moved, "F"), MOVED_270)
        # Everything outside the footprint is untouched.
        self.assertEqual(moved.replace(block(moved, "F"), ""), SOURCE.replace(block(SOURCE, "F"), ""))

    def test_half_turn_keeps_courtyard_size(self):
        moved = block(rf.transform(SOURCE, "F", 180), "F")
        self.assertIn("pad 1: P at (2.5mm, 1.27mm) rotate 180\n", moved)
        self.assertIn("pad 2: P at (-2.5mm, -1.27mm) rotate 270\n", moved)
        self.assertIn("at: (-0.5mm, 0mm), size: (7.4mm, 5.4mm)", moved)

    def test_a6_rigid_oracle(self):
        # Independent hand-written move of the A6 fixture by (1mm, 2mm, 90).
        fixture = TOOLS / "tests/fixtures/a6/rigid"
        moved = rf.transform((fixture / "src/main.cohdl").read_text(), "FP", 90, Decimal(1), Decimal(2))
        oracle = "".join(line + "\n" for line in (fixture / "moved.inc").read_text().splitlines() if not line.startswith("//"))
        self.assertEqual(block(moved, "FP"), oracle)

    def test_round_trip_is_identity(self):
        there = rf.transform(SOURCE, "F", 90, Decimal("0.25"), Decimal("-1.5"))
        # The inverse of (d, R) is (-Rot(-R) d, -R): here Rot(-90)(0.25, -1.5) = (1.5, 0.25).
        back = rf.transform(there, "F", 270, Decimal("-1.5"), Decimal("-0.25"))
        self.assertEqual(back, SOURCE)

    def test_refusals(self):
        with self.assertRaisesRegex(rf.RealignError, "multiple of|0, 90"):
            rf.transform(SOURCE, "F", 45)
        with self.assertRaisesRegex(rf.RealignError, "no footprint"):
            rf.transform(SOURCE, "H", 90)
        arc = SOURCE.replace("        pin_1_marker", "        arc center (0mm, 0mm) radius 1mm start 0 sweep 90 width 0.1mm\n        pin_1_marker")
        with self.assertRaisesRegex(rf.RealignError, "cannot move"):
            rf.transform(arc, "F", 90)


if __name__ == "__main__":
    unittest.main()
