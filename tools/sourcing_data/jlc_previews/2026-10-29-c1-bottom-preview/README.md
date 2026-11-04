# C1 bottom-side preview: final round, 2026-10-29

This is the third and final assembler preview of the C1 test board, the bottom-preview board of `tools/tests/fixtures/a6/bottom-preview.json`. It was uploaded for preview only, with no order. The two `c1-` rows of `jlc_frames.json` cite this record. The C2146 and C97521 results of the same round were measured again on C2 frame board 1 (`../2026-10-30-c2-board-1/`), and their rows cite that record instead.

The board holds four parts: C2146 (SOT-23), C97521 (SOIC-8, 208 mil), C107045 (0603) and C107110 (0805). Each is placed four times: top 0°, top 37°, bottom 0° and bottom 37°. This round's CPL was exported with `--bottom-rotation lcsc=mirrored` and with these maps:

- C2146: `rotate 90`;
- C97521: `rotate 270`;
- the two chip capacitors: none.

`top.jpg` is the top view and `bottom.jpg` the view from below, mirrored left to right. On every copy the preview's pin-1 dot sits on our pin-1 marker, and the chip capacitors sit centred on their pads. The upload files of this round were not kept, so the record has no hashes for them.

| Part | Footprint | Map |
|---|---|---|
| C107045 | `passive::CHIP_0603` | none |
| C107110 | `passive::CHIP_0805` | none |
