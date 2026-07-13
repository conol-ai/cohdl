# RFC-011: Error-code registry (formal v2 baseline)

## Problem

The MVP shipped an **informal-but-stable** registry (`docs/error-codes.md`) with its own header stating "RFC-011 (formal registry) is cut from the MVP." That informal registry is already large and mostly coherent — E0xx (lex/parse) through E8xx (designators/parts) plus D00x (residual DRC) — but auditing it against real source and against the RFCs accepted since the MVP shipped (RFC-008/009/010) surfaces three concrete, real gaps this RFC must close, not just "formalize":

1. **RFC-008 (structural variants) reserved three new diagnostics** (missing pin-role annotation, undeclared variant selected at instantiation, missing `pins[VARIANT]` block for a declared variant) **but no code block exists for them** — grepping the real implementation confirms none of the three is wired to any code at all yet. This is a live gap between the design repo and the registry, not a hypothetical one.
2. **The compliance report's own "housekeeping" notes** (`docs/compliance-report.md`): unit-mismatch checks at *generic* sites report as `E402`/`E404` — inside the E4xx (generics) block, not the E1xx (units) block where every other unit-mismatch diagnostic lives — and a standalone Unicode `Ω` (not attached to a number) reports under `E001` (lex/parse catch-all) rather than `E101` (the dedicated non-ASCII-unit-spelling code). Both are real, already-flagged inconsistencies in the current informal registry.
3. **No block reserved for RFC-010's **`--json`** schema-level errors** (e.g. malformed `--json` invocation itself, as opposed to diagnostics *within* a valid JSON response) — a minor gap, but one this RFC should close while it's formalizing block ownership.

Who this is for: primarily **tool builders** (the repair-loop harness, any future editor integration) who need `code` to be a **permanently stable identifier** they can pattern-match on without re-reading prose; secondarily the **AI author**, who benefits from a code that always means the same thing so a model doesn't have to re-learn a code's meaning across sessions.

## Goals

- Formalize the existing informal registry's **stability guarantee** as an actual, checked constraint: a code is issued once, is never repurposed, and is only ever deprecated (kept documented, marked unused) — never silently reassigned to a different meaning.
- Close the three concrete gaps found above: reserve a code block for RFC-008's structural-variant diagnostics, relocate/re-home the two misfiled codes the compliance report already flagged, and reserve a block for RFC-010 invocation-level errors.
- Make the registry's own completeness mechanically checkable against the compiler source (every `Diagnostic::error`/`Diagnostic::warning` call site's code must appear in the registry; every registry entry must have at least one real call site) — closing the exact "structurally present but not actually enforced" failure class the whole redesign exists to avoid, applied here to documentation-vs-code drift instead of DRC-vs-type-system drift.

## Non-goals

- **Not renumbering or removing any code that is already correctly placed** — E001–E804, D001–D004 keep their existing values; this RFC only touches the specific three gaps identified above, plus formalizing the stability rule going forward.
- **Not defining new diagnostics beyond what RFC-008 already specified** — the three missing structural-variant codes are RFC-008's own stated (but never wired) reservations, not new invention by this RFC.
- **Not building tooling beyond a compile-time/test-time completeness check** — no runtime registry service, no external documentation site; the registry stays a single markdown file plus a mechanical cross-check, consistent with the project's low-infrastructure-cost tooling precedent (RFC-009/010 were both pure-compiler-internals additions, not services).

## Design

### The registry stays one file, with an explicit stability rule stated once at the top

```markdown
# Error-code registry (formal, v2 baseline — RFC-011)

A code is issued once and never repurposed. If a check's behavior changes
enough that its old meaning no longer applies, retire the code (mark
`[DEPRECATED]`, keep the row, keep the meaning documented) and issue a new
one — never edit an existing code's meaning in place.
```

This is the same rule the informal registry already stated informally; this RFC makes it the enforced contract (see Gradeability) rather than a documentation convention.

### Fixing the two real misfilings the compliance report flagged

