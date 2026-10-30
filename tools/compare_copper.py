"""Compare every pad's copper between two boards, as KiCad reads them.

    <kicad python3> -I compare_copper.py BEFORE.kicad_pcb AFTER.kicad_pcb

Run with KiCad's Python.  Pads are keyed by (reference, pad number, index
among same-numbered pads).  Each pad compares its position, size, shape,
layers, drill and absolute orientation; orientation is compared modulo the
pad's symmetry (180 degrees, or 90 for a square or circular pad), because a
rect turned half a turn is the same copper.  Exit 1 on any difference.
"""
import sys

import pcbnew


def per_layer(getter):
    # KiCad 9+ keys pad geometry by layer; earlier versions take no argument.
    try:
        return getter(pcbnew.F_Cu)
    except TypeError:
        return getter()


def pads(path):
    board = pcbnew.LoadBoard(path)
    out = {}
    for fp in board.GetFootprints():
        seen = {}
        for pad in fp.Pads():
            number = pad.GetNumber()
            index = seen.get(number, 0)
            seen[number] = index + 1
            size = per_layer(pad.GetSize)
            w, h = size.x, size.y
            symmetry = 90 if w == h else 180
            angle = round(pad.GetOrientation().AsDegrees(), 6) % symmetry
            if abs(angle - symmetry) < 1e-6:
                angle = 0.0
            drill = pad.GetDrillSize()
            out[(fp.GetReference(), number, index)] = (
                pad.GetPosition().x, pad.GetPosition().y, w, h, int(per_layer(pad.GetShape)),
                angle, pad.GetLayerSet().FmtHex(), drill.x, drill.y,
            )
    return out


def main(before, after):
    a, b = pads(before), pads(after)
    problems = []
    for key in sorted(set(a) | set(b)):
        if a.get(key) != b.get(key):
            problems.append("%s: %s -> %s" % (key, a.get(key), b.get(key)))
    for line in problems[:40]:
        print(line)
    print("compared %d pads, %d differ" % (len(a), len(problems)))
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(*sys.argv[1:3]))
