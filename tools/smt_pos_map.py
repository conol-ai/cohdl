"""Pure footprint-frame and coverage checks for RFC-034 supplier maps.

This module deliberately has no KiCad or CSV dependency.  Board coordinates
are integer nanometres in KiCad's +y-down frame.  Map coordinates are decimal
millimetre strings.  The CLI adapter supplies light-weight dictionaries or
objects with the corresponding ``x``, ``y``, ``anchor`` and ``orientation``
attributes; this module performs no I/O.
"""

import hashlib
import json
import math
import re
from collections import deque
from decimal import Decimal, InvalidOperation, ROUND_HALF_UP

from smt_pos_lib import neutral_rows


TOLERANCE_NM = 1000
_NM_PER_MM = 1_000_000
_MM_RE = re.compile(r"[+-]?(?:[0-9]+(?:\.[0-9]*)?|\.[0-9]+)mm\Z")


class MapError(ValueError):
    """An invalid map, frame mismatch, or board coverage error."""


_MISSING = object()


def natural_key(value):
    """The Rust ``designator_sort_key``: uppercase prefix, then number."""
    value = str(value)
    prefix_len = 0
    while prefix_len < len(value) and "A" <= value[prefix_len] <= "Z":
        prefix_len += 1
    suffix = value[prefix_len:]
    number = int(suffix) if suffix.isdigit() and suffix.isascii() else 0
    return value[:prefix_len], number


def _field(record, name, default=_MISSING):
    if isinstance(record, dict):
        if name in record:
            return record[name]
    elif hasattr(record, name):
        return getattr(record, name)
    if default is not _MISSING:
        return default
    raise MapError("missing field: " + name)


def _pair(value):
    if isinstance(value, dict):
        if "x" in value and "y" in value:
            return value["x"], value["y"]
        raise MapError("expected an (x, y) pair")
    if isinstance(value, (list, tuple)) and len(value) == 2:
        return value
    if hasattr(value, "x") and hasattr(value, "y"):
        return value.x, value.y
    raise MapError("expected an (x, y) pair")


def parse_mm_nm(value):
    """Parse a decimal ``mm`` string to integer nm, half away from zero.

    Decimal parses and scales the text directly: no float is involved.  A
    sub-nanometre decimal (including the RFC's +/-0.0000004mm) is rounded;
    this also gives exact, symmetric behaviour at a half-nanometre boundary.
    """
    if not isinstance(value, str) or not _MM_RE.fullmatch(value):
        raise MapError("expected a decimal mm string, got {!r}".format(value))
    try:
        nm = Decimal(value[:-2]) * _NM_PER_MM
        return int(nm.to_integral_value(rounding=ROUND_HALF_UP))
    except (InvalidOperation, ValueError, OverflowError) as exc:
        raise MapError("invalid decimal mm string: " + value) from exc


def round_nm(value):
    """Round a trigonometric result to nm, with halves away from zero."""
    if not math.isfinite(value):
        raise MapError("non-finite board coordinate")
    if value >= 0:
        return int(math.floor(value + 0.5))
    return -int(math.floor(-value + 0.5))


def _board_xy(value, label):
    x, y = _pair(value)
    if isinstance(x, bool) or isinstance(y, bool) or not isinstance(x, (int, float)):
        raise MapError(label + " coordinates must be integer nm")
    if isinstance(x, float) and (not math.isfinite(x) or not x.is_integer()):
        raise MapError(label + " coordinates must be integer nm")
    if isinstance(y, float) and (not math.isfinite(y) or not y.is_integer()):
        raise MapError(label + " coordinates must be integer nm")
    return int(x), int(y)


