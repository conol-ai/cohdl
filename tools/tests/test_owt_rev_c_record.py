"""The three bindings read on a real board (jlc_previews/2026-11-12-owt-rev-c)."""

import contextlib
import hashlib
import io
import json
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "frame_boards"))
import measure_preview as mp

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "tools" / "sourcing_data"
RECORD = DATA / "jlc_previews" / "2026-11-12-owt-rev-c"
BOARD_4 = DATA / "jlc_previews" / "2026-11-11-board-4"

# frame row id: (package, version, footprint, part number, rotate, board footprint id,
#                library source, placements on the real board)
ROWS = {
    "owtc-fpsot236-c7519": (
        "esd", "0.1.4", "FP_SOT_23_6", "C7519", 270, "esd-FP_SOT_23_6",
        "lib/esd/src/usblc6.cohdl", ["U20"]),
    "owtc-fptidbv0006a-c131941": (
        "load-switch", "0.2.1", "FP_TI_DBV0006A", "C131941", 180, "load_switch-FP_TI_DBV0006A",
        "lib/load-switch/src/tps22918.cohdl", ["U2", "U10"]),
    "owtc-sop8p65x490x110n-c189961": (
        "@contrib/audio-amp", "0.2.1", "SOP8P65X490X110N", "C189961", 270,
        "contrib_audio_amp-SOP8P65X490X110N", "lib/@contrib/audio-amp/src/ns4150b.cohdl", ["U1"]),
}


def off_pad_1(delta):
    """Degrees between the pin 1 mark and pad 1, 0..180."""
    return min(delta % 360, 360 - delta % 360)


