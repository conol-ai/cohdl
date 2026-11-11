# Sampled inference for chip resistors and MLCCs (W3)

Status: **accepted 2026-11-09 by Tony, for the non-polar chip resistors and MLCCs of `lib/passive` only** ([decision](https://github.com/conol-ai/cohdl/pull/79#issuecomment-6734040188)). It does not apply to polar or multi-pin parts. Group bindings come from the generator under this rule, through normal PRs. On 2026-11-10 Tony decided that a sample whose model is not drawn does not block its group, and on 2026-11-11 he accepted the five values of §5 (angle, raw offset, residual, members with no stock, part numbers not drawn) as the W3 standard for chip R/C ([decision](https://github.com/conol-ai/cohdl/pull/104#issuecomment-6784835520)). The rule in the tree, `w3-2026-11-12`, keeps those values and adds three things that are proposed and **await Tony's decision**: the two conditions he asked for in that comment, with proposed numbers (§5), and how a chip body is read, the rule's `body_method` (§6). [中文](library-supplier-w3-sampling.zh-CN.md)

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

`tools/w3_groups.py` builds the groups. The proposal's report is `tools/sourcing_data/w3_sampling/2026-11-07/` (`passive` 0.2.5, before the pilot): 19 groups, from 2 members (Samsung `CL` 0402) to 969 (Yageo `RC` 0402, 0603, 0805). `2026-11-10/` (`passive` 0.2.6, rule `w3-2026-11-10b` and the pilot's readings) has 20 groups; it is the report Tony reviewed. The current one is `2026-11-11/`, the same 20 groups under rule `w3-2026-11-11` with the two conditions of §5. Each is a dated snapshot of the tree it was built from.
- A group's land is pinned by the SHA-256 of the files that define it (`lib/passive/src/footprints.cohdl` and `pads.cohdl`). A version that changes no land keeps its readings; any change to those files lapses every reading taken on them.

### 3.3 A new evidence kind: `sampled`

A frame row borrows a group's samples through its own kind, never through `same_supplier_footprint`. Proposed schema change, for review:

- `jlc_frames.json` rows gain a third evidence field, `sampled`, optional: a row without it reads as `sampled: null`. Exactly one of `evidence`, `evidence_ref` or `sampled` is non-null.
- `sampled` is `{group, record, note}`. `record` is a committed group record (`schema_version` 1) holding `rule`, the whole rule it was made under, and `groups`, each with:
  - `id`, the key (`package`, `package_version`, `footprint`, `type`, `mfr`, `series`) and `status` (`active` or `paused`);
  - `land`, the land files and their SHA-256;
  - `members`, every member part number; `zero_stock`, those with no stock on the confirmed page the record names; and `not_drawn`, those whose model a preview did not draw;
  - `samples`, each `{pn, row, body_method, angle_deg, offset_limit_mm, raw_offset_xy_mm, raw_offset_mm, bias_xy_mm, residual_offset_mm, bias_view, bias_bodies, bias_other_bodies}`: its own frame row id and its reading under the rule. `body_method` is the method the body was read by (§6). The raw offset is the body centre's board-frame offset as measured; the bias is the median taken off (§5); the residual is what is left. `bias_view` names the view the median came from (a preview record and a board side), `bias_bodies` how many drawn bodies it was taken over, and `bias_other_bodies` how many of those are not this group's.
- Both checkers enforce it, `tests/library/supplier.rs` in CI and `tools/sourcing/bindings.py`, which generates no binding from a row that fails: the record's rule is the accepted one; the group is `active`, in scope and keyed on the row's own (package, version, footprint); its land files still hash as recorded; the row's part number is a member, not a sample and (under §5) neither out of stock nor listed as not drawn; there are enough distinct samples; every sample is a member with its own `evidence` row under the same key with mapping (0, 0, 0) and a reading by the rule's body method (§6) within the sample limits (angle, raw offset and residual, each against its own limit, and the raw size and residual what the vectors give); every sample's bias comes from a named view whose median holds enough bodies of other groups (§5), the samples of one view give the same median and body counts, and no view has more samples than it has bodies of the group; the part numbers not drawn are not more than the rule's share of the judged members (the samples and those not drawn); and the row's mapping is (0, 0, 0). A record states the view's body counts; the checkers hold it to the rule and to itself, and `tools/w3_groups.py` recounts them from the preview records. An inferred row never carries an exception. `tests/fixtures/library-ci/s-sampled/` has a passing record and one counterexample per check.
- A sampled row is never described as previewed. The package README and the generated docs list sampled bindings apart from previewed ones.

### 3.4 Sampling

The rule is the plan's: at least 3, or 5% of the members, whichever is larger. It is a sampling policy, not proof that a group shares one model.
- Samples are chosen deterministically and spread through the group's members, sorted by MPN, so the values, tolerances, dielectrics and voltages the MPN encodes are all drawn on.
- Only members with at least 30 pieces in stock are chosen. The assembler builds at least five boards and adds attrition: on 2026-11-07 the pilot's BOM step confirmed no sample whose page read fewer than 20 pieces, and every one with 21 or more. 30 leaves room for stock sold since the page was read.
- A sample the assembler's BOM step finds short although its page read 30 or more is recorded as a dated fact (`short_at_preview`) and replaced by the nearest eligible member of its group in that order; the samples the same run confirmed stay. The fact lives in the preview record's `w3.json`, or, when the run stopped at the BOM step and took no preview, in the board's own `w3.json` under `tools/frame_boards/` until a record exists (the pilot on 2026-11-07, preview board 4 on 2026-11-11).
- Members with own evidence count toward the sample once a preview record judges them under the rule (§5). Three frame rows sit on records that cannot be judged yet (the C1 bottom preview and C2 board 1 kept no `board.json`), so they do not count.
- A group whose in-stock members cannot make up its sample cannot be inferred, and its members need their own evidence. Today there are three: Samsung `CL` 0402 and Murata `GRM` 0402, with 2 members each, and Vishay `CRCW` 0603, whose 15 members all read no stock.

### 3.5 When a sample fails

One sample outside the sample limits (angle, raw offset or residual), left for manual review, or unverified for any reason other than a model not drawn **pauses the whole group**:
- No new row infers from it.
- The next release drops the group's sampled bindings.
- The group is then split (for example by tolerance or dielectric code), with a stated reason and fresh samples, or verified part by part.

A sample whose model is not drawn (bare pads and the centroid marker) does **not** by itself pause its group (Tony, 2026-11-10). Too many of them do: under the proposed §5, a group whose part numbers not drawn are more than 10% of its judged members is paused. It is not a passed sample either: the group still needs its required number of passed, drawn samples. The group record lists that part number under `not_drawn`, and under §5 it is never inferred; CI and the generator reject a `sampled` row for it. This revises the review's position that a body not drawn is a group failure; the review's worry, that a group passes by resampling around holes in the assembler's library, is met only in part: the holes themselves stay unbound, but the members around them can be inferred, as long as the holes stay within that share.

A sample within the limits whose view does not hold enough bodies of other groups (§5) is **unsupported**: its residual was taken against its own group's median. It is not a passed sample and not a failure; the group needs that many more samples on a view that other groups share.

### 3.6 Members with no stock

A member with no stock can be in a group. It is never chosen as a sample, and the group record lists it under `zero_stock`. Under §5 it is not inferred: it gets a binding once it has stock and is inferred from a later record, or its own preview.

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

Tony's comments on #79: the [review](https://github.com/conol-ai/cohdl/pull/79#issuecomment-6731830774) and the [decision](https://github.com/conol-ai/cohdl/pull/79#issuecomment-6734040188); his 2026-11-10 decision on samples not drawn (没画出模型的样本不阻塞整组); and his [2026-11-11 comment on #104](https://github.com/conol-ai/cohdl/pull/104#issuecomment-6784835520), which accepts the five values below and asks for two more conditions. `tools/sourcing/w3_rule.py` holds the rule, version `w3-2026-11-12`; `tests/library/supplier.rs` holds the same. The version carries the accepted values (`w3-2026-11-10b`) unchanged, the two conditions below, and `body_method: 2` (§6).

Which reading the numbers in this section are from: the two conditions count bodies and part numbers, which do not depend on how a body is read, and their tables and the outcome table at the end of this section are under the rule as it stands (method 2, three records, `w3_sampling/2026-11-12/`). The paragraphs on why there are two offset checks, the median as a hypothesis, the calibration table, the per-land median and the angle limit are the record of how the accepted values were weighed on 2026-11-10 and quote method 1 readings; §6 says what became of those readings.

**Decided by Tony**

| Decision | When | In the rule |
|---|---|---|
| Scope: non-polar chip resistors and MLCCs of `lib/passive` only | 2026-11-09 | `types`: `ChipResistor`, `MLCC` |
| The review's statistical concerns do not block chip R/C | 2026-11-09 | at least 3 or 5% of the members |
| A sample whose model is not drawn does not block its group | 2026-11-10 | it is not a passed sample and does not by itself pause the group (§3.5) |
| Angle limit: 10° mod 180 | 2026-11-11 | `angle_limit_deg` |
| Raw offset: within **1.0** of the per-part limit, nothing taken off | 2026-11-11 | `offset_raw_fraction` |
| Residual offset: within **0.5** of the per-part limit once the view's median is taken off | 2026-11-11 | `offset_residual_fraction`, `offset_bias`: `board_median` |
| Members with no stock are not inferred | 2026-11-11 | `infer_zero_stock` |
| A part number whose model is not drawn is not inferred, and does not block | 2026-11-11 | `infer_not_drawn` |

The alternatives weighed for those five (a 7.5° angle limit, a tighter raw fraction, a median per land) are kept below as the record of why.

**Requested by Tony on 2026-11-11, with proposed numbers for him to confirm**

| Tony's condition | Proposed | In the rule | Alternative |
|---|---|---|---|
| For the residual: the view's median must include bodies from at least one other group; on a single-group board it only shows the group agrees with itself | A reading counts only when, among the drawn bodies its view's median is taken over, those of other groups number **at least 3** and are **more than half** | `bias_min_other_bodies`: 3, `bias_other_share_above`: 0.5 | a larger share: at two thirds the pilot view counts for neither Yageo `RC` (50.6%) nor Vishay `CRCW` (57.3%) |
| For samples not drawn: flag or pause a group whose not-drawn share exceeds about 10% of its judged members | **Pause** a group whose part numbers not drawn are more than **10%** of its judged members; while fewer have been judged than its required sample, of the required sample | `not_drawn_max_share`: 0.1, `not_drawn_excess`: `pause` | flag only: the group goes on being sampled and can be inferred; or the share of judged members alone, which also pauses Yageo `RC` 1210 at 1 of 2 |

Still proposed from before, not covered by the 2026-11-11 decision: one fresh sample per group at each quarterly snapshot refresh, and at once if a production assembly shows an offset.

**Other groups in the median (proposed numbers).** A sample of a group that holds half or more of its view's bodies can have the median sit on one of its own bodies, so the residual measures the group against itself. While the group's bodies are fewer than the others, at least one body of another group lies on each side of the median in each axis, and the group cannot move it past them: that is why the share is "more than half" and not a smaller number. The floor of 3 keeps a view of two or three bodies from counting. "Other group" means any drawn body on the view that is not a member of the sample's group, whatever its maker or land. What each existing view's median is made of (`w3_sampling/2026-11-12/groups.md`; the counts are the same by either method):

| View | Bodies in the median | Group | Own | Other groups |
|---|---:|---|---:|---:|
| Pilot, top | 89 | `CHIP_0402` Yageo `RC` | 44 | 45 (50.6%) |
| | | `CHIP_0402` Vishay `CRCW` | 38 | 51 (57.3%) |
| | | `CHIP_0402` Yageo `CC` | 7 | 82 (92.1%) |
| C2 board 3, top | 39 | twelve groups, 2 to 5 bodies each (`CHIP_0402` Yageo `RC`: 5) | 2–5 | 34–37 (87.2–94.9%) |
| C2 board 3, bottom | 12 | five groups, 2 to 4 bodies each | 2–4 | 8–10 (66.7–83.3%) |
| Board 4, top | 120 | ten groups, 2 to 28 bodies each, and 20 bodies of no group (`CHIP_0402` Yageo `RC`: 15; `CHIP_1210` Yageo `RC`: 28) | 2–28 | 92–118 (76.7–98.3%) |
| Board 4, bottom | 84 | nine groups, 2 to 16 bodies each, and 20 bodies of no group (`CHIP_0402` Yageo `RC`: 15) | 2–16 | 68–82 (81.0–97.6%) |

Board 4's 20 bodies of no group on each side are drawn chip parts whose part numbers are not mapped members of any group; they are in the median and count as other bodies.

Every existing reading counts, so this condition changes no group today. The pilot counts for Yageo `RC` by one body: 45 others against 44 of its own. With one other body fewer the 44 pilot samples would stop counting and the group would need 44 more; on a pilot with nothing but RC drawn, all 44 are unsupported and the 5 on board 3 still count (`tools/tests/test_w3_groups.py`). As a check of what the condition is for, the pilot median taken over the other 45 bodies alone is (+0.011, −0.008) mm against (+0.011, −0.010) with RC in it, 0.002 mm apart, and every RC pilot sample is still within its residual limit against it (largest C137958, 0.036 of 0.064 mm). For Vishay `CRCW` and Yageo `CC` the same test also moves the median 0.002 mm and no sample leaves its limit (largest 0.032 of 0.064 mm each). A stricter form of this condition, the median of the other groups only, would therefore change nothing on these readings. (By method 1 it put one CRCW sample, C482142, over its limit.) That form is not proposed, because Tony accepted the board median; it is noted for him.

**Not-drawn share: pause, not flag (proposed).** A flag changes nothing that CI or the generator can hold a record to: a flagged group would still be sampled around its holes and inferred. Pausing is the conservative reading of "flag or pause", and it is what a group record can enforce: both checkers reject an `active` record whose not-drawn part numbers are more than 10% of its samples and not-drawn members together. Exactly 10% is not above it. Two groups are over it today, and neither has a failed sample:

- Vishay `CRCW` 0402: 10 of 58 judged part numbers not drawn (17.2%); 48 of its 49 required samples pass. Flagged instead of paused, it would take one more sample and be inferred with 10 holes.
- Yageo `RC` 1210: 15 of 44 judged not drawn (34.1%), 14 of them among board 4's 42 samples; 29 of 43 pass. Flagged instead, it would go on to 14 more samples.

Paused, no more samples are chosen and nothing is inferred until the group is reviewed (split, or verified part by part, §3.5). The share is taken of the required sample while fewer members have been judged, because the first few readings of a group say little about its share: before board 4, Yageo `RC` 1210 had 1 not drawn of 2 judged, against 43 required, and was not paused; Yageo `RC` 2512 is there now (1 of 2, 44 required, 2.3%). Yageo `CC` 0402 has 1 not drawn of 13 judged (7.7%) and is sampled.

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

**What the rule gives today** (`w3_sampling/2026-11-12/`, rule `w3-2026-11-12`: method 2 readings of the pilot, board 3 and board 4): 2 groups sampled, 14 need samples, 4 paused. Each row: passed of required; part numbers not drawn of judged; then, over every copy of the passed samples, the largest raw offset and residual against their limits and the largest angle. Every view any of these groups is read on has other groups in the majority (the table above), and no reading is unsupported.

| Group (members) | Status | Passed | Not drawn | Largest raw / residual / angle | Why |
|---|---|---|---|---|---|
| `CHIP_0402` Yageo `RC` (969) | **sampled** | 49 of 49 | 0 of 49 | 0.053 of 0.128 mm / 0.041 of 0.064 mm / 5.5° | 44 pilot and 5 board 3 samples; three of them again on board 4, ten copies each, all passing |
| `CHIP_0402` Yageo `CC` (199) | **sampled** | 12 of 10 | 1 of 13 (7.7%) | 0.056 of 0.128 / 0.034 of 0.064 / 6.3° | C576731 not drawn; board 4 adds three part numbers |
| `CHIP_0402` Vishay `CRCW` (968) | **paused** | 48 of 49 | 10 of 58 (17.2%) | 0.040 of 0.128 / 0.032 of 0.064 / 4.4° | not-drawn share above 10%; no sample fails |
| `CHIP_1210` Yageo `RC` (846) | **paused** | 29 of 43 | 15 of 44 (34.1%) | 0.036 of 0.250 / 0.043 of 0.125 / 1.2° | not-drawn share above 10%; no sample fails |
| `CHIP_0201` Yageo `RC` (741) | **paused** | 0 of 38 | 0 of 1 | | C138161 fails at board 3's R4: raw 0.099 mm against 0.080, residual 0.062 against 0.040 |
| `CHIP_0805` Yageo `CC` (142) | **paused** | 1 of 8 | 0 of 4 | 0.058 of 0.228 / 0.041 of 0.114 / 0.2° | C527206, C527232 and C576985 residual 0.120, 0.138 and 0.156 mm against 0.114 |
| `CHIP_0201` Yageo `CC` (117) | needs samples | 2 of 6 | 0 of 2 | 0.056 of 0.080 / 0.025 of 0.040 / 5.2° | |
| `CHIP_0402` Samsung `CL` (11) | needs samples | 1 of 3 | 0 of 1 | 0.035 of 0.128 / 0.022 of 0.064 / 2.9° | |
| `CHIP_0603` Yageo `RC` (969) | needs samples | 4 of 49 | 0 of 4 | 0.074 of 0.206 / 0.044 of 0.103 / 0.3° | |
| `CHIP_0603` Samsung `CL` (24) | needs samples | 2 of 3 | 0 of 2 | 0.040 of 0.206 / 0.029 of 0.103 / 1.2° | |
| `CHIP_0603` Yageo `CC` (273) | needs samples | 1 of 14 | 0 of 1 | 0.030 of 0.206 / 0.018 of 0.103 / 0.7° | |
| `CHIP_0805` Yageo `RC` (969) | needs samples | 5 of 49 | 0 of 5 | 0.045 of 0.228 / 0.059 of 0.114 / 0.2° | |
| `CHIP_1206` Yageo `RC` (967) | needs samples | 4 of 49 | 0 of 4 | 0.087 of 0.250 / 0.103 of 0.125 / 0.3° | |
| `CHIP_2010` Yageo `RC` (865) | needs samples | 2 of 44 | 0 of 2 | 0.070 of 0.250 / 0.036 of 0.125 / 0.3° | |
| `CHIP_2512` Yageo `RC` (863) | needs samples | 1 of 44 | 1 of 2 (2.3% of the 44 required) | 0.082 of 0.250 / 0.113 of 0.125 / 0.2° | C878293 not drawn |
| the other 5 groups | needs samples | 0 | 0 | | no record has touched them |

These parts keep their own bindings: a paused group only stops inference. What each of the three proposals changes against the accepted rule on main's two records (`2026-11-10/`: 1 sampled, 14 need samples, 5 paused) is in §6's table and the dated reports: the conditions alone on those records (`2026-11-11/`) pause Vishay `CRCW` 0402; method 2 alone with board 4 (`2026-11-11-method-2/`) gives 2 / 16 / 2; both together pause `CRCW` 0402 and `RC` 1210 on top of that.

## 6. How a chip body is read (proposed 2026-11-11, pending Tony)

**Status: proposed, not accepted.** The rule `w3-2026-11-12` carries `body_method: 2` beside the limits of `w3-2026-11-10b`, which are unchanged, and the two conditions of §5. Tony decides whether method 2 becomes the W3 reading. (This section was first written under the interim version `w3-2026-11-11-m2`, the accepted limits plus `body_method` alone; its report is kept as `w3_sampling/2026-11-11-method-2/`.)

**The defect.** `measure_preview.py` read a chip body as every pixel within 1.5 pad half-spacings of the origin that is not board green, bare copper or marker black (method 1). That window also holds part of the part's own white reference text, which our generator puts on one side of the land and which turns with the part. So the reading was of the model and its text together. Board 4 (`jlc_previews/2026-11-11-board-4/`), the first record with one part number at five angles on both sides, shows it: of 204 drawn chip bodies, 58 read more than 10° off their pads (up to 78.5°) and offsets reach 1.54 of the per-part limit, while by eye every one lies along its pads, centred.

**Method 2** (`measure_preview.body_model`) reads the drawn model alone: the pixels joined to the footprint origin.

1. Board green is not the model, and neither is a pad's bare copper (copper colour in an area at least 3 px across). A thinner run of copper colour is the export's blend where the red designator meets a white terminal; it is inside the model and stays.
2. Of the rest, the model is what is joined to the origin by a path at least 3 px wide. Text beside the model is cut off by the board between them, and text that only grazes it (a bridge 1 or 2 px wide, as on the 0201 R68 of board 4) is cut off too.
3. The angle, the centroid and the limits are computed as before. A reading names its method (`"method": 2` in `body`); one that names none is method 1.

**What does not change.** The thresholds: 10°, raw offset within 1.0 of the per-part limit, residual within 0.5 after the view's median, zero-stock and not-drawn members not inferred. The archived records: `measurements.json` and `remeasure-2026-11-10.json` are untouched and still reproduce byte for byte with `measure --method 1`. The method 2 readings are new files beside them (`remeasure-2026-11-11-method-2.json`, named by each record's `w3-readings.json`). The dated reports are snapshots, each under the rule of its day: `2026-11-07/`, `2026-11-10/` (`w3-2026-11-10b`), `2026-11-11/` (`w3-2026-11-11`, the two conditions on method 1 readings), `2026-11-11-method-2/` (`w3-2026-11-11-m2`, method 2 without the conditions). The report under the rule in the tree is `w3_sampling/2026-11-12/`.

