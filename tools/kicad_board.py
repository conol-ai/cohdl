#!/usr/bin/env python3
"""SUPERSEDED (2026-08-25): `cohdl build --emit kicad_pcb` now writes the
board natively — no KiCad installation, no IPC-XML sidecar, byte-stable
(docs/kicad_pcb.md). This script is kept as the independent pcbnew-based
reference implementation the native emitter was verified against.

Build a real, openable KiCad board (.kicad_pcb) from a CoHDL build's own
outputs: the KiCad netlist (.net), the emitted footprints (out/footprints/
*.kicad_mod — RFC-018 geometry), and — since RFC-020 — the board outline and
component placements taken from the emitted IPC-2581 document (out/<name>.xml).

The result mirrors the IPC-2581 / Quilter view: the real board outline on
Edge.Cuts (straight segments + corner arcs), every footprint placed at its
IPC-2581 `Component/Location` with its `Xform` rotation (the pre-positioned
interface ports at their board-edge spots, the rest staged just outside), and
the full netlist (KiCad draws the ratsnest from the pad net assignments).
When a design has neither an outline nor explicit placements, IPC's `(0,0)`
locations are placeholders; in that case the footprints are staged on a grid
so the starter board remains inspectable instead of stacking every part.

The IPC-2581 document is in the standard IPC frame (+y up), matching KiCad's
own `kicad-cli pcb export ipc2581` convention. KiCad's board frame is +y down,
so every y read from the document is negated on import (placements and the
outline) — the resulting board then reads the same way up as the IPC-2581 /
Quilter view, and as the CoHDL design intends.

Run with KiCad's bundled Python (so `pcbnew` imports):

  KPY=/Applications/KiCad/KiCad.app/Contents/Frameworks/Python.framework/Versions/Current/bin/python3
  cargo run -- build examples/rpi-pico2 --emit ipc2581     # refresh .net + footprints + .xml
  "$KPY" tools/kicad_board.py examples/rpi-pico2/out/rpi-pico2.net

Output: <name>.kicad_pcb next to the netlist (File > Open in pcbnew).

To cross-check a native board against this reference semantically (UUIDs and
serialization deliberately differ), run ``tools/validate_kicad_pcb.py`` with
the same KiCad Python.
"""
import json
import math
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

import pcbnew  # noqa: E402  (KiCad-bundled module)

NS = "{http://webstds.ipc.org/2581}"


# ---- minimal S-expression parser for the legacy netlist ---------------------
def tokenize(text):
    i, n = 0, len(text)
    while i < n:
        c = text[i]
        if c.isspace():
            i += 1
        elif c in "()":
            yield c
            i += 1
        elif c == '"':
            j, out = i + 1, []
            while text[j] != '"':
                if text[j] == "\\":
                    j += 1
                out.append(text[j])
                j += 1
            yield ("str", "".join(out))
            i = j + 1
        else:
            j = i
            while j < n and not text[j].isspace() and text[j] not in "()":
                j += 1
            yield ("atom", text[i:j])
            i = j


def parse_sexpr(tokens):
    for tok in tokens:
        if tok == "(":
            lst = []
            for item in parse_sexpr(tokens):
                lst.append(item)
            yield lst
        elif tok == ")":
            return
        else:
            yield tok[1]


def find_all(node, head):
    if isinstance(node, list):
        if node and node[0] == head:
            yield node
        for c in node:
            yield from find_all(c, head)


def child(node, head):
    for c in node:
        if isinstance(c, list) and c and c[0] == head:
            return c
    return None


def mm(v):
    return pcbnew.FromMM(float(v))


def vec(x, y):
    return pcbnew.VECTOR2I(mm(x), mm(y))


