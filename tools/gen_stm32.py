#!/usr/bin/env python3
"""Generate the broad STM32 catalog from pinned ST source snapshots.

STM32_open_pin_data identifies package-specific product patterns such as
``STM32F103C(8-B)Tx``: the lowercase ``x`` is an ordering-code wildcard, not an
exact purchasable MPN.  The broad import therefore emits devices.  Exact
``pub part`` declarations are emitted only after an exact order code, pinned ST
pinout, official ST datasheet URL, and concrete dependency-owned land pattern
join uniquely.  ``tools/stm32_data/parts.json`` retains the stronger local-PDF
audit for fourteen F072 identities; ``kicad_parts.json`` freezes the broader
attributed source join.

Normal regeneration is offline and deterministic:

    python3 tools/gen_stm32.py
    python3 tools/gen_stm32.py --check

Refreshing the checked-in snapshot is an explicit maintainer operation.  The
source repositories must be checked out at the exact commits below:

    python3 tools/gen_stm32.py --import-sources \
        /path/to/STM32_open_pin_data /path/to/stm32c5xx-dfp

Only Python's standard library is used.
"""

from __future__ import annotations

import argparse
import hashlib
import itertools
import json
import pathlib
import re
import subprocess
import sys
import xml.etree.ElementTree as ET


ROOT = pathlib.Path(__file__).resolve().parent.parent
SNAPSHOT = ROOT / "tools" / "stm32_data" / "pin_data.json"
ORDER_CODES = ROOT / "tools" / "stm32_data" / "order_codes.txt"
PARTS = ROOT / "tools" / "stm32_data" / "parts.json"
KICAD_PARTS = ROOT / "tools" / "stm32_data" / "kicad_parts.json"
DEFAULT_OUT = ROOT / "lib" / "@st" / "stm32" / "src"
PACKAGE_ROOT = DEFAULT_OUT.parent

OPEN_PIN_DATA_COMMIT = "7d1f1514ed5583ec5007ad91236b4e1d377295b1"
OPEN_PIN_DATA_TAG = "STM32CubeMX-DB.6.0.180"
C5_DFP_COMMIT = "a5f65bc64535cfa723e9d25f58d7ce23d0937aed"
C5_DFP_TAG = "2.1.0"
SNAPSHOT_SCHEMA = 1
PARTS_SCHEMA = 1
KICAD_PARTS_SCHEMA = 1
SNAPSHOT_SHA256 = "c652aec5052601b99b9aacc066354a2f20450599e03d261e6fadebd95a48b682"
ORDER_CODES_SHA256 = "5d6ae32eb20f8cbb82225b90c8332ebc93219a3398a93db634889644832cf8dc"
PARTS_SHA256 = "3d0b76e8461f8fff3d33bbd6c1519d8ab9d6d1e7881e4b5ae494215003b6d6db"
KICAD_PARTS_SHA256 = "3770ec5e52a60cf07f2134e4a721520acf200591a7ab6ead07e0614425e94c1f"

DOC_OPEN = "docs/stm32-open-pin-data.md"
DOC_C5 = "docs/stm32c5xx-dfp.md"
DOC_CATALOG = "docs/stm32-part-catalog.md"

KICAD_PACKAGE_OWNERS = {
    "Package_BGA": "bga",
    "Package_CSP": "csp",
    "Package_QFP": "qfp",
    "Package_SO": "soic",
}
CHECKED_FOCUSED_FOOTPRINTS: set[str] = set()

# Preserve distinct names such as PC2_C/PC3_C.  Only strip the oscillator,
# bracketed-remap, and similar annotations that follow a complete GPIO name.
GPIO_NAME = re.compile(r"^(P[A-Z][0-9]+)(?=$|[ ()/\-\[])")
GROUP = re.compile(r"\(([^()]*)\)")
NON_IDENT = re.compile(r"[^A-Za-z0-9_]")
MULTI_UNDERSCORE = re.compile(r"_+")
NATURAL = re.compile(r"([0-9]+)")

# Pins with these names (optionally followed by ST's numeric discriminator)
# are physically present but must not become one
# electrically shorted multi-pad logical pin.  The package position is part
# of their generated name.
NO_CONNECT_NAME = re.compile(r"^(?:NC|DNC|DNU|RFU)[0-9]*$")
NO_CONNECT_PIN = re.compile(r"^(?:NC|DNC|DNU|RFU)[0-9]*_[A-Za-z0-9_]+$")

# CubeMX's `I/O` type is a configurability bucket, not an electrical role.
# Ordinary GPIO names are bidirectional.  Every dedicated name currently in
# the pinned snapshot is reviewed here; an unfamiliar future `I/O` name fails
# closed instead of silently acquiring a bidirectional role.
GPIO_PIN_NAME = re.compile(r"^P[A-Z][0-9]+(?:_C)?$")
IO_BIDIRECTIONAL_NAMES = {
    "OTG_HS_DM",
    "OTG_HS_DP",
    "PC14OSC32_IN",
    "PC15OSC32_OUT",
    "PF11BOOT0",
}
MONO_BIDIRECTIONAL_NAMES = {
    "OTG1_HSDM",
    "OTG1_HSDP",
    "OTG2_HSDM",
    "OTG2_HSDP",
    # WBA exposes these fixed USB signals under their die-pad names.
    "PD6",
    "PD7",
    "SW_CTL",
    "UCPD1_CC1",
    "UCPD1_CC2",
}
DEDICATED_INPUT_NAMES = {
    "ANT_IN",
    "AOP1_INN",
    "AOP2_INN",
    "CSI_CKN",
    "CSI_CKP",
    "CSI_D0N",
    "CSI_D0P",
    "CSI_D1N",
    "CSI_D1P",
    "LPAWUR_RFI",
    "OPAMP1_VINM",
    "OPAMP2_VINM",
    "OPAMP3_VINM",
    "OSCIN",
    "OSC_IN",
    "OTG1_ID",
    "OTG2_ID",
    "RFI",
    "RFI_N",
    "RFI_P",
}
DEDICATED_OUTPUT_NAMES = {
    "ANT_OUT",
    "DSIHOST_CKN",
    "DSIHOST_CKP",
    "DSIHOST_D0N",
    "DSIHOST_D0P",
    "DSIHOST_D1N",
    "DSIHOST_D1P",
    "DSI_CKN",
    "DSI_CKP",
    "DSI_D0N",
    "DSI_D0P",
    "DSI_D1N",
    "DSI_D1P",
    "OSCOUT",
    "OSC_OUT",
    "RFO_HP",
    "RFO_LP",
    "RF_OUT",
}
# `passive` is deliberate for special analog/RF nodes whose direction cannot
# be represented safely by CoHDL's digital role vocabulary.
DEDICATED_PASSIVE_NAMES = {
    "ANT",
    "ANT_NC",
    "BIAS_HP",
    "BIAS_LP",
    "CSI_REXT",
    "OTG1_TXRTUNE",
    "OTG2_TXRTUNE",
    "REXTPHYHS",
    "RF",
    "RF1",
}

# ST's `Power` type covers both external rails and regulator/passive nodes.
# These are connection nodes, not power inputs.  They stay required so the
# final assembly must account for them, but use the passive role to avoid a
# false supply-driver assertion.
PASSIVE_POWER_PREFIXES = (
    "VCAP",
    "VDDCAP",
    "V08CAP",
    "VDD11",
    "VDD12",
    "VDDCORE",
    "VLX",
    "VFB",
    "VLCD",
    "VR_PA",
    "V15SMPS",
    "VDDA_VCAP",
)

# Every other upstream `Power` name must match an explicitly reviewed supply
# or ground family.  A new name hard-fails its pinout instead of silently
# becoming a power input.
POWER_INPUT_PREFIXES = (
    "VDD",
    "VSS",
    "VDDA",
    "VSSA",
    "VBAT",
    "VREF",
    "DVDD",
    "EXTGND",
    "V12_PHYHS",
)

