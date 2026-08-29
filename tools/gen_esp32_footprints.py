#!/usr/bin/env python3
"""Freeze and generate the focused ESP32 land-pattern catalog.

Normal generation is deliberately offline::

    python3 tools/gen_esp32_footprints.py
    python3 tools/gen_esp32_footprints.py --check

Maintainers refresh the normalized snapshot only from all three pinned inputs::

    python3 tools/gen_esp32_footprints.py --import-sources \
        /path/to/espressif-kicad /path/to/kicad-footprints \
        --direct-cad-root /path/to/downloaded-espressif-cad

The two Git repositories are open-licensed and their exact notices are shipped
with the generated packages.  Espressif's website PADS files are dimensional
evidence without a redistribution grant: their URL and SHA-256 are retained,
but only normalized geometric facts are checked in.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import re
import subprocess
import sys
import zipfile
from collections import Counter
from decimal import Decimal, InvalidOperation, ROUND_HALF_UP


ROOT = pathlib.Path(__file__).resolve().parent.parent
DATA = ROOT / "tools" / "esp32_footprint_data"
SNAPSHOT = DATA / "footprints.json"
ESP_LICENSE = DATA / "LICENSE.espressif-kicad.md"
KICAD_LICENSE = DATA / "LICENSE.kicad.md"

ESP_REPOSITORY = "https://github.com/espressif/kicad-libraries.git"
ESP_COMMIT = "1dfc3110895c9cd62daf332f49c49ee0ee200831"
ESP_LICENSE_SHA256 = "6eb43c2548ac6714db47ccbd62354bd194e918f606b071a5e9893680b941d75a"
# The checked-in Markdown has the repository text plus the conventional final
# newline required by source-control tooling.  The upstream byte hash above is
# still verified during import and retained in the normalized provenance.
ESP_NOTICE_SHA256 = "891a1de695e924bf0725d11e210d5848a7ac1d0cb12829c07f041f8740db13cd"
KICAD_REPOSITORY = "https://gitlab.com/kicad/libraries/kicad-footprints.git"
KICAD_COMMIT = "819223b66f96508feaeaa305301b5e6bb5c1038b"
KICAD_LICENSE_SHA256 = "45d2bce75e5a4208f5afb01b8fb2c406e700371c4fe2b5f5cd5c443d46db4d8f"
DIRECT_SOURCE_PAGE = "https://www.espressif.com/en/products/socs"
DBU_PER_MM = Decimal("1500000")
MM_QUANTUM = Decimal("0.000000000000001")

# Updated after the complete normalized geometry has been reviewed.  Import
# mode deliberately refuses to move this pin implicitly.
SNAPSHOT_SHA256 = "fca1484784d12c8224bdbaefda665a38d1baae89544f76aaabc15d4b123101b1"


class ImportFailure(ValueError):
    """A pinned source contains data this exact projection cannot preserve."""


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: pathlib.Path) -> str:
    return sha256_bytes(path.read_bytes())


def git_head(path: pathlib.Path) -> str:
    try:
        return subprocess.check_output(
            ["git", "-C", str(path), "rev-parse", "HEAD"],
            text=True,
            stderr=subprocess.STDOUT,
        ).strip()
    except (OSError, subprocess.CalledProcessError) as exc:
        raise ImportFailure(f"cannot read git commit for {path}: {exc}") from exc


def require_clean_checkout(path: pathlib.Path, commit: str) -> None:
    actual = git_head(path)
    if actual != commit:
        raise ImportFailure(f"{path}: checkout is {actual}, expected {commit}")
    dirty = subprocess.check_output(
        ["git", "-C", str(path), "status", "--porcelain", "--untracked-files=all"],
        text=True,
    ).strip()
    if dirty:
        raise ImportFailure(f"{path}: checkout is dirty; refusing source import")


def decimal(value: str | int, context: str) -> Decimal:
    try:
        result = Decimal(value)
    except InvalidOperation as exc:
        raise ImportFailure(f"{context}: invalid decimal {value!r}") from exc
    if not result.is_finite():
        raise ImportFailure(f"{context}: non-finite decimal {value!r}")
    return result


def decimal_text(value: Decimal) -> str:
    value = value.quantize(MM_QUANTUM, rounding=ROUND_HALF_UP)
    text = format(value, "f")
    if "." in text:
        text = text.rstrip("0").rstrip(".")
    return "0" if text in {"", "-0"} else text


def dbu(value: str | int) -> Decimal:
    # PADS' 1/1,500,000 mm database unit is recurring in base ten.  CoHDL
    # Length literals accept at most 15 fractional digits, so normalize once,
    # deterministically, at the boundary.  Error is bounded to 0.5 fm.
    return (decimal(value, "PADS database coordinate") / DBU_PER_MM).quantize(
        MM_QUANTUM, rounding=ROUND_HALF_UP
    )


def mm(value: str | Decimal) -> str:
    number = value if isinstance(value, Decimal) else decimal(value, "millimetres")
    return f"{decimal_text(number)}mm"


def public_module_name(stem: str) -> str:
    normalized = re.sub(r"[^A-Za-z0-9]+", "_", stem).strip("_").upper()
    if not normalized:
        raise ImportFailure(f"cannot normalize module name {stem!r}")
    return f"FP_{normalized}"


# Production module lands in Espressif's stable 3.2.1 library.  Development
# boards and alternate hand-soldering/thermal-via patterns are intentionally
# excluded; the hand-audited ESP32-S3-WROOM-1 remains in footprints.cohdl.
MODULE_STEMS = (
    "ESP32-C3-MINI-1", "ESP32-C3-MINI-1U", "ESP32-C3-WROOM-02",
    "ESP32-C3-WROOM-02U", "ESP32-C5-MINI-1", "ESP32-C5-WROOM-1",
    "ESP32-C5-WROOM-1U", "ESP32-C6-MINI-1", "ESP32-C6-MINI-1U",
    "ESP32-C6-WROOM-1", "ESP32-C6-WROOM-1U", "ESP32-H2-MINI-1",
    "ESP32-H2-MINI-1U", "ESP32-MINI-1", "ESP32-MINI-1U",
    "ESP32-PICO-MINI-02", "ESP32-PICO-MINI-02U", "ESP32-S2-MINI-1",
    "ESP32-S2-MINI-1U", "ESP32-S2-SOLO-2U", "ESP32-S2-SOLO",
    "ESP32-S2-WROOM", "ESP32-S2-WROVER", "ESP32-S3-MINI-1",
    "ESP32-S3-MINI-1U", "ESP32-S3-WROOM-1U", "ESP32-S3-WROOM-2",
    "ESP32-S31-WROOM-3", "ESP32-WROOM-32E", "ESP32-WROOM-32UE",
    "ESP32-WROOM-DA", "ESP32-WROVER-E", "ESP8684-WROOM-02C",
    "ESP8684-WROOM-02UC", "ESP8685-WROOM-06", "ESP8685-Wroom-05",
)


# Public name, package owner, source-relative path (or ZIP member), source
# SHA-256, and source URL.  Aliases retain distinct public product identities
# even when Espressif publishes byte-identical source files.
DIRECT_SPECS = (
    ("QFN32_0P5_5", "qfn", "soc/ESP32-C3_Footprint_0.asc", None,
     "f565919ef57f7734604d0818f25848098b125b20db6947c3c0ec868c78a010d1",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-C3_Footprint_0.asc"),
    ("QFN48_0P4_6_E4P7", "qfn", "soc/ESP32-C5_0.asc", None,
     "d1e386f61c74be65ebb53035554c127698cba0edb48e8d1b082601a0a41cd7b4",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-C5_0.asc"),
    ("QFN40_0P4_5_E3P3", "qfn", "soc/ESP32-C6_Footprint_0.zip",
     "ESP32-C6_Footprint/ESP32-C6_Footprint.asc",
     "30c99a2d7f22bdbfa81cfb994156c22e66f9d4a51957593853ef5713400eceab",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-C6_Footprint_0.zip"),
    ("QFN32_0P5_5_E3P7X3P2", "qfn", "soc/ESP32-C6_Footprint_0.zip",
     "ESP32-C6_Footprint/ESP32-C6F_Footprint.asc",
     "9247a0938b84843b0844b789e0231f7ca7ad4a022d949add2d8952581c96ebde",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-C6_Footprint_0.zip"),
    ("QFN48EB_0P35_5", "qfn", "soc/ESP32-D0WD-V3_Footprint.asc", None,
     "a04e3e6e1da3dc5e98b57fbfef69ca121ec17c7688180c78e7f584eb068edcda",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-D0WD-V3_Footprint.asc"),
    ("QFN48E_0P4_6", "qfn", "soc/ESP32-D0WDQ6-V3_Footprint.asc", None,
     "92fed76721724481494f8f6496b9386542417f429e4c7ebf9dd174177187f466",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-D0WDQ6-V3_Footprint.asc"),
    ("QFN36_0P35_4_E2P8", "qfn", "soc/ESP32-H21_Footprint_0.asc", None,
     "c267dcf7a277a9c342b0a48b8b222f6313ae8b78430b20c39a76436a794d9fc4",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-H21_Footprint_0.asc"),
    ("QFN32_0P4_4_E2P8", "qfn", "soc/ESP32-H2_Footprint.asc", None,
     "8e1d435105f95286e0b1d5afe3b7760b47e98565ed27884a0640bd548fa9fda1",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-H2_Footprint.asc"),
    ("QFN56_0P35_6_E4P7", "qfn", "soc/ESP32-H4_Footprint_0.asc", None,
     "d1c05dd9aabfa1fb7f0c5f8ce97273b0b22e63a18d333edc614447bc77ba7dd5",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-H4_Footprint_0.asc"),
    ("QFN104_0P35_10_E7P5_A", "qfn", "soc/ESP32-P4.asc", None,
     "459b2e11feb477aa21ef6bd80dd6bfd1cd3ddf9363398db24d52af50d182161b",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-P4.asc"),
    ("QFN104_0P35_10_E7P5_B", "qfn", "soc/ESP32-P4_16x16_Footprint_0.asc", None,
     "104a2fbac19dc0a037fd917040525c7877e23bbb5ac2b293c842acca7a31c507",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-P4_16x16_Footprint_0.asc"),
    ("QFN56_0P4_7B", "qfn", "soc/ESP32-S3_Footprint.asc", None,
     "2f0e059f9ecef4350413068280a14e6f1f3e3792f512116b27138d4a6810fb5c",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-S3_Footprint.asc"),
    ("QFN80_0P35_8_E6P5_A", "qfn", "soc/ESP32-S31_Footprint.asc", None,
     "0e69529ae93b580965f518214b2e69fce68af9af9231cb070aeef8387aa586fd",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-S31_Footprint.asc"),
    ("QFN24_0P5_4_E2P8", "qfn", "soc/ESP8684_Footprint.asc", None,
     "f2edebc94dd430731152689facc15fde79c9a5207f52dee5347edde0d0069fe5",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP8684_Footprint.asc"),
    ("QFN28_0P4_4", "qfn", "soc/ESP8685_Footprint.asc", None,
     "57e8d1055f1ca3b60669c7b7c6be64e89e4e8091d82ae77f935f992e75f80941",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP8685_Footprint.asc"),
    ("FP_ESP32_PICO_D4", "@espressif/esp32", "soc/ESP32-PICO-D4_Footprint.asc", None,
     "a09990da3e9f2e204af79767be55368af256d87d3a11770fa222479ceabdff15",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-PICO-D4_Footprint.asc"),
    ("FP_ESP32_PICO_V3", "@espressif/esp32", "soc/ESP32-PICO-V3_Footprint.asc", None,
     "5164253b83262982cfa3f203ecb10f60679646bd0bc89e962a9be113df743efe",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-PICO-V3_Footprint.asc"),
    ("FP_ESP32_PICO_V3_02", "@espressif/esp32", "soc/ESP32-PICO-V3-02_Footprint.asc", None,
     "5164253b83262982cfa3f203ecb10f60679646bd0bc89e962a9be113df743efe",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-PICO-V3-02_Footprint.asc"),
    ("FP_ESP32_S3_PICO_1", "@espressif/esp32", "soc/ESP32-S3-PICO-1_Footprint.asc", None,
     "2f0e059f9ecef4350413068280a14e6f1f3e3792f512116b27138d4a6810fb5c",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-S3-PICO-1_Footprint.asc"),
)

# Separate current product downloads that normalize to the same reusable land.
# Import mode proves geometry equality rather than merely trusting the matching
# decal name or byte hash.
DIRECT_CORROBORATION = (
    ("ESPRESSIF_QFN40_0P4_5_E3P3", "QFN40_0P4_5_E3P3", "qfn",
     "soc/ESP32-C61_Footprint_0.asc", None,
     "30c99a2d7f22bdbfa81cfb994156c22e66f9d4a51957593853ef5713400eceab",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-C61_Footprint_0.asc"),
    ("ESPRESSIF_QFN56_0P4_7B", "QFN56_0P4_7B", "qfn",
     "soc/ESP32-S2_Footprint.asc", None,
     "7b0c548f0e7673a9edfe2802d010f29e8e7e7e7c1ed05d10a30d084f5ffb7ed3",
     "https://www.espressif.com/sites/default/files/chips-dxf/ESP32-S2_Footprint.asc"),
)


GENERIC_SECONDARY = (
    "QFN-24-1EP_4x4mm_P0.5mm_EP2.8x2.8mm",
    "QFN-32-1EP_5x5mm_P0.5mm_EP3.45x3.45mm",
    "QFN-32-1EP_5x5mm_P0.5mm_EP3.6x3.6mm",
    "QFN-40-1EP_5x5mm_P0.4mm_EP3.6x3.6mm",
    "QFN-48-1EP_6x6mm_P0.4mm_EP4.3x4.3mm",
    "QFN-48-1EP_6x6mm_P0.4mm_EP4.6x4.6mm",
    "QFN-48-1EP_7x7mm_P0.5mm_EP5.15x5.15mm",
    "QFN-56-1EP_7x7mm_P0.4mm_EP5.6x5.6mm",
)


# --- KiCad S-expression reader -------------------------------------------------

Sexp = list["str | Sexp"]


def tokens(text: str, source: pathlib.Path) -> list[str]:
    out: list[str] = []
    i = 0
    while i < len(text):
        if text[i].isspace():
            i += 1
        elif text[i] == ";":
            newline = text.find("\n", i)
            i = len(text) if newline < 0 else newline + 1
        elif text[i] in "()":
            out.append(text[i]); i += 1
        elif text[i] == '"':
            i += 1; value: list[str] = []
            while i < len(text) and text[i] != '"':
                if text[i] == "\\":
                    i += 1
                    if i >= len(text):
                        raise ImportFailure(f"{source}: unterminated string escape")
                    value.append({"n": "\n", "r": "\r", "t": "\t"}.get(text[i], text[i]))
                else:
                    value.append(text[i])
                i += 1
            if i >= len(text):
                raise ImportFailure(f"{source}: unterminated string")
            out.append("".join(value)); i += 1
        else:
            end = i
            while end < len(text) and not text[end].isspace() and text[end] not in "();":
                end += 1
            out.append(text[i:end]); i = end
    return out


def parse_sexp(path: pathlib.Path) -> Sexp:
    stack: list[Sexp] = []; roots: list[Sexp] = []
    for token in tokens(path.read_text(), path):
        if token == "(":
            stack.append([])
        elif token == ")":
            if not stack:
                raise ImportFailure(f"{path}: unmatched ')'")
            node = stack.pop()
            (stack[-1] if stack else roots).append(node)
        else:
            if not stack:
                raise ImportFailure(f"{path}: atom outside expression")
            stack[-1].append(token)
    if stack or len(roots) != 1:
        raise ImportFailure(f"{path}: malformed S-expression root")
    return roots[0]


def children(node: Sexp, key: str) -> list[Sexp]:
    return [x for x in node[1:] if isinstance(x, list) and x and x[0] == key]


def child(node: Sexp, key: str, context: str, required: bool = False) -> Sexp | None:
    found = children(node, key)
    if len(found) > 1:
        raise ImportFailure(f"{context}: repeated ({key} ...)")
    if required and not found:
        raise ImportFailure(f"{context}: missing ({key} ...)")
    return found[0] if found else None


def atom(node: Sexp | None, index: int, context: str) -> str:
    if node is None or len(node) <= index or not isinstance(node[index], str):
        raise ImportFailure(f"{context}: missing atom {index}")
    return node[index]


def xy(node: Sexp | None, key: str, context: str) -> tuple[Decimal, Decimal]:
    if node is None or len(node) != 3 or atom(node, 0, context) != key:
        raise ImportFailure(f"{context}: expected ({key} X Y)")
    return decimal(atom(node, 1, context), context), decimal(atom(node, 2, context), context)


def layer(node: Sexp, context: str) -> str | None:
    value = child(node, "layer", context)
    return atom(value, 1, context) if value else None


def normalized_geometry(points: list[tuple[Decimal, Decimal]], context: str) -> dict:
    """Return an exact rect or one-45-degree-corner chamfer."""
    unique = list(dict.fromkeys(points))
    xs = [x for x, _ in unique]; ys = [y for _, y in unique]
    min_x, max_x = min(xs), max(xs); min_y, max_y = min(ys), max(ys)
    corners = {
        "top_left": (min_x, min_y), "top_right": (max_x, min_y),
        "bottom_left": (min_x, max_y), "bottom_right": (max_x, max_y),
    }
    present = set(unique)
    result = {
        "shape": "rect",
        "size": [decimal_text(max_x - min_x), decimal_text(max_y - min_y)],
        "at_offset": [decimal_text((min_x + max_x) / 2), decimal_text((min_y + max_y) / 2)],
    }
    if len(unique) == 4 and set(corners.values()) == present:
        return result
    missing = [name for name, point in corners.items() if point not in present]
    if len(unique) != 5 or len(missing) != 1:
        raise ImportFailure(f"{context}: polygon is not a rectangle or one-corner chamfer")
    name = missing[0]; cx, cy = corners[name]
    along_x = [abs(x - cx) for x, y in unique if y == cy and x != cx]
    along_y = [abs(y - cy) for x, y in unique if x == cx and y != cy]
    if not along_x or not along_y or min(along_x) != min(along_y):
        raise ImportFailure(f"{context}: chamfer is not a 45-degree corner cut")
    result["chamfer"] = {"corner": name, "cut": decimal_text(min(along_x))}
    return result


def kicad_pad(node: Sexp, context: str, unnamed_index: int) -> dict:
    number = atom(node, 1, context)
    if not number:
        number = f"MP{unnamed_index}"
    if not re.fullmatch(r"[A-Za-z0-9_]+", number):
        raise ImportFailure(f"{context}: unsupported pad number {number!r}")
    if atom(node, 2, context) != "smd":
        raise ImportFailure(f"{context}: only SMD module lands are in scope")
    source_shape = atom(node, 3, context)
    at_node = child(node, "at", context, True)
    assert at_node is not None
    if len(at_node) not in {3, 4}:
        raise ImportFailure(f"{context}: malformed pad placement")
    px = decimal(atom(at_node, 1, context), context)
    py = decimal(atom(at_node, 2, context), context)
    rotation = decimal(atom(at_node, 3, context), context) if len(at_node) == 4 else Decimal(0)
    if rotation != rotation.to_integral_value() or not 0 <= rotation <= 359:
        raise ImportFailure(f"{context}: unsupported rotation {rotation}")
    size_node = child(node, "size", context, True)
    assert size_node is not None
    width = decimal(atom(size_node, 1, context), context)
    height = decimal(atom(size_node, 2, context), context)
    layers = child(node, "layers", context, True)
    assert layers is not None
    names = [x for x in layers[1:] if isinstance(x, str)]
    if set(names) != {"F.Cu", "F.Paste", "F.Mask"} or len(names) != 3:
        raise ImportFailure(f"{context}: unsupported module-pad layers {names}")
    stack: dict = {
        "layer": "top_copper", "plating": "smd", "paste": "follow_copper",
    }
    if source_shape == "circle":
        if width != height:
            raise ImportFailure(f"{context}: elliptical circle")
        stack.update({"shape": "circle", "size": [decimal_text(width)]})
    elif source_shape in {"rect", "roundrect"}:
        stack.update({"shape": "rect", "size": [decimal_text(width), decimal_text(height)]})
        if source_shape == "roundrect":
            ratio = decimal(atom(child(node, "roundrect_rratio", context, True), 1, context), context)
            radius = min(width, height) * ratio
            if radius:
                stack["corner_radius"] = decimal_text(radius)
    elif source_shape == "custom":
        primitives = child(node, "primitives", context, True)
        assert primitives is not None
        polys = children(primitives, "gr_poly")
        if len(polys) != 1:
            raise ImportFailure(f"{context}: custom pad needs exactly one polygon")
        pts = child(polys[0], "pts", context, True)
        assert pts is not None
        points = [xy(p, "xy", context) for p in pts[1:] if isinstance(p, list)]
        geometry = normalized_geometry(points, context)
        off_x, off_y = map(Decimal, geometry.pop("at_offset"))
        if off_x or off_y:
            raise ImportFailure(f"{context}: custom pad polygon is not anchor-centred")
        stack.update(geometry)
    else:
        raise ImportFailure(f"{context}: unsupported pad shape {source_shape!r}")
    ratio_node = child(node, "chamfer_ratio", context)
    if ratio_node:
        corners = child(node, "chamfer", context, True)
        assert corners is not None
        values = [x for x in corners[1:] if isinstance(x, str)]
        if len(values) != 1:
            raise ImportFailure(f"{context}: only one chamfered corner is supported")
        ratio = decimal(atom(ratio_node, 1, context), context)
        stack["chamfer"] = {"corner": values[0], "cut": decimal_text(min(width, height) * ratio)}
    return {
        "at": [decimal_text(px), decimal_text(py)], "number": number,
        "rotation": int(rotation), "stack": stack,
        "source_number": atom(node, 1, context),
    }


def courtyard_from_kicad(root: Sexp, context: str, pads: list[dict]) -> dict:
    points: list[tuple[Decimal, Decimal]] = []
    kinds: Counter[str] = Counter()
    for node in root[2:]:
        if not isinstance(node, list) or not node or layer(node, context) != "F.CrtYd":
            continue
        kind = atom(node, 0, context); kinds[kind] += 1
        if kind in {"fp_line", "fp_rect"}:
            points.extend([xy(child(node, "start", context, True), "start", context),
                           xy(child(node, "end", context, True), "end", context)])
        elif kind == "fp_poly":
            pts = child(node, "pts", context, True)
            assert pts is not None
            points.extend(xy(p, "xy", context) for p in pts[1:] if isinstance(p, list))
        else:
            raise ImportFailure(f"{context}: unsupported courtyard primitive {kind}")
    if not points:
        # One pinned upstream module (ESP8685-WROOM-06) omits F.CrtYd.  Retain
        # the land exactly and synthesize a conservative 0.5 mm pad-envelope
        # clearance, recording that projection explicitly.
        for pad in pads:
            px, py = map(Decimal, pad["at"])
            size = list(map(Decimal, pad["stack"]["size"]))
            width, height = (size[0], size[0]) if len(size) == 1 else size
            if pad["rotation"] in {90, 270}:
                width, height = height, width
            points.extend([(px - width / 2, py - height / 2),
                           (px + width / 2, py + height / 2)])
        if not points:
            raise ImportFailure(f"{context}: no courtyard and no pad envelope")
        min_x = min(x for x, _ in points) - Decimal("0.5")
        max_x = max(x for x, _ in points) + Decimal("0.5")
        min_y = min(y for _, y in points) - Decimal("0.5")
        max_y = max(y for _, y in points) + Decimal("0.5")
        return {
            "projection": "synthesized_pad_bounding_rect",
            "at": [decimal_text((min_x + max_x) / 2), decimal_text((min_y + max_y) / 2)],
            "size": [decimal_text(max_x - min_x), decimal_text(max_y - min_y)],
            "source_primitives": {},
        }
    min_x = min(x for x, _ in points); max_x = max(x for x, _ in points)
    min_y = min(y for _, y in points); max_y = max(y for _, y in points)
    exact = kinds == Counter({"fp_rect": 1})
    return {
        "projection": "exact_rect" if exact else "conservative_axis_aligned_bounding_rect",
        "at": [decimal_text((min_x + max_x) / 2), decimal_text((min_y + max_y) / 2)],
        "size": [decimal_text(max_x - min_x), decimal_text(max_y - min_y)],
        "source_primitives": dict(sorted(kinds.items())),
    }


def kicad_keepout_guides(root: Sexp, context: str) -> list[list[list[str]]]:
    guides = []
    for zone in children(root, "zone"):
        keepout = child(zone, "keepout", context)
        if not keepout:
            continue
        polygon = child(zone, "polygon", context, True)
        pts = child(polygon, "pts", context, True) if polygon else None
        assert pts is not None
        points = [[decimal_text(x), decimal_text(y)] for x, y in
                  (xy(p, "xy", context) for p in pts[1:] if isinstance(p, list))]
        if len(points) < 3:
            raise ImportFailure(f"{context}: keepout polygon has fewer than three points")
        guides.append(points)
    return guides


def kicad_reference(root: Sexp, courtyard: dict, context: str) -> list[str]:
    refs = [p for p in children(root, "property") if len(p) >= 3 and p[1] == "Reference"]
    if refs:
        at = child(refs[0], "at", context, True)
        assert at is not None
        return [decimal_text(decimal(atom(at, 1, context), context)),
                decimal_text(decimal(atom(at, 2, context), context))]
    return [courtyard["at"][0], decimal_text(Decimal(courtyard["at"][1]) - Decimal(courtyard["size"][1]) / 2 - 1)]


def import_module(source_root: pathlib.Path, stem: str) -> dict:
    relative = pathlib.Path("footprints/Espressif.pretty") / f"{stem}.kicad_mod"
    path = source_root / relative
    if not path.is_file():
        raise ImportFailure(f"missing Espressif module footprint {path}")
    root = parse_sexp(path); context = f"PCM_Espressif:{stem}"
    if atom(root, 0, context) != "footprint" or atom(root, 1, context) != stem:
        raise ImportFailure(f"{context}: root name differs from filename")
    pads: list[dict] = []; unnamed = 0; seen: set[str] = set(); deduped = 0
    for index, node in enumerate(children(root, "pad"), 1):
        if atom(node, 1, context) == "":
            unnamed += 1
        pad = kicad_pad(node, f"{context}: pad {index}", unnamed)
        key = json.dumps(pad, sort_keys=True)
        if key in seen:
            deduped += 1
            continue
        seen.add(key); pads.append(pad)
    if not pads:
        raise ImportFailure(f"{context}: no pads")
    courtyard = courtyard_from_kicad(root, context, pads)
    return {
        "owner": "@espressif/esp32", "public_name": public_module_name(stem),
        "source": {"kind": "espressif_kicad", "path": relative.as_posix(),
                   "sha256": sha256_file(path), "name": stem},
        "pads": pads, "courtyard": courtyard,
        "reference_at": kicad_reference(root, courtyard, context),
        "keepout_guides": kicad_keepout_guides(root, context),
        "pin_1_marker": any(p["number"] == "1" for p in pads),
        "normalization": {"identical_duplicate_pads_removed": deduped,
                          "unnumbered_source_pads": unnamed},
    }


# --- PADS ASCII reader --------------------------------------------------------

def direct_bytes(root: pathlib.Path, relative: str, member: str | None) -> tuple[bytes, str | None]:
    path = root / relative
    if member is None:
        return path.read_bytes(), None
    with zipfile.ZipFile(path) as archive:
        return archive.read(member), sha256_file(path)


def pads_polygon(points: list[tuple[int, int]], context: str) -> dict:
    converted = [(dbu(x), -dbu(y)) for x, y in points]
    return normalized_geometry(converted, context)


def pad_stack(row: list[str], context: str) -> dict:
    if len(row) < 3 or row[0] != "-2":
        raise ImportFailure(f"{context}: missing top-copper stack line")
    width = dbu(row[1]); shape = row[2]
    stack: dict = {"layer": "top_copper", "plating": "smd", "paste": "follow_copper"}
    if shape == "S":
        stack.update({"shape": "rect", "size": [decimal_text(width), decimal_text(width)]})
        rotation = 0
    elif shape in {"RF", "OF"}:
        if len(row) < 6:
            raise ImportFailure(f"{context}: truncated finger stack")
        rotation_value = decimal(row[3], context)
        if rotation_value != rotation_value.to_integral_value():
            raise ImportFailure(f"{context}: fractional finger rotation")
        rotation = int(rotation_value) % 360
        length = dbu(row[4])
        stack.update({"shape": "oval" if shape == "OF" else "rect",
                      "size": [decimal_text(length), decimal_text(width)]})
        if shape == "RF" and len(row) >= 7 and re.fullmatch(r"-?\d+", row[6]):
            radius = dbu(row[6])
            if radius:
                stack["corner_radius"] = decimal_text(radius)
    else:
        raise ImportFailure(f"{context}: unsupported PADS shape {shape}")
    return {"rotation": rotation, "stack": stack}


def import_direct(data: bytes, spec: tuple, archive_sha: str | None) -> dict:
    public_name, owner, relative, member, expected_sha, url = spec
    if owner == "qfn":
        # RFC-021 reserves a leading QFN family token for its closed IPC-7351
        # grammar.  These are manufacturer-source decal identities rather than
        # generic IPC names, so make that provenance explicit.
        public_name = f"ESPRESSIF_{public_name}"
    actual_sha = sha256_bytes(data)
    if actual_sha != expected_sha:
        raise ImportFailure(f"{relative}{'!' + member if member else ''}: sha256 {actual_sha}, expected {expected_sha}")
    lines = data.decode("utf-8", errors="strict").splitlines()
    start_marker = next((i for i, line in enumerate(lines) if line.startswith("*PARTDECAL*")), None)
    if start_marker is None:
        raise ImportFailure(f"{relative}: no PARTDECAL section")
    header_index = next(i for i in range(start_marker + 1, len(lines))
                        if lines[i] and not lines[i].startswith("*"))
    header = lines[header_index].split()
    if len(header) != 9:
        raise ImportFailure(f"{relative}: malformed PARTDECAL header")
    decal_name = header[0]; terminal_count = int(header[5])
    end = next(i for i in range(header_index + 1, len(lines)) if lines[i].startswith("*PARTTYPE*"))
    has_linestyle = any("LINESTYLE" in line for line in lines[start_marker:header_index])
    terminals: list[tuple[int, int, str]] = []
    stacks: dict[int, dict] = {}; copper: list[tuple[int, dict]] = []
    paste: list[dict] = []; mask_top_polygons = 0; mask_bottom_polygons = 0
    vias: list[tuple[Decimal, Decimal, Decimal]] = []
    keepout_points: list[tuple[Decimal, Decimal]] = []
    i = header_index + 1
    while i < end:
        fields = lines[i].split()
        if not fields:
            i += 1; continue
        kind = fields[0]
        if kind == "COPCLS":
            count = int(fields[1]); raw = []
            for _ in range(count):
                i += 1; pair = lines[i].split(); raw.append((int(pair[0]), int(pair[1])))
            if has_linestyle:
                level = int(fields[4]); pin = int(fields[5]) if len(fields) > 5 else None
            else:
                level = int(fields[3]); pin = int(fields[4]) if len(fields) > 4 else None
            geometry = pads_polygon(raw, f"{relative}: COPCLS")
            if level == 1:
                if pin is None:
                    raise ImportFailure(f"{relative}: unnumbered copper polygon")
                copper.append((pin + 1, geometry))
            elif level == 123:
                paste.append(geometry)
            elif level == 121:
                mask_top_polygons += 1
            elif level == 128:
                mask_bottom_polygons += 1
        elif kind in {"OPEN", "CLOSED", "KPTCLS"}:
            count = int(fields[1]); raw = []
            for _ in range(count):
                i += 1; pair = lines[i].split(); raw.append((int(pair[0]), int(pair[1])))
            if kind == "KPTCLS":
                keepout_points.extend((dbu(x), -dbu(y)) for x, y in raw)
        elif kind == "CIRCLE":
            count = int(fields[1]); level_index = 4 if has_linestyle else 3
            level = int(fields[level_index]); raw = []
            for _ in range(count):
                i += 1; pair = lines[i].split(); raw.append((int(pair[0]), int(pair[1])))
            if level == 127 and len(raw) == 2:
                dx = abs(dbu(raw[0][0] - raw[1][0]))
                dy = abs(dbu(raw[0][1] - raw[1][1]))
                if bool(dx) == bool(dy):
                    raise ImportFailure(f"{relative}: assembly via circle is not axis-aligned")
                diameter = dx or dy
                if diameter not in {Decimal("0.4"), Decimal("0.5")}:
                    raise ImportFailure(f"{relative}: unknown thermal-via land diameter {diameter}mm")
                vias.append((dbu(raw[0][0] + raw[1][0]) / 2,
                             -dbu(raw[0][1] + raw[1][1]) / 2, diameter))
        elif kind.startswith("T") and re.fullmatch(r"T-?\d+", kind):
            values = (kind[1:] + " " + " ".join(fields[1:])).split()
            terminals.append((int(values[0]), int(values[1]), values[-1]))
        elif kind == "PAD":
            pin = int(fields[1]); count = int(fields[2]); rows = []
            for _ in range(count):
                i += 1; rows.append(lines[i].split())
            top = next((row for row in rows if row and row[0] == "-2"), None)
            if top is None:
                raise ImportFailure(f"{relative}: PAD {pin} lacks top copper")
            stacks[pin] = pad_stack(top, f"{relative}: PAD {pin}")
        i += 1
    if len(terminals) != terminal_count:
        raise ImportFailure(f"{relative}: expected {terminal_count} terminals, found {len(terminals)}")
    if 0 not in stacks:
        raise ImportFailure(f"{relative}: no default PAD 0 stack")
    ep_number = terminals[-1][2]
    pads: list[dict] = []
    for x, y, number in terminals:
        selected = stacks.get(int(number), stacks[0])
        stack = json.loads(json.dumps(selected["stack"]))
        if number == ep_number and paste:
            stack["paste"] = "none"
        pads.append({"number": number, "at": [decimal_text(dbu(x)), decimal_text(-dbu(y))],
                     "rotation": selected["rotation"], "stack": stack})
    for number, geometry in copper:
        stack = {"layer": "top_copper", "plating": "smd", "paste": "none" if paste else "follow_copper",
                 "shape": geometry["shape"], "size": geometry["size"]}
        if "chamfer" in geometry:
            stack["chamfer"] = geometry["chamfer"]
        pads.append({"number": str(number), "at": geometry["at_offset"], "rotation": 0, "stack": stack})
    containment_proofs = 0
    for geometry in paste:
        paste_x, paste_y = map(Decimal, geometry["at_offset"])
        paste_w, paste_h = map(Decimal, geometry["size"])
        paste_box = (paste_x - paste_w / 2, paste_x + paste_w / 2,
                     paste_y - paste_h / 2, paste_y + paste_h / 2)
        containers = []
        for candidate in pads:
            if (candidate["number"] != ep_number
                    or candidate["stack"]["plating"] != "smd"):
                continue
            copper_x, copper_y = map(Decimal, candidate["at"])
            copper_size = list(map(Decimal, candidate["stack"]["size"]))
            copper_w, copper_h = ((copper_size[0], copper_size[0])
                                  if len(copper_size) == 1 else copper_size)
            if candidate["rotation"] in {90, 270}:
                copper_w, copper_h = copper_h, copper_w
            copper_box = (copper_x - copper_w / 2, copper_x + copper_w / 2,
                          copper_y - copper_h / 2, copper_y + copper_h / 2)
            if (copper_box[0] <= paste_box[0] and copper_box[1] >= paste_box[1]
                    and copper_box[2] <= paste_box[2] and copper_box[3] >= paste_box[3]):
                # Equal envelopes that carry a chamfer must carry the same cut;
                # otherwise the nominal paste rectangle would enter missing
                # copper at the corner despite passing a bounding-box check.
                same_box = copper_box == paste_box
                if (not same_box
                        or candidate["stack"].get("chamfer") == geometry.get("chamfer")):
                    containers.append(candidate)
        if not containers:
            raise ImportFailure(
                f"{relative}: paste aperture at {geometry['at_offset']} is not "
                f"contained by one exposed-pad copper placement"
            )
        containment_proofs += 1
        stack = {"layer": "top_copper", "plating": "smd", "paste": "follow_copper",
                 "shape": geometry["shape"], "size": geometry["size"]}
        if "chamfer" in geometry:
            stack["chamfer"] = geometry["chamfer"]
        pads.append({"number": ep_number, "at": geometry["at_offset"], "rotation": 0, "stack": stack,
                     "projection_role": "contained_paste_overlay"})
    for x, y, diameter in vias:
        pads.append({"number": ep_number, "at": [decimal_text(x), decimal_text(y)], "rotation": 0,
                     "stack": {"layer": "through_all", "plating": "plated_through_hole",
                               "shape": "circle", "size": [decimal_text(diameter)], "drill": ["0.25"]},
                     "projection_role": "thermal_via"})
    # Exact duplicate source placements are no-ops.  Stack and role participate
    # in the key, so copper/paste overlays at the same location survive.
    unique = []; seen = set(); duplicates = 0
    for pad in pads:
        key = json.dumps(pad, sort_keys=True)
        if key in seen:
            duplicates += 1; continue
        seen.add(key); unique.append(pad)
    pads = unique
    bounds = keepout_points[:]
    if not bounds:
        for pad in pads:
            x, y = map(Decimal, pad["at"]); size = list(map(Decimal, pad["stack"]["size"]))
            w = size[0]; h = size[0] if len(size) == 1 else size[1]
            if pad["rotation"] in {90, 270}: w, h = h, w
            bounds.extend([(x - w / 2, y - h / 2), (x + w / 2, y + h / 2)])
    min_x = min(x for x, _ in bounds); max_x = max(x for x, _ in bounds)
    min_y = min(y for _, y in bounds); max_y = max(y for _, y in bounds)
    if not keepout_points:
        min_x -= Decimal("0.1"); max_x += Decimal("0.1")
        min_y -= Decimal("0.1"); max_y += Decimal("0.1")
    courtyard = {"projection": "conservative_axis_aligned_bounding_rect",
                  "at": [decimal_text((min_x + max_x) / 2), decimal_text((min_y + max_y) / 2)],
                  "size": [decimal_text(max_x - min_x), decimal_text(max_y - min_y)]}
    source = {"kind": "espressif_direct_cad", "path": relative, "member": member,
              "sha256": actual_sha, "url": url}
    if archive_sha:
        source["archive_sha256"] = archive_sha
    return {
        "owner": owner, "public_name": public_name, "source": source,
        "pads": pads, "courtyard": courtyard,
        "reference_at": [courtyard["at"][0], decimal_text(min_y - Decimal("1"))],
        "keepout_guides": [], "pin_1_marker": True,
        "normalization": {"decal_name": decal_name,
                          "identical_duplicate_pads_removed": duplicates,
                          "source_mask_polygons_not_projected": {
                              "top": mask_top_polygons,
                              "bottom": mask_bottom_polygons,
                          },
                          "thermal_vias": len(vias), "paste_overlays": len(paste),
                          "paste_containment_proofs": containment_proofs},
    }


def import_snapshot(esp_root: pathlib.Path, kicad_root: pathlib.Path, cad_root: pathlib.Path) -> dict:
    require_clean_checkout(esp_root, ESP_COMMIT); require_clean_checkout(kicad_root, KICAD_COMMIT)
    if sha256_file(esp_root / "LICENSE.md") != ESP_LICENSE_SHA256:
        raise ImportFailure("Espressif KiCad license differs from pinned notice")
    if sha256_file(kicad_root / "LICENSE.md") != KICAD_LICENSE_SHA256:
        raise ImportFailure("KiCad license differs from pinned notice")
    footprints = [import_module(esp_root, stem) for stem in MODULE_STEMS]
    archive_hashes: dict[str, str] = {}
    for spec in DIRECT_SPECS:
        data, archive_sha = direct_bytes(cad_root, spec[2], spec[3])
        if archive_sha:
            archive_hashes[spec[2]] = archive_sha
        footprints.append(import_direct(data, spec, archive_sha))
    by_name = {row["public_name"]: row for row in footprints}
    for target_name, raw_name, owner, relative, member, expected_sha, url in DIRECT_CORROBORATION:
        spec = (raw_name, owner, relative, member, expected_sha, url)
        data, archive_sha = direct_bytes(cad_root, relative, member)
        corroborating = import_direct(data, spec, archive_sha)
        target = by_name[target_name]
        for field in ("pads", "courtyard"):
            if corroborating[field] != target[field]:
                raise ImportFailure(
                    f"{relative}: normalized {field} differs from reusable {target_name}"
                )
        target.setdefault("alternate_sources", []).append(corroborating["source"])
    footprints.sort(key=lambda row: (row["owner"], row["public_name"]))
    public = [row["public_name"] for row in footprints]
    duplicates = [name for name, count in Counter(public).items() if count > 1]
    if duplicates:
        raise ImportFailure(f"public footprint-name collisions: {duplicates}")
    secondary = []
    for stem in GENERIC_SECONDARY:
        relative = pathlib.Path("Package_DFN_QFN.pretty") / f"{stem}.kicad_mod"
        path = kicad_root / relative
        if not path.is_file():
            raise ImportFailure(f"missing generic KiCad cross-check {path}")
        secondary.append({"path": relative.as_posix(), "sha256": sha256_file(path)})
    return {
        "schema_version": 1,
        "coverage": {
            "footprints": len(footprints),
            "footprints_by_owner": dict(sorted(Counter(r["owner"] for r in footprints).items())),
            "placements": sum(len(r["pads"]) for r in footprints),
            "module_footprints": len(MODULE_STEMS),
            "direct_cad_footprints": len(DIRECT_SPECS),
            "direct_cad_evidence_files": len(DIRECT_SPECS) + len(DIRECT_CORROBORATION),
        },
        "footprints": footprints,
        "projection_contract": {
            "copper": "Shape, size, position, rotation, repeated number, chamfer, and plated thermal-via drill are exact normalized source facts.",
            "coordinates": "PADS integer coordinates use 1/1500000 mm units and are deterministically rounded half-up to CoHDL's 1e-15 mm literal grid (maximum projection error 0.5 fm).",
            "paste": "Native module paste follows copper. Direct PADS level 123 (Paste Mask Top) polygons would be containment-proved same-number overlays; the pinned current sources contain none, so paste follows the exact copper islands or continuous EP.",
            "mask": "Native pad mask is retained. PADS levels 121/128 are explicitly Solder Mask Top/Bottom; their arbitrary polygons are counted separately but not projected because CoHDL supports expansion, not independent mask polygons.",
            "courtyard": "Exact rectangles remain exact; segmented, polygonal, or PADS component keepouts become a conservative axis-aligned bounding rectangle.",
            "silkscreen": "Pin 1 is regenerated as the semantic marker. Antenna keepout polygons are disclosed as exact silkscreen outlines; source body/fabrication graphics are omitted.",
            "unnumbered": "KiCad unnumbered copper lands receive stable MP1..MPn physical identifiers so CoHDL can retain them and device declarations can model them explicitly.",
        },
        "sources": {
            "espressif_kicad": {"repository": ESP_REPOSITORY, "commit": ESP_COMMIT,
                                "license": "CC-BY-SA-4.0 with KiCad library exception",
                                "license_sha256": ESP_LICENSE_SHA256},
            "kicad_footprints_secondary": {"repository": KICAD_REPOSITORY, "commit": KICAD_COMMIT,
                                           "license": "CC-BY-SA-4.0 with KiCad library exception",
                                           "license_sha256": KICAD_LICENSE_SHA256,
                                           "files": secondary,
                                           "role": "symbol-reference cross-check only; direct Espressif CAD wins on mismatch"},
            "espressif_direct_cad": {"source_page": DIRECT_SOURCE_PAGE,
                                     "redistribution": "normalized facts only; raw CAD is not bundled",
                                     "coordinate_unit": "1/1500000mm",
                                     "coordinate_projection": "ROUND_HALF_UP to 0.000000000000001mm",
                                     "archives": archive_hashes},
        },
    }


def snapshot_bytes(snapshot: dict) -> bytes:
    return (json.dumps(snapshot, indent=2, sort_keys=True) + "\n").encode()


def require_snapshot_hash(data: bytes) -> None:
    actual = sha256_bytes(data)
    if SNAPSHOT_SHA256 != "TO_BE_PINNED" and actual != SNAPSHOT_SHA256:
        raise ImportFailure(f"{SNAPSHOT}: sha256 {actual}, expected {SNAPSHOT_SHA256}")


def validate_snapshot(snapshot: dict) -> None:
    if snapshot.get("schema_version") != 1:
        raise ImportFailure(f"{SNAPSHOT}: unsupported schema")
    coverage = snapshot.get("coverage", {})
    expected = len(MODULE_STEMS) + len(DIRECT_SPECS)
    if coverage.get("footprints") != expected or len(snapshot.get("footprints", [])) != expected:
        raise ImportFailure(f"{SNAPSHOT}: expected {expected} generated footprints")
    if snapshot.get("sources", {}).get("espressif_kicad", {}).get("commit") != ESP_COMMIT:
        raise ImportFailure(f"{SNAPSHOT}: stale Espressif KiCad commit")
    names = [r["public_name"] for r in snapshot["footprints"]]
    if len(names) != len(set(names)):
        raise ImportFailure(f"{SNAPSHOT}: duplicate public names")


def geometry_key(stack: dict) -> tuple:
    return (
        stack["layer"], stack["plating"], stack["shape"], tuple(stack["size"]),
        tuple(stack.get("drill", [])), stack.get("corner_radius", ""),
        stack.get("mask_expansion", ""), stack.get("paste", "follow_copper"),
        stack.get("chamfer", {}).get("corner", ""), stack.get("chamfer", {}).get("cut", ""),
    )


def generated_source(owner: str, rows: list[dict]) -> str:
    geometries = sorted({geometry_key(p["stack"]) for row in rows for p in row["pads"]})
    prefix = "P_ESP_QFN" if owner == "qfn" else "P_ESPRESSIF"
    pad_names = {geometry: f"{prefix}_{index:04d}" for index, geometry in enumerate(geometries, 1)}
    source_lines = (
        [f"// Secondary generic cross-check: {KICAD_REPOSITORY} @ {KICAD_COMMIT}"]
        if owner == "qfn"
        else [f"// Espressif KiCad source: {ESP_REPOSITORY} @ {ESP_COMMIT}"]
    )
    out = [
        "// GENERATED by tools/gen_esp32_footprints.py; do not edit.",
        *source_lines,
        f"// Direct dimensional evidence: {DIRECT_SOURCE_PAGE}",
        "// See the package docs and LICENSE.kicad.md for provenance and projection details.", "",
    ]
    for geometry in geometries:
        layer_name, plating, shape, size, drill, radius, mask, paste, corner, cut = geometry
        out.append(f"pad {pad_names[geometry]} {{")
        out.append(f"    shape: {shape}")
        out.append(f"    size: ({', '.join(mm(v) for v in size)})")
        out.append(f"    layer: {layer_name}"); out.append(f"    plating: {plating}")
        if drill:
            out.append(f"    drill: ({', '.join(mm(v) for v in drill)})" if len(drill) > 1 else f"    drill: {mm(drill[0])}")
        if radius:
            out.append(f"    corner_radius: {mm(radius)}")
        if mask:
            out.append(f"    mask_expansion: {mm(mask)}")
        if corner:
            out.append(f"    chamfer: ({corner}, {mm(cut)})")
        if paste == "none":
            out.append("    paste: none")
        elif paste != "follow_copper":
            raise ImportFailure(f"unsupported generated paste mode {paste}")
        out.extend(["}", ""])
    for row in rows:
        source = row["source"]
        label = source.get("member") or source["path"]
        out.extend([f"// Source: {label}", f"// Source file SHA-256: {source['sha256']}"])
        if row["courtyard"]["projection"] != "exact_rect":
            out.append("// Courtyard is the conservative source bounding rectangle.")
        if row["keepout_guides"]:
            out.append("// Silk outlines below disclose unenforced antenna keepout zones.")
        out.append(f"pub footprint {row['public_name']} {{")
        for pad in row["pads"]:
            x, y = pad["at"]; rotation = f" rotate {pad['rotation']}" if pad["rotation"] else ""
            out.append(f"    pad {pad['number']}: {pad_names[geometry_key(pad['stack'])]} at ({mm(x)}, {mm(y)}){rotation}")
        if row["pin_1_marker"] or row["keepout_guides"]:
            out.append("    silkscreen {")
            if row["pin_1_marker"]:
                out.append("        pin_1_marker near pad 1 shape dot")
            for polygon in row["keepout_guides"]:
                for index, start in enumerate(polygon):
                    end = polygon[(index + 1) % len(polygon)]
                    out.append(f"        line from ({mm(start[0])}, {mm(start[1])}) to ({mm(end[0])}, {mm(end[1])}) width 0.12mm")
            out.append("    }")
        court = row["courtyard"]
        out.append(f"    courtyard {{ shape: rect, at: ({mm(court['at'][0])}, {mm(court['at'][1])}), size: ({mm(court['size'][0])}, {mm(court['size'][1])}) }}")
        ref = row["reference_at"]
        out.extend([f"    silkscreen_ref {{ at: ({mm(ref[0])}, {mm(ref[1])}) }}", "}", ""])
    return "\n".join(out)


def docs_text(owner: str, snapshot: dict) -> str:
    count = sum(1 for row in snapshot["footprints"] if row["owner"] == owner)
    if owner == "qfn":
        license_paragraph = """KiCad's generic footprint library is pinned only as a secondary symbol-reference