def parse_ipc(xml_path):
    """Outline segments + per-refDes (x, y, rotation) from the IPC-2581 doc."""
    root = ET.parse(xml_path).getroot()
    step = next(root.iter(NS + "Step"))
    # Outline: PolyBegin then a run of PolyStepSegment / PolyStepCurve.
    outline = []
    prof = step.find(NS + "Profile")
    if prof is not None:
        # IPC +y-up -> KiCad +y-down: negate every y; an arc's winding flips too.
        for e in prof.iter():
            t = e.tag.split("}", 1)[-1]
            if t == "PolyBegin":
                outline.append(("begin", float(e.get("x")), -float(e.get("y"))))
            elif t == "PolyStepSegment":
                outline.append(("line", float(e.get("x")), -float(e.get("y"))))
            elif t == "PolyStepCurve":
                outline.append(("arc", float(e.get("x")), -float(e.get("y")),
                                float(e.get("centerX")), -float(e.get("centerY")),
                                e.get("clockwise") != "true"))
    # Component placements (y negated: IPC +y-up -> KiCad +y-down).
    # RFC-026: layerRef "B.Cu" marks a bottom-side component.
    place = {}
    for c in step.iter(NS + "Component"):
        loc = c.find(NS + "Location")
        xf = c.find(NS + "Xform")
        rot = float(xf.get("rotation")) if xf is not None and xf.get("rotation") else 0.0
        bottom = c.get("layerRef") == "B.Cu"
        place[c.get("refDes")] = (float(loc.get("x")), -float(loc.get("y")), rot, bottom)
    return outline, place


def draw_outline(board, outline):
    if not outline:
        return
    prev = None
    for seg in outline:
        if seg[0] == "begin":
            prev = (seg[1], seg[2])
            continue
        cur = (seg[1], seg[2])
        s = pcbnew.PCB_SHAPE(board)
        s.SetLayer(pcbnew.Edge_Cuts)
        s.SetWidth(mm(0.1))
        if seg[0] == "arc":
            cx, cy, cw = seg[3], seg[4], seg[5]
            # Arc from prev to cur about (cx, cy); midpoint via the bisector of
            # the two radii (correct for the <=180° corner arcs a board uses).
            r = math.hypot(prev[0] - cx, prev[1] - cy)
            bx, by = (prev[0] - cx) + (cur[0] - cx), (prev[1] - cy) + (cur[1] - cy)
            blen = math.hypot(bx, by) or 1.0
            midx, midy = cx + r * bx / blen, cy + r * by / blen
            try:
                s.SetShape(pcbnew.SHAPE_T_ARC)
                s.SetArcGeometry(vec(*prev), vec(midx, midy), vec(*cur))
            except Exception:
                s.SetShape(pcbnew.SHAPE_T_SEGMENT)
                s.SetStart(vec(*prev))
                s.SetEnd(vec(*cur))
        else:
            s.SetShape(pcbnew.SHAPE_T_SEGMENT)
            s.SetStart(vec(*prev))
            s.SetEnd(vec(*cur))
        board.Add(s)
        prev = cur


