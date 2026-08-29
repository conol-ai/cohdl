#!/usr/bin/env python3
"""Generate the source-backed Espressif ESP32 catalog.

The checked-in snapshots deliberately separate acquisition from ordinary
generation.  Normal use is offline and deterministic::

    python3 tools/gen_esp32.py
    python3 tools/gen_esp32.py --check

Refreshing either authority is an explicit maintainer operation.  The first
argument is the byte-for-byte response from Espressif's Product Selector API;
the second is the official ``espressif/kicad-libraries`` checkout at tag
``3.2.1``::

    python3 tools/gen_esp32.py --import-sources \
        /path/to/espressif-products.json /path/to/kicad-libraries

Only Python's standard library is used.  Every Product Selector MPN is placed
in exactly one of the emitted, pre-existing, or omitted ledgers.  Admission is
fail-closed: an exact part is emitted only when a curated selector-family join
has an official symbol, an official concrete footprint, and an identical set
of physical pad numbers after explicitly reviewed reserved-pad supplements.
"""

from __future__ import annotations

import argparse
import collections
import dataclasses
import hashlib
import json
import pathlib
import re
import subprocess
import sys
from typing import Any, Iterable


ROOT = pathlib.Path(__file__).resolve().parent.parent
DATA_DIR = ROOT / "tools" / "esp32_data"
SELECTOR_SNAPSHOT = DATA_DIR / "selector.json"
SYMBOL_SNAPSHOT = DATA_DIR / "kicad_symbols.json"
PACKAGE_ROOT = ROOT / "lib" / "@espressif" / "esp32"
DEVICE_OUT = PACKAGE_ROOT / "src" / "catalog_devices.cohdl"
PART_OUT = PACKAGE_ROOT / "src" / "catalog_parts.cohdl"
DOC_OUT = PACKAGE_ROOT / "docs" / "esp32-part-catalog.md"

SELECTOR_URL = "https://products.espressif.com/api/user/products?language=en"
SELECTOR_RETRIEVED = "2026-08-28"
SELECTOR_RAW_SHA256 = (
    "16cc33b25cc86e7dcbced8cdd2c2d8fb696fc3363453e862a32167bdf5aa6795"
)
KICAD_REPOSITORY = "https://github.com/espressif/kicad-libraries"
KICAD_TAG = "3.2.1"
KICAD_COMMIT = "1dfc3110895c9cd62daf332f49c49ee0ee200831"
KICAD_SYMBOL_PATH = "symbols/Espressif.kicad_sym"
KICAD_SYMBOL_SHA256 = (
    "5e9785368a1cccbd904f8ceeab1c9165a8ef82ab00c99bf21275dc91749356cc"
)
KICAD_LICENSE = "CC-BY-SA-4.0 WITH KiCad-libraries-exception"

SNAPSHOT_SCHEMA = 1
# Filled with the SHA-256 of the canonical checked-in snapshots.  Updating a
# source is intentionally a two-step review: import, inspect the diff, then
# update these constants to the reviewed hashes.
SELECTOR_SNAPSHOT_SHA256 = "fe9d2c5c97cfc53eaaa77bc60ff54216b90f7364a31566f795e663ed2c8ff4b2"
SYMBOL_SNAPSHOT_SHA256 = "801a066f3930bc20adda64db3984ad5027700cfb745934c08089eb4ba584b5f9"

EXPECTED_ROWS = 318
EXPECTED_LIFECYCLES = {
    "EOL": 60,
    "Mass Production": 173,
    "NRND": 36,
    "Replaced": 34,
    "Sample": 15,
}
DOC_CATALOG = "docs/esp32-part-catalog.md"

NON_IDENT = re.compile(r"[^A-Za-z0-9_]+")
MULTI_UNDERSCORE = re.compile(r"_+")
NUMBER = re.compile(r"^[0-9]+$")
NC_NAME = re.compile(r"^(?:NC|DNC|DNU|RFU|RESERVED)(?:_|$)")


@dataclasses.dataclass(frozen=True)
class Rule:
    symbol: str
    source_footprint: str
    footprint_ref: str
    note: str


def ident(value: str) -> str:
    value = MULTI_UNDERSCORE.sub("_", NON_IDENT.sub("_", value)).strip("_")
    if not value:
        raise ValueError("empty identifier")
    if value[0].isdigit():
        value = "P_" + value
    return value


def manufacturer_footprint(source_name: str) -> str:
    return "FP_" + ident(source_name).upper()


