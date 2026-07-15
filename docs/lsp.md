# The CoHDL language server (`cohdl lsp`)

RFC-014's LSP server: a thin JSON-RPC/stdio frontend over the exact same
`pipeline::check` the CLI runs — the same diagnostics source, projected into
LSP shape. Implementation: `src/lsp.rs`; conformance tests (the diagnostics
equivalence suite against `cohdl check --json` runs the full four-field
projection over a fixture corpus): `tests/lsp.rs`. RFC-019 (DR-025) packages this server as an
installable VS Code extension (`editors/vscode/`); a pass in a live VS Code
session is still a human checkpoint, but the extension is now buildable,
packaged to a `.vsix`, and grammar-coverage tested (docs/compliance-report.md).

## Capabilities (exactly RFC-014's four)

| Capability | Behavior |
|---|---|
| `textDocument/publishDiagnostics` | On didOpen/didChange/didSave: re-checks the file's containing project (walks up to `cohdl.toml`, else single-file + std) and publishes the RFC-010 diagnostics — code, severity, message, and range are equivalence-tested against `cohdl check --json` over the corpus in tests/lsp.rs; secondary labels and `help:` lines ride `relatedInformation` when the client advertises support for it. A project that fails to LOAD (bad manifest, broken std) surfaces as `window/showMessage` — never a false-clean empty publish |
| `textDocument/hover` | On an `impl Trait for Device {}` block: the resolved by-name pin/spec mappings (DR-013's ask). On a device/trait pin declaration OR any pin use site (`d.A` — obligation and role, resolved through the instance's SELECTED structural variant, RFC-008). On a unit literal (incl. a function's own generic-parameter default `<V: Voltage = 3.3V>`): its unit type and RFC-001's allowed-prefix table row. On a `part` name (RFC-017): its MPN/MFR, resolved footprint symbol, and `#[doc]` reference paths. On a `pad N: Sym` placement (RFC-018): the resolved pad's shape/size/layer/plating/drill |
| `textDocument/definition` | A device/trait/fn/part/footprint/pad name at a REFERENCE use site (inst type, impl names, generic bounds, super-traits, calls, part device refs, footprint symbol refs, pad-placement symbols — RFC-016/017/018) resolves to its declaration. NOTE (open, review R5-10): definition on the qualified path INSIDE a `use` import is not yet supported (`UseDecl` is discarded during World construction) |
| `textDocument/references` | On a trait or device name (in an `impl` or at its declaration): every `impl` statement involving it, across the project and std |

Unsaved buffers: `didChange` (full-sync) contents override the on-disk file,
so diagnostics track what the editor shows, not what was last saved.

Not included, per the RFC's non-goals: code actions, rename, semantic tokens,
workspace symbols, incremental compilation (every event re-runs the full
check — acceptable at current project scale; incremental compilation is
tracked separately). Completion is excluded by RFC-014 even though RFC-001
asks for unit×prefix data in completion — a direct conflict between two
Accepted texts awaiting a note-side decision (docs/compliance-report.md).

Platform scope: POSIX hosts only. `file://` URIs with an empty or
`localhost` authority are accepted; Windows drive letters, backslashes, and
UNC paths are not supported.

## Dependency note (DR-020)

This is the project's single scoped dependency exception: `lsp-types`
(pinned) with `serde`/`serde_json` as its serialization requirements — used
only in the LSP layer. In practice `lsp-types` supplies the typed RESPONSE
shapes (`Hover`, `Location`, `Range`, `Position`, `Uri`); the JSON-RPC
transport loop, request dispatch, and publishDiagnostics payloads are
hand-rolled `serde_json` values (an honest narrowing of DR-020's original
framing — see docs/compliance-report.md). The compiler pipeline/emitters
remain dependency-free.

## Editor setup

Build the binary once (`cargo build --release`), then point any LSP client at
`cohdl lsp`.

### VS Code — packaged extension (RFC-019)

The recommended path: build and install the real extension at
[`editors/vscode/`](../editors/vscode/), which registers `.cohdl`, ships a
TextMate grammar for syntax color, and wires this server automatically —

```sh
cd editors/vscode && npm install && npm run package
code --install-extension cohdl.vsix
```

Set `cohdl.path` if the `cohdl` binary is not on `PATH`. The snippets below
remain valid for a generic LSP client or a quick extension-development-host
try-out.

### VS Code (minimal client, via an extension development host or a generic
LSP client extension)

Using a generic client such as the "LSP client" pattern, the entire
configuration is:

```json
{
  "command": ["/path/to/cohdl", "lsp"],
  "languageId": "cohdl",
  "filetypes": ["cohdl"],
  "rootPatterns": ["cohdl.toml"]
}
```

Or as a tiny VS Code extension `extension.js` (with `vscode-languageclient`):

```js
const { LanguageClient } = require("vscode-languageclient/node");
let client;
exports.activate = () => {
  client = new LanguageClient(
    "cohdl",
    "CoHDL",
    { command: "/path/to/cohdl", args: ["lsp"] },
    { documentSelector: [{ scheme: "file", pattern: "**/*.cohdl" }] }
  );
  client.start();
};
exports.deactivate = () => client && client.stop();
```

### Neovim (built-in LSP)

```lua
vim.api.nvim_create_autocmd("FileType", {
  pattern = "cohdl",
  callback = function()
    vim.lsp.start({
      name = "cohdl",
      cmd = { "/path/to/cohdl", "lsp" },
      root_dir = vim.fs.dirname(vim.fs.find({ "cohdl.toml" }, { upward = true })[1]),
    })
  end,
})
vim.filetype.add({ extension = { cohdl = "cohdl" } })
```

The packaged extension (grammar + `.vsix`) that RFC-014 deferred is now real
at [`editors/vscode/`](../editors/vscode/) (RFC-019). Marketplace publishing
(publisher account, version cadence) remains a separate distribution decision
per RFC-019's non-goals.