# Strap/control pins occasionally carry the broad upstream `Power` type.
# They are not supply rails and may legitimately be left at their documented
# reset default.  Product-specific board requirements still come from the
# datasheet; the generated device does not invent them.
CONTROL_POWER_NAMES = {
    "BYPASS_REG",
    "IRROFF",
    "NPOR",
    "PDR_ON",
    "PWR_LP",
    "PWR_ON",
    "REGOFF",
    "RSTN",
}


def local_name(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]


def git_head(path: pathlib.Path) -> str:
    try:
        return subprocess.check_output(
            ["git", "-C", str(path), "rev-parse", "HEAD"], text=True
        ).strip()
    except (OSError, subprocess.CalledProcessError) as exc:
        raise SystemExit(f"cannot read source commit for {path}: {exc}") from exc


def require_commit(path: pathlib.Path, expected: str, label: str) -> None:
    actual = git_head(path)
    if actual != expected:
        raise SystemExit(
            f"{label} is at {actual}; expected pinned commit {expected}"
        )
    dirty = subprocess.check_output(
        ["git", "-C", str(path), "status", "--porcelain", "--untracked-files=all"],
        text=True,
    ).strip()
    if dirty:
        raise SystemExit(f"{label} checkout {path} is dirty; refusing source import")


def open_pin_snapshot(source: pathlib.Path) -> tuple[list[dict], list[dict]]:
    require_commit(source, OPEN_PIN_DATA_COMMIT, "STM32_open_pin_data")
    mcu_dir = source / "mcu"
    if not mcu_dir.is_dir():
        raise SystemExit(f"missing STM32_open_pin_data/mcu under {source}")

    models: list[dict] = []
    pinouts: list[dict] = []
    for path in sorted(mcu_dir.glob("*.xml"), key=lambda item: item.name):
        root = ET.parse(path).getroot()
        family = root.attrib.get("Family", "")
        # `@st/stm32` is the MCU library. STM32MP application processors are a
        # separate product category and need their own interface policy.
        if family.startswith("STM32MP"):
            continue
        ref = root.attrib["RefName"]
        pinout_id = f"open:{path.name}"
        package = root.attrib["Package"]
        incomplete = None
        if "QFN" in package or "QFPN" in package or "-EP" in package:
            incomplete = (
                f"{package} requires an exposed-pad audit but the XML "
                "lists only perimeter positions"
            )
        elif root.attrib.get("HasPowerPad") == "true":
            incomplete = (
                "ST marks HasPowerPad=true but the XML has no exposed-pad position"
            )
        pins = [
            [pin.attrib["Position"], pin.attrib["Name"], pin.attrib["Type"]]
            for pin in root.findall("{*}Pin")
            # CubeMX emits PINREMAP aliases as a second Pin element on the
            # same physical position.  CoHDL needs one physical pin identity;
            # retain the primary entry, whose raw name already records useful
            # bracketed aliases such as `PA11 [PA9]`.
            if not pin.attrib.get("Variant", "").startswith("PINREMAP")
        ]
        if not pins:
            raise SystemExit(f"{path} has no package pins")
        pinouts.append(
            {
                "id": pinout_id,
                "incomplete": incomplete,
                "package": package,
                "pins": pins,
                "source": f"mcu/{path.name}",
            }
        )
        models.append(
            {
                "family": family,
                "kind": "pattern",
                "pinout": pinout_id,
                "ref": ref,
                "source": "open_pin_data",
            }
        )
    return models, pinouts


def c5_raw_type(die_pad: str, details: dict) -> str:
    if NO_CONNECT_NAME.fullmatch(die_pad) or details.get("type") == "not_connected":
        return "NC"
    function = details.get("function")
    if function == "power_supply":
        return "Power"
    if function == "reset" or details.get("io_structure", {}).get("type") == "RST":
        return "Reset"
    if details.get("type") == "gpio":
        return "I/O"
    return "MonoIO"


def c5_snapshot(source: pathlib.Path) -> tuple[list[dict], list[dict]]:
    require_commit(source, C5_DFP_COMMIT, "stm32c5xx-dfp")
    pdsc_path = source / "STMicroelectronics.stm32c5xx_dfp.pdsc"
    if not pdsc_path.is_file():
        raise SystemExit(f"missing {pdsc_path}")
    pdsc = ET.parse(pdsc_path).getroot()

    parents = {child: parent for parent in pdsc.iter() for child in parent}

    def pinout_descriptors(element: ET.Element, skip_variants: bool) -> list[ET.Element]:
        found: list[ET.Element] = []

        def visit(node: ET.Element) -> None:
            for child in node:
                kind = local_name(child.tag)
                if skip_variants and kind == "variant":
                    continue
                if (
                    kind == "descriptor"
                    and child.attrib.get("schemaType") == "pinout"
                ):
                    found.append(child)
                visit(child)

        visit(element)
        return found

    pinouts_by_path: dict[str, dict] = {}
    models: list[dict] = []
    seen_models: dict[str, str] = {}
    for variant in (element for element in pdsc.iter() if local_name(element.tag) == "variant"):
        ref = variant.attrib.get("Dvariant")
        if not ref:
            continue
        descriptors = pinout_descriptors(variant, skip_variants=False)
        if not descriptors:
            ancestor = parents.get(variant)
            while ancestor is not None and local_name(ancestor.tag) != "device":
                ancestor = parents.get(ancestor)
            if ancestor is not None:
                descriptors = pinout_descriptors(ancestor, skip_variants=True)
        if len(descriptors) != 1:
            raise SystemExit(
                f"C5 variant {ref} has {len(descriptors)} pinout descriptors"
            )
        relative = descriptors[0].attrib["path"]
        pinout_id = f"c5:{relative}"
        if relative not in pinouts_by_path:
            path = source / relative
            data = json.loads(path.read_text())
            die_pads = data["die_pads"]
            pins = []
            for bond in data["bonds"]:
                die_pad = bond["die_pad"]
                details = die_pads.get(die_pad, {})
                pins.append(
                    [bond["position"], die_pad, c5_raw_type(die_pad, details)]
                )
            pinouts_by_path[relative] = {
                "id": pinout_id,
                "incomplete": (
                    "DFP package_type=4-edges-internal but bonds omit the exposed pad"
                    if data["characteristics"].get("package_type")
                    == "4-edges-internal"
                    else None
                ),
                "package": data["characteristics"]["package_name"],
                "pins": pins,
                "source": relative,
            }

        # Tape/reel changes packing, never the electrical device.  Some DFP
        # revisions list both spellings, so collapse them deterministically.
        device_ref = ref[:-2] if ref.endswith("TR") else ref
        previous = seen_models.get(device_ref)
        if previous is not None:
            if previous != pinout_id:
                raise SystemExit(
                    f"C5 model {device_ref} maps to both {previous} and {pinout_id}"
                )
            continue
        seen_models[device_ref] = pinout_id
        models.append(
            {
                "family": "STM32C5",
                "kind": "exact",
                "pinout": pinout_id,
                "ref": device_ref,
                "source": "c5_dfp",
            }
        )
    return models, list(pinouts_by_path.values())


def import_sources(open_source: pathlib.Path, c5_source: pathlib.Path) -> dict:
    open_models, open_pinouts = open_pin_snapshot(open_source)
    c5_models, c5_pinouts = c5_snapshot(c5_source)
    models = sorted(open_models + c5_models, key=lambda row: (row["family"], row["ref"]))
    pinouts = sorted(open_pinouts + c5_pinouts, key=lambda row: row["id"])
    snapshot = {
        "models": models,
        "pinouts": pinouts,
        "schema_version": SNAPSHOT_SCHEMA,
        "sources": {
            "open_pin_data": {
                "commit": OPEN_PIN_DATA_COMMIT,
                "tag": OPEN_PIN_DATA_TAG,
                "url": "https://github.com/STMicroelectronics/STM32_open_pin_data",
            },
            "c5_dfp": {
                "commit": C5_DFP_COMMIT,
                "tag": C5_DFP_TAG,
                "url": "https://github.com/STMicroelectronics/stm32c5xx-dfp",
            },
        },
    }
    SNAPSHOT.parent.mkdir(parents=True, exist_ok=True)
    encoded = (
        json.dumps(snapshot, ensure_ascii=True, separators=(",", ":"), sort_keys=True)
        + "\n"
    ).encode()
    actual_hash = hashlib.sha256(encoded).hexdigest()
    if actual_hash != SNAPSHOT_SHA256:
        raise SystemExit(
            f"refreshed STM32 snapshot hash is {actual_hash}; "
            f"review it and update SNAPSHOT_SHA256 (expected {SNAPSHOT_SHA256})"
        )
    SNAPSHOT.write_bytes(encoded)
    return snapshot