# These names are the stable public geometry API shared with
# tools/gen_esp32_footprints.py.  Generic package geometry is dependency-owned
# by qfn; module and SiP geometry remains manufacturer-owned.
SOC_RULES: dict[str, Rule] = {
    "ESP32-D0WDQ6": Rule(
        "ESP32", "QFN48E_0P4_6", "qfn::ESPRESSIF_QFN48E_0P4_6", "6x6 mm base ESP32"
    ),
    "ESP32-D0WDQ6-V3": Rule(
        "ESP32", "QFN48E_0P4_6", "qfn::ESPRESSIF_QFN48E_0P4_6", "6x6 mm base ESP32"
    ),
    "ESP32-C3": Rule(
        "ESP32-C3", "QFN32_0P5_5", "qfn::ESPRESSIF_QFN32_0P5_5", "base C3"
    ),
    "ESP32-C5HR2": Rule(
        "ESP32-C5", "QFN48_0P4_6_E4P7", "qfn::ESPRESSIF_QFN48_0P4_6_E4P7", "C5 family-specific QFN"
    ),
    "ESP32-C5HR8": Rule(
        "ESP32-C5", "QFN48_0P4_6_E4P7", "qfn::ESPRESSIF_QFN48_0P4_6_E4P7", "C5 family-specific QFN"
    ),
    "ESP32-C5HF4": Rule(
        "ESP32-C5", "QFN48_0P4_6_E4P7", "qfn::ESPRESSIF_QFN48_0P4_6_E4P7", "C5 family-specific QFN"
    ),
    "ESP32-C6": Rule(
        "ESP32-C6", "QFN40_0P4_5_E3P3", "qfn::ESPRESSIF_QFN40_0P4_5_E3P3", "base C6"
    ),
    "ESP32-C6FH4": Rule(
        "ESP32-C6FH4", "QFN32_0P5_5_E3P7X3P2", "qfn::ESPRESSIF_QFN32_0P5_5_E3P7X3P2", "C6 embedded-flash pinout"
    ),
    "ESP32-C6FH8": Rule(
        "ESP32-C6FH4", "QFN32_0P5_5_E3P7X3P2", "qfn::ESPRESSIF_QFN32_0P5_5_E3P7X3P2", "C6 embedded-flash pinout"
    ),
    "ESP32-H2FH2": Rule(
        "ESP32-H2", "QFN32_0P4_4_E2P8", "qfn::ESPRESSIF_QFN32_0P4_4_E2P8", "H2 family QFN"
    ),
    "ESP32-H2FH4": Rule(
        "ESP32-H2", "QFN32_0P4_4_E2P8", "qfn::ESPRESSIF_QFN32_0P4_4_E2P8", "H2 family QFN"
    ),
    "ESP32-H2FH2S": Rule(
        "ESP32-H2", "QFN32_0P4_4_E2P8", "qfn::ESPRESSIF_QFN32_0P4_4_E2P8", "H2 family QFN"
    ),
    "ESP32-H2FH4S": Rule(
        "ESP32-H2", "QFN32_0P4_4_E2P8", "qfn::ESPRESSIF_QFN32_0P4_4_E2P8", "H2 family QFN"
    ),
    # The A/B association is verified against Espressif's two direct ASC
    # package sources by the geometry generator.
    "ESP32-P4NRW16": Rule(
        "ESP32-P4", "QFN104_0P35_10_E7P5_A", "qfn::ESPRESSIF_QFN104_0P35_10_E7P5_A", "P4 revision-A pinout"
    ),
    "ESP32-P4NRW32": Rule(
        "ESP32-P4", "QFN104_0P35_10_E7P5_A", "qfn::ESPRESSIF_QFN104_0P35_10_E7P5_A", "P4 revision-A pinout"
    ),
    "ESP32-P4NRW16X": Rule(
        "ESP32-P4X", "QFN104_0P35_10_E7P5_B", "qfn::ESPRESSIF_QFN104_0P35_10_E7P5_B", "P4X revision-B pinout"
    ),
    "ESP32-P4NRW32X": Rule(
        "ESP32-P4X", "QFN104_0P35_10_E7P5_B", "qfn::ESPRESSIF_QFN104_0P35_10_E7P5_B", "P4X revision-B pinout"
    ),
    "ESP32-PICO-V3": Rule(
        "ESP32-PICO-V3", "ESP32-PICO-V3", "FP_ESP32_PICO_V3", "direct Espressif SiP land pattern"
    ),
    "ESP32-S2": Rule(
        "ESP32-S2", "QFN56_0P4_7B", "qfn::ESPRESSIF_QFN56_0P4_7B", "base S2"
    ),
    "ESP8684H2": Rule(
        "ESP8684", "QFN24_0P5_4_E2P8", "qfn::ESPRESSIF_QFN24_0P5_4_E2P8", "ESP8684 family QFN"
    ),
    "ESP8684H2X": Rule(
        "ESP8684", "QFN24_0P5_4_E2P8", "qfn::ESPRESSIF_QFN24_0P5_4_E2P8", "ESP8684 family QFN"
    ),
    "ESP8684H4": Rule(
        "ESP8684", "QFN24_0P5_4_E2P8", "qfn::ESPRESSIF_QFN24_0P5_4_E2P8", "ESP8684 family QFN"
    ),
    "ESP8684H4X": Rule(
        "ESP8684", "QFN24_0P5_4_E2P8", "qfn::ESPRESSIF_QFN24_0P5_4_E2P8", "ESP8684 family QFN"
    ),
    "ESP8685H4": Rule(
        "ESP8685", "QFN28_0P4_4", "qfn::ESPRESSIF_QFN28_0P4_4", "ESP8685 family QFN"
    ),
}


def module_rule(symbol: str, source_footprint: str, note: str = "official module land pattern") -> Rule:
    return Rule(symbol, source_footprint, manufacturer_footprint(source_footprint), note)


MODULE_RULES: dict[str, Rule] = {
    "ESP32-C3-MINI-1": module_rule("ESP32-C3-MINI-1", "ESP32-C3-MINI-1"),
    "ESP32-C3-MINI-1U": module_rule("ESP32-C3-MINI-1", "ESP32-C3-MINI-1U"),
    "ESP32-C3-WROOM-02": module_rule("ESP32-C3-WROOM-02", "ESP32-C3-WROOM-02"),
    "ESP32-C3-WROOM-02U": module_rule("ESP32-C3-WROOM-02", "ESP32-C3-WROOM-02U"),
    "ESP32-C5-MINI-1": module_rule("ESP32-C5-MINI-1", "ESP32-C5-MINI-1"),
    "ESP32-C5-WROOM-1": module_rule("ESP32-C5-WROOM-1", "ESP32-C5-WROOM-1"),
    "ESP32-C5-WROOM-1U": module_rule("ESP32-C5-WROOM-1U", "ESP32-C5-WROOM-1U"),
    "ESP32-C6-MINI-1": module_rule("ESP32-C6-MINI-1/U", "ESP32-C6-MINI-1"),
    "ESP32-C6-MINI-1U": module_rule("ESP32-C6-MINI-1/U", "ESP32-C6-MINI-1U"),
    "ESP32-C6-WROOM-1": module_rule("ESP32-C6-WROOM-1", "ESP32-C6-WROOM-1"),
    "ESP32-C6-WROOM-1U": module_rule("ESP32-C6-WROOM-1", "ESP32-C6-WROOM-1U"),
    "ESP32-H2-MINI-1": module_rule("ESP32-H2-MINI-1", "ESP32-H2-MINI-1"),
    "ESP32-H2-MINI-1U": module_rule("ESP32-H2-MINI-1", "ESP32-H2-MINI-1U"),
    "ESP32-MINI-1": module_rule("ESP32-MINI-1", "ESP32-MINI-1"),
    "ESP32-MINI-1U": module_rule("ESP32-MINI-1", "ESP32-MINI-1U"),
    "ESP32-PICO-MINI-02": module_rule("ESP32-PICO-MINI-02", "ESP32-PICO-MINI-02"),
    "ESP32-PICO-MINI-02U": module_rule("ESP32-PICO-MINI-02", "ESP32-PICO-MINI-02U"),
    "ESP32-S2-MINI-1": module_rule("ESP32-S2-MINI-1", "ESP32-S2-MINI-1"),
    "ESP32-S2-MINI-1U": module_rule("ESP32-S2-MINI-1", "ESP32-S2-MINI-1U"),
    "ESP32-S2-SOLO": module_rule("ESP32-S2-SOLO", "ESP32-S2-SOLO"),
    "ESP32-S2-SOLO-2U": module_rule("ESP32-S2-SOLO", "ESP32-S2-SOLO-2U"),
    "ESP32-S2-WROOM": module_rule("ESP32-S2-WROOM", "ESP32-S2-WROOM"),
    "ESP32-S2-WROOM-I": module_rule("ESP32-S2-WROOM", "ESP32-S2-WROOM"),
    "ESP32-S2-WROVER": module_rule("ESP32-S2-WROVER", "ESP32-S2-WROVER"),
    "ESP32-S2-WROVER-I": module_rule("ESP32-S2-WROVER", "ESP32-S2-WROVER"),
    "ESP32-S3-MINI-1": module_rule("ESP32-S3-MINI-1", "ESP32-S3-MINI-1"),
    "ESP32-S3-MINI-1U": module_rule("ESP32-S3-MINI-1", "ESP32-S3-MINI-1U"),
    # The non-U package is the pre-existing hand-audited implementation.
    "ESP32-S3-WROOM-1": Rule(
        "ESP32-S3-WROOM-1", "ESP32-S3-WROOM-1", "FP_ESP32_S3_WROOM_1", "hand-audited local DXF land pattern"
    ),
    "ESP32-S3-WROOM-1U": module_rule("ESP32-S3-WROOM-1", "ESP32-S3-WROOM-1U"),
    "ESP32-S3-WROOM-2": module_rule("ESP32-S3-WROOM-2", "ESP32-S3-WROOM-2"),
    "ESP32-S31-WROOM-3": module_rule("ESP32-S31-WROOM-3", "ESP32-S31-WROOM-3"),
    "ESP32-WROOM-32E": module_rule("ESP32-WROOM-E", "ESP32-WROOM-32E"),
    "ESP32-WROOM-32UE": module_rule("ESP32-WROOM-E", "ESP32-WROOM-32UE"),
    "ESP32-WROVER-E": module_rule("ESP32-WROVER-E", "ESP32-WROVER-E"),
    "ESP32-WROVER-IE": module_rule("ESP32-WROVER-E", "ESP32-WROVER-E"),
    "ESP8684-WROOM-02C": module_rule("ESP8684-WROOM-02C/U", "ESP8684-WROOM-02C"),
    "ESP8684-WROOM-02UC": module_rule("ESP8684-WROOM-02C/U", "ESP8684-WROOM-02UC"),
    "ESP8685-WROOM-06": module_rule("ESP8685-WROOM-06", "ESP8685-WROOM-06"),
}

