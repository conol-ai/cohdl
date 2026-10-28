# RFC-034 W9: supplier bottom-side rotation

Implemented on `rfc034/7-bottom-rotation`, based on `5efea7b`.

- Added top-level `supplier ID { bottom_rotation: kicad | mirrored }` parsing,
  package-set storage, E816 validation, one-line formatting, and E817 at the
  physical CPL boundary. IDs remain contextual and flat; declarations do not
  enter the ordinary reference namespace.
- Supplier CPL now applies `(180 - theta) mod 360` to mapped bottom rows for
  `mirrored`; top rows, positions, sides, neutral CPL and BOM are unchanged.
  `kicad` and undeclared CPL bytes match. E817 is absent with a declaration or
  `--logical-only`.
- Supplier maps are schema 2 and record the convention used. `smt_pos.py --map`
  supports schema 1 as `kicad`, schema 2 conventions, and rejects unknown
  schemas/values before creating or truncating output.
- Added the independently sourced bottom-preview fixture. Its checked-in
  reference comes from the assembler-aligned CPL (LF-normalized only), not from
  compiler calculations. Compiler CPL and Python's mapped emitted board both
  match that reference byte-for-byte. Existing map-integration board/CPL bytes
  were verified unchanged; only map schema/fields changed.

## Validation

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test` (also rerun with `--no-fail-fast` for a complete summary): all
  targets passed, 0 failed.
- `python3 -I -m unittest discover -s tools/tests`: 52 passed.
- CLI probe: undeclared physical build emitted E817; undeclared logical-only
  and declared physical builds did not.
- Registry files: unchanged. RFC design documents, Note 10 and the outer-loop
  blackboard: unchanged.

## Not manually verified

No physical assembler preview or interactive KiCad/pcbnew GUI session was run.
Python board compatibility is covered by the standard-library S-expression
board adapter against the independently sourced emitted-board fixture.
