"""Standard-library tests for smt_pos; no KiCad or pytest installation needed."""

import contextlib
import io
from pathlib import Path
import subprocess
import sys
import tempfile
import types
import unittest
from unittest import mock


TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))

import smt_pos  # noqa: E402
from smt_pos_lib import (  # noqa: E402
    legacy_csv,
    legacy_origin,
    legacy_rows,
    natural_key,
    neutral_csv,
    neutral_mm,
    neutral_origin,
    neutral_rows,
    normalize_rotation,
    outline_geometry_bounds,
)


NM = 1_000_000
HEADER = "Designator,Mid X,Mid Y,Layer,Rotation\n"


def to_mm(value):
    return value / NM


def pad(number, x, y, kind="smd"):
    return {
        "number": number,
        "position": (x * NM, y * NM),
        "smd": kind == "smd",
        "copper": kind != "npth",
    }


def footprint(
    ref,
    x,
    y,
    angle=0,
    flipped=False,
    value="part",
    pads=None,
    footprint_name="FP",
):
    if pads is None:
        pads = [pad("1", x, y)]
    return {
        "ref": ref,
        "value": value,
        "anchor": (x * NM, y * NM),
        "orientation": angle,
        "flipped": flipped,
        "footprint": footprint_name,
        "pads": pads,
    }


class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y


def rectangle_drawings(width_nm):
    corners = [(0, 0), (50 * NM, 0), (50 * NM, 30 * NM), (0, 30 * NM)]
    half_width = width_nm // 2
    records = []
    for index, start in enumerate(corners):
        end = corners[(index + 1) % len(corners)]
        records.append(
            {
                "shape": "segment",
                "start": start,
                "end": end,
                "width": width_nm,
                "bbox": (
                    min(start[0], end[0]) - half_width,
                    min(start[1], end[1]) - half_width,
                    max(start[0], end[0]) + half_width,
                    max(start[1], end[1]) + half_width,
                ),
            }
        )
    return records


class FakePad:
    def __init__(self, record):
        self.record = record

    def GetAttribute(self):
        if not self.record["copper"]:
            return FakePcbnew.PAD_ATTRIB_NPTH
        return (
            FakePcbnew.PAD_ATTRIB_SMD
            if self.record["smd"]
            else FakePcbnew.PAD_ATTRIB_PTH
        )

    def GetPadName(self):
        return self.record["number"]

    def GetPosition(self):
        return Point(*self.record["position"])


class FakeFootprint:
    def __init__(self, record):
        self.record = record

    def GetReference(self):
        return self.record["ref"]

    def GetValue(self):
        return self.record["value"]

    def GetPosition(self):
        return Point(*self.record["anchor"])

    def GetOrientationDegrees(self):
        return self.record["orientation"]

    def IsFlipped(self):
        return self.record["flipped"]

    def GetFPIDAsString(self):
        return "library:prefix-" + self.record["footprint"]

    def Pads(self):
        return [FakePad(record) for record in self.record["pads"]]


class FakeBox:
    def __init__(self, bounds):
        self.bounds = bounds

    def GetLeft(self):
        return self.bounds[0]

    def GetTop(self):
        return self.bounds[1]

    def GetRight(self):
        return self.bounds[2]

    def GetBottom(self):
        return self.bounds[3]


class FakeDrawing:
    def __init__(self, layer, record=None):
        self.layer = layer
        self.record = record or {
            "shape": "segment",
            "start": (-15 * NM, -10 * NM),
            "end": (15 * NM, 10 * NM),
            "width": 0,
            "bbox": (-15 * NM, -10 * NM, 15 * NM, 10 * NM),
        }

    def GetLayer(self):
        return self.layer

    def GetShape(self):
        return getattr(FakePcbnew, "SHAPE_T_" + self.record["shape"].upper())

    def GetStart(self):
        return Point(*self.record["start"])

    def GetEnd(self):
        return Point(*self.record["end"])

    def GetWidth(self):
        return self.record["width"]

    def GetBoundingBox(self):
        return FakeBox(self.record["bbox"])

    def GetArcMid(self):
        return Point(*self.record["mid"])

    def GetCenter(self):
        return Point(*self.record["center"])

    def GetPolyPoints(self):
        return [Point(*point) for point in self.record["points"]]


