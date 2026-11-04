#!/usr/bin/env python3
"""Realign literal CoHDL placements for a changed supplier footprint frame.

Input JSON is ``{"changes":[{"package":"pkg", "footprint":"pkg::FP",
"dx_mm":2,"dy_mm":1,"rotate":90}]}``.  ``--layout`` and ``--board`` must be the
compiler's layout JSON and KiCad PCB from the same build.  Layout
``placements[].instance`` is the expanded instance path (``x_0`` maps to source
``x[0]``); its ``at``/``rotate``/``side`` identifies the corresponding PCB
footprint block.  That block supplies the complete footprint name (PCB base
names replace ``::`` with ``-``).  Missing/ambiguous matches are errors.  Never
resolve a source target by refdes or supplier-map designator.

Only single-line decimal-mm literal placements inside the built design's own
``design NAME { … }`` body are changed; comments are ignored.  Expressions,
``for`` loops, subdesign declarations (generic ones included) and reach-in
paths are diagnosed as ``file:line: reason``; a same-named placement in another
design is left alone with a note.  Every rewritten placement must still equal
the layout's: a source already realigned, or edited since the build, is
refused, so a second forward run cannot compose the delta again.  Translation
uses the exact femto-mm fixed-point algorithm and checked-in sine table of
``src/trig.rs``.  Forward runs write an undo JSON containing original source
bytes and never overwrite an existing one; ``--reverse`` restores those bytes
exactly after checking that the project, the change table and the files are
the ones the forward run recorded, then removes the undo log.
"""

import argparse
from decimal import Decimal, ROUND_HALF_UP
import hashlib
import json
import os
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
    sign = "-" if value < 0 else ""
    digits = str(abs(value)).rjust(16, "0")
    return sign + (digits[:-15] + "." + digits[-15:]).rstrip("0").rstrip(".")


def literal_femto(value):
    if not re.fullmatch(LIT, value.strip()):
        raise ValueError("coordinates must be decimal mm literals")
    return int((Decimal(value.strip()[:-2]) * 10 ** 15).to_integral_value(rounding=ROUND_HALF_UP))


def compiler_rotate(x, y, degrees):
    """Exact Python projection of src/trig.rs, including product rounding."""
    source = (Path(__file__).resolve().parents[1] / "src/trig.rs").read_text(encoding="utf-8")
    body = source.split("// BEGIN GENERATED TABLE (tools/gen_trig.py)", 1)[1].split("// END GENERATED TABLE", 1)[0]
    table = [int(v) for v in re.findall(r"^\s*(\d+),\s*//", body, re.M)]
    if len(table) != 91:
        raise ValueError("cannot read compiler sine table")
    d = degrees % 360
    if d < 90:
        sine, cosine = table[d], table[90 - d]
    elif d < 180:
        sine, cosine = table[180 - d], -table[d - 90]
    elif d < 270:
        sine, cosine = -table[d - 180], -table[270 - d]
    else:
        sine, cosine = -table[360 - d], table[d - 270]
    def product(a, b):
        magnitude = (abs(a) * abs(b) + (1 << 63)) >> 64
        return -magnitude if (a < 0) != (b < 0) else magnitude
    return product(x, cosine) - product(y, sine), product(x, sine) + product(y, cosine)


CHANGE_KEYS = {"package", "footprint", "dx_mm", "dy_mm", "rotate", "r"}


def read_changes(path):
    doc = json.loads(path.read_text(encoding="utf-8"))
    rows = doc.get("changes") if isinstance(doc, dict) else None
    if not isinstance(rows, list):
        raise ValueError("change table must contain a changes array")
    changes = {}
    for i, row in enumerate(rows):
        if not isinstance(row, dict):
            raise ValueError("change row %d is not an object" % i)
        unknown = sorted(set(row) - CHANGE_KEYS)
        if unknown:
            # A frame-data row spells its angle `rotate_deg`; silently dropping
            # it would report success with no rotation applied.
            raise ValueError("change row %d has unknown key(s): %s (expected %s)"
                             % (i, ", ".join(unknown), ", ".join(sorted(CHANGE_KEYS))))
        fp = row.get("footprint", "")
        if not fp:
            raise ValueError("change row %d has no footprint" % i)
        dx = int((Decimal(str(row.get("dx_mm", 0))) * 10 ** 6).to_integral_value(rounding=ROUND_HALF_UP))
        dy = int((Decimal(str(row.get("dy_mm", 0))) * 10 ** 6).to_integral_value(rounding=ROUND_HALF_UP))
        rotate = row.get("rotate", row.get("r", 0))
        if isinstance(rotate, bool) or not isinstance(rotate, int) or not 0 <= rotate < 360:
            raise ValueError("change row %d rotate must be integer 0..359" % i)
        value = (dx, dy, rotate)
        if fp in changes and changes[fp] != value:
            raise ValueError("conflicting mappings for " + fp)
        changes[fp] = value
    return changes


