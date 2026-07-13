# RFC / decision-record compliance report

**Exit criterion** (docs/design/09-mvp-definition.md): *"Every P0 RFC's
decision record accurately reflects what was actually built — no RFC claims a
mechanism that the implementation doesn't actually enforce."*

**Method** (2026-07-13): eight independent audit agents — one per Accepted RFC
(RFC-001…007) plus one for the Product Constitution's hard constraints — each
read its document in full, enumerated every normative claim, and classified it
as *verified* (with `file:line` evidence in the implementation or tests) or a
*violation*. Every reported violation was then re-examined by an independent
adversarial verifier with instructions to refute it (including running the
compiler on probe fixtures). Full per-claim evidence:
[`docs/compliance-audit.json`](compliance-audit.json).

## Summary

| Document | Claims verified | Violations confirmed | Outcome |
|---|---|---|---|
| RFC-001 units-as-types | 16 | 2 | **both fixed** (see below) |
| RFC-002 pin obligations | 16 | 0 | clean |
| RFC-003 trait satisfaction | 16 | 0 | clean |
| RFC-004 DRC reclassification | 11 | 3 | 2 fixed, 1 documented deviation |
| RFC-005 designator allocation | 18 | 1 confirmed (1 refuted) | fixed |
| RFC-006 nested fn calls | 12 | 1 | fixed |
| RFC-007 generics-over-specs | 12 | 1 | fixed |
| Product Constitution | 15 | 3 confirmed (1 refuted) | 2 fixed, 1 documented provisional |

Deduplicated across documents (three audits independently found the
net-annotation gap), the confirmed deviations were **seven distinct issues:
six fixed the same day, one retained as a documented deviation.**

## Fixed deviations (each with a regression test in `tests/exit_criteria.rs`)

1. **Net voltage annotations accepted any unit literal.** `net X [100nF]: …`
   compiled, then D001 compared capacitance against voltage — violating
   RFC-001's same-unit comparison discipline. Fixed: a non-`Voltage`
   annotation is now `E110` naming expected vs. actual
   (`src/parse.rs`, test `net_annotation_must_be_voltage_typed`).

2. **D001 compared raw magnitudes without a unit check.** A (legal but odd)
   non-Voltage `voltage_rating` spec was magnitude-compared against the net
   voltage. Fixed: D001 compares only Voltage-vs-Voltage
   (`src/drc.rs`, test `drc_d001_never_compares_across_unit_types`).

3. **RFC-005 reserved set omitted override-shadowed prior assignments.**
   Step 3 defines the reserved set over *all* prior assignments; the
   implementation used only live-and-not-overridden ones, so a fresh instance
   could take a number whose path had just been overridden. Fixed: reserved =
   all prior assignments ∪ overrides ∪ all tombstones
   (`src/lock.rs`, test `overridden_paths_prior_designator_stays_reserved`).

4. **`impl Trait` parameters used a second trait-bound-checking code path.**
   DR-016 mandates exactly one mechanism. Fixed: both named generic
   parameters and `impl Trait` value parameters now route through the single
   `check::generics::check_trait_bounds` (`src/check/generics.rs`,
   `src/check/expand.rs`; behavior covered by the existing E403 fixtures).

5. **The promised ambiguous-part-binding note was never emitted.**
   provisional §2 says the deterministic pick is "noted in the build output".
   Fixed: `cohdl build` now prints a note naming the candidates and the
   winner (test `ambiguous_part_binding_is_deterministic_and_noted`).

6. **The `__fn`/`__net` mangling namespace was user-forgeable.** RFC-006's
   collision-free naming guarantee requires the compiler-generated namespace
   to be reserved. Fixed: instance/net names beginning with `__` are `E206`
   (test `dunder_names_are_reserved`).

## Retained, documented deviations

- **D003 single-driver** — *superseded 2026-07-13 (external review F10).*
  This deviation is retired: D003 is now role-aware, firing only when a net's
  lone connected pin is a driver (`output`/`power_out`), the reading of
  RFC-004's "exactly one output-type (driver) pin connected" that is both
  faithful to the text and electrically sensible (a literal driver-count==1
  over any net would flag every ordinary signal net). A dangling
  passive/input pin is RFC-002's territory, not DRC's.

- **`pub` is parsed but not enforced.** The MVP's single flat scope has no
  visibility boundary; provisional-syntax.md §1 documents this openly. The
  adversarial verifier rejected the "correct-by-convention" reading: `pub`
  today has zero semantic weight (nothing behaves differently), so nothing is
  *claimed* that isn't enforced. Real visibility arrives with the module RFC.

- **RFC-005's "show the reserved-number set on request" debugging affordance**
  is a Tooling & operations item; MVP tooling beyond `check`/`build` is
  explicitly cut (09-mvp-definition.md). Tracked for post-MVP.

