# RFC-0XX (number to be allocated): Population — fitted, do-not-populate and board-feature instances

> **PROPOSAL — not accepted; no number allocated.** Draft, 2026-11-13, written at the maintainer's request ("draft the DNP concept first"). The RFC number, the decision-record number and every error code (`E-TBD-*` below) are placeholders pending central allocation in the conol.ai design repository; this file is a working copy in the repository snapshot. Nothing here is implemented. Baseline: main `74403a6` (2026-11-13). Builds on RFC-002 (pin obligations), RFC-004 (residual DRC), RFC-005 (designators, `design.lock`), RFC-013/020/026 (layout, `place`), RFC-015 (IPC-2581), RFC-024 (instance arrays), RFC-032 (`subdesign`), RFC-033 (provisional; `for`) and RFC-034 (supplier bindings, BOM/CPL/map, provisional E814/E815).

## Problem

CoHDL has no way to say that a footprint on the board is not fitted with a component. Every instance is bought and placed.

- RFC-034 §3.1 states the rule in force: "There are no exclusions by value or designator. A part that is bought is placed", and names the gap: "A way to mark an instance as bought but not assembled is out of scope for this RFC." RFC-032's Non-goals also set "DNP/DNI assembly-population states" aside.
- **Evidence (compiler 0.10.0 and the published library).** Any board with a site that is laid out but not fitted meets this. What the compiler and the library do with such sites today:
  - Tuning and option sites, such as the shunt positions of an RF matching network or a bench substitute for a sensor, are ordinary instances. They often sit inside `subdesign` bodies, and several can be elements of one array declaration.
  - A site with no chosen value at all still needs a part, so a project declares one with a placeholder identity such as `mfr: "Unselected", mpn: "DNP"`.
  - Bare-copper test points (`misc::TEST_POINT_ROUND_1MM`) and non-plated mounting holes are parts too, and carry a placeholder identity such as `mfr: "PCB Fabrication"`, because a part must name an MPN.
  - `cohdl build --supplier lcsc` lists every one of these in the BOM, and each one with an SMD pad in both CPLs. An unfitted site whose part has an `lcsc` binding is ordered and placed.
  - A project therefore has to keep population in a side file of its own (a do-not-populate table and an exclude-from-BOM list, both keyed by designator) and a script that removes those rows from the order files, removes paste from the DNP pads, and fails if one of them reaches an order file.
- The side file is keyed by designator, which is allocator output and not something the source names. It is not checked by the compiler, not visible to an AI author reading the design, and not versioned with the meaning of the source: a misspelt designator silently populates a site.

Who this is for: anyone sending a build to an assembler; reviewers, who need to see in the source which sites are empty and why; AI authors, who cannot be expected to maintain a second file in a project-specific format.

## Goals

- One concept, **population**, that every assembly-facing emitter reads through one predicate, so the BOM, the CPLs, the map and the board cannot disagree about what is fitted.
- The fact lives in the source, on the declaration it describes, and is checked.
- Same source and same dependency set give the same bytes (Constitution). Population is not a build flag.
- A design that does not use the construct emits byte-identical output.
- Serves ladder ranks 1 (correctness: an order file that fits a DNP site is a wrong output), 3 (reviewability) and 5 (faithful output).

## Non-goals

- **Assembly variants** (one source, several populations selected per build). Named as future work and as a decision for the maintainer (Open questions, 1).
- **Bought but not machine-placed** (hand-soldered parts, a battery, a part shipped loose). This is a third state with its own BOM/CPL split; it is not designed here.
- Per-element or per-use population (one element of an array, one use of a subdesign). Future work together with variants.
- Removing a DNP part from the electrical design, substituting its value, or checking the unfitted board as a different circuit.
- Changing the provisional rule that a `part` names an MPN (Open questions, 5).
- Deciding which sites a design should leave empty.

## Design

### 1. The concept

Every physical instance has exactly one **population**, a closed set of three values:

| Population | Meaning | Source |
|---|---|---|
| `fitted` | Bought and assembled. The default. | nothing written |
| `dnp` | A component site left empty on the assembled board. The footprint, its copper and its nets are part of the board. | `#[dnp]` on the `inst` declaration |
| `board_feature` | Not a component: made by board fabrication (test point, mounting hole, fiducial, printed antenna, touch pad). Nothing is ever bought or placed for it. | `#[board_feature]` on the `part` declaration |

Two sources, because the two facts have different owners. Whether a resistor site is fitted is a decision of the design that instantiates it. That a test point is bare copper is a property of the part, true in every design, and belongs with the part's author. Emitters do not care which source produced the value: they ask one question, *is this instance fitted?*

### 2. `#[dnp]` on an instance

```cohdl
subdesign Matching {
    ports {
        required FEED: Pin
        required GND: Pin
    }

    #[dnp]
    #[intent("matching shunts, tuned on hardware")]
    inst shunt: [passive::C_10p_50V_C0G_0402; 2]

    #[dnp]
    inst shunt_l: SHUNT_UNFITTED
    ...
}
```

- A bare attribute with no arguments, attachable to `inst` only, at most once per declaration. The reason is written with the existing `#[intent("...")]` (RFC-012); population adds no second rationale channel.
- It marks **every physical instance the declaration expands to**: every element of an array (RFC-024), every iteration when the `inst` is in a `for` body (RFC-033), and every expansion of the enclosing `fn` or use of the enclosing `subdesign`.
- The instance is otherwise unchanged: it is still bound to its part, which still supplies its footprint, value and pins.
- **E-TBD-1** (error): `#[dnp]` on an `inst` whose type is a `subdesign` (a subdesign instance is not a component site; population of its components belongs to their own declarations); a repeated `#[dnp]`; `#[dnp]` with arguments; `#[dnp]` on anything that is not an `inst`.
- **E-TBD-2** (error): `#[dnp]` on an instance whose part is `#[board_feature]`. There is nothing to leave unfitted, and the two markers would give two answers to one question.
- `cohdl fmt` writes it as a single-line attribute before its target, in the existing attribute order.

### 3. `#[board_feature]` on a part

```cohdl
#[board_feature]
pub part TEST_POINT_ROUND_1MM: TestPoint {
    primary { mfr: "PCB Fabrication", mpn: "TESTPOINT-ROUND-1.0MM-SMD",
              footprint: FP_TEST_POINT_ROUND_1MM }
}
```

- A bare part marker, in the same position as the existing `#[source_unspecified(...)]` marker. Every instance bound to the part has population `board_feature`.
- **E-TBD-3** (error): a `#[board_feature]` part with an `alt` entry or a `supplier` clause (RFC-034). Nothing is bought, so an alternate source or a supplier part number is a contradiction.
- The part's `primary` is unchanged in this proposal, so existing parts gain the marker with a one-line edit.
- Adding the marker to a published part changes its consumers' BOM and CPL, so it ships as a new package version like any other content change (RFC-029).

### 4. What stays the same: the electrical design

A non-fitted instance is fully in the design. The copper exists whether or not a part is soldered to it.

- **Designators and `design.lock` (RFC-005).** The instance keeps its path, its lock row, its designator and its footprint. The allocator does not read population. Adding or removing `#[dnp]` changes no designator, in this design or any other.
- **Pin obligations (RFC-002).** Unchanged. Every `required` pin of a DNP instance must still be connected: the pad is on the board and will carry a net whenever the part is fitted on the bench.
- **Nets.** A DNP instance's pins stay members of their nets in every netlist. An unfitted two-terminal site is two pads on two nets, not an open circuit in the source.
- **Residual DRC (RFC-004).** All four rules are evaluated as if every site were fitted. No rule is added and none is weakened.
  - D001 voltage-exceed and D002 polarity-mismatch: a site that may be fitted later must be safe when it is.
  - D003 single-driver and D004 multi-driver read pin roles, and here "as if fitted" is conservative in one direction only. Two alternative driver sites on one net, one of them DNP, still report D004; a net whose only driver is a DNP site reports nothing. Both are stated limitations of this first step. Making the verdict depend on population would turn each assembly state into a different checked circuit, which is what variants are; see Open questions, 2.
