# RFC-012: #[intent(...)] annotations (pure metadata)

## Problem

A `.cohdl` design's source captures *what* was built (devices, traits, generics, wiring), fully checked by the type system and residual DRC. It has no place to capture *why* — the human-language rationale behind a specific choice (e.g. "100nF here specifically to meet the ESP32-S3 datasheet's decoupling recommendation, not just convention," or "10kohm pull-up chosen for USB-C sink current budget, see spec §4.2"). Today that rationale lives only in `//` comments, which are informal, unstructured, and — per RFC-009's canonical form — deliberately preserved verbatim by the formatter but never otherwise inspectable as structured data.

Who this is for: **human reviewers** (who want to see design rationale attached to the specific instance/net it explains, not scattered in prose above it) and, secondarily, **future tooling** (a hypothetical "explain this design" view, or an AI repair loop that wants to preserve stated intent when regenerating a fix) that could consume rationale as structured data instead of parsing comments.

## Goals

- Give authors (human or AI) a way to attach **structured, queryable rationale** to a declaration or statement, distinguishable from an ordinary `//` comment.
- Guarantee, by construction, that `#[intent(...)]` can **never** affect compilation — not the parse tree's semantic content, not the type checker's verdict, not residual DRC, not the emitted netlist/BOM bytes. This is the single load-bearing property the whole RFC exists to prove, per its own P2/gated priority ("gated on zero-netlist-impact").

## Non-goals

- **Not a documentation-generation system** — no rendering pipeline, no doc site. This RFC defines the annotation's grammar and its zero-impact guarantee only; what (if anything) reads and displays `#[intent(...)]` content is future tooling, out of scope here.
- **Not a replacement for **`//`** comments** — ordinary comments remain the default, lightweight way to explain anything. `#[intent(...)]` exists for the narrower case where rationale should be structured and attributable to one specific declaration, not free-floating prose.
- **Not a mechanism for encoding anything the compiler should check.** If a stated "intent" is actually a constraint (e.g. "this must never exceed 16V"), that belongs in the type system or residual DRC (RFC-001/RFC-004), not as unchecked prose in an annotation — this RFC's design must make that temptation structurally awkward to fall into (see Design, Failure modes).

## Design

### Grammar: an attribute, attachable to any top-level or body statement

```cohdl
#[intent("100nF chosen per ESP32-S3 datasheet §3.4 decoupling recommendation, not a generic default")]
inst c_esp_decouple: MLCC_100nF_16V_0402

#[intent("5.1k pull-downs per USB-C spec Table 4-15 (Rd for a UFP sink advertising default USB power)")]
net CC1: usb.CC1, r_cc1.A
```

