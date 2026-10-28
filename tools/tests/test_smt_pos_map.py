import copy
import math
from pathlib import Path
import sys
import unittest


sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import smt_pos_map as sm


FP = "acme::demo::SOIC"


def document(pads, designators=None):
    return {
        "schema_version": 1,
        "pad_projection": 1,
        "footprints": {
            FP: {
                "package": "acme/demo",
                "version": "1.0.0",
                "pads": pads,
                "frame": "0" * 64,
            }
        },
        "designators": {
            "U1": {
                "part": "acme::demo::part",
                "footprint": FP,
                "entry": "primary",
                "mfr": "M",
                "mpn": "P",
                "pn": "S1",
                "at": ["0mm", "0mm"],
                "rotate": 0,
            }
        }
        if designators is None
        else designators,
    }


def board_pads(pads):
    return [(number, (x, y), i) for i, (number, x, y) in enumerate(pads)]


class DecimalAndRotationTests(unittest.TestCase):
    def test_decimal_mm_is_exact_and_symmetric(self):
        cases = {
            "0mm": 0,
            "1mm": 1_000_000,
            "-0.001mm": -1000,
            "+0.0000004mm": 0,
            "-0.0000004mm": 0,
            "0.0000005mm": 1,
            "-0.0000005mm": -1,
            "0.0000015mm": 2,
            "-0.0000015mm": -2,
            ".000001mm": 1,
        }
        for text, expected in cases.items():
            with self.subTest(text=text):
                self.assertEqual(sm.parse_mm_nm(text), expected)
        for text in [
            "1",
            "1m",
            "0x1mm",
            "1e-6mm",
            "NaNmm",
            "Infmm",
            " 1mm",
            "1 mm",
        ]:
            with self.subTest(invalid=text):
                with self.assertRaises(sm.MapError):
                    sm.parse_mm_nm(text)

    def test_projection_opposite_sign_subtolerance_x_errors(self):
        # RFC 3.2: equal x coordinates can exchange order when rounded.  A
        # sort/zip matcher would pair these with the wrong 2mm y coordinates.
        mapped = sm.parse_map_pads(
            [["1", "0.0000004mm", "0mm"], ["1", "0.0000004mm", "0.002mm"]]
        )
        board = board_pads([("1", 0, 2000), ("1", 0, 0)])
        self.assertEqual(sm.match_pads(board, mapped), (True, None))

    def test_projection_one_degree_rotation_rounded_to_nm(self):
        # RFC 3.2: 1-degree board rotation followed by integer-nm recovery.
        local = [(1_000_000, 0), (0, 2_000_000), (-3_000_000, 4_000_000)]
        radians = math.radians(1)
        board = []
        for x, y in local:
            bx = sm.round_nm(x * math.cos(radians) - y * math.sin(radians))
            by = sm.round_nm(x * math.sin(radians) + y * math.cos(radians))
            board.append({"number": "1", "xy_nm": (bx + 100, by + 200)})
        recovered = sm.recover_pads_nm(board, (100, 200), 1)
        mapped = sm.parse_map_pads(
            [
                ["1", "{}mm".format(x / 1_000_000), "{}mm".format(y / 1_000_000)]
                for x, y in local
            ]
        )
        self.assertEqual(sm.match_pads(recovered, mapped), (True, None))

    def test_negative_rotation_half_rounds_away_from_zero(self):
        self.assertEqual(sm.round_nm(0.5), 1)
        self.assertEqual(sm.round_nm(-0.5), -1)
        self.assertEqual(sm.round_nm(1.5), 2)
        self.assertEqual(sm.round_nm(-1.5), -2)