EXISTING_MPNS = {
    "ESP32-S3",
    "ESP32-S3R8",
    "ESP32-S3-WROOM-1-N8",
    "ESP32-S3-WROOM-1-N8R2",
}

# A family symbol is not proof that every memory ordering option leaves the
# same user-visible pins available.  These exact selector rows are withheld
# until a variant-specific pin table is frozen.  Families whose *only* public
# model is inherently memory-equipped (WROVER, PICO-MINI, S31-WROOM-3) retain
# their family-specific official symbols; the mixed no-PSRAM/PSRAM families
# below do not silently share one device.
PIN_AFFECTING_MEMORY_MPNS = {
    # C5 WROOM: R8 rows are mixed with no-PSRAM N4 rows under one symbol.
    "ESP32-C5-WROOM-1-N8R8",
    "ESP32-C5-WROOM-1-N16R8",
    "ESP32-C5-WROOM-1-N32R8",
    "ESP32-C5-WROOM-1U-N8R8",
    "ESP32-C5-WROOM-1U-N16R8",
    "ESP32-C5-WROOM-1U-N32R8",
    # S2 MINI/SOLO family symbols mix no-PSRAM and R2 identities.
    "ESP32-S2-MINI-1-N4R2",
    "ESP32-S2-MINI-1U-N4R2",
    "ESP32-S2-SOLO-N4R2",
    "ESP32-S2-SOLO-2U-N4R2",
    # S3 MINI R2 needs a variant-specific availability audit.
    "ESP32-S3-MINI-1-N4R2",
    "ESP32-S3-MINI-1U-N4R2",
    # S3 WROOM Octal PSRAM makes IO35/IO36/IO37 unavailable.  R16V also
    # changes GPIO47/GPIO48 to the 1.8 V domain.  The generic symbol cannot
    # express either fact.
    "ESP32-S3-WROOM-1-N4R8",
    "ESP32-S3-WROOM-1-N8R8",
    "ESP32-S3-WROOM-1-N16R8",
    "ESP32-S3-WROOM-1-N16R16VA",
    "ESP32-S3-WROOM-1U-N4R8",
    "ESP32-S3-WROOM-1U-N8R8",
    "ESP32-S3-WROOM-1U-N16R8",
    "ESP32-S3-WROOM-2-N16R8V",
    "ESP32-S3-WROOM-2-N32R8V",
    "ESP32-S3-WROOM-2-N32R16V",
    # Classic WROOM-E R2 uses GPIO16/GPIO17 internally for PSRAM.
    "ESP32-WROOM-32E-N4R2",
    "ESP32-WROOM-32E-N8R2",
    "ESP32-WROOM-32E-N16R2",
    "ESP32-WROOM-32UE-N4R2",
    "ESP32-WROOM-32UE-N8R2",
    "ESP32-WROOM-32UE-N16R2",
}

# Reserved or internally bonded physical lands intentionally omitted from the
# schematic-oriented upstream symbol.  Their presence is proven by the exact
# official land pattern.  Naming them by number prevents false electrical
# grouping and makes their non-user-I/O status explicit.
SUPPLEMENTAL_PINS: dict[str, dict[str, str]] = {
    "ESP32-PICO-V3": {
        "25": "RESERVED_INTERNAL_25",
        "35": "RESERVED_INTERNAL_35",
        "36": "RESERVED_INTERNAL_36",
        "44": "RESERVED_INTERNAL_44",
        "45": "RESERVED_INTERNAL_45",
        "47": "RESERVED_INTERNAL_47",
        "48": "RESERVED_INTERNAL_48",
    },
    "ESP32-WROOM-E": {
        "17": "RESERVED_FLASH_17",
        "18": "RESERVED_FLASH_18",
        "19": "RESERVED_FLASH_19",
        "20": "RESERVED_FLASH_20",
        "21": "RESERVED_FLASH_21",
        "22": "RESERVED_FLASH_22",
        "32": "NC_32",
    },
    "ESP32-WROVER-E": {
        "17": "RESERVED_FLASH_17",
        "18": "RESERVED_FLASH_18",
        "19": "RESERVED_FLASH_19",
        "20": "RESERVED_FLASH_20",
        "21": "RESERVED_FLASH_21",
        "22": "RESERVED_FLASH_22",
        "27": "RESERVED_PSRAM_27",
        "28": "RESERVED_PSRAM_28",
        "32": "NC_32",
    },
    "ESP8685": {
        "18": "RESERVED_INTERNAL_18",
        "19": "RESERVED_INTERNAL_19",
        "20": "RESERVED_INTERNAL_20",
    },
}