- `#[intent("...")]` takes exactly one string literal argument — free-form human-language text, no structured sub-fields (no `#[intent(reason = "...", author = "...")]`). A single string keeps the grammar minimal and keeps the temptation to encode checkable structure inside it (see Failure modes) as far away as possible.
- Attachable to: `inst`, `net`, `nc`, `impl`, `device`, `trait`, `fn`, and `part` declarations — anywhere a `//` comment could already explain a design choice.
- **Not attachable to**: expressions, individual pin declarations within a `pins {}` block, or generic parameters — these are sub-statement granularity where an attribute would clutter the grammar for marginal benefit; a comment on the enclosing declaration covers this today, and this RFC does not need to go finer-grained to satisfy its stated goal.
- Multiple `#[intent(...)]` attributes on the same declaration are a compile error (`use one string, or use a `//` comment for anything beyond a single rationale` — see Failure modes) — exactly one, or none.

### The zero-impact guarantee, enforced structurally

`#[intent(...)]`'s string content is parsed into the AST as an **opaque, uninterpreted string attached to its target node** — the type checker, residual DRC, designator allocator, and netlist/BOM emitters never read this field at all. This is not merely a convention the implementation happens to follow; it's enforced by never threading the field into any of those passes' input types in the first place (an intent string is not a parameter any checking/emission function accepts) — the same "provably true by construction, not by testing discipline" already used for RFC-009's idempotence and RFC-010's schema equivalence.

## Type-system-first test

N/A — this RFC explicitly defines *non-checked* metadata; by design, nothing here is ever a `rule`/DRC candidate. (If a future author wants a checkable constraint, that's RFC-001/002/003/004's job, not this one's — see Non-goals and Failure modes.)

## Conceptual impact

Low. No new core concept — an attribute mechanism analogous in shape to the existing `#[designator("Xxx")]` override attribute (RFC-005) and `pub` visibility marker, both already-accepted small syntactic additions that carry metadata alongside a declaration without becoming new first-class concepts.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | Low | Low | Low | Low | Med |

**Trust (Med, the only non-Low cell):** this RFC's entire value proposition rests on the zero-impact guarantee actually holding — if `#[intent(...)]` content ever silently leaked into a checked or emitted artifact, that would be a serious trust violation (an unchecked string quietly influencing something it shouldn't). Mitigated by the structural-enforcement design above and the mandatory test in Gradeability.
**Everything else Low:** by design — this RFC is deliberately the smallest, lowest-risk addition in the entire backlog, consistent with its P2 priority and "gated on zero-netlist-impact" framing.

## Gradeability

Enforced by a direct **non-impact test**: for every fixture, compile it once as-is and once with every `#[intent(...)]` attribute's string content mutated to arbitrary different text (including strings that look like they might be checkable constraints, e.g. `#[intent("must be < 10V")]`) — the verdict, every diagnostic (code/severity/span/message), the designator assignment, and the emitted netlist/BOM bytes must be byte-identical across both runs. Any divergence is a bug in this RFC's implementation, full stop — this is the single test that matters for this RFC, and it should be as mechanically simple as RFC-009's idempotence test and RFC-010's equivalence test.

## AI-generatability

High — an AI author can attach rationale to a declaration the same way it already writes `//` comments, with a very small syntax delta (`#[intent("...")]` vs `// ...`) and a single clear rule (one string, optional, never load-bearing). No special-casing or memorization burden beyond that.

## Alternatives

- **Structured sub-fields** (`#[intent(reason = "...", ticket = "...", author = "...")]`) — rejected: adds grammar surface and encourages the "why not make `reason` checkable too" slope this RFC's Non-goals section explicitly guards against; a single string is sufficient for the stated goal (human-readable rationale) and keeps the temptation-to-encode-constraints as far away as possible.
- **A dedicated top-level **`intent { ... }`** block per design, separate from inline attributes** — rejected: this would decouple rationale from the specific instance/net it explains, reproducing the exact "scattered prose, not locally attributable" problem inline `//` comments already have; the whole point is per-declaration attribution.
- **Doc-comment syntax** (`/// ...`, Rust-style) reused as-is, with tooling that treats `///` specially — rejected: this would make "is this comment structured metadata or prose" ambiguous by convention rather than explicit syntax, the opposite of the redesign's "no implicit convention, make the fact explicit syntax" discipline (already applied in RFC-008 retiring the implicit pin-role default for the same reason).

## Compatibility

Purely additive — new optional attribute syntax; no existing source is affected, no existing diagnostic/netlist/designator behavior changes for source that doesn't use it.

## Tooling & operations

- `cohdl fmt` (RFC-009) treats `#[intent("...")]` as a single-line attribute preceding its target declaration, same placement/spacing conventions as the existing `#[designator("Xxx")]` override — no new formatting rule category needed, this reuses the existing "attribute precedes declaration" convention.
- `cohdl check --json` (RFC-010) does **not** surface `#[intent(...)]` content in the diagnostics schema — it's not a diagnostic; if future tooling wants to expose it, that's a separate, explicit addition to a future schema version, not an implicit inclusion now.
- No new error codes are needed for well-formed usage; a malformed `#[intent(...)]` (wrong argument shape, e.g. missing the string or passing a non-string) is a parse error under the existing E0xx block (attribute-argument-shape errors are already a parsing concern, not a new mechanism).

## Teaching cost

Very low — one rule to learn ("attach a rationale string to a declaration; it's never checked, purely for readers"), directly analogous to a doc comment in any mainstream language.

## Failure modes

- **An author writes a checkable-sounding constraint inside **`#[intent(...)]` (e.g. `#[intent("voltage must never exceed 10V")]`) expecting it to somehow be enforced — this RFC's design cannot prevent someone from writing misleading prose, but the zero-impact guarantee (tested per Gradeability) ensures it's never silently *partially* enforced or inconsistently interpreted; the failure mode is "the string is decorative and wrong," a human-review problem, not a compiler-correctness one. Worth stating plainly in any author-facing documentation of this feature: if you want a rule, use RFC-001/004's actual mechanisms.
- **Multiple **`#[intent(...)]`** on one declaration** — explicitly a compile error (see Design), preventing an accidental "which one wins" ambiguity.
- `#[intent(...)]`** used as a workaround to avoid writing a real **`impl`** mapping or spec** (e.g. `#[intent("trust me, this satisfies Capacitor")]` instead of an actual `impl Capacitor for X`) — structurally impossible to "work," since the attribute carries zero compiler weight; the trait-satisfaction check (RFC-003) runs exactly as if the attribute weren't there.

## Migration path

N/A — purely additive, no existing source needs to change.

## Decision

**Accepted** — 2026-07-13. Recorded as DR-010 (see note 7 — this number was reserved for this exact RFC back when DR-007's pending-list entry was carried forward from v1). Language Specification (note 10) gains a small "`#[intent(...)]` annotations" section. This closes the P1/P2 tooling backlog started at RFC-008; only RFC-013 (layout, explicitly gated pending its own goal-change proposal) remains in note 6's backlog.