cross-check and is CC-BY-SA-4.0 with the KiCad library exception; its exact
notice is shipped as `LICENSE.kicad.md`.  Generated QFN land geometry comes
from the direct Espressif PADS evidence described below."""
    else:
        license_paragraph = """Espressif's pinned KiCad 3.2.1 library is CC-BY-SA-4.0 with the KiCad library
exception; its notice text is shipped as `LICENSE.kicad.md`."""
    return f"""# Generated ESP32 footprint geometry

This package contains {count} generated ESP32-family land patterns frozen by
`tools/gen_esp32_footprints.py`.  Ordinary generation is offline.  The
normalized snapshot records every pad stack and placement, source URL/path and
SHA-256, ownership, and the projection contract.

{license_paragraph}  Direct website PADS files are used only as dimensional
evidence: raw files are not bundled.

CoHDL cannot emit footprint keepout zones.  Exact source antenna keepout
polygons are therefore visible silkscreen guides only and remain unenforced;
apply the module datasheet's RF clearance in board layout.  Non-rectangular
courtyards are conservative bounding rectangles.  Pin 1 uses CoHDL's semantic
marker, and non-land body/fabrication graphics are omitted.

Direct PADS exposed-pad copper and repeated-number VIA16_10/VIA20_10 thermal
vias are retained.  The pinned files contain no level-123 Paste Mask Top
polygons, so paste follows the exact copper islands (or the continuous EP).
Levels 121/128 are top/bottom solder-mask evidence, not stencil windows; their
independent arbitrary polygons cannot be represented and are counted
separately in the snapshot rather than projected as invented paste.

