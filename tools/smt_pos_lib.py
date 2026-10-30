"""Pure placement calculations for :mod:`smt_pos`.

Footprint records are dictionaries with ``ref``, ``value``, ``anchor`` (an
integer-nanometre ``(x, y)`` pair), ``orientation``, ``flipped`` and ``pads``.
Pad records have ``number``, ``position`` (also integer nanometres), ``smd``
and ``copper`` booleans.  This module never imports KiCad.

The legacy path deliberately retains its historical floating-point formatting
and filter heuristics.  Neutral coordinates stay in integer nanometres until
fixed-point formatting, so ties round half away from zero without float error.
"""

import math
import re


BARE_COPPER_VALUES = {"TouchSensor", "TouchPad"}
LEGACY_HEADER = (
    "Designator,Footprint,Mid X,Mid Y,Ref X,Ref Y,Pad X,Pad Y,"
    "Layer,Rotation,Comment"
)
NEUTRAL_HEADER = "Designator,Mid X,Mid Y,Layer,Rotation"


def legacy_origin(left_mm, bottom_mm):
    """Return the legacy lower-left outline origin in millimetres."""
    return left_mm, bottom_mm


def neutral_origin(outline_bounds_nm):
    """Return ``(x_min, y_max)``, or the authoring origin without an outline."""
    if outline_bounds_nm is None:
        return 0, 0
    return outline_bounds_nm[0], outline_bounds_nm[3]


def _include_point(bounds, point):
    x, y = point
    if bounds is None:
        return [x, y, x, y]
    bounds[0] = min(bounds[0], x)
    bounds[1] = min(bounds[1], y)
    bounds[2] = max(bounds[2], x)
    bounds[3] = max(bounds[3], y)
    return bounds


def _arc_delta(start, end, clockwise):
    return (start - end if clockwise else end - start) % (2 * math.pi)


def _arc_bounds(start, mid, end, center):
    """Bounds of the circular centreline through start/mid/end.

    KiCad's mid point disambiguates the arc and its direction. Cardinal
    extrema are included only when the directed arc actually crosses them.
    """
    a = math.atan2(start[1] - center[1], start[0] - center[0])
    b = math.atan2(mid[1] - center[1], mid[0] - center[0])
    c = math.atan2(end[1] - center[1], end[0] - center[0])
    first = _arc_delta(a, b, clockwise=True)
    second = _arc_delta(b, c, clockwise=True)
    clockwise = first + second < 2 * math.pi
    sweep = _arc_delta(a, c, clockwise)
    # KiCad stores integer nm; round only a newly computed cardinal extremum.
    radius = round(math.hypot(start[0] - center[0], start[1] - center[1]))
    bounds = None
    for point in (start, end):
        bounds = _include_point(bounds, point)
    for angle, point in (
        (0.0, (center[0] + radius, center[1])),
        (math.pi / 2, (center[0], center[1] + radius)),
        (math.pi, (center[0] - radius, center[1])),
        (3 * math.pi / 2, (center[0], center[1] - radius)),
    ):
        travelled = (a - angle if clockwise else angle - a) % (2 * math.pi)
        if travelled <= sweep:
            bounds = _include_point(bounds, point)
    return bounds


def shape_geometry_bounds(shape):
    """Return one neutral outline shape's bounds without its stroke width."""
    kind = shape["shape"]
    if kind == "arc":
        return _arc_bounds(
            shape["start"], shape["mid"], shape["end"], shape["center"]
        )
    if kind == "segment":
        bounds = _include_point(None, shape["start"])
        return _include_point(bounds, shape["end"])
    if kind == "rect":
        start, end = shape["start"], shape["end"]
        return [
            min(start[0], end[0]),
            min(start[1], end[1]),
            max(start[0], end[0]),
            max(start[1], end[1]),
        ]
    if kind == "circle":
        center, end = shape["center"], shape["end"]
        radius = math.hypot(end[0] - center[0], end[1] - center[1])
        return [
            math.floor(center[0] - radius),
            math.floor(center[1] - radius),
            math.ceil(center[0] + radius),
            math.ceil(center[1] + radius),
        ]
    if kind == "poly":
        bounds = None
        for point in shape["points"]:
            bounds = _include_point(bounds, point)
        if bounds is not None:
            return bounds
    # Compatibility fallback for other PCB_SHAPE types: undo KiCad's
    # half-width inflation on this shape only, never on the whole board.
    left, top, right, bottom = shape["bbox"]
    half_width = shape["width"] / 2
    return [
        math.ceil(left + half_width),
        math.ceil(top + half_width),
        math.floor(right - half_width),
        math.floor(bottom - half_width),
    ]


def outline_geometry_bounds(shapes):
    """Combine exact centreline bounds; ``None`` means no Edge.Cuts shape."""
    bounds = None
    for shape in shapes:
        current = shape_geometry_bounds(shape)
        if bounds is None:
            bounds = current
        else:
            bounds = [
                min(bounds[0], current[0]),
                min(bounds[1], current[1]),
                max(bounds[2], current[2]),
                max(bounds[3], current[3]),
            ]
    return bounds


def legacy_project(x_mm, y_mm, origin_mm):
    """Project a legacy point around the lower-left origin, with Y upward."""
    return x_mm - origin_mm[0], origin_mm[1] - y_mm


def neutral_project(anchor_nm, origin_nm):
    """Project a footprint anchor using exact integer-nanometre arithmetic."""
    return anchor_nm[0] - origin_nm[0], origin_nm[1] - anchor_nm[1]


