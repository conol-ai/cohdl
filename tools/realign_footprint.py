#!/usr/bin/env python3
"""Move a footprint's frame onto the supplier's: a rigid source rewrite.

    realign_footprint.py FILE FOOTPRINT --rotate R [--dx MM --dy MM]

`(dx, dy, R)` is the RFC-034 section 2 mapping that the footprint needed
before the move: the supplier's origin at (dx, dy) in our local frame, its
0 degree axis at R.  After the move the supplier frame IS the footprint
frame, so the mapping becomes (0, 0, 0).  Every local point p becomes
Rot(-R)(p - (dx, dy)), with the compiler's rotation sense (KiCad's: counter-
clockwise on screen, +y down); a pad's own rotation becomes (a - R) mod 360
and a rectangular courtyard swaps its size at 90 and 270 degrees.  Pad
numbers, pad symbols and land sizes never change.

Consumers keep their copper by composing each `place` with the same mapping
(`tools/realign_place.py`).  R must be a multiple of 90, so the rewrite is
exact.  Lines the tool does not understand inside the footprint body are an
error, never copied unchanged.
"""

import argparse
from decimal import Decimal
import re
import sys
from pathlib import Path

NUM = r"[+-]?(?:\d+(?:\.\d*)?|\.\d+)"
POINT = re.compile(r"\((" + NUM + r")mm,\s*(" + NUM + r")mm\)")
PAD = re.compile(r"^(?P<head>\s*pad \S+: \S+ at )\((?P<x>" + NUM + r")mm, (?P<y>" + NUM + r")mm\)(?: rotate (?P<a>\d+))?(?P<tail>\s*(?://.*)?)$")
COURTYARD = re.compile(r"^(?P<head>\s*courtyard \{ shape: rect, at: )\((?P<x>" + NUM + r")mm, (?P<y>" + NUM + r")mm\), size: \((?P<w>" + NUM + r")mm, (?P<h>" + NUM + r")mm\)(?P<tail> \}\s*)$")
REF = re.compile(r"^\s*silkscreen_ref \{ at: \(" + NUM + r"mm, " + NUM + r"mm\) \}\s*$")
CIRCLE = re.compile(r"^\s*circle at \(" + NUM + r"mm, " + NUM + r"mm\) radius .*$")
POLYGON = re.compile(r"^\s*polygon \[.*$")
POLYGON_MORE = re.compile(r"^\s*(?:\(" + NUM + r"mm, " + NUM + r"mm\),?\s*)+\]?\s*$")
MOUNT = re.compile(r"^(?P<head>\s*mount_hole \S+: \w+ at )\((?P<x>" + NUM + r")mm, (?P<y>" + NUM + r")mm\)(?P<tail> diameter " + NUM + r"mm\s*)$")
LINE = re.compile(r"^\s*line from \(" + NUM + r"mm, " + NUM + r"mm\) to \(" + NUM + r"mm, " + NUM + r"mm\) .*$")
KEEP = re.compile(r"^\s*(?://.*|silkscreen \{|\}|pin_1_marker near pad \S+ shape \w+|polarity_marker cathode_pin \S+ shape \w+)?\s*$")


class RealignError(Exception):
    pass


def fmt(value):
    if value == 0:
        return "0mm"
    text = format(value.normalize(), "f")
    return text + "mm"


def rotate_point(x, y, r):
    """Rot(-r) in the compiler's sense: Rot(t)(x, y) = (x cos t + y sin t, -x sin t + y cos t)."""
    return {0: (x, y), 90: (-y, x), 180: (-x, -y), 270: (y, -x)}[r]


def footprint_span(text, name):
    m = re.search(r"(?m)^(?:pub )?footprint " + re.escape(name) + r" \{\n", text)
    if not m:
        raise RealignError("no footprint %s" % name)
    if len(re.findall(r"(?m)^(?:pub )?footprint " + re.escape(name) + r" \{\n", text)) != 1:
        raise RealignError("footprint %s is declared more than once" % name)
    end = text.find("\n}\n", m.end() - 1)
    if end < 0:
        raise RealignError("footprint %s has no closing brace at column 0" % name)
    return m.end(), end + 1


def transform(text, name, rotate, dx=Decimal(0), dy=Decimal(0)):
    if rotate % 90 or not 0 <= rotate < 360:
        raise RealignError("rotate must be 0, 90, 180 or 270")
    start, end = footprint_span(text, name)

    def point(m):
        x, y = rotate_point(Decimal(m.group(1)) - dx, Decimal(m.group(2)) - dy, rotate)
        return "(%s, %s)" % (fmt(x), fmt(y))

    out, in_polygon = [], False
    for number, line in enumerate(text[start:end].split("\n"), 1):
        if in_polygon or POLYGON.match(line):
            if in_polygon and not POLYGON_MORE.match(line):
                raise RealignError("%s line %d: unexpected polygon continuation: %s" % (name, number, line.strip()))
            out.append(POINT.sub(point, line))
            in_polygon = "]" not in line
            continue
        m = PAD.match(line)
        if m:
            x, y = rotate_point(Decimal(m["x"]) - dx, Decimal(m["y"]) - dy, rotate)
            angle = (int(m["a"] or 0) - rotate) % 360
            out.append("%s(%s, %s)%s%s" % (m["head"], fmt(x), fmt(y), " rotate %d" % angle if angle else "", m["tail"]))
            continue
        m = COURTYARD.match(line)
        if m:
            x, y = rotate_point(Decimal(m["x"]) - dx, Decimal(m["y"]) - dy, rotate)
            w, h = (m["h"], m["w"]) if rotate in (90, 270) else (m["w"], m["h"])
            out.append("%s(%s, %s), size: (%smm, %smm)%s" % (m["head"], fmt(x), fmt(y), w, h, m["tail"]))
            continue
        m = MOUNT.match(line)
        if m:
            x, y = rotate_point(Decimal(m["x"]) - dx, Decimal(m["y"]) - dy, rotate)
            out.append("%s(%s, %s)%s" % (m["head"], fmt(x), fmt(y), m["tail"]))
            continue
        if REF.match(line) or CIRCLE.match(line) or LINE.match(line):
            out.append(POINT.sub(point, line))
            continue
        if KEEP.match(line):
            out.append(line)
            continue
        raise RealignError("%s line %d: cannot move: %s" % (name, number, line.strip()))
    if in_polygon:
        raise RealignError("%s: unterminated polygon" % name)
    return text[:start] + "\n".join(out) + text[end:]


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("file", type=Path)
    ap.add_argument("footprint")
    ap.add_argument("--rotate", type=int, required=True)
    ap.add_argument("--dx", type=Decimal, default=Decimal(0), help="mm")
    ap.add_argument("--dy", type=Decimal, default=Decimal(0), help="mm")
    args = ap.parse_args(argv)
    try:
        text = args.file.read_text(encoding="utf-8")
        args.file.write_text(transform(text, args.footprint, args.rotate, args.dx, args.dy), encoding="utf-8")
    except (RealignError, OSError) as exc:
        print("realign_footprint.py: %s" % exc, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
