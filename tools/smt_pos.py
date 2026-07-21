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

Run with KiCad's own python:
  /Applications/KiCad/KiCad.app/Contents/Frameworks/Python.framework/\
Versions/3.9/bin/python3 tools/smt_pos.py <board.kicad_pcb> [out.csv]
"""

import sys
from pathlib import Path

import pcbnew

# Bare-copper "components" that exist only as etched features — never placed.
BARE_COPPER_VALUES = {"TouchSensor", "TouchPad"}


def mm(v):
    return pcbnew.ToMM(v)


def main():
    board_path = Path(sys.argv[1])
    out_path = (
        Path(sys.argv[2])
        if len(sys.argv) > 2
        else board_path.with_name(board_path.stem + "-smt.csv")
    )
    board = pcbnew.LoadBoard(str(board_path))

    # Board outline bbox -> lower-left origin, Y-up.
    bb = board.GetBoardEdgesBoundingBox()
    x0, y_max = mm(bb.GetLeft()), mm(bb.GetBottom())

    def X(v):
        return mm(v) - x0

    def Y(v):
        return y_max - mm(v)

    rows = []
    skipped = []
    for fp in board.GetFootprints():
        ref = fp.GetReference()
        pads = list(fp.Pads())
        smd = [p for p in pads if p.GetAttribute() == pcbnew.PAD_ATTRIB_SMD]
        if not smd:
            skipped.append((ref, "no SMD pads (through-hole/mechanical)"))
            continue
        if fp.GetValue() in BARE_COPPER_VALUES or ref.startswith("TP"):
            skipped.append((ref, "bare copper feature, no physical part"))
            continue
        # Mid = centroid of copper pads (SMD + PTH; ignore holes-only).
        cop = [p for p in pads if p.GetAttribute() != pcbnew.PAD_ATTRIB_NPTH]
        cx = sum(p.GetPosition().x for p in cop) / len(cop)
        cy = sum(p.GetPosition().y for p in cop) / len(cop)
        anchor = fp.GetPosition()
        # Pad 1 (numeric-aware; falls back to the first pad).
        def key(p):
            n = p.GetPadName()
            return (0, int(n)) if n.isdigit() else (1, n)
        p1 = sorted((p for p in pads if p.GetPadName()), key=key)[0]
        # FPID = the projected .kicad_mod name: the CoHDL footprint's fq path
        # with `::` collapsed to `-` (e.g. `rpi_pico2-CHIP_0201`). CoHDL
        # identifiers can't contain `-`, so everything after the last `-` is
        # the short name — the exact form the BOM CSV's Footprint column uses.
        fpid = fp.GetFPIDAsString().split(":")[-1].split("-")[-1]
        rot = fp.GetOrientationDegrees() % 360
        rows.append(
            (
                ref,
                fpid,
                X(cx),
                Y(cy),
                X(anchor.x),
                Y(anchor.y),
                X(p1.GetPosition().x),
                Y(p1.GetPosition().y),
                "B" if fp.IsFlipped() else "T",
                rot,
                fp.GetValue(),
            )
        )

    # Sort: refs naturally (C1, C2, ... C10).
    def refkey(r):
        import re
        m = re.match(r"([A-Za-z]+)(\d+)", r[0])
        return (m.group(1), int(m.group(2))) if m else (r[0], 0)
    rows.sort(key=refkey)

    def f(v):
        return f"{v:.4f}".rstrip("0").rstrip(".") + "mm"

    lines = ["Designator,Footprint,Mid X,Mid Y,Ref X,Ref Y,Pad X,Pad Y,Layer,Rotation,Comment"]
    for r in rows:
        lines.append(
            f"{r[0]},{r[1]},{f(r[2])},{f(r[3])},{f(r[4])},{f(r[5])},"
            f"{f(r[6])},{f(r[7])},{r[8]},{r[9]:g},{r[10]}"
        )
    out_path.write_text("\n".join(lines) + "\n")
    print(f"wrote {out_path}  ({len(rows)} SMT components)")
    for ref, why in sorted(skipped, key=lambda s: s[0]):
        print(f"  skipped {ref:6s} — {why}")


if __name__ == "__main__":
    main()
