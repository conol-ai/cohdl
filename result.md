# RFC-034 W9: supplier bottom-side rotation

Implemented on `rfc034/7-bottom-rotation`, based on `5efea7b`.

- Added top-level `supplier ID { bottom_rotation: kicad | mirrored }` parsing,
  package-set storage, E816 validation, one-line formatting, and E817 at the
  physical CPL boundary. IDs remain contextual and flat; declarations do not
  enter the ordinary reference namespace.
- Supplier CPL now applies `(180 - theta) mod 360` to mapped bottom rows for
  `mirrored`; top rows, positions, sides, neutral CPL and BOM are unchanged.
  `kicad` and undeclared CPL bytes match. E817 is reported per supplier only
  when that supplier's CPL is actually written; it is absent with a declaration,
  without a placement/CPL, or with `--logical-only`. Its span is the selected
  build top's design name, including in a multi-design package.
- Supplier maps are schema 2 and record the convention used. `smt_pos.py --map`
  supports schema 1 as `kicad`; schema 2 requires `bottom_rotation`. Unknown
  schemas/values and a missing schema-2 convention fail before creating or
  truncating output.
- Added the independently sourced bottom-preview fixture. Its checked-in
  reference comes from the assembler-aligned CPL (LF-normalized only), not from
  compiler calculations. Compiler CPL and Python's mapped emitted board both
  match that reference byte-for-byte. Existing map-integration board/CPL bytes
  were verified unchanged; only map schema/fields changed.

## Validation

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test --no-fail-fast`: 70 test targets, 1014 passed, 0 failed.
- `python3 -I -m unittest discover -s tools/tests`: 52 passed.
- CLI regressions: no placement writes no supplier CPL and emits no E817; a
  placement writes the supplier CPL and emits one E817; a multi-design package
  anchors E817 at selected `ZTop`, not alphabetically-first `AFirst`.
- Python regressions: schema 1 still defaults to `kicad`; schema 2 missing
  `bottom_rotation` raises `MapError` and the CLI preserves existing output.
- Registry files: unchanged. RFC design documents, Note 10 and the outer-loop
  blackboard: unchanged.

## Not manually verified

No physical assembler preview or interactive KiCad/pcbnew GUI session was run.
Python board compatibility is covered by the standard-library S-expression
board adapter against the independently sourced emitted-board fixture.
