# PR #78–#101 probe board: assembler preview, 2026-11-08

This is the assembler's placement preview of the probe board for PRs #78–#101: 14 part numbers, each placed four times (top 0° and 37°, bottom 0° and 37°), all with **zero rotation correction**. It is the board the two re-verify boards (`../2026-11-08-reverify/`, `../2026-11-09-reverify-2/`) took their candidates from. It was archived on 2026-11-10, together with body readings of its three bidirectional TVS parts, which appear on no other board.

The package is `pr78-101-frame-preview.zip`, SHA-256 `1a5d25174b39e91880b5a89eca5af0b345b2ac25990a295e3fa676718e556335`. It was built by `tools/frame_boards/preview_package.py` from `package/manifest.json`, on a local merge of the heads of PRs #78–#101 (package commit `7200cf2`; see "Commits named in this record"). Alex approved this upload for the preview only. The flow stopped at Component Placements: nothing was added to the cart and no order was placed.

## Files

| File | What it is |
|---|---|
| `top.png`, `bottom.png` | The preview's own 2D whole-board exports, kept byte for byte (JPEG data under a `.png` name), 4096 × 4096 px. `bottom.png` is the view from below, mirrored left to right. It draws the top side's lands (mirrored, bare) and the bottom parts, but not the bottom lands. No reading here uses drawn copper: every position comes from `package/board.json` through the board outline. |
| `top.pixels.png`, `bottom.pixels.png` | The pixels that were measured, as lossless PNGs. The measured files were BMPs (`sips -s format bmp top.png --out top.bmp`, 50 MB each), which are not committed. These PNGs are `sips -s format png` of those BMPs and decode to the same pixels: see "Reproducing the readings". |
| `png_to_bmp.py` | Standard-library script that writes the measured BMP from a `.pixels.png`, byte for byte, and prints its SHA-256. |
| `measurements.json` | `measure_preview.py measure package/board.json top.bmp bottom.bmp`, as recorded on 2026-11-08. |
| `checks.json` | `jlc_preview.py`'s per-instance coverage check: 44 placements with a pin 1 mark, 12 (the TVS parts) "drawn, with no pin 1 mark of its own". |
| `assessment.json` | The first automatic reading of `measurements.json`, as recorded. It is provisional: `manual-review-correction.txt` overrides it, and its `rotation_correction_required` verdicts are not binding input. |
| `manual-review-correction.txt`, `run-record.json`, `run-notes.zh-CN.md` | The 2026-11-08 manual correction, the run record (BOM step, counts, stop point) and the run's own notes, as written at the time (the first and last in Chinese). `run-notes.zh-CN.md` was the run directory's `README.md`; it is redacted (below). |
| `body.json`, `tvs_body_axes.py` | Body readings with the stock reach, added 2026-11-10, for the 12 TVS placements and 16 control placements. The script imports `../2026-11-09-reverify-2/body_axes.py` unchanged. |
| `body-wide.json`, `tvs_body_wide.py` | Body readings with a reach 0.8 mm wider, added 2026-11-10, for the 12 TVS placements and SS34. The SMC row rests on these (see "The SMC caveat"). |
| `crops/U1.png`, `U5.png`, `U6.png`, `U7.png`, `U29.png`, `U31.png` | Crops around a placement, cut from `top.png` (U1, U5, U6, U29) and `bottom.png` (U7, U31) and enlarged to 640 × 640 px for viewing. No reading uses them. |
| `frames.json` | Three archived candidate rows for the TVS parts (same row format as `jlc_frames.json`). Not read by any generator; see the last section. |
| `package/` | Every file of the uploaded package, unpacked: Gerber archive, LCSC BOM and CPL as uploaded, manifest, `board.json`, both lock files, `verification.json`, the checklist and `README.txt`. `SHA256SUMS` covers the others (`shasum -a 256 -c SHA256SUMS`). |

