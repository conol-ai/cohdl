# Joint motor controller

One daisy-chainable CAN 2.0B joint node for a 2S lithium pack (7.4V
nominal, 6.0-8.4V), built as three RFC-032 `subdesign` regions around an
STM32F072:

- **`LogicRail`** — a JW5033S synchronous buck running the datasheet's own
  3.3V/2A typical application verbatim (49.9k/16k divider on the 0.8V
  reference, 3.3uH, 100nF bootstrap, 100k enable pull-up to the pack).
- **`CanPort`** — an SN65HVD230 in slope-control mode (10k RS, ~15V/us
  edges). No on-board termination: joints sit mid-bus and chain through
  two identical Micro-Fit connectors; the 120-ohm terminations belong at
  the two harness ends.
- **`MotorStage`** — a DRV8231A H-bridge straight off the pack (VM
  4.5-33V), reporting load current through IPROPI (1500uA/A into 560 ohm
  = 3.11V at the 3.7A peak) and chopping at I_TRIP = 1.96A from a halved
  3.3V VREF — stall protection with no firmware in the loop.

Joint angle comes from an external potentiometer on the AUX socket's ADC
input; the 12MHz HSE crystal exists because bxCAN bit timing needs crystal
accuracy, not the HSI RC. The node ID lives in firmware flash, set over
SWD at commissioning.

Each subdesign carries its own default internal layout and drops onto the
board as one placed unit; the design overrides exactly one internal
position (`place motor.c_vm_bulk_b …`) to pull the second bulk capacitor
into the motor connector's current loop — the RFC-032 placement reach-in.

Build it:

```sh
cargo run -- build examples/joint-motor-controller --emit kicad_pcb --emit easyeda --emit ipc2581
```
