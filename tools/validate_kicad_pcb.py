#!/usr/bin/env python3
"""Semantic diff for CoHDL's native board and the pcbnew-built reference.

Run this with KiCad's bundled Python, after building the reference board with
``tools/kicad_board.py``::

    "$KPY" tools/validate_kicad_pcb.py \
        out/design.kicad_pcb out/design.pcbnew-reference.kicad_pcb

The two files are deliberately not byte-comparable: pcbnew assigns random
UUIDs, numeric net codes, and its own serialization order.  This tool instead
loads both through KiCad's official pcbnew API and compares the board facts
that matter: footprint placement/side/orientation, every pad copy and net,
footprint fields and graphics, and the Edge.Cuts geometry.  Coordinates permit
small nanometer-scale differences because the retained Python reference goes
through float millimeters while the native emitter keeps exact femto-mm values.
"""

import argparse
import math
import sys
from pathlib import Path

try:
    import pcbnew
except ImportError:
    print(
        "error: pcbnew is unavailable; run with KiCad's bundled Python",
        file=sys.stderr,
    )
    raise SystemExit(2)


class Comparison:
    def __init__(self, tolerance_nm):
        self.tolerance_nm = tolerance_nm
        self.errors = []
        self.max_coordinate_delta_nm = 0
        self.max_angle_delta_deg = 0.0

    def exact(self, label, native, reference):
        if native != reference:
            self.errors.append(f"{label}: native={native!r}, pcbnew={reference!r}")

    def scalar(self, label, native, reference, tolerance=1e-9):
        if native is None or reference is None:
            self.exact(label, native, reference)
        elif not math.isclose(native, reference, rel_tol=0.0, abs_tol=tolerance):
            self.errors.append(f"{label}: native={native!r}, pcbnew={reference!r}")

    def coordinate(self, label, native, reference):
        if len(native) != len(reference):
            self.errors.append(
                f"{label}: coordinate lengths differ "
                f"(native={len(native)}, pcbnew={len(reference)})"
            )
            return
        deltas = [abs(a - b) for a, b in zip(native, reference)]
        delta_nm = max(deltas, default=0)
        self.max_coordinate_delta_nm = max(self.max_coordinate_delta_nm, delta_nm)
        if delta_nm > self.tolerance_nm:
            self.errors.append(
                f"{label}: native={native!r}, pcbnew={reference!r} "
                f"(max delta {delta_nm} nm)"
            )

    def angle(self, label, native, reference, tolerance_deg=1e-7):
        delta = abs((native - reference + 180.0) % 360.0 - 180.0)
        self.max_angle_delta_deg = max(self.max_angle_delta_deg, delta)
        if delta > tolerance_deg:
            self.errors.append(
                f"{label}: native={native:g}°, pcbnew={reference:g}° "
                f"(delta {delta:g}°)"
            )


def xy(value):
    """pcbnew VECTOR2I -> integer nanometer pair."""
    return (value.x, value.y)


def optional_number(value):
    return None if value is None else float(value)


def pad_snapshot(pad):
    return {
        "number": str(pad.GetNumber()),
        "net": str(pad.GetNetname()),
        "position": xy(pad.GetPosition()),
        "relative_position": xy(pad.GetFPRelativePosition()),
        "orientation": pad.GetOrientationDegrees() % 360.0,
        "relative_orientation": pad.GetFPRelativeOrientation().AsDegrees() % 360.0,
        "size": xy(pad.GetSize()),
        "drill": xy(pad.GetDrillSize()),
        "drill_shape": int(pad.GetDrillShape()),
        "shape": int(pad.GetShape()),
        "attribute": int(pad.GetAttribute()),
        "layers": pad.GetLayerSet().FmtHex(),
        "chamfer_positions": int(pad.GetChamferPositions()),
        "chamfer_ratio": float(pad.GetChamferRectRatio()),
        "roundrect_ratio": float(pad.GetRoundRectRadiusRatio()),
        "mask_margin": optional_number(pad.GetLocalSolderMaskMargin()),
        "paste_margin": optional_number(pad.GetLocalSolderPasteMargin()),
        "paste_margin_ratio": optional_number(pad.GetLocalSolderPasteMarginRatio()),
    }


def pad_sort_key(pad):
    # Pair duplicate-number pads deterministically without relying on pcbnew's
    # container iteration order.  Position is intentionally last: electrical
    # and shape identity wins before geometry when diagnosing a mismatch.
    return (
        pad["number"],
        pad["net"],
        pad["attribute"],
        pad["shape"],
        pad["layers"],
        pad["size"],
        pad["drill"],
        pad["position"],
        pad["orientation"],
    )


