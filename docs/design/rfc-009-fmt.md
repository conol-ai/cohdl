# RFC-009: cohdl fmt canonical form

## Problem

The MVP implementation shipped with no `fmt` command at all (`main.rs`'s own header comment: "no fmt, no LSP" — the MVP surface is deliberately just `check`/`build`). In its absence, the real std library and examples already show concrete style drift: `std/connectors.cohdl` and `std/audio.cohdl` were updated for RFC-008's pin-role annotations, but `std/passives.cohdl` was not (`pins { A: 1, B: 2 }`, still missing `[passive]` on every pin) — a live, present-tense example of exactly the "diffs become unstable without a canonical form" problem the Constitution already named as a generatability constraint, not polish. Worse, `std/connectors.cohdl`'s own `optional SBU1: A8` / `optional SBU2: B8` lines are missing their role annotations entirely (a real gap RFC-008's migration should have caught) — a second, independent symptom of not having a single authoritative form to check output against.

Who this is for: primarily the **AI author**, whose repair-loop diffs (RFC-003/004's whole reason for existing — a diagnostic pointing at the smallest responsible span) are only meaningfully "small" if the surrounding formatting doesn't also churn on every regeneration. Secondarily the **human reviewer**, who benefits from every `.cohdl` file in the repository looking the same regardless of which agent or person wrote it.

## Goals

- Define **one canonical textual form** for every construct RFC-001 through RFC-008 introduced — units, pins (with role brackets), traits, free-standing `impl`, designators/attributes, residual-DRC-relevant annotations (net voltage/gnd brackets), `fn`/nested calls, generics (turbofish), and structural variants (`variants {}`, `pins[VARIANT]`, `spec[VARIANT]`).
- Make `cohdl fmt` **idempotent** (`fmt(fmt(x)) == fmt(x)`) and **semantically inert** (formatting never changes a design's verdict — same parse tree, same type-check result, same emitted netlist bytes).
- Retroactively normalize the existing std library and example, surfacing exactly the kind of drift described above as part of landing this RFC (see Migration path) — not as a separate, deferred cleanup.

## Non-goals

- **Not a linter.** `cohdl fmt` normalizes whitespace, layout, and token spacing; it does not warn about semantic issues (a missing role annotation is a parse error already, not something `fmt` needs to flag separately — see Design's note on this).
- **Not configurable.** One canonical form, no style options/flags (`.cohdl-fmt.toml` or similar) — configurability is exactly the "two ways to express the same thing" smell the Constitution rejects at the *layout* level, the same way it's rejected at the *syntax* level.
- **Not solving **`cohdl check --json`** (RFC-010) or the error-code registry (RFC-011)** — those are separate P1 RFCs; this one only defines the textual canonical form.

## Design

### General rules

- **Indentation: 4 spaces**, no tabs — matches every real `.cohdl` file already in the repository (std library, example) without exception; codifying the existing convention rather than inventing a new one.
- **One statement per line** inside any block (`pins {}`, `spec {}`, `net`, `variants {}`) — already the universal practice in every real file inspected.
- **Trailing line comments** (`// ...`) are preserved verbatim and never reformatted or moved — comments carry human-authored intent/context (e.g. `std/audio.cohdl`'s per-pin I2S role comments) that `fmt` must never risk altering.
- **Blank lines**: at most one consecutive blank line; a blank line is preserved where an author put one (grouping related declarations, e.g. `std/passives.cohdl`'s `// ---- demo-board part bindings ----` section breaks), never inserted or removed except collapsing 2+ consecutive blanks to 1.
- **No trailing whitespace**; every file ends with exactly one newline.

### Per-construct canonical forms

- **Pin declarations**: `required NAME: PINSPEC [role]` / `optional NAME: PINSPEC [role]` — one space before `[role]`, no space inside the brackets. Pin buses (`A1, A12, B1, B12`) are comma-space-separated on one line unless the line would exceed the line-length rule below, in which case they wrap with continuation lines indented to align under the first pin number (already `std/connectors.cohdl`'s own practice for `GND`/`VBUS`).
- `spec {}`**/generic argument lists**: comma-space-separated, no space before the comma, one space after — `MLCC<100nF, 16V, 10%>`, `primary { mfr: "Yageo", mpn: "RC0402FR-075K1L", footprint: "..." }` (already universal practice).
- `net`** declarations**: `net NAME [annotation]: member, member, …` — annotation bracket immediately after the name (no space before `[`, matching `net VBUS [5V]:` as already written); members wrap with continuation lines indented to align under the first member after the colon, exactly as `std/connectors.cohdl`'s multi-line `net GND` block already does.
- `impl`** statements**: `impl Trait for Device {}` on one line when the body is empty (the common case per RFC-003) — never split across lines just because a body is empty. A non-empty (mapped) body uses one `role: actual` mapping per line, 4-space indented.
- `variants {}`** / **`pins[VARIANT]`** / **`spec[VARIANT]`: no space between the block keyword and `[`; variant names inside `variants { ... }` are comma-space-separated on one line unless wrapping is needed.
- **Turbofish generic calls**: `name::<Arg1, Arg2>(args)` — no space around `::`, standard comma-space inside the argument lists.
- **Line length**: soft target 100 columns before wrapping is triggered (pin buses, net member lists, long `spec {}`/`primary {}` argument lists) — chosen to match the longest real lines already in the repository (`std/connectors.cohdl`'s `USB_C_Receptacle_HRO_TYPE-C-31-M-12` footprint string) without forcing unnecessary wraps on typical short declarations.

### `fmt` is a pure function of the parsed AST, not a text-munging pass

`cohdl fmt` parses the source into the existing AST (the same parser `check`/`build` already use), then re-serializes it using the canonical rules above — it never does regex/string-level reformatting of the original text. This is what guarantees idempotence and semantic inertness by construction: two different-but-equivalent input spellings of the same construct (e.g. inconsistent spacing around a colon) produce byte-identical output, because both parse to the same AST node and the same AST node always serializes the same way.

### `fmt` does not fix missing role annotations or other parse errors

A pin missing its `[role]` bracket (like `std/connectors.cohdl`'s current `SBU1`/`SBU2` lines) is **already a parse error** per RFC-008 — `cohdl fmt` operates on a source file that must already parse successfully; it is not a repair tool and does not paper over or silently complete missing required syntax. (This is a deliberate non-goal boundary: normalizing layout and fixing missing-required-syntax are different jobs, and conflating them would make `fmt`'s behavior unpredictable — sometimes "just formatting," sometimes "also completing your source." The existing `SBU1`/`SBU2` gap in the real repository must be fixed by hand/by the type checker's diagnostic, not by `fmt`.)

## Type-system-first test

N/A — this RFC is a tooling/formatting mechanism, not a `rule`/DRC proposal.

## Conceptual impact

None. `cohdl fmt` introduces no new concept, syntax, or grammar — it is a canonical *serialization* of the grammar RFC-001 through RFC-008 already defined. This is the purest possible instance of "tooling and operations are part of the product, not an afterthought" (Constitution design principle) — a real capability with zero conceptual-vocabulary cost.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | High | Low | Low | Low | Med | High |

**Grammar (High):** defining a canonical form for *every* construct across eight prior RFCs is real surface area, even though it adds no new grammar rules — every existing construct now has an opinion about its own layout.
**Trust (High):** eliminates an entire class of "is this diff meaningful or just noise" review friction — a `git diff` on a `fmt`-clean repository only ever shows semantic changes.
**Compat (Med):** running `cohdl fmt` on the existing std library will produce a real, large diff (normalizing `std/passives.cohdl` and fixing the `std/connectors.cohdl` incompleteness) — a one-time, mechanical, and expected cost (see Migration path), not an ongoing one.
**Oracle/Diagnostics/Netlist (Low):** `fmt` is semantically inert by construction — it cannot change the verdict, diagnostics, or emitted bytes of anything, so it doesn't touch these dimensions at all.

## Gradeability

`cohdl fmt`'s own correctness is checked by two properties, both directly testable: **idempotence** (`fmt(fmt(x)) == fmt(x)` for every fixture) and **semantic inertness** (running the full pipeline — parse → resolve → type-check → residual DRC → build — before and after `fmt` produces an identical verdict and identical emitted netlist/BOM bytes for every fixture). Both are mechanically checkable regression tests, not subjective style review.

## AI-generatability

High, and this is the actual point of the RFC: once `cohdl fmt` exists, an AI author's repair-loop output can be normalized before diffing, so a diagnostic-driven fix shows up as a small, targeted diff rather than a wall of incidental whitespace changes mixed in with the real fix. This directly serves the redesign's stated generatability principle (a canonical form is "a generatability constraint, not polish," per the Constitution) — it was simply not yet built.

## Alternatives

- **Ship **`cohdl fmt`** as a thin wrapper around an existing generic formatter framework** (e.g. adapting `rustfmt`'s engine) — considered, rejected for MVP-scope reasons: CoHDL's grammar is small and specific enough that a purpose-built AST-to-text serializer (as described in Design) is simpler to get exactly right than adapting general infrastructure built for a much larger language.
- **Configurable style (indent width, line length, etc.)** — rejected per Non-goals: one canonical form only, consistent with the whole redesign's "one canonical way to express each thing" principle applied to layout, not just syntax.
- **Have **`fmt`** also auto-insert missing role annotations with a placeholder** (e.g. `[passive]` by default) — rejected: this would resurrect the exact "silent default" smell RFC-008 just eliminated, and would make `fmt` a repair tool with implicit judgment calls rather than a pure, predictable serializer.

## Compatibility

**Real, one-time diff on the existing repository** when first run: `std/passives.cohdl` gains role brackets on every pin (a mechanical fix, not a behavior change — see RFC-008's own migration precedent), and `std/connectors.cohdl`'s `SBU1`/`SBU2` role-annotation gap must be fixed by hand first (since `fmt` requires already-parsing source, per Design) before `fmt` can run on that file at all. No emitted netlist/BOM bytes change as a result of formatting itself.

## Tooling & operations

- `cohdl fmt --check` (exits non-zero if any file isn't already in canonical form, without modifying anything) should exist alongside plain `cohdl fmt` (which rewrites in place) — the former is what a CI/review gate would use.
- The repair-loop harness (`harness/repair_loop.py`) should run generated source through `cohdl fmt` before computing/displaying a diff between attempts, once this RFC ships — directly realizing the AI-generatability goal.
- Idempotence + semantic-inertness tests belong in the same `tests/exit_criteria.rs`-style fixture suite the project already uses for RFC compliance, run against every existing fixture plus the std library and demo example.

## Teaching cost

None for `.cohdl` authors beyond "run `cohdl fmt`" — the canonical form is discovered by running the tool, not memorized from a style guide (the same reasoning that makes `rustfmt`/`gofmt` low-teaching-cost in their respective ecosystems).

## Failure modes

- `fmt`** accidentally changes semantics** (e.g. reordering `spec {}` fields in a way that affects diagnostic messages naming "the first mismatched field") — must be caught by the mandatory semantic-inertness test suite; if any fixture's verdict/diagnostics/netlist bytes differ pre/post-`fmt`, that's a `fmt` bug, full stop.
- `fmt`** is not idempotent for some construct** (e.g. a comment-placement edge case near a wrapped pin bus) — must be caught by the idempotence test; every fixture is fed through `fmt` twice and the outputs compared.
- **An author or model relies on **`fmt`** to silently complete missing syntax** (the exact temptation the Alternatives section rejected) — must fail loudly: `fmt` on non-parsing source is a parse error from the existing pipeline, not a special "best effort" mode.

## Migration path

Land this RFC together with: (1) the formatter implementation, (2) running `cohdl fmt` once over the entire existing std library and example, committed as its own change, and (3) fixing `std/connectors.cohdl`'s `SBU1`/`SBU2` missing role annotations by hand first (a real, pre-existing RFC-008 compliance gap uncovered while grounding this RFC, not something `fmt` itself should paper over). This mirrors the project's established "ship with its check" discipline — an Accepted RFC that changes canonical form ships together with a repository that's actually clean, not one that's now retroactively "wrong by the new standard."

## Decision

**Accepted** — 2026-07-13. Recorded as DR-018 (see note 7). Language Specification (note 10) gains a "Canonical form (`cohdl fmt`)" section. Flags a real, pre-existing implementation gap (missing role brackets on `std/connectors.cohdl`'s `SBU1`/`SBU2`, and unmigrated `std/passives.cohdl`) for separate, immediate fixing regardless of when the formatter itself is implemented — these are RFC-008 compliance bugs already, not new findings this RFC invents.
