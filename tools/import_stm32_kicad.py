#!/usr/bin/env python3
"""Freeze the source-backed KiCad cross-reference used by gen_stm32.py.

This is an explicit maintainer import, not part of normal generation.  It
joins the exact ST order-code inventory and pinned ST pin snapshot to an exact
checkout of the official KiCad symbol and footprint libraries.  A row is
admitted only when:

* the ST order code resolves uniquely to one complete generated device;
* KiCad resolves it to one concrete footprint and an official ST datasheet;
* the footprint exists at the pinned footprint commit; and
* its SMD pad-number set is exactly the ST source pin-position set.

The resulting JSON is deterministic and is consumed offline.  KiCad library
data is CC-BY-SA-4.0; the generated package retains its attribution and notice.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import re
import subprocess
import sys


ROOT = pathlib.Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))
import gen_stm32 as stm32  # noqa: E402


SYMBOLS_COMMIT = "7800d91437ce44e2ed0928f2ad31a287457b8a68"
FOOTPRINTS_COMMIT = "819223b66f96508feaeaa305301b5e6bb5c1038b"
DEFAULT_OUT = ROOT / "tools" / "stm32_data" / "kicad_parts.json"
SYMBOL_FILE = re.compile(r"^\s*\(symbol \"([^\"]+)\"", re.MULTILINE)
PROPERTY = r'\(property "{}" "([^"]*)"'
SMD_PAD = re.compile(r'^\s*\(pad "([^"]*)"\s+smd\s+', re.MULTILINE)


def git_head(path: pathlib.Path) -> str:
    return subprocess.check_output(
        ["git", "-C", str(path), "rev-parse", "HEAD"], text=True
    ).strip()


def require_commit(path: pathlib.Path, expected: str, label: str) -> None:
    actual = git_head(path)
    if actual != expected:
        raise ValueError(f"{label} checkout is {actual}; expected {expected}")
    dirty = subprocess.check_output(
        ["git", "-C", str(path), "status", "--porcelain", "--untracked-files=all"],
        text=True,
    ).strip()
    if dirty:
        raise ValueError(f"{label} checkout {path} is dirty; refusing source import")


def sha256(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def symbol_patterns(name: str) -> list[str]:
    """Expand KiCad's one underscore-delimited choice group."""
    if "_" not in name:
        return [name]
    pieces = name.split("_")
    if len(pieces) != 3 or not pieces[0] or not pieces[1] or not pieces[2]:
        raise ValueError(f"unsupported KiCad STM32 symbol choice syntax {name!r}")
    return sorted(pieces[0] + choice + pieces[2] for choice in pieces[1].split("-"))


def identity_pattern(value: str) -> re.Pattern[str]:
    return re.compile("^" + re.escape(value).replace("x", "[A-Z0-9]") + "$")


def load_symbols(root: pathlib.Path) -> list[dict]:
    rows = []
    for path in sorted(root.glob("MCU_ST_STM32*.kicad_symdir/*.kicad_sym")):
        text = path.read_text()
        name_match = SYMBOL_FILE.search(text)
        if not name_match:
            continue
        name = name_match.group(1)
        if not name.startswith("STM32"):
            continue
        footprint_match = re.search(PROPERTY.format("Footprint"), text)
        datasheet_match = re.search(PROPERTY.format("Datasheet"), text)
        if not footprint_match or not footprint_match.group(1):
            continue
        if not datasheet_match or not datasheet_match.group(1):
            continue
        datasheet = datasheet_match.group(1).replace("http://www.st.com/", "https://www.st.com/")
        if not datasheet.startswith("https://www.st.com/"):
            raise ValueError(f"{path}: STM32 datasheet is not on www.st.com: {datasheet}")
        expanded = symbol_patterns(name)
        rows.append(
            {
                "choice_count": len(expanded),
                "datasheet": datasheet,
                "footprint": footprint_match.group(1),
                "name": name,
                "path": path.relative_to(root).as_posix(),
                "patterns": [identity_pattern(value) for value in expanded],
            }
        )
    if not rows:
        raise ValueError(f"no STM32 symbols found under {root}")
    return rows


