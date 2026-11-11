"""Fail-closed supplier bindings for the offline library generators.

Both sourcing tables are optional during generation. Missing or malformed data
produces no binding; it never makes an otherwise valid catalog regeneration
fail. Entries participate only with an unambiguous effective footprint; an
alternate without an explicit footprint inherits its primary's footprint.
"""

from __future__ import annotations

from datetime import date
from decimal import Decimal, InvalidOperation
import hashlib
import json
from pathlib import Path, PurePosixPath
import re
from typing import Any

from .w3_rule import RULE as W3_RULE, required as w3_required, sample_ok as w3_sample_ok


SCHEMA_VERSION = 1
SHA256 = re.compile(r"[0-9a-f]{64}")
ROTATIONS = {0, 90, 180, 270}
MAP_FIELDS = {"mfr", "mpn", "pn", "rule", "category", "snapshot_sha256", "date"}
FRAME_FIELDS = {
    "id", "package", "package_version", "footprint", "supplier", "pn",
    "mapping", "measured", "exception", "evidence", "evidence_ref",
}
# `sampled` is optional: a row without it reads as `sampled: null`.
SAMPLED_FIELD = "sampled"


def _rows(path: Path) -> list[dict[str, Any]]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
        rows = data["rows"]
        if data["schema_version"] != SCHEMA_VERSION or not isinstance(rows, list):
            return []
        return [row for row in rows if isinstance(row, dict)]
    except (OSError, ValueError, KeyError, TypeError):
        return []


def _text(row: dict[str, Any], fields: set[str]) -> bool:
    return all(isinstance(row.get(field), str) and row[field] for field in fields)


def _date(value: Any) -> bool:
    if not isinstance(value, str):
        return False
    try:
        return date.fromisoformat(value).isoformat() == value
    except ValueError:
        return False


def _number(value: Any) -> Decimal | None:
    if isinstance(value, bool) or not isinstance(value, (int, float, str)):
        return None
    try:
        number = Decimal(str(value))
        return number if number.is_finite() else None
    except InvalidOperation:
        return None


def _mapping(row: dict[str, Any]) -> tuple[Decimal, Decimal, int] | None:
    mapping = row.get("mapping")
    if not isinstance(mapping, dict) or set(mapping) != {"dx_mm", "dy_mm", "rotate_deg"}:
        return None
    dx, dy = _number(mapping["dx_mm"]), _number(mapping["dy_mm"])
    rotate = _number(mapping["rotate_deg"])
    if dx is None or dy is None or rotate is None or rotate != rotate.to_integral_value():
        return None
    if not 0 <= rotate < 360:
        return None
    rotate = int(rotate)
    nonzero = dx != 0 or dy != 0 or rotate != 0
    exception = row.get("exception")
    if nonzero != (isinstance(exception, str) and bool(exception.strip())):
        return None
    return dx, dy, rotate


def _inside(root: Path, record: Any) -> Path | None:
    """ROOT / RECORD when RECORD names an existing file inside ROOT."""
    if not isinstance(record, str) or not record:
        return None
    path = PurePosixPath(record)
    if path.is_absolute() or ".." in path.parts:
        return None
    try:
        resolved = (root / path).resolve()
        resolved.relative_to(root.resolve())
        return resolved if resolved.is_file() else None
    except (OSError, ValueError):
        return None


def _zero(row: dict[str, Any]) -> bool:
    mapping = _mapping(row)
    return mapping is not None and mapping == (0, 0, 0)


def _sampled(row: dict[str, Any], rows_by_id: dict[str, dict[str, Any]], root: Path) -> bool:
    """A row inferred under the W3 rule: every check of the group record holds."""
    sampled = row[SAMPLED_FIELD]
    if not isinstance(sampled, dict) or set(sampled) != {"group", "record", "note"}:
        return False
    if not (isinstance(sampled["note"], str) and sampled["note"].strip()):
        return False
    path = _inside(root, sampled["record"])
    if path is None:
        return False
    try:
        record = json.loads(path.read_text(encoding="utf-8"))
        if record["schema_version"] != SCHEMA_VERSION or record["rule"] != W3_RULE:
            return False
        groups = [g for g in record["groups"] if g["id"] == sampled["group"]]
        if len(groups) != 1:
            return False
        group = groups[0]
        if (group["status"] != "active" or group["package"] != W3_RULE["package"]
                or group["type"] not in W3_RULE["types"]
                or (group["package"], group["package_version"], group["footprint"])
                != (row["package"], row["package_version"], row["footprint"])):
            return False
        land = group["land"]
        if not land or not all(
            (found := _inside(root, name)) is not None
            and hashlib.sha256(found.read_bytes()).hexdigest() == digest
            for name, digest in land.items()
        ):
            return False
        members = set(group["members"])
        if len(members) != len(group["members"]) or not isinstance(group["not_drawn"], list):
            return False
        samples = group["samples"]
        sample_pns = {s["pn"] for s in samples}
        if (row["pn"] not in members or row["pn"] in sample_pns
                or (not W3_RULE["infer_zero_stock"] and row["pn"] in group["zero_stock"])
                or (not W3_RULE["infer_not_drawn"] and row["pn"] in group["not_drawn"])):
            return False
        if len(sample_pns) != len(samples) or len(samples) < w3_required(len(members)):
            return False
        for sample in samples:
            target = rows_by_id.get(sample["row"])
            if (sample["pn"] not in members or target is None or target is row
                    or target.get("evidence") is None or target.get("evidence_ref") is not None
                    or target.get(SAMPLED_FIELD) is not None or target["pn"] != sample["pn"]
                    or (target["package"], target["package_version"], target["footprint"])
                    != (row["package"], row["package_version"], row["footprint"])
                    or not _zero(target) or not _evidence(target, rows_by_id, root)
                    or not w3_sample_ok(sample)):
                return False
    except (OSError, ValueError, KeyError, TypeError, AttributeError):
        return False
    return _zero(row)