**What the rule requires.** `w3_groups.py` judges a record only by readings of the rule's method, takes each view's median over those readings, and stops if a file listed for the method holds another's. The two conditions of §5 are applied to those readings: a view's bodies are the ones read by the rule's method. A group record's samples each carry `body_method`; `sourcing/bindings.py` and `tests/library/supplier.rs` reject a sample read any other way (the `method` counterexample in `tests/fixtures/library-ci/s-sampled/`). So a record made from method 1 readings cannot be checked against these limits by accident, in either direction.

**On the three records** (drawn chip bodies):

| Record | Bodies | Failing the body check, method 1 → 2 | Largest angle | Largest offset, share of its limit |
|---|---:|---|---|---|
| Board 4, 2026-11-11 | 204 | 67 → 0 | 78.5° → 6.3° | 1.54 → 0.41 (0.053 mm on `CHIP_0402`) |
| W3 pilot, 2026-11-07 | 89 | 1 → 0 | 10.5° → 2.6° | 0.95 (0.121 mm) → 0.31 (0.040 mm) |
| C2 board 3, 2026-11-05 | 51 | 0 → 1 (R4) | 7.9° → 5.2° | 0.88 → 1.24 (R4, 0.099 of 0.080 mm) |

The view medians the residual is taken against:

| View | Bodies | Method 1 | Method 2 |
|---|---:|---|---|
| Pilot, top | 89 | (+0.005, −0.077) mm | (+0.011, −0.010) mm |
| Board 3, top | 39 | (−0.010, −0.025) | (−0.021, +0.026) |
| Board 3, bottom | 12 | (−0.038, −0.011) | (−0.041, +0.013) |
| Board 4, top | 120 | (−0.005, −0.064) | (+0.004, −0.011) |
| Board 4, bottom | 84 | (−0.021, −0.060) | (−0.010, −0.009) |