# Generic focused footprints have dense numeric pad sets.  Manufacturer
# module sets are frozen in kicad_symbols.json directly from each source file.
FOCUSED_PAD_SETS: dict[str, set[str]] = {
    "QFN48E_0P4_6": {str(i) for i in range(1, 50)},
    "QFN32_0P5_5": {str(i) for i in range(1, 34)},
    "QFN48_0P4_6_E4P7": {str(i) for i in range(1, 50)},
    "QFN40_0P4_5_E3P3": {str(i) for i in range(1, 42)},
    "QFN32_0P5_5_E3P7X3P2": {str(i) for i in range(1, 34)},
    "QFN32_0P4_4_E2P8": {str(i) for i in range(1, 34)},
    "QFN104_0P35_10_E7P5_A": {str(i) for i in range(1, 106)},
    "QFN104_0P35_10_E7P5_B": {str(i) for i in range(1, 106)},
    "QFN56_0P4_7B": {str(i) for i in range(1, 58)},
    "QFN24_0P5_4_E2P8": {str(i) for i in range(1, 26)},
    "QFN28_0P4_4": {str(i) for i in range(1, 30)},
}

DATASHEET_OVERRIDES = {
    "ESP32-C5-MINI-1": "https://documentation.espressif.com/esp32-c5-mini-1_mini-1u_datasheet_en.html",
    "ESP32-C5-WROOM-1": "https://documentation.espressif.com/esp32-c5-wroom-1_wroom-1u_datasheet_en.pdf",
    "ESP32-C5-WROOM-1U": "https://documentation.espressif.com/esp32-c5-wroom-1_wroom-1u_datasheet_en.pdf",
    "ESP32-P4": "https://documentation.espressif.com/esp32-p4_datasheet_en.html",
    "ESP8684-WROOM-02C/U": "https://documentation.espressif.com/esp8684-wroom-02c_datasheet_en.html",
}


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True, ensure_ascii=False) + "\n").encode()


def git_head(path: pathlib.Path) -> str:
    try:
        return subprocess.check_output(
            ["git", "-C", str(path), "rev-parse", "HEAD"], text=True
        ).strip()
    except (OSError, subprocess.CalledProcessError) as exc:
        raise SystemExit(f"cannot read source commit for {path}: {exc}") from exc


def require_clean_commit(path: pathlib.Path) -> None:
    actual = git_head(path)
    if actual != KICAD_COMMIT:
        raise SystemExit(
            f"Espressif KiCad checkout is at {actual}; expected {KICAD_COMMIT} ({KICAD_TAG})"
        )
    dirty = subprocess.check_output(
        ["git", "-C", str(path), "status", "--porcelain", "--untracked-files=all"],
        text=True,
    ).strip()
    if dirty:
        raise SystemExit("Espressif KiCad checkout is dirty; refusing source import")


def tokenize_sexp(text: str) -> list[str]:
    return re.findall(r'\(|\)|"(?:\\.|[^"\\])*"|[^\s()]+', text)


def parse_sexp(text: str) -> list[Any]:
    tokens = tokenize_sexp(text)
    pos = 0

    def one() -> Any:
        nonlocal pos
        if pos >= len(tokens):
            raise ValueError("unexpected end of S-expression")
        token = tokens[pos]
        pos += 1
        if token == "(":
            out: list[Any] = []
            while pos < len(tokens) and tokens[pos] != ")":
                out.append(one())
            if pos >= len(tokens):
                raise ValueError("unclosed S-expression")
            pos += 1
            return out
        if token == ")":
            raise ValueError("unexpected ')' in S-expression")
        if token.startswith('"'):
            return json.loads(token)
        return token

    result = one()
    if pos != len(tokens) or not isinstance(result, list):
        raise ValueError("trailing tokens or non-list S-expression")
    return result


def children(node: list[Any], tag: str) -> Iterable[list[Any]]:
    for item in node:
        if isinstance(item, list) and item and item[0] == tag:
            yield item


def walk(node: Any, tag: str) -> Iterable[list[Any]]:
    if not isinstance(node, list):
        return
    if node and node[0] == tag:
        yield node
        return
    for item in node:
        yield from walk(item, tag)


def named_child(node: list[Any], tag: str) -> str:
    for child in children(node, tag):
        if len(child) >= 2 and isinstance(child[1], str):
            return child[1]
    raise ValueError(f"missing {tag} in {node[:3]!r}")


def import_selector(path: pathlib.Path) -> dict[str, Any]:
    raw = path.read_bytes()
    actual = sha256_bytes(raw)
    if actual != SELECTOR_RAW_SHA256:
        raise SystemExit(
            f"Product Selector response hash is {actual}; expected {SELECTOR_RAW_SHA256}"
        )
    source = json.loads(raw)
    results = source.get("results")
    if not isinstance(results, list):
        raise SystemExit("Product Selector response has no results array")
    rows: list[dict[str, Any]] = []
    for item in results:
        row = {
            "dimensions": item.get("dimensions", ""),
            "id": item.get("id"),
            "mpn": item.get("mpn", ""),
            "name": item.get("name", ""),
            "pins": item.get("pins"),
            "replaced_by": item.get("replacedByName", ""),
            "series": item.get("seriesName", ""),
            "status": item.get("status", ""),
            "type": item.get("type", ""),
        }
        if not row["mpn"] or not row["name"] or row["type"] not in {"SoC", "Module"}:
            raise SystemExit(f"malformed Product Selector row: {row!r}")
        rows.append(row)
    rows.sort(key=lambda r: (r["mpn"], r["id"]))
    return {
        "schema_version": SNAPSHOT_SCHEMA,
        "source": {
            "raw_sha256": SELECTOR_RAW_SHA256,
            "retrieved": SELECTOR_RETRIEVED,
            "url": SELECTOR_URL,
        },
        "rows": rows,
    }


