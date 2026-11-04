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
# The BOM step's text as the assembler showed it on 2026-11-04, cut to two rows.
BOM_STEP = ("Both Sides\nTotal\n2\nparts detected\n2\nParts confirmed\nComplete File\n"
            "Top Designator\tBottom Designator\tComment\tFootprint\tMatched Part Detail\tQty\n"
            "U1,U2\t\nU3\t\nSTM32F427VGT6\t\nKICAD_LQFP_100_14X14MM_P0_5MM\t\nSTM32F427VGT6\nC117815\n"
            "\t\n20\t\nJLCPCB\t\nExtended\t\n"
            "U4\t\n\t\nSTM32G431RBI6\t\nKICAD_UFBGA_64\t\nSTM32G431RBI6\nC730103\n\t\n5\t\n")
SIDES = {"Top": {"U1", "U2", "U4"}, "Bottom": {"U3"}}


def package(path, members, tampered=None):
    sums = "".join("%s  %s\n" % (hashlib.sha256(data).hexdigest(), name) for name, data in sorted(members.items()))
    with zipfile.ZipFile(path, "w") as archive:
        for name, data in dict(members, **(tampered or {})).items():
            archive.writestr(name, data)
        archive.writestr("SHA256SUMS", sums)


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
        members = {"x-gerbers.zip": b"g", "x-lcsc-cpl.csv": CPL.encode(),
                   "verification.json": json.dumps({"part_numbers": ["C1"]}).encode()}
        with tempfile.TemporaryDirectory() as td:
            good = Path(td) / "good.zip"
            package(good, members)
            upload, verification = jp.unpack(good, Path(td) / "run")
            self.assertEqual(verification["part_numbers"], ["C1"])
            self.assertEqual(jp.expected_sides(upload), {"Top": {"U1", "U2"}, "Bottom": {"U3"}})

            bad = Path(td) / "bad.zip"
            package(bad, members, tampered={"x-gerbers.zip": b"tampered"})
            with self.assertRaisesRegex(jp.Unverified, "does not match the package's SHA256SUMS"):
                jp.unpack(bad, Path(td) / "run2")

    def test_the_bom_step_must_show_every_part_number_and_placement_without_stock_warnings(self):
        self.assertEqual(jp.check_bom_step(BOM_STEP, ["C117815", "C730103"], SIDES), [])
        cases = [
            (BOM_STEP.replace("C730103", "C999"), "does not list C730103"),
            (BOM_STEP.replace("2\nParts confirmed", "1\nParts confirmed"), "expected 2 parts detected and confirmed"),
            (BOM_STEP + "Insufficient stock for C730103\n", "stock warning: Insufficient"),
            (BOM_STEP.replace("U3\t", "\t"), "Bottom designators missing from the BOM step: U3"),
        ]
        for text, message in cases:
            with self.subTest(message=message):
                self.assertTrue(any(message in p for p in jp.check_bom_step(text, ["C117815", "C730103"], SIDES)),
                                jp.check_bom_step(text, ["C117815", "C730103"], SIDES))

    def test_no_cart_tab_or_no_permission_hands_over_to_a_person(self):
        for result, message in [(subprocess.CompletedProcess([], 0, "__no_tab__\n", ""), "no https://cart.jlcpcb.com/ tab"),
                                (subprocess.CompletedProcess([], 1, "", "not authorized"), "Automation")]:
            with self.subTest(message=message), mock.patch.object(jp.subprocess, "run", return_value=result):
                with self.assertRaisesRegex(jp.Stop, message):
                    jp.js("1")

    def test_a_quoted_result_is_unescaped(self):
        result = subprocess.CompletedProcess([], 0, '"/quote {\\"a\\":\\"\\u003Cb\\"}"\n', "")
        with mock.patch.object(jp.subprocess, "run", return_value=result):
            self.assertEqual(jp.js("1"), '/quote {"a":"<b"}')

    def test_an_export_without_a_mark_for_each_part_is_taken_again(self):
        with tempfile.TemporaryDirectory() as td:
            downloads, out = Path(td) / "dl", Path(td) / "top.png"
            downloads.mkdir()

            def js(code):
                if "_download-icon_" in code:
                    (downloads / "Capture from JLCPCB.png").write_bytes(b"png")
                return "true"
            for counts, expect in (([3, 16], "ok"), ([3, 3, 3], "never showed")):
                with self.subTest(counts=counts), mock.patch.object(jp, "js", side_effect=js), \
                        mock.patch.object(jp, "pin1_marks", side_effect=counts) as marks, \
                        mock.patch.object(sys, "stdout", io.StringIO()):
                    if expect == "ok":
                        self.assertEqual(jp.export_capture("Top", downloads, out, 12, settle=0), out)
                    else:
                        with self.assertRaisesRegex(jp.Unverified, expect):
                            jp.export_capture("Top", downloads, out, 12, settle=0)
                    self.assertEqual(marks.call_count, len(counts))

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
