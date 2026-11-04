import hashlib
import io
import json
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "frame_boards"))
import jlc_preview as jp

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

    def test_an_export_without_a_mark_on_each_part_is_taken_again(self):
        fps = [fp("U1", 10, 10, 4), fp("U2", 40, 10, 4)]
        both = [(6, 6, 9), (36, 6, 9)]
        with tempfile.TemporaryDirectory() as td:
            downloads, out = Path(td) / "dl", Path(td) / "top.png"
            downloads.mkdir()

            def js(code):
                if "_download-icon_" in code:
                    (downloads / "Capture from JLCPCB.png").write_bytes(b"png")
                return "true"
            for marks, expect in (([[], both], "ok"), ([both[:1]] * 3, "never showed a pin 1 mark on U2")):
                with self.subTest(expect=expect), mock.patch.object(jp, "js", side_effect=js), \
                        mock.patch.object(jp, "pin1_marks", side_effect=marks) as found, \
                        mock.patch.object(sys, "stdout", io.StringIO()):
                    if expect == "ok":
                        self.assertEqual(jp.export_capture("Top", downloads, out, fps, (0, 0, 50, 50), settle=0), out)
                    else:
                        with self.assertRaisesRegex(jp.Unverified, expect):
                            jp.export_capture("Top", downloads, out, fps, (0, 0, 50, 50), settle=0)
                    self.assertEqual(found.call_count, len(marks))

    def test_main_reports_a_stop_as_exit_2_and_a_failed_check_as_exit_1(self):
        err = io.StringIO()
        with mock.patch.object(jp, "run_preview", side_effect=jp.Stop("not signed in")), mock.patch.object(sys, "stderr", err):
            self.assertEqual(jp.main(["p.zip"]), 2)
        with mock.patch.object(jp, "run_preview", side_effect=jp.Unverified("stock warning")), mock.patch.object(sys, "stderr", err):
            self.assertEqual(jp.main(["p.zip"]), 1)
        self.assertIn("a person has to take over: not signed in", err.getvalue())
        self.assertIn("unverified: stock warning", err.getvalue())


if __name__ == "__main__":
    unittest.main()
