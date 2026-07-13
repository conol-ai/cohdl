#!/usr/bin/env python3
"""Execute the MVP human checkpoint mechanically, with real KiCad.

Run with KiCad's bundled Python (so `pcbnew` is importable):

  /Applications/KiCad/KiCad.app/Contents/Frameworks/Python.framework/Versions/Current/bin/python3 \
      tools/kicad_checkpoint.py docs/demo/sensor-node.net docs/demo/sensor-node.kicad_pcb

What it does — the same work "File → Import Netlist" performs in pcbnew:
  1. Parse the legacy .net S-expression netlist.
  2. Load every component's footprint from KiCad's official libraries
     (a missing/renamed footprint fails the checkpoint).
  3. Verify every netlist node's pin exists as a pad on that footprint.
  4. Place the footprints, assign pad nets, draw a ratsnest-style line for
     every net on a user layer, and save a real .kicad_pcb.
  5. Print a checkpoint report (components, designator uniqueness, nets,
     unresolved pads).

The saved board can be opened directly in pcbnew; render it with
  kicad-cli pcb export pdf/svg.
"""

import sys
from pathlib import Path

import pcbnew  # noqa: E402  (KiCad-bundled module)

FOOTPRINT_DIR = Path("/Applications/KiCad/KiCad.app/Contents/SharedSupport/footprints")


# --------------------------------------------------------------------------
# Minimal S-expression parser for the legacy netlist.

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
            j = i + 1
            out = []
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
        for child in node:
            yield from find_all(child, head)


def child(node, head):
    for c in node:
        if isinstance(c, list) and c and c[0] == head:
            return c
    return None


# --------------------------------------------------------------------------

def main():
    net_path, out_path = sys.argv[1], sys.argv[2]
    text = Path(net_path).read_text()
    root = next(parse_sexpr(tokenize(text)))
    assert root[0] == "export", "not a KiCad legacy netlist"

    comps = []
    for c in find_all(root, "comp"):
        ref = child(c, "ref")[1]
        value = child(c, "value")[1]
        footprint = child(c, "footprint")[1]
        comps.append((ref, value, footprint))

    nets = []
    for n in find_all(root, "net"):
        name = child(n, "name")[1]
        nodes = [(child(nd, "ref")[1], child(nd, "pin")[1]) for nd in find_all(n, "node")]
        nets.append((name, nodes))

    print(f"netlist: {len(comps)} components, {len(nets)} nets")

    # Designator uniqueness (the RFC-005 promise, re-checked in KiCad's court).
    refs = [r for r, _, _ in comps]
    assert len(refs) == len(set(refs)), "designator collision!"

    board = pcbnew.NewBoard(out_path)

    # Nets.
    netinfo = {}
    for name, _ in nets:
        item = pcbnew.NETINFO_ITEM(board, name)
        board.Add(item)
        netinfo[name] = item

    # Footprints, in a grid.
    missing_fp, placed = [], {}
    col = row = 0
    for ref, value, fpid in sorted(comps):
        lib, fpname = fpid.split(":", 1)
        libdir = FOOTPRINT_DIR / f"{lib}.pretty"
        fp = pcbnew.FootprintLoad(str(libdir), fpname) if libdir.is_dir() else None
        if fp is None:
            missing_fp.append((ref, fpid))
            continue
        fp.SetReference(ref)
        fp.SetValue(value)
        fp.SetPosition(pcbnew.VECTOR2I(
            pcbnew.FromMM(30 + col * 25), pcbnew.FromMM(30 + row * 25)))
        board.Add(fp)
        placed[ref] = fp
        col += 1
        if col == 5:
            col, row = 0, row + 1
        print(f"  footprint OK: {ref:4s} {fpid}")

    if missing_fp:
        print("\nFOOTPRINTS NOT FOUND IN KICAD LIBRARIES:")
        for ref, fpid in missing_fp:
            print(f"  {ref}: {fpid}")

    # Pad nets + ratsnest lines on a user layer.
    unresolved_pads = []
    for name, nodes in nets:
        positions = []
        for ref, pin in nodes:
            fp = placed.get(ref)
            if fp is None:
                continue
            pad = next((p for p in fp.Pads() if p.GetNumber() == pin), None)
            if pad is None:
                unresolved_pads.append((ref, pin, name))
                continue
            pad.SetNet(netinfo[name])
            positions.append(pad.GetPosition())
        for a, b in zip(positions, positions[1:]):
            seg = pcbnew.PCB_SHAPE(board)
            seg.SetShape(pcbnew.SHAPE_T_SEGMENT)
            seg.SetStart(a)
            seg.SetEnd(b)
            seg.SetLayer(pcbnew.Dwgs_User)
            seg.SetWidth(pcbnew.FromMM(0.05))
            board.Add(seg)

    if unresolved_pads:
        print("\nNETLIST PINS WITH NO MATCHING FOOTPRINT PAD:")
        for ref, pin, name in unresolved_pads:
            print(f"  {ref}.{pin} (net {name})")

    pcbnew.SaveBoard(out_path, board)
    print(f"\nboard written: {out_path}")

    ok = not missing_fp and not unresolved_pads
    print("CHECKPOINT:", "PASS — KiCad resolved every footprint and pad" if ok
          else "FAIL — see findings above")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
