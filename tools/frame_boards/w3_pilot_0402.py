"""W3 sampling pilot manifest: the samples the grouping report chose for three 0402 groups.

Usage:
  python3 -I tools/frame_boards/w3_pilot_0402.py OUT.json [--inputs INPUTS.json]
  python3 -I tools/frame_boards/w3_pilot_0402.py --freeze INPUTS.json [--report REPORT.json]

The proposal (docs/library-supplier-w3-sampling.md, section 4) pilots its draft
rule on one board of 0402 parts, which tests both part types and two
manufacturers on one land:

- CHIP_0402 ChipResistor Yageo RC;
- CHIP_0402 ChipResistor Vishay CRCW;
- CHIP_0402 MLCC Yageo CC.

The part numbers are the report's `sample` for each group
(tools/sourcing_data/w3_sampling/2026-11-07/groups.json), chosen there among
members with at least the report's `min_stock` (30) pieces in stock. Each row declares a local part
with the map row's identity on the library entry's device type and footprint,
so the preview shows the raw frame with no map, as on lcsc_batch_2_passive.py.
Chip parts are not polarised, and the bottom-side convention is settled (C1),
so each part gets one top copy at 0 degrees.

The board was previewed on 2026-11-07, so its manifest is a record pinned to
`passive` 0.2.5, not to the current library. Two steps keep it rebuildable:

- `--freeze` reads a report, `lib/passive`, `lcsc_map.json` and the library
  inventory of the current tree and writes the inputs the board needs: each
  sample's map identity, library part, entry, device type and pins, and the
  land's courtyard. It refuses a report for another `passive` version. The
  2026-11-07 inputs are committed as w3-pilot-0402/inputs.json.
- The default mode builds the manifest from frozen inputs alone (the committed
  ones unless `--inputs`), plus the report and confirmed pages they name, each
  checked against the SHA-256 the inputs record. It does not read `lib/`, so
  a later `passive` version does not change or break it.

The board is evidence for the pilot only. A sample that reads (0, 0, 0) gets
its own frame row; the group's other members get nothing from it until the
proposal is accepted.
"""
import argparse, hashlib, json, math, re, subprocess, sys, tempfile
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "tools/sourcing_data"
REPORT = DATA / "w3_sampling/2026-11-07/groups.json"
INPUTS = ROOT / "tools/frame_boards/w3-pilot-0402/inputs.json"
GROUPS = [("CHIP_0402", "ChipResistor", "Yageo", "RC"),
          ("CHIP_0402", "ChipResistor", "Vishay", "CRCW"),
          ("CHIP_0402", "MLCC", "Yageo", "CC")]
GAP, WIDTH = 2.0, 100.0


