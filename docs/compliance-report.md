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

- **D003 single-driver is a count check, not a driver-role check.** RFC-004's
  table describes v1's W003 as "exactly one output/driver-type pin connected";
  v1's actual implementation (which the RFC says it maps — "mapped from v1's
  rule names, not literal ports") fired on any single-member net. A literal
  driver-count==1 reading would flag every ordinary signal net (one output,
  N inputs). The implementation keeps the v1-faithful single-connected-pin
  check and reports the lone pin's role in the message; documented in
  provisional-syntax.md §7. Revisit when `rule` syntax returns via RFC.

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