def import_kicad(repo: pathlib.Path) -> dict[str, Any]:
    require_clean_commit(repo)
    symbol_path = repo / KICAD_SYMBOL_PATH
    symbol_raw = symbol_path.read_bytes()
    actual = sha256_bytes(symbol_raw)
    if actual != KICAD_SYMBOL_SHA256:
        raise SystemExit(
            f"Espressif symbol library hash is {actual}; expected {KICAD_SYMBOL_SHA256}"
        )
    root = parse_sexp(symbol_raw.decode())
    symbols: list[dict[str, Any]] = []
    for node in children(root, "symbol"):
        if len(node) < 2 or not isinstance(node[1], str):
            continue
        properties = {
            item[1]: item[2]
            for item in children(node, "property")
            if len(item) >= 3 and isinstance(item[1], str) and isinstance(item[2], str)
        }
        pins: list[dict[str, str]] = []
        for pin in walk(node[2:], "pin"):
            if len(pin) < 3:
                raise SystemExit(f"malformed pin in symbol {node[1]}")
            pins.append(
                {
                    "electrical_type": str(pin[1]),
                    "name": named_child(pin, "name"),
                    "number": named_child(pin, "number"),
                }
            )
        pins.sort(key=lambda p: natural_key(p["number"]))
        symbols.append(
            {
                "datasheet": properties.get("Datasheet", ""),
                "footprint": properties.get("Footprint", ""),
                "name": node[1],
                "pins": pins,
            }
        )
    symbols.sort(key=lambda s: s["name"])

    footprint_dir = repo / "footprints" / "Espressif.pretty"
    footprints: list[dict[str, Any]] = []
    for path in sorted(footprint_dir.glob("*.kicad_mod")):
        raw = path.read_bytes()
        parsed = parse_sexp(raw.decode())
        numbers = {
            str(pad[1])
            for pad in walk(parsed, "pad")
            if len(pad) >= 2 and str(pad[1])
        }
        footprints.append(
            {
                "name": path.stem,
                "pad_numbers": sorted(numbers, key=natural_key),
                "path": str(path.relative_to(repo)),
                "sha256": sha256_bytes(raw),
            }
        )
    return {
        "schema_version": SNAPSHOT_SCHEMA,
        "source": {
            "commit": KICAD_COMMIT,
            "license": KICAD_LICENSE,
            "repository": KICAD_REPOSITORY,
            "symbol_path": KICAD_SYMBOL_PATH,
            "symbol_sha256": KICAD_SYMBOL_SHA256,
            "tag": KICAD_TAG,
        },
        "symbols": symbols,
        "footprints": footprints,
    }


def import_sources(selector_path: pathlib.Path, repo: pathlib.Path) -> None:
    DATA_DIR.mkdir(parents=True, exist_ok=True)
    SELECTOR_SNAPSHOT.write_bytes(canonical_json(import_selector(selector_path)))
    SYMBOL_SNAPSHOT.write_bytes(canonical_json(import_kicad(repo)))
    print(f"imported {SELECTOR_SNAPSHOT.relative_to(ROOT)}")
    print(f"  sha256 {sha256_bytes(SELECTOR_SNAPSHOT.read_bytes())}")
    print(f"imported {SYMBOL_SNAPSHOT.relative_to(ROOT)}")
    print(f"  sha256 {sha256_bytes(SYMBOL_SNAPSHOT.read_bytes())}")


def load_snapshot(path: pathlib.Path, expected_sha: str) -> dict[str, Any]:
    raw = path.read_bytes()
    actual = sha256_bytes(raw)
    if expected_sha == "TO_BE_FILLED":
        # This branch exists only to bootstrap the first checked-in import.
        # A committed generator must replace the sentinel before review.
        pass
    elif actual != expected_sha:
        raise SystemExit(f"{path.relative_to(ROOT)} hash is {actual}; expected {expected_sha}")
    parsed = json.loads(raw)
    if canonical_json(parsed) != raw:
        raise SystemExit(f"{path.relative_to(ROOT)} is not canonical JSON")
    if parsed.get("schema_version") != SNAPSHOT_SCHEMA:
        raise SystemExit(f"unsupported snapshot schema in {path.relative_to(ROOT)}")
    return parsed


def natural_key(value: str) -> tuple[Any, ...]:
    return tuple(int(x) if x.isdigit() else x for x in re.split(r"([0-9]+)", value))


def validate_selector(snapshot: dict[str, Any]) -> list[dict[str, Any]]:
    rows = snapshot["rows"]
    if len(rows) != EXPECTED_ROWS:
        raise SystemExit(f"selector snapshot has {len(rows)} rows; expected {EXPECTED_ROWS}")
    mpns = [r["mpn"] for r in rows]
    if len(set(mpns)) != len(mpns):
        duplicates = [m for m, n in collections.Counter(mpns).items() if n != 1]
        raise SystemExit(f"duplicate exact selector MPNs: {duplicates}")
    lifecycles = dict(sorted(collections.Counter(r["status"] for r in rows).items()))
    if lifecycles != EXPECTED_LIFECYCLES:
        raise SystemExit(f"selector lifecycle counts changed: {lifecycles!r}")
    return rows


def symbol_map(snapshot: dict[str, Any]) -> dict[str, dict[str, Any]]:
    out = {s["name"]: s for s in snapshot["symbols"]}
    if len(out) != len(snapshot["symbols"]):
        raise SystemExit("duplicate symbol names in Espressif snapshot")
    return out


def footprint_map(snapshot: dict[str, Any]) -> dict[str, dict[str, Any]]:
    out = {f["name"]: f for f in snapshot["footprints"]}
    if len(out) != len(snapshot["footprints"]):
        raise SystemExit("duplicate footprint names in Espressif snapshot")
    return out


def rule_for(row: dict[str, Any]) -> Rule | None:
    if row["mpn"] in PIN_AFFECTING_MEMORY_MPNS:
        return None
    if row["type"] == "SoC":
        return SOC_RULES.get(row["mpn"])
    return MODULE_RULES.get(row["name"])


def omission_reason(row: dict[str, Any]) -> str:
    name = row["name"]
    mpn = row["mpn"]
    if mpn in PIN_AFFECTING_MEMORY_MPNS:
        return "memory variant changes pin availability or I/O voltage; generic family symbol is insufficient"
    if name in {"ESP8266EX", "ESP8285"} or name.startswith("ESP-WROOM"):
        return "outside this ESP32-lineage package (ESP8266/ESP8285 family)"
    if name == "ESP32-WROOM-DA":
        return "official symbol and footprint disagree on physical pad numbers"
    if name == "ESP32-C5-MINI-1U":
        return "official 3.2.1 library has no exact MINI-1U footprint"
    if name in {"ESP32-S2-SOLO-2", "ESP32-S2-SOLO-U"}:
        return "no directly attributed official footprint for this antenna/package name"
    if row["type"] == "SoC":
        if name == "ESP32" and "Q6" not in mpn:
            return "official symbol is fixed to the 6x6 mm Q6 package, not this 5x5 mm variant"
        if name == "ESP32-S3":
            return "integrated-memory variant lacks a variant-specific audited pin model"
        if name in {"ESP32-PICO-V3-02", "ESP32-PICO-D4", "ESP32-S3-PICO-1"}:
            return "no exact variant-specific official symbol/pin model in the pinned source"
        if name in {"ESP32-C3", "ESP32-S2", "ESP32-S2F"}:
            return "generic family symbol does not prove this embedded-memory variant's pin semantics"
        return "pinned official sources do not provide a complete exact symbol-and-footprint join"
    return "pinned official sources do not provide a complete exact symbol-and-footprint pair"


