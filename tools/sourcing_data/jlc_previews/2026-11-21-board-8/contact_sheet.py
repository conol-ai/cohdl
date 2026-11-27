"""Contact sheets of a preview record's copies, for reading polarity by eye.

Usage (from the repository root):
  python3 -I tools/sourcing_data/jlc_previews/2026-11-21-board-8/contact_sheet.py RECORD_DIR OUT_PREFIX [PER_SHEET]

Crops each footprint of RECORD_DIR/package/board.json from the record's lossless
`top.pixels.png` / `bottom.pixels.png` (a 5.6 mm square around its origin,
calibrated by `measure_preview.View`), scales each to 170 px and lays them out
eight to a row in designator order, so a part number's four copies (top 0, top
37, bottom 0, bottom 37) sit side by side. Writes OUT_PREFIX<k>.png. Uses only
macOS `sips`; nothing is fetched. The bottom export is the view from below.
"""
import json, os, struct, subprocess, sys, tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "tools/frame_boards"))
import measure_preview as mp

HALF, SIZE, COLS = 2.8, 170, 8


def read_bmp(path):
    d = Path(path).read_bytes()
    off = struct.unpack_from("<I", d, 10)[0]
    w, h = struct.unpack_from("<ii", d, 18)
    bpp = struct.unpack_from("<H", d, 28)[0] // 8
    row = (w * bpp + 3) // 4 * 4
    rows = [d[off + i * row: off + i * row + w * bpp] for i in range(abs(h))]
    return w, abs(h), bpp, rows[::-1] if h > 0 else rows


def main(record, prefix, per_sheet=64):
    record = Path(record)
    board = json.loads((record / "package/board.json").read_text())
    outline, views = mp.outline(board), {}
    fps = sorted(board["fps"], key=lambda f: int("".join(c for c in f["ref"] if c.isdigit()) or 0))
    with tempfile.TemporaryDirectory() as td:
        tiles = []
        for f in fps:
            side = "bottom" if f.get("bottom") else "top"
            v = views.get(side) or views.setdefault(side, mp.View(str(record / f"{side}.pixels.png"), outline, side == "bottom"))
            ix = (v.right - (f["x"] - outline[0]) * v.sx) if v.bottom else (v.left + (f["x"] - outline[0]) * v.sx)
            iy = v.top + (f["y"] - outline[1]) * v.sy
            w = int(2 * HALF * v.sx)
            crop, bmp = os.path.join(td, f["ref"] + ".png"), os.path.join(td, f["ref"] + ".bmp")
            subprocess.run(["sips", "-c", str(w), str(w), "--cropOffset", str(int(iy - HALF * v.sy)),
                            str(int(ix - HALF * v.sx)), str(record / f"{side}.pixels.png"), "--out", crop],
                           check=True, capture_output=True)
            subprocess.run(["sips", "-z", str(SIZE), str(SIZE), "-s", "format", "bmp", crop, "--out", bmp],
                           check=True, capture_output=True)
            tiles.append(read_bmp(bmp))
        white = b"\xff\xff\xff"
        for k in range(0, len(tiles), per_sheet):
            sheet, out_rows = tiles[k:k + per_sheet], []
            for r in range((len(sheet) + COLS - 1) // COLS):
                for y in range(SIZE):
                    line = b""
                    for c in range(COLS):
                        i = r * COLS + c
                        line += (b"".join(sheet[i][3][y][x * 3:x * 3 + 3] for x in range(SIZE)) if i < len(sheet)
                                 else white * SIZE) + white * 4
                    out_rows.append(line)
                out_rows += [white * (COLS * (SIZE + 4))] * 4
            w, h = COLS * (SIZE + 4), len(out_rows)
            rl = (w * 3 + 3) // 4 * 4
            body = b"".join(l + b"\0" * (rl - len(l)) for l in out_rows[::-1])
            bmp = os.path.join(td, "sheet.bmp")
            Path(bmp).write_bytes(b"BM" + struct.pack("<IHHI", 54 + len(body), 0, 0, 54) +
                                  struct.pack("<IiiHHIIiiII", 40, w, h, 1, 24, 0, len(body), 2835, 2835, 0, 0) + body)
            out = f"{prefix}{k // per_sheet}.png"
            subprocess.run(["sips", "-s", "format", "png", bmp, "--out", out], check=True, capture_output=True)
            print(out)


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2], *(int(a) for a in sys.argv[3:4]))