def sha256(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def repo_path(path):
    """PATH relative to the repository when it is inside it, else absolute."""
    path = Path(path).resolve()
    return str(path.relative_to(ROOT)) if path.is_relative_to(ROOT) else str(path)


def report_group(report, key):
    return next(g for g in report["groups"] if (g["footprint"], g["type"], g["mfr"], g["series"]) == key)


def report_pages(report):
    """The confirmed pages the report cites in its sources."""
    return next(p for p in sorted((DATA / "lcsc_backfill").glob("*/confirmed.json"))
                if p.parent.name in report["sources"] or str(p.relative_to(DATA)) in report["sources"])


def freeze(report_path):
    """The board's inputs, read from the current tree for REPORT_PATH's samples."""
    with tempfile.TemporaryDirectory() as td:
        subprocess.run([sys.executable, "-I", str(ROOT / "tools/frame_boards/inventory.py"), td + "/inventory.json"],
                       check=True, capture_output=True)
        inv = json.load(open(td + "/inventory.json"))
    report = json.load(open(report_path))
    mapped = {r["pn"]: r for r in json.load(open(DATA / "lcsc_map.json"))["rows"]}
    pages_path = report_pages(report)
    pages = {r["c"]: r for r in json.load(open(pages_path))}

    ENTRY = re.compile(r'(primary|alt) \{ mfr: "([^"]+)", mpn: "([^"]+)"')
    entries = {}
    for user in [u for users in inv["users"].values() for u in users if u["pkg"] == "passive"]:
        entries.setdefault(user["part"], user)
    by_identity = {}
    for path in sorted((ROOT / "lib/passive/src").glob("*.cohdl")):
        for m in re.finditer(r"\bpub part (\w+): [^{]+\{(.*?)\n\}", path.read_text(), re.S):
            footprint = re.search(r"footprint: (\w+)", m.group(2)).group(1)
            for kind, mfr, mpn in ENTRY.findall(m.group(2)):
                by_identity.setdefault((mfr, mpn), []).append((m.group(1), kind, footprint))

    version = inv["pkgs"]["passive"][1]
    samples = []
    for key in GROUPS:
        group = report_group(report, key)
        if group["package_version"] != version:
            raise SystemExit(f"{key}: the report is for passive {group['package_version']}, "
                             f"the library is {version}")
        for pn in group["sample"]:
            row = mapped[pn]
            hits = by_identity.get((row["mfr"], row["mpn"]), [])
            if len(hits) != 1:
                raise SystemExit(f"{pn}: {len(hits)} lib/passive entries")
            if pages[pn]["stock"] < report["rule"]["min_stock"]:
                raise SystemExit(f"{pn}: stock {pages[pn]['stock']} on its confirmed page, below the report's minimum")
            part, kind, footprint = hits[0]
            t = entries[part]["type"]
            base, variant = t.split("<")[0].strip(), t[t.index("["):]
            pins = inv["devices"].get("passive::" + base + variant) or ["A", "B"]
            samples.append({"group": list(key), "pn": pn, "mfr": row["mfr"], "mpn": row["mpn"], "part": part,
                            "kind": kind, "footprint": footprint, "type": t, "pins": pins})
    footprints = {"passive::" + fp: inv["footprints"]["passive::" + fp]["courtyard"]
                  for fp in sorted({s["footprint"] for s in samples})}
    return {"schema_version": 1,
            "report": {"path": repo_path(report_path), "sha256": sha256(report_path)},
            "pages": {"path": repo_path(pages_path), "sha256": sha256(pages_path)},
            "dependencies": {"std": "0.3.0", "passive": version},
            "courtyards": footprints, "samples": samples}


def build(inputs):
    """The manifest from frozen INPUTS; reads nothing under lib/."""
    report_path, pages_path = ROOT / inputs["report"]["path"], ROOT / inputs["pages"]["path"]
    for name, path in (("report", report_path), ("pages", pages_path)):
        if sha256(path) != inputs[name]["sha256"]:
            raise SystemExit(f"{path}: SHA-256 is not the {name} the inputs were frozen from")
    report = json.load(open(report_path))
    if report_pages(report).resolve() != pages_path.resolve():
        raise SystemExit(f"{pages_path}: not the confirmed pages the report cites")
    pages = {r["c"]: r for r in json.load(open(pages_path))}
    version = inputs["dependencies"]["passive"]
    expected = []
    for key in GROUPS:
        group = report_group(report, key)
        if group["package_version"] != version:
            raise SystemExit(f"{key}: the report is for passive {group['package_version']}, the inputs {version}")
        expected += [(key, pn) for pn in group["sample"]]
    if [(tuple(s["group"]), s["pn"]) for s in inputs["samples"]] != expected:
        raise SystemExit("the inputs' samples are not the report's, in its order")

    rows, x, y, shelf = [], 0.0, 0.0, 0.0
    for s in inputs["samples"]:
        page = pages[s["pn"]]
        if page["stock"] < report["rule"]["min_stock"]:
            raise SystemExit(f"{s['pn']}: stock {page['stock']} on its confirmed page, below the report's minimum")
        if page["mpn"] != s["mpn"]:
            raise SystemExit(f"{s['pn']}: page reads {page['mpn']}, map {s['mpn']}")
        cx, cy, w, h = map(float, inputs["courtyards"]["passive::" + s["footprint"]])
        side = 2 * math.hypot(abs(cx) + w / 2, abs(cy) + h / 2)
        cell = side + GAP
        if x + cell > WIDTH and x > 0:
            x, y, shelf = 0.0, y + shelf, 0.0
        at = [round(x + side / 2 + GAP / 2, 3), round(y + side / 2 + GAP / 2, 3)]
        x += cell
        shelf = max(shelf, cell)
        rows.append({"name": s["part"] + ("_ALT" if s["kind"] == "alt" else ""),
                     "footprint": "passive::" + s["footprint"], "part": "passive::" + s["type"], "pn": s["pn"],
                     "mfr": s["mfr"], "mpn": page["mpn"], "pins": s["pins"],
                     "width_mm": round(side, 3), "height_mm": round(side, 3), "covers_rotation": True,
                     "placements": [{"at_mm": at, "rotate": 0, "side": "top"}],
                     "note": f"W3 pilot sample, group {' '.join(s['group'])}; lib/passive {s['part']} {s['kind']}",
                     "jlc_package": page["package"], "category": page["category"]})
    return {"name": "w3_pilot_0402", "dependencies": dict(inputs["dependencies"]), "rows": rows}, y + shelf


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("out", nargs="?", type=Path, help="the manifest to write")
    parser.add_argument("--inputs", type=Path, default=INPUTS, help="frozen inputs to build from")
    parser.add_argument("--freeze", type=Path, metavar="INPUTS", help="write the inputs from the current tree")
    parser.add_argument("--report", type=Path, default=REPORT, help="the grouping report to freeze from")
    args = parser.parse_args(argv)
    if (args.out is None) == (args.freeze is None):
        parser.error("give OUT.json, or --freeze INPUTS.json")
    if args.freeze:
        inputs = freeze(args.report)
        args.freeze.write_text(json.dumps(inputs, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")
        print(len(inputs["samples"]), "samples frozen for passive", inputs["dependencies"]["passive"])
        return 0
    manifest, depth = build(json.load(open(args.inputs)))
    json.dump(manifest, open(args.out, "w", encoding="utf-8"), indent=1, ensure_ascii=False)
    rows = manifest["rows"]
    print(len(rows), "rows,", sum(len(r["placements"]) for r in rows), "placements, board ~",
          WIDTH, "x", round(depth, 1), "mm")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
