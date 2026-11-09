"""The W3 sampled-inference rule for chip resistors and MLCCs.

Tony accepted the rule on 2026-11-09 for the non-polar chip resistors and MLCCs of
``lib/passive`` only (https://github.com/conol-ai/cohdl/pull/79#issuecomment-6734040188).
The numbers below are the ones proposed in ``docs/library-supplier-w3-sampling.md``
§5; the sample limits and the zero-stock rule await his confirmation there.

``tests/library/supplier.rs`` holds the same numbers and rejects a group record
whose ``rule`` differs from ``RULE``, so a record cannot loosen its own rule.
"""

from __future__ import annotations

import math
import statistics

RULE = {
    "version": "w3-2026-11-10",
    "package": "passive",
    "types": ["ChipResistor", "MLCC"],
    # At least MIN_SAMPLE members or FRACTION of them, whichever is larger.
    "min_sample": 3,
    "fraction": 0.05,
    # Samples are drawn only from members with this much stock on their confirmed page.
    "min_stock": 30,
    # A sample's body must lie along its pads within this, mod 180.
    "angle_limit_deg": 10.0,
    # ... and be centred within this fraction of the per-part limit, after the board's
    # median body offset is taken off (see board_bias).
    "offset_fraction": 0.5,
    "offset_bias": "board_median",
    # A member with no stock on its confirmed page is not bound by inference.
    "infer_zero_stock": False,
}


def required(members: int) -> int:
    return max(RULE["min_sample"], math.ceil(RULE["fraction"] * members))


def board_bias(offsets: list[tuple[float, float]]) -> tuple[float, float]:
    """The median body-centre offset over every drawn chip body of one preview view.

    On the 2026-11-07 pilot all three groups, two makers and both part types sat
    about 0.077 mm low in the export (Yageo CC 0.061, Yageo RC 0.079, Vishay CRCW
    0.067), with a spread of 0.01-0.03 mm about it: a property of the export, not
    of any one model. Judging samples against the board's own median leaves the
    spread, which is what tells one model from another.
    """
    if not offsets:
        raise ValueError("no drawn chip bodies to take a bias from")
    return (statistics.median(x for x, _ in offsets), statistics.median(y for _, y in offsets))


def sample_reading(angle_deg: float, offset_xy_mm: tuple[float, float], offset_limit_mm: float,
                   bias: tuple[float, float]) -> dict:
    """The record entry for one sample: its angle and its offset from the board's bias."""
    offset = math.hypot(offset_xy_mm[0] - bias[0], offset_xy_mm[1] - bias[1])
    return {"angle_deg": round(angle_deg, 1), "offset_mm": round(offset, 3),
            "offset_limit_mm": round(offset_limit_mm, 3)}


def sample_ok(reading: dict) -> bool:
    return (abs(reading["angle_deg"]) <= RULE["angle_limit_deg"]
            and reading["offset_mm"] <= RULE["offset_fraction"] * reading["offset_limit_mm"])
