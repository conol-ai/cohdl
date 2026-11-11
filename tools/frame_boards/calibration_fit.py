"""Fit a preview's chip body offsets: a shift of the export, or an offset of the model?

Usage: python3 -I tools/frame_boards/calibration_fit.py board.json measurements.json BOM.csv OUT.json
           [--min-angles N]

board.json is `measure_preview.py board` (in a preview package), measurements.json
is `measure_preview.py measure` with `offset_xy_mm`, and BOM.csv is the package's
supplier BOM, which names each designator's part number.

`docs/library-supplier-w3-sampling.md` section 5 leaves a question open: the W3
rule takes a view's median body offset off each sample, on the hypothesis that the
shift is the export's. Two things can move a drawn body off its land:

- an export shift E: fixed in the board frame, whatever the part's angle;
- a model offset m: fixed in the part's own frame, so it turns with the part.

A part number placed at one angle cannot tell them apart. Placed at several angles
in one view, each reading is  offset = E + R(part) m,  where R(part) carries the
part's frame into the board's. This fits, by least squares, three models to each
set of readings:

  export_only   offset = E
  model_only    offset = R m
  both          offset = E + R m

for three poolings, each within one view (an export is one view):

  parts   one part number: its own E and m;
  lands   the part numbers of one land: one E and one m they share (an offset
          between the assembler's land and ours would look like this);
  views   every fitted part number of the view: one E for the view and an m for
          each part number.

Each fit gives its components and its misfit: `rms_mm` and `max_mm` are over the
readings' residual vectors, and `sigma_mm` is the residual per coordinate with the
fitted parameters taken off the count (absent when nothing is left over). A
hypothesis holds when its misfit is no worse than the scatter of repeated
readings; `both` always fits at least as well as either alone, so what matters is
how much worse the single hypotheses are and how large each component of `both`
is. `separation` is 0 when the angles cannot tell E from m and near 1 when they
can; `both` is absent at 0.

The part's frame is read from board.json's pads, not from an angle convention:
`along` runs from pad 1 to pad 2, and `across` is the land's +y at authoring angle
0 (a quarter turn from `along`, clockwise as the view from above shows a top part,
mirrored for a bottom part). A bottom part's frame is the top one mirrored, which
assumes the assembler mirrors its model with the part.

Only chip bodies are read (`measure_preview.py`'s `is_chip`). A part number enters
the fits of a view when at least N (default 3) of its orientations there have a
body reading; the rest are listed under `not_fitted` with the reason. A body the
preview did not draw is unverified, not a zero.
"""
import argparse
import csv
import hashlib
import json
import math
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import measure_preview as mp  # noqa: E402

MODELS = ("export_only", "model_only", "both")


def part_frame(fp):
    """The board-frame unit vectors of a chip's `along` and `across` axes."""
    pads = {p["num"]: p for p in fp["pads"] if p["num"]}
    dx, dy = pads["2"]["x"] - pads["1"]["x"], pads["2"]["y"] - pads["1"]["y"]
    size = math.hypot(dx, dy)
    ux, uy = dx / size, dy / size
    return (ux, uy), ((uy, -ux) if fp["bottom"] else (-uy, ux))


def readings(board, measured, bom_text):
    """The chip body readings, and the chip placements that have none.

    Each reading: ref, pn, land, view, the part's frame and the body's offset.
    """
    pn_of = {}
    for row in csv.DictReader(bom_text.splitlines()):
        for ref in row["Designator"].split(","):
            pn_of[ref] = row["LCSC Part #"]
    by_ref = {row["ref"]: row for row in measured}
    read, unread = [], []
    for fp in board["fps"]:
        if not mp.is_chip(fp):
            continue
        row = by_ref.get(fp["ref"])
        item = {"ref": fp["ref"], "pn": pn_of.get(fp["ref"]), "land": fp["fpid"],
                "view": "bottom" if fp["bottom"] else "top", "kicad_angle": fp["angle"]}
        body = (row or {}).get("body") or {}
        if item["pn"] is None:
            reason = "not in the BOM"
        elif row is None:
            reason = "not measured"
        elif row.get("drawn") is False:
            reason = "not drawn"
        elif "offset_xy_mm" not in body or body.get("fill", 0) < mp.CHIP_FILL_MIN:
            reason = "no body reading"
        else:
            item["frame"] = part_frame(fp)
            item["offset"] = tuple(body["offset_xy_mm"])
            read.append(item)
            continue
        unread.append(dict(item, reason=reason))
    return read, unread


