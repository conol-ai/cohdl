#!/usr/bin/env python3
"""Check the agent-facing part notes in docs/part-notes/ against the library (plan 5b).

``python3 -I tools/part_notes.py check [NOTE.md ...]`` checks the given notes, or
every ``docs/part-notes/<package>/*.md``. Nothing is fetched. A note follows
``docs/part-notes/TEMPLATE.md`` (schema ``part-note/1``). The check is:

- **Identity.** The JSON block parses. Its device is declared in its package. Each listed
  part instantiates that device. Each identity is an AVL entry of one of those parts:
  exact ``mfr``/``mpn``, or for a series note ``mfr`` with an ``mpn_pattern`` over the
  parts that ``parts_pattern`` names. A given ``lcsc`` must be the part number
  ``lcsc_map.json`` maps it to.
- **Sources.** Each source has an https URL and a SHA-256. A ``local`` file must exist
  and hash to that SHA-256.
- **Pins.** The pin table maps every pad of the device (or of each listed variant) to
  the library pin name the device declares, each pad once.
- **Citations.** Every row of the parameter, rating, condition and pin tables cites a
  page of a listed source, as ``<id> p. N``.
- **Finished.** No template comment or ``<placeholder>`` is left.

Exit 0 when every note passes, 1 otherwise.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys

# `python3 -I` does not put the script's directory on sys.path.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from lcsc_backfill import _balanced_block, _string_fields, _strip_comments
from sourcing.bindings import package_metadata

ROOT = Path(__file__).resolve().parents[1]
NOTES = ROOT / "docs/part-notes"
LIB = ROOT / "lib"
CITED = ("Key parameters", "Absolute maximum ratings", "Recommended operating conditions", "Pins")
# Pads are numbers (1, 14), contact names (A1, B12, SH1), an exposed pad (EP)
# or a connector's mechanical pads (MP).
PIN = re.compile(r"^\s*(?:required|optional)\s+(\w+)\s*:\s*([A-Za-z0-9_,\s]+?)\s*\[", re.M)
PAD = re.compile(r"\b(?:[A-Z]{0,3}\d+|EP|MP)\b")


def package_dirs() -> dict[str, Path]:
    dirs = {}
    for manifest in sorted([*LIB.glob("*/cohdl.toml"), *LIB.glob("@*/*/cohdl.toml")]):
        metadata = package_metadata(manifest)
        if metadata:
            dirs[metadata[0]] = manifest.parent
    return dirs


def declarations(root: Path) -> tuple[dict[str, str], dict[str, tuple[str, str]]]:
    """{device: block} and {part: (device, block)} for one package."""
    devices, parts = {}, {}
    for path in sorted((root / "src").rglob("*.cohdl")):
        text = _strip_comments(path.read_text(encoding="utf-8"))
        for match in re.finditer(r"\bpub\s+device\s+(\w+)[^{]*\{", text):
            devices[match.group(1)] = _balanced_block(text, match.end() - 1)[0]
        # A part may name its device by path (`espressif_esp32::modules::wroom_s3::ESP32_S3_WROOM_1`).
        for match in re.finditer(r"\bpub\s+part\s+(\w+)\s*:\s*((?:\w+::)*\w+)[^{]*\{", text):
            device = match.group(2).rpartition("::")[2]
            parts[match.group(1)] = (device, _balanced_block(text, match.end() - 1)[0])
    return devices, parts


def device_pins(block: str) -> dict[str | None, dict[str, str]]:
    """{variant or None: {pad: pin}} from a device's pins blocks."""
    found = {}
    for match in re.finditer(r"\bpins(?:\[(\w+)\])?\s*\{", block):
        body = _balanced_block(block, match.end() - 1)[0]
        pads = {}
        for pin in PIN.finditer(body):
            for pad in pin.group(2).replace(",", " ").split():
                pads[pad] = pin.group(1)
        found[match.group(1)] = pads
    return found


def entries(block: str) -> list[tuple[str, str]]:
    found = []
    for match in re.finditer(r"\b(primary|alt)\s*\{", block):
        fields = dict(_string_fields(_balanced_block(block, match.end() - 1)[0]))
        found.append((fields.get("mfr"), fields.get("mpn")))
    return found


def sections(text: str) -> dict[str, str]:
    parts = re.split(r"^## (.+)$", text, flags=re.M)
    return {parts[i].strip(): parts[i + 1] for i in range(1, len(parts) - 1, 2)}


def table(body: str) -> list[dict[str, str]]:
    lines = [l.strip() for l in body.splitlines() if l.strip().startswith("|")]
    if len(lines) < 2:
        return []
    header = [c.strip() for c in lines[0].strip("|").split("|")]
    rows = []
    for line in lines[2:]:
        cells = [c.strip() for c in line.strip("|").split("|")]
        rows.append(dict(zip(header, cells)))
    return rows