def load_snapshot() -> dict:
    if not SNAPSHOT.is_file():
        raise SystemExit(
            f"missing {SNAPSHOT}; refresh it with --import-sources first"
        )
    encoded = SNAPSHOT.read_bytes()
    actual_hash = hashlib.sha256(encoded).hexdigest()
    if actual_hash != SNAPSHOT_SHA256:
        raise SystemExit(
            f"STM32 snapshot hash is {actual_hash}; expected {SNAPSHOT_SHA256}"
        )
    data = json.loads(encoded)
    if data.get("schema_version") != SNAPSHOT_SCHEMA:
        raise SystemExit(
            f"unsupported STM32 snapshot schema {data.get('schema_version')!r}"
        )
    expected = {
        "open_pin_data": OPEN_PIN_DATA_COMMIT,
        "c5_dfp": C5_DFP_COMMIT,
    }
    for name, commit in expected.items():
        actual = data.get("sources", {}).get(name, {}).get("commit")
        if actual != commit:
            raise SystemExit(
                f"snapshot source {name} is {actual!r}; generator pins {commit}"
            )
    return data


def expand_ref(ref: str, kind: str) -> list[str]:
    if kind == "exact":
        return [ref]
    matches = list(GROUP.finditer(ref))
    if not matches:
        return [ref]
    pieces: list[str] = []
    choices: list[list[str]] = []
    end = 0
    for match in matches:
        pieces.append(ref[end : match.start()])
        choices.append(match.group(1).split("-"))
        end = match.end()
    pieces.append(ref[end:])
    expanded = []
    for selected in itertools.product(*choices):
        value = pieces[0]
        for choice, suffix in zip(selected, pieces[1:]):
            value += choice + suffix
        expanded.append(value)
    return sorted(expanded)


def natural_key(value: str) -> tuple:
    return tuple(
        int(piece) if piece.isdigit() else piece
        for piece in NATURAL.split(value)
        if piece
    )


def portfolio_matches(models: list[dict]) -> dict:
    """Map the exact ST portfolio inventory onto source pinout patterns.

    The website inventory supplies exact factual order-code spelling only. It
    never supplies pin data, connection semantics, or footprints; those must
    arrive through the separately pinned source join before a part is emitted.
    """
    if not ORDER_CODES.is_file():
        raise ValueError(f"missing exact STM32 portfolio inventory {ORDER_CODES}")
    encoded = ORDER_CODES.read_bytes()
    actual_hash = hashlib.sha256(encoded).hexdigest()
    if actual_hash != ORDER_CODES_SHA256:
        raise ValueError(
            f"STM32 portfolio inventory hash is {actual_hash}; "
            f"expected {ORDER_CODES_SHA256}"
        )
    rows = [
        line.strip()
        for line in encoded.decode().splitlines()
        if line.strip() and not line.startswith("#")
    ]
    if len(rows) != len(set(rows)):
        raise ValueError("duplicate exact order code in STM32 portfolio inventory")
    if any(not re.fullmatch(r"STM32[A-Z0-9]+", code) for code in rows):
        raise ValueError("malformed exact order code in STM32 portfolio inventory")

    by_identity: dict[str, list[str]] = {}
    for code in rows:
        identity = code[:-2] if code.endswith("TR") else code
        by_identity.setdefault(identity, []).append(code)

    candidates: list[tuple[int, str, re.Pattern[str]]] = []
    for index, model in enumerate(models):
        for expanded in expand_ref(model["ref"], model["kind"]):
            if model["kind"] == "exact":
                pattern = re.escape(expanded)
            else:
                pattern = re.escape(expanded).replace("x", "[A-Z0-9]")
            candidates.append((index, expanded, re.compile(f"^{pattern}$")))

    unmatched = []
    matched_identities: dict[str, tuple[int, str]] = {}
    for identity in sorted(by_identity):
        matches = [
            (index, expanded)
            for index, expanded, pattern in candidates
            if pattern.fullmatch(identity)
        ]
        if len(matches) > 1:
            rendered = ", ".join(
                f"{models[index]['ref']} -> {expanded}"
                for index, expanded in matches
            )
            raise ValueError(
                f"portfolio identity {identity} ambiguously matches {rendered}"
            )
        if not matches:
            unmatched.append(identity)
            continue
        index, expanded = matches[0]
        matched_identities[identity] = (index, expanded)

    return {
        "by_identity": by_identity,
        "matched_identities": matched_identities,
        "rows": rows,
        "unmatched": unmatched,
    }


