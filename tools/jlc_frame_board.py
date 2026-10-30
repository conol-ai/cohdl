#!/usr/bin/env python3
"""Generate an RFC-034/JLC footprint-frame alignment board.

The input is a JSON object with ``rows``.  Each row needs ``footprint`` (the
CoHDL ``package::FOOTPRINT`` name) and ``pn`` (the LCSC part number).  Optional
Top-level ``dependencies`` is the exact CoHDL package/version map required to
resolve the rows' parts. Optional row fields are ``name``, ``part`` (a CoHDL
part type), ``mfr``, ``mpn``, ``bottom``, ``width_mm``, ``height_mm``, ``pins`` (the
CoHDL pin names used to make two nets), ``mapping`` (``dx_mm``, ``dy_mm`` and
``rotate_deg``), and ``placements`` (an explicit list of ``at_mm``, ``rotate``
and ``side`` objects).  ``bottom: true`` adds bottom 0 and 37 degree copies;
top 0 is always generated.  Explicit coordinates replay an independently
reviewed board.  Default grid coordinates are conservative rectangles and
``--spacing-mm`` controls their clearance.  Example::

  {"name":"frame_preview", "rows":[
    {"footprint":"demo::SOIC", "part":"demo::U", "pn":"C1",
     "mfr":"Demo", "mpn":"D-1", "bottom":true, "pins":["A","B"]}
  ]}

Run ``python3 -I tools/jlc_frame_board.py manifest.json -o BOARD
--cohdl target/debug/cohdl``.  CoHDL writes the board, BOM and supplier CPL.
``kicad-cli`` writes Gerbers and drill files with the outline's lower-left
corner as the drill/place origin.  No pcbnew module is imported; tests can mock
both command runners.  ``CHECKLIST.md`` must be checked against the assembler
preview before ordering.
"""

import argparse
import hashlib
import json
import math
import re
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path


class BoardError(ValueError):
    pass


def number(value, name, default=None):
    if value is None and default is not None:
        return default
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        raise BoardError(name + " must be a finite number")
    return value


def text(value, name, default=None):
    value = default if value is None else value
    if not isinstance(value, str) or not value.strip():
        raise BoardError(name + " must be a non-empty string")
    return value.strip()


def mm(value):
    value = number(value, "millimetres")
    return ("%.12f" % value).rstrip("0").rstrip(".") or "0"


def load_manifest(path):
    doc = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(doc, dict) or not isinstance(doc.get("rows"), list) or not doc["rows"]:
        raise BoardError("manifest must contain a non-empty rows array")
    rows = []
    names = set()
    for i, raw in enumerate(doc["rows"]):
        if not isinstance(raw, dict):
            raise BoardError("rows[%d] must be an object" % i)
        name = text(raw.get("name"), "rows[%d].name" % i, "part%d" % (i + 1))
        if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", name) or name in names:
            raise BoardError("row names must be unique CoHDL identifiers: " + name)
        names.add(name)
        fp = text(raw.get("footprint"), name + ".footprint")
        if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+", fp):
            raise BoardError(name + ".footprint must be package::FOOTPRINT")
        item = dict(raw)
        item.update(name=name, footprint=fp,
                    pn=text(raw.get("pn"), name + ".pn"),
                    part=text(raw.get("part"), name + ".part", fp.rsplit("::", 1)[0] + "::" + name),
                    mfr=text(raw.get("mfr"), name + ".mfr", "unknown"),
                    mpn=text(raw.get("mpn"), name + ".mpn", name),
                    bottom=raw.get("bottom", False))
        if not isinstance(item["bottom"], bool):
            raise BoardError(name + ".bottom must be boolean")
        for key in ("width_mm", "height_mm"):
            if number(raw.get(key), name + "." + key, 10) <= 0:
                raise BoardError(name + "." + key + " must be positive")
        mapping = raw.get("mapping", {})
        if not isinstance(mapping, dict):
            raise BoardError(name + ".mapping must be an object")
        item["mapping"] = {k: number(mapping.get(k), name + ".mapping." + k, 0)
                           for k in ("dx_mm", "dy_mm", "rotate_deg")}
        if item["mapping"]["rotate_deg"] % 1:
            raise BoardError(name + ".mapping.rotate_deg must be integral")
        item["placements"] = raw.get("placements")
        if item["placements"] is not None:
            if not isinstance(item["placements"], list) or not item["placements"]:
                raise BoardError(name + ".placements must be a non-empty array")
        item["pins"] = raw.get("pins", ["A", "B"])
        if not isinstance(item["pins"], list) or not item["pins"]:
            raise BoardError(name + ".pins must be a non-empty array")
        rows.append(item)
    dependencies = doc.get("dependencies", {"std": "0.3.0"})
    if not isinstance(dependencies, dict) or not all(isinstance(k, str) and isinstance(v, str) for k, v in dependencies.items()):
        raise BoardError("dependencies must be a package/version string map")
    return {"name": text(doc.get("name"), "name", "frame_preview"), "design": doc.get("design", "FramePreview"), "net_names": doc.get("net_names", ["N0", "N1"]), "outline": doc.get("outline"), "nets": doc.get("nets"), "rows": rows, "dependencies": dependencies}


