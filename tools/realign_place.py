#!/usr/bin/env python3
"""Move literal CoHDL placements so supplier-mapped copper stays in place.

Input is JSON ``{"changes":[{"package":"pkg", "footprint":"FP",
"dx_mm":2, "dy_mm":1, "rotate":90}]}``, or an RFC-034 compiler map whose
``designators`` rows supply ``footprint``, ``at`` and ``rotate``.  ``--reverse``
undoes a previous forward run.  Ordinary ``place name`` and
``place mounting[0]`` forms are supported, including array indices and either
``rotate``/``side`` clause order.  Expression coordinates/rotations, layout
``for`` loops and subdesign declarations are never rewritten: every requested
unsupported target is reported as ``file:line: reason`` before any write.

This is the inverse of RFC-034 compose_place.  Translation and KiCad rotation
use :mod:`smt_pos_map` (including its exact decimal and nm rounding), rather
than a second geometric convention.  Verify the result by rebuilding and
comparing compiler-emitted ``.kicad_pcb`` pad coordinates, not Gerbers.
"""

import argparse
from decimal import Decimal, ROUND_HALF_UP
import json
import math
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import smt_pos_map as sm

FEMTO_PER_NM = 1_000_000_000
NUM = r"[+-]?(?:\d+(?:\.\d*)?|\.\d+)"
LIT = NUM + r"mm"
TARGET = r"[A-Za-z_][A-Za-z0-9_]*(?:\[\d+\])?"
PLACE = re.compile(r"(?m)(?P<indent>^[ \t]*|(?<=[{;])[ \t]*)place[ \t]+(?P<target>" + TARGET + r")[ \t]+at[ \t]*\((?P<body>[^\n)]*)\)(?P<tail>(?:[ \t]+(?:rotate[ \t]+[^\s]+|side[ \t]+(?:top|bottom))){0,2})(?=[ \t]*(?:\n|$))")


