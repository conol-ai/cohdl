"""Offline regression tests for generator-side RFC-034 supplier bindings."""

from __future__ import annotations

import difflib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


TOOLS = Path(__file__).resolve().parents[1]
ROOT = TOOLS.parent
sys.path.insert(0, str(TOOLS))

from sourcing.bindings import Bindings  # noqa: E402
import gen_stm32  # noqa: E402


MPN = "RC0201JR-070RL"


def map_row(pn="C00000"):
    return {
        "mfr": "Yageo",
        "mpn": MPN,
        "pn": pn,
        "rule": "exact",
        "category": "basic",
        "snapshot_sha256": "0" * 64,
        "date": "2026-10-30",
    }


def frame_row(version="0.2.3"):
    return {
        "id": "chip-0201",
        "package": "passive",
        "package_version": version,
        "footprint": "CHIP_0201",
        "supplier": "lcsc",
        "pn": "C00000",
        "mapping": {"dx_mm": 0.1, "dy_mm": -0.2, "rotate_deg": 90},
        "measured": None,
        "exception": "supplier preview uses a different frame",
        "evidence": {
            "method": "preview",
            "date": "2026-10-30",
            "record": "evidence/preview.txt",
        },
        "evidence_ref": None,
    }


class PassiveCopy:
    """Copy passive inputs and the mapping files, never its generated output."""

    def __init__(self, test):
        self.tempdir = tempfile.TemporaryDirectory()
        test.addCleanup(self.tempdir.cleanup)
        self.root = Path(self.tempdir.name)
        shutil.copytree(
            TOOLS,
            self.root / "tools",
            ignore=shutil.ignore_patterns("tests", "__pycache__", "*.pyc"),
        )
        package = self.root / "lib" / "passive"
        (package / "src").mkdir(parents=True)
        shutil.copy2(ROOT / "lib" / "passive" / "cohdl.toml", package / "cohdl.toml")
        (self.root / "evidence").mkdir()
        (self.root / "evidence" / "preview.txt").write_text("offline test evidence\n")
        self.output = package / "src" / "resistors_0201.cohdl"
        self.env = dict(os.environ, PYTHONPATH=str(self.root / "tools"))
        self.run()

    def write_tables(self, mappings, frames):
        data = self.root / "tools" / "sourcing_data"
        data.mkdir(exist_ok=True)
        (data / "lcsc_map.json").write_text(
            json.dumps({"schema_version": 1, "supplier": "lcsc", "rows": mappings})
        )
        (data / "jlc_frames.json").write_text(
            json.dumps({"schema_version": 1, "rows": frames})
        )

    def run(self):
        subprocess.run(
            [sys.executable, "-B", str(self.root / "tools" / "gen_passive.py")],
            env=self.env,
            check=True,
            capture_output=True,
            text=True,
        )
        return self.output.read_text()


