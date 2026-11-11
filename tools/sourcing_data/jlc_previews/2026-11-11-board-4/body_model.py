"""Body readings of the drawn model alone, for this record's two-terminal parts.

Written 2026-11-11 for the preview of board 4. `measure_preview.py`'s `body`
takes every pixel within 1.5 pad half-spacings of the origin that is not
board, bare copper or marker black. On this board that window also holds
part of the white silkscreen reference text, which sits beside each part and
turns with it, so the reading is of the model and the text together. This
tool reads the same exports with the same outline calibration and the same
window, but keeps only the pixels connected to the part's origin: the drawn
model (body, terminals and the red designator the preview prints over it).
Board green and bare copper separate the model from the silkscreen.

    python3 -I body_model.py tools/frame_boards board.json top.bmp bottom.bmp OUT.json

It reads every two-pad footprint of the library's `passive` package: the
`CHIP_*` lands (`measure_preview.is_chip`) and the two inductor lands. Each
row has the keys of a `measure_preview.py measure` row, so
`calibration_fit.py` reads OUT.json as it reads measurements.json:

- `body.delta_mod_180`: the model's long axis against pad 1 to pad 2;
- `body.offset_xy_mm`, `body.offset_mm`: its centroid from the footprint
  origin, in the board frame (x right, y down, as the view shows it);
- `body.size_mm`: its extent along and across the pads (0.5th to 99.5th
  percentile);
- `body.elongation`, `body.offset_limit_mm`: as in `measure_preview.py`;
- `body.fill`: the model's area over the pad half-spacing squared;
- `body.core_offset_xy_mm`: the centroid of the body's own colour only
  (`measure_preview._core`: a resistor's grey, an MLCC's tan, the red
  designator), connected or not; absent when the model has none.
- `body.window_px`, `body.model_px`: the non-board, non-copper, non-black
  pixels in the window, and how many of them are the model. The difference
  is what `measure_preview.py` also read: silkscreen, mostly.

A model that is not drawn has `"drawn": false` and no body. These are
measurements for this record's README, not the W3 rule's reading: the rule
reads `measurements.json`.
"""
import json
import math
import sys

try:
    import measure_preview as mp
except ImportError:  # run as a script: TOOLS is the first argument
    sys.path.insert(0, sys.argv[1])
    import measure_preview as mp

LANDS = ("passive-CHIP_", "passive-FP_L_")
SEED_MM = 0.15


def pct(values, q):
    values = sorted(values)
    return values[min(len(values) - 1, max(0, int(q * len(values))))]


def read(view, fp):
    (ax, ay), (bx, by) = [(p["x"], p["y"]) for p in fp["pads"] if p["num"]]
    half = math.dist((ax, ay), (bx, by)) / 2
    ux, uy = (bx - ax) / (2 * half), (by - ay) / (2 * half)
    window = {}
    for x, y, dx, dy in view.around(fp["x"], fp["y"], 1.5 * half):
        b, g, r = view.px(x, y)
        if not (mp._board_px(b, g, r) or mp._copper(b, g, r)):
            window[(x, y)] = (dx, dy, max(b, g, r) >= 30, mp._core(b, g, r))
    # The model: what is connected to the origin, through marker black too (a body's outline).
    stack = [p for p, (dx, dy, _, _) in window.items() if math.hypot(dx, dy) <= SEED_MM]
    seen = set(stack)
    while stack:
        x, y = stack.pop()
        for n in ((x + i, y + j) for i in (-1, 0, 1) for j in (-1, 0, 1)):
            if n in window and n not in seen:
                seen.add(n)
                stack.append(n)
    pts = [window[p][:2] for p in seen if window[p][2]]
    out = {"window_px": sum(v[2] for v in window.values()), "model_px": len(pts),
           "fill": round(len(pts) / (view.sx * view.sy) / half ** 2, 3)}
    if len(pts) < 5:
        return out
    mx, my = sum(p[0] for p in pts) / len(pts), sum(p[1] for p in pts) / len(pts)
    sxx = sum((p[0] - mx) ** 2 for p in pts)
    syy = sum((p[1] - my) ** 2 for p in pts)
    sxy = sum((p[0] - mx) * (p[1] - my) for p in pts)
    long_ = (sxx + syy) / 2 + math.sqrt(((sxx - syy) / 2) ** 2 + sxy ** 2)
    short = max(sxx + syy - long_, 1e-12)
    axis = math.degrees(0.5 * math.atan2(2 * sxy, sxx - syy))
    along = [p[0] * ux + p[1] * uy for p in pts]
    across = [-p[0] * uy + p[1] * ux for p in pts]
    out.update({"delta_mod_180": round((axis - math.degrees(math.atan2(uy, ux)) + 90) % 180 - 90, 1),
                "offset_mm": round(math.hypot(mx, my), 3), "offset_xy_mm": [round(mx, 3), round(my, 3)],
                "offset_limit_mm": round(mp.chip_offset_limit(half), 3),
                "size_mm": [round(pct(along, 0.995) - pct(along, 0.005), 2),
                            round(pct(across, 0.995) - pct(across, 0.005), 2)],
                "elongation": round(math.sqrt(long_ / short), 2)})
    core = [v[:2] for v in window.values() if v[3]]
    if core:
        out["core_offset_xy_mm"] = [round(sum(p[0] for p in core) / len(core), 3),
                                    round(sum(p[1] for p in core) / len(core), 3)]
    return out


def main(board_json, top_bmp, bottom_bmp, out):
    with open(board_json) as f:
        doc = json.load(f)
    views = {False: mp.View(top_bmp, mp.outline(doc), False), True: mp.View(bottom_bmp, mp.outline(doc), True)}
    rows = []
    for fp in sorted(doc["fps"], key=lambda f: (f["ref"].rstrip("0123456789"), int(f["ref"].lstrip("ABCDEFGHIJKLMNOPQRSTUVWXYZ") or 0))):
        if not fp["fpid"].startswith(LANDS) or len([p for p in fp["pads"] if p["num"]]) != 2:
            continue
        row = {"ref": fp["ref"], "side": "bottom" if fp["bottom"] else "top", "kicad_angle": fp["angle"]}
        if not mp.drawn(views[fp["bottom"]], fp):
            row["drawn"] = False
        else:
            row["body"] = read(views[fp["bottom"]], fp)
        rows.append(row)
    with open(out, "w") as f:
        json.dump(rows, f, indent=1)
        f.write("\n")


if __name__ == "__main__":
    if len(sys.argv) != 6:
        sys.exit(__doc__)
    main(*sys.argv[2:])