| File | SHA-256 |
|---|---|
| `top.png` | `be079f19c59f17b788fc47ee0ea2572a60c3da8caf727ba1a9467571f770ae06` |
| `bottom.png` | `632ce33f17c5683239fcf31739355a42e651a006144ad85b4a5e8c2d5464ac42` |
| `top.bmp` (measured, not committed) | `6672e3adc1d832effe691be46306f29aa71d029f77c51f2ba57b4acd9ebca07a` |
| `bottom.bmp` (measured, not committed) | `8aa0e53b24c03efd97c75fd3713ef20e56871a8e3ef9f7a88ad5ee120970ed62` |
| `top.pixels.png` | `cc6293dc3dc29ba7730626f05ce60aa570c335617078e3fc4c07aa3b239cbc51` |
| `bottom.pixels.png` | `5a2a5cec19495c155a00ff3e40560e018e7e3e7b3f7f1bd524b5eceaa281f806` |
| `measurements.json` | `37546fec9e2a23f5c5afa4e0b064bf90135ba94c5d9dc173329e3657d08f15f1` |
| `checks.json` | `e79b3227171489e596b7791ce4c19c7706c8a944e86d6dab8cacca9f77efd44e` |
| `assessment.json` | `3d93d6b71db3d44297b84587796895679a16e3cf424d4b9a09193e50b18adb91` |

### Redacted files

Three files named a local working directory. In each, that one directory prefix was replaced by `<scratch>/`. `SHA256SUMS` is unchanged and still holds the uploaded files' hashes, so `shasum -a 256 -c SHA256SUMS` reports exactly `README.txt` and `verification.json` as failed and every other file as OK, and the zip's SHA-256 above no longer reproduces from this tree.

| File | What changed | SHA-256 before redaction |
|---|---|---|
| `package/README.txt` | The prefix, once | `0e5bdf724e89564332918904f02ec89fa8fe94a3db9fb295b1b073fb9ee32e45` |
| `package/verification.json` | The prefix, once | `1c90e41559b3f5287eb5bbcbbf68fa2d1b2f4e00b89e917dd79c29dccff23a7d` |
| `run-notes.zh-CN.md` | In its "reproduce" command: the prefix, three times; the interpreter's absolute path, now `python3`; and a temporary output directory, now `<scratch>/` | `5021aee910166251085267250c9fee5c040090cc6459d083b61cb19e55f6926a` |

`verification.json` says `supplier_preview: not uploaded` because it was written before the upload. `run-record.json` and `run-notes.zh-CN.md` list the SHA-256 of the exports and JSON files; those match the table above.

### Left out

- The two 50 MB BMPs (replaced by the `.pixels.png` files) and the package zip itself (its files are under `package/`).
- The run directory's `analysis/` folder: a first top-only pass and the first bottom export, superseded by the files here.
- The other eight crops made on 2026-11-10 (U2, U3, U4, U8, U25, U30, U32, U53).
- The run directory held nothing else: no saved web pages, API responses, credentials or screen captures.

## What was uploaded

56 placements, 28 top and 28 bottom. Parts with no library part use a two-pin or multi-pin review device on the library's land (`package/manifest.json`). CPL angles are 0 / 37 / 0 / 323 for every part: no correction.

