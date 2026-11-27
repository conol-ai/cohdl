"""Body outline readings for the Zener copies read with the cathode at pad 1.

Written 2026-11-21, when those readings became bindings. It is
`../2026-11-09-reverify-2/body_axes.py` unchanged, with the SOD-123 land's axis
(pad 1, the cathode, to pad 2) added, run on a record's lossless pixels:

    python3 -I tools/sourcing_data/jlc_previews/2026-11-21-board-8/body.py RECORD_DIR OUT.json

from the repository root, for board 8 or board 9. It reads every copy of each
part number whose `readings.json` entry is "cathode at pad 1" and writes one
row per designator (see body_axes.py for the fields).
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "tools/frame_boards"))
sys.path.insert(0, str(ROOT / "tools/sourcing_data/jlc_previews/2026-11-09-reverify-2"))
import measure_preview as mp  # noqa: E402
import body_axes as ba  # noqa: E402

ba.AXIS["sod-SOD3716X135N"] = ("1", "2")


def main(record, out):
    record = Path(record)
    doc = json.loads((record / "package/board.json").read_text(encoding="utf-8"))
    outline = mp.outline(doc)
    views = {False: mp.View(str(record / "top.pixels.png"), outline, False),
             True: mp.View(str(record / "bottom.pixels.png"), outline, True)}
    parts = json.loads((record / "readings.json").read_text(encoding="utf-8"))["parts"]
    refs = {d for p in parts if p["reading"] == "cathode at pad 1" for d in p["designators"]}
    rows = {fp["ref"]: ba.read(views[fp["bottom"]], fp) for fp in doc["fps"] if fp["ref"] in refs}
    Path(out).write_text(json.dumps(rows, indent=1, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main(*sys.argv[1:])
