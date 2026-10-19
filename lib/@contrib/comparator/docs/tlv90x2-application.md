# TLV9022DR / TLV9032DR application contract

Exact SOIC-8 parts: TLV9022DR has open-drain outputs; TLV9032DR has push-pull
outputs. Both use `soic::TI_D0008A_4214825C` from soic 0.2.1.
They are comparators, not PD decoders, controllers, or qualified sniffer circuits.

## Bindings and supporting circuit

| Pin | Library name | Function |
|---|---|---|
| 1 | OUT1 | Channel 1 output |
| 2 | IN1_N | Channel 1 inverting input |
| 3 | IN1_P | Channel 1 noninverting input |
| 4 | V_MINUS | Negative supply; ground for single-supply use |
| 5 | IN2_P | Channel 2 noninverting input |
| 6 | IN2_N | Channel 2 inverting input |
| 7 | OUT2 | Channel 2 output |
| 8 | V_PLUS | Positive supply |

Source: page 4, Figure 4-3/Table 4-2. No SOIC exposed pad.
Use 0.1 uF local supply bypassing (page 34). Unused inputs need distinct valid
levels with at least 50 mV separation; do not tie them together. Unused outputs
may float. TLV9022 needs pullups; TI suggests 100 uA-1 mA pullup current.
TLV9032 outputs must not be shorted to rails or other outputs (pages 21-23).

## Limits to preserve in design evidence

Supply: 1.65-5.5 V; ambient: -40 to 125 C (page 7).
The page-22 fault-tolerance statement covers 0-5.5 V inputs during supply-off
and supply ramps, without a clamp to V_PLUS. It does not specify maximum
leakage. Page 11 gives 5 pA bias, 2 pF differential and 3 pF common-mode
capacitance as **typical**, not maxima. Negative-input clamps remain.

Page 12 timing is **typical at 5 V, 25 C, 15 pF load and 100 mV overdrive**:
100 ns falling; 115 ns rising push-pull or 150 ns open-drain with 2.5 kohm
pullup. These are not worst-case propagation guarantees. No internal
hysteresis is provided (page 23). Output validity and full specifications
require the documented input/supply conditions; fault survival is different.

## Applying this to a passive receiver

Engineering use: the input structure makes these candidates for observing CC
without a controller's automatic termination or packet responses. A complete
design must still bound tap loading, transients, receiver thresholds and
timing across its operating envelope. Do not turn typical numbers into pass
limits or connect an output to monitored CC/VBUS. Power the comparator and
capture logic from the separate host domain. Include protection and reference
networks in the power-off/backfeed analysis, not just the IC input.

External-comparator outputs require verified digital MCU input/timer routes;
an internal-comparator firmware example does not prove that routing. Packet
decoding, buffering, timestamp accuracy, orientation and VCONN remain separate
design obligations. No stock or JLC assembly availability is asserted here.

## Source

[TI datasheet](https://www.ti.com/lit/ds/symlink/tlv9022.pdf), SNOSDA3H,
November 2025; captured 2026-10-17 Asia/Shanghai. Orderable parts on PDF
pages 38-39 (May 16, 2026 addendum); geometry on pages 68-69.
Unmodified PDF SHA-256:
`874c1d4f5ec1c1b4bf6c714d3fe8623d57ad6bff18b83930e3894061a0701c8a`.
PDF copyright remains with Texas Instruments. Authored code and note: MIT.
