# Frozen STM32 generator input

`pin_data.json` is the deterministic, normalized import consumed by
`tools/gen_stm32.py`. It combines these pinned STMicroelectronics sources:

- `STM32_open_pin_data`, tag `STM32CubeMX-DB.6.0.180`, commit
  `7d1f1514ed5583ec5007ad91236b4e1d377295b1`;
- `stm32c5xx-dfp`, tag `2.1.0`, commit
  `a5f65bc64535cfa723e9d25f58d7ce23d0937aed`.

SHA-256: `c652aec5052601b99b9aacc066354a2f20450599e03d261e6fadebd95a48b682`.

Both upstream sources are BSD-3-Clause licensed. Their notices and detailed
transformation policy are preserved under `lib/@st/stm32/docs/`.

`order_codes.txt` is a snapshot of the exact factual strings on ST's official
MCU portfolio page on 2026-08-27: 4,930 order-code rows, or 3,578 electrical
identities after terminal `TR` packaging collapse. No page markup, descriptive
copy, or page structure is retained. The exact strings are admitted to package
source only after a unique join to separately licensed ST pin data, an official
ST datasheet URL, and a concrete dependency-owned footprint whose entire pad
set matches that pin data. The pinned open pinouts plus exact C5 PDSC variants
match 4,696 rows / 3,398 identities; after incomplete exposed-pad packages are
quarantined, emitted devices represent 3,708 rows / 2,731 identities. The
website list never supplies pin data, connection semantics, or fabrication
geometry.

Order-code inventory SHA-256:
`5d6ae32eb20f8cbb82225b90c8332ebc93219a3398a93db634889644832cf8dc`.
Both input hashes are enforced by the generator.

`parts.json` is the stronger, fail-closed local-PDF fabrication overlay. Each
row names its local manufacturer document, checksum, ordering-code locator,
generated device, audited package variant, and dependency-owned footprint.
Schema 1 covers the 14 DS9826-backed F072 electrical identities whose LQFP or
UFBGA land patterns are complete: 22 exact order-code rows after their
tape-and-reel variants are included. These rows override the broader mapping.

Audited-part overlay SHA-256:
`3d0b76e8461f8fff3d33bbd6c1519d8ab9d6d1e7881e4b5ae494215003b6d6db`.
The generator enforces this hash independently from both broad source inputs.

`kicad_parts.json` is the deterministic catalog-scale source join produced by
`tools/import_stm32_kicad.py`. It pins the official KiCad symbol repository at
commit `7800d91437ce44e2ed0928f2ad31a287457b8a68` and the footprint repository at
commit `819223b66f96508feaeaa305301b5e6bb5c1038b`. Both are CC-BY-SA-4.0. The
importer accepts an identity only when KiCad gives it one concrete footprint,
an official `www.st.com` datasheet URL, and that footprint's complete SMD
pad-number set exactly equals the physical positions in the pinned ST source.
It covers 2,389 identities / 3,303 order-code rows through 1,240 official ST
datasheet URLs and 103 concrete footprint variants. The remaining represented
identities stay device-only; no nearest package or label-only guess is made.

KiCad part-catalog SHA-256:
`3770ec5e52a60cf07f2134e4a721520acf200591a7ab6ead07e0614425e94c1f`.
The published package retains KiCad attribution and its unmodified license
notice, and the focused geometry packages use CC-BY-SA-4.0.

Normal generation and CI are offline:

```sh
python3 tools/gen_stm32.py
python3 tools/gen_stm32.py --check
```

Only an intentional upstream refresh uses `--import-sources`; the generator
verifies both checkout commits before it changes this snapshot.

Refreshing the catalog join is likewise explicit:

```sh
python3 tools/import_stm32_kicad.py \
  /path/to/kicad-symbols /path/to/kicad-footprints
python3 tools/gen_stm32.py
```
