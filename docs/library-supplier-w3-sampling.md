# Sampled inference for chip resistors and MLCCs (W3)

Status: **accepted 2026-11-09 by Tony, for the non-polar chip resistors and MLCCs of `lib/passive` only** ([decision](https://github.com/conol-ai/cohdl/pull/79#issuecomment-6734040188)). It does not apply to polar or multi-pin parts. Group bindings come from the generator under this rule, through normal PRs. On 2026-11-10 Tony decided that a sample whose model is not drawn does not block its group. The decisions set the scope and that; the numbers in §5 (sample limits, members with no stock, rechecks, whether a part number not drawn may be inferred) are proposed and **await Tony's confirmation**. [中文](library-supplier-w3-sampling.zh-CN.md)

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
  - `members`, every member part number; `zero_stock`, those with no stock on the confirmed page the record names; and `not_drawn`, those whose model a preview did not draw;
  - `samples`, each `{pn, row, angle_deg, offset_limit_mm, raw_offset_xy_mm, raw_offset_mm, bias_xy_mm, residual_offset_mm}`: its own frame row id and its reading under the rule. The raw offset is the body centre's board-frame offset as measured; the bias is the median taken off (§5); the residual is what is left.
- Both checkers enforce it, `tests/library/supplier.rs` in CI and `tools/sourcing/bindings.py`, which generates no binding from a row that fails: the record's rule is the accepted one; the group is `active`, in scope and keyed on the row's own (package, version, footprint); its land files still hash as recorded; the row's part number is a member, not a sample and (under §5) neither out of stock nor listed as not drawn; there are enough distinct samples; every sample is a member with its own `evidence` row under the same key with mapping (0, 0, 0) and a reading within the sample limits (angle, raw offset and residual, each against its own limit, and the raw size and residual what the vectors give); and the row's mapping is (0, 0, 0). An inferred row never carries an exception. `tests/fixtures/library-ci/s-sampled/` has a passing record and one counterexample per check.
- A sampled row is never described as previewed. The package README and the generated docs list sampled bindings apart from previewed ones.

### 3.4 Sampling

The rule is the plan's: at least 3, or 5% of the members, whichever is larger. It is a sampling policy, not proof that a group shares one model.
- Samples are chosen deterministically and spread through the group's members, sorted by MPN, so the values, tolerances, dielectrics and voltages the MPN encodes are all drawn on.
- Only members with at least 30 pieces in stock are chosen. The assembler builds at least five boards and adds attrition: on 2026-11-07 the pilot's BOM step confirmed no sample whose page read fewer than 20 pieces, and every one with 21 or more. 30 leaves room for stock sold since the page was read.
- Members with own evidence count toward the sample once a preview record judges them under the rule (§5). Three frame rows sit on records that cannot be judged yet (the C1 bottom preview and C2 board 1 kept no `board.json`), so they do not count.
- A group whose in-stock members cannot make up its sample cannot be inferred, and its members need their own evidence. Today there are three: Samsung `CL` 0402 and Murata `GRM` 0402, with 2 members each, and Vishay `CRCW` 0603, whose 15 members all read no stock.

### 3.5 When a sample fails

One sample outside the sample limits (angle, raw offset or residual), left for manual review, or unverified for any reason other than a model not drawn **pauses the whole group**:
- No new row infers from it.
- The next release drops the group's sampled bindings.
- The group is then split (for example by tolerance or dielectric code), with a stated reason and fresh samples, or verified part by part.

A sample whose model is not drawn (bare pads and the centroid marker) does **not** pause its group (Tony, 2026-11-10). It is not a passed sample either: the group still needs its required number of passed, drawn samples. The group record lists that part number under `not_drawn`, and under the proposed §5 it is never inferred; CI and the generator reject a `sampled` row for it. This revises the review's position that a body not drawn is a group failure; the review's worry, that a group passes by resampling around holes in the assembler's library, is met only in part: the holes themselves stay unbound, but the members around them can be inferred.

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

Tony's comments on #79: the [review](https://github.com/conol-ai/cohdl/pull/79#issuecomment-6731830774) and the [decision](https://github.com/conol-ai/cohdl/pull/79#issuecomment-6734040188); and his 2026-11-10 decision on samples not drawn (没画出模型的样本不阻塞整组). `tools/sourcing/w3_rule.py` holds the rule, version `w3-2026-11-10b`; `tests/library/supplier.rs` holds the same.

**Decided by Tony**

| Decision | When | In the rule |
|---|---|---|
| Scope: non-polar chip resistors and MLCCs of `lib/passive` only | 2026-11-09 | `types`: `ChipResistor`, `MLCC` |
| The review's statistical concerns do not block chip R/C | 2026-11-09 | at least 3 or 5% of the members |
| A sample whose model is not drawn does not block its group | 2026-11-10 | it is not a passed sample and does not pause the group (§3.5) |

**Proposed, for Tony to confirm**

| Question | Proposed | Alternative |
|---|---|---|
| Angle limit | 10° mod 180, as before | 7.5°: two RC 0402 samples fail on scatter alone (below) |
| Raw offset | within **1.0** of the per-part limit, nothing taken off (`offset_raw_fraction`): the pilot's per-part pass line | a tighter fraction; at 0.75, 15 of the 44 RC pilot samples fail |
| Residual offset | within **0.5** of the per-part limit once the view's median is taken off (`offset_residual_fraction`) | — |
| What the residual is taken against | the median of every drawn chip body in the view (`board_median`) | the median per land and view (below) |
| Members with no stock | not inferred (`infer_zero_stock`), as the review recommended | inferred |
| A part number whose model is not drawn | not inferred (`infer_not_drawn`); CI and the generator reject a `sampled` row for it | inferred: Tony's decision covers the group, not this |
| Rechecks | one fresh sample per group at each quarterly snapshot refresh; at once if a production assembly shows an offset | — |

**Why two offset checks.** The residual alone proves that a sample agrees with the other bodies of its view, not that it is centred. A reviewer's counterexample: 49 samples all off by 0.12 mm against a 0.127 mm limit leave a residual of 0 and pass. The raw check bounds that: the same 49 pass at 0.12 mm and fail at 0.13 mm (`tools/tests/test_w3_groups.py`, and the `raw` counterexample in `tests/fixtures/library-ci/s-sampled/`). Every sample in a group record and in the grouping report carries its raw offset (vector and size), the bias taken off, and the residual, and both checkers recompute the arithmetic.

**The median as the export's error is a hypothesis.** For it: the pilot export draws every body about 0.077 mm off along board −y (median of 88 drawn bodies; Yageo `CC` 0.061, Yageo `RC` 0.079, Vishay `CRCW` 0.067), with 0.01–0.03 mm spread about it, a shift shared by two makers and both part types. Judged raw, 15 of the 44 RC samples would fail a limit three quarters of the old one; judged against the median, every pilot sample is within 0.063 mm, half the old limit. The re-measure is `remeasure-2026-11-10.json` in each record, `measure_preview.py` run again on the committed exports: identical to the original readings apart from the added `offset_xy_mm`. What it does not prove: every pilot body sits at 0° on the one `CHIP_0402` land, so a land-level offset that every maker's model shares (the assembler's model origin against our land) looks exactly the same as an export shift. The export's shift is not even stable: C2 board 3's 0402 bodies, at 0° too, sit at a median of (−0.011, −0.047) mm against the pilot's (0.005, −0.077).

**What the existing data says (calibration).** An export's shift stays fixed in the board frame whatever the part's rotation; a model's offset turns with the part. Only C2 board 3 has the same part number at two rotations: 0201, 1210, 2010 and 2512 parts at 0° and 37° on top and at 180° and −143° on the bottom, 37° apart in each view. For each pair, the table gives how far apart the two readings are (an export shift predicts 0) and how far the second is from the first turned by 37° (a model offset predicts 0). Readings of one part number scatter by about 0.01–0.02 mm.

| Land | Part | View | First reading (mm) | Second, 37° on (mm) | Misfit as export | Misfit as model |
|---|---|---|---|---|---:|---:|
| `CHIP_2010` | C877226 | top | (−0.036, −0.173) | (−0.142, −0.125) | 0.116 | **0.012** |
| `CHIP_2010` | C877226 | bottom | (−0.053, −0.163) | (−0.155, −0.136) | 0.106 | **0.040** |
| `CHIP_2512` | C878096 | top | (+0.023, −0.149) | (−0.016, −0.135) | **0.041** | 0.055 |
| `CHIP_2512` | C878096 | bottom | (+0.059, −0.149) | (−0.026, −0.118) | 0.090 | **0.040** |
| `CHIP_0201` | C138161 | top | (−0.038, +0.032) | (−0.037, +0.029) | **0.003** | 0.032 |
| `CHIP_0201` | C138161 | bottom | (−0.046, +0.035) | (−0.054, +0.045) | **0.013** | 0.040 |
| `CHIP_0201` | C541229 | top | (−0.010, +0.009) | (−0.007, +0.004) | 0.006 | 0.010 |
| `CHIP_0201` | C541229 | bottom | (−0.015, +0.016) | (−0.024, +0.015) | 0.009 | 0.023 |
| `CHIP_0201` | C718246 | top | (−0.027, +0.011) | (−0.011, +0.012) | 0.016 | 0.014 |
| `CHIP_0201` | C718246 | bottom | (−0.046, +0.012) | (−0.031, +0.016) | 0.016 | 0.021 |
| `CHIP_1210` | C309589 | top | (+0.005, −0.038) | (−0.015, −0.044) | 0.021 | 0.011 |
| `CHIP_1210` | C309589 | bottom | (−0.015, −0.033) | (−0.047, −0.033) | 0.032 | 0.022 |

- The `CHIP_2010` offset of about 0.18 mm turns with the part in both views: it is the part's (its model, or the model's origin against our land), not the export's. Solved for both at once, the top view's export share is about (−0.017, +0.009) mm.
- `CHIP_2512` C878096 is mixed: the bottom pair turns, the top pair fits neither well.
- On 0201 and 1210 the offsets are too small for the two readings to differ by more than their scatter, so the data cannot tell. C138161, the 0201 sample that pauses Yageo `RC` 0201, leans towards a fixed (export-like) shift, yet the other two 0201 part numbers on the same board are not shifted alike.
- There is no rotated `CHIP_0402` reading on any record, so nothing in the repository can say whether the pilot's 0.077 mm is the export's or the land's.

This does not change any number in the rule. It is why the raw check is there: on the one land where a large shared shift could be tested, it belonged to the part.

**Proposed calibration board (for Tony).** One frame board on which each of a few part numbers is placed at 0°, 90°, 180° and 270° and one odd angle, on both sides, spread across the board: three Yageo `RC` and two Yageo `CC` 0402 part numbers that the pilot drew, two drawn Vishay `CRCW` 0402, and C877226, C878096 and C138161 again. Each view's readings then fit "export shift + the part's own offset turned by its angle" with the part's offset over-determined. If the 0402 shift is the export's, the median correction stands. If every maker's 0402 shares it, the offset is between the assembler's land and ours, a placement offset to fix in the mapping, not something to take off. The upload needs approval like every preview.

**Board median or per land.** On board 3, bodies of one land shift alike across part numbers: about 0.05 mm on 0402, 0.12–0.17 mm on 2010 and 2512, and about 0.1 mm along the pads on the 0805 `CC` models. A median per land and view would clear Yageo `RC` 2010 and 2512 and Yageo `CC` 0805 on the residual, but a series that fills a land on its board would then hide its own shift, and the 2010 shift is now shown to be the part's. The board median stands unless Tony chooses otherwise.

**Why the angle limit stays.** A chip model is wrong by a quarter turn, not by a few degrees, and single readings scatter with a standard deviation of 3.4°. At 7.5° two RC samples would fail on scatter alone (C138033 at 9.9°, C185445 at 8.1°), and the group would pause without a fault.

**What the rule gives today** (`w3_sampling/2026-11-10/`): 1 group sampled, 14 need samples, 5 paused.

| Group | Status | Why |
|---|---|---|
| `CHIP_0402` Yageo `RC` | **sampled**, 49 of 49 | 44 pilot and 5 board 3 samples; all within the raw limit (largest C185445, 0.121 of 0.128 mm, 95%) and the residual (largest 0.059 mm, 46%) |
| `CHIP_0402` Vishay `CRCW` | needs samples, 40 of 49 | 9 not drawn: no longer a pause, but they do not count and are not inferred; 9 more samples chosen |
| `CHIP_1210` Yageo `RC` | needs samples, 1 of 43 | C874860 not drawn: no longer a pause |
| `CHIP_0402` Yageo `CC` | paused | C113781 left for manual review (+10.5°); C576731 not drawn does not pause it, but the manual review does |
| `CHIP_0201` Yageo `RC` | paused | C138161 residual 0.046–0.064 mm against 0.040 (raw 0.047–0.070, within 0.080) |
| `CHIP_0805` Yageo `CC` | paused | C527232 and C576985 residual 0.124 and 0.136 mm against 0.114 (raw within) |
| `CHIP_2010` Yageo `RC` | paused | C877226 residual 0.150–0.171 mm against 0.125 (raw 0.171–0.206, within 0.250) |
| `CHIP_2512` Yageo `RC` | paused | C878096 residual up to 0.169 mm against 0.125 (raw within); C878293 not drawn |

These parts keep their own bindings: a paused group only stops inference. No sample of any group fails the raw check today; the raw check changes no outcome on the current records.