So the pilot's 0.077 mm "board bias" along −y (§5) was the reference text above each part, not the export and not the land: with the model alone it is 0.010 mm, a third of a pixel. Board 3's `CHIP_2010` "0.18 mm that turns with the part" reads 0.031–0.068 mm (0.171–0.206 by method 1). Board 3 is drawn differently from the other two (shaded models with a dark side face, which moves a centroid by a few hundredths of a millimetre), and its medians do not shrink.

**Every group, three ways, by the reading alone.** This table leaves the two conditions of §5 out, to show what the reading by itself changes: it is the accepted rule `w3-2026-11-10b` in (a) and (b), and `w3-2026-11-11-m2` in (c). With the conditions (§5's table), `CHIP_0402` Vishay `CRCW` is paused in all three columns and `CHIP_1210` Yageo `RC` in (b) and (c), by their not-drawn shares; nothing else differs. (a) is method 1 on the two records main has, the outcome on main today. (b) is method 1 with board 4's record added, what PR #111 alone gives. (c) is method 2 with all three records. Each cell: status, passed of required, then the largest raw offset and residual (each against its limit) and the largest angle over every copy of the passed samples. Groups no record has touched are left out (5 of 20, all `needs samples`).

| Group (members) | (a) method 1, main today | (b) method 1 with board 4 (#111 alone) | (c) method 2 with board 4 |
|---|---|---|---|
| `CHIP_0201` Yageo `RC` (741) | **paused** 0/38 (1 failed) | **paused** 0/38 (1 failed) | **paused** 0/38 (1 failed) |
| `CHIP_0201` Yageo `CC` (117) | needs samples 2/6; raw 0.048/0.080, residual 0.040/0.040, 3.2° | needs samples 2/6; raw 0.048/0.080, residual 0.040/0.040, 3.2° | needs samples 2/6; raw 0.056/0.080, residual 0.025/0.040, 5.2° |
| `CHIP_0402` Vishay `CRCW` (968) | needs samples 40/49 (9 not drawn); raw 0.100/0.127, residual 0.063/0.064, 7.5° | **paused** 45/49 (3 failed, 10 not drawn); raw 0.100/0.127, residual 0.063/0.064, 7.1° | needs samples 48/49 (10 not drawn); raw 0.040/0.128, residual 0.032/0.064, 4.4° |
| `CHIP_0402` Yageo `RC` (969) | **sampled** 49/49; raw 0.121/0.128, residual 0.059/0.064, 9.9° | **paused** 46/49 (3 failed); raw 0.116/0.127, residual 0.059/0.064, 6.6° | **sampled** 49/49; raw 0.053/0.128, residual 0.041/0.064, 5.5° |
| `CHIP_0402` Samsung `CL` (11) | needs samples 0/3 | **paused** 0/3 (1 failed) | needs samples 1/3; raw 0.035/0.127, residual 0.022/0.064, 2.9° |
| `CHIP_0402` Yageo `CC` (199) | **paused** 8/10 (1 failed, 1 not drawn); raw 0.082/0.127, residual 0.048/0.064, 7.9° | **paused** 7/10 (5 failed, 1 not drawn); raw 0.075/0.128, residual 0.048/0.064, 7.9° | **sampled** 12/10 (1 not drawn); raw 0.056/0.128, residual 0.034/0.064, 6.3° |
| `CHIP_0603` Yageo `RC` (969) | needs samples 4/49; raw 0.038/0.206, residual 0.020/0.103, 3.6° | needs samples 4/49; raw 0.038/0.206, residual 0.020/0.103, 3.6° | needs samples 4/49; raw 0.074/0.206, residual 0.044/0.103, 0.3° |
| `CHIP_0603` Samsung `CL` (24) | needs samples 0/3 | **paused** 1/3 (1 failed); raw 0.110/0.206, residual 0.050/0.103, 4.1° | needs samples 2/3; raw 0.040/0.206, residual 0.029/0.103, 1.2° |
| `CHIP_0603` Yageo `CC` (273) | needs samples 0/14 | **paused** 0/14 (1 failed) | needs samples 1/14; raw 0.030/0.206, residual 0.018/0.103, 0.7° |
| `CHIP_0805` Yageo `RC` (969) | needs samples 5/49; raw 0.042/0.228, residual 0.052/0.114, 2.3° | needs samples 5/49; raw 0.042/0.228, residual 0.052/0.114, 2.3° | needs samples 5/49; raw 0.045/0.228, residual 0.059/0.114, 0.2° |
| `CHIP_0805` Yageo `CC` (142) | **paused** 2/8 (2 failed); raw 0.092/0.228, residual 0.110/0.114, 2.4° | **paused** 2/8 (2 failed); raw 0.092/0.228, residual 0.110/0.114, 2.4° | **paused** 1/8 (3 failed); raw 0.058/0.228, residual 0.041/0.114, 0.2° |
| `CHIP_1206` Yageo `RC` (967) | needs samples 4/49; raw 0.135/0.250, residual 0.121/0.125, 0.8° | needs samples 4/49; raw 0.135/0.250, residual 0.121/0.125, 0.8° | needs samples 4/49; raw 0.087/0.250, residual 0.103/0.125, 0.3° |
| `CHIP_1210` Yageo `RC` (846) | needs samples 1/43 (1 not drawn); raw 0.057/0.250, residual 0.033/0.125, 3.3° | needs samples 29/43 (15 not drawn); raw 0.087/0.250, residual 0.045/0.125, 3.3° | needs samples 29/43 (15 not drawn); raw 0.036/0.250, residual 0.043/0.125, 1.2° |
| `CHIP_2010` Yageo `RC` (865) | **paused** 1/44 (1 failed); raw 0.134/0.250, residual 0.107/0.125, 0.9° | **paused** 1/44 (1 failed); raw 0.134/0.250, residual 0.107/0.125, 0.9° | needs samples 2/44; raw 0.070/0.250, residual 0.036/0.125, 0.3° |
| `CHIP_2512` Yageo `RC` (863) | **paused** 0/44 (1 failed, 1 not drawn) | **paused** 0/44 (1 failed, 1 not drawn) | needs samples 1/44 (1 not drawn); raw 0.082/0.250, residual 0.113/0.125, 0.2° |

Totals (sampled / need samples / paused): (a) 1 / 14 / 5, (b) 0 / 10 / 10, (c) 2 / 16 / 2. Method 2 on main's two records alone gives 1 / 17 / 2. With the two conditions: (a) 1 / 13 / 6, (b) 0 / 9 / 11, (c) 2 / 14 / 4, and 1 / 16 / 3 for method 2 on main's two records.

**Outcomes do change, and this is the part to weigh.** No threshold moved, but under method 2 more samples pass, because the text is no longer measured:

- `CHIP_0402` Yageo `CC` becomes **sampled** (12 of 10): C113781, left for manual review at 10.5° by method 1, reads within 2.6°, and board 4 adds three part numbers. On main it is paused. Without board 4's record it is one sample short.
- `CHIP_2010` and `CHIP_2512` Yageo `RC` are no longer paused (they need samples): their residuals of 0.171 and 0.169 mm against 0.125 were the text.
- `CHIP_0402` Yageo `RC` stays sampled with board 4's record instead of pausing, and its margins widen from 0.121 of 0.128 mm raw and 9.9° to 0.053 mm and 5.5°.
- Of 451 judged copies, 225 pass by method 1 and 340 by method 2; 116 move from fail to pass.

**Method 2 can fail, and does.** One sample that passed by method 1 fails by method 2: C527206 (`CHIP_0805` Yageo `CC`, board 3 C12), residual 0.110 → 0.120 mm against 0.114. Its group was already paused by C527232 and C576985, which still fail (0.138 and 0.156 mm): board 3 draws all three 0805 `CC` models 0.11–0.15 mm toward one pad, leaving the other bare. C138161 (`CHIP_0201` Yageo `RC`, board 3 R4 at −143°) failed on the residual and now fails on the raw offset too, 0.099 mm against 0.080: its model is drawn along its pads off centre. `tools/tests/test_measure_preview.py` shows on synthetic exports that a model shifted or turned beyond the limits fails with its text beside it.

**Limits of the method.**

- Silkscreen that overlaps the model, or touches it along 3 px or more, is read as part of it, as is anything else drawn joined to it. The reference text is drawn in the same white as a model's terminals, so colour cannot tell them apart; only position can.
- A model the export draws in separate pieces is read as its piece at the origin. A model with nothing drawn at the origin gives no reading, which fails.
- At about 31 px/mm an 0201 model is about 20 × 10 px. The 3 px rule is in pixels, so a much coarser export would lose small models (no reading, which fails) and a much finer one would cut fewer grazes.
- The red designator printed over each body is part of the reading. Board 3's shaded rendering is read as drawn.
- Validated on three records (344 bodies, five views, two rendering styles) and synthetic tests; crops of each land class were checked by eye. Inductors and other parts are not read by it.
- `jlc_preview.py`'s live check uses method 2 from now on. On board 4's archived exports it would pass all 67 chip instances it left for manual review (204 `body`, 16 `mark`, 15 `unverified`, 8 `manual review`: the inductors, which have no check).

**With the other open PRs.** The four are a stack, to merge in this order:

1. **#111** (board 4's record). Under method 1 alone its record pauses five more groups, among them the one #107 binds; under method 2 it pauses none.
2. **#115** (method 2 and the interim rule `w3-2026-11-11-m2`), on #111.
3. **#112** (the two conditions), on #115: the rule becomes `w3-2026-11-12`, one version carrying `body_method: 2` and the two conditions. The conditions count bodies and part numbers, which do not depend on the method, so they apply unchanged to method 2 readings; `CHIP_1210` Yageo `RC` (15 of 44 judged not drawn, 34.1%) and `CHIP_0402` Vishay `CRCW` (10 of 58, 17.2%) pause, `CHIP_0402` Yageo `CC` (1 of 13, 7.7%) does not.
4. **#107** (`CHIP_0402` Yageo `RC` group record), on #112: its record is regenerated under `w3-2026-11-12` from method 2 readings. Its 49 samples were first recorded by method 1 (largest raw 0.121 of 0.128 mm, residual 0.059 of 0.064, 9.9°). The same 49 by method 2, on the pilot and board 3: largest raw 0.050 mm (C93943), largest residual 0.036 mm (C137958), largest angle 2.6° (C137920); medians 0.022 mm, 0.013 mm and 0.5°. With board 4's 30 further copies of three of them: 0.053 mm, 0.041 mm and 5.5°, all 79 copies passing. The bindings it carries do not change.
