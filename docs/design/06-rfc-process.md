# 6. Feature Proposal Process (RFC)

# Status

v2 — process carried forward, backlog reset, 2026-07-13. The RFC mechanism itself (why we have it, when it's required, the lifecycle) is exactly as effective as it was in v1 — this redesign didn't touch how we govern change, only what the change was. What's new: one mandatory template section, and a completely fresh backlog (the v1 RFC-001…010 backlog described bugs/features in an implementation that no longer exists).

# Purpose — unchanged

Major features go through a structured proposal, not a ticket. The one question every RFC must answer:

> Does this feature strengthen CoHDL's coherence — its ability to be reliably generated, gradeable, and trusted — or is it a local patch?
>

Small fixes don't need an RFC. An RFC is required when a change touches a core concept, the grammar, the oracle/verdict, the diagnostic/error-code contract, or scores High/Crit on any Coherence Matrix dimension.

# RFC template — one new mandatory section (marked NEW)

```text
$3a
```

# The lifecycle of an RFC — unchanged, one step added

1. Draft — author fills the template. Incomplete gradeability, coherence, or (new) type-system-first sections = not ready.
2. Review against the ladder — reconcile with the Constitution's priority ladder and note 4's constraints. A rank-1/rank-2 violation is a blocker.
3. **Coherence check** — run the design regression checklist (note 8) and the matrix row (note 5).
4. **Decision** — Accepted / Rejected / Delayed / Experimental. Every accepted or notably-rejected RFC gets a decision record (note 7).
5. Ship with its check — an accepted correctness-affecting RFC ships together with the mechanism that enforces it (type checker rule, or narrowly, a DRC rule). No "we'll wire it later" — this is the single most important lesson carried forward from v1's postmortem.
6. (NEW) Ship with its spec update — an Accepted RFC is not done until note 10 (Language Specification) reflects it: new syntax gets a reference entry, changed semantics get the existing entry updated in place. The spec book is the single compiled statement of "what the language currently is" — RFCs and decision records are the history/rationale (the git log); the spec book is HEAD. An RFC whose Decision is "Accepted" but whose construct never appears in note 10 is exactly as incomplete as one that shipped without its compiler check — don't let the discipline that fixed v1's dormant-DRC problem quietly not apply to documentation.

# RFC backlog (v2 — reset from zero)

The v1 backlog (RFC-001…010) described defects and features in a discarded implementation; none of it carries over as-is. The new backlog is the actual redesign's open design questions — each of these needs a full RFC pass before implementation starts, because each is a Layer-1 decision that everything else depends on.

| RFC | Title | Priority | Why |
|---|---|---|---|
| RFC-001 | Units-as-types: the primitive unit type system (`Farads`, `Voltage`, `Ohms`, `Hertz`, …) and coercion rules — **Accepted 2026-07-13**, see child note RFC-001 + DR-007 | **P0 — done** | The foundational strictness mechanism; every spec, every device, every trait bound depends on this existing first |
| RFC-002 | Pin connection-obligation typing (`required` / `optional` / `nc`) and its exhaustiveness check — **Accepted 2026-07-13**, see child note RFC-002 + DR-012 | **P0 — done** | The direct type-level fix for v1's "silently unconnected pin" failure class |
| RFC-003 | Trait-satisfaction-at-`impl`-time checking rules — **Accepted 2026-07-13**, see child note RFC-003 + DR-013 | **P0 — done** | The direct type-level fix for v1's dormant E004/E005; must be designed before any std-library trait is written |
| RFC-004 | Classification pass: for each v1-era DRC concern (E001–E005, W001–W004 equivalents), decide "becomes a type mechanism" vs. "stays narrowed residual DRC" — **Accepted 2026-07-13**, see child note RFC-004 + DR-014 | **P0 — done** | Mandated by the Coherence Matrix's mitigation note — narrowing DRC without this is how a safety net silently disappears |
| RFC-005 | Designator allocator design: a provably collision-free, total/injective function over hierarchical path — **Accepted 2026-07-13**, see child note RFC-005 + DR-008 | **P0 — done** | Replaces v1's incrementing-counter allocator that collided; must be proven correct by construction, not tested after the fact |
| RFC-006 | Nested `fn` call semantics (monomorphization + inlining across call depth) — **Accepted 2026-07-13**, see child note RFC-006 + DR-015 | **P0 — done** | Must be first-class from v2 launch, not a P1 patch — composability depends on it |
| RFC-007 | Generics-over-specs syntax + visible-default rules — **Accepted 2026-07-13**, see child note RFC-007 + DR-016 | **P0 — done** | The expressiveness half of the core bet; must land alongside RFC-001–003, not after |
| RFC-008 | Grammar design for exhaustive pattern-matching over structural variants | P1 | Needed once devices/packages have enough variants to require it; can follow the P0 core |
| RFC-009 | `cohdl fmt` canonical form, co-designed with the grammar | P1 | Ship-from-launch requirement per the Constitution; design the form as the grammar is finalized, not after |
| RFC-010 | `cohdl check --json` schema, co-designed with the type checker's diagnostic shape | P1 | Same co-design discipline as RFC-009 |
| RFC-011 | Error-code registry (v2 baseline, informed by RFC-004's classification pass) | P1 | Depends on RFC-004 landing first — can't finalize codes until we know what moved from DRC to type-checking |
| RFC-012 | `#[intent(...)]` annotations (pure metadata) | P2 | Gated on zero-netlist-impact, same as v1; not urgent for the redesign's core bet |
| RFC-013 | Layout-constraint concept (the door) | **Gated** | Unchanged from v1 — needs a goal-change proposal, not just an RFC |

Note the shift in what's P0: in v1, P0 was "fix what's broken in a working compiler." In v2, P0 is "settle the type-system mechanisms that make the whole redesign's thesis true" — there's no partial credit for shipping Layer 3/4 work before these seven land, because nothing downstream can be honest until they do.

Milestone (2026-07-13): all seven P0 RFCs are Accepted. The Layer-1 type-system foundation — units-as-types, pin obligations, trait satisfaction, the DRC/type-system reclassification, a collision-free designator allocator, nested fn semantics, and generics-over-specs — is now formally specified end to end. RFC-004's flagged E004 dependency is closed by RFC-007. The redesign's central thesis (strictness buys expressiveness) has a complete Layer-1 specification to stand on. Remaining backlog (RFC-008–012) is P1/P2 tooling and polish; RFC-013 (layout) stays gated per its own governance. This is the natural point to revisit 9. MVP Definition, which was intentionally left void until this moment.

# The principle behind the process — unchanged

The RFC exists so that CoHDL's growth is explicit, traceable, and system-aware. A feature that can't fill the gradeability, coherence, and (now) type-system-first sections honestly is a feature CoHDL shouldn't ship.