def pins_for_symbol(symbol: dict[str, Any]) -> list[dict[str, str]]:
    pins = [dict(p) for p in symbol["pins"]]
    existing_numbers = {p["number"] for p in pins}
    for number, name in SUPPLEMENTAL_PINS.get(symbol["name"], {}).items():
        if number in existing_numbers:
            raise SystemExit(f"supplemental pin {symbol['name']}:{number} already exists upstream")
        pins.append(
            {
                "electrical_type": "no_connect",
                "name": name,
                "number": number,
                "supplemental": "physical land present in exact official footprint; reserved/internal",
            }
        )
    pins.sort(key=lambda p: natural_key(p["number"]))
    numbers = [p["number"] for p in pins]
    if len(numbers) != len(set(numbers)):
        raise SystemExit(f"symbol {symbol['name']} contains duplicate physical pin numbers")
    return pins


def expected_pad_set(rule: Rule, footprints: dict[str, dict[str, Any]]) -> set[str]:
    if rule.source_footprint in FOCUSED_PAD_SETS:
        return FOCUSED_PAD_SETS[rule.source_footprint]
    source = footprints.get(rule.source_footprint)
    if source is None:
        # Direct-ASC SiP aliases are imported by the geometry generator and
        # have dense pad numbers established by that exact source.
        if rule.source_footprint == "ESP32-PICO-V3":
            return {str(i) for i in range(1, 50)}
        raise SystemExit(f"missing source footprint {rule.source_footprint!r}")
    return set(source["pad_numbers"])


def validate_rules(
    rows: list[dict[str, Any]], symbols: dict[str, dict[str, Any]], footprints: dict[str, dict[str, Any]]
) -> None:
    row_mpns = {r["mpn"] for r in rows}
    unknown = set(SOC_RULES) - row_mpns
    if unknown:
        raise SystemExit(f"SoC admission rules name absent selector MPNs: {sorted(unknown)}")
    row_families = {r["name"] for r in rows if r["type"] == "Module"}
    unknown = set(MODULE_RULES) - row_families
    if unknown:
        raise SystemExit(f"module admission rules name absent selector families: {sorted(unknown)}")

    for rule in sorted(set(SOC_RULES.values()) | set(MODULE_RULES.values()), key=lambda r: (r.symbol, r.source_footprint)):
        symbol = symbols.get(rule.symbol)
        if symbol is None:
            raise SystemExit(f"admission rule references missing symbol {rule.symbol!r}")
        pin_set = {p["number"] for p in pins_for_symbol(symbol)}
        pad_set = expected_pad_set(rule, footprints)
        if pin_set != pad_set:
            missing = sorted(pad_set - pin_set, key=natural_key)
            extra = sorted(pin_set - pad_set, key=natural_key)
            raise SystemExit(
                f"pad-set mismatch for {rule.symbol} -> {rule.source_footprint}: "
                f"missing symbol pins {missing}, extra symbol pins {extra}"
            )


def datasheet_for(symbol: dict[str, Any]) -> str:
    return DATASHEET_OVERRIDES.get(symbol["name"], symbol.get("datasheet", ""))


def pin_primary_name(full_name: str) -> str:
    # KiCad's slash-separated names are aliases.  Preserve the manufacturer's
    # first-listed name as the API name and retain the full alias string in a
    # generated comment.
    return full_name.split("/", 1)[0]


def pin_role(items: list[dict[str, str]]) -> str:
    types = {item["electrical_type"] for item in items}
    if "power_out" in types:
        return "power_out"
    if "power_in" in types:
        return "power_in"
    if "bidirectional" in types:
        return "bidirectional"
    if "output" in types:
        return "output"
    if "input" in types:
        return "input"
    if types <= {"passive", "no_connect"}:
        return "passive"
    raise SystemExit(f"unsupported mixed KiCad electrical types: {sorted(types)}")


def pin_required(name: str, role: str) -> bool:
    upper = name.upper()
    if NC_NAME.match(upper):
        return False
    if role in {"power_in", "power_out"}:
        return True
    if upper in {"EN", "CHIP_EN", "CHIP_PU", "ENABLE", "LNA_IN", "ANT", "XTAL_N", "XTAL_P"}:
        return True
    if upper.startswith(("VDD", "VDDA", "VSS", "GND", "3V3")):
        return True
    return False


def logical_pins(symbol: dict[str, Any]) -> list[dict[str, Any]]:
    groups: dict[str, list[dict[str, str]]] = collections.defaultdict(list)
    used: set[str] = set()
    for pin in pins_for_symbol(symbol):
        primary = pin_primary_name(pin["name"])
        # A KiCad `no_connect` classification is authoritative even when its
        # display name lists a possible die-level GPIO alias.  Never expose
        # that alias as a connectable-looking public pin on an integrated
        # memory variant (ESP32-C5 pins 25..32 are the current example).
        if pin["electrical_type"] == "no_connect" and not NC_NAME.match(ident(primary).upper()):
            primary = f"RESERVED_{pin['number']}"
        base = ident(primary)
        if NC_NAME.match(base.upper()):
            base = f"{base}_{pin['number']}" if not base.endswith("_" + pin["number"]) else base
        candidate = base
        if candidate in used and candidate not in groups:
            candidate = f"{base}_{pin['number']}"
        groups[candidate].append(pin)
        used.add(candidate)
    result: list[dict[str, Any]] = []
    for name, items in groups.items():
        role = pin_role(items)
        result.append(
            {
                "comments": [
                    f"{item['number']}: {item['name']} [{item['electrical_type']}]"
                    + (f"; {item['supplemental']}" if item.get("supplemental") else "")
                    for item in items
                ],
                "name": name,
                "numbers": sorted((item["number"] for item in items), key=natural_key),
                "required": pin_required(name, role),
                "role": role,
            }
        )
    result.sort(key=lambda p: natural_key(p["numbers"][0]))
    return result


def device_name(symbol_name: str) -> str:
    return "DEV_" + ident(symbol_name).upper()


def part_name(mpn: str) -> str:
    return ident(mpn).upper()


def rendered_pin_lines(pin: dict[str, Any]) -> list[str]:
    """Mirror RFC-009's 100-column pin-bus wrapping exactly."""
    obligation = "required" if pin["required"] else "optional"
    prefix = f"{obligation} {pin['name']}: "
    suffix = f" [{pin['role']}]"
    numbers = pin["numbers"]
    comment = " | ".join(pin["comments"])
    indent = "        "
    oneline = prefix + ", ".join(numbers) + suffix
    if len(indent) + len(oneline) <= 100 or len(numbers) <= 1:
        return [f"{indent}{oneline} // upstream {comment}"]

    align = " " * len(prefix)
    current = prefix
    first_on_line = True
    out: list[str] = []
    for index, number in enumerate(numbers):
        piece = number + ("," if index + 1 < len(numbers) else suffix)
        extra = 0 if first_on_line else 1
        if not first_on_line and len(indent) + len(current) + extra + len(piece) > 100:
            out.append(indent + current)
            current = align
            first_on_line = True
        if not first_on_line:
            current += " "
        current += piece
        first_on_line = False
    out.append(f"{indent}{current} // upstream {comment}")
    return out


