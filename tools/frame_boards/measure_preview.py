"""Measure pin 1 on an assembler placement preview against the board.

The preview marks each part's pin 1 (and a connector's locating pegs) with a
magenta dot.  This finds those dots in a screenshot of the whole board, maps
them back to board millimetres through the board outline, and compares each
with the footprint's own pad 1:

    # 1. footprint and pad positions, with KiCad's Python
    <kicad python3> -I measure_preview.py board BOARD.kicad_pcb board.json
    # 2. screenshots to uncompressed BMP (macOS)
    sips -s format bmp top.jpg --out top.bmp
    # 3. measure, with any Python
    python3 -I measure_preview.py measure board.json top.bmp bottom.bmp out.json

The bottom screenshot is the preview's view from below, mirrored left to
right.  `delta_ccw_from_top` is how far the preview's pin 1 sits
counter-clockwise from ours, seen from the top.  The supplier map's local
`rotate` that cancels it is `(360 - delta) mod 360` rounded to 90 degrees;
seen from the top, the bottom side's delta is the mirror image of the top's.
Two-terminal parts carry polarity signs instead of a dot and are read by eye.
"""
import json
import math
import struct
import sys


def board(path, out):
    import pcbnew

    b = pcbnew.LoadBoard(path)
    box = b.GetBoardEdgesBoundingBox()
    doc = {"bbox": [pcbnew.ToMM(v) for v in (box.GetX(), box.GetY(), box.GetRight(), box.GetBottom())],
           "fps": []}
    for fp in b.GetFootprints():
        doc["fps"].append({
            "ref": fp.GetReference(), "fpid": fp.GetFPIDAsString(), "bottom": fp.IsFlipped(),
            "x": pcbnew.ToMM(fp.GetPosition().x), "y": pcbnew.ToMM(fp.GetPosition().y),
            "angle": fp.GetOrientationDegrees(),
            "pads": [{"num": p.GetNumber(), "x": pcbnew.ToMM(p.GetPosition().x),
                      "y": pcbnew.ToMM(p.GetPosition().y)} for p in fp.Pads()],
        })
    with open(out, "w") as f:
        json.dump(doc, f, indent=1)


def load_bmp(path):
    with open(path, "rb") as f:
        d = f.read()
    off = struct.unpack_from("<I", d, 10)[0]
    w, h = struct.unpack_from("<ii", d, 18)
    bpp = struct.unpack_from("<H", d, 28)[0] // 8
    stride = (w * bpp + 3) & ~3
    up, h = h > 0, abs(h)
    rows = []
    for y in range(h):
        r = h - 1 - y if up else y
        rows.append(d[off + r * stride: off + r * stride + w * bpp])
    return w, h, bpp, rows


def is_green(px):  # BGR
    return px[1] > 80 and px[1] > px[2] + 40 and px[1] > px[0] + 30


def is_magenta(px):
    return px[2] > 170 and px[0] > 170 and px[1] < 110


def dots(path, outline, bottom):
    """Magenta clusters in board millimetres."""
    x0, y0, x1, y1 = outline
    w, h, bpp, rows = load_bmp(path)
    mid = rows[h // 2]
    xs = [x for x in range(w) if is_green(mid[x * bpp:x * bpp + 3])]
    ys = [y for y in range(h) if is_green(rows[y][(w // 2) * bpp:(w // 2) * bpp + 3])]
    left, right, top, low = min(xs), max(xs) + 1, min(ys), max(ys) + 1
    sx, sy = (right - left) / (x1 - x0), (low - top) / (y1 - y0)
    hits = {(x, y) for y in range(top, low) for x in range(left, right)
            if is_magenta(rows[y][x * bpp:x * bpp + 3])}
    found, seen = [], set()
    for start in hits:
        if start in seen:
            continue
        seen.add(start)
        stack, pts = [start], []
        while stack:
            p = stack.pop()
            pts.append(p)
            for n in ((p[0] + dx, p[1] + dy) for dx in (-1, 0, 1) for dy in (-1, 0, 1)):
                if n in hits and n not in seen:
                    seen.add(n)
                    stack.append(n)
        if len(pts) < 6:
            continue
        cx = sum(p[0] for p in pts) / len(pts)
        cy = sum(p[1] for p in pts) / len(pts)
        found.append((x0 + ((right - cx) if bottom else (cx - left)) / sx, y0 + (cy - top) / sy, len(pts)))
    return found


def measure(board_json, top_bmp, bottom_bmp, out):
    with open(board_json) as f:
        doc = json.load(f)
    half = 0.05  # half the Edge.Cuts stroke
    x0, y0, x1, y1 = doc["bbox"]
    outline = (x0 + half, y0 + half, x1 - half, y1 - half)
    found = {False: dots(top_bmp, outline, False), True: dots(bottom_bmp, outline, True)}

    def angle(dx, dy):  # seen from the top, +y up, counter-clockwise
        return math.degrees(math.atan2(-dy, dx)) % 360

    rows = []
    for fp in sorted(doc["fps"], key=lambda f: (f["ref"].rstrip("0123456789"), int(f["ref"].lstrip("ABCDEFGHIJKLMNOPQRSTUVWXYZ") or 0))):
        pads = [p for p in fp["pads"] if p["num"]]
        reach = max(math.hypot(p["x"] - fp["x"], p["y"] - fp["y"]) for p in pads) + 1.2
        near = [d for d in found[fp["bottom"]] if math.hypot(d[0] - fp["x"], d[1] - fp["y"]) <= reach]
        pad1 = next((p for p in pads if p["num"] == "1"), None)
        row = {"ref": fp["ref"], "side": "bottom" if fp["bottom"] else "top", "kicad_angle": fp["angle"],
               "pad1": [round(pad1["x"] - fp["x"], 3), round(pad1["y"] - fp["y"], 3)] if pad1 else None,
               "dots": [[round(d[0] - fp["x"], 2), round(d[1] - fp["y"], 2)] for d in near]}
        if pad1 and len(near) == 1:
            d = near[0]
            row["delta_ccw_from_top"] = round((angle(d[0] - fp["x"], d[1] - fp["y"])
                                               - angle(pad1["x"] - fp["x"], pad1["y"] - fp["y"])) % 360, 1)
        rows.append(row)
    with open(out, "w") as f:
        json.dump(rows, f, indent=1)
        f.write("\n")


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "board":
        board(sys.argv[2], sys.argv[3])
    elif len(sys.argv) == 6 and sys.argv[1] == "measure":
        measure(*sys.argv[2:])
    else:
        sys.exit(__doc__)
