# CoHDL for Zed

The Zed extension for [CoHDL](https://cohdl.org): tree-sitter syntax
highlighting, outline, bracket/indent behavior, and the `cohdl lsp`
language server (RFC-014 — diagnostics on open/change, hover,
go-to-definition, find-references).

Like `editors/vscode/`, this is a packaging layer over `cohdl lsp` with
**zero compiler changes** (RFC-019's discipline, DR-025). It is a
standalone package outside the compiler's zero-dependency rule.

## Layout

- `extension.toml` — the Zed manifest. The grammar is referenced by
  `repository` + `rev` + `path` back into this same repository, so `rev`
  must name a commit that already contains `grammar/` — bump it whenever
  the grammar changes.
- `src/lib.rs` — the WASM extension (`zed_extension_api`). Server
  resolution order: the user's `lsp.cohdl.binary` setting → `cohdl` on
  PATH → download of the newest GitHub compiler release (the same
  `cohdl-vX.Y.Z-<target>.tar.gz` artifact contract as `install.sh` and
  `cohdl self-update`).
- `grammar/` — tree-sitter-cohdl. An **editor** grammar, not a second
  compiler front end: declaration headers and the statements worth naming
  parse precisely, everything else degrades to typed tokens inside
  recursive bracket groups, so unknown constructs can never produce
  ERROR nodes. `grammar/check.sh` regenerates the parser (must be a
  no-op), runs the corpus tests, and parses every `.cohdl` file in the
  repository expecting zero error nodes — CI runs the same script.
- `languages/cohdl/` — the language config and the highlight, bracket,
  indent, and outline queries. Scope coverage mirrors the VS Code
  TextMate grammar; an RFC that adds a keyword updates both editors in
  the same change.

## Developing

```sh
cd grammar && npm ci && ./check.sh          # grammar gate
cargo build --release --target wasm32-wasip2  # the target current Zed builds for
```

In Zed: **Extensions → Install Dev Extension** and select this
directory. Note the grammar is fetched through `extension.toml`'s
`repository`/`rev`, not your working tree — to iterate on an uncommitted
grammar, point `repository` at `file:///path/to/cohdl` and `rev` at a
local commit, and revert before committing.

## Configuration

The extension finds `cohdl` on PATH (`curl -fsSL
https://raw.githubusercontent.com/conol-ai/cohdl/main/install.sh | sh`)
or downloads the newest release on demand. To pin a specific binary, in
Zed's `settings.json`:

```json
{
  "lsp": {
    "cohdl": {
      "binary": { "path": "/path/to/cohdl", "arguments": ["lsp"] }
    }
  }
}
```

## Publishing

Zed extensions ship through a PR to
[zed-industries/extensions](https://github.com/zed-industries/extensions)
adding this directory as a git submodule + an `extensions.toml` entry;
version bumps update the submodule pin. Not yet submitted.
