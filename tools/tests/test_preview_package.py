import contextlib
import hashlib
import io
import os
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "frame_boards"))
import preview_package as pp

ROOT = Path(__file__).resolve().parents[2]
COHDL = ROOT / "target/debug/cohdl"
KICAD = Path(os.environ.get("KICAD_APP", "/Applications/KiCad/KiCad.app")) / "Contents"
KICAD_CLI = KICAD / "MacOS/kicad-cli"
KICAD_PYTHON = KICAD / "Frameworks/Python.framework/Versions/Current/bin/python3"

MANIFEST = {"rows": [{"pn": "C1"}, {"pn": "C2"}]}
BOM = 'Comment,Designator,Footprint,LCSC Part #\n"a","U1,U2","FP","C1"\n"b","U3","FP","C2"\n'
NEUTRAL = ("Designator,Mid X,Mid Y,Layer,Rotation\n"
           "U1,0mm,0mm,Top,0\nU2,9mm,0mm,Bottom,180\nU3,0mm,9mm,Top,0\n")
NET = '(comp (ref "U1") (field (name "LCSC") "C1"))\n(comp (ref "U3") (field (name "LCSC") "C2"))\n'


class PreviewPackageTests(unittest.TestCase):
    def test_zips_are_byte_stable(self):
        members = [("b.txt", b"two"), ("a.txt", b"one")]
        first, second = io.BytesIO(), io.BytesIO()
        pp.write_zip(first, members)
        pp.write_zip(second, list(reversed(members)))
        self.assertEqual(first.getvalue(), second.getvalue())
        with zipfile.ZipFile(first) as archive:
            self.assertEqual(archive.namelist(), ["a.txt", "b.txt"])
            self.assertEqual(archive.getinfo("a.txt").date_time, (1980, 1, 1, 0, 0, 0))

    def test_only_the_fabricated_layers_and_the_drill_file_are_uploaded(self):
        with tempfile.TemporaryDirectory() as td:
            fab = Path(td)
            for layer in pp.GERBER_LAYERS + ("F_Courtyard", "User_1", "F_Fab"):
                (fab / ("b-%s.gbr" % layer)).write_text("")
            (fab / "b.drl").write_text("")
            (fab / "b-job.gbrjob").write_text("")
            names = [path.name for path in pp.fabrication_files(fab, "b")]
            self.assertEqual(len(names), 10)
            self.assertEqual(names[-1], "b.drl")
            self.assertFalse(any("Courtyard" in n or "User" in n or "Fab" in n or "job" in n for n in names))
            (fab / "b-B_Paste.gbr").unlink()
            with self.assertRaisesRegex(pp.PackageError, "expected one B_Paste Gerber"):
                pp.fabrication_files(fab, "b")

    def test_outputs_must_list_every_placement_once_with_the_manifests_part_numbers(self):
        summary = pp.audit_outputs(MANIFEST, 3, BOM, NEUTRAL, NEUTRAL, NET)
        self.assertEqual(summary, {"placements": 3, "top": 2, "bottom": 1,
                                   "part_numbers": ["C1", "C2"], "netlist_lcsc_fields": 2})
        cases = [
            ((BOM.replace("U1,U2", "U1"), NEUTRAL, NEUTRAL), "BOM does not list each placement once"),
            ((BOM.replace('"C2"', '"C9"'), NEUTRAL, NEUTRAL), "BOM names C1, C9, the manifest C1, C2"),
            ((BOM, NEUTRAL, NEUTRAL.replace("U3,0mm,9mm,Top,0\n", "")), "same placements once each"),
            ((BOM, NEUTRAL.replace("U3,0mm,9mm,Top,0\n", ""), NEUTRAL), "neutral CPL has 2 placements, the manifest 3"),
        ]
        for (bom, neutral, supplier), message in cases:
            with self.subTest(message=message), self.assertRaisesRegex(pp.PackageError, message):
                pp.audit_outputs(MANIFEST, 3, bom, neutral, supplier, NET)

    def test_a_stale_manifest_writes_no_package(self):
        with tempfile.TemporaryDirectory() as td:
            generator = Path(td) / "gen.py"
            generator.write_text("import sys; open(sys.argv[1], 'w').write('{}')\n")
            out = Path(td) / "pkg.zip"
            with mock.patch.dict(pp.REGENERATE, {"lcsc-batch-1": str(generator)}):
                with self.assertRaisesRegex(pp.PackageError, "is not what .*gen.py writes now"):
                    pp.build("lcsc-batch-1", out, "cohdl", "kicad-cli", "python3", True, Path(td))
            self.assertFalse(out.exists())
            self.assertEqual(list(Path(td).glob("*.partial")), [])

    @unittest.skipUnless(COHDL.exists() and KICAD_CLI.exists() and KICAD_PYTHON.exists(),
                         "needs target/debug/cohdl and KiCad")
    def test_lcsc_batch_1_package(self):
        with tempfile.TemporaryDirectory() as td:
            out = Path(td) / "pkg.zip"
            log = io.StringIO()
            with contextlib.redirect_stdout(log):
                code = pp.main(["lcsc-batch-1", "-o", str(out), "--cohdl", str(COHDL), "--kicad-cli", str(KICAD_CLI),
                                "--kicad-python", str(KICAD_PYTHON), "--allow-dirty"])
            self.assertEqual(code, 0, log.getvalue())
            with zipfile.ZipFile(out) as archive:
                members = {name: archive.read(name) for name in archive.namelist()}
        self.assertEqual(sorted(members), sorted([
            "CHECKLIST.md", "README.txt", "SHA256SUMS", "cohdl.lock", "design.lock", "lcsc_batch_1-gerbers.zip",
            "lcsc_batch_1-lcsc-bom.csv", "lcsc_batch_1-lcsc-cpl.csv", "manifest.json", "verification.json"]))
        sums = dict(reversed(line.split("  ")) for line in members["SHA256SUMS"].decode().splitlines())
        for name, data in members.items():
            if name != "SHA256SUMS":
                self.assertEqual(sums[name], hashlib.sha256(data).hexdigest(), name)
        # The supplier files the batch's README records (tools/frame_boards/lcsc-batch-1).
        self.assertTrue(sums["lcsc_batch_1-lcsc-bom.csv"].startswith("dcfe8fae"))
        self.assertTrue(sums["lcsc_batch_1-lcsc-cpl.csv"].startswith("7a363758"))
        self.assertIn("24 placements, 6 part numbers, KiCad positions 24 of 24", log.getvalue())


if __name__ == "__main__":
    unittest.main()
