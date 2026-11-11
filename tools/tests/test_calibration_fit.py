import contextlib
import io
import json
import math
import random
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "frame_boards"))
import calibration_fit as cf

ANGLES = (0, 90, 180, 270, 37)
# Synthetic calibration parts: part number, land, pad half-spacing.
PARTS = [("C1", "passive-CHIP_0402", 0.51), ("C2", "passive-CHIP_0402", 0.51), ("C3", "passive-CHIP_2010", 2.2)]


def to_board(local, angle, bottom):
    """A footprint-local vector on the board, as KiCad places a footprint: a bottom
    footprint is mirrored in x, then either turns counter-clockwise as seen from above
    (y runs down). board.json's pads follow this; so must a model's own offset."""
    x, y = (-local[0], local[1]) if bottom else local
    c, s = math.cos(math.radians(angle)), math.sin(math.radians(angle))
    return (x * c + y * s, -x * s + y * c)


def synthetic(export, model, noise=0.0, seed=1, parts=PARTS, angles=ANGLES, sides=("top", "bottom"), skip=()):
    """board.json, measurements and a BOM for PARTS at ANGLES on SIDES.

    EXPORT maps a view to its shift; MODEL maps a part number to its offset in the
    part's own frame (along pad 1 to pad 2, across).
    """
    rng = random.Random(seed)
    fps, rows, refs, n = [], [], {}, 0
    for pn, land, half in parts:
        for side in sides:
            for angle in angles:
                n += 1
                ref, bottom, at = "R%d" % n, side == "bottom", (7.0 * n, 3.0 + n % 5)
                pads = [{"num": num, "x": at[0] + p[0], "y": at[1] + p[1]}
                        for num, p in (("1", to_board((-half, 0), angle, bottom)), ("2", to_board((half, 0), angle, bottom)))]
                fps.append({"ref": ref, "fpid": land, "bottom": bottom, "x": at[0], "y": at[1],
                            "angle": angle, "pads": pads})
                refs.setdefault(pn, []).append(ref)
                if ref in skip:
                    rows.append({"ref": ref, "drawn": False})
                    continue
                turned = to_board(model.get(pn, (0, 0)), angle, bottom)
                offset = [round(export[side][k] + turned[k] + rng.gauss(0, noise), 3) if noise
                          else round(export[side][k] + turned[k], 3) for k in (0, 1)]
                rows.append({"ref": ref, "body": {"offset_xy_mm": offset, "offset_mm": round(math.hypot(*offset), 3),
                                                  "fill": 0.6}})
    # A part the preview marks with a dot, not a chip body: it must stay out.
    fps.append({"ref": "U1", "fpid": "soic-SOIC8", "bottom": False, "x": 0, "y": 0, "angle": 0,
                "pads": [{"num": str(k), "x": k, "y": 0} for k in range(1, 9)]})
    rows.append({"ref": "U1", "delta_ccw_from_top": 0.0})
    bom = "Comment,Designator,Footprint,LCSC Part #\n" + "".join(
        '"x","%s","%s","%s"\n' % (",".join(r), "F", pn) for pn, r in refs.items()) + '"u","U1","S","C9"\n'
    return {"bbox": [0, 0, 200, 20], "fps": fps}, rows, bom


