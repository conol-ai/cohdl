"""Body readings for the 2026-11-08 probe board with the stock reach (`body.json`).

Written 2026-11-10.  It imports `read()` from the second re-verify record's
`body_axes.py` and does not change it.  The only adaptation is that the AXIS
table gains this board's two-terminal lands (pad 1 to pad 2).  Besides the
fields `body_axes.py` documents, each row carries the pixel counts by class
inside the same reach, and the output starts with each view's calibration.

    python3 -I -B tvs_body_axes.py tools/frame_boards \
        tools/sourcing_data/jlc_previews/2026-11-09-reverify-2 \
        package/board.json top.bmp bottom.bmp body.json REF...

`body_axes.py` gives `long_axis_vs_land_deg_mod180` only when the fitted
rectangle's long side is more than 1.2 times its short side, and it reaches
0.6 mm beyond the farthest pad centre.  The SMC body fails both (see
`tvs_body_wide.py`).
"""
import json
import math
import sys

tools, ba_dir, board_json, top_bmp, bottom_bmp, out = sys.argv[1:7]
refs = sys.argv[7:]
sys.argv = [sys.argv[0], tools]
sys.path.insert(0, ba_dir)
import body_axes as ba  # noqa: E402
mp = ba.mp

for fpid in ("do214-DIOM5336X265N", "do214-DIOM7959X261N", "sod-SODFL3818X120N",
             "sod-SOD3716X135N", "smaf-FP_DO221AC_SMAF"):
    ba.AXIS.setdefault(fpid, ("1", "2"))

with open(board_json) as f:
    doc = json.load(f)
for fp in doc["fps"]:
    if len([p for p in fp["pads"] if p["num"]]) == 2:
        ba.AXIS.setdefault(fp["fpid"], ("1", "2"))
outline = mp.outline(doc)
views = {False: mp.View(top_bmp, outline, False), True: mp.View(bottom_bmp, outline, True)}
res = {"calibration": {("bottom" if b else "top"): {
    "px_per_mm_x": round(v.sx, 4), "px_per_mm_y": round(v.sy, 4),
    "mm_per_px": round(2 / (v.sx + v.sy), 5),
    "board_px_box": [v.left, v.top, v.right, v.low]} for b, v in views.items()}, "rows": {}}
for fp in doc["fps"]:
    if fp["ref"] not in refs:
        continue
    v = views[fp["bottom"]]
    row = ba.read(v, fp)
    pads = [(p["x"] - fp["x"], p["y"] - fp["y"]) for p in fp["pads"] if p["num"]]
    row["fpid"] = fp["fpid"]
    row["pad_centre_span_mm"] = round(math.dist(*pads[:2]), 3) if len(pads) == 2 else None
    # extra diagnostics (not in body_axes): pixel counts by class inside the same reach
    reach = max(math.hypot(*xy) for xy in pads) + 0.6
    n = {"dark": 0, "board": 0, "copper": 0, "magenta": 0, "white": 0, "other": 0}
    dark_by_thr = {90: 0, 110: 0, 130: 0, 150: 0}
    for x, y, dx, dy in v.around(fp["x"], fp["y"], reach):
        px = v.px(x, y)
        if mp._board_px(*px):
            n["board"] += 1
        elif mp._copper(*px):
            n["copper"] += 1
        elif mp.is_magenta(px):
            n["magenta"] += 1
        else:
            for t in dark_by_thr:
                dark_by_thr[t] += max(px) < t
            if max(px) < 110:
                n["dark"] += 1
            elif min(px) > 200:
                n["white"] += 1
            else:
                n["other"] += 1
    row["reach_mm"] = round(reach, 3)
    row["pixels"] = n
    row["dark_area_mm2"] = round(n["dark"] / (v.sx * v.sy), 3)
    row["dark_px_by_threshold"] = dark_by_thr
    x0, y0 = outline[:2]
    ix = (v.right - (fp["x"] - x0) * v.sx) if v.bottom else (v.left + (fp["x"] - x0) * v.sx)
    iy = v.top + (fp["y"] - y0) * v.sy
    row["image_centre_px"] = [round(ix, 1), round(iy, 1)]
    res["rows"][fp["ref"]] = row
res["rows"] = {r: res["rows"][r] for r in refs}
with open(out, "w") as f:
    json.dump(res, f, indent=1)
    f.write("\n")