def render_devices(selected: list[tuple[dict[str, Any], Rule]], symbols: dict[str, dict[str, Any]]) -> str:
    symbol_names = sorted({rule.symbol for _, rule in selected if rule.symbol != "ESP32-S3-WROOM-1"})
    lines = [
        "// @generated by tools/gen_esp32.py; do not edit by hand.",
        "// Pin maps: official Espressif KiCad library 3.2.1 at",
        f"// {KICAD_COMMIT} ({KICAD_SYMBOL_SHA256}).",
        "// Every device's physical pin set is checked against its exact source footprint.",
        "",
    ]
    for symbol_name in symbol_names:
        symbol = symbols[symbol_name]
        doc_url = datasheet_for(symbol)
        if not doc_url:
            raise SystemExit(f"selected symbol {symbol_name!r} has no official datasheet URL")
        lines.extend(
            [
                f"// Upstream symbol: {symbol_name}",
                f"// Official datasheet: {doc_url}",
                f'#[doc("{DOC_CATALOG}")]',
                f"pub device {device_name(symbol_name)} {{",
                "    pins {",
            ]
        )
        for pin in logical_pins(symbol):
            lines.extend(rendered_pin_lines(pin))
        lines.extend(
            [
                "    }",
                "}",
                "",
                f"impl IC for {device_name(symbol_name)} {{}}",
                "",
            ]
        )
    return "\n".join(lines).rstrip() + "\n"


def render_parts(selected: list[tuple[dict[str, Any], Rule]]) -> str:
    lines = [
        "// @generated by tools/gen_esp32.py; do not edit by hand.",
        "// One public part is emitted per exact Product Selector MPN; no marketing-family alternates.",
        "",
    ]
    seen_names: dict[str, str] = {}
    for row, rule in sorted(selected, key=lambda pair: pair[0]["mpn"]):
        if row["mpn"] in EXISTING_MPNS:
            continue
        name = part_name(row["mpn"])
        if name in seen_names:
            raise SystemExit(f"part identifier collision: {name} for {seen_names[name]} and {row['mpn']}")
        seen_names[name] = row["mpn"]
        device = (
            "espressif_esp32::modules::wroom_s3::ESP32_S3_WROOM_1"
            if rule.symbol == "ESP32-S3-WROOM-1"
            else device_name(rule.symbol)
        )
        primary = (
            f'    primary {{ mfr: "Espressif", mpn: "{row["mpn"]}", '
            f"footprint: {rule.footprint_ref} }}"
        )
        primary_lines = [primary]
        if len(primary) > 100:
            primary_lines = [
                f'    primary {{ mfr: "Espressif", mpn: "{row["mpn"]}",',
                f"              footprint: {rule.footprint_ref} }}",
            ]
        lines.extend(
            [
                f"// Selector family: {row['name']}; lifecycle: {row['status']}; {rule.note}.",
                f'#[doc("{DOC_CATALOG}")]',
                f"pub part {name}: {device} {{",
                *primary_lines,
                "}",
                "",
            ]
        )
    return "\n".join(lines).rstrip() + "\n"


def md(value: Any) -> str:
    return str(value).replace("|", "\\|").replace("\n", " ")


def render_docs(
    rows: list[dict[str, Any]],
    selected: list[tuple[dict[str, Any], Rule]],
    excluded: list[tuple[dict[str, Any], str]],
    symbols: dict[str, dict[str, Any]],
    selector_sha: str,
    symbol_sha: str,
) -> str:
    selected_map = {row["mpn"]: rule for row, rule in selected}
    generated_count = sum(row["mpn"] not in EXISTING_MPNS for row, _ in selected)
    lines = [
        "# Espressif ESP32 exact-part catalog",
        "",
        "This file is generated by `tools/gen_esp32.py`. It is the local, deterministic",
        "source index used in place of copying one datasheet binary per ordering code.",
        "Every admitted part has one exact Espressif Product Selector MPN, one audited",
        "official pin symbol, and one concrete footprint whose complete pad-number set",
        "matches that device. Lifecycle state is descriptive; EOL/NRND parts remain valid",
        "exact identities and are intentionally represented when the evidence join is complete.",
        "",
        "## Frozen authorities",
        "",
        f"- Product Selector API: [{SELECTOR_URL}]({SELECTOR_URL}), retrieved {SELECTOR_RETRIEVED}; raw SHA-256 `{SELECTOR_RAW_SHA256}`; normalized snapshot SHA-256 `{selector_sha}`.",
        f"- Espressif KiCad library: [{KICAD_REPOSITORY}]({KICAD_REPOSITORY}), tag `{KICAD_TAG}`, commit `{KICAD_COMMIT}`; `{KICAD_SYMBOL_PATH}` SHA-256 `{KICAD_SYMBOL_SHA256}`; normalized snapshot SHA-256 `{symbol_sha}`.",
        f"- Espressif KiCad source license: `{KICAD_LICENSE}`. The package carries the corresponding license notice.",
        "- Generic QFN geometry is dependency-owned by `qfn`; manufacturer module/SiP geometry remains in this package. Geometry source commits and per-file checksums are recorded beside the generated footprints.",
        "",
        "Refresh is explicit and normal regeneration is offline:",
        "",
        "```sh",
        "python3 tools/gen_esp32.py --import-sources /path/to/espressif-products.json /path/to/kicad-libraries",
        "python3 tools/gen_esp32.py",
        "python3 tools/gen_esp32.py --check",
        "```",
        "",
        "## Coverage",
        "",
        f"The frozen selector contains **{len(rows)}** unique exact MPNs: **{len(selected)} admitted** ({len(EXISTING_MPNS)} preserved hand-audited declarations + {generated_count} generated declarations) and **{len(excluded)} omitted**. `{len(selected)} + {len(excluded)} = {len(rows)}`.",
        "",
        "Lifecycle counts in the complete selector snapshot:",
        "",
        "| Lifecycle | Rows |",
        "|---|---:|",
    ]
    lifecycle = collections.Counter(row["status"] for row in rows)
    for status in sorted(lifecycle):
        lines.append(f"| {md(status)} | {lifecycle[status]} |")
    lines.extend(
        [
            "",
            "## Admitted exact parts",
            "",
            "| Exact MPN | API declaration | Type/family | Lifecycle | Device source | Footprint | Official datasheet |",
            "|---|---|---|---|---|---|---|",
        ]
    )
    for row in sorted(rows, key=lambda r: r["mpn"]):
        rule = selected_map.get(row["mpn"])
        if rule is None:
            continue
        symbol = symbols.get(rule.symbol)
        doc_url = datasheet_for(symbol) if symbol else ""
        declaration = part_name(row["mpn"])
        if row["mpn"] in EXISTING_MPNS:
            declaration += " (preserved)"
        device_source = (
            "hand-audited local ESP32-S3 model"
            if row["mpn"] in {"ESP32-S3", "ESP32-S3R8"}
            else f"Espressif KiCad `{rule.symbol}`"
        )
        link = f"[source]({doc_url})" if doc_url else "local hand-audited PDF"
        lines.append(
            f"| `{md(row['mpn'])}` | `{declaration}` | {md(row['type'])} / {md(row['name'])} | {md(row['status'])} | {md(device_source)} | `{md(rule.footprint_ref)}` | {link} |"
        )
    lines.extend(
        [
            "",
            "## Deliberate omission ledger",
            "",
            "An omitted selector identity is not silently generalized to a neighboring part.",
            "The reason below is deterministic and every one of the 318 source rows appears",
            "exactly once in either this table or the admitted table above.",
            "",
            "| Exact MPN | Type/family | Lifecycle | Dimensions | Reason |",
            "|---|---|---|---|---|",
        ]
    )
    for row, reason in sorted(excluded, key=lambda pair: pair[0]["mpn"]):
        lines.append(
            f"| `{md(row['mpn'])}` | {md(row['type'])} / {md(row['name'])} | {md(row['status'])} | {md(row['dimensions'])} | {md(reason)} |"
        )
    lines.extend(
        [
            "",
            "## Modeling notes",
            "",
            "- Slash-separated KiCad pin aliases are retained verbatim in generated comments; the manufacturer's first-listed alias is the CoHDL API name.",
            "- Supply, ground, enable, RF, and primary crystal pins are required. Ordinary GPIO and optional functions remain optional. `no_connect` and audited reserved/internal lands are passive and optional, but stay in the physical pad set so a footprint can never pass by omission.",
            "- ESP32-WROOM-32E/32UE, ESP32-WROVER-E/IE, ESP32-PICO-V3, and ESP8685 upstream symbols omit reserved or internally bonded physical land numbers. The generator adds individually named reserved pins only after proving those lands exist in the exact official footprint; they are never exposed as GPIO aliases.",
            "- Antenna keepout outlines in generated module footprints are fabrication guidance, not enforceable board-level copper keepouts in the current language. Enforce the manufacturer keepout during PCB layout and perform production stencil/thermal-via review.",
            "- Product Selector `pins` is retained as catalog metadata but is not used as a package-pad count: for modules and internally bonded devices it is not consistently the physical-land total. The symbol-to-footprint pad-set equality is the admission gate.",
        ]
    )
    return "\n".join(lines).rstrip() + "\n"


