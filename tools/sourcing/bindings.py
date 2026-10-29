"""Fail-closed supplier bindings for the offline library generators.

Both sourcing tables are optional during generation. Missing or malformed data
produces no binding; it never makes an otherwise valid catalog regeneration
fail. Only primary entries with a local, unambiguous footprint participate.
"""

from __future__ import annotations

from datetime import date
from decimal import Decimal, InvalidOperation
import json
from pathlib import Path, PurePosixPath
import re
from typing import Any


SCHEMA_VERSION = 1
SHA256 = re.compile(r"[0-9a-f]{64}")
ROTATIONS = {0, 90, 180, 270}
MAP_FIELDS = {"mfr", "mpn", "pn", "rule", "category", "snapshot_sha256", "date"}
FRAME_FIELDS = {
    "id", "package", "package_version", "footprint", "supplier", "pn",
    "mapping", "measured", "exception", "evidence", "evidence_ref",
}


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
    rotate = mapping["rotate_deg"]
    if dx is None or dy is None or isinstance(rotate, bool) or not isinstance(rotate, int):
        return None
    if not 0 <= rotate <= 359:
        return None
    nonzero = dx != 0 or dy != 0 or rotate != 0
    exception = row.get("exception")
    if nonzero != (isinstance(exception, str) and bool(exception.strip())):
        return None
    return dx, dy, rotate


def _evidence(row: dict[str, Any], rows_by_id: dict[str, dict[str, Any]], root: Path) -> bool:
    evidence, reference = row.get("evidence"), row.get("evidence_ref")
    if (evidence is None) == (reference is None):
        return False
    if evidence is not None:
        if not isinstance(evidence, dict) or set(evidence) != {"method", "date", "record"}:
            return False
        if evidence["method"] not in {"preview", "product_page"} or not _date(evidence["date"]):
            return False
        record = evidence["record"]
        if not isinstance(record, str) or not record:
            return False
        path = PurePosixPath(record)
        if path.is_absolute() or ".." in path.parts:
            return False
        try:
            resolved = (root / path).resolve()
            resolved.relative_to(root.resolve())
            return resolved.is_file()
        except (OSError, ValueError):
            return False
    if not isinstance(reference, dict) or set(reference) != {"row", "reason", "note"}:
        return False
    target = rows_by_id.get(reference["row"])
    return (
        target is not None
        and target is not row
        and target.get("evidence") is not None
        and target.get("evidence_ref") is None
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


def _number_text(value: Decimal) -> str:
    if value == 0:
        return "0"
    text = format(value.normalize(), "f")
    return text.rstrip("0").rstrip(".") if "." in text else text


class Bindings:
    """The validated subset of the two tables, indexed for generator lookup."""

    def __init__(self, root: Path):
        self._root = root
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
                set(row) == FRAME_FIELDS
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

    def clause(self, manifest: Path, footprint: str, mfr: str, mpn: str) -> str:
        """Return one complete ``supplier lcsc`` clause, or an empty string."""
        owner, _, footprint = footprint.rpartition("::")
        if not footprint.isidentifier() or (owner and not re.fullmatch(r"[a-z][a-z0-9_]*", owner)):
            return ""
        if owner:
            manifest = self._root / "lib" / owner / "cohdl.toml"
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