def recover_local_nm(pad_xy_nm, anchor_xy_nm, orientation_degrees, bottom=False):
    """Recover a front-side footprint-local pad anchor in integer nm.

    Undo the KiCad footprint rotation, then undo the bottom-side y reflection.
    Both flip directions have the same local-pad representation in pcbnew, so
    no flip-direction flag is needed here.
    """
    px, py = _board_xy(pad_xy_nm, "pad")
    ax, ay = _board_xy(anchor_xy_nm, "anchor")
    if isinstance(orientation_degrees, bool) or not isinstance(
        orientation_degrees, (int, float)
    ):
        raise MapError("footprint orientation must be a finite number")
    if not math.isfinite(orientation_degrees):
        raise MapError("footprint orientation must be a finite number")
    # KiCad RotatePoint in the +y-down frame is
    # (x*cos+y*sin, y*cos-x*sin).  These formulas are its inverse, with the
    # footprint's reported (positive) orientation, not a negated angle.
    radians = math.radians(orientation_degrees)
    cosine, sine = math.cos(radians), math.sin(radians)
    dx, dy = px - ax, py - ay
    x = round_nm(dx * cosine - dy * sine)
    y = round_nm(dx * sine + dy * cosine)
    return x, -y if bottom else y


def recover_pads_nm(pads, anchor_xy_nm, orientation_degrees, bottom=False):
    """Filter to numbered pads and recover their local anchors."""
    result = []
    for index, pad in enumerate(pads):
        number = _field(pad, "number", "")
        if number is None or not str(number):
            continue
        if isinstance(number, bool) or not isinstance(number, (str, int)):
            raise MapError("board pad number must be a string or integer")
        xy = _field(pad, "xy_nm", None)
        if xy is None:
            for key in ("position_nm", "xy", "at", "position"):
                xy = _field(pad, key, None)
                if xy is not None:
                    break
        if xy is None:
            xy = _pair(pad)
        result.append(
            (
                str(number),
                recover_local_nm(
                    xy, anchor_xy_nm, orientation_degrees, bottom=bottom
                ),
                index,
            )
        )
    return result


def _pad_number(value):
    if isinstance(value, bool) or not isinstance(value, (str, int)):
        raise MapError("map pad number must be a string or integer")
    text = str(value)
    if not text:
        raise MapError("map pad number must not be empty")
    return text


def parse_map_pads(pads):
    """Parse the RFC ``[number, x-mm, y-mm]`` projection list."""
    if not isinstance(pads, list):
        raise MapError("footprint pads must be a list")
    parsed = []
    for index, pad in enumerate(pads):
        if not isinstance(pad, (list, tuple)) or len(pad) != 3:
            raise MapError("map pad {} must be [number, x, y]".format(index))
        parsed.append(
            (
                _pad_number(pad[0]),
                (parse_mm_nm(pad[1]), parse_mm_nm(pad[2])),
                index,
            )
        )
    return parsed


def _require_mapping(value, label):
    if not isinstance(value, dict):
        raise MapError(label + " must be an object")
    return value


def _require_text(value, label, empty=False):
    if not isinstance(value, str) or (not empty and not value):
        raise MapError(label + " must be a non-empty string")
    return value