class MatchingTests(unittest.TestCase):
    def test_same_x_different_y_and_same_number_swap(self):
        mapped = sm.parse_map_pads([["2", "0mm", "0mm"], ["2", "0mm", "0.002mm"]])
        board = board_pads([("2", 0, 2000), ("2", 0, 0)])
        self.assertEqual(sm.match_pads(board, mapped), (True, None))

    def test_projection_one_degree_fixture_has_actual_inverse_error(self):
        recovered = sm.recover_local_nm((999_848, 17_452), (0, 0), 1)
        self.assertEqual(recovered, (1_000_000, 0))
        recovered = sm.recover_local_nm((-34_905, 1_999_695), (0, 0), 1)
        self.assertEqual(recovered, (0, 2_000_000))

    def test_perfect_matching_not_greedy(self):
        # First board pad sees both; second sees only the second map pad.
        # Taking the nearest/first edge for board[0] would reject this.
        board = [(0, 0), (0, 1500)]
        mapped = [(0, 0), (0, 2500)]
        self.assertEqual(sm.perfect_matching(board, mapped), [(0, 0), (1, 1)])
        self.assertEqual(sm.perfect_matching(board[::-1], mapped), [(0, 1), (1, 0)])

    def test_long_alternating_path_matching(self):
        board = [(i * 1000, 0) for i in range(20)]
        self.assertIsNotNone(sm.perfect_matching(board, board))
        self.assertIsNone(sm.perfect_matching(board, [(0, 5000)] + board[1:]))

    def test_tolerance_is_per_axis_and_inclusive(self):
        self.assertIsNotNone(sm.perfect_matching([(0, 0)], [(1000, -1000)]))
        self.assertIsNone(sm.perfect_matching([(0, 0)], [(1001, 0)]))
        self.assertIsNone(sm.perfect_matching([(0, 0)], [(0, -1001)]))

    def test_duplicate_number_missing_pad_is_not_reused(self):
        mapped = sm.parse_map_pads([["1", "0mm", "0mm"], ["1", "0mm", "0mm"]])
        board = board_pads([("1", 0, 0)])
        self.assertEqual(sm.match_pads(board, mapped), (False, "1"))

    def test_lowest_natural_failing_group_and_sorted_positions(self):
        mapped = sm.parse_map_pads(
            [["10", "0mm", "0mm"], ["2", "0.002mm", "0mm"], ["2", "0mm", "0.002mm"]]
        )
        board = board_pads([("10", 50000, 0), ("2", 0, 0), ("2", 10000, 0)])
        expected = (
            "U2: frame mismatch or older pad_projection at pad 2; "
            "board=[(0nm,0nm), (10000nm,0nm)]; map=[(0nm,2000nm), (2000nm,0nm)]"
        )
        self.assertEqual(sm.check_frame("U2", board, mapped), expected)

    def test_diagnostics_do_not_depend_on_input_order(self):
        mapped = sm.parse_map_pads([["2", "0.002mm", "0mm"], ["2", "0mm", "0.002mm"]])
        board = board_pads([("2", 0, 0), ("2", 10000, 0)])
        messages = {
            sm.check_frame("U1", board, mapped),
            sm.check_frame("U1", board[::-1], mapped[::-1]),
        }
        self.assertEqual(len(messages), 1)


