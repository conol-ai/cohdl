"""Body readings with a wider reach, for bodies the stock reach clips (`body-wide.json`).

Written 2026-11-10 for the 2026-11-08 probe board.  `body_axes.py` (in the
second re-verify record) reaches max(pad-centre distance) + 0.6 mm from the
footprint origin.  The SMC body (6.60-7.11 x 5.59-6.22 mm) has corners about
4.5 mm out, beyond the stock 4.05 mm, so the circle clips the stock rectangle.
This copy applies the same pixel test (not board, not copper, not magenta,
every channel below 110) and the same minimum-area rectangle, principal axis
and centroid, with reach = stock + EXTRA mm.  It gives the principal axis for
every body, with no elongation gate, and reports the elongation beside it.

It also reports the body's extents along and across the land axis, the median
colour of the body pixels, the farthest dark pixel, and the share of the dark
pixels that lie in the largest 8-connected component, as a check on what else
dark is inside the wider reach.

    python3 -I -B tvs_body_wide.py tools/frame_boards package/board.json \
        top.bmp bottom.bmp body-wide.json EXTRA REF...
"""
import json
import math
import sys

tools, board_json, top_bmp, bottom_bmp, out, extra = sys.argv[1:7]
refs = sys.argv[7:]
extra = float(extra)
sys.path.insert(0, tools)
import measure_preview as mp  # noqa: E402


def pct(values, q):
    values = sorted(values)
    return values[min(len(values) - 1, max(0, int(q * len(values))))]


def span(values):
    return pct(values, 0.995) - pct(values, 0.005)


def angle_mod(value, period):
    return (value + period / 2) % period - period / 2


def read(view, fp):
    pads = {p["num"]: (p["x"] - fp["x"], p["y"] - fp["y"]) for p in fp["pads"] if p["num"]}
    land = math.atan2(pads["2"][1] - pads["1"][1], pads["2"][0] - pads["1"][0])
    reach = max(math.hypot(*xy) for xy in pads.values()) + 0.6 + extra
    dark, cols, grid = [], [], {}
    for x, y, dx, dy in view.around(fp["x"], fp["y"], reach):
        px = view.px(x, y)
        if mp._board_px(*px) or mp._copper(*px) or mp.is_magenta(px):
            continue
        if max(px) < 110:
            dark.append((dx, dy))
            cols.append(tuple(px))
            grid[(x, y)] = (dx, dy)
    # largest 8-connected dark component
    seen, biggest = set(), 0
    for start in grid:
        if start in seen:
            continue
        seen.add(start)
        stack, n = [start], 0
        while stack:
            q = stack.pop()
            n += 1
            for nb in ((q[0] + i, q[1] + j) for i in (-1, 0, 1) for j in (-1, 0, 1)):
                if nb in grid and nb not in seen:
                    seen.add(nb)
                    stack.append(nb)
        biggest = max(biggest, n)
    mx = sum(p[0] for p in dark) / len(dark)
    my = sum(p[1] for p in dark) / len(dark)
    best = None
    for step in range(1800):
        t = math.radians(step / 20)
        c, s = math.cos(t), math.sin(t)
        u = span([p[0] * c + p[1] * s for p in dark])
        v = span([-p[0] * s + p[1] * c for p in dark])
        if best is None or u * v < best[0]:
            best = (u * v, t, u, v)
    # extents along / across the LAND axis (not the fitted rectangle)
    c, s = math.cos(land), math.sin(land)
    along = [p[0] * c + p[1] * s for p in dark]
    across = [-p[0] * s + p[1] * c for p in dark]
    sxx = sum((p[0] - mx) ** 2 for p in dark)
    syy = sum((p[1] - my) ** 2 for p in dark)
    sxy = sum((p[0] - mx) * (p[1] - my) for p in dark)
    axis = 0.5 * math.atan2(2 * sxy, sxx - syy)
    med = [sorted(c[i] for c in cols)[len(cols) // 2] for i in range(3)]
    return {
        "side": "bottom" if fp["bottom"] else "top", "kicad_angle": fp["angle"], "fpid": fp["fpid"],
        "reach_mm": round(reach, 2),
        "max_dark_radius_mm": round(max(math.hypot(*p) for p in dark), 2),
        "rect_mm": sorted([round(best[2], 2), round(best[3], 2)], reverse=True),
        "rect_edges_vs_land_deg_mod90": round(angle_mod(math.degrees(best[1] - land), 90), 1),
        "extent_along_land_mm": round(span(along), 2),
        "extent_across_land_mm": round(span(across), 2),
        "long_side_is_along_land": span(along) > span(across),
        "principal_axis_vs_land_deg_mod180": round(angle_mod(math.degrees(axis - land), 180), 1),
        "elongation": round(max(best[2], best[3]) / min(best[2], best[3]), 3),
        "centre_offset_mm": round(math.hypot(mx, my), 3),
        "centre_offset_along_across_mm": [round(mx * c + my * s, 3), round(-mx * s + my * c, 3)],
        "box_midpoint_offset_along_across_mm": [
            round((pct(along, 0.995) + pct(along, 0.005)) / 2, 3),
            round((pct(across, 0.995) + pct(across, 0.005)) / 2, 3)],
        "dark_px": len(dark), "dark_area_mm2": round(len(dark) / (view.sx * view.sy), 2),
        "largest_component_share": round(biggest / len(dark), 4),
        "median_body_bgr": med,
        "offset_limit_mm": round(mp.chip_offset_limit(math.dist(pads["1"], pads["2"]) / 2), 3),
        "angle_tol_deg": mp.CHIP_ANGLE_TOL,
    }


with open(board_json) as f:
    doc = json.load(f)
outline = mp.outline(doc)
views = {False: mp.View(top_bmp, outline, False), True: mp.View(bottom_bmp, outline, True)}
rows = {fp["ref"]: read(views[fp["bottom"]], fp) for fp in doc["fps"] if fp["ref"] in refs}
with open(out, "w") as f:
    json.dump({"extra_reach_mm": extra, "rows": {r: rows[r] for r in refs}}, f, indent=1)
    f.write("\n")
