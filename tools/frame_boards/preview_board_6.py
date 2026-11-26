"""Rebuild the UNI-ROYAL larger-size preview manifest from its frozen inputs."""
import hashlib
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from gen_passive import ohm_literal, uniroyal_decode
from sourcing import w3_rule


def build():
    inputs = json.loads((HERE / "preview-board-6/inputs.json").read_text())
    run = ROOT / inputs["backfill"]
    provenance = json.loads((run / "provenance.json").read_text())
    for name in ("confirmed.json", "lcsc_map.json"):
        if hashlib.sha256((run / name).read_bytes()).hexdigest() != provenance["sha256"][name]:
            raise ValueError(f"{name} differs from the archived backfill")
    pages = {p["c"]: p for p in json.loads((run / "confirmed.json").read_text())}
    mapped = {r["pn"]: r for r in json.loads((run / "lcsc_map.json").read_text())["rows"]}
    sets = inputs["samples"]
    if [len(sets[k]) for k in ("0603WA", "0805W8", "1206W4")] != [w3_rule.required(n) for n in (116, 97, 17)]:
        raise ValueError("sample counts do not cover the three W3 groups")
    board4 = ROOT / "tools/sourcing_data/jlc_previews/2026-11-11-board-4/package/manifest.json"
    controls = {r["pn"]: r for r in json.loads(board4.read_text())["rows"]}
    batches = [*sets.values(), inputs["controls"]]
    order = [batch[i] for i in range(max(map(len, batches))) for batch in batches if i < len(batch)]
    if len(order) != len(set(order)):
        raise ValueError("repeated sample or control")
    rows = []
    for index, pn in enumerate(order):
        page, identity = pages[pn], mapped[pn]
        size = page["package"]
        if page["mpn"] != identity["mpn"] or page["stock"] < w3_rule.RULE["min_stock"]:
            raise ValueError(f"{pn}: mismatched MPN or insufficient recorded stock")
        if pn in inputs["controls"]:
            old = controls[pn]
            part = old["part"]
            group = old.get("group")
            if pn == "C11702":
                group = "CHIP_0402 ChipResistor UNI-ROYAL 0402WG"
            if not group:
                raise ValueError(f"{pn}: unknown control group")
            if old["mpn"] != page["mpn"] or old["footprint"] != "passive::CHIP_" + size:
                raise ValueError(f"{pn}: control differs from the board 4 record")
            purpose = "control"
        else:
            mpn = page["mpn"]
            decoded = uniroyal_decode(mpn)
            if (identity["mfr"] != "UNI-ROYAL" or decoded is None or decoded["size"] != size
                    or pn not in sets[mpn[:6]]):
                raise ValueError(f"{pn}: identity does not match its UNI-ROYAL series")
            jumper = decoded["ohms"] == 0
            value = "0ohm" if jumper else ohm_literal(decoded["mantissa"], decoded["decade"])
            tol = "5%" if jumper else {"F": "1%", "J": "5%"}[decoded["tolerance"]]
            part = f"passive::ChipResistor<{value}, {tol}>[R{size}]"
            group = f"CHIP_{size} ChipResistor UNI-ROYAL {mpn[:6]}"
            purpose = "sample"
        # Interleave sides down the board and series across it.
        col, pair = index % 9, index // 9
        places = [{"at_mm": [10 + col * 10, 10 + (4 * pair + 2 * angle_index + side_index) * 10],
                   "rotate": angle, "side": side}
                  for side_index, side in enumerate(("top", "bottom")) for angle_index, angle in enumerate((0, 37))]
        rows.append({"name": f"{purpose}_{pn}", "footprint": "passive::CHIP_" + size, "part": part,
                     "pn": pn, "mpn": page["mpn"], "mfr": identity["mfr"], "pins": ["A", "B"],
                     "width_mm": 6, "height_mm": 6, "covers_rotation": True, "placements": places,
                     "purpose": purpose, "group": group, "jlc_package": size, "stock": page["stock"],
                     "page_fetched": page["fetched_date"], "category": identity["category"]})
    deps = {k: v["version"] for k, v in inputs["packages"].items()}
    return {"name": "preview_board_6", "dependencies": deps, "rows": rows}


if __name__ == "__main__":
    Path(sys.argv[1]).write_text(json.dumps(build(), ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