- **Verdict.** Population never changes whether a design parses, resolves, type-checks, connects or passes DRC.

### 5. Effect on every emitted artifact

"Fitted" below means population `fitted`; "non-fitted" means `dnp` or `board_feature`.

| Artifact | Effect |
|---|---|
| KiCad `.net` | The component, its footprint and every pin's net membership are unchanged. A `dnp` component gains KiCad's `dnp` component property; a `board_feature` component gains `exclude_from_bom`. The exact tokens are pinned against a netlist written by KiCad before implementation, as `docs/kicad_pcb.md` did for the board. |
| `-bom.csv` (neutral) | Rows list fitted instances only. A group left with no designator is not written. Columns and ordering are unchanged. |
| `-population.csv` (new) | Written only when the design has at least one non-fitted instance: `Designator,Population,Footprint,Manufacturer,Comment`, one row per non-fitted instance in designator natural order. It is the record of what the other files left out, for the reviewer and the assembler's notes. Goes through `write_artifact` and the stale sweep like every artifact. |
| `-layout.json` | Unchanged. A layout tool must place every footprint, fitted or not. |
| `.kicad_pcb` | Every footprint is present at the same position with the same nets and the same uuid. `(attr …)` gains `dnp` for a `dnp` instance, and `exclude_from_pos_files exclude_from_bom` for a `board_feature` instance. **Paste:** the embedded footprint of a `dnp` instance carries no paste: its pads lose their paste layer and its paste-override apertures (RFC-018) are not written. A `board_feature` footprint is written as declared. The attribute tokens and the paste-free pad encoding are pinned against a pcbnew-written board before implementation. |
| `.kicad_mod` | Unchanged. It is library geometry, not an instance. |
| IPC-2581 | The `BomItem` and its `RefDes` stay (package linkage and logical nets need them); `RefDes/@populate` is `"false"` for a non-fitted instance. Paste features of a `dnp` instance are omitted wherever the document carries paste. |
| `.enet` | `Add into BOM` is `"no"` for a non-fitted instance. `Convert to PCB` stays `"yes"`; nets are unchanged. |
| `-cpl.csv` (neutral) | RFC-034 §3.1's "machine-placed instance" gains one condition: population `fitted`. Non-fitted instances are never rows. |
| `-<ID>-bom.csv`, `-<ID>-cpl.csv` | Fitted instances only. Entry selection (RFC-034 §3.2) is not performed for a non-fitted instance. |
| `-<ID>-map.json` | `designators` lists fitted machine-placed instances only. A new additive key `unfitted`, present only when non-empty, maps each non-fitted designator whose footprint has a numbered SMD pad to `"dnp"` or `"board_feature"`. `tools/smt_pos.py --map` needs it: its coverage check rejects a board footprint that is "not in this map", and with this key it accepts those footprints and writes no row for them. |
| Quilter CSVs (RFC-027) | Unchanged. They describe copper and placement physics; a DNP bypass site is still a site. |
| `design.lock`, `cohdl.lock` | Unchanged. |
| `cohdl docs` JSON | A part item gains `board_feature: true` only when marked (additive). |
| `build --json` | The `build` object gains `population` (path), present only when the file is written. |

`tools/smt_pos.py`'s legacy default output keeps its `TP…`/value heuristics unchanged; `--neutral` and `--map` follow this RFC.

### 6. E814 and E815

Both are RFC-034 warnings about an incomplete order, so both consider fitted instances only.

