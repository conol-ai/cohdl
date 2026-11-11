import hashlib
import json
import math
import os
import shutil
import struct
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "frame_boards"))
import measure_preview as mp

PREVIEWS = Path(__file__).resolve().parents[1] / "sourcing_data/jlc_previews"
SCALE, SIZE = 40, 10.0  # px per mm; the synthetic board is 10 mm square
GREEN, GREY, WHITE, COPPER = (20, 100, 40), (70, 70, 70), (255, 255, 255), (192, 128, 128)  # RGB


def write_bmp(path, paint):
    """A 24-bit BMP of the synthetic board whose pixel (x, y), y down, is paint(x, y) as (r, g, b)."""
    n = int(SIZE * SCALE)
    stride = (n * 3 + 3) & ~3
    rows = []
    for y in range(n - 1, -1, -1):
        row = bytearray()
        for x in range(n):
            r, g, b = paint(x, y)
            row += bytes((b, g, r))
        rows.append(bytes(row) + bytes(stride - n * 3))
    header = struct.pack("<2sIHHI", b"BM", 54 + stride * n, 0, 0, 54)
    info = struct.pack("<IiiHHIIiiII", 40, n, n, 1, 24, 0, stride * n, 2835, 2835, 0, 0)
    Path(path).write_bytes(header + info + b"".join(rows))


def footprint(angle=0.0, half=2.3):
    """A two-pad chip land at the board's centre, pads HALF mm either side along ANGLE."""
    c, s = math.cos(math.radians(angle)), math.sin(math.radians(angle))
    return {"ref": "R1", "fpid": "passive-CHIP_2010", "bottom": False, "x": 5.0, "y": 5.0, "angle": angle,
            "pads": [{"num": "1", "x": 5.0 - half * c, "y": 5.0 - half * s},
                     {"num": "2", "x": 5.0 + half * c, "y": 5.0 + half * s}]}


def scene(land_angle=0.0, turn=0.0, shift=(0.0, 0.0), text=True, extra=None):
    """A chip model and its reference text, as a preview draws them.

    The model is a grey body 4.0 x 2.4 mm with a white terminal 0.5 mm long at each
    end. It is drawn TURN degrees from the land's axis and SHIFT mm (along, across)
    from the land's origin. The text is a white bar 3.0 x 0.4 mm lying 0.7 mm beside
    the model, on the land's own frame as the generator places it. EXTRA(u, v), in
    the land's frame, may paint over both.
    """
    c, s = math.cos(math.radians(land_angle)), math.sin(math.radians(land_angle))
    tc, ts = math.cos(math.radians(turn)), math.sin(math.radians(turn))

    def paint(x, y):
        dx, dy = (x + 0.5) / SCALE - 5.0, (y + 0.5) / SCALE - 5.0
        u, v = dx * c + dy * s, -dx * s + dy * c  # along and across the land's pads
        if extra is not None:
            colour = extra(u, v)
            if colour is not None:
                return colour
        mu, mv = u - shift[0], v - shift[1]
        mu, mv = mu * tc + mv * ts, -mu * ts + mv * tc  # the model's own frame
        if abs(mu) <= 2.0 and abs(mv) <= 1.2:
            return GREY
        if abs(mu) <= 2.5 and abs(mv) <= 1.2:
            return WHITE
        if text and abs(u) <= 1.5 and -2.3 <= v <= -1.9:
            return WHITE
        return GREEN
    return paint


def read(paint, fp, method):
    with tempfile.TemporaryDirectory() as td:
        write_bmp(Path(td) / "top.bmp", paint)
        view = mp.View(str(Path(td) / "top.bmp"), (0.0, 0.0, SIZE, SIZE), False)
        return mp.body(view, fp, method)


