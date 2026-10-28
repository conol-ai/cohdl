"""Board/compiler integration tests using a test-only KiCad S-expression reader."""

import csv
from decimal import Decimal
import io
import json
import math
from pathlib import Path
import re
import sys
import unittest

TOOLS = Path(__file__).resolve().parents[1]
ROOT = TOOLS.parent
sys.path.insert(0, str(TOOLS))

import smt_pos_map as sm  # noqa: E402
from smt_pos_lib import (  # noqa: E402
    neutral_csv,
    neutral_origin,
    outline_geometry_bounds,
)

FIXTURE = ROOT / "tests/fixtures/rfc034"
BOARD = FIXTURE / "map_integration.kicad_pcb"
MAP = FIXTURE / "map_integration-acme-map.json"
CPL = FIXTURE / "map_integration-acme-cpl.csv"
NM = Decimal(1_000_000)


def atoms(text):
    return re.findall(r'\(|\)|"(?:\\.|[^"\\])*"|[^\s()]+', text)


def parse_sexpr(text):
    stack = [[]]
    for token in atoms(text):
        if token == "(":
            child = []
            stack[-1].append(child)
            stack.append(child)
        elif token == ")":
            stack.pop()
        elif token.startswith('"'):
            stack[-1].append(json.loads(token))
        else:
            stack[-1].append(token)
    if len(stack) != 1:
        raise AssertionError("unbalanced test board")
    return stack[0][0]


def children(node, name):
    return [child for child in node if isinstance(child, list) and child[0] == name]


def child(node, name):
    found = children(node, name)
    return found[0] if found else None


def nm(text):
    return int((Decimal(text) * NM).to_integral_value())


def point(node):
    return nm(node[1]), nm(node[2])


def read_board(path):
    """Read only the compiler-emitted board forms used by this fixture."""
    board = parse_sexpr(path.read_text(encoding="utf-8"))
    shapes = []
    for segment in children(board, "segment") + children(board, "gr_line"):
        start, end = point(child(segment, "start")), point(child(segment, "end"))
        shapes.append(
            {
                "shape": "segment",
                "start": start,
                "end": end,
                "width": nm(child(child(segment, "stroke"), "width")[1]),
                "bbox": [min(start[0], end[0]), min(start[1], end[1]),
                         max(start[0], end[0]), max(start[1], end[1])],
            }
        )
    footprints = []
    for node in children(board, "footprint"):
        at = child(node, "at")
        reference = next(prop for prop in children(node, "property") if prop[1] == "Reference")
        bottom = child(node, "layer")[1] == "B.Cu"
        pads = []
        for pad in children(node, "pad"):
            local = point(child(pad, "at"))
            radians = math.radians(float(at[3]) if len(at) > 3 else 0)
            cosine, sine = math.cos(radians), math.sin(radians)
            position = (
                point(at)[0] + sm.round_nm(local[0] * cosine + local[1] * sine),
                point(at)[1] + sm.round_nm(local[1] * cosine - local[0] * sine),
            )
            layers = child(pad, "layers")[1:]
            pads.append(
                {
                    "number": pad[1],
                    "position": position,
                    "smd": pad[2] == "smd",
                    "copper": any(layer.endswith(".Cu") or layer == "*.Cu" for layer in layers),
                }
            )
        footprints.append(
            {
                "ref": reference[2],
                "value": next(prop[2] for prop in children(node, "property") if prop[1] == "Value"),
                "footprint": node[1],
                "anchor": point(at),
                "orientation": float(at[3]) if len(at) > 3 else 0,
                "flipped": bottom,
                "pads": pads,
            }
        )
    origin = neutral_origin(outline_geometry_bounds(shapes))
    return footprints, origin


def mapped_cpl(board_path=BOARD, map_path=MAP):
    footprints, origin = read_board(board_path)
    document = json.loads(map_path.read_text(encoding="utf-8"))
    board_records = [
        {
            "designator": fp["ref"],
            "footprint": fp["footprint"],
            "anchor_xy_nm": fp["anchor"],
            "orientation": fp["orientation"],
            "bottom": fp["flipped"],
            "machine_placed": any(p["smd"] and p["number"] for p in fp["pads"]),
            "pads": fp["pads"],
        }
        for fp in footprints
    ]
    validated = sm.validate_map(document)
    errors, warnings, mappings = sm.check_board(document, board_records)
    return errors, warnings, neutral_csv(
        sm.mapped_neutral_rows(
            footprints, mappings, origin, validated["bottom_rotation"]
        )
    )


class CompilerFixtureTests(unittest.TestCase):
    def test_supplier_cpl_matches_compiler_including_bottom_angles(self):
        errors, warnings, actual = mapped_cpl()
        self.assertEqual(errors, [])
        self.assertEqual(warnings, [])
        expected_rows = list(csv.DictReader(io.StringIO(CPL.read_text(encoding="utf-8"))))
        actual_rows = list(csv.DictReader(io.StringIO(actual)))
        self.assertEqual(actual, CPL.read_text(encoding="utf-8"))
        self.assertEqual(len(actual_rows), len(expected_rows))
        for expected, row in zip(expected_rows, actual_rows):
            self.assertEqual(row["Designator"], expected["Designator"])
            self.assertEqual(row["Layer"], expected["Layer"])
            self.assertEqual(row["Rotation"], expected["Rotation"])
            for column in ("Mid X", "Mid Y"):
                self.assertLessEqual(
                    abs(Decimal(row[column][:-2]) - Decimal(expected[column][:-2])),
                    Decimal("0.0001"),
                )

    def test_mirrored_fixture_matches_independent_reference_byte_for_byte(self):
        board = FIXTURE / "bottom_preview.kicad_pcb"
        supplier_map = FIXTURE / "bottom_preview-acme-map.json"
        reference = FIXTURE / "bottom-preview/handoff/cpl-acme-reference.csv"
        compiler_cpl = FIXTURE / "bottom_preview-acme-cpl.csv"
        self.assertEqual(compiler_cpl.read_bytes(), reference.read_bytes())
        errors, warnings, actual = mapped_cpl(board, supplier_map)
        self.assertEqual(errors, [])
        self.assertEqual(warnings, [])
        self.assertEqual(actual.encode(), reference.read_bytes())

    def test_schema_one_remains_kicad_and_unknown_v2_convention_fails(self):
        document = json.loads(MAP.read_text(encoding="utf-8"))
        document["schema_version"] = 1
        del document["bottom_rotation"]
        self.assertEqual(sm.validate_map(document)["bottom_rotation"], "kicad")
        document["schema_version"] = 2
        for invalid in ("unknown", None, True):
            document["bottom_rotation"] = invalid
            with self.subTest(invalid=invalid), self.assertRaises(sm.MapError):
                sm.validate_map(document)
        document["bottom_rotation"] = "kicad"
        document["schema_version"] = 3
        with self.assertRaises(sm.MapError):
            sm.validate_map(document)

    def test_moving_one_pad_by_0_01mm_fails_frame_check(self):
        import tempfile
        with tempfile.TemporaryDirectory() as directory:
            moved = Path(directory) / BOARD.name
            moved.write_text(BOARD.read_text(encoding="utf-8").replace("(at 1 1.3375 180)", "(at 1.01 1.3375 180)", 1))
            errors, _, _ = mapped_cpl(moved)
        self.assertEqual(len(errors), 1)
        self.assertIn("U1: frame mismatch or older pad_projection at pad 3", errors[0])


if __name__ == "__main__":
    unittest.main()
