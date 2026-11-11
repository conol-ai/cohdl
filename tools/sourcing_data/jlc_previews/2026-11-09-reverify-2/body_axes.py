"""Body outline readings for the re-verify boards' drawn IC and diode bodies.

Written 2026-11-10, when the rotations read on these boards became bindings.
It reads the same exports as `measure_preview.py` and uses its outline
calibration and colour tests, so run it with `tools/frame_boards` on the path:

    python3 -I body_axes.py tools/frame_boards board.json top.bmp bottom.bmp REF...

A body pixel is any pixel within reach of the footprint origin that is darker
than 110 in every channel and is not board, bare copper or a magenta mark.
For each REF it prints, in board millimetres:

- `rect_mm`: the minimum-area rectangle around the body (0.5th to 99.5th
  percentile along each side), longer side first;
- `rect_edges_vs_land_deg_mod90`: that rectangle's edges against the land
  axis named in AXIS, mod 90;
- `long_axis_vs_land_deg_mod180`: for an elongated body, its principal axis
  against the same land axis, mod 180;
- `centre_offset_mm`: the body's centroid from the footprint origin;
- `dimple_nearest_pads_mm` (XSON-8 only): the two pads nearest the small,
  compact white cluster farthest from the body centre (the model's pin 1
  dimple; the part text is the other white).
"""
import json
import math
import sys

sys.path.insert(0, sys.argv[1])
import measure_preview as mp  # noqa: E402

# The land axis each body is read against: from the first pad to the second.
# XSON-8: pad 1 to pad 8, across the two pad rows (the body's 3.0 mm side).
# QFN-14: pad 2 to pad 6, down the left row (square body, read mod 90).
# Two-terminal diodes: pad 1 (cathode) to pad 2.
AXIS = {
    "flash-FP_W25Q32RV_XSON8_2X3": ("1", "8"),
    "contrib_level_shifter-QFN14N50P350X350_1EP200X150": ("2", "6"),
    "sod-SOD2613X110N": ("1", "2"),
    "sod-SOD1608X70N": ("1", "2"),
    "do214-DIOM5026X241N": ("1", "2"),
}
DIMPLE = {"flash-FP_W25Q32RV_XSON8_2X3"}


def pct(values, q):
    values = sorted(values)
    return values[min(len(values) - 1, max(0, int(q * len(values))))]


def span(values):
    return pct(values, 0.995) - pct(values, 0.005)


def angle_mod(value, period):
    return (value + period / 2) % period - period / 2


def read(view, fp):
    pads = {p["num"]: (p["x"] - fp["x"], p["y"] - fp["y"]) for p in fp["pads"] if p["num"]}
    a, b = AXIS[fp["fpid"]]
    land = math.atan2(pads[b][1] - pads[a][1], pads[b][0] - pads[a][0])
    reach = max(math.hypot(*xy) for xy in pads.values()) + 0.6
    dark, white = [], []
    for x, y, dx, dy in view.around(fp["x"], fp["y"], reach):
        px = view.px(x, y)
        if mp._board_px(*px) or mp._copper(*px) or mp.is_magenta(px):
            continue
        if max(px) < 110:
            dark.append((dx, dy))
        elif min(px) > 200:
            white.append((x, y, dx, dy))
    mx = sum(p[0] for p in dark) / len(dark)
    my = sum(p[1] for p in dark) / len(dark)
    best = None
    for step in range(1800):  # 0.05 degree steps over a quarter turn
        t = math.radians(step / 20)
        c, s = math.cos(t), math.sin(t)
        u = span([p[0] * c + p[1] * s for p in dark])
        v = span([-p[0] * s + p[1] * c for p in dark])
        if best is None or u * v < best[0]:
            best = (u * v, t, u, v)
    row = {"side": "bottom" if fp["bottom"] else "top", "kicad_angle": fp["angle"],
           "rect_mm": sorted([round(best[2], 2), round(best[3], 2)], reverse=True),
           "rect_edges_vs_land_deg_mod90": round(angle_mod(math.degrees(best[1] - land), 90), 1),
           "centre_offset_mm": round(math.hypot(mx, my), 3)}
    if max(best[2], best[3]) > 1.2 * min(best[2], best[3]):
        sxx = sum((p[0] - mx) ** 2 for p in dark)
        syy = sum((p[1] - my) ** 2 for p in dark)
        sxy = sum((p[0] - mx) * (p[1] - my) for p in dark)
        axis = 0.5 * math.atan2(2 * sxy, sxx - syy)
        row["long_axis_vs_land_deg_mod180"] = round(angle_mod(math.degrees(axis - land), 180), 1)
    if fp["fpid"] in DIMPLE:
        long_side = max(best[2], best[3])
        hits = {(x, y): (dx, dy) for x, y, dx, dy in white if math.hypot(dx - mx, dy - my) < 0.5 * long_side}
        seen, clusters = set(), []
        for start in hits:
            if start in seen:
                continue
            seen.add(start)
            stack, pts = [start], []
            while stack:
                q = stack.pop()
                pts.append(hits[q])
                for n in ((q[0] + i, q[1] + j) for i in (-1, 0, 1) for j in (-1, 0, 1)):
                    if n in hits and n not in seen:
                        seen.add(n)
                        stack.append(n)
            xs, ys = [p[0] for p in pts], [p[1] for p in pts]
            if len(pts) >= 4 and max(max(xs) - min(xs), max(ys) - min(ys)) < 0.3:
                clusters.append((sum(xs) / len(xs), sum(ys) / len(ys)))
        if clusters:
            d = max(clusters, key=lambda c: math.hypot(c[0] - mx, c[1] - my))
            near = sorted((math.hypot(d[0] - xy[0], d[1] - xy[1]), k) for k, xy in pads.items())
            row["dimple_nearest_pads_mm"] = [[k, round(dist, 2)] for dist, k in near[:2]]
    return row


def main(_tools, board_json, top_bmp, bottom_bmp, *refs):
    with open(board_json) as f:
        doc = json.load(f)
    outline = mp.outline(doc)
    views = {False: mp.View(top_bmp, outline, False), True: mp.View(bottom_bmp, outline, True)}
    rows = {fp["ref"]: read(views[fp["bottom"]], fp) for fp in doc["fps"] if fp["ref"] in refs}
    json.dump({ref: rows[ref] for ref in refs}, sys.stdout, indent=1)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main(*sys.argv[1:])
