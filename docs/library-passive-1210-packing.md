# The 1210 MLCC hole, its one-letter cause, and what blocks the fix

Status: **diagnosed 2026-11-27 in the nightly demand sweep; not fixed.** This
note is written outside `lib/passive/` on purpose — see §4, which is the reason
it cannot live in that package's own documentation today.

## 1. Symptom

`lib/passive/src/capacitors_1210.cohdl` has shipped with **zero parts** since it
was generated. Every other size emits hundreds: 179 in 0201, 279 in 0402, 364 in
0603, 204 in 0805, 199 in 1206. A generated file with a count of zero is a
sentence that reads as a fact about the vendor — Yageo offers no 1210 MLCCs —
and that fact is false.

## 2. Cause

Yageo's CC part number is

```
CC <size> <tolerance> <packing> <dielectric> <voltage> <process> <value>
```

and `tools/gen_passive.py` wrote `R` for the packing character of **every** size:
paper tape. A 1210 chip is too tall to be paper-taped and is offered on embossed
tape, whose letter is `K`. All 235 part numbers the 1210 tables list were
therefore spelled in a form Yageo has never published, `verify_passive_mpns.py`
recorded `false` for every one of them, and the generator dropped the size
rather than filling it wrongly.

The endpoint settles the diagnosis with size, dielectric, voltage and
capacitance all held fixed, so the packing letter is the only difference:

| Part number | Answer |
|---|---|
| `CC1210KKX7R8BB226` | 200, a specsheet |
| `CC1210KRX7R8BB226` | 404 |
| `CC1210KKX7R8BB106` | 200, a specsheet |

The distributor snapshot agrees independently: of its 483,022 rows, 48 part
numbers begin `CC1210KK` and **none** begins `CC1210KR`.

## 3. It is not one letter per size

The same snapshot shows both letters inside one size — 0805 carries 157 `KR`
and 76 `KK`, 1206 carries 79 `KR` and 107 `KK` — because the tape follows the
body height, and within a size the taller values need embossing. So the
correction is not "1210 means K"; it is a per-size table with a still-open rule
for the values that admit both.

This is also why 0805 and 1206 look thin: their tables list 380 and 342
combinations against 204 and 199 emitted, and 176 and 143 respectively are
withheld. Those withheld sets are hidden by the same bug.

The corrected 1210 numbers were probed, not assumed: **235 requests, 230
accepted** (79 under `KK` for X7R/X5R, 151 under `JK` for C0G) and five refused
— `CC1210KKX5R5BB106`, `CC1210KKX5R5BB107`, `CC1210KKX5R5BB227`,
`CC1210KKX5R6BB107`, `CC1210KKX5R7BB107`. The verdicts are recorded in
`tools/passive_data/yageo_verified.json`, which now holds 9,958 checks.

## 4. Why it is not fixed in this change

Emitting those 230 parts changes the package's content, so `passive` must move
from 0.2.12 to a new version — a docs-only edit inside `lib/passive/` already
breaks every example that locks the package, which is how this was found. And a
version bump is not cheap here:

- A supplier binding resolves by `(package, package_version, footprint, pn)` in
  `tools/sourcing/bindings.py`. **Every frame row is valid for exactly one
  package version.**
- Bumping `passive` to 0.2.13 therefore matches none of the 4,011 `passive`
  frame rows, and regeneration strips the `supplier lcsc` clause from every
  resistor and every capacitor in the package. Measured: `Bindings` resolves 333
  frame rows instead of 4,059, and the clauses that survive are alternates.
- Re-keying the frame rows alone does not help. The rows that survive are
  sampled ones whose W3 group records in `tools/sourcing_data/w3_records/`
  carry the version they were measured at, and rebuilding a record at a new
  version is an evidence operation, not a re-labelling.

The precedent is `590a054`, the 0.2.12 release: it re-keyed 1,383 frame rows
**and** rebuilt seven group records in the same commit.

### What a release that adds capacitances has to run

1. Rebuild the ten group records in `tools/sourcing_data/w3_records/` at the
   new version (`tools/w3_record.py`, one group each).
2. Re-key the `passive` rows of `tools/sourcing_data/jlc_frames.json`.
3. Bump `lib/passive/cohdl.toml`.
4. Apply `C_PACKING` in `tools/gen_passive.py` and regenerate.
5. `cohdl update` on each example, then the usual verification.

Until that runs, the 230 parts stay out and the file stays empty. That is a
deliberate ordering, not an oversight: shipping the parts by bumping the version
would silently delete 4,009 bindings.

## 5. A second over-claim, also not yet changed

The generated header states that each capacitor file holds *every* combination
the datasheet tables list. It never has: the endpoint withholds 56 in 0201, 64
in 0402, 80 in 0603, 176 in 0805, 143 in 1206 and 5 in 1210. The wording in
`tools/gen_passive.py` states both directions once the files are regenerated;
until then the files carry the old sentence.

## 6. What is recorded now

- The 235 verdicts for the corrected 1210 numbers, in
  `tools/passive_data/yageo_verified.json` — evidence, not conclusions, and
  checkable by re-probing any of them.
- `--match=PREFIX` on `tools/verify_passive_mpns.py`, so one size's rule can be
  re-checked without re-probing the 9,958.
- This note. Nothing under `lib/` was changed, so no package version moved and
  no example lock is stale.