PADS integer coordinates use 1/1,500,000 mm database units.  Recurring values
are rounded half-up once to CoHDL's 10^-15 mm literal grid, so the maximum
coordinate projection is 0.5 femtometres.
"""


def generated_files(snapshot: dict) -> dict[pathlib.Path, bytes]:
    grouped = {"qfn": [], "@espressif/esp32": []}
    for row in snapshot["footprints"]:
        grouped[row["owner"]].append(row)
    files = {
        ROOT / "lib/qfn/src/esp32_kicad_generated.cohdl": generated_source("qfn", grouped["qfn"]).encode(),
        ROOT / "lib/@espressif/esp32/src/kicad_generated_footprints.cohdl": generated_source("@espressif/esp32", grouped["@espressif/esp32"]).encode(),
        ROOT / "lib/qfn/docs/esp32-generated-footprints.md": docs_text("qfn", snapshot).encode(),
        ROOT / "lib/@espressif/esp32/docs/generated-footprints.md": docs_text("@espressif/esp32", snapshot).encode(),
    }
    esp_license = ESP_LICENSE.read_bytes(); kicad_license = KICAD_LICENSE.read_bytes()
    if sha256_bytes(esp_license) != ESP_NOTICE_SHA256 or sha256_bytes(kicad_license) != KICAD_LICENSE_SHA256:
        raise ImportFailure("checked-in license notice checksum differs from pinned source")
    files[ROOT / "lib/@espressif/esp32/LICENSE.kicad.md"] = esp_license
    files[ROOT / "lib/qfn/LICENSE.kicad.md"] = kicad_license
    return files


def write_or_check(path: pathlib.Path, data: bytes, check: bool) -> bool:
    if check:
        actual = path.read_bytes() if path.is_file() else None
        if actual != data:
            print(f"out of date: {path.relative_to(ROOT)}", file=sys.stderr); return False
        return True
    path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(data)
    print(f"wrote {path.relative_to(ROOT)} ({len(data)} bytes, sha256 {sha256_bytes(data)})")
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--import-sources", nargs=2, type=pathlib.Path,
                        metavar=("ESPRESSIF_KICAD", "KICAD_FOOTPRINTS"))
    parser.add_argument("--direct-cad-root", type=pathlib.Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    try:
        if bool(args.import_sources) != bool(args.direct_cad_root):
            raise ImportFailure("--import-sources and --direct-cad-root must be used together")
        if args.import_sources:
            snapshot = import_snapshot(args.import_sources[0].resolve(), args.import_sources[1].resolve(),
                                       args.direct_cad_root.resolve())
            encoded = snapshot_bytes(snapshot); require_snapshot_hash(encoded)
            ok = write_or_check(SNAPSHOT, encoded, args.check)
        else:
            encoded = SNAPSHOT.read_bytes(); require_snapshot_hash(encoded)
            snapshot = json.loads(encoded); ok = True
        validate_snapshot(snapshot)
        for path, data in generated_files(snapshot).items():
            ok = write_or_check(path, data, args.check) and ok
        if not ok:
            return 1
        coverage = snapshot["coverage"]
        print(f"verified {coverage['footprints']} ESP32 footprints and {coverage['placements']} exact pad placements")
        return 0
    except (ImportFailure, OSError, UnicodeError, zipfile.BadZipFile, json.JSONDecodeError, KeyError, TypeError) as exc:
        print(f"error: {exc}", file=sys.stderr); return 1


if __name__ == "__main__":
    raise SystemExit(main())
