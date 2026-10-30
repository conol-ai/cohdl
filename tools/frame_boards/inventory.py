"""Index library footprints: users (part, device type, required pins) and courtyard.

Usage: python3 -I tools/frame_boards/inventory.py OUT.json
"""
import json, re, sys
from pathlib import Path
LIB = Path(__file__).resolve().parents[2] / "lib"

def strip(t):
    return re.sub(r"//[^\n]*", "", t)

def block(t, i):
    depth = 0
    for j in range(i, len(t)):
        if t[j] == "{": depth += 1
        elif t[j] == "}":
            depth -= 1
            if depth == 0: return t[i:j + 1]
    return t[i:]

pkg_of = {}
for toml in LIB.glob("*/cohdl.toml"):
    m = re.search(r'^name\s*=\s*"([^"]+)"', toml.read_text(), re.M)
    v = re.search(r'^version\s*=\s*"([^"]+)"', toml.read_text(), re.M)
    pkg_of[toml.parent.name] = (m.group(1), v.group(1))
for toml in LIB.glob("@*/*/cohdl.toml"):
    m = re.search(r'^name\s*=\s*"([^"]+)"', toml.read_text(), re.M)
    v = re.search(r'^version\s*=\s*"([^"]+)"', toml.read_text(), re.M)
    pkg_of[str(toml.parent.relative_to(LIB))] = (m.group(1), v.group(1))

devices, users, courts = {}, {}, {}
for d, (pkg, ver) in pkg_of.items():
    for f in (LIB / d / "src").rglob("*.cohdl"):
        t = strip(f.read_text())
        for m in re.finditer(r"\bdevice\s+(\w+)\s*(?:<[^>{]*>)?\s*\{", t):
            b = block(t, m.end() - 1)
            # Required pins per pin block; "pins[VARIANT]" blocks are kept apart.
            for pm in re.finditer(r"\bpins(?:\[(\w+)\])?\s*\{", b):
                pb = block(b, pm.end() - 1)
                pins = re.findall(r"^\s*required\s+(\w+)\s*:\s*[A-Za-z0-9_, ]+\[", pb, re.M)
                devices[(pkg, m.group(1) + ("[" + pm.group(1) + "]" if pm.group(1) else ""))] = pins
        for m in re.finditer(r"\bpub\s+part\s+(\w+)\s*:\s*([^{]+?)\s*\{", t):
            b = block(t, m.end() - 1)
            fm = re.search(r"footprint:\s*([\w:]+)", b)
            if not fm: continue
            fp = fm.group(1) if "::" in fm.group(1) else pkg + "::" + fm.group(1)
            users.setdefault(fp, []).append({"pkg": pkg, "ver": ver, "part": m.group(1), "type": m.group(2).strip()})
        for m in re.finditer(r"\bfootprint\s+(\w+)\s*\{", t):
            b = block(t, m.end() - 1)
            c = re.search(r"courtyard\s*\{[^}]*?at:\s*\(\s*(-?[\d.]+)mm\s*,\s*(-?[\d.]+)mm\s*\)[^}]*?size:\s*\(\s*([\d.]+)mm\s*,\s*([\d.]+)mm\s*\)", b)
            pads = [(float(x), float(y)) for x, y in re.findall(r"\bat\s*\(\s*(-?[\d.]+)mm\s*,\s*(-?[\d.]+)mm\s*\)", b)]
            courts[pkg + "::" + m.group(1)] = {"courtyard": c.groups() if c else None, "pads": len(pads),
                "span": (max(p[0] for p in pads) - min(p[0] for p in pads), max(p[1] for p in pads) - min(p[1] for p in pads)) if pads else None}
json.dump({"pkgs": {k: list(v) for k, v in pkg_of.items()}, "devices": {f"{k[0]}::{k[1]}": v for k, v in devices.items()},
           "users": users, "footprints": courts}, open(sys.argv[1], "w"), indent=1)
print(len(devices), len(users), len(courts))