class BodyMethodTests(unittest.TestCase):
    def test_method_1_reads_the_text_with_the_model_and_method_2_the_model_alone(self):
        for angle in (0, 90, 37):
            with self.subTest(angle=angle):
                fp, paint = footprint(angle), scene(angle)
                c, s = math.cos(math.radians(angle)), math.sin(math.radians(angle))
                both, model = read(paint, fp, 1), read(paint, fp, 2)
                # The text pulls method 1's centroid toward itself, across the pads, and turns with the part.
                self.assertLess(-both["offset_xy_mm"][0] * s + both["offset_xy_mm"][1] * c, -0.1)
                self.assertNotIn("method", both)
                self.assertEqual((model["method"], mp.body_method(model), mp.body_method(both)), (2, 2, 1))
                self.assertLess(model["offset_mm"], 0.02)
                self.assertLess(abs(model["delta_mod_180"]), 0.5)
                self.assertAlmostEqual(model["size_mm"][0], 5.0, delta=0.06)
                self.assertAlmostEqual(model["size_mm"][1], 2.4, delta=0.06)
                self.assertGreater(model["window_px"], model["model_px"])
                self.assertTrue(mp.body_ok(model))

    def test_method_2_still_fails_a_model_that_is_off_its_pads_or_turned(self):
        # The limit for this land is 0.25 mm and 10 degrees. The text is beside every one.
        fp = footprint()
        self.assertEqual(mp.chip_offset_limit(2.3), 0.25)
        cases = [
            # (turn, shift along and across, passes, what the reading must show)
            (0.0, (0.0, 0.0), True, lambda r: r["offset_mm"] < 0.02),
            (5.0, (0.0, 0.0), True, lambda r: abs(r["delta_mod_180"] - 5.0) < 0.5),
            (0.0, (0.2, 0.0), True, lambda r: abs(r["offset_mm"] - 0.2) < 0.02),
            (0.0, (0.4, 0.0), False, lambda r: abs(r["offset_xy_mm"][0] - 0.4) < 0.02),  # along its pads
            (0.0, (0.0, 0.3), False, lambda r: abs(r["offset_xy_mm"][1] - 0.3) < 0.02),  # across them
            (0.0, (0.0, -0.3), False, lambda r: abs(r["offset_xy_mm"][1] + 0.3) < 0.02),  # toward its text
            (12.0, (0.0, 0.0), False, lambda r: abs(r["delta_mod_180"] - 12.0) < 0.5),
            # Turned this far the model's corner runs into its text, which is then read with
            # it (the method's stated limit); the reading is 30 degrees, not 25, and fails.
            (-25.0, (0.0, 0.0), False, lambda r: r["delta_mod_180"] < -20),
            (90.0, (0.0, 0.0), False, lambda r: abs(r["delta_mod_180"]) > 89),  # a quarter turn
        ]
        for turn, shift, passes, shows in cases:
            with self.subTest(turn=turn, shift=shift):
                reading = read(scene(turn=turn, shift=shift), fp, 2)
                self.assertTrue(shows(reading), reading)
                self.assertEqual(mp.body_ok(reading), passes, reading)

    def test_text_that_grazes_the_model_is_cut_off_and_text_that_overlaps_it_is_not(self):
        fp = footprint()

        def bridge(width_px):  # a white run from the text to the model's edge, WIDTH_PX wide
            def extra(u, v):
                x = round((u + 5.0) * SCALE - 0.5)
                return WHITE if 200 <= x < 200 + width_px and -1.9 < v < -1.2 else None
            return extra
        for width in (1, 2):
            with self.subTest(width=width):
                reading = read(scene(extra=bridge(width)), fp, 2)
                self.assertLess(reading["offset_mm"], 0.02)
                self.assertAlmostEqual(reading["size_mm"][1], 2.4, delta=0.06)
        # The stated limit: joined along 3 px or more, the text is read as model. The reading
        # moves toward it (and here the model's extent across the pads grows by the text's).
        joined = read(scene(extra=bridge(20)), fp, 2)
        self.assertGreater(joined["offset_mm"], 0.1)
        self.assertGreater(joined["size_mm"][1], 3.0)

    def test_a_thin_run_of_copper_colour_does_not_split_the_model_and_a_pad_is_not_the_model(self):
        fp = footprint()

        def seam(u, v):  # the export's blend where a red designator meets a white terminal: 1 px, copper-coloured
            x = round((u + 5.0) * SCALE - 0.5)
            return COPPER if x in (120, 279) and abs(v) <= 1.2 else None
        whole = read(scene(extra=seam), fp, 2)
        self.assertAlmostEqual(whole["size_mm"][0], 5.0, delta=0.06)
        self.assertLess(whole["offset_mm"], 0.02)

        def pads(u, v):  # bare copper showing 0.5 mm beyond one terminal, as on a land longer than its model
            return COPPER if 2.5 < u <= 3.0 and abs(v) <= 1.2 else None
        model = read(scene(extra=pads), fp, 2)
        self.assertAlmostEqual(model["size_mm"][0], 5.0, delta=0.06)
        self.assertLess(model["offset_mm"], 0.02)

    def test_a_model_drawn_away_from_the_origin_gives_no_reading(self):
        # Nothing drawn at the origin: no angle, no offset, and so no pass.
        reading = read(scene(shift=(0.0, 2.0), text=False), footprint(), 2)
        self.assertEqual((reading["method"], reading["model_px"]), (2, 0))
        self.assertNotIn("offset_mm", reading)
        self.assertFalse(mp.body_ok(reading))

    def test_measure_records_the_method_and_method_1_writes_what_it_always_wrote(self):
        doc = {"bbox": [-0.05, -0.05, SIZE + 0.05, SIZE + 0.05], "fps": [footprint()]}
        with tempfile.TemporaryDirectory() as td:
            write_bmp(Path(td) / "top.bmp", scene())
            (Path(td) / "board.json").write_text(json.dumps(doc))
            args = [str(Path(td) / "board.json"), str(Path(td) / "top.bmp"), "-"]
            mp.measure(*args, str(Path(td) / "two.json"))
            mp.measure(*args, str(Path(td) / "one.json"), method=1)
            (two,), (one,) = (json.loads((Path(td) / name).read_text()) for name in ("two.json", "one.json"))
        self.assertEqual(mp.BODY_METHOD, 2)
        self.assertEqual(two["body"]["method"], 2)
        self.assertEqual(list(one["body"]), ["delta_mod_180", "offset_mm", "offset_xy_mm", "offset_limit_mm",
                                             "elongation", "fill"])
        self.assertEqual({k: v for k, v in two.items() if k not in ("body", "body_ok")},
                         {k: v for k, v in one.items() if k not in ("body", "body_ok")})


