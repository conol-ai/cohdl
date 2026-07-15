# RFC-019: VS Code extension for CoHDL

## Problem

RFC-014 (LSP support) shipped `cohdl lsp` — a real, fully-wired, equivalence-tested language server (`src/lsp.rs`, `tests/lsp.rs`) — plus, as its own explicitly-scoped deliverable, a *minimal client launch snippet*: a bare `extension.js` sketch in `docs/lsp.md` showing how a VS Code extension would wire `vscode-languageclient` to `cohdl lsp`. RFC-014's own text drew the line explicitly: "a full marketplace extension (grammar, packaging) is separate scope per the RFC; these snippets are the promised minimal launch configuration." That line is this RFC.

Confirmed directly against the real repository: there is no `editors/`, `vscode/`, `.tmLanguage.json`, or any packaged extension anywhere in the tree — only the doc-embedded `extension.js` snippet. `docs/lsp.md` itself states plainly: "A pass in a live VS Code session has not yet been recorded — the RFC's real-client acceptance item is open (docs/compliance-report.md)." So this RFC closes two real, disclosed gaps at once: (1) RFC-014's own deferred packaging/grammar scope, and (2) RFC-014's still-open real-client acceptance item — a VS Code session actually exercising the server has never been recorded.

Who this is for: **human reviewers** using VS Code to read/review AI-generated `.cohdl` source (the Constitution's own stated review-loop persona) — currently they'd get a bare-text `.cohdl` file with zero syntax color, no error squiggles unless they hand-wire the snippet themselves, and no realistic path to actually trying the language server RFC-014 already built.

## Goals

- Ship a real, installable VS Code extension (a `.vsix`, buildable from source in this repo) that:Registers the `.cohdl` file extension and a TextMate grammar for syntax highlighting (a static, LSP-independent capability — the LSP protocol itself has no syntax-highlighting verb).Wires `vscode-languageclient` to spawn `cohdl lsp`, turning RFC-014's four capabilities (diagnostics, hover, goto-def, references) on for free — this is exactly the doc snippet already in `docs/lsp.md`, now packaged as a real extension rather than copy-paste boilerplate.Auto-discovers the `cohdl` binary (workspace-relative build, or `PATH`), with a settings key to override the path explicitly, rather than the doc snippet's hardcoded `/path/to/cohdl`.
- Close RFC-014's still-open real-client acceptance item: this RFC's own test/verification step is running the extension against a real fixture in an actual VS Code session, not just unit-testing `cohdl lsp` in isolation again.
- Do this as a thin packaging/grammar layer over `cohdl lsp` — zero new diagnostic logic, zero new checks, exactly the same "purely a new transport/frontend" discipline RFC-014 itself established for the LSP server relative to the compiler pipeline.

## Non-goals

- **Not a new diagnostic, check, or compiler capability.** This RFC adds nothing `cohdl check`/`cohdl lsp` doesn't already produce — it is packaging, syntax highlighting, and a settings surface only.
- **Not marketplace publishing/CI automation for publishing** — this RFC produces a buildable, installable `.vsix` and the source to build it; actually publishing to the VS Code Marketplace (account, versioning cadence, publisher identity) is a distribution/ops decision out of this RFC's scope, similar to how RFC-017 explicitly separated "what a library is" from "hosting infrastructure."
- **Not other editors.** Neovim/Emacs/other LSP clients already have their own snippets in `docs/lsp.md` (RFC-014) and need no packaging — a generic LSP client just needs the launch config, which already exists. This RFC is VS-Code-specific because VS Code is the only client that needs bespoke syntax-highlighting grammar packaging to get color at all (Neovim's/Emacs's Tree-sitter or regex-based highlighting is a separate, out-of-scope concern per editor).
- **Not a debugger, formatter-on-save wiring beyond calling **`cohdl fmt`**, or snippet library** for common device/trait boilerplate — these are plausible future extension features, not required to close RFC-014's deferred scope.

## Design

### Directory layout (new, in-repo)

```javascript
editors/vscode/
  package.json           — extension manifest (name, activation events, contributes)
  language-configuration.json — brackets/comments/auto-closing pairs for .cohdl
  syntaxes/cohdl.tmLanguage.json — TextMate grammar
  src/extension.ts        — activation: spawns `cohdl lsp`, wires vscode-languageclient
  README.md               — build/install instructions
```

### Language registration + grammar (static, LSP-independent)

`package.json`'s `contributes.languages` registers `.cohdl` as language id `cohdl`; `contributes.grammars` points at `syntaxes/cohdl.tmLanguage.json`. The grammar's scope coverage is derived directly from the real, Accepted grammar (notes 10/RFC-001–018, not invented): keywords (`device`, `trait`, `part`, `fn`, `design`, `impl`, `pub`, `use`, `pad`, `footprint`, `pins`, `spec`, `net`, `nc`, `variants`, `layout`), the ten RFC-001 unit-literal suffixes (as a single regex class, not ten separate rules), attribute syntax (`#[...]`), and string/comment tokens. This is a real authoring task (grammar-writing), not automatically derivable from the compiler's own grammar definition in this pass — noted honestly, not glossed over (see Non-goals' sibling concern in Failure modes).