def load_audited_parts(snapshot: dict, portfolio: dict) -> list[dict]:
    """Load the fail-closed exact-part overlay and bind it to source models."""
    if not PARTS.is_file():
        raise ValueError(f"missing audited STM32 part overlay {PARTS}")
    encoded = PARTS.read_bytes()
    actual_hash = hashlib.sha256(encoded).hexdigest()
    if actual_hash != PARTS_SHA256:
        raise ValueError(
            f"STM32 part overlay hash is {actual_hash}; expected {PARTS_SHA256}"
        )
    data = json.loads(encoded)
    root_keys = {"documents", "package_variants", "parts", "schema_version"}
    if not isinstance(data, dict) or set(data) != root_keys:
        raise ValueError(
            "STM32 part overlay must contain exactly "
            + ", ".join(sorted(root_keys))
        )
    if data.get("schema_version") != PARTS_SCHEMA:
        raise ValueError(
            f"unsupported STM32 part overlay schema {data.get('schema_version')!r}"
        )
    for table in ("documents", "package_variants", "parts"):
        if not isinstance(data.get(table), list):
            raise ValueError(f"STM32 part overlay {table} must be a list")

    model_rows = snapshot["models"]
    pinouts = {row["id"]: row for row in snapshot["pinouts"]}
    portfolio_rows = set(portfolio["rows"])
    identifier = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
    metadata_id = re.compile(r"^[a-z0-9][a-z0-9._-]*$")
    qualified_identifier = re.compile(
        r"^(?:[A-Za-z_][A-Za-z0-9_]*::)+[A-Za-z_][A-Za-z0-9_]*$"
    )

    def require_focused_footprint(symbol: str, package_id: str) -> None:
        segments = symbol.split("::")
        package_root = ROOT / "lib" / segments[0]
        source_root = package_root / "src"
        if not (package_root / "cohdl.toml").is_file() or not source_root.is_dir():
            raise ValueError(
                f"STM32 package variant {package_id} footprint owner "
                f"{segments[0]!r} is not a focused bare library"
            )
        expected_modules = segments[1:-1]
        declaration = re.compile(
            rf"^pub footprint {re.escape(segments[-1])}\s*\{{", re.MULTILINE
        )
        matches = []
        for source in sorted(source_root.rglob("*.cohdl")):
            relative = source.relative_to(source_root).with_suffix("")
            actual_modules = [] if len(relative.parts) == 1 else list(relative.parts)
            if actual_modules == expected_modules and declaration.search(
                source.read_text()
            ):
                matches.append(source)
        if len(matches) != 1:
            raise ValueError(
                f"STM32 package variant {package_id} footprint {symbol} resolves "
                f"to {len(matches)} focused-library declarations"
            )

    def safe_text(value: object, label: str) -> str:
        if (
            not isinstance(value, str)
            or not value
            or any(char in value for char in {'"', "\\", "\n", "\r"})
        ):
            raise ValueError(f"invalid {label} in audited STM32 part overlay")
        return value

    document_keys = {
        "date",
        "id",
        "official_url",
        "path",
        "revision",
        "sha256",
        "title",
    }
    documents: dict[str, dict] = {}
    for document in data["documents"]:
        if not isinstance(document, dict) or set(document) != document_keys:
            raise ValueError(
                "each STM32 document row must contain exactly "
                + ", ".join(sorted(document_keys))
            )
        doc_id = document["id"]
        if not isinstance(doc_id, str) or not metadata_id.fullmatch(doc_id):
            raise ValueError(f"invalid STM32 document id {doc_id!r}")
        if doc_id in documents:
            raise ValueError(f"duplicate STM32 document id {doc_id}")
        doc = document["path"]
        if not isinstance(doc, str):
            raise ValueError(f"invalid path for STM32 document {doc_id}")
        doc_path = pathlib.PurePosixPath(doc)
        if (
            doc_path.is_absolute()
            or len(doc_path.parts) < 2
            or doc_path.parts[0] != "docs"
            or any(piece in {"", ".", ".."} for piece in doc_path.parts)
        ):
            raise ValueError(
                f"STM32 document {doc_id} is not a package-relative docs path"
            )
        local_doc = PACKAGE_ROOT.joinpath(*doc_path.parts).resolve()
        try:
            local_doc.relative_to(PACKAGE_ROOT.resolve())
        except ValueError as exc:
            raise ValueError(f"STM32 document {doc_id} escapes the package") from exc
        if not local_doc.is_file():
            raise ValueError(f"STM32 document {doc_id} is missing local file {doc}")
        expected_doc_hash = document["sha256"]
        if not isinstance(expected_doc_hash, str) or not re.fullmatch(
            r"[0-9a-f]{64}", expected_doc_hash
        ):
            raise ValueError(f"invalid hash for STM32 document {doc_id}")
        actual_doc_hash = hashlib.sha256(local_doc.read_bytes()).hexdigest()
        if actual_doc_hash != expected_doc_hash:
            raise ValueError(
                f"STM32 document {doc_id} hash is {actual_doc_hash}; "
                f"expected {expected_doc_hash}"
            )
        safe_text(document["title"], f"title for document {doc_id}")
        safe_text(document["revision"], f"revision for document {doc_id}")
        date = safe_text(document["date"], f"date for document {doc_id}")
        if not re.fullmatch(r"[0-9]{4}-[0-9]{2}-[0-9]{2}", date):
            raise ValueError(f"invalid date for STM32 document {doc_id}")
        official_url = safe_text(
            document["official_url"], f"official URL for document {doc_id}"
        )
        if not official_url.startswith("https://www.st.com/"):
            raise ValueError(
                f"STM32 document {doc_id} official URL must be on www.st.com"
            )
        documents[doc_id] = document

    package_keys = {
        "document",
        "footprint",
        "geometry_locator",
        "id",
        "pinout_package",
    }
    package_variants: dict[str, dict] = {}
    for package in data["package_variants"]:
        if not isinstance(package, dict) or set(package) != package_keys:
            raise ValueError(
                "each STM32 package variant row must contain exactly "
                + ", ".join(sorted(package_keys))
            )
        package_id = package["id"]
        if not isinstance(package_id, str) or not metadata_id.fullmatch(package_id):
            raise ValueError(f"invalid STM32 package variant id {package_id!r}")
        if package_id in package_variants:
            raise ValueError(f"duplicate STM32 package variant id {package_id}")
        document_id = package["document"]
        if document_id not in documents:
            raise ValueError(
                f"STM32 package variant {package_id} references unknown "
                f"document {document_id!r}"
            )
        footprint = package["footprint"]
        if not isinstance(footprint, str) or not qualified_identifier.fullmatch(
            footprint
        ):
            raise ValueError(
                f"STM32 package variant {package_id} footprint must be qualified"
            )
        require_focused_footprint(footprint, package_id)
        safe_text(
            package["pinout_package"],
            f"pinout package for package variant {package_id}",
        )
        safe_text(
            package["geometry_locator"],
            f"geometry locator for package variant {package_id}",
        )
        package_variants[package_id] = package

    seen_names: set[str] = set()
    seen_mpns: set[str] = set()
    used_package_ids: set[str] = set()
    used_document_ids: set[str] = set()
    loaded = []
    part_keys = {
        "alts",
        "device",
        "family",
        "name",
        "package_variant",
        "primary",
    }
    avl_keys = {"mfr", "mpn", "order_code_source"}
    source_keys = {"document", "locator"}
    for raw_part in data["parts"]:
        if not isinstance(raw_part, dict) or set(raw_part) != part_keys:
            raise ValueError(
                "each STM32 part overlay row must contain exactly "
                + ", ".join(sorted(part_keys))
            )
        name = raw_part["name"]
        device = raw_part["device"]
        family = raw_part["family"]
        if not isinstance(name, str) or not identifier.fullmatch(name):
            raise ValueError(f"invalid audited STM32 part name {name!r}")
        if name in seen_names:
            raise ValueError(f"duplicate audited STM32 part name {name}")
        seen_names.add(name)
        if not isinstance(device, str) or not identifier.fullmatch(device):
            raise ValueError(f"invalid audited STM32 device name {device!r}")
        if not isinstance(family, str):
            raise ValueError(f"invalid family for audited STM32 part {name}")
        # This also checks that the family maps to a safe root module filename.
        family_file(family)
        package_id = raw_part["package_variant"]
        if package_id not in package_variants:
            raise ValueError(
                f"audited STM32 part {name} references unknown package variant "
                f"{package_id!r}"
            )
        package = package_variants[package_id]
        used_package_ids.add(package_id)
        used_document_ids.add(package["document"])

        primary = raw_part["primary"]
        alts = raw_part["alts"]
        if not isinstance(primary, dict) or set(primary) != avl_keys:
            raise ValueError(f"invalid primary AVL row for audited STM32 part {name}")
        if not isinstance(alts, list) or any(
            not isinstance(alt, dict) or set(alt) != avl_keys for alt in alts
        ):
            raise ValueError(f"invalid alternate AVL rows for audited STM32 part {name}")
        entries = [primary, *alts]
        entry_documents = []
        for entry in entries:
            mfr = safe_text(
                entry["mfr"], f"manufacturer in audited STM32 part {name}"
            )
            mpn = entry["mpn"]
            if not isinstance(mpn, str) or not re.fullmatch(r"STM32[A-Z0-9]+", mpn):
                raise ValueError(f"invalid MPN {mpn!r} in audited STM32 part {name}")
            if mpn not in portfolio_rows:
                raise ValueError(
                    f"audited STM32 part {name} MPN {mpn} is absent from the "
                    "pinned exact order-code inventory"
                )
            if mpn in seen_mpns:
                raise ValueError(f"audited STM32 MPN {mpn} is declared more than once")
            seen_mpns.add(mpn)
            source = entry["order_code_source"]
            if not isinstance(source, dict) or set(source) != source_keys:
                raise ValueError(
                    f"invalid order-code source for {mpn} in audited STM32 part {name}"
                )
            source_document = source["document"]
            if source_document not in documents:
                raise ValueError(
                    f"audited STM32 MPN {mpn} references unknown document "
                    f"{source_document!r}"
                )
            safe_text(source["locator"], f"order-code locator for {mpn}")
            entry_documents.append(source_document)
            used_document_ids.add(source_document)

        primary_mpn = primary["mpn"]
        if name != "MCU_" + primary_mpn:
            raise ValueError(
                f"audited STM32 part {name} must be named MCU_{primary_mpn}"
            )
        identity = (
            primary_mpn[:-2] if primary_mpn.endswith("TR") else primary_mpn
        )
        inventory_codes = set(portfolio["by_identity"].get(identity, []))
        if primary_mpn.endswith("TR"):
            if alts or inventory_codes != {primary_mpn}:
                raise ValueError(
                    f"audited STM32 part {name} may use a terminal-TR primary "
                    "only when it is the inventory's sole packaging row"
                )
        else:
            for alt in alts:
                if (
                    alt["mfr"] != primary["mfr"]
                    or alt["mpn"] != primary_mpn + "TR"
                ):
                    raise ValueError(
                        f"audited STM32 part {name} alternates must be only the "
                        "same manufacturer's terminal-TR packing code"
                    )
        overlay_codes = {entry["mpn"] for entry in entries}
        if overlay_codes != inventory_codes:
            missing = sorted(inventory_codes - overlay_codes)
            extra = sorted(overlay_codes - inventory_codes)
            raise ValueError(
                f"audited STM32 part {name} does not cover its full packaging identity; "
                f"missing={missing}, extra={extra}"
            )
        match = portfolio["matched_identities"].get(identity)
        if match is None:
            raise ValueError(
                f"audited STM32 part {name} has no unique pinned pinout match"
            )
        model_index, expanded_device = match
        model = model_rows[model_index]
        if expanded_device != device:
            raise ValueError(
                f"audited STM32 part {name} selects device {device}, but exact "
                f"order code {identity} maps to {expanded_device}"
            )
        if model["family"] != family:
            raise ValueError(
                f"audited STM32 part {name} selects family {family}, but its "
                f"pinned model is in {model['family']}"
            )
        pinout = pinouts.get(model["pinout"])
        if pinout is None:
            raise ValueError(
                f"audited STM32 part {name} model references missing pinout "
                f"{model['pinout']}"
            )
        if package["pinout_package"] != pinout["package"]:
            raise ValueError(
                f"audited STM32 part {name} package variant says "
                f"{package['pinout_package']}, but pinned device {device} says "
                f"{pinout['package']}"
            )

        part = dict(raw_part)
        part["model_index"] = model_index
        part["identity"] = identity
        part["footprint"] = package["footprint"]
        doc_ids = [package["document"], *entry_documents]
        part["docs"] = [
            documents[doc_id]["path"] for doc_id in dict.fromkeys(doc_ids)
        ]
        loaded.append(part)

    unused_packages = sorted(set(package_variants) - used_package_ids)
    if unused_packages:
        raise ValueError(
            f"unused STM32 package variants in audited overlay: {unused_packages}"
        )
    unused_documents = sorted(set(documents) - used_document_ids)
    if unused_documents:
        raise ValueError(
            f"unused STM32 documents in audited overlay: {unused_documents}"
        )
    return sorted(loaded, key=lambda part: (part["family"], part["name"]))