| Refs | Part | Footprint |
|---|---|---|
| U1–U4 | C19077561 SMBJ6.0CA | `do214::DIOM5336X265N` (`do214` 0.1.0) |
| U5–U8 | C19077609 SMCJ36CA | `do214::DIOM7959X261N` (`do214` 0.1.0) |
| U9–U12 | C18199202 SS24F | `do214::FP_DO221AC_SMAF` (`do214` 0.1.0) |
| U13–U16 | C32246052 W25Q32RVXHJQ | `flash::FP_W25Q32RV_XSON8_2X3` (`flash` 0.3.0) |
| U17–U20 | C5372421 RS0104YTQF14 | `contrib_level_shifter::QFN14N50P350X350_1EP200X150` (`@contrib/level-shifter` 0.2.0) |
| U21–U24 | C19077402 BZT52C5V6 | `sod::SOD3716X135N` (`sod` 0.1.0) |
| U25–U28 | C7420331 B5819WS | `sod::SOD2613X110N` (`sod` 0.1.0) |
| U29–U32 | C19077510 SMF15CA | `sod::SODFL3818X120N` (`sod` 0.1.0) |
| U33–U36 | C7420320 1N4148WT | `sod::SOD1608X70N` (`sod` 0.1.0) |
| U37–U40 | C7420357 MMBT5551 | `sot::SOT3P190X290X130N` (`sot` 0.1.0) |
| U41–U44 | C20069128 D882 | `sot::SOT3P150X450X250N` (`sot` 0.1.0) |
| U45–U48 | C7420322 2N7002W | `sot::SOT3P130X200X125N` (`sot` 0.1.0) |
| U49–U52 | C6186 AMS1117-3.3 | `sot::SOT4P230X651X351N` (`sot` 0.1.0) |
| U53–U56 | C7420365 SS34 | `do214::DIOM5026X241N` (`do214` 0.1.0) |

## BOM step

14 part numbers detected and confirmed. Each of the 28 top and 28 bottom designators appeared once, on its side, with its part number, selected. No stock warning. Detected board: 2 layers, 145 × 145 mm.

## Pin 1 marks (2026-11-08)

`run-notes.zh-CN.md` has the per-part table and `manual-review-correction.txt` the corrections. In short: MMBT5551 and D882 read within 10° on all four copies; RS0104 read 86° (its 270 candidate, now bound, was confirmed on the re-verify boards); W25Q32's 95° reading was withdrawn as a rotation and settled on the second re-verify board; the five diodes carry two marks each and were read by hand; 2N7002W and AMS1117 read 10–11° and stay under manual review; the three TVS parts have no mark. Nothing was bound from this board.

## Body readings of the bidirectional TVS parts (2026-11-10)

The rule (`docs/library-supplier-plan.md`, "Bidirectional TVS: judged by the body only"): on every copy, the body's long axis within 10° of the pad axis mod 180, and its centre within the chip offset limit, which is 0.25 mm for all three lands.

The land axis is pad 1 to pad 2. SMB and SMF are from `body.json` (stock reach); SMC is from `body-wide.json`.

| Part | Ref | Side, angle | Body (mm) | Long axis vs pad axis (mod 180) | Edges vs pad axis (mod 90) | Centre offset |
|---|---|---|---|---:|---:|---:|
| SMBJ6.0CA C19077561 | U1 | top 0° | 4.49 × 3.49 | −0.2° | 1.0° | 0.033 mm |
| | U2 | top 37° | 4.49 × 3.49 | 0.2° | 0.1° | 0.028 mm |
| | U3 | bottom 0° | 4.49 × 3.49 | −0.2° | −0.8° | 0.033 mm |
| | U4 | bottom 37° | 4.48 × 3.50 | 0.0° | 0.3° | 0.021 mm |
| SMCJ36CA C19077609 (wide reach) | U5 | top 0° | 6.84 × 5.75 | −0.1° | −0.4° | 0.004 mm |
| | U6 | top 37° | 6.80 × 5.77 | −0.1° | 0.1° | 0.027 mm |
| | U7 | bottom 0° | 6.81 × 5.78 | −0.1° | −0.4° | 0.007 mm |
| | U8 | bottom 37° | 6.79 × 5.77 | −0.2° | 0.1° | 0.008 mm |
| SMF15CA C19077510 | U29 | top 0° | 2.65 × 1.74 | 0.6° | 0.0° | 0.045 mm |
| | U30 | top 37° | 2.69 × 1.73 | 0.7° | 1.0° | 0.042 mm |
| | U31 | bottom 0° | 2.65 × 1.63 | −1.0° | 0.0° | 0.040 mm |
| | U32 | bottom 37° | 2.67 × 1.67 | −0.1° | 0.1° | 0.023 mm |