class GeneratorBindingTests(unittest.TestCase):
    def test_fake_tables_add_exactly_one_clause_with_nonzero_mapping(self):
        copy = PassiveCopy(self)
        before = copy.output.read_text()
        copy.write_tables([map_row()], [frame_row()])
        after = copy.run()
        changes = [
            line
            for line in difflib.ndiff(before.splitlines(), after.splitlines())
            if not line.startswith("  ")
        ]
        self.assertEqual(len([line for line in changes if line.startswith("- ")]), 1)
        self.assertEqual(len([line for line in changes if line.startswith("+ ")]), 3)
        self.assertEqual(after.count('supplier lcsc "C00000"'), 1)
        self.assertIn('supplier lcsc "C00000" at (0.1mm, -0.2mm) rotate 90', after)
        self.assertIn('pub part R_0R_J_0201:', after.split('supplier lcsc')[0].split("#[doc")[-1])
        subprocess.run(
            [sys.executable, "-B", str(copy.root / "tools" / "gen_passive.py"), "--check"],
            env=copy.env,
            check=True,
            capture_output=True,
        )

    def test_alt_only_mapping_inherits_primary_footprint(self):
        copy = PassiveCopy(self)
        mapping = map_row("C54321")
        mapping.update(mfr="Vishay", mpn="CRCW02011R00FKED")
        frame = frame_row()
        frame.update(pn="C54321")
        copy.write_tables([mapping], [frame])
        after = copy.run()
        self.assertEqual(after.count('supplier lcsc "C54321"'), 1)
        self.assertNotIn('supplier lcsc "C00000"', after)
        declaration = after[after.index("pub part R_1R_F_0201:"):]
        declaration = declaration[:declaration.index("\n}\n")]
        self.assertIn(
            'alt { mfr: "Vishay", mpn: "CRCW02011R00FKED",\n'
            '              supplier lcsc "C54321" at (0.1mm, -0.2mm) rotate 90 }',
            declaration,
        )
        self.assertNotIn("supplier lcsc", declaration.split("alt {", 1)[0])

    def test_duplicate_pn_and_stale_frame_version_do_not_generate(self):
        for duplicate, stale in ((True, False), (False, True)):
            with self.subTest(duplicate=duplicate, stale=stale):
                copy = PassiveCopy(self)
                mappings = [map_row(), map_row("C00001")] if duplicate else [map_row()]
                version = "0.0.1" if stale else "0.2.3"
                copy.write_tables(mappings, [frame_row(version)])
                self.assertNotIn("supplier lcsc", copy.run())

    def test_missing_tables_and_invalid_evidence_are_silent(self):
        copy = PassiveCopy(self)
        self.assertNotIn("supplier lcsc", copy.output.read_text())
        row = frame_row()
        row["evidence"]["record"] = "missing.txt"
        copy.write_tables([map_row()], [row])
        self.assertNotIn("supplier lcsc", copy.run())

    def test_integral_numeric_rotation_is_canonical_and_invalid_angles_are_silent(self):
        for rotate in (90, 90.0):
            with self.subTest(rotate=rotate, valid=True):
                copy = PassiveCopy(self)
                frame = frame_row()
                frame["mapping"]["rotate_deg"] = rotate
                copy.write_tables([map_row()], [frame])
                self.assertIn('supplier lcsc "C00000" at (0.1mm, -0.2mm) rotate 90', copy.run())
        for rotate in (90.5, -1, 360, float("inf"), float("nan"), True):
            with self.subTest(rotate=rotate, valid=False):
                copy = PassiveCopy(self)
                frame = frame_row()
                frame["mapping"]["rotate_deg"] = rotate
                copy.write_tables([map_row()], [frame])
                self.assertNotIn("supplier lcsc", copy.run())

    def test_cross_package_footprint_uses_owner_package_and_version(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for package, version in (("@espressif/esp32", "0.3.1"), ("qfn", "0.2.1")):
                manifest = root / "lib" / package / "cohdl.toml"
                manifest.parent.mkdir(parents=True)
                manifest.write_text(
                    f'[package]\nname = "{package}"\nversion = "{version}"\n'
                )
            bindings = Bindings(root)
            bindings._by_identity[("Espressif", "ESP32-D0WDQ6")] = ["C12345"]
            for package, version, expected in (
                ("qfn", "0.2.1", 'supplier lcsc "C12345"'),
                ("@espressif/esp32", "0.3.1", ""),
                ("qfn", "0.2.0", ""),
            ):
                key = (package, version, "ESPRESSIF_QFN48E_0P4_6", "C12345")
                bindings._frames = {
                    key: {
                        "mapping": {"dx_mm": 0, "dy_mm": 0, "rotate_deg": 0},
                        "exception": None,
                    }
                }
                self.assertEqual(
                    bindings.clause(
                        root / "lib/@espressif/esp32/cohdl.toml",
                        "qfn::ESPRESSIF_QFN48E_0P4_6",
                        "Espressif",
                        "ESP32-D0WDQ6",
                    ),
                    expected,
                )

    def test_stm32_overlay_allows_new_optional_keys(self):
        data = {
            "schema_version": gen_stm32.PARTS_SCHEMA,
            "documents": [],
            "package_variants": [],
            "parts": [],
            "future_optional": {"anything": True},
        }
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch.object(gen_stm32, "PARTS", Path(directory) / "parts.json"):
                gen_stm32.PARTS.write_text(json.dumps(data))
                with mock.patch.object(gen_stm32, "PARTS_SHA256", gen_stm32.hashlib.sha256(gen_stm32.PARTS.read_bytes()).hexdigest()):
                    loaded = gen_stm32.load_audited_parts(
                        {"models": [], "pinouts": []},
                        {"rows": [], "by_identity": {}},
                    )
        self.assertEqual(loaded, [])

    def test_generated_clause_compiles(self):
        compiler = ROOT / "target" / "debug" / "cohdl"
        if not compiler.is_file():
            self.skipTest("target/debug/cohdl is not built")
        copy = PassiveCopy(self)
        mapping = map_row("C54321")
        mapping.update(mfr="Vishay", mpn="CRCW02011R00FKED")
        frame = frame_row()
        frame.update(pn="C54321")
        copy.write_tables([mapping], [frame])
        generated = copy.run()
        declaration = generated[generated.index("#[doc", generated.index("pub part R_0R_J_0201") - 20):]
        declaration = declaration[:declaration.index("\n}\n") + 3]
        source = """
pub footprint CHIP_0201 {}
pub device ChipResistor_0ohm_5pct_R0201 { pins { A: 1 [passive], B: 2 [passive] } }
pub part R_0R_J_0201: ChipResistor_0ohm_5pct_R0201 {
""" + declaration[declaration.index("    primary"):]
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory)
            (project / "src").mkdir()
            (project / "cohdl.toml").write_text('[package]\nname = "binding_test"\nversion = "0.1.0"\n')
            (project / "src" / "main.cohdl").write_text(source)
            result = subprocess.run(
                [str(compiler), "check", str(project), "--no-std"],
                capture_output=True,
                text=True,
            )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