def _evidence(row: dict[str, Any], rows_by_id: dict[str, dict[str, Any]], root: Path) -> bool:
    evidence, reference = row.get("evidence"), row.get("evidence_ref")
    sampled = row.get(SAMPLED_FIELD)
    if [evidence, reference, sampled].count(None) != 2:
        return False
    if sampled is not None:
        return _sampled(row, rows_by_id, root)
    if evidence is not None:
        if not isinstance(evidence, dict) or set(evidence) - {"note"} != {"method", "date", "record"}:
            return False
        if "note" in evidence and not (
            isinstance(evidence["note"], str) and evidence["note"].strip()
        ):
            return False
        if evidence["method"] not in {"preview", "product_page"} or not _date(evidence["date"]):
            return False
        return _inside(root, evidence["record"]) is not None
    if not isinstance(reference, dict) or set(reference) != {"row", "reason", "note"}:
        return False
    target = rows_by_id.get(reference["row"])
    return (
        target is not None
        and target is not row
        and target.get("evidence") is not None
        and target.get("evidence_ref") is None
        and target.get(SAMPLED_FIELD) is None
        and reference["reason"] in {"same_supplier_footprint", "same_preview_board"}
        and isinstance(reference["note"], str)
        and bool(reference["note"].strip())
        and _evidence(target, rows_by_id, root)
    )


def package_metadata(manifest: Path) -> tuple[str, str] | None:
    try:
        package: dict[str, Any] | None = None
        found = False
        for raw_line in manifest.read_text(encoding="utf-8").splitlines():
            line = raw_line.split("#", 1)[0].strip()
            if line.startswith("["):
                if found:
                    break
                found = line == "[package]"
                package = {} if found else None
            elif package is not None and "=" in line:
                key, value = (part.strip() for part in line.split("=", 1))
                if key in {"name", "version"} and len(value) >= 2 and value[0] == value[-1] == '"':
                    package[key] = value[1:-1]
        if package is None or "name" not in package or "version" not in package:
            return None
        name, version = package.get("name"), package.get("version")
        return (name, version) if name and version else None
    except OSError:
        return None


def package_root(name: str) -> str:
    """The module root of a package name, as ``src/pipeline.rs`` derives it."""
    name = name[1:] if name.startswith("@") else name
    root = "".join(c if (c.isascii() and c.isalnum()) or c == "_" else "_" for c in name)
    return "_" + root if not root or root[0].isdigit() else root


def _number_text(value: Decimal) -> str:
    if value == 0:
        return "0"
    text = format(value.normalize(), "f")
    return text.rstrip("0").rstrip(".") if "." in text else text