class CalibrationFitTests(unittest.TestCase):
    TOL = 0.0015  # readings are rounded to the micrometre

    def near(self, got, want, tol=None):
        for g, w in zip(got, want):
            self.assertAlmostEqual(g, w, delta=tol or self.TOL)

    def test_a_pure_export_shift_fits_as_a_shift_and_not_as_a_model_offset(self):
        export = {"top": (0.005, -0.077), "bottom": (-0.03, 0.04)}
        doc = cf.report(*synthetic(export, {}))
        self.assertEqual(len(doc["parts"]), 6)
        self.assertEqual(doc["not_fitted"], [])
        for part in doc["parts"]:
            with self.subTest(part=part["pn"], view=part["view"]):
                self.assertEqual((part["n"], part["orientations"]), (5, sorted(part["orientations"])))
                self.assertEqual(len(part["orientations"]), 5)
                self.near(part["export_only"]["export_shift_xy_mm"], export[part["view"]])
                self.assertLess(part["export_only"]["rms_mm"], self.TOL)
                self.near(part["both"]["export_shift_xy_mm"], export[part["view"]])
                self.near(part["both"]["model_offset_along_across_mm"], (0, 0))
                # Turning the shift with the part leaves most of it unexplained.
                self.assertGreater(part["model_only"]["rms_mm"], 0.8 * math.hypot(*export[part["view"]]))
        for pooled in doc["lands"] + doc["views"]:
            self.near(pooled["both"]["export_shift_xy_mm"], export[pooled["view"]])
            self.assertLess(pooled["export_only"]["rms_mm"], self.TOL)

    def test_a_pure_model_offset_fits_as_an_offset_that_turns_with_the_part(self):
        model = {"C1": (0.05, -0.02), "C2": (0.05, -0.02), "C3": (0.18, 0.03)}
        doc = cf.report(*synthetic({"top": (0, 0), "bottom": (0, 0)}, model))
        for part in doc["parts"]:
            with self.subTest(part=part["pn"], view=part["view"]):
                self.near(part["model_only"]["model_offset_along_across_mm"], model[part["pn"]])
                self.assertLess(part["model_only"]["rms_mm"], self.TOL)
                self.near(part["both"]["export_shift_xy_mm"], (0, 0))
                self.near(part["both"]["model_offset_along_across_mm"], model[part["pn"]])
                self.assertAlmostEqual(part["both"]["model_offset_mm"], math.hypot(*model[part["pn"]]), delta=self.TOL)
                self.assertGreater(part["export_only"]["rms_mm"], 0.8 * math.hypot(*model[part["pn"]]))
        # C1 and C2 share their land's offset, so the land fits with one offset.
        for land in doc["lands"]:
            self.assertLess(land["model_only"]["rms_mm"], self.TOL)
        for view in doc["views"]:
            self.assertEqual(sorted(view["both"]["model_offset_along_across_mm"]), ["C1", "C2", "C3"])
            self.near(view["both"]["model_offset_along_across_mm"]["C3"], model["C3"])

    def test_a_mixture_gives_both_components_and_neither_alone_fits(self):
        export = {"top": (0.005, -0.077), "bottom": (0.02, 0.06)}
        model = {"C1": (0.04, 0.0), "C2": (-0.03, 0.05), "C3": (0.18, 0.03)}
        doc = cf.report(*synthetic(export, model))
        for part in doc["parts"]:
            with self.subTest(part=part["pn"], view=part["view"]):
                self.near(part["both"]["export_shift_xy_mm"], export[part["view"]])
                self.near(part["both"]["model_offset_along_across_mm"], model[part["pn"]])
                self.assertLess(part["both"]["rms_mm"], self.TOL)
                self.assertGreater(part["export_only"]["rms_mm"], 0.03)
                self.assertGreater(part["model_only"]["rms_mm"], 0.03)
        for view in doc["views"]:
            with self.subTest(view=view["view"]):
                self.near(view["both"]["export_shift_xy_mm"], export[view["view"]])
                for pn, offset in model.items():
                    self.near(view["both"]["model_offset_along_across_mm"][pn], offset)
                self.assertLess(view["both"]["rms_mm"], self.TOL)
        # C1 and C2 are on one land with different offsets: one shared offset does not fit them.
        shared = next(l for l in doc["lands"] if l["view"] == "top" and l["land"] == "passive-CHIP_0402")
        self.assertEqual(shared["pns"], ["C1", "C2"])
        self.near(shared["both"]["export_shift_xy_mm"], export["top"])
        self.near(shared["both"]["model_offset_along_across_mm"], (0.005, 0.025))
        self.assertGreater(shared["both"]["rms_mm"], 0.03)

    def test_noisy_readings_recover_both_components_within_the_noise(self):
        export = {"top": (0.005, -0.077), "bottom": (0.02, 0.06)}
        model = {"C1": (0.04, 0.0), "C2": (-0.03, 0.05), "C3": (0.18, 0.03)}
        doc = cf.report(*synthetic(export, model, noise=0.015, seed=7))
        for part in doc["parts"]:
            with self.subTest(part=part["pn"], view=part["view"]):
                # Five readings: each component is good to about the noise over the square root of five.
                self.near(part["both"]["export_shift_xy_mm"], export[part["view"]], tol=0.03)
                self.near(part["both"]["model_offset_along_across_mm"], model[part["pn"]], tol=0.03)
                self.assertLess(part["both"]["rms_mm"], part["export_only"]["rms_mm"])
                self.assertLess(part["both"]["rms_mm"], part["model_only"]["rms_mm"])
        for view in doc["views"]:
            with self.subTest(view=view["view"]):
                # Fifteen readings share the view's shift.
                self.near(view["both"]["export_shift_xy_mm"], export[view["view"]], tol=0.015)
                self.assertGreater(view["both"]["sigma_mm"], 0.008)
                self.assertLess(view["both"]["sigma_mm"], 0.025)
        # Noise alone, with nothing to find: both components stay within it.
        quiet = cf.report(*synthetic({"top": (0, 0), "bottom": (0, 0)}, {}, noise=0.015, seed=3))
        for view in quiet["views"]:
            self.assertLess(view["both"]["export_shift_mm"], 0.015)
            self.assertLess(max(view["both"]["model_offset_mm"].values()), 0.03)

    def test_a_bottom_part_frame_is_the_top_one_mirrored(self):
        board, _, _ = synthetic({"top": (0, 0), "bottom": (0, 0)}, {}, parts=PARTS[:1])
        for fp in board["fps"][:-1]:
            along, across = cf.part_frame(fp)
            with self.subTest(ref=fp["ref"]):
                self.near(along, to_board((1, 0), fp["angle"], fp["bottom"]), tol=1e-9)
                self.near(across, to_board((0, 1), fp["angle"], fp["bottom"]), tol=1e-9)

    def test_one_orientation_cannot_separate_and_two_few_are_not_fitted(self):
        export, model = {"top": (0.06, -0.05), "bottom": (0, 0)}, {"C1": (0.04, 0.0)}
        board, rows, bom = synthetic(export, model, parts=PARTS[:1], angles=(0,), sides=("top",))
        read, unread = cf.readings(board, rows, bom)
        one = cf.fit(read, lambda i: i["pn"])
        self.assertEqual(one["separation"], 0.0)
        self.assertNotIn("both", one)
        self.assertLess(one["export_only"]["rms_mm"], self.TOL)
        self.assertLess(one["model_only"]["rms_mm"], self.TOL)
        # Board 4's A and E rows: two orientations a view. Not fitted by default.
        two = cf.report(*synthetic(export, model, parts=PARTS[:1], angles=(0, 37), sides=("top",)))
        self.assertEqual(two["parts"], [])
        self.assertEqual([(r["ref"], r["reason"]) for r in two["not_fitted"]],
                         [("R1", "2 of its orientations read in this view"), ("R2", "2 of its orientations read in this view")])
        fitted = cf.report(*synthetic(export, model, parts=PARTS[:1], angles=(0, 37), sides=("top",)), min_angles=2)
        self.assertNotIn("sigma_mm", fitted["parts"][0]["both"])  # four numbers, four parameters
        self.assertLess(fitted["parts"][0]["separation"], 0.2)
        # Two orientations 37 degrees apart barely separate the two, but exact readings still do.
        self.near(fitted["parts"][0]["both"]["export_shift_xy_mm"], export["top"], tol=0.005)
        self.near(fitted["parts"][0]["both"]["model_offset_along_across_mm"], model["C1"], tol=0.005)

    def test_a_body_that_was_not_drawn_is_listed_and_not_read_as_zero(self):
        export = {"top": (0.005, -0.077), "bottom": (0, 0)}
        # R1-R3 are three of C1's five top copies; R11 is one of C2's.
        doc = cf.report(*synthetic(export, {"C1": (0.04, 0.0)}, skip=("R1", "R2", "R3", "R11")))
        gone = {(r["ref"], r["reason"]) for r in doc["not_fitted"]}
        self.assertEqual(gone, {("R1", "not drawn"), ("R2", "not drawn"), ("R3", "not drawn"), ("R11", "not drawn"),
                                ("R4", "2 of its orientations read in this view"),
                                ("R5", "2 of its orientations read in this view")})
        self.assertNotIn(("top", "C1"), {(p["view"], p["pn"]) for p in doc["parts"]})
        c2 = next(p for p in doc["parts"] if (p["view"], p["pn"]) == ("top", "C2"))
        self.assertEqual(c2["n"], 4)
        self.near(c2["both"]["export_shift_xy_mm"], export["top"])
        self.assertNotIn("U1", json.dumps(doc))

    def test_command_writes_the_fits_with_its_inputs_hashes(self):
        board, rows, bom = synthetic({"top": (0.005, -0.077), "bottom": (0, 0)}, {"C3": (0.18, 0.03)})
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            (td / "board.json").write_text(json.dumps(board))
            (td / "m.json").write_text(json.dumps(rows))
            (td / "bom.csv").write_text(bom)
            argv = [str(td / name) for name in ("board.json", "m.json", "bom.csv", "out.json")]
            with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(cf.main(argv), 0)
                with self.assertRaises(SystemExit):
                    cf.main(argv + ["--min-angles", "1"])
            doc = json.loads((td / "out.json").read_text())
        self.assertEqual(sorted(doc["inputs_sha256"]), ["board.json", "bom.csv", "m.json"])
        self.assertEqual((len(doc["parts"]), len(doc["lands"]), len(doc["views"])), (6, 4, 2))


if __name__ == "__main__":
    unittest.main()