# Each W3 record: its board.json, its exports, the method 1 readings W3 used, and the method 2 readings.
RECORDS = {
    "2026-11-05-c2-board-3": ("board.json", ("top.png", "bottom.png"), "remeasure-2026-11-10.json",
                              "feb3a0e23542dadeaa5fbbcfdf91eb2d6482b36bd3dadf55194348a4fed9110e"),
    "2026-11-07-w3-pilot-0402": ("board.json", ("top.png", None), "remeasure-2026-11-10.json",
                                 "adf69a89a6a6c31bb0c261a443897ca1bdd931e17718a3cf6a38a47b16de0935"),
    "2026-11-11-board-4": ("package/board.json", ("top.png", "bottom.png"), "measurements.json",
                           "963ac558c9e2589efa5fe0a17e9ed71df7b1147c1c5b1133abac7405e0a509fc"),
}
METHOD_2 = "remeasure-2026-11-11-method-2.json"


class ArchivedReadingsTests(unittest.TestCase):
    def test_the_method_2_readings_are_new_files_beside_the_method_1_ones(self):
        for name, (_, _, method_1, digest) in RECORDS.items():
            with self.subTest(record=name):
                record = PREVIEWS / name
                self.assertEqual(hashlib.sha256((record / METHOD_2).read_bytes()).hexdigest(), digest)
                index = json.loads((record / "w3-readings.json").read_text(encoding="utf-8"))
                self.assertEqual(index["readings"], {"2": METHOD_2})
                self.assertEqual(json.loads((record / "w3.json").read_text(encoding="utf-8"))["measurements"], method_1)
                one = json.loads((record / method_1).read_text())
                two = json.loads((record / METHOD_2).read_text())
                # The same rows; only how a chip body is read differs.
                strip = lambda rows: [{k: v for k, v in row.items() if k not in ("body", "body_ok")} for row in rows]
                self.assertEqual(strip(one), strip(two))
                self.assertEqual([("body" in a) for a in one], [("body" in b) for b in two])
                bodies = [row["body"] for row in two if "body" in row]
                self.assertTrue(bodies and all(b["method"] == 2 for b in bodies))
                self.assertFalse(any("method" in row["body"] for row in one if "body" in row))
                self.assertTrue(all(row["body_ok"] == mp.body_ok(row["body"]) for row in two if "body" in row))

    def test_what_method_2_reads_on_the_three_records(self):
        def bodies(name, file):
            return {row["ref"]: row for row in json.loads((PREVIEWS / name / file).read_text()) if "body" in row}
        # Board 4: 67 of 204 drawn chip bodies failed method 1's check; none fails method 2's.
        one, two = bodies("2026-11-11-board-4", "measurements.json"), bodies("2026-11-11-board-4", METHOD_2)
        self.assertEqual((len(two), sum(not r["body_ok"] for r in one.values()), sum(not r["body_ok"] for r in two.values())),
                         (204, 67, 0))
        self.assertEqual(max(abs(r["body"]["delta_mod_180"]) for r in one.values()), 78.5)
        self.assertEqual(max(abs(r["body"]["delta_mod_180"]) for r in two.values()), 6.3)
        self.assertEqual(max(r["body"]["offset_mm"] for r in two.values()), 0.053)
        # R68, the 0201 whose text grazes its model: 1.54 of its limit by method 1, 0.21 by method 2,
        # and 0.55 mm long, terminals included (the record's own tool read 0.42 mm of it).
        self.assertEqual((one["R68"]["body"]["offset_mm"], two["R68"]["body"]["offset_mm"], two["R68"]["body"]["size_mm"][0]),
                         (0.123, 0.017, 0.55))
        # The pilot: its largest offset was 0.121 mm of 0.128; it is 0.040.
        one, two = bodies("2026-11-07-w3-pilot-0402", "remeasure-2026-11-10.json"), bodies("2026-11-07-w3-pilot-0402", METHOD_2)
        self.assertEqual((len(two), max(r["body"]["offset_mm"] for r in one.values()),
                          max(r["body"]["offset_mm"] for r in two.values())), (89, 0.121, 0.04))
        self.assertEqual(max(abs(r["body"]["delta_mod_180"]) for r in two.values()), 2.6)
        # Board 3: one body fails method 2's own check, R4 (0201 at -143 deg), 0.099 mm against 0.080.
        one, two = bodies("2026-11-05-c2-board-3", "remeasure-2026-11-10.json"), bodies("2026-11-05-c2-board-3", METHOD_2)
        self.assertEqual([ref for ref, r in two.items() if not r["body_ok"]], ["R4"])
        self.assertEqual((one["R4"]["body_ok"], one["R4"]["body"]["offset_mm"], two["R4"]["body"]["offset_mm"],
                          two["R4"]["body"]["offset_limit_mm"]), (True, 0.07, 0.099, 0.08))

    @unittest.skipUnless(os.environ.get("COHDL_REMEASURE_PREVIEWS") and shutil.which("sips"),
                         "set COHDL_REMEASURE_PREVIEWS=1 on macOS to re-measure the archived exports (about a minute)")
    def test_each_record_reproduces_byte_for_byte_under_the_method_it_was_made_with(self):
        for name, (board, (top, bottom), method_1, _) in RECORDS.items():
            with self.subTest(record=name), tempfile.TemporaryDirectory() as td:
                record = PREVIEWS / name
                shots = []
                for shot in (top, bottom):
                    if shot is None:
                        shots.append("-")
                        continue
                    shots.append(str(Path(td) / (Path(shot).stem + ".bmp")))
                    subprocess.run(["sips", "-s", "format", "bmp", str(record / shot), "--out", shots[-1]],
                                   check=True, capture_output=True)
                for method, archived in ((1, method_1), (2, METHOD_2)):
                    out = Path(td) / ("method-%d.json" % method)
                    mp.measure(str(record / board), *shots, str(out), method=method)
                    self.assertEqual(out.read_bytes(), (record / archived).read_bytes(), (name, method))


if __name__ == "__main__":
    unittest.main()
