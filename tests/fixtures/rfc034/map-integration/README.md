# RFC-034 4b board/map/CPL fixture

`src/main.cohdl` exercises an annulus with segmented paste, rect/circle paste,
plated/non-plated mount holes, repeated pad number 1 at the same x and distinct
y, top/bottom cardinal and 37-degree placements, supplier mapping, and one
unbound instance (U5).

Regenerate from the repository root with the compiler at the baseline revision:

```sh
rm -rf tests/fixtures/rfc034/map-integration/out
cargo run -- build tests/fixtures/rfc034/map-integration --no-std \
  --supplier acme --emit kicad_pcb
cp tests/fixtures/rfc034/map-integration/out/map_integration.kicad_pcb \
  tests/fixtures/rfc034/map_integration.kicad_pcb
cp tests/fixtures/rfc034/map-integration/out/map_integration-acme-map.json \
  tests/fixtures/rfc034/map_integration-acme-map.json
cp tests/fixtures/rfc034/map-integration/out/map_integration-acme-cpl.csv \
  tests/fixtures/rfc034/map_integration-acme-cpl.csv
```

Regenerate `map-golden.json` and `map-golden.kicad_pcb` with the small board-backed
golden test in `tests/rfc034_map.rs`.

The board, map and compiler supplier CPL at the fixture root are the checked-in
contract; the Python integration tests read those three root files, never
`out/`. The entire generated `out/` directory and `design.lock` are intentionally
not part of this fixture. Output contains no machine-specific absolute paths.
E814 for U5 is expected.