## Error-code housekeeping (resolved by RFC-011)

The two wrinkles the audits flagged in the informal registry are fixed by
RFC-011 (DR-009, docs/design/rfc-011-error-registry.md): unit-mismatch checks
at generic sites moved from `E402`/`E404` into the `E1xx` unit block as
`E112`/`E113`, and a standalone Unicode `Ω` now reports under its own `E107`
instead of the `E001` catch-all. The registry (docs/error-codes.md) is now the
formal v2 baseline, enforced in both directions by `tests/error_registry.rs`.

## External review 2026-07-13 (Codex, at 596acf8): dispositions

An independent review audited RFC-001–013 conformance adversarially. Every
high-confidence reproduction was verified; dispositions below. Fixed items
carry regression tests.

**Fixed in code:**

- F3 — fn-local `net_class` names now carry call-chain-scoped identity
  (RFC-006 style), so a layout-bearing fn is reusable across calls.
- F4 — `diff_pair`/`length_match` require *distinct* resolved nets, catching
  both direct repetition and electrically-merged aliases (E1003/E1004).
- F1 (partial) — `[tolerance: …]` accepts RFC-001 unit literals (`1ms`)
  alongside the quoted-string escape hatch; `fmt` canonicalizes unit-literal
  tolerances to the unquoted spelling.
- F5 — `build` removes a stale `<name>-layout.json` when layout metadata is
  removed from the source.
- F8 — a successful plain `build` renders warnings again (D003 was hidden; a
  regression from the RFC-010 refactor).
- F10 — D003 is role-aware per RFC-004/008: it fires only when a net's lone
  pin is a driver (`output`/`power_out`).
- F12.2 — `10kΩ` produces one targeted E101 (`write \`10kohm\``), not an
  E103+E107 cascade.
- F6/F7/F11 — `fmt` keeps attribute spans (trailing comments on `#[intent]`/
  `#[placement_hint]`/`#[designator]` survive; comments between an attribute
  and its target stay between); comments inside `layout {}`, trait, impl, and
  part bodies are preserved in place (comment maps are consume-once, so
  double-emission is impossible by construction); pin buses, AVL entries,
  variants lists, and layout constraints wrap at the 100-column soft target;
  author blanks after `{` are preserved. The four >100-column std lines are
  rewrapped.
- Gradeability — the RFC-010 equivalence suite now *decodes* the JSON (local
  parser, no deps) and compares every field (secondary labels, help, end
  positions, build object); the RFC-012 non-impact suite compares verdict,
  diagnostics, `--json` document, netlist, BOM, and the designator lock, on
  clean and failing/warning fixtures.

**Documented contracts (deliberate dispositions, not code changes):**

- F9 — invocation-level failures (bad flags, missing project, design
  selection, nothing-to-build) are the `E000` class: exit 2, prose on stderr,
  never a JSON document. Machine consumers key on the exit code (the harness
  already does). Documented in README and docs/error-codes.md; tested in
  tests/cli.rs.
- RFC-012 target set — the parser accepts `#[intent]` on `design` and call
  statements (a superset of the spec's list, matching the RFC's own "any
  top-level or body statement" heading). Zero-impact makes the superset
  harmless; kept pending note-side resolution of the RFC's internal conflict.
- RFC-013 zero-impact wording — the tested guarantee is precisely scoped in
  docs/layout-json.md: *valid* layout metadata never changes any observable
  output; an *invalid* block is an ordinary compile error (E1001–E1004).
- `layout.json` — the versioned output contract RFC-013 requires is now
  documented (docs/layout-json.md).

**Note-side items (need conol.ai amendments, not repo changes — the
`docs/design/` snapshot is extraction-only):**

- RFC-013's unquoted `[tolerance: 0.15mm]` example cannot lex: RFC-001's
  closed ten-type set has no length unit. Either a Length unit type RFC or an
  RFC-013 amendment blessing the string form is needed.
- RFC-013's E1005 ("net_class referenced before declaration") is
  unrepresentable in its own four-kind grammar — nothing references a class
  by name. Kept `[RESERVED]` in the registry until the vocabulary grows.
- RFC-013's normative placement example (`#[placement_hint] inst esp` as a
  *re*-annotation of an already-declared instance) is not valid grammar.
- RFC-011's accepted E9xx table (five codes) conflicts with the
  earlier-shipped E901–E908; the registry documents the reconciliation, but
  the accepted RFC/DR text still needs amending.
- The language spec's error-block table omits E10xx (its own layout section
  references it).
- RFC-002's "omitted obligation defaults to required" (implemented, and used
  by the spec's own examples) contradicts the RFC's explicit-obligation
  wording; RFC-001's in-`rule` comparison surface and RFC-007's literal
  impl-Trait desugaring remain out of scope pending `rule` syntax / a
  dedicated refactor, as previously documented above.
