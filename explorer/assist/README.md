# Explorer Assist

Source-cited AI proposals for the native CoHDL Explorer. Node.js, the actual
CoHDL compiler/extractor and a configured Codex CLI are required. PDF excerpts
use Python3 with pypdf; unavailable extraction is recorded explicitly.

Build the compiler and standalone extractor before preparing a project:

```sh
cargo build
cargo build --manifest-path explorer/extractor/Cargo.toml
export COHDL_LIB="$PWD/lib"
```

Selection JSON uses exact extracted instance paths and qualified native groups:

```json
{"schema":1,"instances":[],"groups":[]}
```

Empty selection means the complete circuit. Put the requested review/change in
a UTF-8 text file. Run directories must be new and directly under the project's
`explorer/proposals/` directory, with a bounded alphanumeric/hyphen/underscore ID.

```sh
node explorer/assist/cli.mjs prepare PROJECT --selection selection.json --out bundle.json
node explorer/assist/cli.mjs propose bundle.json --kind group --engine codex --request request.txt --out PROJECT/explorer/proposals/review-1
node explorer/assist/cli.mjs validate PROJECT PROJECT/explorer/proposals/review-1 --out PROJECT/explorer/proposals/review-1/validation.json
node explorer/assist/cli.mjs apply PROJECT PROJECT/explorer/proposals/review-1 --validation PROJECT/explorer/proposals/review-1/validation.json --out PROJECT/explorer/proposals/review-1/receipt.json
node explorer/assist/cli.mjs revert PROJECT PROJECT/explorer/proposals/review-1/receipt.json
```

`group` changes only the native view JSON. `plan` is read-only and cannot be
applied. `connect` proposes exact replacements in existing `src/**/*.cohdl`.
Every proposal needs a valid source or reviewed exact document citation. Related
or unreviewed documents remain uncertain context. Electrical conflicts require
explicit review; compiler success does not qualify a physical circuit.

The provider runs in an isolated read-only working directory. Full input bundle,
exact proposal bytes, raw process logs and heartbeat/exit evidence are retained.
The prompt omits duplicate registry text/geometry with explicit omission messages
and fails if its bounded context is still too large. Optional `--timeout-ms`
accepts1000..600000; default180000. SIGINT/SIGTERM cancel the child process group.

Validation uses the actual compiler in an isolated copy. Apply requires a saved
PASS, repeats compilation, checks the complete source/document snapshot and uses
guarded atomic writes. Changed inputs invalidate the proposal. Revert refuses
unrelated subsequent edits. Validation freezes once a receipt starts; create a
new run to propose another operation. Interrupted operations remain explicit.

Serve the same project with the official `cohdl-explorer --serve`; the native
Proposals inspector displays retained runs, citations, exact changes, compiler
diagnostics, operation receipts and raw logs. Its API is read-only.

Executables can be configured with `COHDL_CODEX`, `COHDL_COMPILER`,
`COHDL_EXTRACTOR` and `COHDL_PDF_PYTHON`. Commands return JSON and nonzero on
failure. Unsupported engines are rejected; no simulated provider fallback exists.

```sh
node --test explorer/assist/test/*.test.mjs
```
