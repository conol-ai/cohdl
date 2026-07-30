#!/usr/bin/env python3
"""Check every generated Yageo part number against Yageo's own specsheet
endpoint, and record the verdict in tools/passive_data/yageo_verified.json.

    GET https://yageogroup.com/component-documentation/download/specsheet/<MPN>?lang=en
      200 -> Yageo publishes a specsheet for that exact part number
      404 -> no such part

A 200 is what the generator treats as licence to emit. A 404 is not absolute
proof of non-existence (a part can be stocked by a distributor and missing
from this endpoint), so the effect of a 404 is a part omitted, never a part
asserted — the catalog errs small, not wrong.

Usage:
    python3 tools/verify_passive_mpns.py            # capacitors + a resistor sample
    python3 tools/verify_passive_mpns.py --all      # every part (slow, ~10k requests)
"""

import concurrent.futures
import json
import pathlib
import random
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SRC = ROOT / "lib" / "passive" / "src"
OUT = ROOT / "tools" / "passive_data" / "yageo_verified.json"
URL = "https://yageogroup.com/component-documentation/download/specsheet/"
WORKERS = 12
SAMPLE = 120  # resistors sampled when not running --all


def primaries():
    found = []
    for path in sorted(SRC.glob("*.cohdl")):
        for m in re.finditer(r'primary \{ mfr: "Yageo", mpn: "([^"]+)"', path.read_text()):
            found.append(m.group(1))
    return found


def probe(mpn):
    out = subprocess.run(
        # HEAD: the status code is the whole answer, and a specsheet is a
        # ~1 MB PDF nobody needs downloaded ten thousand times.
        [
            "curl",
            "-s",
            "-I",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "--max-time",
            "30",
            URL + mpn + "?lang=en",
        ],
        capture_output=True,
        text=True,
    ).stdout.strip()
    return mpn, out == "200"


def main():
    every = "--all" in sys.argv
    mpns = primaries()
    caps = [m for m in mpns if m.startswith("CC")]
    res = [m for m in mpns if m.startswith("RC")]
    if not every:
        random.seed(0)
        res = sorted(random.sample(res, min(SAMPLE, len(res))))
    targets = sorted(set(caps + res))
    print(f"probing {len(targets)} part numbers ({len(caps)} capacitors, {len(res)} resistors)")

    verdicts = {}
    if OUT.is_file():
        verdicts = json.loads(OUT.read_text()).get("verdicts", {})
    with concurrent.futures.ThreadPoolExecutor(max_workers=WORKERS) as pool:
        for i, (mpn, ok) in enumerate(pool.map(probe, targets), 1):
            verdicts[mpn] = ok
            if i % 200 == 0:
                print(f"  {i}/{len(targets)}")

    bad = sorted(m for m, ok in verdicts.items() if not ok)
    OUT.write_text(
        json.dumps(
            {
                "_provenance": {
                    "source": "GET https://yageogroup.com/component-documentation/download/specsheet/<MPN>?lang=en",
                    "meaning": "true = Yageo publishes a specsheet for this exact part number. The generator emits a capacitor only when its part number is true here; false means omitted, never asserted.",
                    "checked": len(verdicts),
                    "rejected": len(bad),
                },
                "verdicts": dict(sorted(verdicts.items())),
            },
            indent=1,
        )
        + "\n"
    )
    print(f"{len(verdicts) - len(bad)}/{len(verdicts)} resolved; {len(bad)} rejected")
    for m in bad[:20]:
        print("  reject", m)


if __name__ == "__main__":
    main()
