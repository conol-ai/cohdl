# CoHDL — VS Code extension

Syntax highlighting and language-server support for CoHDL (`.cohdl`) source,
packaging the built-in `cohdl lsp` (RFC-014) as an installable extension
(RFC-019 / DR-025).

It adds **no new diagnostics or compiler behavior** — it is a TextMate grammar
(for color) plus a thin `vscode-languageclient` wiring that spawns `cohdl lsp`,
turning on RFC-014's four capabilities: diagnostics, hover, goto-definition,
and find-references. The server's output is exactly `cohdl lsp`'s, unmodified.

## Build from source

Requires Node.js ≥ 18 and a built `cohdl` binary (`cargo build` in the repo root).

```sh
cd editors/vscode
npm install
npm run compile      # tsc -> out/extension.js
npm test             # grammar-coverage regression test
npm run package      # -> cohdl.vsix
```

## Install locally

```sh
code --install-extension cohdl.vsix
```

## Configure

The extension resolves the `cohdl` binary via the `cohdl.path` setting
(default `"cohdl"`, found on `PATH`). Point it at a workspace build if needed:

```json
{
  "cohdl.path": "${workspaceFolder}/target/debug/cohdl"
}
```

If the binary cannot be started, the extension shows a visible error
notification (never a silently blank Problems panel).

## Editor support beyond VS Code

Neovim, Emacs, and other generic LSP clients don't need this package — a launch
config is enough; see [`docs/lsp.md`](../../docs/lsp.md) for those snippets.
This extension exists because VS Code needs a packaged TextMate grammar to get
syntax color at all, which the LSP protocol has no verb for.

## Maintenance note (grammar drift)

`syntaxes/cohdl.tmLanguage.json` is hand-authored from the Accepted grammar
(RFC-001…018). Any future RFC that adds, renames, or removes a top-level
keyword must update this grammar in the same change — the same "ship with its
spec update" discipline the language spec itself follows. `npm test` catches a
keyword that falls through to unstyled text, but cannot catch one styled as the
wrong class; grammar review remains a human step.
