#!/usr/bin/env python3
"""Write a board-outline DXF that BOTH CoHDL and mechanical CAD can read.

CoHDL's reader (RFC-020, src/dxf.rs) is deliberately narrow: it wants one
closed LWPOLYLINE on the `Edge.Cuts` layer and ignores the rest of the file.
That narrowness makes it tempting to hand-write a 50-line DXF containing only
that entity — which CoHDL accepts and every real CAD package rejects, because
a conforming DXF also needs a HEADER (so the reader knows the version and the
units), a LAYER table defining the layer the entity claims, block records,
entity handles, and the `AcDbEntity`/`AcDbPolyline` subclass markers that
LWPOLYLINE has required since R13. Autodesk Fusion refuses such a file
outright; so does a strict `ezdxf.readfile`.

So the outline is generated, not hand-written. Requires ezdxf (not a project
dependency — install it in a throwaway venv, the same way tools/kicad_board.py
borrows KiCad's own Python):

    python3 -m venv /tmp/dxfenv && /tmp/dxfenv/bin/pip install ezdxf
    /tmp/dxfenv/bin/python tools/make_board_outline_dxf.py \
        examples/openmicro/mechanical/openmicro-outline.dxf \
        --width 95 --height 95 --radius 6

Output bytes are reproducible: the writer's "now" timestamps and freshly
generated GUIDs are pinned afterwards, so regenerating an unchanged outline
produces an unchanged file (and an empty git diff).
"""

from __future__ import annotations

import argparse
import math
import re
import sys

# Header variables a DXF writer fills with the current time or a fresh random
# GUID. Committed assets must not churn, so each is pinned to a fixed value.
PINNED_FLOAT_VARS = (
    "$TDCREATE",
    "$TDUCREATE",
    "$TDUPDATE",
    "$TDUUPDATE",
    "$TDINDWG",
    "$TDUSRTIMER",
)
PINNED_GUID_VARS = ("$FINGERPRINTGUID", "$VERSIONGUID")
ZERO_GUID = "{00000000-0000-0000-0000-000000000000}"


def rounded_rect(width: float, height: float, radius: float):
    """A rounded rectangle centred on the origin, as (x, y, bulge) vertices.

    A bulge is tan(sweep / 4); each corner sweeps 90 degrees, so every corner
    arc carries tan(22.5 deg). DXF attaches a bulge to the vertex that STARTS
    the segment, so each corner's bulge rides the vertex where its arc begins.
    """
    if radius <= 0:
        raise SystemExit("--radius must be positive (a square corner is radius 0 — not supported)")
    if radius > min(width, height) / 2:
        raise SystemExit(f"--radius {radius} exceeds half the shorter side of {width}x{height}")
    x, y = width / 2, height / 2
    xi, yi = x - radius, y - radius  # where the straight edges meet the arcs
    corner = math.tan(math.radians(90) / 4)
    return [
        (-xi, -y, 0.0),  # bottom edge, left to right
        (xi, -y, corner),  # bottom-right corner
        (x, -yi, 0.0),  # right edge
        (x, yi, corner),  # top-right corner
        (xi, y, 0.0),  # top edge, right to left
        (-xi, y, corner),  # top-left corner
        (-x, yi, 0.0),  # left edge
        (-x, -yi, corner),  # bottom-left corner
    ]


def pin_nondeterministic_values(text: str) -> str:
    """Replace the VALUE line of each pinned header variable.

    Records are edited, never removed: dropping a record would orphan handles
    that other records point at, which is exactly the kind of damage that makes
    a DXF unreadable.
    """
    def pin(text: str, var: str, code: int, value: str) -> str:
        pattern = rf"(\n[ \t]*9\n{re.escape(var)}\n[ \t]*{code}\n)[^\n]*"
        text, n = re.subn(pattern, lambda m: m.group(1) + value, text)
        if n != 1:
            raise SystemExit(f"expected exactly one {var} record to pin, found {n}")
        return text

    for var in PINNED_FLOAT_VARS:
        text = pin(text, var, 40, "0.0")
    for var in PINNED_GUID_VARS:
        text = pin(text, var, 2, ZERO_GUID)

    # The writer also records itself as "<version> @ <ISO timestamp>" in the
    # object dictionary. Keep which version wrote the file (useful provenance),
    # drop the clock reading (pure churn).
    text, _ = re.subn(
        r"(\n[ \t]*100\nDictionaryVariables\n[ \t]*280\n[ \t]*0\n[ \t]*1\n)(\d+\.\d+\.\d+) @ [^\n]*",
        lambda m: m.group(1) + m.group(2),
        text,
    )
    return text


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("output", help="path of the .dxf to write")
    ap.add_argument("--width", type=float, required=True, help="board width in mm")
    ap.add_argument("--height", type=float, required=True, help="board height in mm")
    ap.add_argument("--radius", type=float, required=True, help="corner radius in mm")
    ap.add_argument("--layer", default="Edge.Cuts", help="outline layer (default: %(default)s)")
    args = ap.parse_args()

    try:
        import ezdxf
        from ezdxf.audit import Auditor
    except ImportError:
        print(
            "ezdxf is required: python3 -m venv /tmp/dxfenv && "
            "/tmp/dxfenv/bin/pip install ezdxf",
            file=sys.stderr,
        )
        return 1

    points = rounded_rect(args.width, args.height, args.radius)

    doc = ezdxf.new("R2000", setup=False)
    doc.header["$INSUNITS"] = 4  # millimetres — without this an importer must guess
    doc.header["$MEASUREMENT"] = 1  # metric
    doc.header["$EXTMIN"] = (-args.width / 2, -args.height / 2, 0.0)
    doc.header["$EXTMAX"] = (args.width / 2, args.height / 2, 0.0)
    doc.layers.add(args.layer)
    doc.modelspace().add_lwpolyline(
        points, format="xyb", close=True, dxfattribs={"layer": args.layer}
    )
    doc.saveas(args.output)

    # The writer stamps time and GUIDs during save, so pin them in the file.
    with open(args.output, encoding="utf-8") as fh:
        text = fh.read()
    with open(args.output, "w", encoding="utf-8") as fh:
        fh.write(pin_nondeterministic_values(text))

    # Validate what was actually written, strictly — no recover mode.
    doc = ezdxf.readfile(args.output)
    auditor = Auditor(doc)
    auditor.run()
    if auditor.errors:
        for e in auditor.errors:
            print(f"  audit error {e.code}: {e.message}", file=sys.stderr)
        return 1
    written = list(doc.modelspace())[0].get_points(format="xyb")
    if [(x, y, float(b)) for x, y, b in written] != points:
        print("  geometry did not survive the round trip", file=sys.stderr)
        return 1

    lo, hi = -args.width / 2, args.width / 2
    print(
        f"  wrote {args.output}: {args.width}x{args.height} mm, r{args.radius} corners, "
        f"layer {args.layer} ({len(points)} vertices, x/y in [{lo}, {hi}])"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