def footprint_path(root: pathlib.Path, qualified: str) -> pathlib.Path:
    if qualified.count(":") != 1:
        raise ValueError(f"invalid KiCad footprint name {qualified!r}")
    library, name = qualified.split(":", 1)
    return root / f"{library}.pretty" / f"{name}.kicad_mod"


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("symbols", type=pathlib.Path)
    parser.add_argument("footprints", type=pathlib.Path)
    parser.add_argument("--out", type=pathlib.Path, default=DEFAULT_OUT)
    args = parser.parse_args()

    require_commit(args.symbols, SYMBOLS_COMMIT, "KiCad symbols")
    require_commit(args.footprints, FOOTPRINTS_COMMIT, "KiCad footprints")
    symbols = load_symbols(args.symbols)
    snapshot = stm32.load_snapshot()
    portfolio = stm32.portfolio_matches(snapshot["models"])
    pinouts = {row["id"]: row for row in snapshot["pinouts"]}
    footprint_pads: dict[str, set[str]] = {}
    mappings = []

    for identity, (model_index, device) in sorted(portfolio["matched_identities"].items()):
        model = snapshot["models"][model_index]
        pinout = pinouts[model["pinout"]]
        if pinout.get("incomplete"):
            continue
        try:
            normalized = stm32.normalized_pins(pinout)
        except ValueError:
            continue
        matches = [
            row
            for row in symbols
            if any(pattern.fullmatch(identity) for pattern in row["patterns"])
        ]
        if not matches:
            continue
        concrete_footprints = {row["footprint"] for row in matches}
        if len(concrete_footprints) != 1:
            raise ValueError(
                f"{identity} maps to multiple KiCad footprints: "
                f"{sorted(concrete_footprints)}"
            )
        smallest = min(row["choice_count"] for row in matches)
        preferred = [row for row in matches if row["choice_count"] == smallest]
        preferred_pairs = {(row["footprint"], row["datasheet"]) for row in preferred}
        if len(preferred_pairs) != 1:
            raise ValueError(
                f"{identity} has ambiguous preferred KiCad mappings: "
                f"{sorted(preferred_pairs)}"
            )
        selected = sorted(
            preferred, key=lambda row: (row["name"], row["path"])
        )[0]
        qualified = selected["footprint"]
        if qualified not in footprint_pads:
            path = footprint_path(args.footprints, qualified)
            if not path.is_file():
                raise ValueError(f"missing pinned KiCad footprint {qualified}: {path}")
            footprint_pads[qualified] = set(SMD_PAD.findall(path.read_text()))
        source_pads = {
            position for _, positions, _, _, _ in normalized for position in positions
        }
        if footprint_pads[qualified] != source_pads:
            raise ValueError(
                f"{identity}: {qualified} pad set differs from ST pinout; "
                f"missing={sorted(source_pads - footprint_pads[qualified])}, "
                f"extra={sorted(footprint_pads[qualified] - source_pads)}"
            )
        mappings.append(
            {
                "datasheet": selected["datasheet"],
                "device": device,
                "family": model["family"],
                "identity": identity,
                "kicad_footprint": qualified,
                "package": pinout["package"],
                "pinout_id": model["pinout"],
                "source_model": model["ref"],
                "symbol": selected["name"],
                "symbol_path": selected["path"],
            }
        )

    rows = sum(len(portfolio["by_identity"][row["identity"]]) for row in mappings)
    output = {
        "coverage": {
            "electrical_identities": len(mappings),
            "exact_order_code_rows": rows,
            "unique_datasheets": len({row["datasheet"] for row in mappings}),
            "unique_footprints": len({row["kicad_footprint"] for row in mappings}),
        },
        "mappings": mappings,
        "schema_version": 1,
        "sources": {
            "kicad_footprints": {
                "commit": FOOTPRINTS_COMMIT,
                "license": "CC-BY-SA-4.0",
                "license_sha256": sha256(args.footprints / "LICENSE.md"),
                "repository": "https://gitlab.com/kicad/libraries/kicad-footprints.git",
            },
            "kicad_symbols": {
                "commit": SYMBOLS_COMMIT,
                "license": "CC-BY-SA-4.0",
                "license_sha256": sha256(args.symbols / "LICENSE.md"),
                "repository": "https://gitlab.com/kicad/libraries/kicad-symbols.git",
            },
        },
    }
    encoded = (json.dumps(output, indent=2, sort_keys=True) + "\n").encode()
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_bytes(encoded)
    print(
        f"wrote {args.out}: {len(mappings)} identities, {rows} order-code rows, "
        f"{len(footprint_pads)} footprints, sha256 {hashlib.sha256(encoded).hexdigest()}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