def orientation(item):
    (ux, uy), _ = item["frame"]
    return round(math.degrees(math.atan2(uy, ux))) % 360


def to_board(frame, m):
    (ux, uy), (vx, vy) = frame
    return (ux * m[0] + vx * m[1], uy * m[0] + vy * m[1])


def to_part(frame, o):
    (ux, uy), (vx, vy) = frame
    return (ux * o[0] + uy * o[1], vx * o[0] + vy * o[1])


def fit(items, key):
    """The three least-squares fits of ITEMS; KEY names which readings share a model offset.

    The part frames are orthonormal, so with E given each key's offset is the mean
    of its readings, less E, turned into the part frame. Putting that into E's own
    equation leaves two equations in E:
        (n I - sum_k S_k S_k^T / n_k) E = sum_i o_i - sum_k S_k b_k / n_k
    where S_k sums the frames of key k and b_k its readings turned into the part frame.
    """
    n = len(items)
    keys = sorted({key(i) for i in items})
    groups = {k: [i for i in items if key(i) == k] for k in keys}
    total = (sum(i["offset"][0] for i in items), sum(i["offset"][1] for i in items))
    turned = {k: (sum(to_part(i["frame"], i["offset"])[0] for i in g),
                  sum(to_part(i["frame"], i["offset"])[1] for i in g)) for k, g in groups.items()}

    def misfit(shift, offsets, parameters):
        sizes = []
        for i in items:
            px, py = to_board(i["frame"], offsets[key(i)]) if offsets else (0.0, 0.0)
            sizes.append(math.hypot(i["offset"][0] - shift[0] - px, i["offset"][1] - shift[1] - py))
        ssr, left = sum(s * s for s in sizes), 2 * n - parameters
        out = {"rms_mm": round(math.sqrt(ssr / n), 4), "max_mm": round(max(sizes), 4)}
        if left > 0:
            out["sigma_mm"] = round(math.sqrt(ssr / left), 4)
        return out

    def vector(v):
        return [round(v[0], 4), round(v[1], 4)]

    def described(shift, offsets, parameters):
        out = {}
        if shift is not None:
            out["export_shift_xy_mm"] = vector(shift)
            out["export_shift_mm"] = round(math.hypot(*shift), 4)
        if offsets is not None:
            out["model_offset_along_across_mm"] = {k: vector(m) for k, m in offsets.items()}
            out["model_offset_mm"] = {k: round(math.hypot(*m), 4) for k, m in offsets.items()}
        out.update(misfit(shift or (0.0, 0.0), offsets, parameters))
        return out

    result = {"n": n, "orientations": sorted({orientation(i) for i in items})}
    result["export_only"] = described((total[0] / n, total[1] / n), None, 2)
    result["model_only"] = described(None, {k: (turned[k][0] / len(groups[k]), turned[k][1] / len(groups[k]))
                                            for k in keys}, 2 * len(keys))
    # The two equations in E.
    a, b, d = float(n), 0.0, float(n)
    rx, ry = total
    for k, g in groups.items():
        # S_k as a matrix whose columns are the summed along and across vectors.
        sux, suy = sum(i["frame"][0][0] for i in g), sum(i["frame"][0][1] for i in g)
        svx, svy = sum(i["frame"][1][0] for i in g), sum(i["frame"][1][1] for i in g)
        a -= (sux * sux + svx * svx) / len(g)
        b -= (sux * suy + svx * svy) / len(g)
        d -= (suy * suy + svy * svy) / len(g)
        rx -= (sux * turned[k][0] + svx * turned[k][1]) / len(g)
        ry -= (suy * turned[k][0] + svy * turned[k][1]) / len(g)
    # The smaller eigenvalue of the E equations over n: 0 when E and m cannot be told apart.
    separation = ((a + d) / 2 - math.hypot((a - d) / 2, b)) / n
    result["separation"] = round(max(separation, 0.0), 4)
    if separation > 1e-9:
        det = a * d - b * b
        shift = ((d * rx - b * ry) / det, (a * ry - b * rx) / det)
        offsets = {}
        for k, g in groups.items():
            ex, ey = 0.0, 0.0
            for i in g:
                px, py = to_part(i["frame"], shift)
                ex, ey = ex + px, ey + py
            offsets[k] = ((turned[k][0] - ex) / len(g), (turned[k][1] - ey) / len(g))
        result["both"] = described(shift, offsets, 2 + 2 * len(keys))
    return result