def validate_map(document):
    """Validate and normalize the map structures needed by the matcher.

    Returns ``{"footprints": {name: parsed pads}, "designators": rows,
    "bottom_rotation": "kicad" | "mirrored"}``.
    Rows retain their validated mapping fields.  Package/version and purchase
    identity fields are validated for the future CLI, but are not otherwise
    interpreted by the frame matcher.
    """
    document = _require_mapping(document, "map")
    schema = document.get("schema_version")
    if schema not in (1, 2) or isinstance(schema, bool):
        raise MapError("map schema_version must be 1 or 2")
    if schema == 2 and "bottom_rotation" not in document:
        raise MapError("schema 2 map requires bottom_rotation")
    bottom_rotation = document["bottom_rotation"] if schema == 2 else "kicad"
    if bottom_rotation not in ("kicad", "mirrored"):
        raise MapError("map bottom_rotation must be kicad or mirrored")
    if document.get("pad_projection") != 1 or isinstance(
        document.get("pad_projection"), bool
    ):
        raise MapError("map pad_projection must be 1")
    footprints = _require_mapping(document.get("footprints"), "footprints")
    designators = _require_mapping(document.get("designators"), "designators")

    parsed_footprints = {}
    for name in sorted(footprints, key=natural_key):
        _require_text(name, "footprint name")
        entry = _require_mapping(footprints[name], "footprint " + name)
        _require_text(entry.get("package"), "footprint " + name + " package")
        _require_text(entry.get("version"), "footprint " + name + " version")
        pads = parse_map_pads(entry.get("pads"))
        frame = _require_text(entry.get("frame"), "footprint " + name + " frame")
        if not re.fullmatch(r"[0-9a-f]{64}", frame):
            raise MapError("footprint " + name + " frame must be lowercase SHA-256")
        expected_frame = canonical_frame(entry["pads"])
        if frame != expected_frame:
            raise MapError(
                "footprint {} frame mismatch: declared={}; canonical={}".format(
                    name, frame, expected_frame
                )
            )
        parsed_footprints[name] = pads

    rows = []
    used_footprints = set()
    for ref in sorted(designators, key=natural_key):
        _require_text(ref, "designator")
        entry = _require_mapping(designators[ref], "designator " + ref)
        part = _require_text(entry.get("part"), "designator " + ref + " part")
        footprint = _require_text(
            entry.get("footprint"), "designator " + ref + " footprint"
        )
        selection = entry.get("entry")
        if selection != "primary" and (
            isinstance(selection, bool) or not isinstance(selection, int) or selection < 0
        ):
            raise MapError("designator " + ref + " entry must be primary or an index")
        mfr = entry.get("mfr")
        mpn = entry.get("mpn")
        if not isinstance(mfr, str) or not isinstance(mpn, str):
            raise MapError("designator " + ref + " mfr/mpn must be strings")
        pn = entry.get("pn")
        if pn is not None and (not isinstance(pn, str) or not pn.strip()):
            raise MapError("designator " + ref + " pn must be null or non-empty")
        at = entry.get("at")
        if not isinstance(at, (list, tuple)) or len(at) != 2:
            raise MapError("designator " + ref + " at must be [x, y]")
        at_nm = (parse_mm_nm(at[0]), parse_mm_nm(at[1]))
        rotate = entry.get("rotate")
        if isinstance(rotate, bool) or not isinstance(rotate, int) or not 0 <= rotate <= 359:
            raise MapError("designator " + ref + " rotate must be in 0..359")
        if footprint not in parsed_footprints:
            raise MapError(
                "designator " + ref + " references footprint not in footprints: " + footprint
            )
        used_footprints.add(footprint)
        rows.append(
            {
                "designator": ref,
                "part": part,
                "footprint": footprint,
                "entry": selection,
                "mfr": mfr,
                "mpn": mpn,
                "pn": pn,
                "at": at_nm,
                "rotate": rotate,
                "unbound": pn is None,
            }
        )
    unused = sorted(set(parsed_footprints) - used_footprints, key=natural_key)
    if unused:
        raise MapError("footprints contains unused footprint: " + unused[0])
    return {
        "footprints": parsed_footprints,
        "designators": rows,
        "bottom_rotation": bottom_rotation,
    }


