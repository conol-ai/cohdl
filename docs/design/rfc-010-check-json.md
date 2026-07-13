# RFC-010: cohdl check --json schema

## Problem

The MVP implementation's `cohdl check`/`build` render diagnostics as human-readable text only — `main.rs`'s own header comment states the cut explicitly: "no fmt, no LSP, no --json." This is fine for a human staring at a terminal, but the redesign's actual interaction model (DR-002: text-in/text-out, AI writes `.cohdl` directly) means the **primary consumer of a verdict is a repair-loop program**, not a human eyeballing colored terminal output. Today, the only way for `harness/repair_loop.py` (or any future tool) to know "did it pass, and if not, what exactly is wrong and where" is to parse the same string a human would read — a brittle contract that breaks the moment a diagnostic's wording changes, and that already forces the existing harness into text-scraping it shouldn't need to do.

Who this is for: **tool builders** (the repair-loop harness, any future LSP/editor integration, CI systems) who need a **stable, structured, versioned contract** for "what did `cohdl check` find," decoupled from the human-facing rendered text.

## Goals

- Define a JSON schema that exposes exactly what the real `Diagnostic` struct (`src/diag.rs`) already carries — `code`, `severity`, `message`, a primary label (span + message), secondary labels, and `help` suggestions — with zero information loss and zero invention of new diagnostic content.
- Make the schema **versioned** from day one, so future additions (e.g. RFC-011's formal error-code registry) are additive, not breaking.
- Decouple the repair-loop harness (and any future tool) from human-readable text-scraping entirely.

## Non-goals

- **Not changing what diagnostics say or which checks run** — this RFC exposes the existing diagnostic pipeline's output in a new format; it does not add, remove, or reword any check.
- **Not formalizing error codes** — that's RFC-011's job. This RFC's schema has a `code: string` field whose values are today's informal registry (`docs/error-codes.md`); RFC-011 formalizing the registry doesn't change this schema's shape, only the guarantees behind the `code` field's values.
- **Not a general compiler-plugin or query API** — no incremental compilation, no "ask about a specific span" query mode. `cohdl check --json` is a single request/response: compile the given project, return the full diagnostic list as JSON, exit.

## Design

### Invocation

`cohdl check [PATH] --json` — same arguments as plain `cohdl check`, with one additional flag. When `--json` is present, **all output goes to stdout as a single JSON document** (see schema below) and human-readable rendering is suppressed entirely (no mixing of text and JSON on the same stream) — stderr is reserved for genuine tool-invocation failures (bad arguments, missing project) that occur before any diagnostic collection begins, mirroring the plain-text CLI's existing `ExitCode::from(2)` argument-error path.

### Schema

```json
{
  "schema_version": 1,
  "verdict": "fail",
  "diagnostics": [
    {
      "code": "E110",
      "severity": "error",
      "message": "expected `Voltage`, found `Capacitance`",
      "primary": {
        "file": "src/main.cohdl",
        "start_line": 12,
        "start_col": 18,
        "end_line": 12,
        "end_col": 24,
        "message": "this net annotation must be Voltage-typed"
      },
      "secondary": [
        {
          "file": "src/main.cohdl",
          "start_line": 8,
          "start_col": 5,
          "end_line": 8,
          "end_col": 9,
          "message": "capacitance declared here"
        }
      ],
      "help": [
        "did you mean `net VBUS [5V]: ...`?"
      ]
    }
  ]
}
```

- `schema_version` — an integer, incremented only on a breaking change to this schema's own shape (not on new diagnostic codes or messages, which are ordinary content, not schema changes).
- `verdict` — `"pass"` or `"fail"`, computed identically to the existing CLI's exit-code logic (`diagnostics.has_errors()` — any `Severity::Error` present means `"fail"`; a design with only warnings is `"pass"`, matching today's exit-code behavior exactly).
- `diagnostics` — the full, ordered list, one entry per `Diagnostic` the pipeline already produces (parse, resolve, type-check, residual DRC — every stage's diagnostics go into the same flat list, exactly as the existing `Diagnostics` collector already does; no per-stage nesting).
- Each diagnostic entry maps directly from the real `Diagnostic` struct: `code` (the `&'static str`), `severity` (`"error"` or `"warning"`, lowercased from the existing `Severity` enum), `message` (top-level message), `primary` (the primary `Label`, span resolved to 1-based `file`/`start_line`/`start_col`/`end_line`/`end_col` via the existing `SourceMap`, plus the label's own `message`), `secondary` (zero or more, same shape as `primary` minus the "primary" designation), `help` (the existing `Vec<String>` of suggestion lines, verbatim).
- **Span resolution**: `file` is the source-relative path (matching what the text renderer already shows); line/column are 1-based, matching `SourceMap`'s existing `LineCol` (already 1-based per its own doc comment) — no new position-encoding scheme invented.

### `cohdl build --json`

Same `--json` flag, same diagnostic schema, plus (only on `"pass"`) a `build` object naming the emitted artifact paths (`netlist`, `bom`) — mirrors what the plain-text `build` command already prints on success, structured instead of prose.

## Type-system-first test

N/A — this RFC is a tooling/API-surface mechanism, not a `rule`/DRC proposal. It exposes the existing diagnostic pipeline's output; it adds no new check.

## Conceptual impact

None. No new concept, no new diagnostic content — a structured re-projection of data the `Diagnostic`/`Span`/`SourceMap` types already hold. This is Constitution-aligned tooling-as-product-surface, the same category as RFC-009's `fmt`.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | Low | High | Low | Low | High |

**Diagnostics (High):** this RFC's entire content is the diagnostic-surface contract — the schema must be complete and stable enough that a repair-loop program never needs to fall back to text-scraping for anything the plain-text renderer shows a human.
**Trust (High):** a versioned, schema'd contract is what lets a tool builder depend on `cohdl check --json` without fear of silent breakage on a wording change — directly serves the redesign's "trust" dimension the same way RFC-009's canonical form serves diff-stability.
**Grammar/Oracle/Netlist (Low):** no grammar change, no change to what's checked or how netlists are emitted — pure output-format addition.
**Compat (Low):** purely additive — a new flag on an existing command; plain-text output is completely unchanged for anyone not passing `--json`.

## Gradeability

`cohdl check --json`'s own correctness is checked by a direct **equivalence test**: for every existing fixture, the diagnostics reported in `--json` mode (decoded and compared field-by-field) must exactly match the diagnostics the plain-text renderer would produce for the same input — same codes, same severities, same spans, same messages, same ordering. This is mechanically checkable (deserialize the JSON, compare against the existing `Diagnostic` values the pipeline already produced) and belongs in the same `tests/exit_criteria.rs`-style suite the project already uses.

## AI-generatability

High, and this is the direct payoff: with a stable JSON contract, a repair-loop program can programmatically extract "code + span + message + help" per diagnostic and hand exactly that to the model, instead of a full rendered text blob (which mixes signal with formatting the model doesn't need to reparse). This is a tooling improvement for the harness, not a change to what the AI author writes.

## Alternatives

- **Extend the existing plain-text renderer with a machine-parseable prefix format** (e.g. compiler-style `file:line:col: code: message` one-liners) instead of full JSON — rejected: still couples tooling to a text format that can drift, and loses structure (secondary labels, help lines, multi-line messages) that JSON represents naturally without escaping gymnastics.
- **A general LSP server** exposing diagnostics via the Language Server Protocol — rejected for MVP scope per note 9's cut list ("full LSP" is explicitly cut); `--json` is the minimal structured-output mechanism that unblocks the repair-loop harness without building a stateful, protocol-speaking server.
- **Nest diagnostics per pipeline stage** (`parse_diagnostics`, `typecheck_diagnostics`, `drc_diagnostics`) instead of one flat list — rejected: the existing `Diagnostics` collector already flattens across stages (diagnostics don't currently self-report which stage produced them), and inventing that categorization for `--json` alone would mean the JSON schema carries information the plain-text renderer doesn't have access to either — a divergence this RFC's equivalence-test gradeability principle forbids.

## Compatibility

Purely additive — a new flag, a new output mode. No existing invocation of `cohdl check`/`build` (without `--json`) changes behavior in any way.

## Tooling & operations

- `schema_version` must be checked by any consumer before parsing further fields — a tool reading `schema_version: 2` with only v1-schema-aware code should fail loudly, not guess.
- The repair-loop harness (`harness/repair_loop.py`) should migrate to `--json` in the same change that lands this RFC, retiring whatever text-scraping it currently does.
- This is the schema RFC-011 (error-code registry) will eventually formalize the `code` field's guarantees against — RFC-011 should not need to change this schema's shape, only strengthen what `code` values mean.

## Teaching cost

Low — tool builders read one schema document; `.cohdl` authors are unaffected (this is a CLI-flag/tooling change, not a language change).

## Failure modes

- `--json`** output silently diverges from plain-text output** (e.g. a future diagnostic added to the renderer but not surfaced in JSON, or vice versa) — must be caught by the mandatory equivalence test described in Gradeability; any new diagnostic-producing code path must be exercised by both output modes in the same test.
- **A consumer parses **`--json`** output without checking **`schema_version` and breaks on a future schema change — mitigated by making `schema_version` the very first field and documenting the check requirement prominently; a genuine misuse risk this RFC can document but not fully prevent at the language level.
- **Mixing **`--json`** with other flags that also produce prose output** (there are none today, but future flags should respect this) — `--json` must always mean "stdout is exactly one JSON document, nothing else," a discipline future CLI additions must preserve.

## Migration path

Land the `--json` flag and schema together with migrating `harness/repair_loop.py` off text-scraping, in the same implementation pass — per the project's established "ship with its check" / "ship with its consumer" discipline (RFC-008's std-library migration, RFC-009's formatter-run-on-existing-repo precedent).

## Decision

**Accepted** — 2026-07-13. Recorded as DR-019 (see note 7). Language Specification (note 10) gains a "Structured diagnostics (`cohdl check --json`)" section documenting the schema as part of the compiled reference. RFC-011 (error-code registry) can now proceed knowing this schema's `code` field shape is fixed; RFC-011 only needs to strengthen the *guarantees* behind `code` values, not restructure this schema.
