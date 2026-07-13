# MVP demo evidence

The acceptance test from the MVP Definition
(`docs/design/09-mvp-definition.md`): from a one-paragraph natural-language
spec — *"An ESP32-S3-based sensor node: USB-C power/data, one MEMS microphone,
one status LED, a 3.3V LDO regulator, standard decoupling"* — an LLM (no
fine-tuning, prompted only with the language reference) writes `.cohdl`
source; the compiler grades it; diagnostics are fed back verbatim; the loop
lands on a clean design that emits a real KiCad netlist + BOM.

Reproduce with `python3 harness/repair_loop.py` (see the harness README
header for backends).

## Runs (2026-07-13)

| Run | Model | Reference | Attempts | Caught & repaired |
|---|---|---|---|---|
| 1 | claude-opus-4-8 | full | 2 | E202 ×2 (unresolved name), D003 warning |
| 2 | claude-opus-4-8 | full | 1 | — clean first try |
| 3 | claude-opus-4-8 | full | 1 | — clean first try |
| 4 | claude-opus-4-8 | full, reworded spec | 1 | — clean first try |
| 5 | claude-opus-4-8 | lean (no design notes) | 1 | — clean first try |
| 6 | claude-opus-4-8 | lean, detailed spec | 2 | E202 (unresolved name) |
| 7 | claude-haiku-4-5 | lean | 1 | — clean first try |
| 8 | claude-opus-4-8 | lean, +custom tantalum device | 1 | — clean first try (wrote device + impl mapping + part correctly) |
| **9** | **claude-haiku-4-5** | **lean, +custom tantalum device** | **2** | **E701 — unresolved required pin, repaired** |

## The proof transcript

[`transcript-flagship-haiku-e701.md`](transcript-flagship-haiku-e701.md) is
the run the MVP's non-negotiable clause asks for. On attempt 1 the model left
the ESP32-S3's chip-enable pin unwired, and the **type checker** — not DRC —
caught it as a compile error naming the exact line:

```text
error[E701]: required pin `SensorNode::esp32.EN` is unresolved: add it to a `net` or explicitly mark it `nc`
 --> src/main.cohdl:20:5
   |
20 |     inst esp32: ESP32_S3_WROOM_1_N8
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  :: std/soc.cohdl:13:9
   |
13 |         required EN: 3 [input]
   |         ---------------------- `EN` is declared `required` on device `ESP32_S3_WROOM_1` here
```

This is the exact mistake class (a forgotten wire) that v1 shipped silently
and that RFC-002 moved into the type system. The diagnostic was fed back
verbatim; attempt 2 wired EN through a 10k pull-up and landed clean —
[`sensor-node.net`](sensor-node.net) and
[`sensor-node-bom.csv`](sensor-node-bom.csv) are its emitted artifacts.

Two supplementary transcripts:

- [`transcript-opus-e202.md`](transcript-opus-e202.md) — a resolution-error
  catch + repair cycle (the model invented a v1-style bare net reference,
  which the MVP grammar deliberately dropped).
- [`transcript-opus-custom-device.md`](transcript-opus-custom-device.md) —
  the expressiveness half of the thesis: asked for a part the std library
  doesn't have, the model declared a polarized tantalum device, the explicit
  `impl TwoTerminal` pin mapping (`A: Anode, B: Cathode`), `impl Capacitor` +
  `impl Polarized`, and a part binding — all first try, all checked at the
  impl statements.

A note on difficulty: 6 of 9 runs were clean on the first attempt. The type
system's catches fire exactly when the model errs — with a complete language
reference in context, frontier models err rarely, which is the
AI-generatability half of the design working as intended (Constitution
priority #2). The failed attempts are what the compiler-as-oracle is for.

## KiCad checkpoint — executed

The import was driven through real KiCad (pcbnew 10.0.4, its own engine, via
`tools/kicad_checkpoint.py` — the same work File → Import Netlist performs):

- **All 14 footprints resolved from KiCad's official libraries**, every
  netlist pin matched a real footprint pad, designators verified unique.
- The resulting board is committed as
  [`sensor-node.kicad_pcb`](sensor-node.kicad_pcb) (open it directly in
  pcbnew — footprints placed, pad nets assigned, connectivity drawn), with a
  rendered [`sensor-node-board.pdf`](sensor-node-board.pdf) showing the
  components and net connectivity: USB-C → CC pulldowns, VBUS → LDO → 3.3V
  rail → decoupling bank + tantalum, ESP32 → I2S mic, EN pull-up, LED chain.

The checkpoint caught a real netlist-fidelity bug, exactly as intended: the
std USB-C device used combined pad names (`A1B12`, `S1`) that don't exist on
KiCad's official HRO footprint (separate `A1…B12` pads, shield `SH`). The std
device was corrected against the real footprint and everything regenerated.

To confirm by eye: open `sensor-node.kicad_pcb` in KiCad (or just the PDF),
and cross-check `sensor-node-bom.csv` — every line item carries a real MPN.
