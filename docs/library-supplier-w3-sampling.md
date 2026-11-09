# Sampled inference for chip resistors and MLCCs (W3)

Status: **accepted 2026-11-09 by Tony, for the non-polar chip resistors and MLCCs of `lib/passive` only** ([decision](https://github.com/conol-ai/cohdl/pull/79#issuecomment-6734040188)). It does not apply to polar or multi-pin parts. Group bindings come from the generator under this rule, through normal PRs. The decision set the scope; the numbers in §5 (sample limits, members with no stock, rechecks) are proposed and **await Tony's confirmation**. [中文](library-supplier-w3-sampling.zh-CN.md)

The rule is `tools/sourcing/w3_rule.py`; `tests/library/supplier.rs` holds the same numbers and rejects a group record that carries any other rule. Sections 1–4 are the proposal as reviewed, with §3 updated to what was built.

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

`tools/w3_groups.py` builds the groups. The proposal's report is `tools/sourcing_data/w3_sampling/2026-11-07/` (`passive` 0.2.5, before the pilot): 19 groups, from 2 members (Samsung `CL` 0402) to 969 (Yageo `RC` 0402, 0603, 0805). The current one is `2026-11-10/` (`passive` 0.2.6, the rule of §5 and the pilot's readings): 20 groups.
- A group's land is pinned by the SHA-256 of the files that define it (`lib/passive/src/footprints.cohdl` and `pads.cohdl`). A version that changes no land keeps its readings; any change to those files lapses every reading taken on them.

### 3.3 A new evidence kind: `sampled`

A frame row borrows a group's samples through its own kind, never through `same_supplier_footprint`. Proposed schema change, for review:

- `jlc_frames.json` rows gain a third evidence field, `sampled`, optional: a row without it reads as `sampled: null`. Exactly one of `evidence`, `evidence_ref` or `sampled` is non-null.
- `sampled` is `{group, record, note}`. `record` is a committed group record (`schema_version` 1) holding `rule`, the whole rule it was made under, and `groups`, each with:
  - `id`, the key (`package`, `package_version`, `footprint`, `type`, `mfr`, `series`) and `status` (`active` or `paused`);
  - `land`, the land files and their SHA-256;
  - `members`, every member part number, and `zero_stock`, those with no stock on the confirmed page the record names;
  - `samples`, each `{pn, row, angle_deg, offset_mm, offset_limit_mm}`: its own frame row id and its reading under the rule.
- Both checkers enforce it, `tests/library/supplier.rs` in CI and `tools/sourcing/bindings.py`, which generates no binding from a row that fails: the record's rule is the accepted one; the group is `active`, in scope and keyed on the row's own (package, version, footprint); its land files still hash as recorded; the row's part number is a member, not a sample and (under §5) not out of stock; there are enough distinct samples; every sample is a member with its own `evidence` row under the same key with mapping (0, 0, 0) and a reading within the sample limits; and the row's mapping is (0, 0, 0). An inferred row never carries an exception. `tests/fixtures/library-ci/s-sampled/` has a passing record and one counterexample per check.
- A sampled row is never described as previewed. The package README and the generated docs list sampled bindings apart from previewed ones.

### 3.4 Sampling

The rule is the plan's: at least 3, or 5% of the members, whichever is larger. It is a sampling policy, not proof that a group shares one model.
- Samples are chosen deterministically and spread through the group's members, sorted by MPN, so the values, tolerances, dielectrics and voltages the MPN encodes are all drawn on.
- Only members with at least 30 pieces in stock are chosen. The assembler builds at least five boards and adds attrition: on 2026-11-07 the pilot's BOM step confirmed no sample whose page read fewer than 20 pieces, and every one with 21 or more. 30 leaves room for stock sold since the page was read.
- Members with own evidence count toward the sample once a preview record judges them under the rule (§5). Three frame rows sit on records that cannot be judged yet (the C1 bottom preview and C2 board 1 kept no `board.json`), so they do not count.
- A group whose in-stock members cannot make up its sample cannot be inferred, and its members need their own evidence. Today there are three: Samsung `CL` 0402 and Murata `GRM` 0402, with 2 members each, and Vishay `CRCW` 0603, whose 15 members all read no stock.

### 3.5 When a sample fails

One sample outside the chip limits (angle, offset, or a body that is not drawn) **pauses the whole group**:
- No new row infers from it.
- The next release drops the group's sampled bindings.
- The group is then split (for example by tolerance or dielectric code), with a stated reason and fresh samples, or verified part by part.

A body that is not drawn or that fails to measure is a failure, not a skip.

### 3.6 Members with no stock

A member with no stock can be in a group. It is never chosen as a sample, and the group record lists it under `zero_stock`. Under the proposed §5 it is not inferred: it gets a binding once it has stock and is inferred from a later record, or its own preview.

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

## 5. Decisions and proposed numbers

Tony's two comments on #79: the [review](https://github.com/conol-ai/cohdl/pull/79#issuecomment-6731830774) and the [decision](https://github.com/conol-ai/cohdl/pull/79#issuecomment-6734040188). The decision accepts the rule for chip R/C and says the statistical concerns of the review do not block it there. It gives no numbers for questions 2–4. The numbers below are proposed for his confirmation; `tools/sourcing/w3_rule.py` holds them.

| Question | Status | Rule `w3-2026-11-10` |
|---|---|---|
| 1. Sample size | Decided: the concerns do not block chip R/C | at least 3 or 5% |
| 2. Members with no stock | Proposed, as the review recommended | not inferred |
| 3. Rechecks | Proposed | one fresh sample per group at each quarterly snapshot refresh; at once if a production assembly shows an offset |
| 4. Tighter sample limits | Proposed: the review said yes, the decision gave no number | angle 10° mod 180 as before; offset within half the per-part limit, after the view's median offset is taken off |
| Not drawn | Decided in the review: a group failure | pauses the group (§3.5) |

**Why the offset is judged against the view's median.** The pilot export draws every body about 0.077 mm low (median of 88 drawn bodies; Yageo `CC` 0.061, Yageo `RC` 0.079, Vishay `CRCW` 0.067), with 0.01–0.03 mm spread about it. A shift shared by two makers and both part types is the export's, not a model's. Judged raw, 15 of the 44 RC samples would fail a limit three quarters of the old one; judged against the median, every pilot sample is within 0.063 mm, half the old limit. The re-measure is `remeasure-2026-11-10.json` in each record, `measure_preview.py` run again on the committed exports: identical to the original readings apart from the added `offset_xy_mm`.

**Why the angle limit stays.** A chip model is wrong by a quarter turn, not by a few degrees, and single readings scatter with a standard deviation of 3.4°. At 7.5° two RC samples would fail on scatter alone (C138033 at 9.9°, C185445 at 8.1°), and the group would pause without a fault.

**What the rule gives today** (`w3_sampling/2026-11-10/`):
- `CHIP_0402` Yageo `RC` is the only sampled group: 49 of 49 required samples pass (44 from the pilot and 5 from C2 board 3).
- 7 groups are paused:
  - Vishay `CRCW` 0402: 9 bodies not drawn.
  - Yageo `CC` 0402: 1 body not drawn, and C113781 left for manual review.
  - On C2 board 3, a body not drawn pauses Yageo `RC` 1210 and 2512.
  - Offsets above half the limit pause Yageo `RC` 0201 (C138161), 2010 (C877226) and 2512 (C878096), and Yageo `CC` 0805 (C527232, C576985).
- The board 3 offsets show the median is a coarse correction. Bodies of one land shift alike across makers: about 0.07 mm on 0402, 0.12–0.17 mm on 2010 and 2512, and about 0.1 mm along the pads on the 0805 `CC` models. Taking the median per land and view instead would clear those groups, but a series that fills a land on its board would then hide its own shift. Tony should choose; until then, the stricter board median stands.
- These parts keep their own bindings: a paused group only stops inference.