def cohdl_kicad_footprint(qualified: str) -> str:
    """Map a pinned KiCad library name to its focused CoHDL package symbol."""
    if qualified.count(":") != 1:
        raise ValueError(f"invalid KiCad footprint name {qualified!r}")
    library, stem = qualified.split(":", 1)
    owner = KICAD_PACKAGE_OWNERS.get(library)
    if owner is None:
        raise ValueError(f"unsupported KiCad footprint library {library!r}")
    symbol = "KICAD_" + re.sub(r"[^A-Za-z0-9]+", "_", stem).strip("_").upper()
    if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", symbol):
        raise ValueError(f"KiCad footprint {qualified!r} cannot name a CoHDL symbol")
    return f"{owner}::{symbol}"


def require_focused_footprint_symbol(symbol: str, context: str) -> None:
    if symbol in CHECKED_FOCUSED_FOOTPRINTS:
        return
    segments = symbol.split("::")
    package_root = ROOT / "lib" / segments[0]
    source_root = package_root / "src"
    if not (package_root / "cohdl.toml").is_file() or not source_root.is_dir():
        raise ValueError(
            f"{context} footprint owner {segments[0]!r} is not a focused bare library"
        )
    expected_modules = segments[1:-1]
    declaration = re.compile(
        rf"^pub footprint {re.escape(segments[-1])}\s*\{{", re.MULTILINE
    )
    matches = []
    for source in sorted(source_root.rglob("*.cohdl")):
        relative = source.relative_to(source_root).with_suffix("")
        actual_modules = [] if len(relative.parts) == 1 else list(relative.parts)
        if actual_modules == expected_modules and declaration.search(source.read_text()):
            matches.append(source)
    if len(matches) != 1:
        raise ValueError(
            f"{context} footprint {symbol} resolves to {len(matches)} "
            "focused-library declarations"
        )
    CHECKED_FOCUSED_FOOTPRINTS.add(symbol)


def load_kicad_part_catalog() -> dict:
    """Load the frozen, attributed KiCad/ST exact-part cross-reference."""
    if not KICAD_PARTS.is_file():
        raise ValueError(f"missing STM32 KiCad part catalog {KICAD_PARTS}")
    encoded = KICAD_PARTS.read_bytes()
    actual_hash = hashlib.sha256(encoded).hexdigest()
    if actual_hash != KICAD_PARTS_SHA256:
        raise ValueError(
            f"STM32 KiCad part catalog hash is {actual_hash}; "
            f"expected {KICAD_PARTS_SHA256}"
        )
    data = json.loads(encoded)
    root_keys = {"coverage", "mappings", "schema_version", "sources"}
    if not isinstance(data, dict) or set(data) != root_keys:
        raise ValueError(
            "STM32 KiCad part catalog must contain exactly "
            + ", ".join(sorted(root_keys))
        )
    if data.get("schema_version") != KICAD_PARTS_SCHEMA:
        raise ValueError(
            f"unsupported STM32 KiCad catalog schema {data.get('schema_version')!r}"
        )
    expected_coverage = {
        "electrical_identities": 2389,
        "exact_order_code_rows": 3303,
        "unique_datasheets": 1240,
        "unique_footprints": 103,
    }
    if data.get("coverage") != expected_coverage:
        raise ValueError(
            f"STM32 KiCad catalog coverage changed: {data.get('coverage')!r}; "
            f"expected {expected_coverage!r}"
        )
    expected_sources = {
        "kicad_footprints": "819223b66f96508feaeaa305301b5e6bb5c1038b",
        "kicad_symbols": "7800d91437ce44e2ed0928f2ad31a287457b8a68",
    }
    sources = data.get("sources")
    if not isinstance(sources, dict) or set(sources) != set(expected_sources):
        raise ValueError("STM32 KiCad catalog has an invalid sources table")
    for name, commit in expected_sources.items():
        source = sources[name]
        if (
            not isinstance(source, dict)
            or source.get("commit") != commit
            or source.get("license") != "CC-BY-SA-4.0"
            or source.get("license_sha256")
            != "45d2bce75e5a4208f5afb01b8fb2c406e700371c4fe2b5f5cd5c443d46db4d8f"
        ):
            raise ValueError(f"STM32 KiCad catalog source {name!r} changed")
    if not isinstance(data.get("mappings"), list):
        raise ValueError("STM32 KiCad catalog mappings must be a list")
    return data