def check(path: Path, dirs: dict[str, Path], lcsc: dict[tuple[str, str], str]) -> list[str]:
    text = path.read_text(encoding="utf-8")
    problems = []
    if "<!--" in text:
        problems.append("a template comment is left")
    body = sections(text)
    found = re.search(r"```json\n(.*?)\n```", body.get("Identity", ""), re.S)
    if not found:
        return problems + ["no JSON block under Identity"]
    try:
        identity = json.loads(found.group(1))
    except ValueError as error:
        return problems + [f"Identity JSON: {error}"]
    if identity.get("schema") != "part-note/1":
        problems.append("schema is not part-note/1")
    if re.search(r'"<[^"]*>"', found.group(1)):
        problems.append("a <placeholder> is left in Identity")

    package, _, device = identity.get("device", "").partition("::")
    if package not in dirs:
        return problems + [f"no library package {package!r}"]
    devices, parts = declarations(dirs[package])
    if device not in devices:
        return problems + [f"{package} declares no device {device}"]
    if "parts_pattern" in identity:
        names = [p for p in parts if re.fullmatch(identity["parts_pattern"], p)]
    else:
        names = []
        for name in identity.get("parts", []):
            pkg, _, part = name.partition("::")
            if pkg != package or part not in parts:
                problems.append(f"no part {name}")
            else:
                names.append(part)
    if not names:
        problems.append("the note covers no part")
    for name in names:
        if parts[name][0] != device:
            problems.append(f"{name} instantiates {parts[name][0]}, not {device}")
    avl = {e for name in names for e in entries(parts[name][1])}
    for ident in identity.get("identities", []):
        if "mpn_pattern" in ident:
            hits = [e for e in avl if e[0] == ident.get("mfr") and re.fullmatch(ident["mpn_pattern"], e[1] or "")]
            if not hits:
                problems.append(f"no {ident.get('mfr')} entry matches {ident['mpn_pattern']}")
            continue
        key = (ident.get("mfr"), ident.get("mpn"))
        if key not in avl:
            problems.append(f"{key[0]} {key[1]} is not an entry of the listed parts")
        if ident.get("lcsc") and lcsc.get(key) != ident["lcsc"]:
            problems.append(f"{key[1]}: lcsc {ident['lcsc']}, but lcsc_map.json maps {lcsc.get(key)}")

    source_ids = set()
    for source in identity.get("sources", []):
        source_ids.add(source.get("id"))
        if not str(source.get("url", "")).startswith("https://"):
            problems.append(f"source {source.get('id')}: no https URL")
        sha = source.get("sha256") or ""
        if not re.fullmatch(r"[0-9a-f]{64}", sha):
            problems.append(f"source {source.get('id')}: no SHA-256")
        if source.get("local"):
            local = ROOT / source["local"]
            if not local.is_file():
                problems.append(f"source {source.get('id')}: {source['local']} does not exist")
            elif hashlib.sha256(local.read_bytes()).hexdigest() != sha:
                problems.append(f"source {source.get('id')}: {source['local']} does not hash to {sha}")

    pins = device_pins(devices[device])
    variants = identity.get("variants") or [None]
    rows = table(body.get("Pins", ""))
    noted = {}
    for row in rows:
        pads = PAD.findall(row.get("Pad", ""))
        for pad in pads:
            if pad in noted:
                problems.append(f"pad {pad} is listed twice")
            noted[pad] = row.get("Library pin", "").strip("` ")
    for variant in variants:
        expected = pins.get(variant)
        if expected is None:
            problems.append(f"{device} has no pins block for variant {variant}")
            continue
        if noted != expected:
            missing = sorted(set(expected) - set(noted), key=lambda p: (len(p), p))
            wrong = sorted((p for p in noted if p in expected and noted[p] != expected[p]), key=lambda p: (len(p), p))
            extra = sorted(set(noted) - set(expected), key=lambda p: (len(p), p))
            for pad in missing:
                problems.append(f"pad {pad} ({expected[pad]}) is missing from the pin table")
            for pad in wrong:
                problems.append(f"pad {pad} is {expected[pad]} in the library, {noted[pad]} in the note")
            for pad in extra:
                problems.append(f"pad {pad} is not a pad of {device}")

    cite = re.compile(r"\b(\w+) pp?\. ?\d")
    for name in CITED:
        if name not in body:
            problems.append(f"no section {name}")
            continue
        for row in table(body[name]):
            source = row.get("Source", "")
            ids = cite.findall(source)
            if not ids:
                problems.append(f"{name}: a row cites no page: {' | '.join(row.values())[:80]}")
            elif not set(ids) <= source_ids:
                problems.append(f"{name}: cites an unknown source in {source!r}")
    return problems


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    c = sub.add_parser("check")
    c.add_argument("notes", nargs="*", type=Path)
    args = parser.parse_args(argv)
    notes = args.notes or sorted(NOTES.glob("*/*.md")) + sorted(NOTES.glob("@*/*/*.md"))
    dirs = package_dirs()
    rows = json.loads((ROOT / "tools/sourcing_data/lcsc_map.json").read_text(encoding="utf-8"))["rows"]
    lcsc = {(row["mfr"], row["mpn"]): row["pn"] for row in rows}
    failed = 0
    for note in notes:
        problems = check(note, dirs, lcsc)
        failed += bool(problems)
        for problem in problems:
            print(f"{note}: {problem}", file=sys.stderr)
    print(f"{len(notes) - failed} of {len(notes)} notes pass")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
