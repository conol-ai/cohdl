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
- RFC-008…RFC-032 — the post-MVP Accepted RFCs (compiler-implemented, with named open gaps recorded in [`docs/compliance-report.md`](docs/compliance-report.md) — notably RFC-014/015 real-client/partner acceptance passes remain open): language, tooling, layout, packaging, registry, footprint, manufacturing-output, and editor features, including `#[virtual]` connectivity-only instances (RFC-032)

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

All MVP exit criteria are met, and all 32 Accepted RFC areas (RFC-001…032) have implementation work with conformance suites. That is deliberately not a claim of full accepted-text compliance: requirements that are incomplete (e.g. RFC-014's real-editor acceptance pass and RFC-015's real-partner/Quilter import pass — both open), deliberately deviating (e.g. the E9xx code assignment, D003's role-aware reading, RFC-015's constraint mapping riding vendor extensions rather than native IPC semantics), or unrepresentable pending note-side amendments (e.g. RFC-013's E1005) are each recorded with rationale in the ledger, [`docs/compliance-report.md`](docs/compliance-report.md). Tested (`cargo test` — [`tests/exit_criteria.rs`](tests/exit_criteria.rs) maps 1:1 to the MVP checklist; each post-MVP RFC has its own conformance suite under `tests/`). The demo loop has run end-to-end: see [`docs/demo/`](docs/demo/) for transcripts (including a genuine E701 unresolved-required-pin catch + repair) and the emitted netlist/BOM. RFC/decision-record accuracy was audited claim-by-claim: [`docs/compliance-report.md`](docs/compliance-report.md). The KiCad checkpoint has been executed with real KiCad (pcbnew imported the netlist, resolved all footprints and pads; board + render in `docs/demo/`).

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
cargo run -- build examples/rpi-pico2 --emit kicad_pcb # + a native KiCad 10 board file — no KiCad needed
cargo run -- build examples/rpi-pico2 --emit easyeda   # + an EasyEDA / LCEDA Pro netlist (.enet)
cargo run -- fmt lib --check                     # canonical-form gate (RFC-009)
cargo run -- lsp                                 # LSP server on stdio (RFC-014, docs/lsp.md)
cargo run -- search TPS59650                     # search registry packages + public parts
cargo run -- search TPS59650 --json              # the same bounded results as JSON
python3 harness/repair_loop.py                   # the generate → check → repair demo
```

`cohdl search QUERY [--json]` is read-only discovery: it needs no project and
no registry login. Queries are trimmed, must contain at least three Unicode
scalar values, may be at most 128 UTF-8 bytes, and may not contain control
characters. Results identify the owning package and exact most-recently-
published version as well as importable public part paths and matching primary
or alternate manufacturer/MPN data; a valid query with no matches still exits
successfully. Use the standard option terminator when a literal query starts
with a hyphen, for example `cohdl search -- -12V`. “Most recently published”
is publication chronology, not the greatest semantic version chosen by an
unversioned `cohdl add`/`cohdl update`.
The package and part result families are bounded independently; each exposes
`has_more` when truncated and never a total count. Human output separates the
two families, while `--json` returns the same rows as one discovery document.
For rollout, owners of existing packages backfill the current searchable
version once with `cohdl docs --publish`; this does not change its tar or hash.

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

`cohdl build` writes to `out/` (or `--out-dir`): the KiCad netlist (`<name>.net`), the BOM (`<name>-bom.csv`), the designator lock (`design.lock` in the project root), a `.kicad_mod` per pad-bearing footprint under `out/footprints/` (RFC-018), and — only when the design declares layout metadata — the layout-constraint artifact `<name>-layout.json` (schema: [`docs/layout-json.md`](docs/layout-json.md)). With `--emit ipc2581` it additionally writes the IPC-2581B1 handoff document `<name>.xml` (RFC-015; [`docs/ipc2581.md`](docs/ipc2581.md)), with `--emit kicad_pcb` a native KiCad 10 board file `<name>.kicad_pcb` — placements, net-bound footprints, and the board outline, with no KiCad installation involved ([`docs/kicad_pcb.md`](docs/kicad_pcb.md)), and with `--emit easyeda` an EasyEDA / LCEDA Pro netlist `<name>.enet` for **File → Import → Netlist** ([`docs/easyeda.md`](docs/easyeda.md)); the flags compose. Stale layout/IPC/board artifacts are removed when their source data is gone (the IPC document only if CoHDL wrote it — its completeness marker establishes ownership).

### Looking at a design

A checked design can be read back as an interactive schematic-style board —
every instance, net and pin the compiler resolved, searchable and traceable,
with datasheets and to-scale footprint previews in a side panel. It is a
read-only projection: the `.cohdl` source stays the only thing that defines
the circuit.

On macOS, download the signed, notarized app from the latest
[`explorer-v*` release](https://github.com/conol-ai/cohdl/releases?q=explorer&expanded=true)
(mount the DMG, drag to Applications, pick a project). Or run it from source:

```sh
cd explorer/web && npm install && npm run build   # once
cd ../extractor
COHDL_LIB=../../lib cargo run --release -- ../../examples/rpi-pico2 \
    --serve --dist ../web/dist --port 5199        # http://127.0.0.1:5199/
```

Editing any source file re-extracts and refreshes the page within ~500 ms; a
source that fails to compile keeps the last good view and shows the
diagnostics. Extraction, layout and wire routing are deterministic code — the
same source always draws the same board. An agent's only input is the
partition file `explorer/views/<Design>.view.json`, which names the page tabs
and the region each part belongs to; the contract for writing one is
[`explorer/skills/view-gen/SKILL.md`](explorer/skills/view-gen/SKILL.md).
See [`explorer/README.md`](explorer/README.md).

### Exit codes

`0` = clean (warnings and a valid no-match search are allowed); `1` = the
accepted command failed (source errors, non-canonical `fmt --check`, or an E1204
search registry/protocol failure); `2` = invocation-level failure (bad flags,
an invalid search query, missing project, design selection, nothing to build).
Human diagnostics use stderr. A successful `search --json` emits its discovery
document on stdout, and an E1204 failure in that mode emits the existing
diagnostic JSON there; bad invocations remain prose-only on stderr. See the
formal [`docs/error-codes.md`](docs/error-codes.md) registry. Note: RFC-010's
text reserves stderr prose for pre-collection failures, so classifying
post-collection selection failures this way is a documented deviation pending
a note-side amendment (or a v2 schema envelope).

## License

MIT — see [`LICENSE`](LICENSE). This covers the compiler crate, the libraries
under `lib/`, the reference designs in `examples/`, the registry
server in `registry/`, the VS Code extension in `editors/vscode/`, and the
tooling in `tools/` and `harness/`.

Every package published to registry.cohdl.org must declare its own
`[package] license`; every package shipped under `lib/` declares `MIT`.