So on all 12 placements the long axis is within 1.0° of the pad axis (limit 10°) and the centre within 0.045 mm (limit 0.25 mm). The drawn sizes lie inside each datasheet's body length × width range, as recorded in the lands' evidence files: SMB 4.25–4.85 × 3.30–3.94 mm, SMC 6.60–7.11 × 5.59–6.22 mm, SOD-123FL 2.55–2.85 × 1.60–1.90 mm. In every case the long side lies along the pad axis (`long_side_is_along_land` in `body-wide.json`).

The wide run also read SMB and SMF. It agrees with the stock run: long axis within 1.0°, centre within 0.049 mm (SMF U29 reads 0.049 instead of 0.045, SMB U1 0.036 instead of 0.033).

### The SMC caveat

`body_axes.py`, unchanged, does not give a long-axis reading for SMC, for two reasons:

- **Reach.** It reads pixels within 0.6 mm of the farthest pad centre: 4.05 mm for this land. The drawn SMC body's corners are 4.46 mm from the centre, so the circle cuts them off. The stock rows in `body.json` still give 6.79–6.80 × 5.74–5.78 mm, edges within 0.4° and centre within 0.020 mm, because the rectangle's sides are read from percentiles and the cut corners are small.
- **Elongation.** It gives a long axis only when the fitted rectangle's long side is more than 1.2 times the short side. The SMC body's ratio is 1.18–1.19, so the field is absent in `body.json`.

`tvs_body_wide.py` is a record-local copy of the same reading with the reach widened by 0.8 mm (4.85 mm for SMC) and no elongation gate; the shared tools are unchanged. With it, the whole body is inside the window. The window then also holds a few stray dark pixels from JPEG ringing at the silkscreen lines, out to the window's edge (`max_dark_radius_mm` 4.82–4.85 on U5–U7, against the corner's 4.46 on U8). They are under 1% of the dark pixels (`largest_component_share` 0.992–1.000) and do not move the result: at +0.5 mm and +1.2 mm the SMC rows give the same sizes within 0.02 mm, the same axis within 0.1° and the same centre within 0.006 mm.

A body this close to square says less about a quarter-turn error than a longer one would: the reading rests on 6.8 mm against 5.8 mm, and on the rectangle's edges lying within 0.4° of the pad axis.

### What these readings do not show

- **Identity.** The preview draws each TVS as a plain rectangle with no band, mark or part text (`crops/U1.png`, `U5.png`, `U29.png`). The readings show alignment and centring of the model for that C number, not that the model is the right part.
- **Direction.** The orientation is known mod 180 only. That is the rule for a part with no polarity.
- **A mirror on the bottom side.** A mirrored model would look the same for a symmetric two-terminal body, and would place the same.
- **A tested correction.** The board carried no candidate: rotate 0 was applied and rotate 0 is what was read.

### Control placements

`body.json` also holds 16 placements of parts whose rotation is known from the re-verify boards. On this board they carry rotate 0, which is wrong for all four:

| Part | Refs | Long axis vs pad axis (mod 180) | Centre offset | Known mapping |
|---|---|---|---|---|
| SS34 C7420365 | U53–U56 | 0.2°, −0.4°, 0.5°, −0.3° | ≤ 0.127 mm | rotate 180 (archived candidate) |
| B5819WS C7420331 | U25–U28 | 4.4°, 0.7°, −3.3°, −6.1° | ≤ 0.091 mm | rotate 180 (archived candidate) |
| 1N4148WT C7420320 | U33–U36 | 0.4°, 1.5°, 1.0°, −3.1° | ≤ 0.117 mm | rotate 180 (archived candidate) |
| RS0104 C5372421 (square) | U17–U20 | edges within 0.1° (mod 90) | ≤ 0.017 mm | rotate 270 (bound) |