def expand(rows, spacing):
    placements = []
    for column, row in enumerate(rows):
        if row.get("placements") is None:
            x = sum(number(r.get("width_mm"), "width_mm", 10) for r in rows[:column]) + spacing * column + number(row.get("width_mm"), "width_mm", 10) / 2
            local = [{"at_mm": [x, 5], "rotate": 0, "side": "top"}]
            if row.get("bottom", False):
                height = max(number(r.get("height_mm"), "height_mm", 10) for r in rows)
                local += [{"at_mm": [x, 5 + height + spacing], "rotate": 0, "side": "bottom"},
                          {"at_mm": [x, 5 + 2 * (height + spacing)], "rotate": 37, "side": "bottom"}]
        else:
            local = row["placements"]
        for index, place in enumerate(local):
            if not isinstance(place, dict) or not isinstance(place.get("at_mm"), list) or len(place["at_mm"]) != 2:
                raise BoardError(row["name"] + ".placements[%d].at_mm must be [x,y]" % index)
            side = place.get("side", "top")
            if side not in ("top", "bottom"):
                raise BoardError(row["name"] + " side must be top or bottom")
            placements.append(dict(row=row, index=index, x=number(place["at_mm"][0], "x"),
                                   y=number(place["at_mm"][1], "y"),
                                   rotate=number(place.get("rotate"), "rotate", 0), side=side))
    boxes = [(p["x"] - number(p["row"].get("width_mm"), "width", 10) / 2,
              p["y"] - number(p["row"].get("height_mm"), "height", 10) / 2,
              p["x"] + number(p["row"].get("width_mm"), "width", 10) / 2,
              p["y"] + number(p["row"].get("height_mm"), "height", 10) / 2) for p in placements]
    for i, a in enumerate(boxes):
        for b in boxes[i + 1:]:
            if a[0] < b[2] and b[0] < a[2] and a[1] < b[3] and b[1] < a[3]:
                raise BoardError("courtyards overlap; increase --spacing-mm or coordinates")
    margin = max(spacing, 1)
    bounds = (min(b[0] for b in boxes) - margin, min(b[1] for b in boxes) - margin,
              max(b[2] for b in boxes) + margin, max(b[3] for b in boxes) + margin)
    return placements, bounds