def main():
    net_path = Path(sys.argv[1])
    out_path = Path(sys.argv[2]) if len(sys.argv) > 2 else net_path.with_suffix(".kicad_pcb")
    out_dir = net_path.parent
    name = net_path.stem
    fp_dir = out_dir / "footprints"
    xml_path = out_dir / f"{name}.xml"
    layout_path = out_dir / f"{name}-layout.json"

    root = next(parse_sexpr(tokenize(net_path.read_text())))
    assert root[0] == "export", "not a KiCad legacy netlist"
    comps = [(child(c, "ref")[1], child(c, "value")[1], child(c, "footprint")[1])
             for c in find_all(root, "comp")]
    nets = [(child(n, "name")[1],
             [(child(nd, "ref")[1], child(nd, "pin")[1]) for nd in find_all(n, "node")])
            for n in find_all(root, "net")]
    print(f"netlist: {len(comps)} components, {len(nets)} nets")

    outline, place = ([], {})
    if xml_path.is_file():
        outline, place = parse_ipc(xml_path)
        print(f"IPC-2581: {len(outline)-1 if outline else 0} outline segments, "
              f"{len(place)} placed components")
        # Without an outline, IPC-2581 deliberately uses (0, 0) as the
        # placeholder location for every unplaced component.  A design with
        # no explicit `place` statements therefore needs the same fallback
        # grid as a build with no IPC document; treating those placeholders as
        # placement data would stack and electrically short every footprint.
        if not outline:
            layout = json.loads(layout_path.read_text()) if layout_path.is_file() else {}
            if not layout.get("placements"):
                print("note: IPC component locations are unplaced placeholders — "
                      "falling back to a grid")
                place = {}
    else:
        print(f"note: {xml_path.name} not found — run `build --emit ipc2581` first "
              f"for the outline + placements; falling back to a grid")

    board = pcbnew.NewBoard(str(out_path))
    draw_outline(board, outline)

    # board.Add assigns each net a unique code; KiCad resolves the pad refs by
    # name on load. (Each net's pads therefore stay on their own net — the dense
    # lines over a STAGED footprint are just the unrouted ratsnest to its
    # far-off net partners, not a short.)
    netinfo = {}
    for nm, _ in nets:
        item = pcbnew.NETINFO_ITEM(board, nm)
        board.Add(item)
        netinfo[nm] = item

    placed, missing = {}, []
    col = rowy = 0
    # The netlist emitter already writes components in canonical natural
    # designator order (C9 before C10).  Preserve that order so the fallback
    # grid is the same deterministic staging convention used by IPC-2581 and
    # the native .kicad_pcb emitter; Python's tuple sort is lexicographic and
    # would incorrectly put C10 before C3.
    for ref, value, fpid in comps:
        fp = pcbnew.FootprintLoad(str(fp_dir), fpid.replace("::", "-"))
        if fp is None:
            missing.append((ref, fpid))
            continue
        fp.SetReference(ref)
        fp.SetValue(value)
        if ref in place:
            x, y, rot, bottom = place[ref]
            fp.SetPosition(vec(x, y))
        else:
            rot, bottom = 0, False
            fp.SetPosition(vec(40 + col * 12, 40 + rowy * 12))
            col += 1
            if col == 8:
                col, rowy = 0, rowy + 1
        # Flip/rotate only AFTER board.Add — Flip consults the owning board's
        # layer table, and a board-less footprint segfaults headless pcbnew.
        board.Add(fp)
        if bottom:
            # RFC-026: KiCad-native back-side convention — flip left/right
            # about the anchor FIRST, then apply the declared rotation.
            # KiCad 9 replaced Flip's bool aFlipLeftRight with a FLIP_DIRECTION
            # enum; a bare True silently coerces to TOP_BOTTOM there, leaving
            # every back-side footprint 180 degrees from the IPC-2581 Xform.
            # Name the direction explicitly (bool True on pre-enum KiCad).
            flip_lr = getattr(pcbnew, "FLIP_DIRECTION_LEFT_RIGHT", True)
            fp.Flip(fp.GetPosition(), flip_lr)
            # Flipping at angle 0 leaves pcbnew holding SOME representation of
            # a pure x-mirror (KiCad 10 stores y-negated pads + a 180 angle).
            # Overwriting that angle with the declared rotation would silently
            # undo half the flip, so COMPOSE the rotation onto the flip's own
            # angle: the result is exactly RFC-026's rotate-after-mirror.
            if rot:
                fp.SetOrientationDegrees(rot + fp.GetOrientationDegrees())
        elif rot:
            fp.SetOrientationDegrees(rot)
        placed[ref] = fp

    unresolved = []
    for nm, nodes in nets:
        for ref, pin in nodes:
            fp = placed.get(ref)
            if fp is None:
                continue
            pads = [p for p in fp.Pads() if p.GetNumber() == pin]
            if not pads:
                unresolved.append((ref, pin, nm))
            else:
                # A logical pin may have multiple physical pads (for example,
                # an exposed land plus same-number thermal vias).  Every one
                # must carry the logical net; assigning only the first leaves
                # the remaining copper apparently shorted to an unnamed net.
                for pad in pads:
                    pad.SetNet(netinfo[nm])

    pcbnew.SaveBoard(str(out_path), board)
    print(f"board written: {out_path}")
    if missing:
        print("MISSING FOOTPRINTS:", missing)
    if unresolved:
        print("UNRESOLVED PADS:", unresolved[:10])
    ok = not missing and not unresolved
    print("RESULT:", "OK — every footprint + pad resolved" if ok else "INCOMPLETE (see above)")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
