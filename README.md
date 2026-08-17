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
- RFC-008…RFC-018 — the post-MVP Accepted RFCs (compiler-implemented, with named open gaps recorded in [`docs/compliance-report.md`](docs/compliance-report.md) — notably RFC-016/017 on-disk dependency loading is not yet built, and RFC-014/015 real-client/partner acceptance passes remain open): structural variants, `cohdl fmt`, `cohdl check --json`, the formal error-code registry, `#[intent]` annotations, layout constraints, the LSP server (`cohdl lsp`, [`docs/lsp.md`](docs/lsp.md)), the IPC-2581 handoff document (`build --emit ipc2581`, [`docs/ipc2581.md`](docs/ipc2581.md)), the module system, the library registry, the pad/footprint format (`pad`/`footprint` geometry → `.kicad_mod` + IPC-2581 pins), and a real VS Code extension (`editors/vscode/`, packaging `cohdl lsp` with a TextMate grammar — RFC-019)

## MVP (v0.1) scope

The smallest slice of the spec that proves the thesis end-to-end:

1. **Parser** — traits, devices, free-standing `impl`, `fn` declarations/calls (nested), design bodies (`inst`, `net`, `nc`), the eleven unit-type literal forms
2. **Type checker** (the heart) — zero-coercion unit types, pin connection-obligation exhaustiveness, trait satisfaction with explicit mapping, generic resolution with trait-bound checking, nested fn expansion with cycle detection
3. **Residual DRC** — exactly four rules: voltage-exceed, polarity-mismatch, single-driver, multi-driver
4. **Designators** — collision-free allocator, `design.lock` with tombstones, `#[designator("…")]` overrides
5. **Codegen** — KiCad `.net` netlist + BOM CSV
6. **Minimal std library** — only the universal traits every component package needs
7. **Repair harness** — generate → check → repair loop driving an LLM against the compiler's diagnostics

The proof: an AI writes a board from a plain-language spec, the type checker catches its structural mistakes as precise compile errors, the AI repairs them from those diagnostics, and the result is a real, importable KiCad netlist with an honest BOM.

## Status

