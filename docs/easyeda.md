# The EasyEDA / LCEDA Pro netlist emitter — `cohdl build --emit easyeda`

`cohdl build --emit easyeda` writes `out/<name>.enet`: an LCEDA Pro
(立创EDA) / EasyEDA Pro netlist, JSON format version 2.0.0, imported via
**File → Import → Netlist**. This is the Constitution's second named
netlist target (KiCad `.net`, LCEDA `.enet`, BOM CSV); the MVP note cut it
as redundant proof of netlist fidelity, and the board author directed its
addition on 2026-08-27 — ledgered in `docs/compliance-report.md`.

It is a **netlist**, not a board: components, pins, and net bindings, so
EasyEDA can place and route with a live ratsnest. Placement/outline
geometry stays with the geometry artifacts (`--emit kicad_pcb`,
`--emit ipc2581`). All three `--emit` values compose in one build.

## Document shape

The byte shape is pinned against the v1 compiler's emitter (`legacy`
branch, `crates/cohdl-codegen-lceda`), whose output this importer
accepted:

- Top level, in v1's struct order: `version` ("2.0.0"), `components`,
  `designRule` (`trackPhysics` then `netRule`), `differentialPair`,
  `netClass`, `equalLengthNetGroup` — the last three empty, and
  `trackPhysics` empty, exactly as v1 wrote them.
- Every JSON object's keys are plain string-sorted (v1's `serde_json`
  BTreeMap bytes — `gge1` before `gge10` before `gge2`; key order carries
  no meaning to a JSON importer). Two-space pretty indentation; the file
  ends with a newline.
- `components` is keyed by `Unique ID` (`gge<n>`, counting up in
  designator natural order — C2 before C10, C9 before U1). Each component:
  - `props`: `Add into BOM` / `Convert to PCB` (both "yes"),
    `Designator`, `DeviceName` (the device's short name — the `.net`'s
    libsource part), `FootprintName` (the resolved footprint symbol's
    fully-qualified path — the `.net`'s footprint field; empty when the
    part has no footprint), `Manufacturer` / `Manufacturer Part` (the
    part's `mfr`/`mpn`, present only when the part carries them), `Name`
    (the same principal value the `.net` and BOM use), `Unique ID`.
  - `pinInfoMap`: one row per **physical pad** of each connected logical
    pin, keyed by pad number — a multi-pad logical pin (GND on 2 and 3)
    flattens to one row per pad, the RFC-027 convention. Each row:
    `name` (logical pin), `number`, `net` (by name), and the v1 `props`
    (`Pin Number`). A pin-less mechanical part keeps an empty map and
    still converts to PCB.
- `designRule.netRule`: one row per net (`net` + the empty
  `ruleMap.TrackPhysics` binding v1 wrote), so every net exists on the
  EasyEDA side even before any rule is authored there.

## Semantics: the `.net`, re-projected

Every derivation is shared with the KiCad netlist emitter — designator
natural order, `principal_value`, the footprint symbol's fq path,
physical-pin expansion through `Device::pins_for` — so the two netlists
cannot disagree about the design (pinned by `tests/easyeda.rs`).

`nc` pins are represented by their guaranteed absence, the same
DR-012 convention the `.net` documents: no row in any `pinInfoMap`, no
net membership.

## Ownership and staleness

`out/<name>.enet` is manifest-owned like every artifact: emitted only
under the flag, byte-stable, swept as stale when a later build omits
`--emit easyeda`. Zero impact on every other artifact and on the verdict.

## Verification status

Byte shape and semantics are pinned by `tests/easyeda.rs` (structure,
determinism, `.net` agreement, JSON well-formedness via python3 — the
xmllint pattern — and both repo examples). A live **File → Import →
Netlist** into LCEDA Pro / EasyEDA Pro is the human checkpoint, the same
role a live pcbnew open plays for `--emit kicad_pcb`.