def canonical_frame(pads):
    """Return the RFC frame hash for a validated map pad list.

    Canonical serialization uses the original decimal strings, not rounded nm.
    """
    # Matching uses rounded integer nm, but Rust orders the frame by exact
    # femto-mm values.  Decimal preserves the original strings for hashing.
    parse_map_pads(pads)
    exact = [
        (_pad_number(pad[0]), Decimal(pad[1][:-2]), Decimal(pad[2][:-2]))
        for pad in pads
    ]
    order = sorted(range(len(pads)), key=lambda i: exact[i])
    canonical = [
        [_pad_number(pads[i][0]), pads[i][1], pads[i][2]] for i in order
    ]
    encoded = json.dumps(canonical, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def _within(a, b, tolerance_nm):
    return abs(a[0] - b[0]) <= tolerance_nm and abs(a[1] - b[1]) <= tolerance_nm


def perfect_matching(board_positions, map_positions, tolerance_nm=TOLERANCE_NM):
    """Return map-index pairs, or ``None`` if no perfect bipartite matching exists.

    Hopcroft--Karp is used instead of sorting or nearest-first pairing.  Edge
    traversal is by original list index, making the selected matching stable.
    """
    if len(board_positions) != len(map_positions):
        return None
    if not board_positions:
        return []
    adjacency = [
        [
            j
            for j, mapped in enumerate(map_positions)
            if _within(board, mapped, tolerance_nm)
        ]
        for board in board_positions
    ]
    n = len(board_positions)
    board_match = [-1] * n
    map_match = [-1] * n

    while True:
        distance = [-1] * n
        queue = deque()
        for i, matched in enumerate(board_match):
            if matched < 0:
                distance[i] = 0
                queue.append(i)
        augmenting = False
        shortest = None
        while queue:
            i = queue.popleft()
            if shortest is not None and distance[i] >= shortest:
                continue
            for j in adjacency[i]:
                other = map_match[j]
                if other < 0:
                    augmenting = True
                    shortest = distance[i] + 1
                elif distance[other] < 0:
                    distance[other] = distance[i] + 1
                    queue.append(other)
        if not augmenting:
            return None

        def augment(i):
            for j in adjacency[i]:
                other = map_match[j]
                # Only free right vertices at the shortest NIL distance and
                # matched vertices in the next BFS layer belong to this phase's
                # layered graph.  Never mark a wrong-layer edge as visited.
                if other < 0 and distance[i] + 1 == shortest:
                    board_match[i] = j
                    map_match[j] = i
                    return True
                if (
                    other >= 0
                    and distance[other] == distance[i] + 1
                    and augment(other)
                ):
                    board_match[i] = j
                    map_match[j] = i
                    return True
            distance[i] = -1
            return False

        progress = False
        for i, matched in enumerate(board_match):
            if matched < 0 and augment(i):
                progress = True
        if not progress:
            return None
        if all(matched >= 0 for matched in board_match):
            return [(i, board_match[i]) for i in range(n)]


def match_pads(board_pads, map_pads, tolerance_nm=TOLERANCE_NM):
    """Match parsed pad records per number; return ``(ok, lowest failing number)``."""
    board_groups = {}
    map_groups = {}
    for number, position, _ in board_pads:
        board_groups.setdefault(number, []).append(position)
    for number, position, _ in map_pads:
        map_groups.setdefault(number, []).append(position)
    for number in sorted(set(board_groups) | set(map_groups), key=natural_key):
        board = board_groups.get(number, [])
        mapped = map_groups.get(number, [])
        if perfect_matching(board, mapped, tolerance_nm) is None:
            return False, number
    return True, None


def _positions(group):
    return sorted((position for _, position, _ in group))


def format_nm(position):
    return "({}nm,{}nm)".format(position[0], position[1])


def frame_mismatch(designator, number, board_group, map_group):
    """The stable diagnostic required by RFC-034 section 3.2."""
    return (
        "{}: frame mismatch or older pad_projection at pad {}; board={}; map={}".format(
            designator,
            number,
            "[" + ", ".join(format_nm(p) for p in _positions(board_group)) + "]",
            "[" + ", ".join(format_nm(p) for p in _positions(map_group)) + "]",
        )
    )


def check_frame(designator, board_pads, map_pads, tolerance_nm=TOLERANCE_NM):
    """Return ``None`` on success, otherwise the deterministic mismatch message."""
    board_groups = {}
    map_groups = {}
    for number, position, _ in board_pads:
        board_groups.setdefault(number, []).append((number, position, 0))
    for number, position, _ in map_pads:
        map_groups.setdefault(number, []).append((number, position, 0))
    for number in sorted(set(board_groups) | set(map_groups), key=natural_key):
        board = board_groups.get(number, [])
        mapped = map_groups.get(number, [])
        if perfect_matching(
            [item[1] for item in board], [item[1] for item in mapped], tolerance_nm
        ) is None:
            return frame_mismatch(designator, number, board, mapped)
    return None


def identity_mapping(entry):
    """Identity placement for an explicitly unbound (``pn: null``) entry."""
    if entry.get("pn") is not None:
        raise MapError("identity mapping is only for an unbound designator")
    return {"at": (0, 0), "rotate": 0, "unbound": True}


def check_coverage(board_designators, map_designators, machine_placed=None):
    """Return deterministic ``(errors, warnings)`` for map/board coverage.

    ``map_designators`` is the validated row list (or a mapping).  By default
    every supplied board designator is machine placed; the optional filter is
    supplied by the neutral-row adapter in 4b-2.  Unbound rows use identity
    mapping and are not coverage errors.
    """
    board = set(board_designators)
    if isinstance(map_designators, dict):
        mapped = set(map_designators)
    else:
        mapped = {
            row["designator"] if isinstance(row, dict) else str(row)
            for row in map_designators
        }
    machine = board if machine_placed is None else set(machine_placed)
    errors = [
        ref + ": machine-placed footprint not in this map"
        for ref in sorted(machine - mapped, key=natural_key)
    ]
    warnings = [
        ref + ": map designator missing from board"
        for ref in sorted(mapped - board, key=natural_key)
    ]
    return errors, warnings


def check_board(document, board_footprints):
    """Check all frame and coverage errors before a caller writes any output.

    ``board_footprints`` is an iterable of records with ``designator``,
    ``footprint``, ``pads``, ``anchor_xy_nm``, ``orientation`` and optional
    ``bottom``/``machine_placed`` fields.  Missing map designators are returned
    as warnings.  No board or map record is mutated.
    """
    validated = validate_map(document)
    footprints = validated["footprints"]
    rows = validated["designators"]
    by_ref = {row["designator"]: row for row in rows}
    board = {}
    for fp in board_footprints:
        ref = _require_text(_field(fp, "designator"), "board designator")
        if ref in board:
            raise MapError(ref + ": duplicate board designator")
        board[ref] = fp

    errors, warnings = check_coverage(
        board,
        rows,
        machine_placed=[
            ref
            for ref, fp in board.items()
            if _field(fp, "machine_placed", True)
        ],
    )
    for ref in sorted(board, key=natural_key):
        fp = board[ref]
        if ref not in by_ref or not _field(fp, "machine_placed", True):
            continue
        row = by_ref[ref]
        frame = _field(fp, "footprint")
        if row["footprint"] not in footprints:
            # Kept as a defensive invariant: validate_map already checks it.
            errors.append(ref + ": footprint frame not in this map: " + str(frame))
            continue
        # kicad_pcb's file_base() is exactly fq.replace("::", "-").
        board_names = {row["footprint"], row["footprint"].replace("::", "-")}
        if frame not in footprints and frame not in board_names:
            errors.append(ref + ": footprint frame not in this map: " + str(frame))
            continue
        expected_frame = row["footprint"]
        # A short-name or fully-qualified-name disagreement is not itself a
        # frame mismatch; compare the actual projected pad frames below.
        bottom = _field(fp, "bottom", _field(fp, "flipped", False))
        anchor = _field(fp, "anchor_xy_nm", None)
        if anchor is None:
            anchor = _field(fp, "anchor", None)
        if anchor is None:
            anchor = _field(fp, "at")
        recovered = recover_pads_nm(
            _field(fp, "pads"),
            anchor,
            _field(fp, "orientation", _field(fp, "rotation", 0)),
            bottom=bottom,
        )
        mismatch = check_frame(ref, recovered, footprints[expected_frame])
        if mismatch:
            errors.append(mismatch)
    return errors, warnings, rows


def mapped_neutral_rows(board_footprints, mappings, origin_nm, bottom_rotation="kicad"):
    """Apply validated supplier mappings to board anchors, in integer nm."""
    by_ref = {row["designator"]: row for row in mappings}
    rows = []
    for fp in board_footprints:
        ref = _field(fp, "ref")
        if ref not in by_ref or not any(
            pad["smd"] and pad["number"] for pad in _field(fp, "pads")
        ):
            continue
        row = by_ref[ref]
        mapping = identity_mapping(row) if row["unbound"] else row
        dx, dy = mapping["at"]
        bottom = _field(fp, "flipped")
        angle = _field(fp, "orientation") - 180 if bottom else _field(fp, "orientation")
        radians = math.radians(angle)
        cosine, sine = math.cos(radians), math.sin(radians)
        if bottom:
            dx = -dx
        anchor = _field(fp, "anchor")
        mapped_fp = dict(
            fp,
            anchor=(
                anchor[0] + round_nm(dx * cosine + dy * sine),
                anchor[1] + round_nm(dy * cosine - dx * sine),
            ),
            orientation=_field(fp, "orientation")
            + (-mapping["rotate"] if bottom else mapping["rotate"]),
        )
        mapped_rows, _ = neutral_rows([mapped_fp], origin_nm)
        if bottom and bottom_rotation == "mirrored":
            ref, x, y, layer, angle = mapped_rows[0]
            mapped_rows = [(ref, x, y, layer, (180 - angle) % 360)]
        rows.extend(mapped_rows)
    return rows