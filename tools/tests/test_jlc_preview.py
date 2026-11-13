import hashlib
import io
import json
import math
import os
import shutil
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


def quad(ref, x, y, half=1.0, angle=0.0, bottom=False, fpid="qfn-QFN4"):
    """A four-pad part: pads 1 to 4 at the corners of a square of half-side HALF, turned ANGLE
    degrees as KiCad turns a footprint."""
    c, s = math.cos(math.radians(angle)), math.sin(math.radians(angle))
    corners = ((-half, -half), (-half, half), (half, half), (half, -half))
    return {"ref": ref, "fpid": fpid, "bottom": bottom, "x": x, "y": y, "angle": angle,
            "pads": [{"num": str(i + 1), "x": x + u * c + v * s, "y": y - u * s + v * c} for i, (u, v) in enumerate(corners)]}


def at(f, num, out=0.0):
    """A mark OUT mm beyond pad NUM of F, away from the origin."""
    pad = next(p for p in f["pads"] if p["num"] == num)
    d = math.hypot(pad["x"] - f["x"], pad["y"] - f["y"])
    return (pad["x"] + (pad["x"] - f["x"]) * out / d, pad["y"] + (pad["y"] - f["y"]) * out / d, 300, 0.2, 0.5)


FIXTURES = Path(__file__).resolve().parent / "fixtures"
PREVIEWS = Path(__file__).resolve().parents[1] / "sourcing_data/jlc_previews"
BODY_OK = {"delta_mod_180": 0.5, "offset_mm": 0.02, "offset_limit_mm": 0.2, "elongation": 1.8, "fill": 1.0}


def all_drawn(fps):
    return {f["ref"]: {"drawn": True, "body": BODY_OK if mp.is_chip(f) else None} for f in fps}


def status(marks, fps, attribution=2, readings=None):
    return {ref: v[0] for ref, v in jp.verdicts(marks, fps, readings or all_drawn(fps), attribution).items()}


def crowded_board(iteration, attribution):
    """The verdicts on the synthetic crowded board (fixtures/preview_marks/crowded-board.json)."""
    board = json.loads((FIXTURES / "preview_marks/crowded-board.json").read_text(encoding="utf-8"))
    out = {}
    for side in ("top", "bottom"):
        fps = [f for f in board["fps"] if f["bottom"] == (side == "bottom")]
        found = [tuple(m) for m in board["marks"][iteration][side]]
        if attribution == 1:  # every cluster, as dots() gives them
            found = [m[:3] for m in found]
        else:  # as marks() filters them
            found = [m for m in found if m[3] <= mp.MARK_AREA_MAX_MM2 and m[4] <= mp.MARK_SIDE_MAX_MM]
        out.update(jp.verdicts(found, fps, all_drawn(fps), attribution))
    return out