def field_snapshot(field):
    return {
        "text": str(field.GetText()),
        "position": xy(field.GetPosition()),
        "relative_position": xy(field.GetFPRelativePosition()),
        "angle": field.GetTextAngleDegrees() % 360.0,
        "layer": int(field.GetLayer()),
        "mirrored": bool(field.IsMirrored()),
        "visible": bool(field.IsVisible()),
        "size": xy(field.GetTextSize()),
        "thickness": int(field.GetTextThickness()),
    }


def shape_snapshot(shape):
    result = {
        "kind": shape.GetShapeStr(),
        "layer": int(shape.GetLayer()),
        "width": int(shape.GetWidth()),
        "fill": int(shape.GetFillMode()),
        "start": xy(shape.GetStart()),
        "end": xy(shape.GetEnd()),
        "center": xy(shape.GetCenter()),
        "points": tuple(xy(p) for p in shape.GetPolyPoints()),
    }
    if result["kind"] == "Arc":
        result["mid"] = xy(shape.GetArcMid())
        result["sweep"] = abs(shape.GetArcAngle().AsDegrees())
    return result


def shape_sort_key(shape):
    return (
        shape["kind"],
        shape["layer"],
        min(shape["start"], shape["end"]),
        max(shape["start"], shape["end"]),
        shape["center"],
    )


def compare_mapping(comparison, label, native, reference, coordinate_keys, angle_keys=()):
    comparison.exact(f"{label}.keys", set(native), set(reference))
    for key in sorted(set(native) & set(reference)):
        if key in coordinate_keys:
            comparison.coordinate(f"{label}.{key}", native[key], reference[key])
        elif key in angle_keys:
            # pcbnew reconstructs an arc centre through float millimeters, so
            # its derived sweep can differ by a few ten-thousandths of a
            # degree even when all defining coordinates agree to nanometers.
            tolerance = 0.001 if key == "sweep" else 1e-7
            comparison.angle(
                f"{label}.{key}", native[key], reference[key], tolerance
            )
        elif key == "points":
            comparison.exact(
                f"{label}.points.count", len(native[key]), len(reference[key])
            )
            for index, (native_point, reference_point) in enumerate(
                zip(native[key], reference[key])
            ):
                comparison.coordinate(
                    f"{label}.points[{index}]", native_point, reference_point
                )
        elif isinstance(native[key], float) or isinstance(reference[key], float):
            comparison.scalar(f"{label}.{key}", native[key], reference[key])
        else:
            comparison.exact(f"{label}.{key}", native[key], reference[key])


def compare_footprint(comparison, ref, native, reference):
    prefix = f"footprint[{ref}]"
    comparison.exact(
        f"{prefix}.value", str(native.GetValue()), str(reference.GetValue())
    )
    comparison.exact(
        f"{prefix}.name",
        str(native.GetFPIDAsString()),
        str(reference.GetFPIDAsString()),
    )
    comparison.exact(f"{prefix}.layer", int(native.GetLayer()), int(reference.GetLayer()))
    comparison.exact(
        f"{prefix}.attributes", int(native.GetAttributes()), int(reference.GetAttributes())
    )
    comparison.coordinate(f"{prefix}.position", xy(native.GetPosition()), xy(reference.GetPosition()))
    comparison.angle(
        f"{prefix}.orientation",
        native.GetOrientationDegrees() % 360.0,
        reference.GetOrientationDegrees() % 360.0,
    )

    native_pads = sorted((pad_snapshot(p) for p in native.Pads()), key=pad_sort_key)
    reference_pads = sorted((pad_snapshot(p) for p in reference.Pads()), key=pad_sort_key)
    comparison.exact(f"{prefix}.pad_count", len(native_pads), len(reference_pads))
    for index, (native_pad, reference_pad) in enumerate(zip(native_pads, reference_pads)):
        number = native_pad["number"] or "<mechanical>"
        compare_mapping(
            comparison,
            f"{prefix}.pad[{index}:{number}]",
            native_pad,
            reference_pad,
            {"position", "relative_position", "size", "drill"},
            {"orientation", "relative_orientation"},
        )

    native_fields = {
        str(field.GetName()): field_snapshot(field) for field in native.GetFields()
    }
    reference_fields = {
        str(field.GetName()): field_snapshot(field) for field in reference.GetFields()
    }
    comparison.exact(f"{prefix}.field_names", set(native_fields), set(reference_fields))
    for name in sorted(set(native_fields) & set(reference_fields)):
        compare_mapping(
            comparison,
            f"{prefix}.field[{name}]",
            native_fields[name],
            reference_fields[name],
            {"position", "relative_position", "size"},
            {"angle"},
        )

    native_shapes = sorted(
        (shape_snapshot(s) for s in native.GraphicalItems()), key=shape_sort_key
    )
    reference_shapes = sorted(
        (shape_snapshot(s) for s in reference.GraphicalItems()), key=shape_sort_key
    )
    comparison.exact(f"{prefix}.graphic_count", len(native_shapes), len(reference_shapes))
    for index, (native_shape, reference_shape) in enumerate(
        zip(native_shapes, reference_shapes)
    ):
        compare_mapping(
            comparison,
            f"{prefix}.graphic[{index}]",
            native_shape,
            reference_shape,
            {"start", "end", "center", "mid"},
            {"sweep"},
        )