### Client wiring (`src/extension.ts`)

```typescript
import { LanguageClient, LanguageClientOptions, ServerOptions } from "vscode-languageclient/node";
import * as vscode from "vscode";

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
  const cohdlPath = vscode.workspace.getConfiguration("cohdl").get<string>("path") ?? "cohdl";
  const serverOptions: ServerOptions = { command: cohdlPath, args: ["lsp"] };
  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "cohdl" }],
  };
  client = new LanguageClient("cohdl", "CoHDL", serverOptions, clientOptions);
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client?.stop();
}
```

- `cohdl.path` — a new, single settings key (default `"cohdl"`, resolved via `PATH`) — replaces the doc snippet's hardcoded `/path/to/cohdl` with a real, discoverable setting. No other settings in v1 — no config surface beyond what's needed to find the binary.
- Identical `documentSelector`/spawn shape to the doc snippet already in `docs/lsp.md` — this RFC packages that snippet, it does not redesign the wiring.

## Type-system-first test

N/A — this RFC introduces no new checkable language construct and no `rule`/DRC candidate. It is packaging and a static grammar file; the only "correctness" property is that the extension faithfully reflects `cohdl lsp`'s existing, already-tested output (see Gradeability).

## Conceptual impact

**None.** No new core concept, no new keyword, no new grammar in the `.cohdl` language itself — the TextMate grammar is a separate artifact describing `.cohdl` syntax for VS Code's own highlighter, not a change to what `.cohdl` source means. This is pure Layer-4 tooling, the same classification RFC-014 itself carried.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | Low | Low | Low | Low | Med |

**Trust (Med):** the one real risk is the TextMate grammar drifting from the real language grammar over time (a keyword renamed in a future RFC but not updated in `cohdl.tmLanguage.json` — see Failure modes) — this is a maintenance discipline risk, not a correctness risk to compiled output (the grammar file has zero influence on `cohdl check`/`cohdl build`).

**Everything else (Low):** no new concept, no grammar change to `.cohdl` itself, no new diagnostic, no new netlist surface, no compatibility break — this is additive packaging only.

## Gradeability

Two mechanically-checkable properties, both new to this RFC:

1. **Equivalence, transitively inherited from RFC-014**: the extension's diagnostics/hover/goto-def/references are exactly `cohdl lsp`'s output, unmodified — there is no new logic in `extension.ts` to diverge. Verified by RFC-014's existing equivalence suite (`tests/lsp.rs`) continuing to pass unchanged; this RFC adds no new server-side test surface.
2. **Grammar coverage regression test**: a fixture `.cohdl` file (drawn from the real std library / example boards) tokenized against `cohdl.tmLanguage.json`, asserting every keyword/literal-class token gets a scope (not falling through to plain text) — a new, small test this RFC does introduce, run via `vscode-tmgrammar-test` or equivalent, checked in CI.

Neither check runs inside `cohdl check`/`cohdl build` — this is tooling-repo CI, not the compiler pipeline, consistent with RFC-009/010/014's own tooling tests living alongside (not inside) the compiler's own test suite.

## AI-generatability

