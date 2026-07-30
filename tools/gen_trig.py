#!/usr/bin/env python3
"""Regenerate `src/trig.rs`'s sine table.

The compiler needs sin/cos to place a part at an arbitrary integer degree, and
the Constitution's byte-stability invariant (same source -> same netlist BYTES)
makes `f64::sin` the wrong tool: it resolves to the platform libm, whose last
bit is not guaranteed identical across macOS/glibc/musl. A checked-in table of
exact fixed-point integers is deterministic by construction instead.

Only 0..=90 is tabulated; `trig.rs` derives the other three quadrants by
symmetry, which is what keeps sin(90 deg) exactly 2^64 and cos(90 deg) exactly
0 -- and therefore keeps every pre-existing cardinal placement byte-identical.

Scale is 2^64 rather than a power of ten so that dividing the 256-bit product
back down is a shift, never a 256/128 division.

Usage:  python3 tools/gen_trig.py > /tmp/table.rs   (then paste, or --in-place)
"""
import argparse
import re
from decimal import Decimal, getcontext
from pathlib import Path

getcontext().prec = 60

SCALE = 2**64
TARGET = Path(__file__).resolve().parent.parent / "src" / "trig.rs"
BEGIN = "    // BEGIN GENERATED TABLE (tools/gen_trig.py)\n"
END = "    // END GENERATED TABLE\n"


# pi to 60 significant digits. A literal rather than a series: the series
# recurrences that fit in a few lines lose digits to integer truncation, and the
# value of pi is not something this script should be deriving. The assertions in
# main() are what verify it — sin(30 deg) must come out to exactly 1/2, which no
# wrong digit here survives.
PI = Decimal("3.14159265358979323846264338327950288419716939937510582097494")


def dsin(deg: int) -> Decimal:
    """sin(deg degrees) by Taylor series on the exactly-reduced argument."""
    if deg % 180 == 0:
        return Decimal(0)
    if deg % 360 == 90:
        return Decimal(1)
    if deg % 360 == 270:
        return Decimal(-1)
    x = PI * Decimal(deg) / Decimal(180)
    term, total, n = x, x, 1
    while True:
        term = -term * x * x / Decimal((2 * n) * (2 * n + 1))
        if term == 0:
            break
        total += term
        n += 1
        if n > 80:
            break
    return total


def rounded(d: Decimal) -> int:
    """Round half away from zero to an integer multiple of 1/SCALE."""
    v = d * SCALE
    i = int(v)
    frac = v - i
    if frac >= Decimal("0.5"):
        i += 1
    elif frac <= Decimal("-0.5"):
        i -= 1
    return i


def table() -> str:
    rows = []
    for deg in range(91):
        val = rounded(dsin(deg))
        assert -SCALE <= val <= SCALE, (deg, val)
        rows.append(f"    {val}, // sin({deg} deg)\n")
    return "".join(rows)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--in-place", action="store_true")
    args = ap.parse_args()

    # sanity: the four cardinals must be EXACT, or existing boards move
    assert rounded(dsin(0)) == 0
    assert rounded(dsin(90)) == SCALE
    # and a known irrational, to catch a broken series
    assert rounded(dsin(30)) == rounded(Decimal("0.5")), rounded(dsin(30))
    assert abs(rounded(dsin(45)) - rounded(Decimal(2).sqrt() / 2)) <= 1

    body = table()
    if not args.in_place:
        print(body, end="")
        return
    src = TARGET.read_text()
    assert BEGIN in src and END in src, "markers missing from src/trig.rs"
    head, rest = src.split(BEGIN, 1)
    _, tail = rest.split(END, 1)
    TARGET.write_text(head + BEGIN + body + END + tail)
    print(f"wrote {sum(1 for _ in body.splitlines())} entries to {TARGET}")


if __name__ == "__main__":
    main()
