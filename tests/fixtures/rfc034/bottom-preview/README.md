# RFC-034 bottom-side convention fixture

`src/main.cohdl` is the independently previewed bottom-preview design, with the
supplier ID changed to `acme` and part numbers to `A-…`. The checked-in
`handoff/cpl-acme-reference.csv` is a byte-for-byte copy of the assembler's
independently aligned CPL (with the compiler's LF line endings), not an
expected value calculated by this compiler.

Rebuild from the repository root (the checked-in dependency lock is generated
by the first build):

```sh
rm -rf tests/fixtures/rfc034/bottom-preview/out
cargo run -- build tests/fixtures/rfc034/bottom-preview   --supplier acme --bottom-rotation acme=mirrored --emit kicad_pcb
cmp tests/fixtures/rfc034/bottom-preview/out/bottom_preview-acme-cpl.csv   tests/fixtures/rfc034/bottom-preview/handoff/cpl-acme-reference.csv
cp tests/fixtures/rfc034/bottom-preview/out/bottom_preview.kicad_pcb   tests/fixtures/rfc034/bottom_preview.kicad_pcb
cp tests/fixtures/rfc034/bottom-preview/out/bottom_preview-acme-map.json   tests/fixtures/rfc034/bottom_preview-acme-map.json
cp tests/fixtures/rfc034/bottom-preview/out/bottom_preview-acme-cpl.csv   tests/fixtures/rfc034/bottom_preview-acme-cpl.csv
```

The three files at `tests/fixtures/rfc034/` are the Python integration test's
contract. Do not commit generated `out/` or lock files.