def emit_project(manifest, root, spacing, supplier="lcsc"):
    root.mkdir(parents=True, exist_ok=True)
    (root / "src").mkdir(exist_ok=True)
    (root / "mechanical").mkdir(exist_ok=True)
    placements, bounds = expand(manifest["rows"], spacing)
    declarations, instances, nets, layout, nc = [], [], [[], []], [], []
    for i, p in enumerate(placements):
        r, ref = p["row"], "%s_%d" % (p["row"]["name"], p["index"])
        map_ = r["mapping"]
        bind = 'supplier %s "%s"' % (supplier, r["pn"])
        if any(map_.values()):
            bind += " at (%smm, %smm) rotate %d" % (mm(map_["dx_mm"]), mm(map_["dy_mm"]), map_["rotate_deg"])
        declarations.append('pub part %s: %s {\n    primary { mfr: "%s", mpn: "%s", footprint: %s, %s }\n}\n' %
                            ("P_" + r["name"], r["part"], r["mfr"], r["mpn"], r["footprint"], bind) if p["index"] == 0 else "")
        instances.append("    inst %s: P_%s" % (ref, r["name"]))
        for n, pin in enumerate(r["pins"][:2]):
            nets[n].append("%s.%s" % (ref, pin))
        nc.extend("%s.%s" % (ref, pin) for pin in r["pins"][2:])
        clause = "        place %s at (%smm, %smm)" % (ref, mm(p["x"]), mm(p["y"]))
        if p["rotate"]:
            clause += " rotate %s" % mm(p["rotate"])
        if p["side"] == "bottom":
            clause += " side bottom"
        layout.append(clause)
    design = manifest.get("design", "FramePreview")
    source = "\n".join(declarations) + "\ndesign %s {\n" % design + "\n".join(instances) + "\n"
    net_names = manifest.get("net_names", ["N0", "N1"])
    explicit_nets = manifest.get("nets")
    if explicit_nets:
        source += "".join("    net %s: %s\n" % (name, ", ".join(members)) for name, members in explicit_nets.items())
    else:
        source += "    net %s: %s\n    net %s: %s\n" % (net_names[0], ", ".join(nets[0]), net_names[1], ", ".join(nets[1]))
    if nc:
        source += "    nc: " + ", ".join(nc) + "\n"
    source += "    layout {\n        board_outline: \"mechanical/outline.dxf\"\n" + "\n".join(layout) + "\n    }\n}\n"
    dependencies = "\n".join('%s = "%s"' % (k, v) for k, v in sorted(manifest["dependencies"].items()))
    (root / "cohdl.toml").write_text('[package]\nname = "%s"\nversion = "0.1.0"\nlicense = "MIT"\n\n[design]\ntop = "%s"\n\n[dependencies]\n%s\n' % (manifest["name"], design, dependencies), encoding="utf-8")
    if manifest.get("outline"):
        outline = Path(manifest["outline"])
        if not outline.is_absolute(): outline = Path(__file__).resolve().parents[1] / outline
        shutil.copyfile(outline, root / "mechanical/outline.dxf")
    else:
        x0, y0, x1, y1 = bounds
        dxf = "0\nSECTION\n2\nENTITIES\n0\nLWPOLYLINE\n8\nEdge.Cuts\n90\n4\n70\n1\n"
        for x, y in ((x0,y0),(x1,y0),(x1,y1),(x0,y1)):
            dxf += "10\n%s\n20\n%s\n" % (mm(x), mm(y))
        (root / "mechanical/outline.dxf").write_text(dxf + "0\nENDSEC\n", encoding="utf-8")
    (root / "src/main.cohdl").write_text(source, encoding="utf-8")
    return placements


