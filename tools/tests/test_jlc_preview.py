import hashlib
import io
import json
import math
import struct
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "frame_boards"))
import jlc_preview as jp
import measure_preview as mp

CPL = "Designator,Mid X,Mid Y,Layer,Rotation\nU1,1mm,1mm,Top,270\nU2,2mm,1mm,Top,307\nU3,3mm,1mm,Bottom,90\n"
BOM = 'Comment,Designator,Footprint,LCSC Part #\n"A","U1,U3","F","C117815"\n"B","U2","G","C730103"\n'
PLACEMENTS = {"U1": ("C117815", "Top"), "U2": ("C730103", "Top"), "U3": ("C117815", "Bottom")}
# The BOM step as the assembler showed it on 2026-11-04, cut to two rows.
BOM_TEXT = "Both Sides\nTotal\n2\nparts detected\n2\nParts confirmed\nComplete File\n"
HEADS = ["Top Designator", "Bottom Designator", "Comment", "Footprint", "Matched Part Detail", "Qty", "Source",
         "Lib Type", "Total Cost", "Select"]


def row(top, bottom, pn, selected=True):
    return [top, bottom, "STM32", "KICAD_LQFP", "STM32\n%s\n-40℃~+85℃ 12bit C99 ARM" % pn, "20", "JLCPCB", "Extended",
            "$1.0", "__checked__" if selected else "__unchecked__"]


TABLE = {"heads": HEADS, "rows": [row("U1", "U3", "C117815"), row("U2", "", "C730103")]}


def package(path, members, tampered=None):
    sums = "".join("%s  %s\n" % (hashlib.sha256(data).hexdigest(), name) for name, data in sorted(members.items()))
    with zipfile.ZipFile(path, "w") as archive:
        for name, data in dict(members, **(tampered or {})).items():
            archive.writestr(name, data)
        archive.writestr("SHA256SUMS", sums)


def fp(ref, x, y, half, bottom=False):
    """A footprint whose farthest pad sits HALF mm out on the diagonal."""
    return {"ref": ref, "bottom": bottom, "x": x, "y": y,
            "pads": [{"num": "1", "x": x - half, "y": y - half}, {"num": "2", "x": x + half, "y": y + half}]}


def chip(ref, x, y, angle=0.0, half=0.8, bottom=False, fpid="passive-CHIP_0603"):
    """A two-pad chip footprint, its pads HALF mm either side along ANGLE (degrees, board frame)."""
    c, s = math.cos(math.radians(angle)), math.sin(math.radians(angle))
    return {"ref": ref, "fpid": fpid, "bottom": bottom, "x": x, "y": y, "angle": angle,
            "pads": [{"num": "1", "x": x - half * c, "y": y - half * s}, {"num": "2", "x": x + half * c, "y": y + half * s}]}


GREEN, COPPER, GREY = (32, 110, 20), (128, 128, 192), (70, 70, 70)  # BGR


def export(path, parts, bottom=False, size=20.0, scale=10):
    """A synthetic preview export of a SIZE mm square board at SCALE px/mm, as a 24-bit BMP.

    PARTS: (footprint, body) where body is None (not drawn: bare pads and the
    checkered centroid marker) or (length, width, angle, dx, dy) of a grey body.
    """
    n = int(size * scale)
    stride = (n * 3 + 3) & ~3
    rows = []
    for py in range(n):
        row = bytearray()
        for px in range(n):
            bx = ((n - (px + 0.5)) if bottom else (px + 0.5)) / scale
            by = (py + 0.5) / scale
            colour = GREEN
            for f, b in parts:
                for p in f["pads"]:
                    if abs(bx - p["x"]) <= 0.25 and abs(by - p["y"]) <= 0.25:
                        colour = COPPER
                if b is None:
                    if abs(bx - f["x"]) <= 0.3 and abs(by - f["y"]) <= 0.3:
                        colour = (0, 0, 0) if (bx > f["x"]) == (by > f["y"]) else (255, 255, 255)
                    continue
                length, width, angle, dx, dy = b
                c, s = math.cos(math.radians(angle)), math.sin(math.radians(angle))
                u, v = bx - f["x"] - dx, by - f["y"] - dy
                if abs(u * c + v * s) <= length / 2 and abs(-u * s + v * c) <= width / 2:
                    colour = GREY
            row += bytes(colour)
        rows.append(bytes(row) + b"\0" * (stride - n * 3))
    pixels = b"".join(reversed(rows))
    header = struct.pack("<2sIHHI", b"BM", 54 + len(pixels), 0, 0, 54)
    info = struct.pack("<IiiHHIIiiII", 40, n, n, 1, 24, 0, len(pixels), 2835, 2835, 0, 0)
    Path(path).write_bytes(header + info + pixels)