def load_catalog_parts(
    snapshot: dict,
    portfolio: dict,
    catalog: dict,
    audited_parts: list[dict],
) -> list[dict]:
    """Derive exact parts from the frozen source/device/footprint join."""
    mapping_keys = {
        "datasheet",
        "device",
        "family",
        "identity",
        "kicad_footprint",
        "package",
        "pinout_id",
        "source_model",
        "symbol",
        "symbol_path",
    }
    model_by_ref = {row["ref"]: (index, row) for index, row in enumerate(snapshot["models"])}
    if len(model_by_ref) != len(snapshot["models"]):
        raise ValueError("duplicate STM32 source model reference")
    pinouts = {row["id"]: row for row in snapshot["pinouts"]}
    audited_by_identity = {part["identity"]: part for part in audited_parts}
    seen_identities: set[str] = set()
    seen_mpns = {
        row["mpn"]
        for part in audited_parts
        for row in [part["primary"], *part["alts"]]
    }
    derived = []

    for mapping in catalog["mappings"]:
        if not isinstance(mapping, dict) or set(mapping) != mapping_keys:
            raise ValueError(
                "each STM32 KiCad mapping must contain exactly "
                + ", ".join(sorted(mapping_keys))
            )
        identity = mapping["identity"]
        if not isinstance(identity, str) or not re.fullmatch(r"STM32[A-Z0-9]+", identity):
            raise ValueError(f"invalid STM32 KiCad identity {identity!r}")
        if identity in seen_identities:
            raise ValueError(f"duplicate STM32 KiCad identity {identity}")
        seen_identities.add(identity)
        match = portfolio["matched_identities"].get(identity)
        if match is None:
            raise ValueError(f"STM32 KiCad identity {identity} has no exact pinout match")
        model_index, device = match
        model = snapshot["models"][model_index]
        if mapping["source_model"] != model["ref"]:
            raise ValueError(
                f"STM32 KiCad identity {identity} selects source model "
                f"{mapping['source_model']!r}, expected {model['ref']!r}"
            )
        if mapping["device"] != device or mapping["family"] != model["family"]:
            raise ValueError(f"STM32 KiCad identity {identity} device/family drifted")
        if mapping["pinout_id"] != model["pinout"]:
            raise ValueError(f"STM32 KiCad identity {identity} pinout drifted")
        pinout = pinouts.get(model["pinout"])
        if pinout is None or pinout.get("incomplete"):
            raise ValueError(f"STM32 KiCad identity {identity} selects an incomplete pinout")
        normalized_pins(pinout)
        if mapping["package"] != pinout["package"]:
            raise ValueError(f"STM32 KiCad identity {identity} package drifted")
        datasheet = mapping["datasheet"]
        if (
            not isinstance(datasheet, str)
            or not datasheet.startswith("https://www.st.com/")
            or any(char in datasheet for char in {'"', "\\", "\n", "\r"})
        ):
            raise ValueError(f"invalid official datasheet URL for {identity}")
        footprint = cohdl_kicad_footprint(mapping["kicad_footprint"])
        require_focused_footprint_symbol(
            footprint, f"STM32 KiCad identity {identity}"
        )

        # The manually reviewed DS9826 overlay is a stronger primary source;
        # keep its exact geometry and local PDF for those fourteen identities.
        if identity in audited_by_identity:
            continue

        codes = sorted(portfolio["by_identity"][identity])
        non_reel = [code for code in codes if not code.endswith("TR")]
        if len(non_reel) > 1:
            raise ValueError(f"STM32 identity {identity} has multiple non-TR order codes")
        if non_reel:
            primary_mpn = non_reel[0]
            alternate_codes = [code for code in codes if code != primary_mpn]
            if alternate_codes not in ([], [primary_mpn + "TR"]):
                raise ValueError(
                    f"STM32 identity {identity} has unsupported packaging rows "
                    f"{alternate_codes}"
                )
        else:
            if len(codes) != 1 or not codes[0].endswith("TR"):
                raise ValueError(f"STM32 identity {identity} has no deterministic primary")
            primary_mpn = codes[0]
            alternate_codes = []
        for code in codes:
            if code in seen_mpns:
                raise ValueError(f"STM32 MPN {code} is emitted more than once")
            seen_mpns.add(code)
        derived.append(
            {
                "alts": [
                    {"mfr": "STMicroelectronics", "mpn": code}
                    for code in alternate_codes
                ],
                "device": device,
                "docs": [DOC_CATALOG],
                "family": model["family"],
                "footprint": footprint,
                "identity": identity,
                "model_index": model_index,
                "name": "MCU_" + primary_mpn,
                "primary": {"mfr": "STMicroelectronics", "mpn": primary_mpn},
            }
        )

    if seen_identities != set(row["identity"] for row in catalog["mappings"]):
        raise ValueError("STM32 KiCad catalog identity accounting failed")
    combined = [*audited_parts, *derived]
    if len(combined) != catalog["coverage"]["electrical_identities"]:
        raise ValueError(
            f"STM32 exact-part coverage is {len(combined)} identities; expected "
            f"{catalog['coverage']['electrical_identities']}"
        )
    rows = sum(1 + len(part["alts"]) for part in combined)
    if rows != catalog["coverage"]["exact_order_code_rows"]:
        raise ValueError(
            f"STM32 exact-part coverage is {rows} rows; expected "
            f"{catalog['coverage']['exact_order_code_rows']}"
        )
    names = [part["name"] for part in combined]
    if len(names) != len(set(names)):
        raise ValueError("duplicate generated STM32 part name")
    return sorted(combined, key=lambda part: (part["family"], part["name"]))


def catalog_document(catalog: dict) -> str:
    """Render the local per-identity provenance document shipped in the tar."""
    coverage = catalog["coverage"]
    lines = [
        "# STM32 exact-part catalog sources",
        "",
        "This generated local document is the source index for the exact `pub part`",
        "declarations in this package. It records the complete MPN-to-device,",
        "official ST datasheet, and dependency-owned land-pattern join.",
        "",
        f"- Electrical identities: {coverage['electrical_identities']}",
        f"- Exact order-code rows (including terminal `TR` packaging): {coverage['exact_order_code_rows']}",
        f"- Official ST datasheet URLs: {coverage['unique_datasheets']}",
        f"- Concrete KiCad footprint variants: {coverage['unique_footprints']}",
        "- Exact-order-code source: STMicroelectronics MCU portfolio, retrieved 2026-08-27",
        "  (<https://www.st.com/content/st_com/en/stm32-mcu-developer-zone/mcu-portfolio.html>)",
        f"- Exact-order-code snapshot SHA-256: `{ORDER_CODES_SHA256}`",
        "- ST pin sources: the pinned BSD-3-Clause repositories recorded in",
        "  `stm32-open-pin-data.md` and `stm32c5xx-dfp.md`.",
        "- KiCad symbol commit: `7800d91437ce44e2ed0928f2ad31a287457b8a68`",
        "- KiCad footprint commit: `819223b66f96508feaeaa305301b5e6bb5c1038b`",
        "- KiCad library license: CC-BY-SA-4.0 with its design-output exception;",
        "  the unmodified notice is shipped as `KICAD_LIBRARY_LICENSE.md`.",
        "",
        "The KiCad mapping is secondary evidence. Import admitted a row only when",
        "the concrete footprint's complete SMD pad-number set exactly equalled the",
        "physical positions in ST's pinned pin source. No package-name-only or",
        "nearest-footprint guesses are admitted. The fourteen DS9826-reviewed F072",
        "parts retain their stronger local-PDF geometry bindings.",
        "",
        "## Per-identity source map",
        "",
        "| Electrical identity | Device | CoHDL footprint | Official ST datasheet |",
        "|---|---|---|---|",
    ]
    for row in catalog["mappings"]:
        lines.append(
            f"| `{row['identity']}` | `{row['device']}` | "
            f"`{cohdl_kicad_footprint(row['kicad_footprint'])}` | "
            f"<{row['datasheet']}> |"
        )
    lines.append("")
    return "\n".join(lines)


def normalize_name(raw: str, position: str) -> str:
    raw = raw.strip()
    gpio = GPIO_NAME.match(raw)
    if gpio:
        return gpio.group(1)
    if raw in {"NRST (NRST)", "NRST(NRST)"}:
        return "NRST"
    value = raw.replace("+", "P").replace("-", "M")
    value = NON_IDENT.sub("_", value)
    value = MULTI_UNDERSCORE.sub("_", value).strip("_")
    if not value:
        raise ValueError(f"pin {raw!r} at {position} normalizes to an empty name")
    if value[0].isdigit():
        value = "PIN_" + value
    if NO_CONNECT_NAME.fullmatch(value):
        suffix = NON_IDENT.sub("_", position)
        value = f"{value}_{suffix}"
    return value


def classify(raw_type: str, normalized: str) -> tuple[str, str]:
    if NO_CONNECT_PIN.fullmatch(normalized):
        return "optional", "passive"
    # CubeMX uses Boot/Reset/Power/MonoIO as configurability classes, so
    # semantic controls must be stable across whichever class a family chose.
    if normalized in CONTROL_POWER_NAMES:
        return "optional", "input"
    if normalized in DEDICATED_INPUT_NAMES:
        return "optional", "input"
    if normalized in DEDICATED_OUTPUT_NAMES:
        return "optional", "output"
    if normalized in DEDICATED_PASSIVE_NAMES:
        return "optional", "passive"
    if raw_type == "I/O":
        if (
            GPIO_PIN_NAME.fullmatch(normalized)
            or normalized in IO_BIDIRECTIONAL_NAMES
        ):
            return "optional", "bidirectional"
        raise ValueError(
            f"unreviewed ST I/O name {normalized!r}; add an explicit semantic rule"
        )
    if raw_type == "Reset":
        return "optional", "input"
    if raw_type == "Boot":
        return "required", "input"
    if raw_type == "NC":
        return "optional", "passive"
    if raw_type == "MonoIO":
        if normalized in {"VREFP", "VREFM", "VREF_P", "VREF_M"}:
            return "required", "power_in"
        if normalized in MONO_BIDIRECTIONAL_NAMES:
            return "optional", "bidirectional"
        raise ValueError(
            f"unreviewed ST MonoIO name {normalized!r}; add an explicit semantic rule"
        )
    if raw_type == "Power":
        if normalized.startswith(PASSIVE_POWER_PREFIXES):
            return "required", "passive"
        if normalized.startswith(POWER_INPUT_PREFIXES):
            return "required", "power_in"
        raise ValueError(
            f"unreviewed ST Power name {normalized!r}; add an explicit semantic rule"
        )
    raise ValueError(f"unknown ST pin type {raw_type!r} on {normalized}")