def run(argv):
    result = subprocess.run(argv, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if result.returncode:
        raise BoardError("command failed (%d): %s\n%s" % (result.returncode, " ".join(argv), result.stdout))
    return result.stdout


def board_sexpr(text):
    tokens = re.findall(r'\(|\)|"(?:\\.|[^"\\])*"|[^\s()]+', text)
    stack = [[]]
    for token in tokens:
        if token == "(":
            child = []
            stack[-1].append(child)
            stack.append(child)
        elif token == ")":
            stack.pop()
        elif token.startswith('"'):
            stack[-1].append(json.loads(token))
        else:
            stack[-1].append(token)
    if len(stack) != 1:
        raise BoardError("unbalanced kicad_pcb")
    return stack[0][0]


def children(node, name):
    return [n for n in node if isinstance(n, list) and n[0] == name]


def set_place_origin(board_path, layout_path):
    """Put KiCad's drill/place origin at the outline's lower-left corner.

    The compiler's CPL measures from that corner (RFC-034 section 3.1), so the
    Gerbers and drill files must use it too, or the assembler's preview shifts
    every part by the outline's offset.  Returns the origin in board units.
    """
    outline = json.loads(layout_path.read_text(encoding="utf-8"))["board_outline"]
    points = [outline["start"]] + [s["to"] for s in outline["segments"] if "to" in s]
    origin = (min(x for x, _ in points), max(y for _, y in points))
    text = board_path.read_text(encoding="utf-8")
    entry = "(aux_axis_origin %s %s)" % (mm(origin[0]), mm(origin[1]))
    if "(aux_axis_origin" in text:
        text = re.sub(r"\(aux_axis_origin [^)]*\)", entry, text, count=1)
    elif "(setup" in text:
        text = text.replace("(setup", "(setup\n\t\t" + entry, 1)
    else:
        raise BoardError("kicad_pcb has no setup section for the place origin")
    board_path.write_text(text, encoding="utf-8")
    return origin


def checklist(board_path, lock_path, design, placements, output):
    board = board_sexpr(board_path.read_text(encoding="utf-8"))
    # design.lock is the compiler's instance -> designator record; the prefix
    # comes from the device's trait, not from the row name.
    designators = tomllib.loads(lock_path.read_text(encoding="utf-8")).get("designators", {})
    by_instance = {"%s::%s_%d" % (design, p["row"]["name"], p["index"]): p for p in placements}
    authored = {ref: by_instance[inst] for inst, ref in designators.items() if inst in by_instance}
    lines = ["# JLC frame alignment checklist", "", "Check every row against the JLC preview before ordering. Angles are CoHDL authoring angles; KiCad's bottom encoding is shown separately.", "",
             "| Ref | LCSC PN | Side | Authoring angle | KiCad angle | Expected pin 1 direction |", "|---|---|---|---:|---:|---|"]
    for fp in children(board, "footprint"):
        ref = next(p[2] for p in children(fp, "property") if p[1] == "Reference")
        bottom = next(c for c in children(fp, "layer"))[1] == "B.Cu"
        angle = next(c for c in children(fp, "at"))
        angle = float(angle[3]) if len(angle) > 3 else 0
        place = authored.get(ref)
        if place is None:
            raise BoardError("design.lock has no frame placement for " + ref)
        lines.append("| %s | %s | %s | %s | %s | verify supplier pad 1 against source footprint |" %
                     (ref, place["row"]["pn"], place["side"], mm(place["rotate"]), mm(angle)))
    output.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("manifest", type=Path)
    ap.add_argument("-o", "--output", type=Path, required=True)
    ap.add_argument("--spacing-mm", type=float, default=5)
    ap.add_argument("--cohdl", default="cohdl")
    ap.add_argument("--kicad-cli", default="kicad-cli")
    ap.add_argument("--skip-kicad", action="store_true")
    args = ap.parse_args(argv)
    if args.spacing_mm <= 0:
        ap.error("--spacing-mm must be positive")
    try:
        manifest = load_manifest(args.manifest)
        placements = emit_project(manifest, args.output, args.spacing_mm)
        name = manifest["name"]
        run([args.cohdl, "build", str(args.output), "--supplier", "lcsc", "--bottom-rotation", "lcsc=mirrored", "--emit", "kicad_pcb"])
        out = args.output / "out"
        board = out / (name + ".kicad_pcb")
        set_place_origin(board, out / (name + "-layout.json"))
        checklist(board, args.output / "design.lock", manifest["design"], placements,
                  args.output / "CHECKLIST.md")
        if not args.skip_kicad:
            fabrication = args.output / "fabrication"
            fabrication.mkdir(exist_ok=True)
            version = run([args.kicad_cli, "--version"]).strip()
            (fabrication / "kicad-cli-version.txt").write_text(version + "\n", encoding="utf-8")
            run([args.kicad_cli, "pcb", "export", "gerbers", "--use-drill-file-origin", "--output", str(fabrication), str(board)])
            run([args.kicad_cli, "pcb", "export", "drill", "--output", str(fabrication), "--excellon-zeros-format", "decimal", "--drill-origin", "plot", str(board)])
            hashes = []
            for artifact in sorted(fabrication.iterdir()):
                if artifact.is_file() and artifact.name != "SHA256SUMS":
                    hashes.append(hashlib.sha256(artifact.read_bytes()).hexdigest() + "  " + artifact.name)
            (fabrication / "SHA256SUMS").write_text("\n".join(hashes) + "\n", encoding="utf-8")
        print("generated %d placements in %s" % (len(placements), args.output))
    except (BoardError, OSError, json.JSONDecodeError) as exc:
        print("jlc_frame_board.py: " + str(exc), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
