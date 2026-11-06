# Proposal: sampled inference for chip resistors and MLCCs (W3)

Status: **draft, for review by Tony.** It changes nothing yet. `docs/library-supplier-plan.md` §W3 stays as written until this is accepted. No binding is generated from it, and `bindings.py` and `tests/library/supplier.rs` accept no new evidence type. [中文](library-supplier-w3-sampling.zh-CN.md)

## 1. Why

W3 lets a part number borrow another's frame evidence for one of two reasons. Neither can carry the mapped chip passives.

1. **"The supplier shows the same footprint identifier."** The part pages do not show one. Checked on 2026-11-07 against the fetch cache of the B2 run: a page's data has `componentSpecificationEn` (`0402`, `SOT-23-5`), the package text the catalog stores. The footprint drawing is loaded from elsewhere (`/user-center/lcsvg/svg.html`) and carries no identifier on the page. The package text is not an identifier:
   - C7972 and C7722 both read `SOT-23-5`, but their current mappings are rotate 180 and rotate 270.
   - The three `KICAD_LQFP_32` rows are *not* a counterexample. All three have mapping 0 now. C8254's 270 is its `measured` reading from before `qfp` moved that land.

   The fetch policy rules out the other places an identifier might be found (EasyEDA, `/api`), and this proposal does not ask to change it.
2. **"Both were checked on the same preview."** That is each part's own evidence.

So today every part number needs its own preview.
- The 9,629 mapped chip-resistor and MLCC entries of `passive` 0.2.5 would need 9,629 previews.
- 3,442 of them read no stock on their confirmed page. The assembler does not place a part without stock, so these can never be bound.

## 2. What the evidence shows so far

`jlc_frames.json` has 43 `passive` rows with their own preview evidence, on three boards (C1, C2 board 1, C2 board 3). Every one reads (0, 0, 0): the body's long axis lies within 10° of the land's axis (mod 180), and its centre lies within the chip offset limit (`measure_preview.py`). Of the 43, 36 are entries of `passive` 0.2.5 that the map maps; the other 7 are B1 parts on the same lands that the library does not list.

That supports a pilot. It does not cover 9,629 part numbers. Being non-polarised removes only the 0°/180° ambiguity. A quarter turn or an offset in one part's model would still show, and 43 readings across 19 groups cannot rule one out.

## 3. Proposal

### 3.1 Scope

Plain, non-polarised, two-terminal chip resistors and MLCCs only: the `ChipResistor` and `MLCC` parts of `lib/passive`. It does not cover:
- polarised parts (tantalum and polymer capacitors, diodes, LEDs);
- arrays and networks;
- inductors and ferrite beads;
- any other package.

### 3.2 Groups

A group is (footprint-owning package, its version, footprint, part type, manufacturer, series). The series is the MPN's leading letters: Yageo `RC` and `CC`, Vishay `CRCW`, Samsung `CL`, Murata `GRM`.
- The page's package text is never part of the key, and two parts are never grouped on it alone.
- Two series are not merged. A future merge needs a written basis, for example the manufacturer's statement that both series share one body and termination drawing, and its own review.
- A footprint move (new package version) starts new groups.

`tools/w3_groups.py` builds the groups. Its report is `tools/sourcing_data/w3_sampling/2026-11-07/`: 19 groups, from 2 members (Samsung `CL` 0402) to 969 (Yageo `RC` 0402, 0603, 0805).

### 3.3 A new evidence kind: `sampled`

A frame row borrows a group's samples through its own kind, never through `same_supplier_footprint`. Proposed schema change, for review:

- `jlc_frames.json` rows gain a third evidence field, `sampled`. Exactly one of `evidence`, `evidence_ref` or `sampled` is non-null.
- `sampled` is `{group, record, note}`. `record` is a committed group record holding:
  - the group key and every member part number;
  - the rule version, and the sampling method with its seed or ordering;
  - the sample part numbers, each with its own frame row id;
  - the status (`active` or `paused`).
- CI checks that:
  - the row's part number is a member;
  - every sample has its own `evidence` row on the same (package, version, footprint) with mapping (0, 0, 0);
  - the sample count meets the rule;
  - the group is `active`;
  - the row's mapping is (0, 0, 0). An inferred row never carries an exception.
- A sampled row is never described as previewed. The package README and the generated docs list sampled bindings apart from previewed ones.

### 3.4 Sampling

The draft rule is the plan's: at least 3, or 5% of the members, whichever is larger. It is a sampling policy under review, not proof that a group shares one model.
- Samples are chosen deterministically and spread through the group's members, sorted by MPN, so the values, tolerances, dielectrics and voltages the MPN encodes are all drawn on.
- Only members with at least 5 pieces in stock are chosen (the assembler builds at least five boards).
- Members with own evidence count toward the sample.
- A group whose in-stock members cannot make up its sample cannot be inferred, and its members need their own evidence. Today there are three: Samsung `CL` 0402 and Murata `GRM` 0402, with 2 members each, and Vishay `CRCW` 0603, whose 15 members all read no stock.

### 3.5 When a sample fails

One sample outside the chip limits (angle, offset, or a body that is not drawn) **pauses the whole group**:
- No new row infers from it.
- The next release drops the group's sampled bindings.
- The group is then split (for example by tolerance or dielectric code), with a stated reason and fresh samples, or verified part by part.

A body that is not drawn or that fails to measure is a failure, not a skip.

### 3.6 Members with no stock

A member with no stock can be in a group and can be inferred once the group is `active`. It is never chosen as a sample, and the group record lists it under `zero_stock`. Whether such a binding should ship at all is an open question (§5).

## 4. Pilot

One frame board of 0402 parts, which tests both part types and two manufacturers on one land:

| Group | Members | Own evidence | Samples to add |
|---|---:|---:|---:|
| `CHIP_0402` ChipResistor Yageo `RC` | 969 | 5 | 44 |
| `CHIP_0402` ChipResistor Vishay `CRCW` | 968 | 2 | 47 |
| `CHIP_0402` MLCC Yageo `CC` | 199 | 2 | 8 |

That makes 99 placements, top side at 0°, built and previewed like C2 board 3. The upload needs Alex's approval, as every preview does.

**Pass:** every sample reads (0, 0, 0) within the chip limits. Then each sample gets its own `evidence` frame row and the three group records are written. Tony then reviews this proposal with the pilot's numbers.

**Fail:** the failing group is paused and its readings are recorded. The proposal is revised before any other group is sampled.

Until Tony accepts the proposal, no `sampled` row is written and no binding is generated from a group.

## 5. Open questions for review

1. Is the 3-or-5% policy acceptable, or should large groups use a fixed count with a stated confidence?
2. Should a member with no stock be bound by inference, or only once it has stock and its own preview?
3. When do groups get rechecked? Proposed: one fresh sample per group with each quarterly snapshot refresh, and at once if a production assembly shows an offset.
4. Should the chip limits used for samples be tighter than those for parts with their own evidence?