- **E814** (no binding for `--supplier ID`): one warning per part declaration that has at least one *fitted* instance without an eligible binding, naming only its fitted designators. A part whose every instance is non-fitted never reports. The "no entry of any instantiated part binds `ID`" form likewise considers only parts with a fitted instance.
- **E815** (unplaced machine-placed instances): counts fitted instances only.
- A `dnp` instance with no `place` is not reported by either. It is an unplaced footprint like any other, which is the layout tool's concern, and `--emit kicad_pcb` stages it as it does today.
- A design whose placed instances are all non-fitted still gets the header-only CPL, by RFC-034's existing rule.

### 7. Worked example

An illustrative board of 100 placed instances. Three sites are not fitted: C10 and C11, the two elements of one array declaration of tuning shunts in a subdesign, and R20, a bench substitute that shares its part, and today its BOM row, with the fitted R18 and R19. It has four test points (TP1–TP4) and two non-plated mounting holes (MH1, MH2).

Source changes: two `#[dnp]` lines (one on the array declaration, one on R20's `inst`), and `#[board_feature]` on `misc::TEST_POINT_ROUND_1MM` (a `misc` release) and on the project's own mounting-hole part.

| | Today | With this proposal |
|---|---|---|
| Neutral and `lcsc` BOM | 9 extra designators in rows | C10, C11, R20, TP1–TP4, MH1, MH2 absent; `R18,R19,R20` becomes `R18,R19` |
| Neutral and `lcsc` CPL | 98 rows each (the holes have no SMD pad) | 91 rows each (the seven SMD-pad sites are gone) |
| `-population.csv` | none | 9 rows: 3 `dnp`, 6 `board_feature` |
| `.kicad_pcb` | all fitted | three footprints `dnp` with no paste; six excluded from position files and BOM |
| Designators, nets, DRC verdict | — | identical |
| A project's side file and script | applied after every build | deleted; a "none reaches an order file" check becomes a comparison against `-population.csv` |

## Type-system-first test

No `rule` is proposed. Population is not a correctness property of connectivity; it selects what assembly outputs list. The checks it adds (E-TBD-1…3) are structural and local to one declaration.

## Conceptual impact

**Medium.** One new concept, Population, attached to Instance (per declaration) and Part (per kind). It overlaps no existing name: `nc` is a pin fact (RFC-002), "unplaced" is a layout fact (RFC-013), "unbound" is a supplier fact (RFC-034). The canonical vocabulary gains "fitted", "DNP" and "board feature".

It is not `#[virtual]` (rejected in RFC-032): a non-fitted instance is never stripped. It keeps its designator, footprint, pins, nets and checks, and only assembly-facing lists change.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Med | Low | Low | Low | Med | Low | High |

- **Netlist (Med):** two component properties in `.net` and one property value in `.enet`, only for marked instances. Connectivity bytes are untouched.
- **Trust (High):** this decides what is ordered and placed. It is addressed by the single predicate (§1), by `-population.csv` making every omission visible, and by conformance tests that pin each artifact in §5 for one fitted, one `dnp` and one `board_feature` instance, top and bottom.

## Gradeability

- E-TBD-1…3 at type-check, with the span of the attribute.
- Emitters: one shared function answers "fitted?" from the IR; no emitter reads the attribute or the part marker itself. A test asserts that the union of the BOM's designators and `-population.csv`'s designators is exactly the design's designator set, and that the two are disjoint.
- A regression test builds both examples without the construct and compares every artifact byte for byte.

## AI-generatability

High. One bare attribute on the line above the instance it describes, in the same shape as `#[designator]` and `#[bga_fanout]`. Meaning is local: no path, no designator, no second file. The part marker is written once by a library author.

## Alternatives

1. **A side file applied by scripts** (today). Rejected: keyed by designator, unchecked, invisible in the source, and different in every project.
2. **An export flag** (`cohdl build --dnp C10,C11`, or a supplier-level exclusion list). Rejected: population would then be a property of an invocation, so the same source gives different order files; it is keyed by allocator output; and a per-supplier list would let two suppliers' files disagree about what the board carries.
3. **A statement with instance paths in the design body** (`dnp matching.shunt[0], power.r_bench`). It would address single elements and single uses. Not chosen for the first step: it needs reach-in into subdesign internals, for which RFC-032 admits placement as "the sole admitted exception"; it adds a statement keyword where an attribute suffices (the RFC-027 correction); and its meaning is not local to the instance. It is the natural shape for per-use overrides and variants later.
4. **Only a part-level "not for assembly" property.** Rejected as the whole answer: an unfitted site is often an ordinary part, such as a `passive::R_10K_F_0402`, that other sites on the same board fit. It is kept as the second source (§3) for the cases where the property really is the part's.
5. **Only `#[dnp]`, with test points marked `#[dnp]` at each instance.** Rejected: ten identical attributes per board, on every board, for a fact the part already knows; and a test point would get KiCad's DNP rendering and paste removal, which describe an empty component site, not a copper feature.
6. **Strip non-fitted instances from the netlist.** Rejected: the board would lose the footprint, which is the opposite of what a DNP site is.
7. **A `DNP` column in the existing BOM.** Rejected: it changes the header, and so every existing BOM's bytes.

## Compatibility

Additive. `#[dnp]` and `#[board_feature]` are new attribute names in existing attribute positions; no keyword is reserved. A design that uses neither, built against packages that use neither, emits byte-identical artifacts and writes no new file. Library packages that mark test points and mounting holes change their consumers' BOM and CPL rows and publish that as a new version with a migration note.

## Tooling & operations

- `fmt`: canonical attribute placement. LSP: completion for both names, hover showing an instance's population.
- `check --json`: three new error codes; no schema change.
- Explorer and registry API docs may show population; neither is required for acceptance.
- Reversible: deleting the attribute restores the previous bytes.

## Teaching cost

Low. "`#[dnp]` on an `inst` leaves the site empty; the footprint and nets stay. Parts that are only copper or holes are marked by their library." One paragraph in note 10.

## Failure modes

- **`#[dnp]` inside a reusable subdesign or `fn` marks every use.** That is the stated meaning, and `-population.csv` lists each resulting designator. A consumer who needs one use fitted cannot say so yet (Non-goals).
- **A DNP site hides a D003, or keeps a D004** (§4). Stated; variants are the real fix.
- **A library marks a purchasable part `#[board_feature]`.** E-TBD-3 catches the cases with an `alt` or a binding; the rest is library review.
- **Paste removed where the assembler wanted it** for later hand-fitting. See Open questions, 3.
- **Old `smt_pos.py` with a new map.** It fails closed on the coverage check ("not in this map") for a DNP footprint on the board.

## Open questions and decisions requested

1. **Variants.** Is a single population per design the right first step, with variants as a later RFC? Recommended: yes. The sketch for later is named variants declared in the source and selected at export, which keeps "same source and same selection give the same bytes" and reuses alternative 3's path form.
2. **DRC and population.** Accept "all four rules as if fitted" (§4) for the first step, including the two stated D003/D004 limitations?
3. **Paste on DNP pads.** Removed unconditionally (recommended), kept, or author-selectable?
4. **`-population.csv`.** Accept the new artifact and its name, or record non-fitted instances only in the board and IPC-2581?
5. **Board-feature part identity.** Keep the placeholder `mfr`/`mpn` for now (recommended), or let a `#[board_feature]` part omit `mpn` in this RFC?
6. **Scope of this RFC.** Accept `#[dnp]` and `#[board_feature]` together as one model (recommended), or `#[dnp]` first?
7. **Names.** `#[dnp]` / `#[board_feature]`, and "fitted / DNP / board feature" as canonical vocabulary.
8. **`#[dnp]` on a subdesign instance.** Error in the first step (recommended), or "every component inside"?

## Migration path

None for the language. A project with a population side file moves each `dnp` entry to an attribute on the declaration that owns the designator (found through `design.lock`), updates to library versions that mark their board features, and compares the new order files with the ones its script produced.

## Decision

**None — proposal.** Awaiting the maintainer's decisions on the eight points above, then central allocation of the RFC number, decision record and error codes.