def code_only(source):
    return re.sub(r'//[^\n]*|"(?:\\.|[^"\\])*"', lambda m: "".join("\n" if c == "\n" else " " for c in m.group(0)), source)


def regions(source, keyword, name=None):
    clean = code_only(source)
    found = []
    if keyword == "for":
        pattern = r"\bfor\b"
    elif keyword == "subdesign":
        # A declaration, generic or not; a use site (`subdesign x: S { … }`)
        # has a colon after its name and never matches.
        pattern = r"(?m)^[ \t]*(?:pub[ \t]+)?subdesign[ \t]+[A-Za-z_][A-Za-z0-9_]*[ \t]*(?:<[^{}\n]*>)?[ \t]*\{"
    else:
        pattern = r"(?m)(?:^|(?<=[;}]))[ \t]*(?:pub[ \t]+)?design[ \t]+" + (
            re.escape(name) if name else r"[A-Za-z_][A-Za-z0-9_]*") + r"[ \t]*\{"
    for match in re.finditer(pattern, clean):
        # A declaration pattern ends on its own `{`; searching on from there
        # would take the first nested block (a use site, `layout`) as the body.
        brace = match.end() - 1 if keyword != "for" else clean.find("{", match.end())
        if brace < 0 or (keyword == "for" and "\n" in clean[match.end():brace]):
            continue
        depth, i = 0, brace
        while i < len(clean):
            if clean[i] == "{": depth += 1
            elif clean[i] == "}":
                depth -= 1
                if depth == 0: break
            i += 1
        found.append((match.start(), i))
    return found


def inside(position, spans):
    return any(start < position < end for start, end in spans)


def realign(source, changes, footprints, reverse=False, design=None, placed=None, notes=None):
    """Return (edits, errors, changed) for one source file.

    With ``design``, only placements inside that design's body are rewritten.
    With ``placed`` (target -> (x_femto, y_femto, rotate, bottom) from the
    build's layout), a placement that no longer equals the layout is refused.
    """
    errors, edits, changed = [], [], []
    clean = code_only(source)
    loops, subdesigns = regions(source, "for"), regions(source, "subdesign")
    body = regions(source, "design", design) if design else None
    seen = set()
    for match in re.finditer(r"\bplace\s+([A-Za-z_][A-Za-z0-9_.]*(?:\[\d+\])?)\s+at\b", clean):
        target = match.group(1)
        fp = footprints.get(target.rsplit(".", 1)[-1])
        if fp is None or inside(match.start(), loops):
            continue
        line = source.count("\n", 0, match.start()) + 1
        if body is not None and not inside(match.start(), body) and not inside(match.start(), subdesigns):
            seen.add(line)
            if notes is not None:
                notes.append((line, "place %s is outside design %s and is not rewritten" % (target, design)))
            continue
        if "." in target:
            seen.add(line)
            errors.append((line, "subdesign reach-in place is not rewritten"))
        elif inside(match.start(), subdesigns):
            seen.add(line)
            errors.append((line, "place inside subdesign is not rewritten"))
        elif not any(m.start() <= match.start() < m.end() for m in PLACE.finditer(clean)):
            seen.add(line)
            errors.append((line, "only single-line literal place statements are rewritten"))
    for match in PLACE.finditer(clean):
        target, line = match.group("target"), source.count("\n", 0, match.start()) + 1
        fp = footprints.get(target)
        if fp is None or line in seen or inside(match.start(), loops) or inside(match.start(), subdesigns):
            continue
        try:
            fields = match.group("body").split(",")
            if len(fields) != 2:
                raise ValueError("expected two literal coordinates")
            x, y = literal_femto(fields[0]), literal_femto(fields[1])
            tail = match.group("tail")
            angles = re.findall(r"\brotate\s+([^\s]+)(?=\s|$)", tail)
            if len(angles) > 1:
                raise ValueError("duplicate rotate clause")
            old_angle = int(angles[0]) if angles else 0
            if not 0 <= old_angle < 360:
                raise ValueError("rotation must be integer 0..359")
            bottom = bool(re.search(r"\bside\s+bottom\b", tail))
            if placed is not None and placed.get(target) != (x, y, old_angle, bottom):
                raise ValueError(
                    "place %s no longer matches the build's layout; rebuild from this source "
                    "before realigning (a placement already realigned is never realigned twice)" % target)
            dx, dy, rotate = changes[fp]
            dx, dy = dx * FEMTO_PER_NM, dy * FEMTO_PER_NM
            if reverse:
                # Inverse child translation: -B(-original_r) * original_d.
                bx, by = compiler_rotate(dx, dy, rotate)
                dx, dy = -bx, -by
                rotate = (360 - rotate) % 360
            local_x, local_y = (-dx if bottom else dx), dy
            rx, ry = compiler_rotate(local_x, local_y, (360 - old_angle) % 360)
            nx, ny = x + rx, y + ry
            angle = (old_angle - rotate if bottom else old_angle + rotate) % 360
            tail = re.sub(r"\brotate\s+" + NUM + r"(?=\s|$)", lambda m: "rotate " + str(angle) if angle else "", tail)
            if not re.search(r"\brotate\b", tail) and angle:
                side = re.search(r"\bside\s+(?:top|bottom)\b", tail)
                at = side.start() if side else len(tail)
                tail = tail[:at] + " rotate " + str(angle) + " " + tail[at:].lstrip()
            if angle == 0 and old_angle != 0:
                tail = re.sub(r"[ \t]+rotate[ \t]+" + NUM + r"(?=\s|$)", "", tail)
            tail = re.sub(r"[ \t]+", " ", tail).rstrip()
            replacement = source[match.start():match.start("body")] + mm_femto(nx) + "mm, " + mm_femto(ny) + "mm)" + tail
            edits.append((match.start(), match.end(), replacement))
            changed.append((target, fp))
        except (ValueError, sm.MapError) as exc:
            errors.append((line, str(exc)))
    for start, end in loops:
        for match in re.finditer(r"\bplace\s+(" + TARGET + r")\s+at\b", clean[start:end]):
            if match.group(1) in footprints:
                errors.append((source.count("\n", 0, start + match.start()) + 1, "place in a for loop is not rewritten"))
    return edits, sorted(set(errors)), changed


