#!/usr/bin/env python
"""SMT pick-and-place (CPL) export from a routed .kicad_pcb.

Emits the JLC/Altium-style coordinate CSV:

    Designator,Footprint,Mid X,Mid Y,Ref X,Ref Y,Pad X,Pad Y,Layer,Rotation,Comment

- Values in mm with an "mm" suffix, matching the common SMT-house template.
- Origin: the board outline's LOWER-LEFT corner, Y positive UPWARD (machine
  convention) — so every coordinate is positive.
- Mid X/Y  = centroid of the footprint's copper pads.
- Ref X/Y  = the footprint anchor (KiCad position).
- Pad X/Y  = pad "1" (lowest-numbered pad as fallback).
- Layer    = T / B; Rotation = KiCad orientation normalized to [0, 360).
  (Per-part rotation offsets against a specific fab's tape orientation are the
  fab's own correction table — not applied here.)

Included: every footprint with at least one SMD pad (hybrids like the USB-C
receptacle count — machine-placed), EXCEPT bare-copper features with no
physical part (the cap-touch pad). Pure through-hole parts (keyswitches,
encoder, headers) and mechanical-only footprints (mount holes) are excluded.

With --all, nothing is excluded: every footprint on the board is emitted
(through-hole, mount holes, bare copper) in the same format/origin — for
plate/case CAD cross-checks, not for the pick-and-place machine. Default
output name becomes <stem>-pos-all.csv.

With --neutral, emit the RFC-034 supplier-neutral five-column CPL instead.
It uses footprint anchors, numbered-SMD row selection, Top/Bottom layers and
fixed four-place millimetres. Its default output name is <stem>-cpl.csv.

Run with KiCad's own python:
  /Applications/KiCad/KiCad.app/Contents/Frameworks/Python.framework/\
Versions/3.9/bin/python3 tools/smt_pos.py [--all | --neutral] <board.kicad_pcb> [out.csv]
"""

import argparse
import sys
from pathlib import Path

from smt_pos_lib import (
    legacy_csv,
    legacy_origin,
    legacy_rows,
    neutral_csv,
    neutral_origin,
    neutral_rows,
    outline_geometry_bounds,
)


def main():
    parser = argparse.ArgumentParser(
        description="Export legacy or RFC-034 supplier-neutral placement CSV."
    )
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--all", action="store_true", help="include every footprint")
    mode.add_argument(
        "--neutral", action="store_true", help="write the RFC-034 five-column CPL"
    )
    parser.add_argument("board", type=Path)
    parser.add_argument("output", nargs="?", type=Path)
    args = parser.parse_args()

    # KiCad is needed only by the board adapter, never by the pure library or
    # argument validation. This also lets --help and usage errors run anywhere.
    import pcbnew

    if args.neutral:
        suffix = "-cpl.csv"
    elif args.all:
        suffix = "-pos-all.csv"
    else:
        suffix = "-smt.csv"
    out_path = (
        args.output
        if args.output is not None
        else args.board.with_name(args.board.stem + suffix)
    )
    board = pcbnew.LoadBoard(str(args.board))

    def mm(value):
        return pcbnew.ToMM(value)

    # The legacy origin intentionally retains the old empty-outline behavior.
    bb = board.GetBoardEdgesBoundingBox()
    old_origin = legacy_origin(mm(bb.GetLeft()), mm(bb.GetBottom()))
    # KiCad inflates each shape's bbox by half its stroke width. Neutral mode
    # needs the authoring geometry instead; legacy deliberately keeps ``bb``.
    new_origin = (0, 0)
    if args.neutral:
        shapes = []
        for drawing in board.GetDrawings():
            if drawing.GetLayer() != pcbnew.Edge_Cuts:
                continue
            shape = drawing.GetShape()
            kinds = {
                pcbnew.SHAPE_T_SEGMENT: "segment",
                pcbnew.SHAPE_T_ARC: "arc",
                pcbnew.SHAPE_T_RECT: "rect",
                pcbnew.SHAPE_T_CIRCLE: "circle",
                pcbnew.SHAPE_T_POLY: "poly",
            }

            def point(value):
                return value.x, value.y

            record = {
                "shape": kinds.get(shape),
                "start": point(drawing.GetStart()),
                "end": point(drawing.GetEnd()),
                "width": drawing.GetWidth(),
                "bbox": (
                    drawing.GetBoundingBox().GetLeft(),
                    drawing.GetBoundingBox().GetTop(),
                    drawing.GetBoundingBox().GetRight(),
                    drawing.GetBoundingBox().GetBottom(),
                ),
            }
            if record["shape"] == "arc":
                record["mid"] = point(drawing.GetArcMid())
                record["center"] = point(drawing.GetCenter())
            elif record["shape"] == "circle":
                record["center"] = point(drawing.GetCenter())
            elif record["shape"] == "poly":
                record["points"] = [point(p) for p in drawing.GetPolyPoints()]
            shapes.append(record)
        new_origin = neutral_origin(outline_geometry_bounds(shapes))

    # Adapt pcbnew once. All subsequent filtering, geometry and CSV generation
    # operates on plain records in the standard-library-only module.
    footprints = []
    for fp in board.GetFootprints():
        pads = []
        for pad in fp.Pads():
            attribute = pad.GetAttribute()
            position = pad.GetPosition()
            pads.append(
                {
                    "number": pad.GetPadName(),
                    "position": (position.x, position.y),
                    "smd": attribute == pcbnew.PAD_ATTRIB_SMD,
                    "copper": attribute != pcbnew.PAD_ATTRIB_NPTH,
                }
            )
        anchor = fp.GetPosition()
        footprints.append(
            {
                "ref": fp.GetReference(),
                "value": fp.GetValue(),
                "anchor": (anchor.x, anchor.y),
                "orientation": fp.GetOrientationDegrees(),
                "flipped": fp.IsFlipped(),
                "footprint": fp.GetFPIDAsString()
                .split(":")[-1]
                .split("-")[-1],
                "pads": pads,
            }
        )

    if args.neutral:
        rows, skipped = neutral_rows(footprints, new_origin)
        out_path.write_text(neutral_csv(rows))
        what = "SMT components (neutral)"
    else:
        rows, skipped = legacy_rows(
            footprints, old_origin, mm, everything=args.all
        )
        out_path.write_text(legacy_csv(rows))
        what = "components (all)" if args.all else "SMT components"

    print(f"wrote {out_path}  ({len(rows)} {what})")
    for ref, why in sorted(skipped, key=lambda item: item[0]):
        print(f"  skipped {ref:6s} — {why}")


if __name__ == "__main__":
    main()