- `E402`**/**`E404`** (unit-mismatch at a generic site) move into the E1xx block**, becoming `E112`/`E113` — unit-mismatch is unit-mismatch regardless of whether it's caught at a plain spec field or a generic-parameter substitution site; keeping them in E4xx (generics) was an artifact of *where in the compiler* the check runs, not *what kind of mistake* it is, and the registry's block structure is defined by mistake-kind, not call-site location (see the block table below). E4xx's remaining codes (E401, E403, E405, E406) are unaffected — they're genuinely generics-specific (wrong arg count, unsatisfied trait bound, non-concrete substitution, invalid parameter declaration), not unit mismatches.
- **A standalone Unicode **`Ω`** gets its own code, **`E107`, inside the E1xx block, instead of falling through to the generic lexer catch-all `E001`. (Not `E101`, which is reserved for "non-ASCII unit spelling directly after a number" — a standalone `Ω` with no preceding number is a different, narrower case worth its own code so the message can be maximally specific, per the Constitution's "name the exact... never a bare mismatch" diagnostic-quality principle.)

### Closing the RFC-008 gap: a new E9xx block

```markdown
## E9xx — structural variants (RFC-008)

| Code | Meaning |
|---|---|
| E901 | pin declared without a required role annotation — lists the six valid roles |
| E902 | instantiation of a device with declared variants is missing its `[VARIANT]` selector — lists the valid variant set |
| E903 | undeclared variant named at an instantiation `[VARIANT]` selector — lists the valid variant set |
| E904 | device `variants {}` declares a variant with no corresponding `pins[VARIANT]` block — names the missing variant |
| E905 | duplicate variant name in a `variants {}` declaration |
```

(Five codes, not RFC-008's originally-estimated three — auditing the actual exhaustiveness rules surfaced two additional distinct failure shapes: E902/E903 are the two different ways an instantiation's variant selector can be wrong, which RFC-008's RFC text conflated into one "missing selector" case.)

### Closing the RFC-010 gap: reserving invocation-level codes

```markdown
## E00x — CLI invocation (not a source diagnostic)

| Code | Meaning |
|---|---|
| E000 | malformed CLI invocation (bad flags, missing path) — exit code 2, never appears inside a `--json` diagnostics array |
```

This documents the existing `ExitCode::from(2)` argument-parsing failure path (`main.rs`) as `E000` for completeness, distinguishing it explicitly from every other code (which always appears as a `Diagnostic` inside the pipeline's output, JSON or text) — `E000` is a pre-pipeline invocation failure, never part of the `diagnostics` array RFC-010 defined.

### Block ownership table (the registry's actual organizing principle, made explicit)

| Block | Owner mechanism | Rationale |
|---|---|---|
| E00x | CLI invocation | Pre-pipeline, not a source diagnostic |
| E0xx (E001-E010 range, renumbered where needed) | Lexing & parsing | Nothing RFC-specific — grammar-level |
| E1xx | Unit system (RFC-001) | All unit-mismatch/unit-literal diagnostics, regardless of call site |
| E2xx | Name resolution | Nothing RFC-specific — universal to any named reference |
| E3xx | Trait satisfaction at impl (RFC-003) |  |
| E4xx | Generics (RFC-007), excluding unit-mismatch (which is E1xx) |  |
| E5xx | Sub-circuit fns (RFC-006) |  |
| E6xx | Design assembly & nets |  |
| E7xx | Pin connection obligations (RFC-002) |  |
| E8xx | Designators & parts (RFC-005) |  |
| E9xx | Structural variants (RFC-008) |  |
| D00x | Residual DRC (RFC-004) — exactly four, never more |  |

**The organizing key is "what kind of mistake," not "which compiler pass catches it"** — this is the principle that the E402/E404 misfiling violated and this RFC corrects.

## Type-system-first test

N/A — this RFC formalizes documentation/registry discipline over diagnostics the type system and residual DRC already produce; it adds no new check.

## Conceptual impact

None. No new concept — a formalization of an already-existing informal artifact (`docs/error-codes.md`), reorganized by a principle (mistake-kind, not call-site) that was already implicit but not stated, and extended to cover two RFCs (RFC-008, RFC-010) that landed after the informal registry was written.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | Low | High | Low | Med | High |

**Diagnostics (High):** this RFC's entire content is the diagnostic-code contract's completeness and stability — directly the redesign's own stated Diagnostics dimension.
**Trust (High):** a mechanically-checked "every code in source has a registry entry, every registry entry has a source call site" guarantee is what makes `code` actually trustworthy for a tool builder to depend on, closing the exact kind of drift RFC-009's formatter and RFC-010's schema equivalence test both guard against in their own domains.
**Compat (Med):** relocating `E402`/`E404` to `E112`/`E113` is a real, visible renumbering for anyone (a tool, a saved test fixture) that already depends on the old codes — a one-time, documented breaking change, justified because these two codes have not yet been depended on outside this repository (MVP-stage, pre-first-release) and because leaving a misfiled code in place would violate this RFC's own newly-stated organizing principle on day one.

## Gradeability

Enforced by a **mechanical completeness test**, in the same `tests/exit_criteria.rs`-style suite: (1) every `&'static str` code literal passed to `Diagnostic::error`/`Diagnostic::warning` anywhere in the compiler source must appear as a row in `docs/error-codes.md` (a source-to-registry check, catching a code that exists in code but was never documented); (2) every registry row not marked `[DEPRECATED]` must have at least one real call site in source (a registry-to-source check, catching a documented-but-dead code, or a typo'd reservation like RFC-008's original three-code estimate that undercounted the real E9xx block). Both directions are necessary — either alone would have missed the RFC-008 gap this RFC's own drafting surfaced.

## AI-generatability

High — this RFC changes documentation/registry organization and two code values, not any diagnostic's message content or when it fires; a repair-loop model consuming `code` via RFC-010's `--json` schema is unaffected except for the two renumbered codes (E402→E112, E404→E113), which is exactly the kind of change the stability rule this RFC establishes is meant to prevent from recurring.

## Alternatives

- **Leave E402/E404 in the E4xx block, treat the compliance report's note as a documentation-only wrinkle** — rejected: the report's own language ("wrinkles to clean up when RFC-011 formalizes codes") explicitly anticipated this RFC fixing it; leaving it would mean RFC-011 shipped without doing the one thing it was flagged to do.
- **A monotonically-increasing single numeric code space (no block structure)** — rejected: the existing block-per-mechanism structure already reads well (an author/model can infer roughly what a code is about from its block on sight) and changing it would be pure churn with no benefit; this RFC's job is completing and correcting the existing structure, not replacing it.
- **Estimate RFC-008's variant-diagnostic codes without re-deriving them from the actual exhaustiveness rules** — rejected: doing the derivation properly surfaced that RFC-008's own "three sub-blocks" estimate undercounted (missing-selector splits into two distinct cases: E902 vs E903), which the completeness test in Gradeability would have caught anyway — better to get it right now than fail the test later.

## Compatibility

**Real, documented breaking change**: `E402`→`E112`, `E404`→`E113`. No other existing code changes value. This is a one-time renumbering, acceptable pre-first-release (no external consumer depends on these two codes yet) — post-launch, this exact kind of change is what the stability rule this RFC establishes exists to prevent from happening again.

## Tooling & operations

- The completeness test (source↔registry cross-check) runs in CI on every change to either the compiler source or `docs/error-codes.md` — this is the enforcement mechanism, not a one-time audit.
- RFC-010's `--json` schema is unaffected in shape — `code` is still a plain string field; only the *set of valid values* and their organization changes.
- `E000` (CLI invocation failure) is documented as explicitly outside the `--json` diagnostics array, closing a small ambiguity RFC-010 left implicit.

## Teaching cost

Low — an author/model doesn't need to memorize the registry; codes are discovered via diagnostics as they occur, same as before. The one relevant fact worth stating explicitly for anyone hand-authoring test fixtures: `E402`/`E404` are retired names for `E112`/`E113`.

## Failure modes

- **A future diagnostic is added to source without a matching registry row** — caught by the mandatory source-to-registry completeness test; this is precisely the "structurally present but not actually enforced" failure class DR-006 named for DRC rules, now guarded against for documentation.
- **A registry row is written speculatively ahead of any real call site** (as RFC-008's original three-code estimate effectively was) — caught by the registry-to-source direction of the same test; a reserved-but-unimplemented code should be explicitly marked `[RESERVED, not yet implemented]` rather than silently present as if wired.
- **Someone repurposes a retired code's number for an unrelated new diagnostic** — explicitly forbidden by the stability rule; the completeness test doesn't catch semantic repurposing directly, so this remains a review-discipline requirement (documented here, not automatable further at this stage).

## Migration path

Land the `E402`→`E112`/`E404`→`E113` renumbering, the `E101`/`E107` standalone-`Ω` split, the new E9xx block (with real call sites wired in the same pass — not just registry rows), and the E00x block, all together with the completeness test — per the project's established "ship with its check" discipline. Any existing test fixture referencing `E402`/`E404` by name must be updated in the same change.

## Decision

**Accepted** — 2026-07-13. Recorded as DR-020 (see note 7). Language Specification (note 10) gains an "Error-code registry" section summarizing the block-ownership table and stability rule (full code-by-code listing stays in `docs/error-codes.md`, not duplicated into note 10, to avoid two sources of truth for the same content — note 10 references it). Flags a real, concrete implementation gap (RFC-008's three diagnostics were never wired to any code; the true count is five, not three) for immediate follow-up alongside whatever build lands this RFC's registry changes.
