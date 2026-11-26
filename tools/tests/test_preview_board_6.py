import collections
import json
from pathlib import Path
import re
import sys
import unittest

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS / "frame_boards"))
import preview_board_6
from sourcing import w3_rule


class PreviewBoard6Tests(unittest.TestCase):
    def test_plan_coverage_and_independent_views(self):
        manifest = preview_board_6.build()
        folder = TOOLS / "frame_boards"
        self.assertEqual(manifest, json.loads((folder / "preview-board-6/manifest.json").read_text()))
        plan = (folder / "uniroyal-larger-samples.md").read_text()
        selected = set(re.findall(r"^\| (\w+) \| \d+ \| \d+ \| (C\d+) \| (\w+) \|", plan, re.M))
        samples = [r for r in manifest["rows"] if r["purpose"] == "sample"]
        self.assertEqual({(r["group"].split()[-1], r["pn"], r["mpn"]) for r in samples}, selected)
        self.assertEqual((len(samples), len(manifest["rows"])), (14, 18))
        for r in manifest["rows"]:
            self.assertCountEqual([(p["side"], p["rotate"]) for p in r["placements"]],
                                  [(s, a) for s in ("top", "bottom") for a in (0, 37)])
        for side in ("top", "bottom"):
            counts = collections.Counter(r["group"] for r in manifest["rows"]
                                         for p in r["placements"] if p["side"] == side)
            for r in samples:
                self.assertTrue(w3_rule.bias_supported(dict(bias_view=side, bias_bodies=counts.total(),
                                                            bias_other_bodies=counts.total() - counts[r["group"]])))


if __name__ == "__main__":
    unittest.main()
