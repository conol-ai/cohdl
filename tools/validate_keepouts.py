#!/usr/bin/env python3
"""KiCad integration gate for CoHDL antenna keepouts (run with pcbnew Python).

Build the compiler first, then pass --cohdl target/debug/cohdl and
--kicad-cli /path/to/kicad-cli. This creates only temporary test boards.
It compares native boards with imported .kicad_mod geometry, enables four
copper layers, and proves DRC rejects deliberate intrusions on every layer.
"""
import argparse
import json
import math
from pathlib import Path
import subprocess
import tempfile
import uuid

import pcbnew
import wx


LIB = """pub pad P { shape: rect, size: (1mm, 1mm), layer: top_copper, plating: smd }
pub footprint F {
    pad 1: P at (0mm, 0mm)
    keepout { shape: rect, at: (2mm, -4mm), size: (4mm, 2mm) }
}
pub device D { pins { P: 1 [passive] } }
pub part M: D { primary { mfr: "test", mpn: "test", footprint: F } }
"""


def vector(x, y):
    return pcbnew.VECTOR2I(round(x * 1e6), round(y * 1e6))


def transform(x, y, ox, oy, rotation, bottom):
    x = -x if bottom else x
    rad = math.radians(rotation)
    return (ox + x * math.cos(rad) + y * math.sin(rad),
            oy - x * math.sin(rad) + y * math.cos(rad))


