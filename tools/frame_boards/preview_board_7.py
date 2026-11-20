"""Build the Samsung CL all-member and sampling board from the board 6 W3 snapshot."""
import hashlib
import json
import re
import subprocess
import sys
from itertools import zip_longest
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]


def build():
    inputs = json.loads((HERE / "preview-board-7/inputs.json").read_text())
    source = (ROOT / inputs["report"]).read_bytes()
    if hashlib.sha256(source).hexdigest() != inputs["report_sha256"]:
        raise ValueError("W3 report differs from the selected snapshot")
    report = json.loads(source)
    pages_path, = (name for name in report["sources"] if name.endswith("/confirmed.json"))
    raw = (ROOT / "tools/sourcing_data" / pages_path).read_bytes()
    if hashlib.sha256(raw).hexdigest() != report["sources"][pages_path]:
        raise ValueError("stock snapshot differs from the W3 report")
    pages = {r["c"]: r for r in json.loads(raw)}
    batches = []
    for size in ("0402", "0603", "0805"):
        group, = (g for g in report["groups"] if
                  (g["footprint"], g["type"], g["mfr"], g["series"]) == ("CHIP_" + size, "MLCC", "Samsung", "CL"))
        src = subprocess.check_output(["git", "show", inputs["packages"]["passive"]["tree"] +
                                       f":src/capacitors_{size}.cohdl"], cwd=ROOT, text=True)
        parts = dict(re.findall(r"^pub part (\w+): (MLCC<[^\n]+?\]) \{", src, re.M))
        selected = group["sample"] if size == "0603" else [row[0] for row in group["member_rows"]]
        batch = []
        for pn, mpn, part, _, category, stock in group["member_rows"]:
            if pn not in selected:
                continue
            p = pages[pn]
            if (p["mpn"], p["package"], p["stock"]) != (mpn, size, stock) or stock < report["rule"]["min_stock"]:
                raise ValueError(f"{pn}: incompatible identity, package or stock")
            batch.append(dict(pn=pn, mpn=mpn, mfr="Samsung", part="passive::" + parts[part],
                              footprint="passive::CHIP_" + size, group=f"CHIP_{size} MLCC Samsung CL",
                              purpose="sample", jlc_package=size, stock=stock, page_fetched=p["fetched_date"],
                              category=category))
        batches.append(batch)
    if [len(b) for b in batches] != [11, 1, 6]:
        raise ValueError("unexpected target population")
    archive = ROOT / "tools/sourcing_data/jlc_previews/2026-11-19-board-6/package/manifest.json"
    old = {r["pn"]: r for r in json.loads(archive.read_text())["rows"]}
    batches.append([dict(old[pn], purpose="control") for pn in inputs["controls"]])
    rows = []
    for row in (r for group in zip_longest(*batches) for r in group if r):
        col, band = len(rows) % 8, len(rows) // 8
        places = [{"at_mm": [10 + col * 10, 10 + (4 * band + 2 * j + i) * 10], "side": side, "rotate": angle}
                  for i, side in enumerate(("top", "bottom")) for j, angle in enumerate((0, 37))]
        rows.append(dict(row, name=row["purpose"] + "_" + row["pn"], pins=["A", "B"], width_mm=6,
                         height_mm=6, covers_rotation=True, placements=places))
    if len({r["pn"] for r in rows}) != len(rows):
        raise ValueError("duplicate target or control")
    return {"name": "preview_board_7", "dependencies": {n: p["version"] for n, p in inputs["packages"].items()},
            "rows": rows}


if __name__ == "__main__":
    Path(sys.argv[1]).write_text(json.dumps(build(), ensure_ascii=False, indent=1) + "\n")
