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
    #    `measure --method 1 ...` reads chip bodies as the records made before
    #    method 2 did (BODY_METHODS); those records reproduce only that way
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

A chip body is read by a numbered method (`BODY_METHODS`). Method 2, the
current one, reads the drawn model alone and writes `"method": 2` into each
`body`. Method 1 read the model together with whatever silkscreen lay in its
window, and wrote no method: a `body` without one is a method 1 reading.

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


def _axis(pts, pads):
    """Centroid, long axis against the pads' axis (mod 180) and elongation of PTS,
    board-frame offsets from the footprint origin. PADS are pad 1 and pad 2."""
    (ax, ay), (bx, by) = pads
    mx, my = sum(p[0] for p in pts) / len(pts), sum(p[1] for p in pts) / len(pts)
    sxx = sum((p[0] - mx) ** 2 for p in pts)
    syy = sum((p[1] - my) ** 2 for p in pts)
    sxy = sum((p[0] - mx) * (p[1] - my) for p in pts)
    long_ = (sxx + syy) / 2 + math.sqrt(((sxx - syy) / 2) ** 2 + sxy ** 2)
    short = max(sxx + syy - long_, 1e-12)
    axis = math.degrees(0.5 * math.atan2(2 * sxy, sxx - syy))
    delta = (axis - math.degrees(math.atan2(by - ay, bx - ax)) + 90) % 180 - 90
    return mx, my, delta, math.sqrt(long_ / short)


def body_all_pixels(view, fp):
    """Method 1: the long axis and centring of everything in a chip part's window.

    The body is every pixel within 1.5 pad half-spacings of the origin that is
    neither board nor bare copper nor marker black. `fill` is the area of body
    colour over the half-spacing squared; a model that is not drawn leaves
    almost none.

    That window also holds whatever silkscreen lies in it. A part's own white
    reference text sits beside it and turns with it, so this reads the model and
    part of its text together (jlc_previews/2026-11-11-board-4/README.md). Kept
    so that the records made with it still reproduce byte for byte.
    """
    pads = [(p["x"], p["y"]) for p in fp["pads"] if p["num"]]
    half = math.dist(*pads) / 2
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
    mx, my, delta, elongation = _axis(pts, pads)
    # offset_xy_mm is the same offset as a board-frame vector. W3 bounds it raw and
    # also after taking off the view's median, on the hypothesis that a shift every
    # body shares may be the export's (tools/sourcing/w3_rule.py, board_bias).
    return {"delta_mod_180": round(delta, 1), "offset_mm": round(math.hypot(mx, my), 3),
            "offset_xy_mm": [round(mx, 3), round(my, 3)],
            "offset_limit_mm": round(chip_offset_limit(half), 3),
            "elongation": round(elongation, 2), "fill": round(fill, 3)}


# Method 2 starts from the pixels this close to the footprint origin.
MODEL_SEED_MM = 0.15
_PLUS = ((1, 0), (-1, 0), (0, 1), (0, -1))
_RING = tuple((i, j) for i in (-1, 0, 1) for j in (-1, 0, 1) if i or j)


def _percentile(values, q):
    values = sorted(values)
    return values[min(len(values) - 1, max(0, int(q * len(values))))]


def _model_pixels(view, fp):
    """Method 2's pixels: {image point: board offset} of the model, and how many
    pixels method 1 would have read in the same window."""
    pads = [(p["x"], p["y"]) for p in fp["pads"] if p["num"]]
    half = math.dist(*pads) / 2
    kind = {}

    def copper(x, y):  # also asked just outside the window, for the opening
        if (x, y) not in kind:
            kind[(x, y)] = _copper(*view.px(x, y))
        return kind[(x, y)]

    seen_px = {}
    for x, y, dx, dy in view.around(fp["x"], fp["y"], 1.5 * half):
        b, g, r = view.px(x, y)
        if not _board_px(b, g, r):
            seen_px[(x, y)] = (dx, dy, max(b, g, r) >= 30 and not _copper(b, g, r))
    # Copper cores: the centre of a 3 x 3 block of copper colour. A pad is its cores grown by one.
    cores = {p for p in seen_px if copper(*p) and all(copper(p[0] + i, p[1] + j) for i, j in _RING)}
    pad = cores | {(x + i, y + j) for x, y in cores for i, j in _RING}
    window = {p: v for p, v in seen_px.items() if not (copper(*p) and p in pad)}
    counted = {p for p, (_, _, lit) in window.items() if lit or copper(*p)}
    walk = {p for p in window if all((p[0] + i, p[1] + j) in window for i, j in _PLUS)}
    stack = [p for p in walk if math.hypot(*window[p][:2]) <= MODEL_SEED_MM]
    model = set(stack)
    while stack:
        x, y = stack.pop()
        for i, j in _RING:
            n = (x + i, y + j)
            if n in walk and n not in model:
                model.add(n)
                stack.append(n)
    model |= {(x + i, y + j) for x, y in model for i, j in _RING if (x + i, y + j) in window}
    return ({p: window[p][:2] for p in sorted(model & counted)},
            sum(v[2] for v in seen_px.values()))