class FakeBoard:
    def __init__(
        self,
        records,
        bounds=(-15 * NM, -10 * NM, 15 * NM, 10 * NM),
        outline=True,
        drawings=None,
    ):
        self.records = records
        self.bounds = bounds
        self.outline = outline
        self.drawings = drawings

    def GetBoardEdgesBoundingBox(self):
        return FakeBox(self.bounds)

    def GetDrawings(self):
        if self.drawings is not None:
            return [FakeDrawing(FakePcbnew.Edge_Cuts, record) for record in self.drawings]
        return [FakeDrawing(FakePcbnew.Edge_Cuts)] if self.outline else []

    def GetFootprints(self):
        return [FakeFootprint(record) for record in self.records]


class FakePcbnew(types.SimpleNamespace):
    PAD_ATTRIB_SMD = 0
    PAD_ATTRIB_PTH = 1
    PAD_ATTRIB_NPTH = 2
    SHAPE_T_SEGMENT = 0
    SHAPE_T_RECT = 1
    SHAPE_T_ARC = 2
    SHAPE_T_CIRCLE = 3
    SHAPE_T_POLY = 4
    Edge_Cuts = 44

    def __init__(self, board):
        super().__init__(
            PAD_ATTRIB_SMD=self.PAD_ATTRIB_SMD,
            PAD_ATTRIB_PTH=self.PAD_ATTRIB_PTH,
            PAD_ATTRIB_NPTH=self.PAD_ATTRIB_NPTH,
            SHAPE_T_SEGMENT=self.SHAPE_T_SEGMENT,
            SHAPE_T_RECT=self.SHAPE_T_RECT,
            SHAPE_T_ARC=self.SHAPE_T_ARC,
            SHAPE_T_CIRCLE=self.SHAPE_T_CIRCLE,
            SHAPE_T_POLY=self.SHAPE_T_POLY,
            Edge_Cuts=self.Edge_Cuts,
            ToMM=to_mm,
            LoadBoard=lambda _path: board,
        )