def normalized_pins(pinout: dict) -> list[tuple[str, list[str], str, str, str | None]]:
    ordered = sorted(pinout["pins"], key=lambda pin: natural_key(pin[0]))
    by_position: dict[str, list[tuple[str, str, str, str]]] = {}
    for position, raw_name, raw_type in ordered:
        name = normalize_name(raw_name, position)
        obligation, role = classify(raw_type, name)
        by_position.setdefault(position, []).append(
            (name, obligation, role, raw_name)
        )

    physical: list[tuple[str, str, str, str, frozenset[str], str | None]] = []
    for position, entries in by_position.items():
        obligations = {entry[1] for entry in entries}
        roles = {entry[2] for entry in entries}
        if len(obligations) != 1 or len(roles) != 1:
            rendered = ", ".join(
                f"{raw}={obligation}/{role}"
                for _, obligation, role, raw in entries
            )
            raise ValueError(
                f"{pinout['id']} has incompatible aliases at position "
                f"{position}: {rendered}"
            )
        names = sorted(
            dict.fromkeys(entry[0] for entry in entries), key=natural_key
        )
        raw_names = frozenset(entry[3] for entry in entries)
        name = "_OR_".join(names)
        note = None
        if len(names) > 1:
            note = (
                f"ST remappable position {position}: "
                + " | ".join(sorted(raw_names))
            )
        physical.append(
            (name, position, entries[0][1], entries[0][2], raw_names, note)
        )

    groups: dict[
        str, tuple[list[str], str, str, frozenset[str], list[str]]
    ] = {}
    for name, position, obligation, role, raw_names, note in physical:
        if name not in groups:
            groups[name] = (
                [position],
                obligation,
                role,
                raw_names,
                [note] if note else [],
            )
            continue
        positions, old_obligation, old_role, old_raw_names, notes = groups[name]
        if (old_obligation, old_role) != (obligation, role):
            raise ValueError(
                f"{pinout['id']} normalization collision on {name}: "
                f"{old_obligation}/{old_role} versus {obligation}/{role}"
            )
        # A repeated manufacturer pin name denotes one logical signal bonded
        # to multiple package positions (the common VDD/VSS case). Different
        # raw names collapsing to one identifier are rejected for review.
        if raw_names != old_raw_names:
            raise ValueError(
                f"{pinout['id']} normalization collision: "
                f"{sorted(old_raw_names)!r} and {sorted(raw_names)!r} become {name}"
            )
        positions.append(position)
        if note:
            notes.append(note)
    return [
        (name, values[0], values[1], values[2], "; ".join(values[4]) or None)
        for name, values in groups.items()
    ]


def audit_snapshot_pin_policy(pinouts: list[dict]) -> None:
    """Apply the closed name/type policy even to incomplete package rows."""
    for pinout in pinouts:
        for position, raw_name, raw_type in pinout["pins"]:
            try:
                classify(raw_type, normalize_name(raw_name, position))
            except ValueError as exc:
                raise ValueError(
                    f"{pinout['id']} position {position} ({raw_name!r}): {exc}"
                ) from exc


def family_file(family: str) -> str:
    if not family.startswith("STM32"):
        raise ValueError(f"unexpected family {family!r}")
    stem = family.removeprefix("STM32").lower().replace("+", "_plus")
    if not re.fullmatch(r"[a-z0-9_]+", stem):
        raise ValueError(f"family {family!r} cannot name a module")
    return f"{stem}.cohdl"


def emit_family(
    family: str,
    models: list[dict],
    pinouts: dict[str, dict],
    parts: list[dict],
) -> str:
    expanded_count = sum(len(expand_ref(row["ref"], row["kind"])) for row in models)
    lines = [
        f"// {family} package-specific MCU device models.",
        "//",
        "// GENERATED by tools/gen_stm32.py — do not hand-edit.",
        f"// {expanded_count} models derived from pinned ST device data.",
        "// Ordering-code `x` is retained where ST publishes a wildcard; these",
        "// declarations are devices, not guessed purchasable parts.",
        "// Exact parts below exist only when the source join supplies a local",
        "// provenance record and a complete dependency-owned footprint.",
        "",
    ]
    docs_by_device: dict[str, list[str]] = {}
    for part in parts:
        docs_by_device.setdefault(part["device"], []).extend(part["docs"])
    emitted: dict[str, str] = {}
    for model in models:
        pinout = pinouts[model["pinout"]]
        pins = pinout["normalized_pins"]
        doc = DOC_C5 if model["source"] == "c5_dfp" else DOC_OPEN
        for name in expand_ref(model["ref"], model["kind"]):
            if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", name):
                raise ValueError(f"expanded model {name!r} is not a CoHDL identifier")
            previous = emitted.get(name)
            if previous is not None:
                if previous != model["pinout"]:
                    raise ValueError(
                        f"model {name} maps to both {previous} and {model['pinout']}"
                    )
                continue
            emitted[name] = model["pinout"]
            lines.extend(
                [
                    f"// ST model {name}; {pinout['package']}; source {pinout['source']}.",
                ]
            )
            docs = [doc, *sorted(set(docs_by_device.get(name, [])))]
            lines.extend(f'#[doc("{device_doc}")]' for device_doc in docs)
            lines.extend([f"pub device {name} {{", "    pins {"])
            for pin_name, positions, obligation, role, note in pins:
                if note:
                    lines.append(f"        // {note}")
                prefix = f"{obligation} {pin_name}: "
                suffix = f" [{role}]"
                oneline = prefix + ", ".join(positions) + suffix
                if 8 + len(oneline) <= 100 or len(positions) <= 1:
                    lines.append("        " + oneline)
                else:
                    # Mirror RFC-009's canonical 100-column pin-bus layout so
                    # generated bytes already pass `cohdl fmt --check`.
                    continuation = " " * len(prefix)
                    current = prefix
                    first_on_line = True
                    for index, position in enumerate(positions):
                        piece = (
                            position + ","
                            if index + 1 < len(positions)
                            else position + suffix
                        )
                        extra = 0 if first_on_line else 1
                        if (
                            not first_on_line
                            and 8 + len(current) + extra + len(piece) > 100
                        ):
                            lines.append("        " + current)
                            current = continuation
                            first_on_line = True
                        if not first_on_line:
                            current += " "
                        current += piece
                        first_on_line = False
                    lines.append("        " + current)
            lines.extend(["    }", "}", "", f"impl IC for {name} {{}}", ""])

    if parts:
        lines.extend(
            [
                "// Source-backed exact order codes with complete copper geometry.",
                "// Tape/reel order codes are alternates only when they collapse",
                "// to the same checked electrical and package identity.",
                "",
            ]
        )
        for part in parts:
            primary = part["primary"]
            lines.extend(
                f'#[doc("{part_doc}")]' for part_doc in part["docs"]
            )
            lines.extend(
                [
                    f'pub part {part["name"]}: {part["device"]} {{',
                    f'    primary {{ mfr: "{primary["mfr"]}", mpn: "{primary["mpn"]}",',
                    f'              footprint: {part["footprint"]} }}',
                ]
            )
            for alt in sorted(
                part["alts"], key=lambda row: (row["mfr"], row["mpn"])
            ):
                lines.append(
                    f'    alt {{ mfr: "{alt["mfr"]}", mpn: "{alt["mpn"]}" }}'
                )
            lines.extend(["}", ""])
    return "\n".join(lines)


