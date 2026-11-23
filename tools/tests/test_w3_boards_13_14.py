"""Preview boards 13 and 14: the committed manifests are what the generator builds."""
import collections
import json
import sys
import unittest
import unittest.mock
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools/frame_boards"))
import w3_boards_13_14 as wb  # noqa: E402


class W3Boards13And14Tests(unittest.TestCase):
    def test_manifests_are_what_the_generator_builds(self):
        for board in wb.BOARDS:
            with self.subTest(board=board):
                path = ROOT / "tools/frame_boards" / board.replace("_", "-") / "manifest.json"
                self.assertEqual(json.loads(path.read_text(encoding="utf-8")), wb.build(board))

    def test_each_group_alternates_between_the_boards_and_none_fills_half_a_side(self):
        built = {board: wb.build(board) for board in wb.BOARDS}
        samples = {board: [r["pn"] for r in m["rows"] if r["purpose"] == "sample"] for board, m in built.items()}
        self.assertFalse(set(samples["preview_board_13"]) & set(samples["preview_board_14"]))
        self.assertEqual(sum(map(len, samples.values())), sum(g[-1] for g in wb.GROUPS))
        for board, manifest in built.items():
            counts = collections.Counter(r["group"] for r in manifest["rows"])
            self.assertLess(max(counts.values()), len(manifest["rows"]) / 2, board)

    def test_every_land_fits_its_box_at_any_angle(self):
        tree = json.loads((ROOT / "tools/frame_boards/preview-board-13/inputs.json").read_text())["packages"]["passive"]["tree"]
        wb.check_box(tree)
        with unittest.mock.patch.object(wb, "BOX", 8):
            with self.assertRaisesRegex(ValueError, "CHIP_2512"):
                wb.check_box(tree)


if __name__ == "__main__":
    unittest.main()
