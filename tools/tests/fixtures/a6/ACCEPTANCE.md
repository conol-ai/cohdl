# A6 repeatable acceptance

Run from the repository root after `cargo build`. The compiler tests run by
default whenever `target/debug/cohdl` exists; otherwise they are skipped. All
source edits and vendored-library changes happen in temporary directories.

```sh
python3 -I tools/tests/test_a6_acceptance.py -v
cd tools && python3 -I -m unittest discover -s tests
```

Expected real-compiler results:

- `joint-motor-controller`: 2 forward and 2 reverse placements.
- `rpi-pico2`: 2 forward and 2 reverse placements.
- `sf32-miniboard`: 1 forward and 1 reverse placement.
- Every example's source bytes are identical after reverse.
- Vendored joint rigid library move: 8 pads compared at 0 nm tolerance.
- Independent rigid fixture, top/bottom/37 degrees: 6 pads compared at 1 nm
  tolerance (the 37-degree fixed-point path differs by at most 1 nm).
- Generated bottom-preview: 60 pads compared at 0 nm and all 16 supplier CPL
  rows match byte-for-byte. Declarations, instances, nets and layout are all
  generated from the explicit-coordinate JSON; no source template is read.
- Expressions, `for`, subdesign declarations and reach-in paths produce
  file:line diagnostics and no source write.

Standalone bottom-preview replay:

```sh
rm -rf /tmp/a6-bottom-new
python3 -I tools/jlc_frame_board.py tools/tests/fixtures/a6/bottom-preview.json \
  -o /tmp/a6-bottom-new --cohdl target/debug/cohdl --skip-kicad
cmp /tmp/a6-bottom-new/out/bottom_preview-lcsc-cpl.csv \
  tests/fixtures/rfc034/bottom-preview/handoff/cpl-acme-reference.csv
```

Grid generation and real KiCad fabrication export (use `--kicad-cli kicad-cli`
when it is on PATH):

```sh
rm -rf /tmp/a6-grid
python3 -I tools/jlc_frame_board.py tools/tests/fixtures/a6/grid.json \
  -o /tmp/a6-grid --cohdl target/debug/cohdl \
  --kicad-cli /Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli \
  --spacing-mm 4
cat /tmp/a6-grid/fabrication/kicad-cli-version.txt
cat /tmp/a6-grid/fabrication/SHA256SUMS
shasum -a 256 /Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli
```

Full gates:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --no-fail-fast
cd tools && python3 -I -m unittest discover -s tests
```