class PureFunctionTests(unittest.TestCase):
    def test_legacy_golden_preserves_all_eleven_columns(self):
        part = footprint(
            "U2",
            -15,
            -10,
            angle=-1,
            value="chip",
            pads=[pad("1", -14, -9), pad("2", -13, -9)],
        )
        rows, skipped = legacy_rows(
            [part], legacy_origin(-15.0, 10.0), to_mm
        )
        self.assertEqual(skipped, [])
        self.assertEqual(
            legacy_csv(rows),
            "Designator,Footprint,Mid X,Mid Y,Ref X,Ref Y,Pad X,Pad Y,Layer,Rotation,Comment\n"
            "U2,FP,1.5mm,19mm,0mm,20mm,1mm,19mm,T,359,chip\n",
        )

    def test_legacy_all_and_default_filtering_are_unchanged(self):
        records = [
            footprint("TP1", 0, 0),
            footprint("U2", 1, 1, value="TouchSensor"),
            footprint("U3", 2, 2, pads=[pad("1", 2, 2, "pth")]),
            footprint("U4", 3, 3, pads=[pad("", 3, 3)]),
            footprint("M1", 4, 4, pads=[]),
        ]
        rows, skipped = legacy_rows(records, (0.0, 0.0), to_mm)
        self.assertEqual([row[0] for row in rows], ["U4"])
        self.assertEqual([item[0] for item in skipped], ["TP1", "U2", "U3", "M1"])
        all_rows, all_skipped = legacy_rows(
            records, (0.0, 0.0), to_mm, everything=True
        )
        self.assertEqual(len(all_rows), 5)
        self.assertEqual(all_skipped, [])
        mount = next(row for row in all_rows if row[0] == "M1")
        self.assertEqual((mount[2], mount[3], mount[6], mount[7]), (4.0, -4.0, 4.0, -4.0))

    def test_neutral_matches_3b_outline_top_bottom_and_natural_order(self):
        # KiCad angles for top R359 and bottom R37 are -1 and -143.
        records = [
            footprint("U2", -15, -10, angle=-1),
            footprint("U1", 15, 10, angle=-143, flipped=True),
        ]
        origin = neutral_origin((-15 * NM, -10 * NM, 15 * NM, 10 * NM))
        rows, skipped = neutral_rows(records, origin)
        self.assertEqual(skipped, [])
        self.assertEqual(
            neutral_csv(rows),
            HEADER
            + "U1,30.0000mm,0.0000mm,Bottom,217\n"
            + "U2,0.0000mm,20.0000mm,Top,359\n",
        )

    def test_neutral_asymmetric_outline_and_no_outline(self):
        part = footprint("U1", 10, 5)
        with_outline = neutral_rows(
            [part], neutral_origin((0, 0, 40 * NM, 25 * NM))
        )[0]
        without_outline = neutral_rows([part], neutral_origin(None))[0]
        self.assertEqual(
            neutral_csv(with_outline),
            HEADER + "U1,10.0000mm,20.0000mm,Top,0\n",
        )
        self.assertEqual(
            neutral_csv(without_outline),
            HEADER + "U1,10.0000mm,-5.0000mm,Top,0\n",
        )

    def test_neutral_anchor_is_not_a_pad_centroid(self):
        part = footprint(
            "U1", 1, 2, pads=[pad("1", 8, 9), pad("2", 10, 11)]
        )
        row = neutral_rows([part], (0, 0))[0][0]
        self.assertEqual(row[1:3], (1 * NM, -2 * NM))

    def test_neutral_numbered_smd_filter_keeps_tp_touch_and_mixed(self):
        records = [
            footprint("TP1", 0, 0),
            footprint("U1", 0, 0, value="TouchSensor"),
            footprint("U3", 0, 0, value="TouchPad"),
            footprint(
                "U10", 0, 0, pads=[pad("1", 0, 0), pad("2", 0, 0, "pth")]
            ),
            footprint("U2", 0, 0, pads=[pad("1", 0, 0, "pth")]),
            footprint("U11", 0, 0, pads=[pad("", 0, 0)]),
            footprint("U12", 0, 0, pads=[]),
        ]
        rows, skipped = neutral_rows(records, (0, 0))
        self.assertEqual([row[0] for row in rows], ["TP1", "U1", "U3", "U10"])
        self.assertEqual([item[0] for item in skipped], ["U2", "U11", "U12"])

    def test_neutral_unnumbered_aperture_on_through_hole_does_not_count(self):
        part = footprint(
            "U1", 0, 0, pads=[pad("", 0, 0), pad("1", 0, 0, "pth")]
        )
        rows, _ = neutral_rows([part], (0, 0))
        self.assertEqual(neutral_csv(rows), HEADER)

    def test_arbitrary_angle_normalization(self):
        cases = [(-1, 359), (-143, 217), (-181, 179), (0, 0), (359, 359), (360, 0), (721, 1)]
        for angle, expected in cases:
            with self.subTest(angle=angle):
                self.assertEqual(normalize_rotation(angle), expected)

    def test_fixed_four_place_mm_rounds_half_away_from_zero(self):
        cases = [
            (0, "0.0000mm"),
            (-1, "0.0000mm"),
            (49, "0.0000mm"),
            (50, "0.0001mm"),
            (-50, "-0.0001mm"),
            (1_234_449, "1.2344mm"),
            (1_234_450, "1.2345mm"),
            (-1_234_450, "-1.2345mm"),
            (12_345_000, "12.3450mm"),
        ]
        for value, expected in cases:
            with self.subTest(value=value):
                self.assertEqual(neutral_mm(value), expected)

    def test_natural_sort_handles_multiple_numbers(self):
        refs = ["U10", "U2", "U1", "D2A10", "D2A2"]
        self.assertEqual(
            sorted(refs, key=natural_key), ["D2A2", "D2A10", "U1", "U2", "U10"]
        )
        rows = [(ref, 0, 0, "Top", 0) for ref in refs[:3]]
        self.assertEqual(
            neutral_csv(rows),
            HEADER
            + "U1,0.0000mm,0.0000mm,Top,0\n"
            + "U2,0.0000mm,0.0000mm,Top,0\n"
            + "U10,0.0000mm,0.0000mm,Top,0\n",
        )