def polygon(zone):
    outline = zone.Outline()
    return sorted((outline.CVertex(i).x, outline.CVertex(i).y)
                  for i in range(outline.TotalVertices()))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cohdl", required=True, type=Path)
    parser.add_argument("--kicad-cli", required=True, type=Path)
    args = parser.parse_args()
    app = wx.App(False)  # pcbnew's footprint loader needs wx's standard paths.
    cohdl = args.cohdl.resolve()
    cases = [(rotation, bottom) for bottom in (False, True)
             for rotation in (0, 37, 90, 180, 270)]
    with tempfile.TemporaryDirectory(prefix="cohdl-keepout-gate-") as temp:
        root = Path(temp)
        source = root / "fixture.cohdl"
        lines = [LIB, "design B {"]
        for i in range(len(cases)):
            lines += [f"    inst m{i}: M", f"    nc: m{i}.P"]
        lines.append("    layout {")
        for i, (rotation, bottom) in enumerate(cases):
            lines.append(f"        place m{i} at ({30 + 20 * i}mm, 40mm) rotate {rotation}"
                         + (" side bottom" if bottom else ""))
        lines += ["    }", "}"]
        source.write_text("\n".join(lines) + "\n")
        subprocess.run([str(cohdl), "build", str(source), "--no-std", "--emit", "kicad_pcb"],
                       check=True, capture_output=True, text=True)
        native = pcbnew.LoadBoard(str(root / "out/fixture.kicad_pcb"))
        native.SetCopperLayerCount(4)
        enabled = native.GetEnabledLayers()
        enabled.AddLayer(pcbnew.In1_Cu)
        enabled.AddLayer(pcbnew.In2_Cu)
        native.SetEnabledLayers(enabled)
        board = root / "intrusions.kicad_pcb"
        pcbnew.SaveBoard(str(board), native)
        reference = pcbnew.BOARD()
        reference.SetCopperLayerCount(4)
        layers = [pcbnew.F_Cu, pcbnew.In1_Cu, pcbnew.In2_Cu, pcbnew.B_Cu]
        forbidden, allowed = set(), set()
        intrusions = []
        footprints = sorted(native.GetFootprints(), key=lambda f: f.GetPosition().x)
        assert len(footprints) == len(cases)
        for i, (footprint, (rotation, bottom)) in enumerate(zip(footprints, cases)):
            ox, oy = 30 + 20 * i, 40
            zones = list(footprint.Zones())
            assert len(zones) == 1
            zone = zones[0]
            assert zone.GetIsRuleArea()
            assert all(zone.IsOnLayer(layer) for layer in layers)
            assert all(getattr(zone, name)() for name in [
                "GetDoNotAllowTracks", "GetDoNotAllowVias", "GetDoNotAllowPads",
                "GetDoNotAllowZoneFills", "GetDoNotAllowFootprints"])
            expected = sorted(tuple(round(c * 1e6) for c in transform(x, y, ox, oy, rotation, bottom))
                              for x, y in [(0, -5), (4, -5), (4, -3), (0, -3)])
            assert all(abs(a - b) <= 3 for actual, want in zip(polygon(zone), expected)
                       for a, b in zip(actual, want)), (rotation, bottom, polygon(zone), expected)

            imported = pcbnew.FootprintLoad(str(root / "out/footprints"), "fixture-F")
            assert imported is not None
            reference.Add(imported)  # Flip requires the owning board in headless pcbnew.
            imported.SetPosition(vector(ox, oy))
            if bottom:
                imported.Flip(imported.GetPosition(), pcbnew.FLIP_DIRECTION_LEFT_RIGHT)
            # KiCad encodes a mirror-x placement with R+180 on the back.
            imported.SetOrientationDegrees(rotation + (180 if bottom else 0))
            ref_zone = list(imported.Zones())[0]
            assert all(abs(a - b) <= 3 for actual, want in zip(polygon(zone), polygon(ref_zone))
                       for a, b in zip(actual, want)), (rotation, bottom, "module/board drift", polygon(zone), polygon(ref_zone))
            for layer in ("F.Cu", "In1.Cu", "In2.Cu", "B.Cu"):
                for inside in (False, True):
                    y = -4 if inside else 2
                    ax, ay = transform(1, y, ox, oy, rotation, bottom)
                    bx, by = transform(3, y, ox, oy, rotation, bottom)
                    identity = str(uuid.uuid4())
                    intrusions.append(f'(segment (start {ax} {ay}) (end {bx} {by}) '
                                      f'(width 0.15) (layer "{layer}") (net "") (uuid "{identity}"))')
                    (forbidden if inside else allowed).add(identity)
            x, y = transform(2, -4, ox, oy, rotation, bottom)
            via_id, fp_id, pad_id = [str(uuid.uuid4()) for _ in range(3)]
            intrusions.append(f'(via (at {x} {y}) (size 0.6) (drill 0.3) '
                              f'(layers "F.Cu" "B.Cu") (net "") (uuid "{via_id}"))')
            intrusions.append(f'(footprint "intruder" (layer "F.Cu") (at {x} {y}) '
                              f'(uuid "{fp_id}") (attr smd) '
                              '(fp_rect (start -0.3 -0.3) (end 0.3 0.3) '
                              '(stroke (width 0.05) (type default)) (fill none) (layer "F.CrtYd")) '
                              f'(property "Reference" "INTRUDER{i}" (at 0 0) (layer "F.SilkS") '
                              '(effects (font (size 1 1) (thickness 0.15)))) '
                              f'(pad "1" smd rect (at 0 0) (size 0.4 0.4) '
                              f'(layers "F.Cu") (uuid "{pad_id}")))')
            forbidden.update([via_id, fp_id, pad_id])
        # Add controlled test objects using KiCad 10 native board syntax.
        contents = board.read_text().rstrip()
        assert contents.endswith(")")
        board.write_text(contents[:-1] + "\n" + "\n".join(intrusions) + "\n)\n")
        report = root / "drc.json"
        subprocess.run([str(args.kicad_cli), "pcb", "drc", "--format", "json",
                        "--output", str(report), str(board)], check=True, capture_output=True, text=True)
        data = json.loads(report.read_text())
        violations = [v for v in data["violations"] if v["type"] == "items_not_allowed"]
        rejected = {item["uuid"] for v in violations for item in v["items"]}
        missing = forbidden - rejected
        assert not missing, ("intrusions escaped DRC", missing)
        assert not (allowed & rejected), "outside control tracks incorrectly rejected"
        print(f"PASS: {len(cases)} rotations/sides, four copper layers, imported footprint parity; "
              f"{len(forbidden)} forbidden tracks/vias/pads/components rejected and "
              f"{len(allowed)} outside tracks allowed by keepouts (KiCad {pcbnew.Version()}).")
    del app


if __name__ == "__main__":
    main()