def single(result):
    """A fit with one model offset: drop the one-key maps."""
    for model in MODELS:
        for name in ("model_offset_along_across_mm", "model_offset_mm"):
            if name in result.get(model, {}):
                (result[model][name],) = result[model][name].values()
    return result


def report(board, measured, bom_text, min_angles=3):
    read, unread = readings(board, measured, bom_text)
    doc = {"min_angles": min_angles, "parts": [], "lands": [], "views": [], "not_fitted": []}
    for item in unread:
        doc["not_fitted"].append({k: item[k] for k in ("ref", "pn", "land", "view", "reason")})
    for view in ("top", "bottom"):
        in_view = [i for i in read if i["view"] == view]
        fitted = []
        for pn in sorted({i["pn"] for i in in_view}, key=lambda pn: (len(pn), pn)):
            items = [i for i in in_view if i["pn"] == pn]
            angles = len({orientation(i) for i in items})
            if angles < min_angles:
                doc["not_fitted"].extend({"ref": i["ref"], "pn": pn, "land": i["land"], "view": view,
                                          "reason": "%d of its orientations read in this view" % angles}
                                         for i in items)
                continue
            fitted.extend(items)
            doc["parts"].append({"view": view, "pn": pn, "land": items[0]["land"],
                                 "refs": [i["ref"] for i in items], **single(fit(items, lambda i: i["pn"]))})
        for land in sorted({i["land"] for i in fitted}):
            items = [i for i in fitted if i["land"] == land]
            doc["lands"].append({"view": view, "land": land, "pns": sorted({i["pn"] for i in items}),
                                 **single(fit(items, lambda i: i["land"]))})
        if fitted:
            doc["views"].append({"view": view, "pns": sorted({i["pn"] for i in fitted}),
                                 **fit(fitted, lambda i: i["pn"])})
    return doc


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("board", type=Path)
    ap.add_argument("measurements", type=Path)
    ap.add_argument("bom", type=Path)
    ap.add_argument("out", type=Path)
    ap.add_argument("--min-angles", type=int, default=3)
    args = ap.parse_args(argv)
    if args.min_angles < 2:
        ap.error("--min-angles must be at least 2: one orientation cannot tell a shift from an offset")
    inputs = (args.board, args.measurements, args.bom)
    doc = report(json.loads(args.board.read_text(encoding="utf-8")),
                 json.loads(args.measurements.read_text(encoding="utf-8")),
                 args.bom.read_text(encoding="utf-8"), args.min_angles)
    doc = {"inputs_sha256": {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}, **doc}
    args.out.write_text(json.dumps(doc, indent=1) + "\n", encoding="utf-8")
    print("%d part fits, %d land fits, %d view fits; %d placements not fitted" %
          (len(doc["parts"]), len(doc["lands"]), len(doc["views"]), len(doc["not_fitted"])))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