def body_model(view, fp):
    """Method 2: the long axis and centring of the drawn model alone.

    The same window as method 1 (1.5 pad half-spacings about the origin), but only
    what is joined to the origin counts:

    1. Board green is not the model. Neither is a pad's bare copper: a
       copper-coloured pixel that belongs to a copper area at least 3 px across
       (it survives a 3 x 3 opening of the copper-coloured pixels). A thinner run
       of copper colour is not a pad: it is the export's blend where the red
       designator meets a white terminal, inside the model, and stays.
    2. Of what is left, the model is the part joined to the origin by a path at
       least 3 px wide: the pixels whose four neighbours are also left, flooded
       from those within MODEL_SEED_MM of the origin, then grown back by one
       pixel. Reference text that only grazes the model (a bridge 1 or 2 px wide)
       is cut off; so is anything beside the model with board between them.
    3. Marker black joins but is not counted, as in method 1.

    Also given: `size_mm`, the model's extent along and across the pads (0.5th to
    99.5th percentile); `window_px`, what method 1 would have read, and
    `model_px`, how much of it is the model. `fill` is the model's area over the
    half-spacing squared.

    What this cannot tell apart: silkscreen that overlaps the model or touches it
    along 3 px or more is read as model, and so is any other drawn thing joined
    to it. A model the export draws in pieces with board between them is read as
    its piece at the origin only. The red designator printed over the body is
    part of the reading.
    """
    pads = [(p["x"], p["y"]) for p in fp["pads"] if p["num"]]
    half = math.dist(*pads) / 2
    (ax, ay), (bx, by) = pads
    ux, uy = (bx - ax) / (2 * half), (by - ay) / (2 * half)
    model, window_px = _model_pixels(view, fp)
    pts = list(model.values())
    out = {"method": 2, "window_px": window_px, "model_px": len(pts),
           "fill": round(len(pts) / (view.sx * view.sy) / half ** 2, 3)}
    if len(pts) < 5:
        return out
    mx, my, delta, elongation = _axis(pts, pads)
    along = [p[0] * ux + p[1] * uy for p in pts]
    across = [-p[0] * uy + p[1] * ux for p in pts]
    out.update({"delta_mod_180": round(delta, 1), "offset_mm": round(math.hypot(mx, my), 3),
                "offset_xy_mm": [round(mx, 3), round(my, 3)],
                "offset_limit_mm": round(chip_offset_limit(half), 3),
                "size_mm": [round(_percentile(along, 0.995) - _percentile(along, 0.005), 2),
                            round(_percentile(across, 0.995) - _percentile(across, 0.005), 2)],
                "elongation": round(elongation, 2)})
    return out


# How a chip body is read. A reading names its method (`body["method"]`); one
# without the key is method 1, the only one there was until 2026-11-11.
BODY_METHODS = {1: body_all_pixels, 2: body_model}
BODY_METHOD = 2


def body_method(reading):
    """The method a `body` reading was made with."""
    return reading.get("method", 1)


def body(view, fp, method=BODY_METHOD):
    """The drawn body of a chip part: its long axis against the pads' axis, mod 180,
    and its centring, read by METHOD (BODY_METHODS)."""
    return BODY_METHODS[method](view, fp)


def body_ok(reading):
    return ("delta_mod_180" in reading and reading["fill"] >= CHIP_FILL_MIN
            and abs(reading["delta_mod_180"]) <= CHIP_ANGLE_TOL
            and reading["offset_mm"] <= reading["offset_limit_mm"]
            and reading["elongation"] >= CHIP_ELONGATION_MIN)


def measure(board_json, top_bmp, bottom_bmp, out, method=BODY_METHOD):
    """Write OUT. METHOD is how chip bodies are read; 1 writes exactly what this tool
    wrote before it had methods."""
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
            row["body"] = body(views[fp["bottom"]], fp, method)
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
    elif len(sys.argv) == 8 and sys.argv[1:3] == ["measure", "--method"] and sys.argv[3] in ("1", "2"):
        measure(*sys.argv[4:], method=int(sys.argv[3]))
    elif len(sys.argv) == 6 and sys.argv[1] == "pegs":
        pegs(*sys.argv[2:])
    else:
        sys.exit(__doc__)