def classify(
    rows: list[dict[str, Any]], symbols: dict[str, dict[str, Any]]
) -> tuple[list[tuple[dict[str, Any], Rule]], list[tuple[dict[str, Any], str]]]:
    selected: list[tuple[dict[str, Any], Rule]] = []
    excluded: list[tuple[dict[str, Any], str]] = []
    for row in rows:
        rule = rule_for(row)
        if row["mpn"] in EXISTING_MPNS:
            if rule is None:
                if row["mpn"] == "ESP32-S3":
                    rule = Rule("ESP32-S3", "QFN56_0P4_7B", "qfn::ESPRESSIF_QFN56_0P4_7B", "hand-audited local model")
                elif row["mpn"] == "ESP32-S3R8":
                    rule = Rule("ESP32-S3", "QFN56_0P4_7B", "qfn::ESPRESSIF_QFN56_0P4_7B", "hand-audited local R8 model")
                else:
                    rule = MODULE_RULES[row["name"]]
            selected.append((row, rule))
        elif rule is not None:
            selected.append((row, rule))
        else:
            excluded.append((row, omission_reason(row)))
    if len(selected) + len(excluded) != len(rows):
        raise SystemExit("coverage arithmetic failed")
    if {r["mpn"] for r, _ in selected} & {r["mpn"] for r, _ in excluded}:
        raise SystemExit("selector row appears in both admitted and omitted ledgers")
    return selected, excluded


def write_or_check(path: pathlib.Path, content: str, check: bool) -> bool:
    expected = content.encode()
    if check:
        actual = path.read_bytes() if path.exists() else b""
        if actual != expected:
            print(f"out of date: {path.relative_to(ROOT)}", file=sys.stderr)
            return False
        return True
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(expected)
    print(f"wrote {path.relative_to(ROOT)}")
    return True


def generate(check: bool) -> bool:
    selector_raw = SELECTOR_SNAPSHOT.read_bytes()
    symbol_raw = SYMBOL_SNAPSHOT.read_bytes()
    selector = load_snapshot(SELECTOR_SNAPSHOT, SELECTOR_SNAPSHOT_SHA256)
    kicad = load_snapshot(SYMBOL_SNAPSHOT, SYMBOL_SNAPSHOT_SHA256)
    rows = validate_selector(selector)
    symbols = symbol_map(kicad)
    footprints = footprint_map(kicad)
    validate_rules(rows, symbols, footprints)
    selected, excluded = classify(rows, symbols)

    outputs = {
        DEVICE_OUT: render_devices(selected, symbols),
        PART_OUT: render_parts(selected),
        DOC_OUT: render_docs(
            rows,
            selected,
            excluded,
            symbols,
            sha256_bytes(selector_raw),
            sha256_bytes(symbol_raw),
        ),
    }
    ok = all(write_or_check(path, content, check) for path, content in outputs.items())
    print(
        f"ESP32 catalog: {len(rows)} selector rows; {len(selected)} admitted "
        f"({len(EXISTING_MPNS)} existing), {len(excluded)} omitted; "
        f"{len({rule.symbol for _, rule in selected})} source symbols"
    )
    return ok


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="fail if generated files differ")
    parser.add_argument(
        "--import-sources",
        nargs=2,
        metavar=("SELECTOR_JSON", "ESPRESSIF_KICAD"),
        help="refresh normalized snapshots from the pinned authorities",
    )
    args = parser.parse_args()
    if args.check and args.import_sources:
        parser.error("--check and --import-sources cannot be combined")
    if args.import_sources:
        import_sources(pathlib.Path(args.import_sources[0]), pathlib.Path(args.import_sources[1]))
    return 0 if generate(args.check) else 1


if __name__ == "__main__":
    raise SystemExit(main())
