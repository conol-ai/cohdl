#!/usr/bin/env python
"""Apply a Quilter layout candidate to a KiCad board.

Quilter exports its result for Cadence Allegro (a SKILL script + a CSV of
design-modification rows). This project is KiCad, so we read the CSV — which
holds the actual changes — and replay it onto the .kicad_pcb with pcbnew.

Frame mapping (derived empirically, see the header comment in the commit):
the Quilter CSV lives in the coordinate space of the IPC-2581 document we fed
it (`build --emit ipc2581`), which is Y-UP and millimetre-based, while KiCad is
Y-DOWN. The CSV itself is in MICRONS. Hence:

    x_mm = x_um / 1000            y_mm = -y_um / 1000        rot_kicad = angle

The rotation sign (+1) was established by a net-aware pad-hit test: with +1 all
129 routed pads of the moved components land on a trace/via carrying that pad's
own net, with 0 net mismatches; -1 produces 38 mismatches.

Run with KiCad's own Python:
  /Applications/KiCad/KiCad.app/Contents/Frameworks/Python.framework/\
Versions/3.9/bin/python3 tools/apply_quilter.py <board.kicad_pcb> <candidate.csv>
"""

import csv
import shutil
import sys
from pathlib import Path

import pcbnew

# Quilter/Allegro layer token -> KiCad layer
LAYER_MAP = {
    "ETCH/F.CU": pcbnew.F_Cu,
    "ETCH/B.CU": pcbnew.B_Cu,
    "BOUNDARY/LAYER_1": pcbnew.F_Cu,
    "BOUNDARY/LAYER_2": pcbnew.B_Cu,
}


def to_mm(um):
    return float(um) / 1000.0


def pt(field):
    """'x:y' in microns (Y-up) -> (x_mm, y_mm) in KiCad's Y-down frame."""
    x, y = field.split(":")
    return to_mm(x), -to_mm(y)


def vec(xy):
    return pcbnew.VECTOR2I(pcbnew.FromMM(xy[0]), pcbnew.FromMM(xy[1]))


def main():
    board_path, csv_path = Path(sys.argv[1]), Path(sys.argv[2])

    backup = board_path.with_suffix(".pre-quilter.kicad_pcb")
    if not backup.exists():
        shutil.copy2(board_path, backup)
        print(f"backup: {backup.name}")

    board = pcbnew.LoadBoard(str(board_path))
    fps = {f.GetReference(): f for f in board.GetFootprints()}
    rows = list(csv.DictReader(open(csv_path)))

    def netcode(name):
        n = board.FindNet(name)
        return n.GetNetCode() if n else 0

    stats = {k: 0 for k in ("move", "rotate", "track", "via", "zone")}
    warn = []

    # --- component placement -------------------------------------------------
    # Apply every MOVE first, then every ROTATE, so a rotate row is never
    # undone by a later move row for the same component.
    for r in rows:
        if r["operation"] != "MOVE_COMPONENT_TO":
            continue
        f = fps.get(r["component"])
        if f is None:
            warn.append(f"move: unknown component {r['component']}")
            continue
        f.SetPosition(vec(pt(r["position"])))
        if r.get("mirror", "").lower() in ("true", "t", "1"):
            f.Flip(f.GetPosition(), False)
        stats["move"] += 1

    for r in rows:
        if r["operation"] != "ROTATE_COMPONENT":
            continue
        f = fps.get(r["component"])
        if f is None:
            warn.append(f"rotate: unknown component {r['component']}")
            continue
        f.SetOrientationDegrees(float(r["angle"]))
        stats["rotate"] += 1

    # --- routing -------------------------------------------------------------
    for r in rows:
        op = r["operation"]

        if op == "CREATE_LINE":
            layer = LAYER_MAP.get(r["layer"].upper())
            if layer is None:
                warn.append(f"line: unmapped layer {r['layer']}")
                continue
            t = pcbnew.PCB_TRACK(board)
            t.SetStart(vec(pt(r["start"])))
            t.SetEnd(vec(pt(r["end"])))
            t.SetWidth(pcbnew.FromMM(to_mm(r["width"])))
            t.SetLayer(layer)
            t.SetNetCode(netcode(r["net"]))
            board.Add(t)
            stats["track"] += 1

        elif op == "CREATE_VIA":
            v = pcbnew.PCB_VIA(board)
            v.SetPosition(vec(pt(r["position"])))
            v.SetViaType(pcbnew.VIATYPE_THROUGH)
            v.SetLayerPair(pcbnew.F_Cu, pcbnew.B_Cu)
            v.SetNetCode(netcode(r["net"]))
            board.Add(v)
            stats["via"] += 1

        elif op == "CREATE_POUR":
            layer = LAYER_MAP.get(r["layer"].upper())
            if layer is None:
                warn.append(f"pour: unmapped layer {r['layer']}")
                continue
            z = pcbnew.ZONE(board)
            z.SetLayer(layer)
            z.SetNetCode(netcode(r["net"]))
            pts = pcbnew.VECTOR_VECTOR2I()
            for p in r["position"].split():
                pts.append(vec(pt(p)))
            z.AddPolygon(pts)
            board.Add(z)
            stats["zone"] += 1

    # Via geometry comes from the CREATE_PADSTACK row (one padstack for the
    # whole candidate): `width` is the drill, `position` the pad x:y.
    for r in rows:
        if r["operation"] != "CREATE_PADSTACK":
            continue
        drill = pcbnew.FromMM(to_mm(r["width"]))
        diam = pcbnew.FromMM(to_mm(r["position"].split(":")[0]))
        for t in board.GetTracks():
            if isinstance(t, pcbnew.PCB_VIA):
                t.SetDrill(drill)
                t.SetWidth(diam)
        print(
            f"padstack {r['padstack']}: drill {pcbnew.ToMM(drill):.3f}mm "
            f"pad {pcbnew.ToMM(diam):.3f}mm"
        )

    filler = pcbnew.ZONE_FILLER(board)
    filler.Fill(board.Zones())

    board.Save(str(board_path))

    print(
        f"applied: {stats['move']} moves, {stats['rotate']} rotates, "
        f"{stats['track']} tracks, {stats['via']} vias, {stats['zone']} pours"
    )
    for w in warn[:20]:
        print("  WARN:", w)
    if len(warn) > 20:
        print(f"  ... {len(warn) - 20} more warnings")


if __name__ == "__main__":
    main()
