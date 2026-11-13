"""The W3 sampled-inference rule for chip resistors and MLCCs.

Tony accepted the rule on 2026-11-09 for the non-polar chip resistors and MLCCs of
``lib/passive`` only (https://github.com/conol-ai/cohdl/pull/79#issuecomment-6734040188),
and decided on 2026-11-10 that a sample whose model is not drawn does not block its
group. The numbers below are the ones proposed in ``docs/library-supplier-w3-sampling.md``
§5; the sample limits, the zero-stock rule and the not-drawn rule await his
confirmation there. (He accepted them on 2026-11-11,
https://github.com/conol-ai/cohdl/pull/104#issuecomment-6784835520.)

``body_method`` is proposed and awaits his decision: which reading of a chip body
the limits apply to. Until 2026-11-11 there was one, ``measure_preview.py``'s
method 1, which reads a part's own silkscreen reference text together with its
model (``jlc_previews/2026-11-11-board-4/README.md``). Method 2 reads the model
alone. The limits themselves are unchanged from ``w3-2026-11-10b``.

``tests/library/supplier.rs`` holds the same numbers and rejects a group record
whose ``rule`` differs from ``RULE``, so a record cannot loosen its own rule.
"""

from __future__ import annotations

import math
import statistics

RULE = {
    "version": "w3-2026-11-11-m2",
    "package": "passive",
    "types": ["ChipResistor", "MLCC"],
    # At least MIN_SAMPLE members or FRACTION of them, whichever is larger.
    "min_sample": 3,
    "fraction": 0.05,
    # Samples are drawn only from members with this much stock on their confirmed page.
    "min_stock": 30,
    # A sample's body must lie along its pads within this, mod 180.
    "angle_limit_deg": 10.0,
    # ... and be centred within this fraction of the per-part limit as measured, with
    # nothing taken off: the absolute check (the pilot's per-part pass line) ...
    "offset_raw_fraction": 1.0,
    # ... and within this fraction of it once the view's median body offset is taken
    # off (board_bias): the consistency check. Both must hold.
    "offset_residual_fraction": 0.5,
    "offset_bias": "board_median",
    # Which reading of a chip body the limits above apply to: a body-reading method
    # of ``frame_boards/measure_preview.py`` (BODY_METHODS). 2 is the drawn model
    # alone. A sample read any other way is not a sample under this rule, and the
    # median is taken over readings of this method only.
    "body_method": 2,
    # A member with no stock on its confirmed page is not bound by inference.
    "infer_zero_stock": False,
    # A member whose model a preview did not draw is not bound by inference.
    "infer_not_drawn": False,
}

# A record's offsets are rounded to the micrometre; its arithmetic must hold within this.
ROUNDING_MM = 0.001


def required(members: int) -> int:
    return max(RULE["min_sample"], math.ceil(RULE["fraction"] * members))


def board_bias(offsets: list[tuple[float, float]]) -> tuple[float, float]:
    """The median body-centre offset over every drawn chip body of one preview view.

    Hypothesis, not established: that this shift belongs to the export rather than
    to the models. The evidence below is from method 1 readings; read by method 2
    the pilot's median is (+0.011, -0.010) mm and board 3's large CHIP_2010 offset is
    0.031-0.068 mm, so most of what it describes was the parts' reference text
    (``docs/library-supplier-w3-sampling.md`` §6). For it: on the 2026-11-07 pilot the three groups, two makers and
    both part types sat about 0.077 mm off along board -y (Yageo CC 0.061, Yageo RC
    0.079, Vishay CRCW 0.067), with a spread of 0.01-0.03 mm about it. Against it,
    or at least not for it: every pilot body was placed at 0 deg on one land, so a
    land-level offset that every maker's model shares on CHIP_0402 would look the
    same; and on C2 board 3, the one record with chip parts at two rotations, the
    large CHIP_2010 offset turns with the part, as a model's would, and not with the
    board (``docs/library-supplier-w3-sampling.md`` §5).

    So taking the median off proves only that a sample agrees with the other bodies
    of its view, not that it is centred. The rule therefore also bounds the raw
    offset (``offset_raw_fraction``): a view whose bodies all sit off by the same
    amount passes only while that amount is within the per-part limit itself.
    """
    if not offsets:
        raise ValueError("no drawn chip bodies to take a bias from")
    return (round(statistics.median(x for x, _ in offsets), 4),
            round(statistics.median(y for _, y in offsets), 4))


def sample_reading(angle_deg: float, offset_xy_mm: tuple[float, float], offset_limit_mm: float,
                   bias: tuple[float, float], *, body_method: int = 1) -> dict:
    """The record entry for one sample: the method its body was read by, its angle,
    its raw offset (vector and size), the view's bias taken off it, and what is left
    (the residual). BODY_METHOD is the reading's own (``measure_preview.body_method``:
    a reading that names none is method 1)."""
    raw = (round(offset_xy_mm[0], 3), round(offset_xy_mm[1], 3))
    return {"body_method": body_method,
            "angle_deg": round(angle_deg, 1),
            "offset_limit_mm": round(offset_limit_mm, 3),
            "raw_offset_xy_mm": list(raw),
            "raw_offset_mm": round(math.hypot(*raw), 3),
            "bias_xy_mm": [bias[0], bias[1]],
            "residual_offset_mm": round(math.hypot(raw[0] - bias[0], raw[1] - bias[1]), 3)}


def sample_ok(reading: dict) -> bool:
    """A reading within the rule: read by the rule's method (one that names none is
    method 1), the angle, the raw offset and the residual each within their limits,
    and the raw size and the residual what its vectors give."""
    limit = reading["offset_limit_mm"]
    (rx, ry), (bx, by) = reading["raw_offset_xy_mm"], reading["bias_xy_mm"]
    method = reading.get("body_method", 1)
    return (isinstance(method, int) and not isinstance(method, bool) and method == RULE["body_method"]
            and abs(reading["angle_deg"]) <= RULE["angle_limit_deg"]
            and abs(reading["raw_offset_mm"] - math.hypot(rx, ry)) <= ROUNDING_MM
            and abs(reading["residual_offset_mm"] - math.hypot(rx - bx, ry - by)) <= ROUNDING_MM
            and reading["raw_offset_mm"] <= RULE["offset_raw_fraction"] * limit
            and reading["residual_offset_mm"] <= RULE["offset_residual_fraction"] * limit)