class Bindings:
    """The validated subset of the two tables, indexed for generator lookup."""

    def __init__(self, root: Path):
        self._root = root
        self._manifests: dict[str, Path] | None = None
        self._by_identity: dict[tuple[str, str], list[str]] = {}
        map_rows = _rows(root / "tools" / "sourcing_data" / "lcsc_map.json")
        map_valid = True
        for row in map_rows:
            valid = (
                set(row) == MAP_FIELDS
                and _text(row, {"mfr", "mpn", "pn"})
                and row["rule"] in {"exact", "alias", "manual"}
                and row["category"] in {"basic", "preferred", "extended"}
                and isinstance(row["snapshot_sha256"], str)
                and SHA256.fullmatch(row["snapshot_sha256"]) is not None
                and _date(row["date"])
            )
            map_valid &= valid
            if valid:
                self._by_identity.setdefault((row["mfr"], row["mpn"]), []).append(row["pn"])
        if not map_valid:
            self._by_identity = {}

        self._frames: dict[tuple[str, str, str, str], dict[str, Any]] = {}
        self._ambiguous: set[tuple[str, str, str, str]] = set()
        frame_rows = _rows(root / "tools" / "sourcing_data" / "jlc_frames.json")
        rows_by_id = {
            row["id"]: row for row in frame_rows if isinstance(row.get("id"), str) and row["id"]
        }
        if len(rows_by_id) != len(frame_rows):
            frame_rows = []
        for row in frame_rows:
            valid = (
                set(row) - {SAMPLED_FIELD} == FRAME_FIELDS
                and _text(row, {"id", "package", "package_version", "footprint", "pn"})
                and row["supplier"] == "lcsc"
                and _mapping(row) is not None
                and _evidence(row, rows_by_id, root)
            )
            if not valid:
                continue
            key = (row["package"], row["package_version"], row["footprint"], row["pn"])
            if key in self._frames:
                self._ambiguous.add(key)
            else:
                self._frames[key] = row

    def _package_manifests(self) -> dict[str, Path]:
        """Map each library package's module root to its manifest, as
        ``pipeline::package_root`` spells it (``@espressif/esp32`` is
        ``espressif_esp32``, ``load-switch`` is ``load_switch``)."""
        if self._manifests is None:
            self._manifests = {}
            lib = self._root / "lib"
            for manifest in sorted([*lib.glob("*/cohdl.toml"), *lib.glob("@*/*/cohdl.toml")]):
                metadata = package_metadata(manifest)
                if metadata is not None:
                    self._manifests.setdefault(package_root(metadata[0]), manifest)
        return self._manifests

    def clause(self, manifest: Path, footprint: str, mfr: str, mpn: str) -> str:
        """Return one complete ``supplier lcsc`` clause, or an empty string.

        A footprint path whose first segment is a library package's module
        root is owned by that package; any other path (bare, or through a
        local module) is owned by ``manifest``'s own package.
        """
        segments = footprint.split("::")
        footprint = segments[-1]
        if not all(segment.isidentifier() for segment in segments):
            return ""
        if len(segments) > 1 and segments[0] in self._package_manifests():
            manifest = self._package_manifests()[segments[0]]
        metadata = package_metadata(manifest)
        if metadata is None:
            return ""
        pns = self._by_identity.get((mfr, mpn), [])
        if len(pns) != 1:
            return ""
        package, version = metadata
        key = (package, version, footprint, pns[0])
        if key in self._ambiguous:
            return ""
        row = self._frames.get(key)
        if row is None:
            return ""
        mapping = _mapping(row)
        if mapping is None:
            return ""
        dx, dy, rotate = mapping
        clause = f'supplier lcsc "{pns[0]}"'
        if dx != 0 or dy != 0:
            clause += f" at ({_number_text(dx)}mm, {_number_text(dy)}mm)"
        if rotate != 0:
            clause += f" rotate {rotate}"
        return clause


def avl_entry(
    bindings: Bindings,
    manifest: Path,
    kind: str,
    mfr: str,
    mpn: str,
    footprint: str,
    effective_footprint: str | None = None,
) -> str:
    """Render one AVL entry exactly as ``cohdl fmt`` lays it out.

    Mirrors ``Formatter::wrapped`` in src/fmt.rs: one line when it fits in 100
    columns, otherwise members packed greedily, each continuation aligned under
    the first member (column 14 for ``primary``, 10 for ``alt``).
    """
    explicit_footprint = effective_footprint is None
    if effective_footprint is None:
        effective_footprint = footprint
    fields = [f'mfr: "{mfr}"', f'mpn: "{mpn}"']
    if explicit_footprint:
        fields.append(f"footprint: {footprint}")
    supplier = bindings.clause(manifest, effective_footprint, mfr, mpn)
    if supplier:
        fields.append(supplier)
    return "\n".join(wrapped(f"{kind} {{ ", fields, " }")) + "\n"


WIDTH = 100
INDENT = "    "


def wrapped(prefix: str, members: list[str], suffix: str) -> list[str]:
    """``Formatter::wrapped`` at indent level 1, returning indented lines."""
    oneline = prefix + ", ".join(members) + suffix
    if len(INDENT) + len(oneline) <= WIDTH or len(members) <= 1:
        return [INDENT + oneline]
    lines: list[str] = []
    line, first = prefix, True
    for index, member in enumerate(members):
        piece = member + ("," if index + 1 < len(members) else suffix)
        if not first and len(INDENT) + len(line) + 1 + len(piece) > WIDTH:
            lines.append(INDENT + line)
            line, first = " " * len(prefix), True
        line += piece if first else " " + piece
        first = False
    lines.append(INDENT + line)
    return lines
