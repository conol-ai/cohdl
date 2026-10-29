# A6 repeatable acceptance

Run from the repository root after `cargo build`. All source rewrites occur in
temporary copies; the test fails with the exact command output on error.

```sh
A6_ACCEPTANCE=1 python3 -I tools/tests/test_a6_acceptance.py
cd tools && python3 -I -m unittest discover -s tests
```

The first command checks two real CoHDL projects and a third array/mounting
example: forward/reverse source bytes return exactly, and the compiler-emitted
`.kicad_pcb` after forward realignment is byte-identical (therefore every pad
coordinate is unchanged). The non-zero change table is:

| Example | Footprint | dx mm | dy mm | Rotation |
|---|---|---:|---:|---:|
| bottom-preview | `transistor::FP_JSCJ_SOT23` | 2 | 1 | 90 |
| bottom-preview | `soic::SOIC8P127X790X216N` | -1 | 2 | 270 |
| map-integration | `map_integration::FP_A` | 2 | 1 | 90 |
| array/mounting | `p::F` | 2 | 1 | 90 |

It also replays bottom-preview and compares `.kicad_pcb`, supplier CPL and map
against the fixture (only the fixture's acme/C-number namespace is normalized
for the map; geometry is not normalized). The standalone replay is:

```sh
rm -rf /tmp/a6-replay
python3 -I tools/jlc_frame_board.py tools/tests/fixtures/a6/bottom-preview.json \
  -o /tmp/a6-replay --template tests/fixtures/rfc034/bottom-preview \
  --cohdl target/debug/cohdl --skip-kicad
cmp /tmp/a6-replay/out/bottom_preview.kicad_pcb \
  tests/fixtures/rfc034/bottom_preview.kicad_pcb
cmp /tmp/a6-replay/out/bottom_preview-lcsc-cpl.csv \
  tests/fixtures/rfc034/bottom-preview/handoff/cpl-acme-reference.csv
```

Grid generation, KiCad Gerbers/drill and hashes (use the installed path on this
machine, or `--kicad-cli kicad-cli` when it is on PATH):

```sh
rm -rf /tmp/a6-grid
python3 -I tools/jlc_frame_board.py tools/tests/fixtures/a6/grid.json \
  -o /tmp/a6-grid --cohdl target/debug/cohdl \
  --kicad-cli /Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli \
  --spacing-mm 4
cat /tmp/a6-grid/CHECKLIST.md
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
