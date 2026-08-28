#!/usr/bin/env python3
"""Import and generate the focused footprints used by the STM32 catalog.

The maintainer-only import mode reads an exact checkout of KiCad's official
footprint library and freezes a small, normalized JSON snapshot.  Ordinary
generation is offline and reads only checked-in inputs:

    python3 tools/gen_stm32_footprints.py

To refresh the snapshot from the pinned upstream commit:

    python3 tools/gen_stm32_footprints.py \
        --import-source /path/to/kicad-footprints

KiCad library data is CC-BY-SA-4.0 with the KiCad library exception.  See the
LICENSE.kicad.md files shipped with the generated packages.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import re
import subprocess
import sys
from collections import Counter
from decimal import Decimal, InvalidOperation, ROUND_HALF_UP


ROOT = pathlib.Path(__file__).resolve().parent.parent
MAPPINGS = ROOT / "tools" / "stm32_data" / "kicad_parts.json"
SNAPSHOT = ROOT / "tools" / "stm32_footprint_data" / "footprints.json"
LICENSE_NOTICE = ROOT / "tools" / "stm32_footprint_data" / "LICENSE.kicad.md"

KICAD_REPOSITORY = "https://gitlab.com/kicad/libraries/kicad-footprints.git"
KICAD_COMMIT = "819223b66f96508feaeaa305301b5e6bb5c1038b"
KICAD_LICENSE_SHA256 = (
    "45d2bce75e5a4208f5afb01b8fb2c406e700371c4fe2b5f5cd5c443d46db4d8f"
)
SNAPSHOT_SHA256 = "173fe24d5e881ec4bfb4d5e9b50ee490ee577ac0509b619954714d74435109e8"
KICAD_FORMAT_VERSION = "20260206"

LIBRARY_TO_PACKAGE = {
    "Package_BGA": "bga",
    "Package_CSP": "csp",
    "Package_QFP": "qfp",
    "Package_SO": "soic",
}
EXPECTED_COUNTS = {
    "Package_BGA": 20,
    "Package_CSP": 70,
    "Package_QFP": 10,
    "Package_SO": 3,
}
EXPECTED_FOOTPRINTS = 103
EXPECTED_PADS = 9147
IU_MM = Decimal("0.000001")  # KiCad PCB internal unit: one nanometre.


class ImportFailure(ValueError):
    """The pinned source contains geometry this importer cannot preserve."""


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


def decimal(value: str, context: str) -> Decimal:
    try:
        number = Decimal(value)
    except InvalidOperation as exc:
        raise ImportFailure(f"{context}: invalid decimal {value!r}") from exc
    if not number.is_finite():
        raise ImportFailure(f"{context}: non-finite decimal {value!r}")
    return number


def decimal_text(value: Decimal) -> str:
    text = format(value, "f")
    if "." in text:
        text = text.rstrip("0").rstrip(".")
    return "0" if text in {"", "-0"} else text


def mm(value: str | Decimal) -> str:
    number = value if isinstance(value, Decimal) else decimal(value, "millimetres")
    return f"{decimal_text(number)}mm"


def tokens(text: str, source: pathlib.Path) -> list[str]:
    """Tokenize the strict S-expression subset used by .kicad_mod files."""
    out: list[str] = []
    i = 0
    while i < len(text):
        char = text[i]
        if char.isspace():
            i += 1
            continue
        if char == ";":
            newline = text.find("\n", i)
            i = len(text) if newline < 0 else newline + 1
            continue
        if char in "()":
            out.append(char)
            i += 1
            continue
        if char == '"':
            i += 1
            value: list[str] = []
            while i < len(text) and text[i] != '"':
                if text[i] == "\\":
                    i += 1
                    if i >= len(text):
                        raise ImportFailure(f"{source}: unterminated string escape")
                    escaped = text[i]
                    value.append({"n": "\n", "r": "\r", "t": "\t"}.get(escaped, escaped))
                else:
                    value.append(text[i])
                i += 1
            if i >= len(text):
                raise ImportFailure(f"{source}: unterminated string")
            out.append("".join(value))
            i += 1
            continue
        end = i
        while end < len(text) and not text[end].isspace() and text[end] not in "();":
            end += 1
        if end == i:
            raise ImportFailure(f"{source}: cannot tokenize byte {i}")
        out.append(text[i:end])
        i = end
    return out


Sexp = list["str | Sexp"]


def parse_sexp(path: pathlib.Path) -> Sexp:
    stack: list[Sexp] = []
    roots: list[Sexp] = []
    for token in tokens(path.read_text(), path):
        if token == "(":
            stack.append([])
        elif token == ")":
            if not stack:
                raise ImportFailure(f"{path}: unmatched ')'")
            node = stack.pop()
            if stack:
                stack[-1].append(node)
            else:
                roots.append(node)
        else:
            if not stack:
                raise ImportFailure(f"{path}: atom outside an S-expression")
            stack[-1].append(token)
    if stack:
        raise ImportFailure(f"{path}: unclosed '('")
    if len(roots) != 1:
        raise ImportFailure(f"{path}: expected one root expression, found {len(roots)}")
    return roots[0]


def children(node: Sexp, key: str) -> list[Sexp]:
    return [
        child
        for child in node[1:]
        if isinstance(child, list) and child and child[0] == key
    ]


def child(node: Sexp, key: str, context: str, *, required: bool = False) -> Sexp | None:
    found = children(node, key)
    if len(found) > 1:
        raise ImportFailure(f"{context}: more than one ({key} ...) field")
    if required and not found:
        raise ImportFailure(f"{context}: missing ({key} ...) field")
    return found[0] if found else None


def atom(node: Sexp | None, index: int, context: str) -> str:
    if node is None or len(node) <= index or not isinstance(node[index], str):
        raise ImportFailure(f"{context}: missing atom {index}")
    return node[index]


def layer_of(node: Sexp, context: str) -> str | None:
    layer = child(node, "layer", context)
    return atom(layer, 1, context) if layer else None


def point(node: Sexp | None, key: str, context: str) -> tuple[Decimal, Decimal]:
    if node is None or len(node) != 3 or atom(node, 0, context) != key:
        raise ImportFailure(f"{context}: expected ({key} X Y)")
    return decimal(atom(node, 1, context), context), decimal(atom(node, 2, context), context)


def stroke(node: Sexp, context: str) -> str:
    value = child(node, "stroke", context, required=True)
    assert value is not None
    width = child(value, "width", context, required=True)
    kind = child(value, "type", context, required=True)
    if atom(kind, 1, context) != "solid":
        raise ImportFailure(f"{context}: only solid strokes are representable")
    return decimal_text(decimal(atom(width, 1, context), context))


def qualified_names() -> list[str]:
    data = json.loads(MAPPINGS.read_text())
    source = data.get("sources", {}).get("kicad_footprints", {})
    if source.get("commit") != KICAD_COMMIT:
        raise ImportFailure(
            f"{MAPPINGS}: footprint commit {source.get('commit')!r}, expected {KICAD_COMMIT}"
        )
    names = sorted(
        {
            row["kicad_footprint"]
            for row in data.get("mappings", [])
            if row["kicad_footprint"].split(":", 1)[0] in LIBRARY_TO_PACKAGE
        }
    )
    counts = Counter(name.split(":", 1)[0] for name in names)
    if len(names) != EXPECTED_FOOTPRINTS or dict(counts) != EXPECTED_COUNTS:
        raise ImportFailure(
            f"{MAPPINGS}: expected {EXPECTED_FOOTPRINTS} focused footprints "
            f"{EXPECTED_COUNTS}, found {len(names)} {dict(counts)}"
        )
    return names


def public_name(qualified: str) -> str:
    """Stable `Library:Stem` -> public CoHDL footprint identifier."""
    stem = qualified.split(":", 1)[1]
    normalized = re.sub(r"[^A-Za-z0-9]+", "_", stem).strip("_").upper()
    if not normalized:
        raise ImportFailure(f"cannot form a CoHDL name from {qualified!r}")
    return f"KICAD_{normalized}"


def coordinate_pair(node: Sexp, key: str, context: str) -> list[str]:
    x, y = point(child(node, key, context, required=True), key, context)
    return [decimal_text(x), decimal_text(y)]


def effective_paste_diameter(
    copper: Decimal,
    paste_margin: Decimal | None,
    paste_ratio: Decimal | None,
) -> Decimal | None:
    if paste_margin is None and paste_ratio is None:
        return None
    if paste_margin is not None and paste_ratio is not None:
        raise ImportFailure("a footprint cannot set both paste margin and paste ratio")
    # KiCad evaluates a ratio into its integer nanometre PCB coordinate grid.
    # `KiROUND` rounds a positive half IU away from zero (ROUND_HALF_UP).
    margin = paste_margin
    if paste_ratio is not None:
        margin = (copper * paste_ratio).quantize(IU_MM, rounding=ROUND_HALF_UP)
    assert margin is not None
    result = copper + Decimal(2) * margin
    if result <= 0:
        raise ImportFailure(f"paste aperture is not positive: {decimal_text(result)}mm")
    return result


def import_pad(
    node: Sexp,
    context: str,
    mask_margin: Decimal | None,
    paste_margin: Decimal | None,
    paste_ratio: Decimal | None,
) -> dict:
    if len(node) < 4:
        raise ImportFailure(f"{context}: truncated pad")
    number = atom(node, 1, context)
    kind = atom(node, 2, context)
    shape = atom(node, 3, context)
    if not number or not re.fullmatch(r"[A-Za-z0-9_]+", number):
        raise ImportFailure(f"{context}: unsupported pad number {number!r}")
    if kind != "smd":
        raise ImportFailure(f"{context}: unsupported pad kind {kind!r}")
    if shape not in {"circle", "roundrect"}:
        raise ImportFailure(f"{context}: unsupported pad shape {shape!r}")

    allowed_fields = {"at", "size", "layers", "property", "roundrect_rratio", "uuid"}
    for field in node[4:]:
        if not isinstance(field, list) or not field:
            raise ImportFailure(f"{context}: unexpected pad atom {field!r}")
        if atom(field, 0, context) not in allowed_fields:
            raise ImportFailure(f"{context}: unsupported pad field {field[0]!r}")
    for prop in children(node, "property"):
        if prop[1:] != ["pad_prop_bga"]:
            raise ImportFailure(f"{context}: unsupported pad property {prop[1:]!r}")

    at = child(node, "at", context, required=True)
    assert at is not None
    if len(at) not in {3, 4}:
        raise ImportFailure(f"{context}: expected (at X Y [ROTATION])")
    x = decimal(atom(at, 1, context), context)
    y = decimal(atom(at, 2, context), context)
    rotation = decimal(atom(at, 3, context), context) if len(at) == 4 else Decimal(0)
    if rotation != rotation.to_integral_value() or not 0 <= rotation <= 359:
        raise ImportFailure(f"{context}: unsupported rotation {rotation}")

    size_node = child(node, "size", context, required=True)
    assert size_node is not None
    if len(size_node) != 3:
        raise ImportFailure(f"{context}: expected (size W H)")
    width = decimal(atom(size_node, 1, context), context)
    height = decimal(atom(size_node, 2, context), context)
    if width <= 0 or height <= 0:
        raise ImportFailure(f"{context}: non-positive pad size")

    layers = child(node, "layers", context, required=True)
    assert layers is not None
    layer_names = [value for value in layers[1:] if isinstance(value, str)]
    if (
        len(layer_names) != 3
        or len(layer_names) != len(layers) - 1
        or set(layer_names) != {"F.Cu", "F.Mask", "F.Paste"}
    ):
        raise ImportFailure(f"{context}: unsupported layers {layers[1:]!r}")

    result = {
        "at": [decimal_text(x), decimal_text(y)],
        "copper": {
            "shape": "circle" if shape == "circle" else "roundrect",
            "size": [decimal_text(width), decimal_text(height)],
        },
        "number": number,
        "rotation": int(rotation),
    }
    if shape == "circle":
        if width != height:
            raise ImportFailure(f"{context}: an elliptical KiCad circle is not representable")
        if child(node, "roundrect_rratio", context):
            raise ImportFailure(f"{context}: circle unexpectedly has roundrect_rratio")
        paste_diameter = effective_paste_diameter(width, paste_margin, paste_ratio)
        result["paste"] = (
            {"mode": "follow_copper"}
            if paste_diameter is None
            else {"diameter": decimal_text(paste_diameter), "mode": "circle"}
        )
    else:
        ratio_node = child(node, "roundrect_rratio", context, required=True)
        ratio = decimal(atom(ratio_node, 1, context), context)
        if not Decimal(0) < ratio <= Decimal("0.5"):
            raise ImportFailure(f"{context}: invalid roundrect ratio {ratio}")
        if paste_margin is not None or paste_ratio is not None:
            raise ImportFailure(
                f"{context}: independent roundrect paste aperture is not representable"
            )
        radius = min(width, height) * ratio
        result["copper"]["corner_radius"] = decimal_text(radius)
        result["paste"] = {"mode": "follow_copper"}
    if mask_margin is not None:
        result["mask_expansion"] = decimal_text(mask_margin)
    return result


def import_courtyard(root: Sexp, context: str) -> dict:
    graphics = [
        node
        for node in root[2:]
        if isinstance(node, list) and node and layer_of(node, context) == "F.CrtYd"
    ]
    if not graphics:
        raise ImportFailure(f"{context}: no F.CrtYd geometry")
    normalized = []
    points: list[tuple[Decimal, Decimal]] = []
    for index, node in enumerate(graphics):
        kind = atom(node, 0, context)
        item_context = f"{context}: courtyard graphic {index + 1}"
        width = stroke(node, item_context)
        if width != "0.05":
            raise ImportFailure(f"{item_context}: courtyard stroke is {width}mm, expected 0.05mm")
        if kind == "fp_line":
            start = point(child(node, "start", item_context, required=True), "start", item_context)
            end = point(child(node, "end", item_context, required=True), "end", item_context)
            points.extend([start, end])
            normalized.append(
                {
                    "end": [decimal_text(end[0]), decimal_text(end[1])],
                    "kind": "line",
                    "start": [decimal_text(start[0]), decimal_text(start[1])],
                }
            )
        elif kind == "fp_rect":
            start = point(child(node, "start", item_context, required=True), "start", item_context)
            end = point(child(node, "end", item_context, required=True), "end", item_context)
            fill = child(node, "fill", item_context, required=True)
            if atom(fill, 1, item_context) != "no":
                raise ImportFailure(f"{item_context}: a filled courtyard is unsupported")
            points.extend([start, end])
            normalized.append(
                {
                    "end": [decimal_text(end[0]), decimal_text(end[1])],
                    "kind": "rect",
                    "start": [decimal_text(start[0]), decimal_text(start[1])],
                }
            )
        else:
            raise ImportFailure(f"{item_context}: unsupported {kind!r}")

    kinds = {item["kind"] for item in normalized}
    if kinds == {"rect"} and len(normalized) == 1:
        projection = "exact_rect"
    elif kinds == {"line"}:
        # Every vertex of a closed, non-branching segmented outline has degree 2.
        degrees: Counter[tuple[str, str]] = Counter()
        for item in normalized:
            degrees[tuple(item["start"])] += 1
            degrees[tuple(item["end"])] += 1
        if not degrees or set(degrees.values()) != {2}:
            raise ImportFailure(f"{context}: courtyard lines do not form closed outlines")
        projection = "conservative_axis_aligned_bounding_rect"
    else:
        raise ImportFailure(f"{context}: mixed courtyard primitives are unsupported")

    min_x = min(value[0] for value in points)
    max_x = max(value[0] for value in points)
    min_y = min(value[1] for value in points)
    max_y = max(value[1] for value in points)
    center = [(min_x + max_x) / 2, (min_y + max_y) / 2]
    size = [max_x - min_x, max_y - min_y]
    return {
        "projection": projection,
        "rect": {
            "at": [decimal_text(value) for value in center],
            "size": [decimal_text(value) for value in size],
        },
        "source_graphics": normalized,
        "stroke_width": "0.05",
    }


def import_pin_one(root: Sexp, context: str) -> dict:
    polys = [
        node
        for node in root[2:]
        if isinstance(node, list)
        and node
        and node[0] == "fp_poly"
        and layer_of(node, context) == "F.SilkS"
    ]
    if len(polys) != 1:
        raise ImportFailure(f"{context}: expected one F.SilkS pin-1 polygon, found {len(polys)}")
    poly = polys[0]
    pts = child(poly, "pts", context, required=True)
    assert pts is not None
    points = []
    for entry in pts[1:]:
        if not isinstance(entry, list):
            raise ImportFailure(f"{context}: malformed pin-1 polygon point")
        x, y = point(entry, "xy", context)
        points.append([decimal_text(x), decimal_text(y)])
    if len(points) < 3:
        raise ImportFailure(f"{context}: pin-1 polygon has fewer than three points")
    fill = child(poly, "fill", context, required=True)
    if atom(fill, 1, context) != "yes":
        raise ImportFailure(f"{context}: pin-1 polygon must be filled")
    source_stroke_width = stroke(poly, context)
    if source_stroke_width != "0.12":
        raise ImportFailure(
            f"{context}: pin-1 source stroke is {source_stroke_width}mm, expected 0.12mm"
        )
    return {
        "points": points,
        "source_stroke_width": source_stroke_width,
    }


def import_reference(root: Sexp, context: str) -> list[str]:
    refs = [
        node
        for node in children(root, "property")
        if len(node) >= 3 and node[1:3] == ["Reference", "REF**"]
    ]
    if len(refs) != 1:
        raise ImportFailure(f"{context}: expected one Reference property")
    at = child(refs[0], "at", context, required=True)
    assert at is not None
    if len(at) not in {3, 4}:
        raise ImportFailure(f"{context}: unsupported Reference placement")
    if len(at) == 4 and decimal(atom(at, 3, context), context) != 0:
        raise ImportFailure(f"{context}: rotated Reference placement is unsupported")
    return [
        decimal_text(decimal(atom(at, 1, context), context)),
        decimal_text(decimal(atom(at, 2, context), context)),
    ]


def import_footprint(source_root: pathlib.Path, qualified: str) -> dict:
    library, stem = qualified.split(":", 1)
    relative = pathlib.Path(f"{library}.pretty") / f"{stem}.kicad_mod"
    path = source_root / relative
    if not path.is_file():
        raise ImportFailure(f"missing pinned KiCad footprint {qualified}: {path}")
    root = parse_sexp(path)
    context = qualified
    if len(root) < 2 or root[0] != "footprint" or root[1] != stem:
        raise ImportFailure(f"{context}: root footprint name does not match filename")
    version = child(root, "version", context, required=True)
    generator = child(root, "generator", context, required=True)
    if atom(version, 1, context) != KICAD_FORMAT_VERSION:
        raise ImportFailure(
            f"{context}: format version {atom(version, 1, context)!r}, "
            f"expected {KICAD_FORMAT_VERSION}"
        )

    def root_decimal(key: str) -> Decimal | None:
        value = child(root, key, context)
        return decimal(atom(value, 1, context), context) if value else None

    mask_margin = root_decimal("solder_mask_margin")
    paste_margin = root_decimal("solder_paste_margin")
    paste_ratio = root_decimal("solder_paste_ratio")
    pads = [
        import_pad(node, f"{context}: pad {index + 1}", mask_margin, paste_margin, paste_ratio)
        for index, node in enumerate(children(root, "pad"))
    ]
    if not pads:
        raise ImportFailure(f"{context}: footprint has no electrical pads")
    numbers = [pad["number"] for pad in pads]
    duplicates = sorted(number for number, count in Counter(numbers).items() if count > 1)
    if duplicates:
        raise ImportFailure(f"{context}: duplicate electrical pad numbers {duplicates}")

    descr = child(root, "descr", context)
    tags = child(root, "tags", context)
    source_rules = {}
    if mask_margin is not None:
        source_rules["solder_mask_margin"] = decimal_text(mask_margin)
    if paste_margin is not None:
        source_rules["solder_paste_margin"] = decimal_text(paste_margin)
    if paste_ratio is not None:
        source_rules["solder_paste_ratio"] = decimal_text(paste_ratio)
    return {
        "courtyard": import_courtyard(root, context),
        "description": atom(descr, 1, context) if descr else "",
        "generator": atom(generator, 1, context),
        "kicad_name": qualified,
        "pads": pads,
        "public_name": public_name(qualified),
        "reference_at": import_reference(root, context),
        "source_path": relative.as_posix(),
        "source_rules": source_rules,
        "source_sha256": sha256_file(path),
        "tags": atom(tags, 1, context) if tags else "",
        "version": KICAD_FORMAT_VERSION,
        "pin_1_polygon": import_pin_one(root, context),
    }


def import_snapshot(source_root: pathlib.Path) -> dict:
    if git_head(source_root) != KICAD_COMMIT:
        raise ImportFailure(
            f"KiCad footprints checkout is {git_head(source_root)}; expected {KICAD_COMMIT}"
        )
    dirty = subprocess.check_output(
        [
            "git",
            "-C",
            str(source_root),
            "status",
            "--porcelain",
            "--untracked-files=all",
        ],
        text=True,
    ).strip()
    if dirty:
        raise ImportFailure(
            f"KiCad footprints checkout {source_root} is dirty; refusing source import"
        )
    license_path = source_root / "LICENSE.md"
    if sha256_file(license_path) != KICAD_LICENSE_SHA256:
        raise ImportFailure(f"{license_path}: license checksum differs from pinned notice")
    footprints = [import_footprint(source_root, name) for name in qualified_names()]
    public = [row["public_name"] for row in footprints]
    collisions = sorted(name for name, count in Counter(public).items() if count > 1)
    if collisions:
        raise ImportFailure(f"public-name collisions: {collisions}")
    pad_count = sum(len(row["pads"]) for row in footprints)
    if pad_count != EXPECTED_PADS:
        raise ImportFailure(f"expected {EXPECTED_PADS} pads, found {pad_count}")
    return {
        "coverage": {
            "footprints": len(footprints),
            "footprints_by_library": EXPECTED_COUNTS,
            "pads": pad_count,
        },
        "footprints": footprints,
        "projection_contract": {
            "courtyard": (
                "Exact rectangles are retained. KiCad segmented courtyards are projected "
                "to their conservative axis-aligned bounding rectangle because CoHDL has "
                "one closed courtyard shape."
            ),
            "pads": (
                "Copper shape/size/position/rotation, solder-mask expansion, electrical "
                "number, and effective circular paste-aperture diameter are exact."
            ),
            "silkscreen": (
                "The source filled pin-1 polygon and Reference anchor are retained; other "
                "source package-outline and fabrication graphics are outside this focused "
                "land-pattern projection."
            ),
        },
        "schema_version": 1,
        "source": {
            "commit": KICAD_COMMIT,
            "format_version": KICAD_FORMAT_VERSION,
            "license": "CC-BY-SA-4.0",
            "license_file": "LICENSE.md",
            "license_sha256": KICAD_LICENSE_SHA256,
            "repository": KICAD_REPOSITORY,
        },
    }


def snapshot_bytes(snapshot: dict) -> bytes:
    return (json.dumps(snapshot, indent=2, sort_keys=True) + "\n").encode()


def require_snapshot_hash(data: bytes) -> None:
    actual = sha256_bytes(data)
    if actual != SNAPSHOT_SHA256:
        raise ImportFailure(
            f"{SNAPSHOT}: sha256 {actual}, expected pinned {SNAPSHOT_SHA256}; "
            "review the complete normalized geometry before updating the pin"
        )


def validate_snapshot(snapshot: dict) -> None:
    if snapshot.get("schema_version") != 1:
        raise ImportFailure(f"{SNAPSHOT}: unsupported schema version")
    source = snapshot.get("source", {})
    if source.get("commit") != KICAD_COMMIT:
        raise ImportFailure(f"{SNAPSHOT}: source commit is not pinned commit")
    footprints = snapshot.get("footprints", [])
    counts = Counter(row["kicad_name"].split(":", 1)[0] for row in footprints)
    pads = sum(len(row["pads"]) for row in footprints)
    if len(footprints) != EXPECTED_FOOTPRINTS or dict(counts) != EXPECTED_COUNTS:
        raise ImportFailure(f"{SNAPSHOT}: focused footprint coverage differs")
    if pads != EXPECTED_PADS:
        raise ImportFailure(f"{SNAPSHOT}: expected {EXPECTED_PADS} pads, found {pads}")
    expected_names = qualified_names()
    actual_names = [row["kicad_name"] for row in footprints]
    if actual_names != expected_names:
        raise ImportFailure(f"{SNAPSHOT}: footprint list differs from {MAPPINGS}")
    for row in footprints:
        if row["public_name"] != public_name(row["kicad_name"]):
            raise ImportFailure(f"{SNAPSHOT}: stale public name for {row['kicad_name']}")


def geometry_key(pad: dict) -> tuple:
    copper = pad["copper"]
    paste = pad["paste"]
    return (
        copper["shape"],
        tuple(copper["size"]),
        copper.get("corner_radius", ""),
        pad.get("mask_expansion", ""),
        paste["mode"],
        paste.get("diameter", ""),
    )


def generated_source(package: str, rows: list[dict]) -> str:
    geometries = sorted({geometry_key(pad) for row in rows for pad in row["pads"]})
    pad_names = {geometry: f"P_KICAD_{index:04d}" for index, geometry in enumerate(geometries, 1)}
    out = [
        "// GENERATED by tools/gen_stm32_footprints.py; do not edit.",
        f"// Source: {KICAD_REPOSITORY}",
        f"// Pinned commit: {KICAD_COMMIT}",
        "// KiCad-derived content: CC-BY-SA-4.0 with the KiCad library exception.",
        "// See ../LICENSE.kicad.md and ../docs/README.md for attribution and projection details.",
        "",
    ]
    for geometry in geometries:
        shape, size, corner_radius, mask_expansion, paste_mode, paste_diameter = geometry
        out.append(f"pad {pad_names[geometry]} {{")
        out.append(f"    shape: {'circle' if shape == 'circle' else 'rect'}")
        if shape == "circle":
            out.append(f"    size: ({mm(size[0])})")
        else:
            out.append(f"    size: ({mm(size[0])}, {mm(size[1])})")
        out.extend(["    layer: top_copper", "    plating: smd"])
        if corner_radius:
            out.append(f"    corner_radius: {mm(corner_radius)}")
        if mask_expansion:
            out.append(f"    mask_expansion: {mm(mask_expansion)}")
        if paste_mode == "circle":
            out.append(f"    paste: circle({mm(paste_diameter)})")
        elif paste_mode != "follow_copper":
            raise ImportFailure(f"unknown paste mode {paste_mode!r}")
        out.extend(["}", ""])

    for row in rows:
        out.extend(
            [
                f"// KiCad source: {row['kicad_name']}",
                f"// Source file SHA-256: {row['source_sha256']}",
            ]
        )
        if row["courtyard"]["projection"] == "conservative_axis_aligned_bounding_rect":
            out.append("// The stepped source courtyard is conservatively projected to its bounding rectangle.")
        out.append(f"pub footprint {row['public_name']} {{")
        for pad in row["pads"]:
            x, y = pad["at"]
            rotation = f" rotate {pad['rotation']}" if pad["rotation"] else ""
            out.append(
                f"    pad {pad['number']}: {pad_names[geometry_key(pad)]} "
                f"at ({mm(x)}, {mm(y)}){rotation}"
            )
        points = ", ".join(
            f"({mm(x)}, {mm(y)})" for x, y in row["pin_1_polygon"]["points"]
        )
        out.extend(["    silkscreen {", f"        polygon [{points}]", "    }"])
        courtyard = row["courtyard"]["rect"]
        out.append(
            "    courtyard { shape: rect, "
            f"at: ({mm(courtyard['at'][0])}, {mm(courtyard['at'][1])}), "
            f"size: ({mm(courtyard['size'][0])}, {mm(courtyard['size'][1])}) }}"
        )
        ref = row["reference_at"]
        out.extend(
            [
                f"    silkscreen_ref {{ at: ({mm(ref[0])}, {mm(ref[1])}) }}",
                "}",
                "",
            ]
        )
    return "\n".join(out)


def generated_files(snapshot: dict) -> dict[pathlib.Path, bytes]:
    grouped: dict[str, list[dict]] = {package: [] for package in LIBRARY_TO_PACKAGE.values()}
    for row in snapshot["footprints"]:
        library = row["kicad_name"].split(":", 1)[0]
        grouped[LIBRARY_TO_PACKAGE[library]].append(row)
    files = {
        ROOT / "lib" / package / "src" / "kicad_generated.cohdl": generated_source(package, rows).encode()
        for package, rows in sorted(grouped.items())
    }
    license_bytes = LICENSE_NOTICE.read_bytes()
    if sha256_bytes(license_bytes) != KICAD_LICENSE_SHA256:
        raise ImportFailure(f"{LICENSE_NOTICE}: checksum differs from pinned KiCad license")
    for package in sorted(grouped):
        files[ROOT / "lib" / package / "LICENSE.kicad.md"] = license_bytes
    return files


def validate_generated_pad_sets(snapshot: dict, files: dict[pathlib.Path, bytes]) -> None:
    """Re-parse generated declarations and prove every ordered pad set survived."""
    expected = {
        row["public_name"]: [pad["number"] for pad in row["pads"]]
        for row in snapshot["footprints"]
    }
    actual: dict[str, list[str]] = {}
    header = re.compile(r"^pub footprint ([A-Za-z_][A-Za-z0-9_]*) \{$")
    placement = re.compile(r"^    pad ([A-Za-z0-9_]+): ")
    for path, data in files.items():
        if path.suffix != ".cohdl":
            continue
        name: str | None = None
        depth = 0
        numbers: list[str] = []
        for line in data.decode().splitlines():
            if name is None:
                match = header.fullmatch(line)
                if match:
                    name = match.group(1)
                    depth = 1
                    numbers = []
                continue
            match = placement.match(line)
            if match:
                numbers.append(match.group(1))
            depth += line.count("{") - line.count("}")
            if depth == 0:
                if name in actual:
                    raise ImportFailure(f"{path}: duplicate generated footprint {name}")
                actual[name] = numbers
                name = None
        if name is not None:
            raise ImportFailure(f"{path}: unclosed generated footprint {name}")
    if actual != expected:
        missing = sorted(set(expected) - set(actual))
        extra = sorted(set(actual) - set(expected))
        differing = sorted(
            name for name in set(actual) & set(expected) if actual[name] != expected[name]
        )
        raise ImportFailure(
            "generated pad-number proof failed: "
            f"missing={missing}, extra={extra}, differing={differing}"
        )


def write_or_check(path: pathlib.Path, data: bytes, check: bool) -> bool:
    if check:
        actual = path.read_bytes() if path.is_file() else None
        if actual != data:
            print(f"out of date: {path.relative_to(ROOT)}", file=sys.stderr)
            return False
        return True
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    print(f"wrote {path.relative_to(ROOT)} ({len(data)} bytes, sha256 {sha256_bytes(data)})")
    return True


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--import-source",
        type=pathlib.Path,
        metavar="KICAD_FOOTPRINTS_CHECKOUT",
        help="refresh normalized source data from the exact pinned checkout",
    )
    parser.add_argument("--check", action="store_true", help="fail if generated files differ")
    args = parser.parse_args()

    try:
        if args.import_source:
            snapshot = import_snapshot(args.import_source.resolve())
            encoded_snapshot = snapshot_bytes(snapshot)
            require_snapshot_hash(encoded_snapshot)
            ok = write_or_check(SNAPSHOT, encoded_snapshot, args.check)
        else:
            encoded_snapshot = SNAPSHOT.read_bytes()
            require_snapshot_hash(encoded_snapshot)
            snapshot = json.loads(encoded_snapshot)
            ok = True
        validate_snapshot(snapshot)
        files = generated_files(snapshot)
        validate_generated_pad_sets(snapshot, files)
        for path, data in files.items():
            ok = write_or_check(path, data, args.check) and ok
        if not ok:
            return 1
        print(
            f"verified {snapshot['coverage']['footprints']} footprints and "
            f"{snapshot['coverage']['pads']} exact electrical pad numbers"
        )
        return 0
    except (ImportFailure, OSError, json.JSONDecodeError, KeyError, TypeError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