N/A for the extension itself (it's authored once, not generated per-project). Indirectly relevant: this RFC makes `.cohdl` source easier for a *human* reviewer to read (syntax color, inline diagnostics) — directly serving the Constitution's "AI writes, human reviews" loop, the same review-loop persona RFC-014's hover/goto-def capabilities were built for.

## Alternatives

- **Ship only the doc snippet, treat that as sufficient (status quo)** — this is literally what RFC-014 left in place, and its own text already named the gap ("a full marketplace extension... is separate scope") rather than pretending the snippet was the whole job. Not chosen because Tony directly requested the next RFC close it.
- **A generic, editor-agnostic syntax-highlighting definition (e.g. a Tree-sitter grammar) instead of a VS-Code-specific TextMate grammar** — considered, not chosen for v1: Tree-sitter grammars are more powerful (used by Neovim, Zed, GitHub's own highlighter) but a materially larger authoring investment, and no other editor's snippet in `docs/lsp.md` currently asks for one. TextMate is the minimum viable grammar format for the one editor (VS Code) that actually needs packaged highlighting today; a Tree-sitter grammar is a plausible future RFC once cross-editor highlighting is a real, named need — not speculative generality now.
- **Auto-generate the TextMate grammar from the compiler's own lexer/parser definitions** — attractive in principle (single source of truth, no drift risk), rejected for v1 as real, nontrivial tooling-generation work with no existing scaffolding in the repo to build on; the grammar is hand-authored this pass, with the drift risk explicitly disclosed (Failure modes) rather than solved by generation machinery that doesn't exist yet.
- **Bundle a full snippet library / debugger / build-task integration in the same RFC** — rejected as scope creep beyond RFC-014's specific deferred item; these are separate, additive extension features that can be their own future RFCs once the base extension is real and in use.

## Compatibility

**None.** No existing `.cohdl` source, diagnostic code, designator, or netlist byte is affected — this RFC adds a new, optional, separately-installed artifact. A user who never installs the extension experiences zero change.

## Tooling & operations

- New CI job: build the `.vsix` (via `vsce package` or equivalent) and run the grammar coverage regression test (see Gradeability) on every push touching `editors/vscode/`.
- `editors/vscode/README.md` documents: build from source (`npm install && npm run package`), install the resulting `.vsix` locally (`code --install-extension`), and configure `cohdl.path` if the binary isn't on `PATH`.
- Does not touch `check --json`'s schema, the error-code registry, or any existing CLI surface — this RFC's only new "public surface" is the `cohdl.path` VS Code setting itself.

## Teaching cost

Low. A human reviewer installs the extension exactly the way they'd install any other language extension (marketplace or local `.vsix`) — no new concept to learn beyond "set `cohdl.path` if needed." For library/RFC authors, the TextMate grammar itself needs no changes per ordinary RFC (only RFCs that add/rename a keyword need a corresponding grammar update — see Failure modes and Migration path).

## Failure modes

- **The TextMate grammar drifts from the real, Accepted grammar** (a future RFC adds/renames a keyword — e.g. this project's own recent `pad`/`footprint` naming correction — but `cohdl.tmLanguage.json` isn't updated) — this is a real, named risk this RFC does not fully close mechanically; the grammar coverage regression test (Gradeability) catches *some* drift (a keyword falling through to unstyled plain text) but cannot catch a keyword being *mis-highlighted* as the wrong token class. Mitigation: the RFC template's own "ship with its spec update" discipline (note 6, lifecycle step 6) is extended by convention — any future RFC introducing/renaming/removing a top-level keyword should touch `cohdl.tmLanguage.json` in the same change, the same way it must touch note 10. This is a process discipline note, not a compiler-enforced guarantee (there is no way to compiler-enforce a VS Code grammar file's correctness).
- `cohdl.path`** resolves to a stale or wrong binary** (e.g. an old build on `PATH` from before a breaking RFC landed) — the extension has no version-compatibility check between itself and the `cohdl` binary it spawns; a mismatch could silently show stale diagnostics. Named here as a real, currently-unaddressed risk, not solved by this RFC — a future RFC could add a `cohdl lsp --version`-style handshake if this proves to matter in practice.
- **The extension's activation fails silently if **`cohdl`** isn't found at all** (typo'd `cohdl.path`, binary not built) — must surface as a real, visible VS Code error notification (`window/showMessage` equivalent, or the client's own connection-failure UI), never a silent no-op with a blank Problems panel that looks like "the file has no errors."

## Migration path

N/A — pre-launch, purely additive. Existing `.cohdl` projects need no changes to benefit from installing this extension.

## Decision

**Accepted — 2026-07-15.** Recorded as DR-025 (see note 7). Closes RFC-014's own explicitly-deferred scope ("a full marketplace extension (grammar, packaging) is separate scope") and its still-open real-client acceptance item (a live VS Code session actually exercising `cohdl lsp`). Zero new diagnostic logic, zero new checks — pure packaging + a static, hand-authored TextMate grammar over the already-Accepted, already-tested `cohdl lsp` server. Language Specification (note 10) gains a short "Editor support (VS Code extension)" tooling note; no construct-level entry is needed since no language semantics change.