class CliTests(unittest.TestCase):
    def run_main(self, board, arguments):
        with mock.patch.dict(sys.modules, {"pcbnew": FakePcbnew(board)}):
            with mock.patch.object(sys, "argv", ["smt_pos.py"] + arguments):
                with contextlib.redirect_stdout(io.StringIO()):
                    smt_pos.main()

    def test_cli_default_names_and_legacy_golden(self):
        records = [
            footprint(
                "U2",
                -15,
                -10,
                angle=-1,
                value="chip",
                pads=[pad("1", -14, -9), pad("2", -13, -9)],
            ),
            footprint("M1", 4, 4, pads=[]),
        ]
        with tempfile.TemporaryDirectory() as directory:
            board_path = Path(directory) / "demo.kicad_pcb"
            board = FakeBoard(records)
            self.run_main(board, [str(board_path)])
            self.assertEqual(
                board_path.with_name("demo-smt.csv").read_text(),
                "Designator,Footprint,Mid X,Mid Y,Ref X,Ref Y,Pad X,Pad Y,Layer,Rotation,Comment\n"
                "U2,FP,1.5mm,19mm,0mm,20mm,1mm,19mm,T,359,chip\n",
            )
            self.run_main(board, ["--all", str(board_path)])
            all_csv = board_path.with_name("demo-pos-all.csv").read_text()
            self.assertIn("M1,FP,", all_csv)
            self.run_main(board, ["--neutral", str(board_path)])
            self.assertEqual(
                board_path.with_name("demo-cpl.csv").read_text(),
                HEADER + "U2,0.0000mm,20.0000mm,Top,359\n",
            )

    def test_cli_explicit_output_and_no_outline(self):
        with tempfile.TemporaryDirectory() as directory:
            board_path = Path(directory) / "demo.kicad_pcb"
            output = Path(directory) / "chosen.csv"
            board = FakeBoard([footprint("U1", 10, 5)], outline=False)
            self.run_main(board, [str(board_path), "--neutral", str(output)])
            self.assertEqual(
                output.read_text(),
                HEADER + "U1,10.0000mm,-5.0000mm,Top,0\n",
            )

    def test_neutral_ignores_outline_stroke_but_legacy_keeps_inflated_bbox(self):
        records = [footprint("U1", 10, 10)]
        with tempfile.TemporaryDirectory() as directory:
            board_path = Path(directory) / "board.kicad_pcb"
            outputs = []
            for width_nm in (0, 100_000):
                drawings = rectangle_drawings(width_nm)
                half_width = width_nm // 2
                board = FakeBoard(
                    records,
                    bounds=(-half_width, -half_width, 50 * NM + half_width, 30 * NM + half_width),
                    drawings=drawings,
                )
                neutral_output = Path(directory) / f"neutral-{width_nm}.csv"
                legacy_output = Path(directory) / f"legacy-{width_nm}.csv"
                self.run_main(board, ["--neutral", str(board_path), str(neutral_output)])
                self.run_main(board, [str(board_path), str(legacy_output)])
                outputs.append(neutral_output.read_text())
                self.assertEqual(
                    legacy_output.read_text().splitlines()[1].split(",")[2:4],
                    ["10mm", "20mm"] if width_nm == 0 else ["10.05mm", "20.05mm"],
                )
            self.assertEqual(outputs[0], outputs[1])
            self.assertEqual(outputs[0], HEADER + "U1,10.0000mm,20.0000mm,Top,0\n")

    def test_arc_bounds_include_extrema_beyond_endpoints(self):
        # Clockwise in KiCad's +y-down frame: right -> bottom -> left.
        arc = {
            "shape": "arc",
            "start": (10 * NM, 0),
            "mid": (0, 10 * NM),
            "end": (-10 * NM, 0),
            "center": (0, 0),
        }
        self.assertEqual(
            outline_geometry_bounds([arc]),
            [-10 * NM, 0, 10 * NM, 10 * NM],
        )
        # The other direction through the same mid point is also a half turn.
        arc["start"], arc["end"] = arc["end"], arc["start"]
        self.assertEqual(
            outline_geometry_bounds([arc]),
            [-10 * NM, 0, 10 * NM, 10 * NM],
        )

    def test_all_and_neutral_are_usage_errors_without_kicad(self):
        for arguments in (
            ["--all", "--neutral", "board.kicad_pcb"],
            ["--neutral", "--all", "board.kicad_pcb"],
        ):
            with self.subTest(arguments=arguments):
                completed = subprocess.run(
                    [sys.executable, str(TOOLS / "smt_pos.py")] + arguments,
                    text=True,
                    capture_output=True,
                    check=False,
                )
                self.assertEqual(completed.returncode, 2)
                self.assertIn("not allowed with", completed.stderr)
                self.assertNotIn("pcbnew", completed.stderr)

    def test_top_level_import_does_not_require_pcbnew(self):
        completed = subprocess.run(
            [
                sys.executable,
                "-c",
                "import sys; sys.path.insert(0, sys.argv[1]); import smt_pos",
                str(TOOLS),
            ],
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)


if __name__ == "__main__":
    unittest.main()
