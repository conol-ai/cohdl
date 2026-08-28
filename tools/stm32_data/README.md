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

`order_codes.txt` is a pre-existing validation-only snapshot of the exact
strings on ST's official MCU portfolio page on 2026-08-27: 4,930 order-code
rows, or 3,578 electrical identities after terminal `TR` packaging collapse.
No redistribution license for the webpage compilation is recorded here. The
generator therefore treats it as repository-only tooling input and does not
copy its rows or unmatched identities into the publishable STM32 package. It
requires every match to be unique. The pinned open pinouts plus exact C5 PDSC
variants match 4,696 rows / 3,398 identities; after incomplete exposed-pad
packages are quarantined, emitted devices represent 3,708 rows / 2,731
identities. The website list never supplies pin data, connection semantics,
parts, or footprints.

Order-code inventory SHA-256:
`5d6ae32eb20f8cbb82225b90c8332ebc93219a3398a93db634889644832cf8dc`.
Both input hashes are enforced by the generator.

`parts.json` is a separate, fail-closed fabrication overlay. It does not turn
the broad portfolio scrape into library source. Each emitted exact part names
its local manufacturer document, checksum, ordering-code locator, generated
device, audited package variant, and dependency-owned footprint. Schema 1
currently covers the 14 DS9826-backed F072 electrical identities whose LQFP or
UFBGA land patterns are complete: 22 exact order-code rows after their
tape-and-reel variants are included. The F072 UFQFPN variants remain excluded
because their exposed pad is absent from the pin source; the WLCSP variants
remain device-only because DS9826 does not recommend a PCB land diameter.

Audited-part overlay SHA-256:
`3d0b76e8461f8fff3d33bbd6c1519d8ab9d6d1e7881e4b5ae494215003b6d6db`.
The generator enforces this hash independently from both broad source inputs.

Normal generation and CI are offline:

```sh
python3 tools/gen_stm32.py
python3 tools/gen_stm32.py --check
```

Only an intentional upstream refresh uses `--import-sources`; the generator
verifies both checkout commits before it changes this snapshot.
