# The CoHDL language server (`cohdl lsp`)

RFC-014's LSP server: a thin JSON-RPC/stdio frontend over the exact same
`pipeline::check` the CLI runs — zero new diagnostic logic. Implementation:
`src/lsp.rs`; conformance tests (including the mandatory diagnostics
equivalence against `cohdl check --json`): `tests/lsp.rs`.

## Capabilities (exactly RFC-014's four)

| Capability | Behavior |
|---|---|
| `textDocument/publishDiagnostics` | On didOpen/didChange/didSave: re-checks the file's containing project (walks up to `cohdl.toml`, else single-file + std) and publishes the RFC-010 diagnostics — code, severity, message, and range are field-identical to `cohdl check --json`; secondary labels and `help:` lines ride `relatedInformation` |
| `textDocument/hover` | On an `impl Trait for Device {}` block: the resolved by-name pin/spec mappings (DR-013's ask — the information exists only in the compiler's resolution when the body is empty). On a device/trait pin declaration: obligation and role |
| `textDocument/definition` | A device/trait/fn/part name at any use site (inst type, impl names, generic bounds, super-traits, calls, part device refs) resolves to its declaration |
| `textDocument/references` | On a trait or device name (in an `impl` or at its declaration): every `impl` statement involving it, across the project and std |

Unsaved buffers: `didChange` (full-sync) contents override the on-disk file,
so diagnostics track what the editor shows, not what was last saved.

Not included, per the RFC's non-goals: code actions, rename, semantic tokens,
workspace symbols, incremental compilation (every event re-runs the full
check — acceptable at current project scale; incremental compilation is
tracked separately).

## Dependency note (DR-020)

This is the project's single scoped dependency exception: `lsp-types`
(pinned) supplies the protocol's message shapes, with `serde`/`serde_json` as
its serialization requirements — used only in the LSP layer. The JSON-RPC
transport loop itself (Content-Length framing, dispatch) is hand-rolled in
`src/lsp.rs`, and the compiler pipeline/emitters remain dependency-free.

## Editor setup

Build the binary once (`cargo build --release`), then point any LSP client at
`cohdl lsp`.

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

A full marketplace extension (grammar, packaging) is separate scope per the
RFC; these snippets are the promised minimal launch configuration.