class JlcPreviewTests(unittest.TestCase):
    def test_cart_quote_order_and_pay_buttons_are_never_clicked(self):
        with mock.patch.object(jp, "js", side_effect=AssertionError("js must not run")):
            for label in ("Save to Cart", "SAVE TO CART", "Place Order", "Pay Now", "Checkout", "Submit Order"):
                with self.subTest(label=label), self.assertRaisesRegex(AssertionError, "refusing to click"):
                    jp.click_button(label)
        with mock.patch.object(jp, "js", return_value="clicked") as run:
            jp.click_button("NEXT")
            jp.click_button("Process BOM & CPL")
        self.assertEqual(run.call_count, 2)

    def test_the_package_is_checked_against_its_sums_before_anything_is_uploaded(self):
        members = {"x-gerbers.zip": b"g", "x-lcsc-cpl.csv": CPL.encode(), "x-lcsc-bom.csv": BOM.encode(),
                   "verification.json": json.dumps({"part_numbers": ["C1"]}).encode()}
        with tempfile.TemporaryDirectory() as td:
            good = Path(td) / "good.zip"
            package(good, members)
            upload, verification = jp.unpack(good, Path(td) / "run")
            self.assertEqual(verification["part_numbers"], ["C1"])
            self.assertEqual(jp.expected_placements(upload), PLACEMENTS)

            bad = Path(td) / "bad.zip"
            package(bad, members, tampered={"x-gerbers.zip": b"tampered"})
            with self.assertRaisesRegex(jp.Unverified, "does not match the package's SHA256SUMS"):
                jp.unpack(bad, Path(td) / "run2")

    def test_the_bom_step_is_checked_row_by_row(self):
        self.assertEqual(jp.check_bom_step(BOM_TEXT, TABLE, PLACEMENTS), [])
        swapped_parts = {"heads": HEADS, "rows": [row("U1", "U3", "C730103"), row("U2", "", "C117815")]}
        swapped_sides = {"heads": HEADS, "rows": [row("U3", "U1", "C117815"), row("U2", "", "C730103")]}
        duplicate = {"heads": HEADS, "rows": [row("U1", "U3", "C117815"), row("U2,U1", "", "C730103")]}
        cases = [
            (BOM_TEXT, swapped_parts, "U1: the BOM step shows C730103 on the Top, the package C117815 on the Top"),
            (BOM_TEXT, swapped_sides, "U3: the BOM step shows C117815 on the Top, the package C117815 on the Bottom"),
            (BOM_TEXT, duplicate, "U1 appears 2 times in the BOM step (rows 1, 2)"),
            (BOM_TEXT, {"heads": HEADS, "rows": [row("U1", "U3", "C117815"), row("U2", "", "C730103", False)]},
             "BOM row 2 (C730103) is not selected"),
            (BOM_TEXT, {"heads": HEADS, "rows": [row("U1", "", "C117815"), row("U2", "", "C730103")]},
             "U3 is missing from the BOM step"),
            (BOM_TEXT, {"heads": HEADS, "rows": [row("U1", "U3,U9", "C117815"), row("U2", "", "C730103")]},
             "U9 is in the BOM step but not in the package"),
            (BOM_TEXT, {"heads": HEADS, "rows": [row("U1", "U3", "C117815")]}, "expected 2 BOM rows, read 1"),
            (BOM_TEXT, {"heads": HEADS[:4], "rows": []}, "the BOM table's columns are"),
            (BOM_TEXT, {"error": "the BOM step shows 0 part tables"}, "shows 0 part tables"),
            (BOM_TEXT.replace("2\nParts confirmed", "1\nParts confirmed"), TABLE, "expected 2 parts detected and confirmed"),
            (BOM_TEXT + "Insufficient stock for C730103\n", TABLE, "stock warning: Insufficient"),
        ]
        for text, table, message in cases:
            with self.subTest(message=message):
                problems = jp.check_bom_step(text, table, PLACEMENTS)
                self.assertTrue(any(message in p for p in problems), problems)

    def test_a_top_only_board_has_no_bottom_designator_column(self):
        # The 2026-11-07 W3 pilot: a top-only board's BOM step shows no Bottom Designator.
        top = {ref: placed for ref, placed in PLACEMENTS.items() if placed[1] == "Top"}
        heads = [h for h in HEADS if h != "Bottom Designator"]
        table = {"heads": heads, "rows": [[c for i, c in enumerate(row("U1", "", "C117815")) if i != 1],
                                          [c for i, c in enumerate(row("U2", "", "C730103")) if i != 1]]}
        self.assertEqual(jp.check_bom_step(BOM_TEXT, table, top), [])
        unmatched = {"heads": heads, "rows": [table["rows"][0], table["rows"][1][:3] + [""] + table["rows"][1][4:]]}
        self.assertIn("BOM row 2 matches no part number", jp.check_bom_step(BOM_TEXT, unmatched, top))
        # A board with bottom parts still needs the column.
        self.assertTrue(any("columns are" in p for p in jp.check_bom_step(BOM_TEXT, table, PLACEMENTS)))

    def test_the_run_works_in_its_own_tab_and_stops_when_it_is_gone(self):
        with mock.patch.object(jp, "TAB", None), self.assertRaisesRegex(AssertionError, "open_tab"):
            jp.js("1")
        with mock.patch.object(jp.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, "W-1\nT-2\n", "")):
            with mock.patch.object(jp, "TAB", None):
                jp.open_tab()
                self.assertEqual(jp.TAB, ("W-1", "T-2"))
        with mock.patch.object(jp, "TAB", ("W-1", "T-2")):
            ok = subprocess.CompletedProcess([], 0, '"x"\n', "")
            with mock.patch.object(jp.subprocess, "run", return_value=ok) as run:
                jp.js("1")
            self.assertEqual(run.call_args.args[0][3:], ["W-1", "T-2", "1"])
            for result, message in [(subprocess.CompletedProcess([], 0, "__gone__\n", ""), "closed or moved"),
                                    (subprocess.CompletedProcess([], 0, "__left__\n", ""), "left https://cart.jlcpcb.com/"),
                                    (subprocess.CompletedProcess([], 1, "", "not authorized"), "Automation")]:
                with self.subTest(message=message), mock.patch.object(jp.subprocess, "run", return_value=result):
                    with self.assertRaisesRegex(jp.Stop, message):
                        jp.js("1")
        # The tab is found by its window and tab ids, never as the first cart tab.
        self.assertIn("if id of t is item 2 of argv", jp.APPLESCRIPT)
        self.assertNotIn("front window", jp.APPLESCRIPT)

    def test_a_quoted_result_is_unescaped(self):
        result = subprocess.CompletedProcess([], 0, '"/quote {\\"a\\":\\"\\u003Cb\\"}"\n', "")
        with mock.patch.object(jp, "TAB", ("W", "T")), mock.patch.object(jp.subprocess, "run", return_value=result):
            self.assertEqual(jp.js("1"), '/quote {"a":"<b"}')

    def test_every_part_needs_a_mark_of_its_own(self):
        qfps = [fp("U%d" % i, 20.0 * i, 0, 8) for i in range(1, 5)]
        bgas = [fp("U5", 100, 0, 3), fp("U6", 120, 0, 3)]
        two = lambda f: [(f["x"] - 3, f["y"] - 3, 9), (f["x"] - 3.6, f["y"] - 3.6, 9)]
        one = lambda f: [(f["x"] - 8, f["y"] - 8, 9)]
        everything = [m for f in qfps for m in one(f)] + [m for f in bgas for m in two(f)]
        self.assertEqual(jp.unmarked(everything, qfps + bgas), [])
        # Two QFPs not drawn: the BGAs' second marks bring the count back to six, but not onto U1 or U3.
        partial = one(qfps[1]) + one(qfps[3]) + [m for f in bgas for m in two(f)]
        self.assertEqual(len(partial), 6)
        self.assertEqual(jp.unmarked(partial, qfps + bgas), ["U1", "U3"])
        # A mark beyond every part's reach counts for none.
        self.assertEqual(jp.unmarked([(500, 500, 9)], qfps[:1]), ["U1"])

    def test_the_board_must_hold_the_cpl_rows_on_their_sides(self):
        with tempfile.TemporaryDirectory() as td:
            board = Path(td) / "board.json"
            for fps, ok in (([fp("U1", 0, 0, 1), fp("U2", 5, 0, 1), fp("U3", 9, 0, 1, True), fp("H1", 20, 0, 1)], True),
                            ([fp("U1", 0, 0, 1), fp("U2", 5, 0, 1), fp("U3", 9, 0, 1)], False),
                            ([fp("U1", 0, 0, 1), fp("U3", 9, 0, 1, True)], False)):
                board.write_text(json.dumps({"bbox": [0, 0, 30, 30], "fps": fps}))
                with self.subTest(refs=[f["ref"] for f in fps]):
                    if ok:
                        self.assertEqual(len(jp.load_board(Path(td), None, PLACEMENTS)["fps"]), 4)
                    else:
                        with self.assertRaisesRegex(jp.Unverified, "CPL's designators on the CPL's sides"):
                            jp.load_board(Path(td), board, PLACEMENTS)
            with self.assertRaisesRegex(jp.Unverified, "pass --board"):
                jp.load_board(Path(td) / "none", None, PLACEMENTS)

    def test_an_export_without_evidence_on_each_part_is_taken_again(self):
        fps = [fp("U1", 10, 10, 4), fp("U2", 40, 10, 4)]
        both = [(6, 6, 9), (36, 6, 9)]
        drawn = {"U1": {"drawn": True, "body": None}, "U2": {"drawn": True, "body": None}}
        with tempfile.TemporaryDirectory() as td:
            downloads, out = Path(td) / "dl", Path(td) / "top.png"
            downloads.mkdir()

            def js(code):
                if "_download-icon_" in code:
                    (downloads / "Capture from JLCPCB.png").write_bytes(b"png")
                return "true"
            # (exports as inspected, verdict, exports taken)
            for shown, expect, taken in (
                    ([([], drawn), (both, drawn)], {"U1": "mark", "U2": "mark"}, 2),
                    # Two exports alike: a third would not differ.
                    ([(both[:1], drawn)] * 3, {"U1": "mark", "U2": "manual review"}, 2),
                    ([([], drawn), (both[:1], drawn), (both[:1], drawn)], {"U1": "mark", "U2": "manual review"}, 3)):
                with self.subTest(expect=expect, taken=taken), mock.patch.object(jp, "js", side_effect=js), \
                        mock.patch.object(jp, "inspect", side_effect=shown) as found, \
                        mock.patch.object(sys, "stdout", io.StringIO()):
                    path, verdict = jp.export_capture("Top", downloads, out, fps, (0, 0, 50, 50), settle=0)
                    self.assertEqual(path, out)
                    self.assertEqual({ref: v[0] for ref, v in verdict.items()}, expect)
                    self.assertEqual(found.call_count, taken)

    def test_a_silent_export_is_clicked_once_more(self):
        fps = [fp("U1", 10, 10, 4)]
        shown = ([(6, 6, 9)], {"U1": {"drawn": True, "body": None}})
        with tempfile.TemporaryDirectory() as td:
            downloads, out = Path(td) / "dl", Path(td) / "bottom.png"
            downloads.mkdir()
            clicks = []

            def js(code):
                if "_download-icon_" in code:
                    clicks.append(code)
                    if len(clicks) == 2:  # the first click is lost
                        (downloads / "JLCPCB PCBA Capture.png").write_bytes(b"png")
                return "true"
            with mock.patch.object(jp, "js", side_effect=js), mock.patch.object(jp, "inspect", return_value=shown), \
                    mock.patch.object(sys, "stdout", io.StringIO()):
                path, verdict = jp.export_capture("Bottom", downloads, out, fps, (0, 0, 50, 50), settle=0, reclick=0)
            self.assertEqual((path, verdict["U1"][0], len(clicks)), (out, "mark", 2))
            self.assertEqual(list(downloads.iterdir()), [])

    def test_a_chip_part_is_the_libraries_two_pad_passive_chip_land(self):
        self.assertTrue(mp.is_chip(chip("R1", 0, 0)))
        self.assertTrue(mp.is_chip(chip("C1", 0, 0, fpid="passive-CHIP_2512")))
        # A polarised two-pad land, a land without its library id, and a chip land with a third pad are not.
        self.assertFalse(mp.is_chip(chip("D1", 0, 0, fpid="contrib_esd-FP_SOD323_2P5X1P25mm")))
        self.assertFalse(mp.is_chip({k: v for k, v in chip("R1", 0, 0).items() if k != "fpid"}))
        three = chip("R1", 0, 0)
        three["pads"].append({"num": "3", "x": 0, "y": 0.5})
        self.assertFalse(mp.is_chip(three))

    def test_a_chip_body_must_lie_along_its_pads_and_be_centred(self):
        cases = [
            # (footprint, body, bottom, drawn, passes)
            (chip("R1", 6, 6), (1.6, 0.8, 0, 0, 0), False, True, True),
            (chip("R1", 6, 6, angle=37), (1.6, 0.8, 37 + 180, 0, 0), False, True, True),
            (chip("R1", 6, 6, angle=37, bottom=True), (1.6, 0.8, 37, 0, 0), True, True, True),
            (chip("R1", 6, 6), (1.6, 0.8, 90, 0, 0), False, True, False),  # a quarter turn
            (chip("R1", 6, 6), (1.6, 0.8, 0, 0.5, 0), False, True, False),  # off its pads
            (chip("R1", 6, 6), None, False, False, False),  # not drawn
        ]
        with tempfile.TemporaryDirectory() as td:
            for f, b, bottom, shown, passes in cases:
                with self.subTest(body=b, bottom=bottom):
                    export(Path(td) / "x.bmp", [(f, b)], bottom)
                    view = mp.View(str(Path(td) / "x.bmp"), (0, 0, 20, 20), bottom)
                    self.assertEqual(mp.drawn(view, f), shown)
                    if shown:
                        reading = mp.body(view, f)
                        self.assertEqual(mp.body_ok(reading), passes, reading)
                        if passes:
                            self.assertLess(abs(reading["delta_mod_180"]), 2)
                        elif b[2] == 90:
                            self.assertGreater(abs(reading["delta_mod_180"]), 80)

    def test_each_instance_stands_on_its_own_evidence(self):
        ok = {"delta_mod_180": 0.5, "offset_mm": 0.02, "offset_limit_mm": 0.2, "elongation": 1.8, "fill": 1.0}
        turned = dict(ok, delta_mod_180=88.0)
        qfp, sw, bga = fp("U1", 20, 20, 4), fp("SW1", 40, 20, 3), fp("U2", 60, 20, 3)
        r1, r2, r3 = chip("R1", 10, 40), chip("R2", 20, 40), chip("R3", 30, 40)
        fps = [qfp, sw, bga, r1, r2, r3]
        readings = {"U1": {"drawn": True, "body": None}, "SW1": {"drawn": True, "body": None},
                    "U2": {"drawn": False, "body": None}, "R1": {"drawn": True, "body": ok},
                    "R2": {"drawn": True, "body": turned}, "R3": {"drawn": False, "body": None}}
        # One mark on U1; a stray mark near R1 is R1's nearest, so it covers neither SW1 nor U2.
        marks = [(16, 16, 9), (10.3, 40, 9)]
        got = {ref: v[0] for ref, v in jp.verdicts(marks, fps, readings).items()}
        self.assertEqual(got, {"U1": "mark", "SW1": "manual review", "U2": "unverified",
                               "R1": "body", "R2": "manual review", "R3": "unverified"})

    def test_main_reports_a_stop_as_exit_2_and_a_failed_check_as_exit_1(self):
        err = io.StringIO()
        with mock.patch.object(jp, "run_preview", side_effect=jp.Stop("not signed in")), mock.patch.object(sys, "stderr", err):
            self.assertEqual(jp.main(["p.zip"]), 2)
        with mock.patch.object(jp, "run_preview", side_effect=jp.Unverified("stock warning")), mock.patch.object(sys, "stderr", err):
            self.assertEqual(jp.main(["p.zip"]), 1)
        # Both exports taken: every instance with its evidence is exit 0; any left open is exit 3.
        done = {"U1": {"status": "mark"}, "R1": {"status": "body"}}
        with mock.patch.object(jp, "run_preview", return_value=done), mock.patch.object(sys, "stderr", err):
            self.assertEqual(jp.main(["p.zip"]), 0)
        with mock.patch.object(jp, "run_preview", return_value=dict(done, U2={"status": "unverified"})), \
                mock.patch.object(sys, "stderr", err):
            self.assertEqual(jp.main(["p.zip"]), 3)
        self.assertIn("1 instances unverified or for manual review: U2", err.getvalue())
        self.assertIn("a person has to take over: not signed in", err.getvalue())
        self.assertIn("unverified: stock warning", err.getvalue())


if __name__ == "__main__":
    unittest.main()
