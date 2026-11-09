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
    #    (bottom.bmp may be - on a board with no bottom parts)
    # 4. a connector's locating pegs against the board's own holes
    python3 -I measure_preview.py pegs board.json top.bmp bottom.bmp REF_PREFIX

The bottom screenshot is the preview's view from below, mirrored left to
right.  `delta_ccw_from_top` is how far the preview's pin 1 sits
counter-clockwise from ours, seen from the top.  The supplier map's local
`rotate` that cancels it is `(360 - delta) mod 360` rounded to 90 degrees;
seen from the top, the bottom side's delta is the mirror image of the top's.
Two-terminal polarised parts carry polarity signs instead of a dot and are
read by eye. Non-polar chip parts (`is_chip`) carry neither: their row gives
the drawn body's long axis against the pads' axis, mod 180, and its centring
(`body`). A part whose model the preview did not draw shows `"drawn": false`.

`pegs` compares a connector's peg dots with its non-plated holes as the other
view shows them through the board.  Both go through the same outline mapping,
so the comparison does not depend on how well the outline calibrates.
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


def is_hole(px):
    return max(px[0], px[1], px[2]) < 60


class View:
    """An export, calibrated to board millimetres through the board outline.

    The green board spans the outline. A bottom export is the view from below,
    mirrored left to right, so its x runs from the right edge.
    """

    def __init__(self, path, outline, bottom):
        self.outline, self.bottom = outline, bottom
        x0, y0, x1, y1 = outline
        w, h, self.bpp, self.rows = load_bmp(path)
        bpp, rows = self.bpp, self.rows
        mid = rows[h // 2]
        xs = [x for x in range(w) if is_green(mid[x * bpp:x * bpp + 3])]
        ys = [y for y in range(h) if is_green(rows[y][(w // 2) * bpp:(w // 2) * bpp + 3])]
        self.left, self.right, self.top, self.low = min(xs), max(xs) + 1, min(ys), max(ys) + 1
        self.sx, self.sy = (self.right - self.left) / (x1 - x0), (self.low - self.top) / (y1 - y0)

    def px(self, x, y):
        """The pixel (BGR) under the image point x, y."""
        return self.rows[y][x * self.bpp:x * self.bpp + 3]

    def to_board(self, x, y):
        x0, y0 = self.outline[:2]
        return (x0 + ((self.right - x) if self.bottom else (x - self.left)) / self.sx, y0 + (y - self.top) / self.sy)

    def around(self, cx, cy, r):
        """The image points within R mm of the board point cx, cy, with their board offsets."""
        x0, y0 = self.outline[:2]
        ix = (self.right - (cx - x0) * self.sx) if self.bottom else (self.left + (cx - x0) * self.sx)
        iy = self.top + (cy - y0) * self.sy
        for y in range(int(iy - r * self.sy) - 1, int(iy + r * self.sy) + 2):
            for x in range(int(ix - r * self.sx) - 1, int(ix + r * self.sx) + 2):
                bx, by = self.to_board(x + 0.5, y + 0.5)
                if math.hypot(bx - cx, by - cy) <= r:
                    yield x, y, bx - cx, by - cy


def dots(path, outline, bottom, pred=is_magenta, size=(6, None)):
    """Clusters of `pred` pixels, magenta by default, in board millimetres."""
    x0, y0, x1, y1 = outline
    view = path if isinstance(path, View) else View(path, outline, bottom)
    left, right, top, low, sx, sy = view.left, view.right, view.top, view.low, view.sx, view.sy
    hits = {(x, y) for y in range(top, low) for x in range(left, right)
            if pred(view.px(x, y))}
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
        if len(pts) < size[0] or (size[1] and len(pts) > size[1](sx)):
            continue
        cx = sum(p[0] for p in pts) / len(pts)
        cy = sum(p[1] for p in pts) / len(pts)
        found.append((x0 + ((right - cx) if bottom else (cx - left)) / sx, y0 + (cy - top) / sy, len(pts)))
    return found


def outline(doc):
    """The board outline's centre line, from a `board` document's bounding box."""
    half = 0.05  # half the Edge.Cuts stroke
    x0, y0, x1, y1 = doc["bbox"]
    return (x0 + half, y0 + half, x1 - half, y1 - half)


def reach(fp):
    """How far from a footprint's origin its pin 1 mark can sit: its farthest pad, plus a margin."""
    return max(math.hypot(p["x"] - fp["x"], p["y"] - fp["y"]) for p in fp["pads"] if p["num"]) + 1.2


def is_chip(fp):
    """A non-polar two-terminal chip part: the library's passive CHIP_* lands, two pads.

    The preview gives these no pin 1 mark; their body is read instead (`body`).
    """
    return fp.get("fpid", "").startswith("passive-CHIP_") and len([p for p in fp["pads"] if p["num"]]) == 2


def drawn(view, fp):
    """False when the preview drew no model: the checkered centroid marker shows at the origin.

    A drawn model covers the marker. The marker is half black, half white; a
    drawn part's centre is never both.
    """
    px = [view.px(x, y) for x, y, dx, dy in view.around(fp["x"], fp["y"], 0.3 * math.sqrt(2))
          if abs(dx) <= 0.3 and abs(dy) <= 0.3]
    black = sum(max(p) < 40 for p in px)
    white = sum(min(p) > 215 for p in px)
    return not (black >= 0.3 * len(px) and white >= 0.3 * len(px))


def _board_px(b, g, r):
    return g > r + 15 and g >= b


def _copper(b, g, r):
    return r > 140 and r > g + 30 and b > 80 and abs(g - b) < 45


def _core(b, g, r):  # a chip body's own colour: a resistor's grey, an MLCC's tan, the red reference
    grey = max(r, g, b) - min(r, g, b) < 30 and 35 < max(r, g, b) < 130
    tan = r > 120 and 70 < g < 180 and b < 70
    red = r > 180 and g < 60 and b < 60
    return grey or tan or red


# A chip body passes when its long axis is within this of the pads' axis (mod 180),
# its centre within the offset limit of the footprint origin, and it is clearly
# longer than wide. Single readings scatter by a few degrees.
CHIP_ANGLE_TOL = 10.0
CHIP_ELONGATION_MIN = 1.1
CHIP_FILL_MIN = 0.1


def chip_offset_limit(half):
    """The centring tolerance in mm for a chip whose pad centres sit HALF mm from its origin."""
    return min(0.25, 0.25 * half)


def body(view, fp):
    """The drawn body of a chip part: its long axis against the pads' axis, mod 180, and its centring.

    The body is every pixel within 1.5 pad half-spacings of the origin that is
    neither board nor bare copper nor marker black. `fill` is the area of body
    colour over the half-spacing squared; a model that is not drawn leaves
    almost none.
    """
    (ax, ay), (bx, by) = [(p["x"], p["y"]) for p in fp["pads"] if p["num"]]
    half = math.dist((ax, ay), (bx, by)) / 2
    pts, core = [], 0
    for x, y, dx, dy in view.around(fp["x"], fp["y"], 1.5 * half):
        b, g, r = view.px(x, y)
        if _board_px(b, g, r) or _copper(b, g, r) or max(b, g, r) < 30:
            continue
        pts.append((dx, dy))
        core += _core(b, g, r)
    fill = core / (view.sx * view.sy) / half ** 2
    if len(pts) < 5:
        return {"fill": round(fill, 3)}
    mx, my = sum(p[0] for p in pts) / len(pts), sum(p[1] for p in pts) / len(pts)
    sxx = sum((p[0] - mx) ** 2 for p in pts)
    syy = sum((p[1] - my) ** 2 for p in pts)
    sxy = sum((p[0] - mx) * (p[1] - my) for p in pts)
    long_ = (sxx + syy) / 2 + math.sqrt(((sxx - syy) / 2) ** 2 + sxy ** 2)
    short = max(sxx + syy - long_, 1e-12)
    axis = math.degrees(0.5 * math.atan2(2 * sxy, sxx - syy))
    delta = (axis - math.degrees(math.atan2(by - ay, bx - ax)) + 90) % 180 - 90
    return {"delta_mod_180": round(delta, 1), "offset_mm": round(math.hypot(mx, my), 3),
            "offset_limit_mm": round(chip_offset_limit(half), 3),
            "elongation": round(math.sqrt(long_ / short), 2), "fill": round(fill, 3)}


def body_ok(reading):
    return ("delta_mod_180" in reading and reading["fill"] >= CHIP_FILL_MIN
            and abs(reading["delta_mod_180"]) <= CHIP_ANGLE_TOL
            and reading["offset_mm"] <= reading["offset_limit_mm"]
            and reading["elongation"] >= CHIP_ELONGATION_MIN)


def measure(board_json, top_bmp, bottom_bmp, out):
    with open(board_json) as f:
        doc = json.load(f)
    # A board with no bottom parts may pass "-" for the bottom view, which it does not export.
    shots = {False: top_bmp, True: bottom_bmp}
    if bottom_bmp == "-" and any(fp["bottom"] for fp in doc["fps"]):
        sys.exit("measure_preview.py: the board has bottom parts, so it needs the bottom view")
    views = {bottom: View(path, outline(doc), bottom) for bottom, path in shots.items() if path != "-"}
    found = {bottom: dots(view, outline(doc), bottom) for bottom, view in views.items()}

    def angle(dx, dy):  # seen from the top, +y up, counter-clockwise
        return math.degrees(math.atan2(-dy, dx)) % 360

    rows = []
    for fp in sorted(doc["fps"], key=lambda f: (f["ref"].rstrip("0123456789"), int(f["ref"].lstrip("ABCDEFGHIJKLMNOPQRSTUVWXYZ") or 0))):
        pads = [p for p in fp["pads"] if p["num"]]
        near = [d for d in found[fp["bottom"]] if math.hypot(d[0] - fp["x"], d[1] - fp["y"]) <= reach(fp)]
        # Pin 1 is ball A1 on a BGA.
        pad1 = next((p for p in pads if p["num"] in ("1", "A1")), None)
        row = {"ref": fp["ref"], "side": "bottom" if fp["bottom"] else "top", "kicad_angle": fp["angle"],
               "pad1": [round(pad1["x"] - fp["x"], 3), round(pad1["y"] - fp["y"], 3)] if pad1 else None,
               "dots": [[round(d[0] - fp["x"], 2), round(d[1] - fp["y"], 2)] for d in near]}
        # Keys only where they apply, so earlier records still reproduce byte for byte.
        if not drawn(views[fp["bottom"]], fp):
            row["drawn"] = False
        elif is_chip(fp):
            row["body"] = body(views[fp["bottom"]], fp)
            row["body_ok"] = body_ok(row["body"])
        if pad1 and len(near) == 1 and not is_chip(fp):
            d = near[0]
            row["delta_ccw_from_top"] = round((angle(d[0] - fp["x"], d[1] - fp["y"])
                                               - angle(pad1["x"] - fp["x"], pad1["y"] - fp["y"])) % 360, 1)
        rows.append(row)
    with open(out, "w") as f:
        json.dump(rows, f, indent=1)
        f.write("\n")


def pegs(board_json, top_bmp, bottom_bmp, prefix):
    """Each footprint's peg dots and, from the other view, its holes."""
    with open(board_json) as f:
        doc = json.load(f)
    hole = (10, lambda sx: int((0.9 * sx) ** 2))  # smaller than a 0.9 mm square
    views = {bottom: (dots(path, outline(doc), bottom), dots(path, outline(doc), bottom, is_hole, hole))
             for bottom, path in ((False, top_bmp), (True, bottom_bmp))}
    for fp in doc["fps"]:
        if not fp["ref"].startswith(prefix):
            continue
        def near(found):
            return sorted((round(d[0] - fp["x"], 2), round(d[1] - fp["y"], 2)) for d in found
                          if math.hypot(d[0] - fp["x"], d[1] - fp["y"]) < 6)
        print(json.dumps({"ref": fp["ref"], "side": "bottom" if fp["bottom"] else "top",
                          "kicad_angle": fp["angle"], "dots": near(views[fp["bottom"]][0]),
                          "holes": near(views[not fp["bottom"]][1])}))


if __name__ == "__main__":
    if len(sys.argv) == 4 and sys.argv[1] == "board":
        board(sys.argv[2], sys.argv[3])
    elif len(sys.argv) == 6 and sys.argv[1] == "measure":
        measure(*sys.argv[2:])
    elif len(sys.argv) == 6 and sys.argv[1] == "pegs":
        pegs(*sys.argv[2:])
    else:
        sys.exit(__doc__)