All four read as aligned and centred although each is a half or quarter turn off. That is the limit of a body reading: it cannot see a half turn, or a quarter turn of a square body. It is evidence only for a part that has no direction, which is why the rule is confined to bidirectional TVS parts. The diode offsets are larger because the lighter cathode band is not counted as body, which moves the centroid along the axis (SS34: `centre_offset_along_across_mm` in `body-wide.json`).

## Reproducing the readings

From the repository root, with `R=tools/sourcing_data/jlc_previews/2026-11-08-probe` and `T` an empty directory outside the repository:

```sh
python3 -I $R/png_to_bmp.py $R/top.pixels.png $T/top.bmp   # prints the BMP's SHA-256
python3 -I $R/png_to_bmp.py $R/bottom.pixels.png $T/bottom.bmp
python3 -I tools/frame_boards/measure_preview.py measure $R/package/board.json $T/top.bmp $T/bottom.bmp $T/measurements.json
python3 -I -B $R/tvs_body_axes.py tools/frame_boards tools/sourcing_data/jlc_previews/2026-11-09-reverify-2 \
  $R/package/board.json $T/top.bmp $T/bottom.bmp $T/body.json \
  U1 U2 U3 U4 U5 U6 U7 U8 U29 U30 U31 U32 U25 U26 U27 U28 U53 U54 U55 U56 U33 U34 U35 U36 U17 U18 U19 U20
python3 -I -B $R/tvs_body_wide.py tools/frame_boards $R/package/board.json $T/top.bmp $T/bottom.bmp $T/body-wide.json 0.8 \
  U1 U2 U3 U4 U5 U6 U7 U8 U29 U30 U31 U32 U53 U54 U55 U56
```

Checked on 2026-11-10: the two BMPs have the SHA-256 values in the table above, and are byte for byte the files `sips -s format bmp` writes from `top.png` and `bottom.png` (sips-316, macOS) and from the `.pixels.png` files; `measurements.json`, `body.json` and `body-wide.json` each come out byte for byte as committed. The steps use only files in the repository; none depends on a commit.

## Commits named in this record

| Commit | Where it is named | Where it exists |
|---|---|---|
| `7200cf2` | `package/README.txt`, `package/verification.json` (the commit the package was built from) | A local merge of the PR #78–#101 heads, never pushed: it is on no GitHub branch and not on `main`. It is kept here as a fact of the upload. On `main` after those PRs merged, `lib/sod`, `lib/do214`, `lib/sot` and `lib/@contrib/level-shifter` are identical to that merge, and `lib/flash` differs only in `docs/README.md` (compared locally on 2026-11-10). `package/cohdl.lock` pins each package by content hash. |

## Why `flash` 0.3.0 has two hashes in the archived locks

`package/cohdl.lock` of the 2026-11-08 probe and of the 2026-11-09 re-verify board 2 both pin `flash` 0.3.0, with different hashes. Each lock is the one the uploaded package was built with and is kept as it was; neither is replaced by the hash of the published 0.3.0.

| Record | Package commit (local merge, never pushed) | `flash` 0.3.0 hash in its lock |
|---|---|---|
| `2026-11-08-probe` | `7200cf2c89a58468dffb9421f8a8c35298ba23a9` | `sha256:7ed37ac57bafde9b63b2a1272bebfcd6b63a8b6d5e82c9aad5c62ca79e0902c8` |
| `2026-11-09-reverify-2` | `38b8434383849e95d24f16349cd6b4f5520c1a32` | `sha256:db726ea6e88bfca60b264c656a78735c93c8d856273076d4571e21fc7414ea76` |

The two `lib/flash` trees differ in two files only (`git diff 7200cf2 38b8434 -- lib/flash`, where both commits are available):
- `src/w25q128jv.cohdl`: the probe's tree binds `FLASH_W25Q128JVSIQ` to LCSC C97521 (PR #80's binding, in that local merge); board 2's tree does not.
- `docs/README.md`: the four lines describing that binding.