def edge_snapshot(board):
    return sorted(
        (
            shape_snapshot(shape)
            for shape in board.GetDrawings()
            if shape.GetLayer() == pcbnew.Edge_Cuts
        ),
        key=shape_sort_key,
    )


def inventory(board):
    footprints = list(board.GetFootprints())
    return {
        "footprints": len(footprints),
        "bottom": sum(fp.GetLayer() == pcbnew.B_Cu for fp in footprints),
        "pads": sum(len(list(fp.Pads())) for fp in footprints),
        "nets": len([name for name in board.GetNetsByName().keys() if str(name)]),
        "edge_cuts": len(edge_snapshot(board)),
        "tracks": len(list(board.GetTracks())),
        "zones": len(list(board.Zones())),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("native", type=Path)
    parser.add_argument("reference", type=Path)
    parser.add_argument(
        "--tolerance-nm",
        type=int,
        default=5,
        help="maximum coordinate delta accepted (default: 5 nm)",
    )
    args = parser.parse_args()
    if args.tolerance_nm < 0:
        parser.error("--tolerance-nm must be non-negative")
    for path in (args.native, args.reference):
        if not path.is_file():
            parser.error(f"board does not exist: {path}")

    native = pcbnew.LoadBoard(str(args.native))
    reference = pcbnew.LoadBoard(str(args.reference))
    comparison = Comparison(args.tolerance_nm)

    native_inventory = inventory(native)
    reference_inventory = inventory(reference)
    comparison.exact("inventory", native_inventory, reference_inventory)
    comparison.exact(
        "named_nets",
        {str(name) for name in native.GetNetsByName().keys() if str(name)},
        {str(name) for name in reference.GetNetsByName().keys() if str(name)},
    )

    native_footprints = {str(fp.GetReference()): fp for fp in native.GetFootprints()}
    reference_footprints = {
        str(fp.GetReference()): fp for fp in reference.GetFootprints()
    }
    comparison.exact(
        "footprint_references", set(native_footprints), set(reference_footprints)
    )
    for ref in sorted(set(native_footprints) & set(reference_footprints)):
        compare_footprint(
            comparison, ref, native_footprints[ref], reference_footprints[ref]
        )

    native_edges = edge_snapshot(native)
    reference_edges = edge_snapshot(reference)
    comparison.exact("edge_cuts.count", len(native_edges), len(reference_edges))
    for index, (native_edge, reference_edge) in enumerate(
        zip(native_edges, reference_edges)
    ):
        compare_mapping(
            comparison,
            f"edge_cuts[{index}]",
            native_edge,
            reference_edge,
            {"start", "end", "center", "mid"},
            {"sweep"},
        )

    print(f"KiCad pcbnew: {pcbnew.GetBuildVersion()}")
    print(f"native inventory:   {native_inventory}")
    print(f"pcbnew inventory:   {reference_inventory}")
    print(f"max coordinate delta: {comparison.max_coordinate_delta_nm} nm")
    print(f"max angle delta: {comparison.max_angle_delta_deg:g}°")
    if comparison.errors:
        print(f"RESULT: FAIL — {len(comparison.errors)} semantic difference(s)")
        for error in comparison.errors[:200]:
            print(f"  - {error}")
        if len(comparison.errors) > 200:
            print(f"  ... {len(comparison.errors) - 200} more")
        return 1
    print("RESULT: OK — native board matches the pcbnew-built reference")
    return 0


if __name__ == "__main__":
    sys.exit(main())