def mm_femto(value):
    if value % (10 ** 15) == 0:
        return str(value // 10 ** 15)
    sign = "-" if value < 0 else ""
    digits = str(abs(value)).rjust(16, "0")
    return sign + (digits[:-15] + "." + digits[-15:]).rstrip("0")


def literal_femto(text):
    if not re.fullmatch(LIT, text.strip()):
        raise ValueError("coordinates and rotation must be decimal literals")
    return int((Decimal(text.strip()[:-2]) * 10 ** 15).to_integral_value(rounding=ROUND_HALF_UP))


def read_changes(path):
    doc = json.loads(path.read_text(encoding="utf-8"))
    if isinstance(doc, dict) and "changes" in doc:
        rows = doc["changes"]
    elif isinstance(doc, dict) and "designators" in doc:
        rows = list(doc["designators"].values())
    else:
        raise ValueError("change table must contain changes or compiler-map designators")
    changes = {}
    for i, row in enumerate(rows):
        if not isinstance(row, dict):
            raise ValueError("change row %d is not an object" % i)
        fp = str(row.get("footprint", ""))
        if not fp:
            raise ValueError("change row %d has no footprint" % i)
        if "at" in row:
            dx, dy = sm.parse_mm_nm(row["at"][0]), sm.parse_mm_nm(row["at"][1])
            rotate = row.get("rotate", 0)
        else:
            dx = int(Decimal(str(row.get("dx_mm", 0))) * 10 ** 6)
            dy = int(Decimal(str(row.get("dy_mm", 0))) * 10 ** 6)
            rotate = row.get("rotate", row.get("r", 0))
        if isinstance(rotate, bool) or not isinstance(rotate, int) or not 0 <= rotate < 360:
            raise ValueError("change row %d rotate must be an integer in 0..359" % i)
        if fp in changes and changes[fp] != (dx, dy, rotate):
            raise ValueError("conflicting mappings for " + fp)
        changes[fp] = (dx, dy, rotate)
    return changes


def layout_regions(source):
    """Return layout and for-loop brace regions, ignoring strings/comments."""
    regions = []
    for match in re.finditer(r"\b(layout|for)\b", source):
        brace = source.find("{", match.end())
        if brace < 0 or "\n" in source[match.end():brace]:
            continue
        depth, i = 0, brace
        while i < len(source):
            if source[i] == "{": depth += 1
            elif source[i] == "}":
                depth -= 1
                if depth == 0: break
            i += 1
        regions.append((match.group(1), match.start(), i))
    return regions


def subdesign_lines(source):
    lines = set()
    # Declarations are top-level; brace matching ignores braces in comments and strings.
    for match in re.finditer(r"(?m)^[ \t]*(?:pub[ \t]+)?subdesign[ \t]+[A-Za-z_]", source):
        brace = source.find("{", match.end())
        if brace < 0:
            continue
        depth, i = 0, brace
        while i < len(source):
            if source[i] == "{": depth += 1
            elif source[i] == "}":
                depth -= 1
                if depth == 0: break
            i += 1
        start = source.count("\n", 0, match.start()) + 1
        lines.update(range(start, source.count("\n", 0, i) + 2))
    return lines


def parse_statement(match, line, source):
    target = match.group("target")
    body = match.group("body")
    if body.count(",") != 1:
        raise ValueError("place coordinates must be two decimal mm literals")
    x_text, y_text = body.split(",")
    x, y = literal_femto(x_text), literal_femto(y_text)
    tail = match.group("tail")
    rotate_matches = list(re.finditer(r"\brotate\s+([^\s]+)(?=\s|$)", tail))
    if len(rotate_matches) > 1:
        raise ValueError("duplicate rotate clause")
    rotate = int(rotate_matches[0].group(1)) if rotate_matches else 0
    if not 0 <= rotate < 360:
        raise ValueError("rotation must be a literal integer in 0..359")
    if re.sub(r"\brotate\s+[^\s]+(?=\s|$)", "", tail).strip() not in ("", "side top", "side bottom"):
        raise ValueError("unsupported place clauses")
    bottom = bool(re.search(r"\bside\s+bottom\b", tail))
    for_start = source.rfind("for ", 0, match.start())
    layout_start = source.rfind("layout", 0, match.start())
    if for_start > layout_start and for_start >= 0:
        raise ValueError("place in a for loop is not rewritten")
    return {"target": target, "x": x, "y": y, "rotate": rotate, "bottom": bottom}


def realign(source, changes, footprints, reverse=False):
    errors, edits, changed = [], [], []
    unsupported = subdesign_lines(source)
    regions = layout_regions(source)
    for match in re.finditer(r"\bplace\s+(" + TARGET + r")\s+at\b", source):
        target = match.group(1)
        if target not in footprints:
            continue
        line = source.count("\n", 0, match.start()) + 1
        if line in unsupported:
            errors.append((line, "place inside subdesign is not rewritten"))
            continue
        if any(kind == "for" and start < match.start() < end for kind, start, end in regions):
            errors.append((line, "place in a for loop is not rewritten"))
            continue
        if not any(start < match.start() < end for kind, start, end in regions if kind == "layout"):
            continue
        if not re.match(r"(?P<indent>[ \t]*)place[ \t]+" + TARGET + r"[ \t]+at[ \t]*\((?P<body>[^\n)]*)\)(?P<tail>(?:[ \t]+(?:rotate[ \t]+[^\s]+|side[ \t]+(?:top|bottom))){0,2})(?=[ \t]*(?:\n|$))", source[match.start():]):
            errors.append((line, "only single-line literal place statements are rewritten"))
    for match in PLACE.finditer(source):
        target = match.group("target")
        line = source.count("\n", 0, match.start()) + 1
        fp = footprints.get(target)
        if line in unsupported or any(kind == "for" and start < match.start() < end for kind, start, end in regions):
            continue
        if fp is None:
            continue
        try:
            place = parse_statement(match, line, source)
            dx, dy, rotate = changes[fp]
            if reverse:
                nr = (place["rotate"] - rotate if place["bottom"] else place["rotate"] + rotate) % 360
                sign = 1
                angle = nr
            else:
                nr = (place["rotate"] + rotate if place["bottom"] else place["rotate"] - rotate) % 360
                sign = -1
                angle = place["rotate"]
            # compose_place(instance, mapping) rotates the local mapping by the
            # inverse angle in KiCad's +y-down frame. recover_local_nm performs
            # that same inverse; undo the bottom reflection first.
            local = (-dx, dy) if place["bottom"] else (dx, dy)
            rx, ry = sm.recover_local_nm(local, (0, 0), angle)
            nx, ny = place["x"] + sign * rx * FEMTO_PER_NM, place["y"] + sign * ry * FEMTO_PER_NM
            tail = match.group("tail")
            tail = re.sub(r"\brotate\s+" + NUM + r"(?=\s|$)", lambda m: "rotate " + str(nr) if nr else "", tail)
            if not re.search(r"\brotate\b", tail) and nr:
                side = re.search(r"\bside\s+(?:top|bottom)\b", tail)
                at = side.start() if side else len(tail)
                tail = tail[:at] + " rotate " + str(nr) + " " + tail[at:].lstrip()
            tail = re.sub(r"[ \t]+", " ", tail).rstrip()
            head = source[match.start():match.start("body")]
            replacement = head + mm_femto(nx) + "mm, " + mm_femto(ny) + "mm)" + tail
            edits.append((match.start(), match.end(), replacement))
            changed.append((target, fp))
        except (ValueError, sm.MapError) as exc:
            errors.append((line, str(exc)))
    return edits, sorted(errors), changed


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("project", type=Path, help="CoHDL project directory")
    ap.add_argument("changes", type=Path)
    ap.add_argument("--map", type=Path, help="compiler supplier map (resolves footprint by designator)")
    ap.add_argument("--board", type=Path, help="compiler .kicad_pcb (resolves footprint by designator)")
    ap.add_argument("--reverse", action="store_true")
    args = ap.parse_args(argv)
    try:
        changes = read_changes(args.changes)
        footprints = {}
        if args.map:
            doc = json.loads(args.map.read_text(encoding="utf-8"))
            for ref, row in doc.get("designators", {}).items():
                if row.get("footprint") in changes:
                    footprints[ref] = row["footprint"]
        if args.board:
            from jlc_frame_board import board_sexpr, children
            board = board_sexpr(args.board.read_text(encoding="utf-8"))
            short_to_full = {}
            for fp in changes:
                short_to_full.setdefault(fp.rsplit("::", 1)[-1], []).append(fp)
            for node in children(board, "footprint"):
                prop = next(p for p in children(node, "property") if p[1] == "Reference")
                short = node[1].split("-", 1)[-1]
                candidates = short_to_full.get(short, [])
                if len(candidates) == 1:
                    footprints[prop[2]] = candidates[0]
        sources = sorted((args.project / "src").glob("*.cohdl"))
        all_edits, errors = {}, []
        for path in sources:
            source = path.read_text(encoding="utf-8")
            edits, problems, _ = realign(source, changes, footprints, args.reverse)
            errors.extend("%s:%d: %s" % (path, line, msg) for line, msg in problems)
            all_edits[path] = (source, edits)
        if errors:
            for error in errors: print(error, file=sys.stderr)
            return 1
        for path, (source, edits) in all_edits.items():
            for start, end, replacement in reversed(edits):
                source = source[:start] + replacement + source[end:]
            if edits: path.write_text(source, encoding="utf-8")
        print("realigned %d placements" % sum(len(e) for _, e in all_edits.values()))
        return 0
    except (ValueError, OSError, KeyError, json.JSONDecodeError, sm.MapError) as exc:
        print("realign_place.py: " + str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