All MVP exit criteria are met, and all nineteen Accepted RFC areas (RFC-001…019) have implementation work with conformance suites. That is deliberately not a claim of full accepted-text compliance: requirements that are incomplete (e.g. RFC-014's real-editor acceptance pass and RFC-015's real-partner/Quilter import pass — both open), deliberately deviating (e.g. the E9xx code assignment, D003's role-aware reading, RFC-015's constraint mapping riding vendor extensions rather than native IPC semantics), or unrepresentable pending note-side amendments (e.g. RFC-013's E1005) are each recorded with rationale in the ledger, [`docs/compliance-report.md`](docs/compliance-report.md). Tested (`cargo test` — [`tests/exit_criteria.rs`](tests/exit_criteria.rs) maps 1:1 to the MVP checklist; each post-MVP RFC has its own conformance suite under `tests/`). The demo loop has run end-to-end: see [`docs/demo/`](docs/demo/) for transcripts (including a genuine E701 unresolved-required-pin catch + repair) and the emitted netlist/BOM. RFC/decision-record accuracy was audited claim-by-claim: [`docs/compliance-report.md`](docs/compliance-report.md). The KiCad checkpoint has been executed with real KiCad (pcbnew imported the netlist, resolved all footprints and pads; board + render in `docs/demo/`).

## Install

One line (macOS and Linux; installs to `~/.cohdl/bin`):

```sh
curl -fsSL https://raw.githubusercontent.com/conol-ai/cohdl/main/install.sh | sh
```

Prebuilt binaries (and their `sha256sums.txt`) are attached to every
[`vX.Y.Z` release](https://github.com/conol-ai/cohdl/releases); Windows users
download `cohdl-vX.Y.Z-x86_64-pc-windows-msvc.tar.gz` by hand. An installed
binary updates itself with `cohdl self-update` (`--check` reports without
installing). Building from source stays `cargo build` (toolchain pinned by
`rust-toolchain.toml`).

## Using it

```sh
cargo run -- check examples/sf32-miniboard       # SF32 miniboard: SiFli BLE eval board
cargo run -- build examples/rpi-pico2            # + designators, parts, KiCad .net + BOM CSV
cargo run -- check examples/rpi-pico2 --json     # structured diagnostics (RFC-010)
cargo run -- build examples/rpi-pico2 --emit ipc2581  # + IPC-2581 handoff document (RFC-015)
cargo run -- fmt lib --check                     # canonical-form gate (RFC-009)
cargo run -- lsp                                 # LSP server on stdio (RFC-014, docs/lsp.md)
python3 harness/repair_loop.py                   # the generate → check → repair demo
```

`std` is a core-traits-only prelude; it carries no devices, parts, pads, or
footprints. Shipped component packages include `passive`, `connectors`, `usb`,
`esd`, `diode`, `flash`, `ldo`, `led`, `mic`, `mosfet`, `osc`, and
manufacturer packages for Espressif, Raspberry Pi, Richtek, ST, and TI.
Projects pin every package they use to an exact version under
`[dependencies]`; see [`lib/README.md`](lib/README.md) for the complete
namespace map.

Two reference designs live in `examples/`: Raspberry Pi **Pico 2** and the
**SF32 miniboard**. (The **OpenMicro** macropad — the wired STM32F072 v1 and
its wireless SF32LB52 successor v2 — graduated to its own repository,
`openmicrokbd`, which carries the whole product: hardware source, firmware,
host app, and fab releases under `hw/v1/` and `hw/v2/`.)

An **AI voice robot-dog mainboard** is in progress and not yet in the tree; its
exit-criteria test is checked in but ignored until the example lands. It
combines an ESP32-S3-N8R2, stereo I2S microphones,
isolated speaker playback, a translated 1.8 V IMU domain, USB-C, a protected
default-off 7 A actuator rail, and eight 5 V servo PWM channels. Keyed 4/6-pin
Micro-Fit harnesses prevent the BEC input from being interchanged with a leg.

### Build artifacts

`cohdl build` writes to `out/` (or `--out-dir`): the KiCad netlist (`<name>.net`), the BOM (`<name>-bom.csv`), the designator lock (`design.lock` in the project root), a `.kicad_mod` per pad-bearing footprint under `out/footprints/` (RFC-018), and — only when the design declares layout metadata — the layout-constraint artifact `<name>-layout.json` (schema: [`docs/layout-json.md`](docs/layout-json.md)). With `--emit ipc2581` it additionally writes the IPC-2581B1 handoff document `<name>.xml` (RFC-015; [`docs/ipc2581.md`](docs/ipc2581.md)). Stale layout/IPC artifacts are removed when their source data is gone (the IPC document only if CoHDL wrote it — its completeness marker establishes ownership).

### Exit codes

`0` = clean (warnings allowed), `1` = source diagnostics reported (errors; text on stderr, or one JSON document on stdout with `--json`), `2` = invocation-level failure (bad flags, invalid flag for the command, missing project, design selection, nothing to build) — prose on stderr, never a JSON document (the `E000` class in [`docs/error-codes.md`](docs/error-codes.md)); source diagnostics collected before the failure still render to stderr first. Note: RFC-010's text reserves stderr prose for pre-collection failures, so classifying post-collection selection failures this way is a documented deviation pending a note-side amendment (or a v2 schema envelope).

## License

MIT — see [`LICENSE`](LICENSE). This covers the compiler crate, the libraries
under `lib/`, the reference designs in `examples/`, the registry
server in `registry/`, the VS Code extension in `editors/vscode/`, and the
tooling in `tools/` and `harness/`.

Every package published to registry.cohdl.org must declare its own
`[package] license`; every package shipped under `lib/` declares `MIT`.
