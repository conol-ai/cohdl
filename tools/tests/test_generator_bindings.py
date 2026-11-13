"""Offline regression tests for generator-side RFC-034 supplier bindings."""

from __future__ import annotations

import difflib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock


TOOLS = Path(__file__).resolve().parents[1]
ROOT = TOOLS.parent
sys.path.insert(0, str(TOOLS))

from sourcing.bindings import FRAME_FIELDS, Bindings, _evidence, package_metadata  # noqa: E402
from sourcing.w3_rule import RULE as W3_RULE  # noqa: E402
import gen_esp32  # noqa: E402
import gen_passive  # noqa: E402
import gen_stm32  # noqa: E402


MPN = "RC0201JR-070RL"
PASSIVE_VERSION = package_metadata(ROOT / "lib" / "passive" / "cohdl.toml")[1]


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


def frame_row(version=PASSIVE_VERSION):
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
    """Copy passive inputs, never its generated output; the tables start absent."""

    def __init__(self, test):
        self.tempdir = tempfile.TemporaryDirectory()
        test.addCleanup(self.tempdir.cleanup)
        self.root = Path(self.tempdir.name)
        shutil.copytree(
            TOOLS,
            self.root / "tools",
            ignore=shutil.ignore_patterns("tests", "__pycache__", "*.pyc"),
        )
        # Start from no tables, so the real ones' bindings stay out of the baseline.
        for table in ("lcsc_map.json", "jlc_frames.json"):
            (self.root / "tools" / "sourcing_data" / table).unlink()
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
        self.assertEqual(len([line for line in changes if line.startswith("+ ")]), 2)
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
            '          supplier lcsc "C54321" at (0.1mm, -0.2mm) rotate 90 }',
            declaration,
        )
        self.assertNotIn("supplier lcsc", declaration.split("alt {", 1)[0])

    def test_duplicate_pn_and_stale_frame_version_do_not_generate(self):
        for duplicate, stale in ((True, False), (False, True)):
            with self.subTest(duplicate=duplicate, stale=stale):
                copy = PassiveCopy(self)
                mappings = [map_row(), map_row("C00001")] if duplicate else [map_row()]
                version = "0.0.1" if stale else PASSIVE_VERSION
                copy.write_tables(mappings, [frame_row(version)])
                self.assertNotIn("supplier lcsc", copy.run())

    def test_missing_tables_and_invalid_evidence_are_silent(self):
        copy = PassiveCopy(self)
        self.assertNotIn("supplier lcsc", copy.output.read_text())
        row = frame_row()
        row["evidence"]["record"] = "missing.txt"
        copy.write_tables([map_row()], [row])
        self.assertNotIn("supplier lcsc", copy.run())
        for extra in ({"note": " "}, {"note": 1}, {"by": "someone"}):
            with self.subTest(extra=extra):
                row = frame_row()
                row["evidence"].update(extra)
                copy.write_tables([map_row()], [row])
                self.assertNotIn("supplier lcsc", copy.run())

    def test_an_evidence_note_is_accepted(self):
        copy = PassiveCopy(self)
        row = frame_row()
        row["evidence"]["note"] = "non-polar chip part: orientation known mod 180 only"
        copy.write_tables([map_row()], [row])
        self.assertIn('supplier lcsc "C00000"', copy.run())

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

    def test_nested_scoped_and_hyphenated_owners_resolve_through_package_roots(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for package in ("@espressif/esp32", "load-switch", "usb", "local"):
                manifest = root / "lib" / package / "cohdl.toml"
                manifest.parent.mkdir(parents=True)
                manifest.write_text(f'[package]\nname = "{package}"\nversion = "1.0.0"\n')
            bindings = Bindings(root)
            bindings._by_identity[("M", "N")] = ["C1"]
            zero = {"mapping": {"dx_mm": 0, "dy_mm": 0, "rotate_deg": 0}, "exception": None}
            local = root / "lib/local/cohdl.toml"
            for reference, package in (
                ("espressif_esp32::FP_E", "@espressif/esp32"),
                ("load_switch::FP_L", "load-switch"),
                ("usb::connectors::type_c::FP_C", "usb"),
                ("modules::FP_M", "local"),
                ("FP_B", "local"),
            ):
                with self.subTest(reference=reference):
                    short = reference.rsplit("::", 1)[-1]
                    bindings._frames = {(package, "1.0.0", short, "C1"): zero}
                    self.assertEqual(bindings.clause(local, reference, "M", "N"), 'supplier lcsc "C1"')

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
            if os.environ.get("COHDL_REQUIRE_BINARY") == "1":
                self.fail("target/debug/cohdl is not built")
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

    def test_every_generated_entry_with_a_clause_is_in_fmt_canonical_form(self):
        """Every AVL entry each generator can emit, with a binding, is fmt-clean.

        A binding is forced onto every entry, in a short and a long form, so
        the entries wrap at every position `cohdl fmt` can choose.
        """
        compiler = ROOT / "target" / "debug" / "cohdl"
        if not compiler.is_file():
            if os.environ.get("COHDL_REQUIRE_BINARY") == "1":
                self.fail("target/debug/cohdl is not built")
            self.skipTest("target/debug/cohdl is not built")

        class Every:
            def clause(self, manifest, footprint, mfr, mpn):
                if sum(map(ord, mpn)) % 2:
                    return f'supplier lcsc "C{len(mpn)}"'
                return f'supplier lcsc "C{len(mpn) * 7919}" at (0.125mm, -0.25mm) rotate 270'

        bindings = Every()
        outputs = {}
        verified = gen_passive.load_verified()
        for size in gen_passive.R_POWER:
            outputs[f"resistors_{size}"] = gen_passive.emit_resistors(size, verified, bindings)
        available = gen_passive.load_mlcc()
        samsung = gen_passive.load_samsung()
        for size in sorted({key.split("|")[0] for key in available}):
            outputs[f"capacitors_{size}"] = gen_passive.emit_capacitors(
                size, available, gen_passive.VERIFIED_ALTERNATES, samsung, verified, bindings
            )
        selector = gen_esp32.load_snapshot(gen_esp32.SELECTOR_SNAPSHOT, gen_esp32.SELECTOR_SNAPSHOT_SHA256)
        kicad = gen_esp32.load_snapshot(gen_esp32.SYMBOL_SNAPSHOT, gen_esp32.SYMBOL_SNAPSHOT_SHA256)
        selected, _ = gen_esp32.classify(gen_esp32.validate_selector(selector), gen_esp32.symbol_map(kicad))
        outputs["esp32"] = gen_esp32.render_parts(selected, bindings)
        stm32 = gen_stm32.generated_sources(
            gen_stm32.load_snapshot(), gen_stm32.load_kicad_part_catalog(), bindings
        )
        outputs.update((f"stm32_{Path(name).stem}", text) for name, text in stm32.items())
        with tempfile.TemporaryDirectory() as directory:
            for name, text in outputs.items():
                Path(directory, f"{name}.cohdl").write_text(text)
            result = subprocess.run(
                [str(compiler), "fmt", directory, "--check"], capture_output=True, text=True
            )
        self.assertEqual(result.returncode, 0, result.stdout[:4000] + result.stderr)
        joined = "".join(outputs.values())
        self.assertGreater(joined.count("supplier lcsc"), 4000)
        self.assertIn("\n          supplier lcsc", joined)


class CommittedTablesTests(unittest.TestCase):
    """The real tables against the committed, published library sources."""

    DATA = TOOLS / "sourcing_data"

    def test_committed_generated_sources_match_the_real_tables(self):
        # A row added to jlc_frames.json for a published package version binds at
        # once and rewrites that version's sources (the W3 pilot rows did, in 21c4ca5).
        for generator in ("gen_passive.py", "gen_esp32.py", "gen_stm32.py"):
            with self.subTest(generator=generator):
                result = subprocess.run(
                    [sys.executable, "-I", "-B", str(TOOLS / generator), "--check"],
                    cwd=ROOT,
                    capture_output=True,
                    text=True,
                )
                self.assertEqual(result.returncode, 0, result.stderr + result.stdout)

    # Every archive under jlc_previews/*/frames.json, with the row ids it must hold.
    # A new archive has to be listed here, so its rows are checked, not just found.
    ARCHIVED = {
        "2026-11-07-w3-pilot-0402": None,  # 88 w3p- rows, checked by count and prefix
        "2026-11-08-probe": [
            "probe-diom5336x265n-c19077561",
            "probe-diom7959x261n-c19077609",
            "probe-sodfl3818x120n-c19077510",
        ],
        "2026-11-08-reverify": [
            "rv1-sod2613x110n-c7420331",
            "rv1-diom5026x241n-c7420365",
            "rv1-sod1608x70n-c7420320",
        ],
    }

    def test_archived_reverify_diode_rows_have_no_library_part(self):
        # The 2026-11-08 diode rows are archived because no library source lists their
        # part numbers. When a part is added (W4), this fails: adopt the row then,
        # re-keyed to the package version the part ships in.
        archive = self.DATA / "jlc_previews" / "2026-11-08-reverify" / "frames.json"
        rows = json.loads(archive.read_text(encoding="utf-8"))["rows"]
        self.assertEqual(
            sorted((row["package"], row["footprint"], row["pn"]) for row in rows),
            [
                ("do214", "DIOM5026X241N", "C7420365"),
                ("sod", "SOD1608X70N", "C7420320"),
                ("sod", "SOD2613X110N", "C7420331"),
            ],
        )
        sources = sorted((ROOT / "lib").rglob("*.cohdl"))
        self.assertGreater(len(sources), 100)
        for row in rows:
            self.assertEqual(row["mapping"]["rotate_deg"], 180, row["id"])
            self.assertEqual(row["package_version"], "0.1.0", row["id"])
            needle = f'"{row["pn"]}"'
            listed = [
                str(source.relative_to(ROOT))
                for source in sources
                if needle in source.read_text(encoding="utf-8")
            ]
            self.assertEqual(listed, [], row["id"])

    def test_archived_probe_tvs_rows_have_no_library_part(self):
        # The 2026-11-08 probe's bidirectional TVS rows are archived for the same
        # reason: no library source lists their part numbers. They were read by the
        # body only, on a board built with zero correction, so each row is rotate 0
        # with no offset. When a part is added (W4), this fails: adopt the row then,
        # re-keyed to the package version the part ships in.
        archive = self.DATA / "jlc_previews" / "2026-11-08-probe" / "frames.json"
        rows = json.loads(archive.read_text(encoding="utf-8"))["rows"]
        self.assertEqual(
            sorted((row["package"], row["footprint"], row["pn"]) for row in rows),
            [
                ("do214", "DIOM5336X265N", "C19077561"),
                ("do214", "DIOM7959X261N", "C19077609"),
                ("sod", "SODFL3818X120N", "C19077510"),
            ],
        )
        sources = sorted((ROOT / "lib").rglob("*.cohdl"))
        self.assertGreater(len(sources), 100)
        zero = {"dx_mm": 0, "dy_mm": 0, "rotate_deg": 0}
        for row in rows:
            self.assertEqual(row["mapping"], zero, row["id"])
            self.assertEqual(row["measured"], zero, row["id"])
            self.assertIsNone(row["exception"], row["id"])
            self.assertEqual(row["package_version"], "0.1.0", row["id"])
            self.assertIn("body only", row["evidence"]["note"], row["id"])
            needle = f'"{row["pn"]}"'
            listed = [
                str(source.relative_to(ROOT))
                for source in sources
                if needle in source.read_text(encoding="utf-8")
            ]
            self.assertEqual(listed, [], row["id"])

    def test_archived_preview_rows_stay_out_of_the_bound_table(self):
        frames = json.loads((self.DATA / "jlc_frames.json").read_text(encoding="utf-8"))["rows"]
        bound_ids = {row["id"] for row in frames}
        cited = {row["evidence"]["record"] for row in frames if row["evidence"] is not None}
        archives = sorted((self.DATA / "jlc_previews").glob("*/frames.json"))
        # Not vacuous: exactly the known archives exist, each with its expected rows.
        self.assertEqual([archive.parent.name for archive in archives], sorted(self.ARCHIVED))
        for archive in archives:
            with self.subTest(archive=str(archive.relative_to(ROOT))):
                document = json.loads(archive.read_text(encoding="utf-8"))
                self.assertEqual(set(document), {"schema_version", "status", "note", "rows"})
                self.assertEqual(document["status"], "archived")
                rows = document["rows"]
                ids = [row["id"] for row in rows]
                expected = self.ARCHIVED[archive.parent.name]
                if expected is None:
                    self.assertEqual(len(rows), 88)
                    self.assertTrue(all(row_id.startswith("w3p-") for row_id in ids))
                else:
                    self.assertEqual(ids, expected)
                self.assertEqual(len(set(ids)), len(ids))
                record = str((archive.parent / "README.md").relative_to(ROOT))
                for row in rows:
                    self.assertEqual(set(row), FRAME_FIELDS, row["id"])
                    self.assertEqual(row["evidence"]["record"], record, row["id"])
                self.assertEqual(bound_ids & {row["id"] for row in rows}, set())
                self.assertNotIn(record, cited)


class SampledEvidenceTests(unittest.TestCase):
    """The generator accepts a W3 inferred row only when every check of its group
    record holds: the same counterexamples ``tests/library/supplier.rs`` rejects."""

    FIXTURE = ROOT / "tests" / "fixtures" / "library-ci" / "s-sampled"
    CASES = ("rule", "paused", "scope", "key", "land", "member", "zero-stock", "sample", "few",
             "own-evidence", "reading", "offset", "raw", "arithmetic", "method", "not-drawn", "mapping",
             "two-kinds", "field")

    def inferred(self, frames: str) -> bool:
        rows = json.loads((self.FIXTURE / frames).read_text(encoding="utf-8"))["rows"]
        by_id = {row["id"]: row for row in rows}
        return _evidence(by_id["inferred"], by_id, self.FIXTURE)

    def test_the_fixture_record_carries_the_rule_both_checkers_hold(self):
        # supplier.rs accepts this record only if its rule equals its own copy,
        # so the Rust and Python rules cannot drift apart unnoticed.
        record = json.loads((self.FIXTURE / "w3_record.json").read_text(encoding="utf-8"))
        self.assertEqual(record["rule"], W3_RULE)

    def test_a_valid_inferred_row_is_accepted(self):
        self.assertTrue(self.inferred("jlc_frames.json"))

    def test_every_counterexample_is_rejected(self):
        for case in self.CASES:
            with self.subTest(case=case):
                self.assertFalse(self.inferred(f"jlc_frames.{case}.json"))

    def test_the_rust_audit_names_the_same_counterexamples(self):
        source = (ROOT / "tests" / "library" / "supplier.rs").read_text(encoding="utf-8")
        files = {path.name for path in self.FIXTURE.glob("jlc_frames.*.json")}
        self.assertEqual(files, {f"jlc_frames.{case}.json" for case in self.CASES})
        cases = source.split("fn every_w3_sampled_rule_has_an_independent_counterexample")[1]
        cases = cases.split("];")[0]
        self.assertEqual(re.findall(r'\(\s*"([a-z-]+)",', cases), list(self.CASES))

    def test_a_sample_row_cannot_be_inferred_itself(self):
        rows = json.loads((self.FIXTURE / "jlc_frames.json").read_text(encoding="utf-8"))["rows"]
        by_id = {row["id"]: row for row in rows}
        self.assertTrue(_evidence(by_id["s1"], by_id, self.FIXTURE))
        row = dict(by_id["s1"], evidence=None, sampled=by_id["inferred"]["sampled"])
        self.assertFalse(_evidence(row, dict(by_id, s1=row), self.FIXTURE))


if __name__ == "__main__":
    unittest.main()