def normalize_rotation(angle):
    """Normalize a KiCad footprint angle to ``[0, 360)``."""
    return angle % 360


def natural_key(text):
    """A total natural-order key, including names with multiple digit runs."""
    return tuple(
        (1, int(part)) if part.isdigit() else (0, part)
        for part in re.split(r"(\d+)", text)
    )


def has_smd(footprint):
    """The legacy inclusion test: at least one SMD pad, numbered or not."""
    return any(pad["smd"] for pad in footprint["pads"])


def has_numbered_smd(footprint):
    """The RFC-034 machine-placement test on the KiCad pad projection."""
    return any(
        pad["smd"] and pad["number"] != "" for pad in footprint["pads"]
    )


def legacy_skip_reason(footprint, everything=False):
    """Return the historical skip diagnostic, or ``None`` for an included row."""
    if everything:
        return None
    if not has_smd(footprint):
        return "no SMD pads (through-hole/mechanical)"
    if (
        footprint["value"] in BARE_COPPER_VALUES
        or footprint["ref"].startswith("TP")
    ):
        return "bare copper feature, no physical part"
    return None


def pad_number_key(pad):
    """Numeric pad names precede other names, as in the original exporter."""
    number = pad["number"]
    return (0, int(number)) if number.isdigit() else (1, number)


def legacy_rows(footprints, origin_mm, to_mm, everything=False):
    """Build legacy rows while preserving the original arithmetic order.

    ``to_mm`` is the KiCad adapter's conversion function.  In particular, pad
    coordinates are averaged in board units *before* conversion, exactly as in
    the original script.
    """
    rows = []
    skipped = []
    for footprint in footprints:
        reason = legacy_skip_reason(footprint, everything)
        if reason is not None:
            skipped.append((footprint["ref"], reason))
            continue

        anchor = footprint["anchor"]
        pads = footprint["pads"]
        copper = [pad for pad in pads if pad["copper"]]
        if copper:
            cx = sum(pad["position"][0] for pad in copper) / len(copper)
            cy = sum(pad["position"][1] for pad in copper) / len(copper)
        else:
            cx, cy = anchor
        named = sorted(
            (pad for pad in pads if pad["number"]), key=pad_number_key
        )
        pad1 = named[0]["position"] if named else anchor
        mid_x, mid_y = legacy_project(to_mm(cx), to_mm(cy), origin_mm)
        ref_x, ref_y = legacy_project(
            to_mm(anchor[0]), to_mm(anchor[1]), origin_mm
        )
        pad_x, pad_y = legacy_project(
            to_mm(pad1[0]), to_mm(pad1[1]), origin_mm
        )
        rows.append(
            (
                footprint["ref"],
                footprint["footprint"],
                mid_x,
                mid_y,
                ref_x,
                ref_y,
                pad_x,
                pad_y,
                "B" if footprint["flipped"] else "T",
                normalize_rotation(footprint["orientation"]),
                footprint["value"],
            )
        )
    return rows, skipped


def neutral_rows(footprints, origin_nm):
    """Build RFC-034 rows from footprint anchors, with no legacy exclusions."""
    rows = []
    skipped = []
    for footprint in footprints:
        if not has_numbered_smd(footprint):
            skipped.append((footprint["ref"], "no numbered SMD pads"))
            continue
        x_nm, y_nm = neutral_project(footprint["anchor"], origin_nm)
        rows.append(
            (
                footprint["ref"],
                x_nm,
                y_nm,
                "Bottom" if footprint["flipped"] else "Top",
                normalize_rotation(footprint["orientation"]),
            )
        )
    return rows, skipped


def legacy_mm(value):
    """Historical variable-precision millimetre formatting (byte-compatible)."""
    return f"{value:.4f}".rstrip("0").rstrip(".") + "mm"


def neutral_mm(value_nm):
    """Format integer nm as four-place mm, with ties rounded away from zero."""
    sign = "-" if value_nm < 0 else ""
    places, remainder = divmod(abs(value_nm), 100)
    if remainder >= 50:
        places += 1
    if places == 0:
        sign = ""
    whole, fraction = divmod(places, 10000)
    return f"{sign}{whole}.{fraction:04d}mm"


def format_rotation(angle):
    return f"{normalize_rotation(angle):g}"


def legacy_csv(rows):
    """Serialize the legacy eleven columns byte-for-byte as before."""
    def refkey(row):
        match = re.match(r"([A-Za-z]+)(\d+)", row[0])
        return (
            (match.group(1), int(match.group(2)))
            if match
            else (row[0], 0)
        )

    lines = [LEGACY_HEADER]
    for row in sorted(rows, key=refkey):
        lines.append(
            f"{row[0]},{row[1]},{legacy_mm(row[2])},{legacy_mm(row[3])},"
            f"{legacy_mm(row[4])},{legacy_mm(row[5])},"
            f"{legacy_mm(row[6])},{legacy_mm(row[7])},{row[8]},"
            f"{format_rotation(row[9])},{row[10]}"
        )
    return "\n".join(lines) + "\n"


def neutral_csv(rows):
    """Serialize the five RFC-034 columns in stable designator natural order."""
    lines = [NEUTRAL_HEADER]
    for row in sorted(rows, key=lambda item: natural_key(item[0])):
        lines.append(
            f"{row[0]},{neutral_mm(row[1])},{neutral_mm(row[2])},"
            f"{row[3]},{format_rotation(row[4])}"
        )
    return "\n".join(lines) + "\n"
