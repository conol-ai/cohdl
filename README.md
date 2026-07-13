# CoHDL

CoHDL is the programming language that makes schematic (PCB) design AI-native — the way software became AI-native. AI generates and repairs hardware; the compiler is the oracle that grades it.

This is the **v2 ground-up implementation**, restarted 2026-07-13 from the redesigned language specification. The central thesis of the redesign: **strictness buys expressiveness** — push correctness into the type system wherever a mistake is structural (wrong units, unresolved pins, unsatisfied trait bounds), and reserve DRC only for genuinely emergent, whole-graph checks (net voltage, driver conflicts).

The previous implementation lives on the [`legacy`](https://github.com/conol-ai/cohdl/tree/legacy) branch. It is not being fixed; this is a fresh build informed by its lessons.

## Design repository

The source of truth for the language design is the Coherent Design Repository on conol.ai:
<https://conol.ai/share/note/od34sne4sa5ujuohyhldr21r>

A snapshot (taken 2026-07-13) is committed under [`docs/design/`](docs/design/). Key documents:

- [MVP Definition](docs/design/09-mvp-definition.md) — the scope line, demo scenario, and exit criteria this repository was built toward
- [Language Specification](docs/design/10-language-specification.md) — the compiled statement of what the language is (Accepted RFCs only)
- RFC-001…RFC-007 — the seven Accepted P0 RFCs: units-as-types, pin connection-obligation typing, trait satisfaction at impl time, DRC/type-system reclassification, collision-free designator allocation, nested fn call semantics, generics-over-specs
- RFC-008…RFC-013 — the post-MVP Accepted RFCs, all implemented: structural variants, `cohdl fmt`, `cohdl check --json`, the formal error-code registry, `#[intent]` annotations, layout constraints

## MVP (v0.1) scope

The smallest slice of the spec that proves the thesis end-to-end:

1. **Parser** — traits, devices, free-standing `impl`, `fn` declarations/calls (nested), design bodies (`inst`, `net`, `nc`), the ten unit-type literal forms
2. **Type checker** (the heart) — zero-coercion unit types, pin connection-obligation exhaustiveness, trait satisfaction with explicit mapping, generic resolution with trait-bound checking, nested fn expansion with cycle detection
3. **Residual DRC** — exactly four rules: voltage-exceed, polarity-mismatch, single-driver, multi-driver
4. **Designators** — collision-free allocator, `design.lock` with tombstones, `#[designator("…")]` overrides
5. **Codegen** — KiCad `.net` netlist + BOM CSV
6. **Minimal std library** — only what the ESP32-S3 sensor-node demo board needs
7. **Repair harness** — generate → check → repair loop driving an LLM against the compiler's diagnostics

The proof: an AI writes a board from a plain-language spec, the type checker catches its structural mistakes as precise compile errors, the AI repairs them from those diagnostics, and the result is a real, importable KiCad netlist with an honest BOM.

## Status

All MVP exit criteria are met, and all thirteen Accepted RFCs (RFC-001…013) are implemented and tested (`cargo test` — [`tests/exit_criteria.rs`](tests/exit_criteria.rs) maps 1:1 to the MVP checklist; each post-MVP RFC has its own conformance suite under `tests/`). The demo loop has run end-to-end: see [`docs/demo/`](docs/demo/) for transcripts (including a genuine E701 unresolved-required-pin catch + repair) and the emitted netlist/BOM. RFC/decision-record accuracy was audited claim-by-claim: [`docs/compliance-report.md`](docs/compliance-report.md). The KiCad checkpoint has been executed with real KiCad (pcbnew imported the netlist, resolved all footprints and pads; board + render in `docs/demo/`).

## Using it

```sh
cargo run -- check examples/sensor-node          # parse → resolve → type-check → residual DRC
cargo run -- build examples/rpi-pico2            # + designators, parts, KiCad .net + BOM CSV
cargo run -- check examples/rpi-pico2 --json     # structured diagnostics (RFC-010)
cargo run -- fmt std --check                     # canonical-form gate (RFC-009)
python3 harness/repair_loop.py                   # the generate → check → repair demo
```

Two reference designs live in `examples/`: the ESP32-S3 **sensor-node** (the MVP demo board) and the Raspberry Pi **Pico 2** (RP2350A — a full transcription of the official schematic, exercising variants, `#[intent]`, and layout constraints).

### Build artifacts

`cohdl build` writes to `out/` (or `--out-dir`): the KiCad netlist (`<name>.net`), the BOM (`<name>-bom.csv`), the designator lock (`design.lock` in the project root), and — only when the design declares layout metadata — the layout-constraint artifact `<name>-layout.json` (schema: [`docs/layout-json.md`](docs/layout-json.md)); a stale layout artifact is removed when the metadata is gone.

### Exit codes

`0` = clean (warnings allowed), `1` = source diagnostics reported (errors; text on stderr, or one JSON document on stdout with `--json`), `2` = invocation-level failure (bad flags, missing project, design selection) — prose on stderr, never a JSON document (the `E000` class in [`docs/error-codes.md`](docs/error-codes.md)).