class MapAndCoverageTests(unittest.TestCase):
    def valid_map(self, pads=None):
        return document(pads or [["1", "0mm", "0mm"]])

    def test_validation_rejects_bad_projection_and_missing_frame(self):
        doc = self.valid_map()
        doc["pad_projection"] = 0
        with self.assertRaisesRegex(sm.MapError, "pad_projection"):
            sm.validate_map(doc)
        doc = self.valid_map()
        doc["designators"]["U1"]["footprint"] = "acme::demo::old"
        with self.assertRaisesRegex(sm.MapError, "not in footprints"):
            sm.validate_map(doc)

    def test_validation_rejects_unused_footprint(self):
        doc = self.valid_map()
        doc["footprints"]["acme::demo::unused"] = copy.deepcopy(
            doc["footprints"][FP]
        )
        with self.assertRaisesRegex(sm.MapError, "unused footprint"):
            sm.validate_map(doc)

    def test_validation_does_not_mutate_input(self):
        doc = self.valid_map()
        before = copy.deepcopy(doc)
        validated = sm.validate_map(doc)
        self.assertEqual(doc, before)
        self.assertEqual(validated["footprints"][FP][0][1], (0, 0))
        self.assertEqual(
            sm.canonical_frame(doc["footprints"][FP]["pads"]),
            "b21afe06c2f31da4337e75443ce4b7d9753dbaafce9dc20d7eb5e774dcb55774",
        )

    def test_unnumbered_board_pads_are_filtered(self):
        pads = [
            {"number": "", "xy_nm": (100000, 100000)},
            {"number": None, "xy_nm": (200000, 100000)},
            {"number": "1", "xy_nm": (0, 0)},
        ]
        self.assertEqual(
            sm.recover_pads_nm(pads, (0, 0), 0), [("1", (0, 0), 2)]
        )

    def test_frame_mismatch_names_designator(self):
        doc = self.valid_map()
        board = [{
            "designator": "U1",
            "footprint": FP,
            "anchor_xy_nm": (0, 0),
            "orientation": 0,
            "pads": [("1", (10000, 0))],
        }]
        errors, warnings, _ = sm.check_board(doc, board)
        self.assertEqual(warnings, [])
        self.assertEqual(len(errors), 1)
        self.assertIn("U1", errors[0])
        self.assertIn("pad 1", errors[0])
        self.assertIn("pad_projection", errors[0])

    def test_coverage_missing_and_extra(self):
        doc = self.valid_map()
        errors, warnings = sm.check_coverage(["U10", "U2"], ["U1", "U3"])
        self.assertEqual(
            errors,
            [
                "U2: machine-placed footprint not in this map",
                "U10: machine-placed footprint not in this map",
            ],
        )
        self.assertEqual(warnings, ["U1: map designator missing from board", "U3: map designator missing from board"])

    def test_unbound_uses_identity_not_not_in_map(self):
        doc = self.valid_map()
        doc["designators"]["U1"]["pn"] = None
        doc["designators"]["U1"]["at"] = ["4mm", "5mm"]
        doc["designators"]["U1"]["rotate"] = 37
        row = sm.validate_map(doc)["designators"][0]
        self.assertTrue(row["unbound"])
        self.assertEqual(sm.identity_mapping(row), {"at": (0, 0), "rotate": 0, "unbound": True})
        errors, _ = sm.check_coverage(["U1"], [row])
        self.assertEqual(errors, [])

    def test_bottom_both_flip_encodings_recover_front_side_frame(self):
        local = (1_000_000, 2_000_000)
        stored = (local[0], -local[1])
        for angle in (30, 210):
            radians = math.radians(angle)
            x = sm.round_nm(
                stored[0] * math.cos(radians) - stored[1] * math.sin(radians)
            )
            y = sm.round_nm(
                stored[0] * math.sin(radians) + stored[1] * math.cos(radians)
            )
            recovered = sm.recover_local_nm(
                (10_000_000 + x, 20_000_000 + y),
                (10_000_000, 20_000_000),
                angle,
                bottom=True,
            )
            self.assertEqual(recovered, local)

    def test_bottom_forward_and_inverse_both_flip_directions(self):
        for angle in (0, 90, 180, 270):
            for direction in (-1, 1):
                local = (1_000_000, 2_000_000)
                stored = (local[0], -local[1])
                actual_angle = angle if direction == 1 else (angle + 180) % 360
                actual = math.radians(actual_angle)
                x = sm.round_nm(
                    stored[0] * math.cos(actual) - stored[1] * math.sin(actual)
                )
                y = sm.round_nm(
                    stored[0] * math.sin(actual) + stored[1] * math.cos(actual)
                )
                self.assertEqual(
                    sm.recover_local_nm(
                        (x, y), (0, 0), actual_angle, bottom=True
                    ),
                    local,
                )

    def test_board_check_collects_errors_and_warnings_without_io(self):
        doc = self.valid_map()
        board = [
            {
                "designator": "U9",
                "footprint": FP,
                "anchor_xy_nm": (0, 0),
                "pads": [],
            }
        ]
        errors, warnings, _ = sm.check_board(doc, board)
        self.assertEqual(errors, ["U9: machine-placed footprint not in this map"])
        self.assertEqual(warnings, ["U1: map designator missing from board"])


if __name__ == "__main__":
    unittest.main()