def generated_sources(snapshot: dict, catalog: dict) -> dict[str, str]:
    portfolio = portfolio_matches(snapshot["models"])
    audited_parts = load_audited_parts(snapshot, portfolio)
    exact_parts = load_catalog_parts(snapshot, portfolio, catalog, audited_parts)
    audit_snapshot_pin_policy(snapshot["pinouts"])
    pinouts = {row["id"]: row for row in snapshot["pinouts"]}
    if len(pinouts) != len(snapshot["pinouts"]):
        raise ValueError("duplicate pinout id in STM32 snapshot")
    excluded_pinouts: dict[str, str] = {}
    for pinout_id, pinout in pinouts.items():
        if pinout.get("incomplete"):
            excluded_pinouts[pinout_id] = (
                f"{pinout_id} is incomplete: {pinout['incomplete']}"
            )
            continue
        try:
            pinout["normalized_pins"] = normalized_pins(pinout)
        except ValueError as exc:
            excluded_pinouts[pinout_id] = str(exc)

    by_family: dict[str, list[dict]] = {}
    excluded_models: list[tuple[str, str]] = []
    for model in snapshot["models"]:
        if model["pinout"] not in pinouts:
            raise ValueError(
                f"model {model['ref']} references missing {model['pinout']}"
            )
        if model["pinout"] in excluded_pinouts:
            excluded_models.append(
                (model["ref"], excluded_pinouts[model["pinout"]])
            )
            continue
        by_family.setdefault(model["family"], []).append(model)
    excluded_model_indices = {
        index
        for index, model in enumerate(snapshot["models"])
        if model["pinout"] in excluded_pinouts
    }
    for part in exact_parts:
        if part["model_index"] in excluded_model_indices:
            model = snapshot["models"][part["model_index"]]
            raise ValueError(
                f"exact STM32 part {part['name']} selects excluded pinout "
                f"{model['pinout']}: {excluded_pinouts[model['pinout']]}"
            )
    parts_by_family: dict[str, list[dict]] = {}
    for part in exact_parts:
        parts_by_family.setdefault(part["family"], []).append(part)
    for family in parts_by_family:
        if family not in by_family:
            raise ValueError(f"exact STM32 parts select ungenerated family {family}")

    sources = {
        family_file(family): emit_family(
            family,
            sorted(models, key=lambda row: row["ref"]),
            pinouts,
            sorted(parts_by_family.get(family, []), key=lambda part: part["name"]),
        )
        for family, models in sorted(by_family.items())
    }
    emitted_devices = sum(content.count("pub device ") for content in sources.values())
    emitted_portfolio_identities = sum(
        1
        for index, _ in portfolio["matched_identities"].values()
        if index not in excluded_model_indices
    )
    emitted_portfolio_rows = sum(
        len(portfolio["by_identity"][identity])
        for identity, (index, _) in portfolio["matched_identities"].items()
        if index not in excluded_model_indices
    )
    exact_order_code_rows = sum(1 + len(part["alts"]) for part in exact_parts)
    exact_identities = {part["identity"] for part in exact_parts}
    coverage = [
        "// Generated STM32 catalog coverage — do not hand-edit.",
        "//",
        f"// Emitted device models: {emitted_devices}",
        f"// Excluded source patterns/variants: {len(excluded_models)}",
        f"// Exact ST portfolio order-code rows: {len(portfolio['rows'])}",
        f"// Electrical identities after terminal-TR collapse: {len(portfolio['by_identity'])}",
        "// Identities matched exactly once to a pinned pinout: "
        f"{len(portfolio['matched_identities'])}",
        f"// Matched identities represented by emitted devices: {emitted_portfolio_identities}",
        f"// Exact order-code rows represented by emitted devices: {emitted_portfolio_rows}",
        f"// Portfolio identities unmatched to pinned pinouts: {len(portfolio['unmatched'])}",
        f"// Source-backed pub parts with concrete footprints: {len(exact_parts)}",
        f"// Electrical identities covered by exact parts: {len(exact_identities)}",
        f"// Exact order-code rows covered by exact parts: {exact_order_code_rows}",
        "// Represented exact rows awaiting fabrication audit: "
        f"{emitted_portfolio_rows - exact_order_code_rows}",
        "// All portfolio rows not emitted as exact parts: "
        f"{len(portfolio['rows']) - exact_order_code_rows}",
        "//",
        "// The broad exact portfolio supplies exact MPN spelling only. A row",
        "// becomes a part only through the frozen ST-pin/KiCad-footprint join",
        "// whose pad set and official ST",
        "// datasheet URL are checked; the website never supplies pin semantics or",
        "// fabrication geometry.",
        "//",
        "// Product boundary: STM32MP application processors are not imported into",
        "// this MCU package; they require a separate interface and package policy.",
        "//",
        "// Exclusions are fail-closed: the upstream package pinout cannot be",
        "// represented as one unambiguous CoHDL device without an alias/remap",
        "// decision or another reviewed normalization rule.",
        "//",
    ]
    coverage.extend(
        f"// - {ref}: {reason}" for ref, reason in sorted(excluded_models)
    )
    coverage.append("")
    sources["catalog_coverage.cohdl"] = "\n".join(coverage)
    return sources


def write_or_check(sources: dict[str, str], out: pathlib.Path, check: bool) -> None:
    mismatches = []
    if not check:
        out.mkdir(parents=True, exist_ok=True)
    expected = set(sources)
    if out.is_dir():
        for path in sorted(out.glob("*.cohdl")):
            if path.name in expected:
                continue
            content = path.read_text()
            generated = (
                "GENERATED by tools/gen_stm32.py" in content
                or content.startswith("// Generated STM32 catalog coverage")
            )
            if not generated:
                continue
            if check:
                mismatches.append(str(path) + " (stale generated file)")
            else:
                path.unlink()
    for name, content in sorted(sources.items()):
        path = out / name
        encoded = content.encode()
        if check:
            if not path.is_file() or path.read_bytes() != encoded:
                mismatches.append(str(path))
        else:
            path.write_bytes(encoded)
    if mismatches:
        raise SystemExit(
            "generated STM32 sources are stale:\n  " + "\n  ".join(mismatches)
        )
    verb = "checked" if check else "wrote"
    devices = sum(content.count("pub device ") for content in sources.values())
    pins = sum(
        content.count("        required ") + content.count("        optional ")
        for content in sources.values()
    )
    parts = sum(content.count("pub part ") for content in sources.values())
    print(
        f"{verb} {len(sources)} files, {devices} devices, {pins} logical pins, "
        f"{parts} exact parts under {out}"
    )


def write_or_check_catalog_document(catalog: dict, check: bool) -> None:
    path = PACKAGE_ROOT / DOC_CATALOG
    encoded = catalog_document(catalog).encode()
    if check:
        if not path.is_file() or path.read_bytes() != encoded:
            raise SystemExit(f"generated STM32 catalog document is stale: {path}")
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(encoded)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--import-sources",
        nargs=2,
        metavar=("OPEN_PIN_DATA", "C5_DFP"),
        type=pathlib.Path,
        help="refresh the frozen snapshot from the two pinned ST repositories",
    )
    parser.add_argument("--check", action="store_true", help="fail if generated bytes differ")
    parser.add_argument(
        "--output-root",
        type=pathlib.Path,
        default=DEFAULT_OUT,
        help="directory for generated family .cohdl files",
    )
    args = parser.parse_args()
    if args.import_sources and args.check:
        parser.error("--import-sources and --check are mutually exclusive")
    if args.import_sources:
        snapshot = import_sources(*args.import_sources)
    else:
        snapshot = load_snapshot()
    catalog = load_kicad_part_catalog()
    write_or_check_catalog_document(catalog, args.check)
    write_or_check(generated_sources(snapshot, catalog), args.output_root, args.check)


if __name__ == "__main__":
    try:
        main()
    except (KeyError, TypeError, ValueError) as exc:
        raise SystemExit(f"STM32 generation failed: {exc}") from exc