def compiler_footprints(layout_path, board_path, targets, changes):
    from jlc_frame_board import board_sexpr, children
    layout = json.loads(layout_path.read_text(encoding="utf-8"), parse_float=Decimal)
    board = board_sexpr(board_path.read_text(encoding="utf-8"))
    full_names = {name.replace("::", "-"): name for name in changes}
    rows = []
    for node in children(board, "footprint"):
        at = next(c for c in children(node, "at"))
        rows.append((full_names.get(node[1]), [Decimal(at[1]), Decimal(at[2])],
                     Decimal(at[3]) if len(at) > 3 else Decimal(0),
                     next(c for c in children(node, "layer"))[1] == "B.Cu"))
    result, unmatched, placed, designs = {}, [], {}, set()
    for placement in layout.get("placements", []):
        designs.add(placement["instance"].split("::", 1)[0])
        parts = placement["instance"].split("::", 1)[-1].split("::")
        # An element of `inst x: [D; N]` is laid out as x_0 but placed as x[0];
        # an ordinary instance may itself be named x_0.  A subdesign member
        # (a::b) is never a top-level target.
        name = "::".join(parts)
        array = re.sub(r"_(\d+)$", r"[\1]", name)
        target = None if len(parts) != 1 else name if name in targets else array if array in targets else None
        bottom = placement.get("side") == "bottom"
        angle = (placement.get("rotate", 0) + (180 if bottom else 0)) % 360
        matches = [row for row in rows if row[1] == [Decimal(str(v)) for v in placement["at"]]
                   and (row[2] + 360) % 360 == angle and row[3] == bottom]
        if len(matches) != 1:
            if any(row[0] is not None for row in rows):
                raise ValueError("compiler layout/PCB match for %s is not unique (%d)" % (target or name, len(matches)))
            continue
        if matches[0][0] is None:
            continue
        if target is None:
            # Laid out on a moved footprint, yet no top-level literal `place`
            # names it (a subdesign member, or a missing statement): skipping
            # it would silently move its copper.
            where = "it is placed inside a subdesign, which is not rewritten" if len(parts) != 1 \
                else "no `place %s at` statement names it" % name
            unmatched.append("%s uses moved footprint %s but %s" % (name, matches[0][0], where))
            continue
        result[target] = matches[0][0]
        placed[target] = (literal_femto("%smm" % placement["at"][0]), literal_femto("%smm" % placement["at"][1]),
                          placement.get("rotate", 0), bottom)
    if len(designs) > 1:
        raise ValueError("layout names more than one design: " + ", ".join(sorted(designs)))
    return result, unmatched, placed, next(iter(designs), None)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("project", type=Path)
    ap.add_argument("changes", type=Path)
    ap.add_argument("--layout", type=Path)
    ap.add_argument("--board", type=Path)
    ap.add_argument("--reverse", action="store_true")
    ap.add_argument("--undo-log", type=Path)
    args = ap.parse_args(argv)
    undo = args.undo_log or args.changes.with_suffix(args.changes.suffix + ".undo.json")
    try:
        project = str(args.project.resolve())
        if args.reverse:
            doc = json.loads(undo.read_text(encoding="utf-8"))
            if doc.get("project") != project:
                raise ValueError("undo log %s belongs to project %s, not %s" % (undo, doc.get("project"), project))
            if doc.get("changes_sha256") != hashlib.sha256(args.changes.read_bytes()).hexdigest():
                raise ValueError("change table %s differs from the one the forward run used" % args.changes)
            # A logged path is a file name in the logged project's src/, never
            # a path from the working directory the forward run happened in.
            src = Path(project) / "src"
            paths = []
            for item in doc["files"]:
                if not isinstance(item["path"], str) or not item["path"].endswith(".cohdl") or \
                        Path(item["path"]).name != item["path"]:
                    raise ValueError("undo log %s names %r, which is not a source file of %s" % (undo, item["path"], project))
                paths.append(src / item["path"])
            for item, path in zip(doc["files"], paths):
                content = path.read_bytes()
                # A file already back at its original source is from an earlier
                # reverse that stopped part way; reversing again resumes it.
                if hashlib.sha256(content).hexdigest() != item["after_sha256"] and \
                        content != item["source"].encode("utf-8"):
                    raise ValueError("refusing reverse after external edit: " + str(path))
            for item, path in zip(doc["files"], paths):
                path.write_text(item["source"], encoding="utf-8")
            undo.unlink()
            print("realigned %d placements" % doc["count"])
            return 0
        if not args.layout or not args.board:
            raise ValueError("forward realignment requires --layout and --board")
        if undo.exists():
            raise ValueError("undo log %s already exists from an earlier forward run; run --reverse first, "
                             "or pass a new --undo-log once that run is known to be wanted" % undo)
        changes = read_changes(args.changes)
        paths = sorted((args.project / "src").glob("*.cohdl"))
        originals = {p: p.read_text(encoding="utf-8") for p in paths}
        targets = {m.group(1) for source in originals.values()
                   for m in re.finditer(r"\bplace\s+(" + TARGET + r")\s+at\b", source)}
        footprints, unmatched, placed, design = compiler_footprints(args.layout, args.board, targets, changes)
        rewritten, errors, undo_files = {}, ["%s: %s" % (args.project, item) for item in unmatched], []
        done = set()
        for path, source in originals.items():
            notes = []
            edits, problems, changed = realign(source, changes, footprints, design=design, placed=placed, notes=notes)
            done.update(target for target, _ in changed)
            errors.extend("%s:%d: %s" % (path, line, msg) for line, msg in problems)
            for line, msg in notes:
                print("%s:%d: note: %s" % (path, line, msg), file=sys.stderr)
            for start, end, replacement in reversed(edits):
                source = source[:start] + replacement + source[end:]
            rewritten[path] = source
            if edits:
                undo_files.append({"path": path.name, "source": originals[path], "count": len(edits),
                                   "after_sha256": hashlib.sha256(source.encode()).hexdigest()})
        # Every laid-out target on a moved footprint must have been rewritten:
        # one skipped without a diagnostic would silently move its copper.
        errors.extend("%s: %s uses moved footprint %s but was not rewritten" % (args.project, target, footprints[target])
                      for target in sorted(set(footprints) - done))
        if errors:
            for error in errors: print(error, file=sys.stderr)
            return 1
        count = sum(item["count"] for item in undo_files)
        # The undo log is complete on disk before any source changes: a log that
        # cannot be written (missing or read-only directory) leaves the sources
        # untouched, and a failed source write restores the ones already written.
        partial = undo.with_name(undo.name + ".partial")
        try:
            partial.write_text(json.dumps({"project": project, "count": count,
                                           "changes_sha256": hashlib.sha256(args.changes.read_bytes()).hexdigest(),
                                           "files": undo_files}, indent=2) + "\n", encoding="utf-8")
            os.replace(partial, undo)
        except OSError:
            partial.unlink(missing_ok=True)
            raise
        written = []
        try:
            for path, source in rewritten.items():
                if source != originals[path]:
                    written.append(path)
                    path.write_text(source, encoding="utf-8")
        except OSError:
            for path in written:
                path.write_text(originals[path], encoding="utf-8")
            undo.unlink()
            raise
        print("realigned %d placements" % count)
        return 0
    except (ValueError, OSError, KeyError, json.JSONDecodeError, sm.MapError) as exc:
        print("realign_place.py: " + str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