Both hashes were reproduced by a reviewer on 2026-11-11 by taking the package out of each commit and running the compiler offline with an isolated cache. `main` at `a0c5d11` differs from each tree again (it carries the C97521 binding and its README section as merged), so the published `flash` 0.3.0 has a third hash.

**This does not touch the W25Q32 evidence.** `src/w25q32.cohdl` is the same blob (`5dd19dc4…`) in both commits and on `main` at `a0c5d11`: the part definition and the land `FP_W25Q32RV_XSON8_2X3` that the previews read are identical. PR #108 changes that file only by adding `supplier lcsc "C32246052" rotate 270` to the primary.

## Status of the TVS rows: archived candidates, not bindings

`frames.json` holds three rows, each rotate 0 with no offset, read by the body only:

- `probe-diom5336x265n-c19077561` (SMBJ6.0CA on `do214::DIOM5336X265N`)
- `probe-diom7959x261n-c19077609` (SMCJ36CA on `do214::DIOM7959X261N`, read with the wider window)
- `probe-sodfl3818x120n-c19077510` (SMF15CA on `sod::SODFL3818X120N`)

They are not in `tools/sourcing_data/jlc_frames.json`. `sod` and `do214` ship lands only, and no library source lists C19077561, C19077609 or C19077510, so a row would bind nothing. No generator reads `frames.json`: `tools/sourcing/bindings.py` reads only `jlc_frames.json`. `tools/tests/test_generator_bindings.py` checks that these rows stay out of `jlc_frames.json` and that no library source lists their part numbers while they are archived.

Adopting a row is W4 discretes work: add a library part for the MPN on the same land with `supplier lcsc "<C number>"` and no rotation, and move the row into `jlc_frames.json`.

The rows were read against the `sod` and `do214` **0.1.0** lands and carry `package_version` 0.1.0. PR #102 moves both packages to 0.1.1. So adoption must re-key each row's `package_version` to the version the new part ships in, after checking that version's land copper is still the copper read here (`package/board.json` holds the pads as placed).

## Since 2026-11-11: library parts exist, still unbound

`esd` 0.1.3 adds `TVS_SMBJ6V0CA`, `TVS_SMCJ36CA` and `TVS_SMF15CA` on these three lands, without a supplier clause. The rows stay archived here, and the statements above still hold: no library source lists C19077561, C19077609 or C19077510.

What is missing is the map row. A binding's part number must be in `tools/sourcing_data/lcsc_map.json`, and rows reach it only through a reviewed `lcsc_backfill.py` run that reads every candidate page for the MPN. On 2026-11-11 the fetch cache held 1 or 2 of the 112 (SMBJ6.0CA), 52 (SMCJ36CA) and 46 (SMF15CA) candidate pages, and nothing was fetched. Once the rows are promoted, adoption is as described above: add the clause with no rotation, and move each row into `jlc_frames.json` with `package_version` 0.1.1 (`sod` and `do214` 0.1.1 change no pad from 0.1.0).

## Since the backfill run of 2026-11-11: adopted

The section above describes the state before the map rows existed. The backfill run in `../../lcsc_backfill/2026-11-11-hongjiacheng/` read every candidate page for the three MPNs and its rows were promoted, so C19077561, C19077609 and C19077510 are in `lcsc_map.json`. `esd` 0.1.3 now binds them with `supplier lcsc "<part number>"` with no rotation, and `jlc_frames.json` holds a copy of each archived row: `w4d-diom5336x265n-c19077561`, `w4d-diom7959x261n-c19077609` and `w4d-sodfl3818x120n-c19077510`, keyed to `sod` 0.1.3 and `do214` 0.1.2 after checking that no pad of these lands has changed since 0.1.0. Each copy's note says what the reading is and is not, and names the archived row it came from.

`frames.json` here is unchanged: it keeps the rows as they were read, with their 0.1.0 key and their original notes, including the sentence that no library part lists the part number, which was true when it was written. `tools/tests/test_generator_bindings.py` now checks the adoption: each part number is bound once, on the land it was read on, and only the adopted copies cite this record.