class OwtRevCRecordTests(unittest.TestCase):
    def setUp(self):
        frames = json.loads((DATA / "jlc_frames.json").read_text(encoding="utf-8"))["rows"]
        self.frames = {row["id"]: row for row in frames}
        self.extract = {fp["ref"]: fp for fp in json.loads((RECORD / "board-extract.json").read_text())["fps"]}
        self.readings = {row["ref"]: row for row in json.loads((RECORD / "measurements.json").read_text())}

    def test_each_row_is_the_binding_and_cites_this_record(self):
        for row_id, (package, version, footprint, pn, rotate, _, source, _) in ROWS.items():
            with self.subTest(row=row_id):
                row = self.frames[row_id]
                self.assertEqual(
                    (row["package"], row["package_version"], row["footprint"], row["supplier"], row["pn"]),
                    (package, version, footprint, "lcsc", pn))
                mapping = {"dx_mm": 0, "dy_mm": 0, "rotate_deg": rotate}
                self.assertEqual((row["mapping"], row["measured"]), (mapping, mapping))
                self.assertEqual(row["evidence"]["record"], str((RECORD / "README.md").relative_to(ROOT)))
                self.assertIn(str((BOARD_4 / "README.md").relative_to(ROOT)), row["evidence"]["note"])
                self.assertIn(f'supplier lcsc "{pn}" rotate {rotate} }}', (ROOT / source).read_text(encoding="utf-8"))
                manifest = (ROOT / source).parents[1] / "cohdl.toml"
                self.assertIn(f'version = "{version}"', manifest.read_text(encoding="utf-8"))

    def test_with_the_mapping_applied_pin_1_reads_on_pad_1(self):
        cpl = (RECORD / "lcsc-cpl-rows.csv").read_text().splitlines()
        sent = {line.split(",")[0]: int(line.split(",")[4]) for line in cpl[1:]}
        refs = []
        for row_id, (_, _, _, _, rotate, fpid, _, placements) in ROWS.items():
            for ref in placements:
                with self.subTest(row=row_id, ref=ref):
                    fp, reading = self.extract[ref], self.readings[ref]
                    self.assertEqual(fp["fpid"], fpid)
                    self.assertEqual(len(reading["dots"]), 1)
                    # 15 deg: a mark drawn beside a SOT-23-6 lead reads 10-12 deg off; a wrong
                    # quarter reads 78 deg or more.
                    self.assertLess(off_pad_1(reading["delta_ccw_from_top"]), 15)
                    # The CPL row that was previewed carries the mapping: top is the board angle
                    # plus rotate, bottom (mirrored convention) 180 minus the board angle plus rotate.
                    base = 180 - fp["angle"] if fp["bottom"] else fp["angle"]
                    self.assertEqual(sent[ref], (base + rotate) % 360)
                    refs.append(ref)
        self.assertEqual(sorted(refs), sorted(self.readings))
        self.assertEqual(sorted(refs), sorted(self.extract))

    def test_board_4_read_the_same_rotation_with_no_mapping(self):
        fps = {fp["ref"]: fp for fp in json.loads((BOARD_4 / "package" / "board.json").read_text())["fps"]}
        readings = json.loads((BOARD_4 / "measurements.json").read_text())
        for row_id, (_, _, _, _, rotate, fpid, _, _) in ROWS.items():
            with self.subTest(row=row_id):
                copies = [row for row in readings if fps[row["ref"]]["fpid"] == fpid]
                self.assertEqual(sorted(row["side"] for row in copies), ["bottom", "bottom", "top", "top"])
                for row in copies:
                    delta = row["delta_ccw_from_top"]
                    quarter = round(delta / 90) * 90 % 360
                    self.assertLess(abs(delta - round(delta / 90) * 90), 15)
                    # The rotate that cancels a reading is 360 minus it on top and the reading
                    # itself from below.
                    self.assertEqual(quarter if row["side"] == "bottom" else (360 - quarter) % 360, rotate)

    def test_the_readme_hashes_and_the_source_hashes_match_the_files(self):
        readme = (RECORD / "README.md").read_text(encoding="utf-8")
        listed = dict(re.findall(r"^\| `([^`]+)` \| `([0-9a-f]{64})` \|$", readme, re.M))
        files = sorted(str(p.relative_to(RECORD)) for p in RECORD.rglob("*") if p.is_file() and p.name != "README.md")
        self.assertEqual(sorted(listed), files)
        self.assertEqual(len(files), 14)
        for name, digest in listed.items():
            with self.subTest(file=name):
                self.assertEqual(hashlib.sha256((RECORD / name).read_bytes()).hexdigest(), digest)
        source = dict(reversed(line.split("  ")) for line in (RECORD / "source-SHA256SUMS").read_text().splitlines())
        for name in ("exports/top.png", "exports/bottom.png", "crops/U1-top.png", "crops/U2-bottom.png",
                     "crops/U10-bottom.png", "crops/U20-top.png"):
            self.assertEqual(listed[name], source[name])

    @unittest.skipUnless(sys.platform == "darwin" and shutil.which("sips"), "needs macOS sips to convert the exports")
    def test_the_exports_calibrate_on_the_board_and_reproduce_the_readings(self):
        with tempfile.TemporaryDirectory() as td:
            for side in ("top", "bottom"):
                subprocess.run(["sips", "-s", "format", "bmp", str(RECORD / "exports" / f"{side}.png"),
                                "--out", str(Path(td) / f"{side}.bmp")], check=True, capture_output=True)
            # The board's bounding box in each export, as the source record's features.json has it.
            for side, box in (("top", (162, 168, 2263, 3804)), ("bottom", (9, 168, 2110, 3804))):
                view = mp.View(str(Path(td) / f"{side}.bmp"), (0.0, 0.0, 1.0, 1.0), side == "bottom")
                self.assertEqual((view.left, view.top, view.right, view.low), box, side)
            out = Path(td) / "out.json"
            with contextlib.redirect_stdout(io.StringIO()):
                # The record was made before marks were attributed by number: attribution 1.
                mp.measure(str(RECORD / "board-extract.json"), str(Path(td) / "top.bmp"),
                           str(Path(td) / "bottom.bmp"), str(out), attribution=1)
                current = Path(td) / "current.json"
                mp.measure(str(RECORD / "board-extract.json"), str(Path(td) / "top.bmp"),
                           str(Path(td) / "bottom.bmp"), str(current))
            self.assertEqual(out.read_bytes(), (RECORD / "measurements.json").read_bytes())
            # Attribution 2, the current one, gives each part the same one mark at the same
            # angle, and passes it as a mark at pad 1.
            rows = json.loads(current.read_text())
            self.assertEqual([dict(row, attribution=None, mark_ok=None) for row in rows],
                             [dict(row, attribution=None, mark_ok=None) for row in json.loads(out.read_text())])
            self.assertTrue(all(row["attribution"] == 2 and row["mark_ok"] is True for row in rows))

    def test_the_record_names_no_local_path(self):
        for path in sorted(p for p in RECORD.rglob("*") if p.is_file()):
            data = path.read_bytes()
            for needle in (b"/Users/", b"/home/", b"/private/"):
                self.assertNotIn(needle, data, f"{path.relative_to(ROOT)} names {needle.decode()}")


if __name__ == "__main__":
    unittest.main()