# The archived records that have checks.json: the board.json, the body method they were
# made with, and what attribution 2 changes: every change is from "mark" to "manual review".
# TURNED: the part's one mark is a quarter or half turn from pad 1 (the angle is the record's
# own measurements.json delta_ccw_from_top). SEVERAL: a two-terminal diode or a BGA with two
# or more marks, which the record read by hand.
ARCHIVED = {
    "2026-11-05-c2-board-3": ("board.json", {
        "turned": "U1 U2 U3 U4 U13 U14 U15 U16 U33 U34 U35 U36 U37 U38 U39 U40",
        "several": "U17 U18 U19 U20 U21 U22 U23 U24 U46"}),
    "2026-11-07-w3-pilot-0402": ("board.json", {}),
    "2026-11-08-probe": ("package/board.json", {
        "turned": "U13 U14 U15 U16 U17 U18 U19 U20",
        "several": "U9 U10 U11 U12 U21 U22 U23 U24 U25 U26 U27 U28 U33 U34 U35 U36 U53 U54 U55 U56"}),
    "2026-11-08-reverify": ("package/board.json", {"several": "U5 U6 U7 U8 U9 U10 U11 U12 U13 U14 U15 U16"}),
    "2026-11-09-reverify-2": ("package/board.json", {}),
    "2026-11-11-board-4": ("package/board.json", {"turned": "U1 U2 U3 U4 U5 U6 U7 U8 U9 U10 U11 U12"}),
}


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

    def test_attribution_1_counted_a_mark_for_the_nearest_origin(self):
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

    def test_a_second_mark_does_not_stand_in_for_a_missing_one(self):
        # A BGA shows two marks near A1. Its neighbour, 4 mm away, shows none: the BGA's second
        # mark lies within the neighbour's margin, but nearer the BGA's own pads.
        bga, other = quad("U1", 10, 10, 2.0), quad("U2", 15, 10, 1.0)
        two = [at(bga, "4", 0.3), at(bga, "4", 0.9)]
        self.assertLessEqual(mp.beyond(other, *two[1][:2]), mp.MARK_MARGIN_MM)
        got = jp.verdicts(two, [bga, other], all_drawn([bga, other]))
        self.assertEqual(got["U2"], ("manual review", "drawn, with no pin 1 mark of its own"))
        self.assertEqual(got["U1"], ("manual review", "drawn, with 2 marks of its own: which is pin 1 is read by hand"))
        # A part that is not drawn owns nothing, and a mark beyond every part's margin counts for none.
        hidden = dict(all_drawn([bga, other]), U1={"drawn": False, "body": None})
        self.assertEqual(status([at(bga, "1")], [bga, other], readings=hidden), {"U1": "unverified", "U2": "manual review"})
        self.assertEqual(status([(500, 500, 9, 0.2, 0.5)], [bga]), {"U1": "manual review"})

    def test_a_chip_part_or_an_inductor_never_takes_a_mark(self):
        # The mark sits at the IC's pad 1; a capacitor's and an inductor's origins are nearer to it.
        ic = quad("U1", 10, 10, 1.5)
        mark = at(ic, "1", 0.4)
        cap = chip("C1", mark[0] - 0.9, mark[1] - 0.3, half=0.5, fpid="passive-CHIP_0402")
        coil = chip("L1", mark[0] + 0.2, mark[1] - 1.2, half=0.95, fpid="passive-FP_L_0806_2016Metric")
        bead = chip("FB1", mark[0] - 1.4, mark[1] + 1.0, half=1.5, fpid="local-FP_BEAD_1206")
        fps = [ic, cap, coil, bead]
        self.assertEqual([mp.expects_mark(f) for f in fps], [True, False, False, False])
        self.assertEqual(jp.unmarked([mark[:3]], fps), ["FB1", "L1", "U1"])  # attribution 1: the capacitor's
        self.assertEqual(status([mark[:3]], fps, attribution=1),
                         {"U1": "manual review", "C1": "body", "L1": "manual review", "FB1": "manual review"})
        self.assertEqual(status([mark], fps), {"U1": "mark", "C1": "body", "L1": "manual review", "FB1": "manual review"})
        self.assertEqual({ref: len(m) for ref, m in mp.owners([mark], fps).items()}, {"U1": 1, "C1": 0, "L1": 0, "FB1": 0})
        # Alone with the mark, the inductor still does not take it.
        self.assertEqual(status([mark], [coil]), {"L1": "manual review"})
        self.assertIn("no pin 1 mark", jp.verdicts([mark], [coil], all_drawn([coil]))["L1"][1])
        # A two-terminal part that has a pin 1 (a diode) can own a mark: one at its pad 1 passes.
        diode = chip("D1", 30, 10, half=1.0, fpid="sod-SOD2613X110N")
        self.assertTrue(mp.expects_mark(diode))
        self.assertEqual(status([at(diode, "1")], [diode]), {"D1": "mark"})
        self.assertEqual(status([at(diode, "1"), at(diode, "2")], [diode]), {"D1": "manual review"})

    def test_a_mark_passes_only_at_pad_1(self):
        for angle in (0.0, 37.0, 90.0):
            for bottom in (False, True):
                ic = quad("U1", 10, 10, 1.5, angle, bottom)
                with self.subTest(angle=angle, bottom=bottom):
                    self.assertEqual(status([at(ic, "1", 0.5)], [ic]), {"U1": "mark"})
                    for num in "234":  # a quarter or half turn off
                        got = jp.verdicts([at(ic, num, 0.5)], [ic], all_drawn([ic]))["U1"]
                        self.assertEqual(got[0], "manual review", num)
                        self.assertRegex(got[1], r"^drawn; pin 1 mark (90|180|270)\.0 deg counter-clockwise from pad 1")
        # Either side of the limit, about the origin.
        ic = quad("U1", 10, 10, 1.5)
        for turn, passes in ((mp.MARK_ANGLE_TOL - 1, True), (-(mp.MARK_ANGLE_TOL - 1), True),
                             (mp.MARK_ANGLE_TOL + 1, False), (-(mp.MARK_ANGLE_TOL + 1), False)):
            a = math.radians(135 + turn)  # pad 1 is up and to the left: 135 deg, seen from the top
            mark = (10 + 2.5 * math.cos(a), 10 - 2.5 * math.sin(a), 300, 0.2, 0.5)
            with self.subTest(turn=turn):
                self.assertEqual(status([mark], [ic]), {"U1": "mark" if passes else "manual review"})
        self.assertEqual(jp.verdicts([at(ic, "1")], [ic], all_drawn([ic]))["U1"],
                         ("mark", "pin 1 mark 0.0 deg counter-clockwise from pad 1 about the origin, seen from the top "
                                  "(limit 20 either way), 0.00 mm from pad 1"))

    def test_a_model_placed_with_an_offset_keeps_its_mark(self):
        # A header whose mark sits 4.1 mm from the origin, inside attribution 1's reach of 4.36;
        # placed with a 1.0 mm offset, the mark is 4.5 mm away and attribution 1 lost it.
        conn = {"ref": "J3", "fpid": "local-FP_HEADER_4", "bottom": False, "x": 20.0, "y": 20.0, "angle": 0.0,
                "pads": [{"num": n, "x": 20.0 + x, "y": 20.0 + y}
                         for n, x, y in (("1", -1.5, -2.0), ("2", -0.5, -2.0), ("3", 0.5, -2.0), ("4", 1.5, -2.0),
                                         ("P1", -3.0, 1.0), ("P2", 3.0, 1.0))]}
        self.assertAlmostEqual(mp.reach(conn), 4.36, places=2)
        mark, moved = (16.0, 20.9, 300, 0.2, 0.5), (16.0, 21.9, 300, 0.2, 0.5)
        self.assertEqual((jp.unmarked([mark[:3]], [conn]), jp.unmarked([moved[:3]], [conn])), ([], ["J3"]))
        self.assertEqual([len(mp.owners([m], [conn])["J3"]) for m in (mark, moved)], [1, 1])
        # It is J3's mark, but not at pad 1: the tool does not pass it either way.
        self.assertEqual(status([moved], [conn]), {"J3": "manual review"})

    def test_a_crowded_board_gives_each_mark_to_the_part_that_owns_it(self):
        # A synthetic board, with two imagined previews of it: in the first U3 is a half turn off,
        # U5 a quarter turn off and J1's model is placed with an offset; in the second all are right.
        # ref: (preview 1 by attribution 1, by 2; preview 2 by attribution 1, by 2)
        m, r = "mark", "manual review"
        table = {
            "U1": (r, m, r, m),    # its mark is nearer to C1's origin than to its own
            "U2": (m, m, m, m),    # its mark is within J1's margin too
            "U3": (m, r, m, m),    # a half turn off, then right
            "U4": (r, m, r, m),    # its mark is nearer to the inductor L1's origin
            "U5": (m, r, m, m),    # a quarter turn off, then right
            "J1": (r, m, m, m),    # placed with an offset: its mark leaves the reach about its origin
            "J2": (m, r, m, r),    # a 3.2 mm piece of the model in magenta, not a mark
            "J3": (m, r, m, r),    # two peg dots, no pin 1 mark
            "L1": (m, r, m, r),    # an inductor: U4's mark
            "L2": (r, r, r, r), "L3": (r, r, r, r),
        }
        got = [crowded_board(iteration, attribution) for iteration in "12" for attribution in (1, 2)]
        self.assertEqual({ref: tuple(g[ref][0] for g in got) for ref in table}, table)
        self.assertTrue(all(g[ref][0] == "body" for g in got for ref in g if ref not in table))
        # What attribution 2 reads.
        first, second = got[1], got[3]
        deg = lambda verdict: float(verdict[1].split("pin 1 mark ")[1].split(" deg")[0])
        self.assertEqual({ref: deg(first[ref]) for ref in ("U1", "U2", "U3", "U4", "U5", "J1")},
                         {"U1": 3.2, "U2": 357.6, "U3": 184.1, "U4": 357.0, "U5": 92.0, "J1": 4.6})
        self.assertEqual({ref: deg(second[ref]) for ref in ("U3", "U5", "J1")}, {"U3": 4.1, "U5": 2.0, "J1": 1.5})
        self.assertEqual(first["J2"][1], "drawn, with no pin 1 mark of its own")
        self.assertEqual(first["J3"][1], "drawn, with 2 marks of its own: which is pin 1 is read by hand")
        self.assertIn("a part of this kind has no pin 1 mark", first["L1"][1])
        board = json.loads((FIXTURES / "preview_marks/crowded-board.json").read_text(encoding="utf-8"))
        fps = {f["ref"]: f for f in board["fps"]}
        nearest = lambda ref, side, k="1": min(board["marks"][k][side],
                                               key=lambda c: math.hypot(c[0] - fps[ref]["x"], c[1] - fps[ref]["y"]))
        # The situations are the ones the table names: U1's and U4's marks are nearer to a neighbour's
        # origin and inside its reach; J1's mark is outside J1's reach in the first preview only.
        for ic, other in (("U1", "C1"), ("U4", "L1")):
            mark = nearest(ic, "bottom" if fps[ic]["bottom"] else "top")
            away = {ref: math.hypot(mark[0] - fps[ref]["x"], mark[1] - fps[ref]["y"]) for ref in (ic, other)}
            self.assertLess(away[other], min(away[ic], mp.reach(fps[other])))
            self.assertLess(away[ic], mp.reach(fps[ic]))
        j1 = [math.hypot(nearest("J1", "top", k)[0] - fps["J1"]["x"], nearest("J1", "top", k)[1] - fps["J1"]["y"]) for k in "12"]
        self.assertTrue(j1[0] > mp.reach(fps["J1"]) > j1[1])
        # U2's mark is also within J1's margin; it is U2's by its pad.
        mark = nearest("U2", "top")
        self.assertLess(mp.beyond(fps["J1"], *mark[:2]), mp.MARK_MARGIN_MM)
        self.assertEqual(len(mp.owners([tuple(mark)], [fps["J1"], fps["U2"]])["U2"]), 1)
        # J2's piece is outside a mark's size; every other cluster is well inside it.
        sizes = sorted((k[3], k[4]) for side in board["marks"]["1"].values() for k in side)
        self.assertEqual(sizes[-1], (2.38, 3.2))
        self.assertLess(sizes[-2][0], mp.MARK_AREA_MAX_MM2 / 3)
        self.assertLess(max(k[1] for k in sizes[:-1]), mp.MARK_SIDE_MAX_MM / 2)

    def test_a_check_names_its_attribution_and_attribution_1_writes_what_it_always_wrote(self):
        ic, cap, gone = quad("U1", 10, 10, 1.5), chip("C1", 20, 10), quad("U2", 30, 10, 1.5)
        fps = [ic, cap, gone]
        readings = dict(all_drawn(fps), U2={"drawn": False, "body": None})
        doc = {"bbox": [0, 0, 40, 20], "fps": fps + [quad("U9", 35, 15)]}
        shown = [([at(ic, "1")], readings), ([at(ic, "1")[:3]], readings)]  # marks(), then dots()
        with mock.patch.object(jp, "inspect", side_effect=shown) as inspect:
            two = jp.recheck(doc, {"Top": Path("top.png")}, {"U1", "C1", "U2"})
            one = jp.recheck(doc, {"Top": Path("top.png")}, {"U1", "C1", "U2"}, attribution=1, body_method=1)
        self.assertEqual([c.args[4:] for c in inspect.call_args_list], [(2, 2), (1, 1)])
        self.assertEqual([f["ref"] for f in inspect.call_args.args[3]], ["U1", "C1", "U2"])  # U9 was not placed
        self.assertEqual({ref: sorted(c) for ref, c in two.items()},
                         {"U1": ["attribution", "detail", "side", "status"], "C1": ["detail", "side", "status"],
                          "U2": ["detail", "side", "status"]})
        self.assertEqual((two["U1"]["attribution"], two["U1"]["status"], two["U2"]["status"]), (2, "mark", "unverified"))
        self.assertEqual(one["U1"], {"side": "top", "status": "mark", "detail": ""})
        self.assertEqual(jp.checks_text({"U10": one["U1"], "U2": one["U2"]}).splitlines()[1], ' "U2": {')

    @unittest.skipUnless(os.environ.get("COHDL_REMEASURE_PREVIEWS") and shutil.which("sips"),
                         "set COHDL_REMEASURE_PREVIEWS=1 on macOS to judge the archived exports again (about two minutes)")
    def test_each_archived_check_reproduces_under_attribution_1_and_changes_only_where_it_was_wrong(self):
        for name, (board, changes) in ARCHIVED.items():
            with self.subTest(record=name):
                record = PREVIEWS / name
                archived = (record / "checks.json").read_text(encoding="utf-8")
                doc = json.loads((record / board).read_text(encoding="utf-8"))
                exports = {side: record / (side.lower() + ".png") for side in ("Top", "Bottom")
                           if (record / (side.lower() + ".png")).is_file()}
                one = jp.recheck(doc, exports, set(json.loads(archived)), attribution=1, body_method=1)
                if name == "2026-11-07-w3-pilot-0402":
                    # Made before a body reading had offset_xy_mm: C7's detail, which quotes its reading, lacks it.
                    self.assertIn('"offset_xy_mm": [-0.008, -0.053]', one["C7"]["detail"])
                    one["C7"]["detail"] = one["C7"]["detail"].replace(', "offset_xy_mm": [-0.008, -0.053]', "")
                self.assertEqual(jp.checks_text(one), archived)
                two = jp.recheck(doc, exports, set(json.loads(archived)), body_method=1)
                changed = {ref: (one[ref]["status"], two[ref]["status"], two[ref]["detail"])
                           for ref in one if one[ref]["status"] != two[ref]["status"]}
                self.assertEqual(sorted(changed, key=jp.natural),
                                 sorted(" ".join(changes.values()).split(), key=jp.natural))
                self.assertTrue(all(v[:2] == ("mark", "manual review") for v in changed.values()))
                if board.startswith("package/") and name != "2026-11-11-board-4":
                    # measurements.json too; test_measure_preview.py re-measures the other three records.
                    with tempfile.TemporaryDirectory() as td:
                        for png in exports.values():
                            subprocess.run(["sips", "-s", "format", "bmp", str(png), "--out",
                                            str(Path(td) / (png.stem + ".bmp"))], check=True, capture_output=True)
                        mp.measure(str(record / board), str(Path(td) / "top.bmp"), str(Path(td) / "bottom.bmp"),
                                   str(Path(td) / "out.json"), method=1, attribution=1)
                        self.assertEqual((Path(td) / "out.json").read_bytes(), (record / "measurements.json").read_bytes())
                measured = {row["ref"]: row for row in json.loads((record / "measurements.json").read_text())}
                for ref in changes.get("turned", "").split():
                    delta = measured[ref]["delta_ccw_from_top"]
                    self.assertGreater(min(delta, 360 - delta), 80, ref)
                    self.assertIn("pin 1 mark %.1f deg" % delta, changed[ref][2])
                for ref in changes.get("several", "").split():
                    self.assertGreaterEqual(len(measured[ref]["dots"]), 2, ref)
                    self.assertNotIn("delta_ccw_from_top", measured[ref])
                    self.assertRegex(changed[ref][2], r"with \d marks of its own")
                # A part that still passes has its one mark at pad 1, as the record measured it.
                for ref, check in two.items():
                    if check["status"] == "mark":
                        delta = measured[ref]["delta_ccw_from_top"]
                        self.assertLessEqual(min(delta, 360 - delta), 11.4, ref)

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

    def test_a_top_only_board_is_measured_without_a_bottom_view(self):
        # The 2026-11-07 W3 pilot: a top-only board exports no bottom view.
        r1 = chip("R1", 6, 6)
        doc = {"bbox": [-0.05, -0.05, 20.05, 20.05], "fps": [r1]}
        with tempfile.TemporaryDirectory() as td:
            export(Path(td) / "top.bmp", [(r1, (1.6, 0.8, 0, 0, 0))])
            (Path(td) / "board.json").write_text(json.dumps(doc))
            mp.measure(str(Path(td) / "board.json"), str(Path(td) / "top.bmp"), "-", str(Path(td) / "out.json"))
            rows = json.loads((Path(td) / "out.json").read_text())
            self.assertEqual([(r["ref"], r["body_ok"]) for r in rows], [("R1", True)])
            doc["fps"].append(chip("R2", 12, 6, bottom=True))
            (Path(td) / "board.json").write_text(json.dumps(doc))
            with self.assertRaisesRegex(SystemExit, "needs the bottom view"):
                mp.measure(str(Path(td) / "board.json"), str(Path(td) / "top.bmp"), "-", str(Path(td) / "out.json"))

    def test_each_instance_stands_on_its_own_evidence(self):
        ok = {"delta_mod_180": 0.5, "offset_mm": 0.02, "offset_limit_mm": 0.2, "elongation": 1.8, "fill": 1.0}
        turned = dict(ok, delta_mod_180=88.0)
        qfp, sw, bga = fp("U1", 20, 20, 4), fp("SW1", 40, 20, 3), fp("U2", 60, 20, 3)
        r1, r2, r3 = chip("R1", 10, 40), chip("R2", 20, 40), chip("R3", 30, 40)
        fps = [qfp, sw, bga, r1, r2, r3]
        readings = {"U1": {"drawn": True, "body": None}, "SW1": {"drawn": True, "body": None},
                    "U2": {"drawn": False, "body": None}, "R1": {"drawn": True, "body": ok},
                    "R2": {"drawn": True, "body": turned}, "R3": {"drawn": False, "body": None}}
        # One mark on U1; a stray mark near R1 belongs to no part, so it covers neither SW1 nor U2.
        marks = [(16, 16, 9), (10.3, 40, 9)]
        for attribution in (1, 2):
            got = {ref: v[0] for ref, v in jp.verdicts(marks, fps, readings, attribution).items()}
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
