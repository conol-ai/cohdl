# RFC-013: Layout-constraint concept (the door)

## Problem

CoHDL's schematic source today has no way to *state* a layout-relevant fact that a partner layout tool would need — which nets belong to the same impedance-controlled class, which two nets form a differential pair, which nets must be length-matched, or a placement hint for an instance. This information exists today only outside the language entirely (a human's head, a separate spec sheet, a partner tool's own manual configuration) — meaning it doesn't survive the schematic → layout handoff as part of the design's own source of truth, and can't be regenerated/repaired by an AI author the way every other electrical fact in a `.cohdl` design can.

Who this is for: **tool builders integrating a partner layout backend** (the newly-unlocked user tier per GC-002's amendment) — they need a structured, versioned surface in the netlist/IR to consume. Secondarily, **AI authors and human reviewers** who want layout-relevant intent (e.g. "these two nets are a USB differential pair") to be stated once, in the schematic itself, rather than communicated out of band.

## Goals

- Give an author a way to declare a **closed set of layout-constraint kinds** (net class, differential-pair pairing, length-matching group, placement hint) attached to nets/instances, following the seam note 2 already designed: "a `net_class`/`constraint` decoration adjacent to Net/Rule."
- Guarantee, structurally, that a layout constraint **never affects CoHDL's own verdict, connectivity, or netlist bytes** — it rides alongside the netlist as inspectable, gradeable-against-its-own-closed-vocabulary metadata, never a second connectivity mechanism.
- Make layout constraints **exhaustively type-checked against their own closed vocabulary** (e.g. a `diff_pair` constraint must name exactly two nets, a `net_class` name must be declared before use) — consistent with the redesign's "gradeable at the earliest possible stage" discipline, applied to this new concept from day one rather than left informal.

## Non-goals

- **Not a router, not a placement engine, not a DRC-for-layout system.** CoHDL still never reasons about physical geometry, trace width vs. current capacity, or keep-out zones — it only lets a constraint be *stated*, per the Constitution's unchanged "not a layout/place-and-route engine" non-goal. GC-002's amendment explicitly does not touch this boundary.
- **Not partner-tool-specific.** This RFC defines CoHDL's own constraint vocabulary and its netlist-projection shape; it does not define how any specific partner tool (an autorouter, a layout-hint consumer) interprets that data — that's the partner's own integration concern.
- **Not validated against a real partner integration yet** — per GC-002's honest disclosure, this RFC proceeds without a concrete partner requirement in hand. The constraint-kind list below is a first, reasoned cut grounded in the most common real PCB layout needs (impedance classes, differential pairs, length matching, placement) — not a partner-confirmed final list. See Failure modes / Compatibility for how this design debt is bounded.

## Design

### A new top-level construct: `layout { ... }`, syntactically parallel to `rule`

```cohdl
design SensorNode {
    inst usb: USB_C_Receptacle_2_0
    inst esp: ESP32_S3_WROOM_1_N8

    net USB_DP: usb.DP, esp.IO20
    net USB_DM: usb.DN, esp.IO19

    layout {
        net_class HighSpeed { USB_DP, USB_DM }
        diff_pair(USB_DP, USB_DM)
        length_match(USB_DP, USB_DM) [tolerance: 0.15mm]
    }

    #[placement_hint("near USB connector, short trace to ESP32-S3 USB pins")]
    inst esp
}
```

- `layout { ... }` is a new top-level block inside a `design` (or a `fn` body, mirroring where `net`/`nc` are already allowed) — a flat list of layout-constraint statements, syntactically parallel to how `rule` blocks would look if CoHDL had in-source `rule` syntax (it doesn't yet — residual DRC is engine-builtin per RFC-004 — but `layout {}`'s shape deliberately previews that same "declarative statement block" form for consistency, per the seam's own "inspectable and gradeable like a rule" description).
- Four closed constraint kinds for this RFC's scope:`net_class NAME { net, net, ... }` — declares a named group of nets sharing a layout treatment (e.g. impedance control). `NAME` must be declared before use in any other constraint that references it.`diff_pair(net_p, net_n)` — declares exactly two nets as a differential pair. Both nets must already exist (declared via `net` elsewhere in the same design/fn).`length_match(net, net, ...)` — declares two or more nets that must be length-matched, with an optional `[tolerance: <Time-or-length-unit>]` bracket (see Type-system-first test on the tolerance unit question).`#[placement_hint("...")]` — a single opaque string attribute on an `inst`, following exactly the same shape and zero-impact discipline as RFC-012's `#[intent(...)]` (this is deliberately the least strict of the four — placement is inherently fuzzy/advisory, unlike the other three which have real structural shape to check).
- **Not attachable inside a **`fn`** body if it references pins/nets that only exist after expansion** — layout constraints, like the pin-obligation exhaustiveness check (RFC-002), are only meaningful once the design is fully assembled; a `layout {}` block inside an unexpanded `fn` may reference the `fn`'s own local nets, resolved the same way RFC-006's call-chain naming already resolves nested-fn net identity.

### Zero-impact guarantee, enforced the same way as `#[intent(...)]`

Layout-constraint data is threaded into a **separate output artifact** — a `layout.json` (or an addendum section in the netlist emitter's existing output), never merged into the `.net`/BOM connectivity data and never read by the type checker, residual DRC, or designator allocator. This is the same "not a parameter any of those passes' functions accept" structural guarantee RFC-012 established, extended from an opaque string to a small closed vocabulary that gets its own (equally non-connectivity-affecting) type-checking pass.

## Type-system-first test

This RFC **is** partially a type-system mechanism (the four constraint kinds are checked exhaustively against their own closed vocabulary — unknown net references, duplicate `net_class` names, wrong arity on `diff_pair`, are all compile errors) — but it is deliberately **not** residual DRC: nothing here is checked against the whole connectivity graph's emergent electrical properties (voltage, driver count). It's structural validation of a new declarative-metadata concept, the same category of check RFC-002's pin-obligation exhaustiveness already established, just for a new kind of declaration. A genuinely emergent layout check (e.g. "are these two length-matched nets actually within tolerance after real routing") is explicitly out of scope — that's the partner router's job to report back, not CoHDL's to verify (CoHDL has no geometry to check it against).

## Conceptual impact

**Medium-High — this is the first genuinely new core concept since the ground-up redesign began.** Layout Constraint joins the canonical vocabulary (Trait/Device/Part/Instance/Pin/Net/Spec/Rule/Module/Fn/Design/Designator) as an explicit addition, per GC-002's amendment. It is deliberately positioned "adjacent to Net/Rule" (per note 2's pre-designed seam) rather than folded into either — it is not a new connectivity mechanism (guarding against the "two ways to connect" smell) and not a DRC rule (guarding against DR-006's "reaching for rule" smell), but its own small, closed thing.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Med | Med | Low | Med | Med | Low | Med |

Concepts (Med, addressed above): a genuinely new concept, but scoped as narrowly as the seam allows — four closed kinds, not an open framework.

Netlist (Med): the first RFC to add a new artifact (the layout-constraint output) alongside the existing .net/BOM — a real, if additive, netlist-surface change.

Diagnostics (Med): four new structural checks need their own error-code sub-block (see Tooling & operations).

Oracle (Low): does not change what "compiles clean" means for the existing schematic-correctness pipeline — a design with zero layout {} blocks behaves identically to today.

Compat (Low): purely additive.

Trust (Med): the constraint-kind list being partner-unvalidated (see Non-goals/Compatibility) is a real, named trust risk — mitigated by the same zero-impact-guarantee discipline that makes worst case "this metadata was wrong or unused," never "this metadata corrupted the schematic's own correctness."

## Gradeability

Enforced by direct type-checking of the four constraint kinds against their own closed vocabulary (unknown net reference, duplicate `net_class` name, wrong `diff_pair` arity, `length_match` referencing fewer than two nets) — all compile errors, all with precise spans, same diagnostic-quality bar as every other Accepted RFC. Additionally, a **zero-impact regression test** analogous to RFC-012's: for every fixture, adding/removing/mutating `layout {}` content must never change the schematic-correctness verdict, any RFC-001–011 diagnostic, designator assignment, or `.net`/BOM bytes — only the separate layout-constraint artifact differs.

## AI-generatability

Medium-High. The four constraint kinds are small and regular (net-class grouping, pair, length-match-group, placement-hint-string) — a model already comfortable with `net`/`nc`/`impl` syntax learns one more small declarative block. The one real generatability risk: since no partner integration exists yet to validate against, a model has no real-world feedback loop telling it whether its stated constraints were *useful* to the eventual layout tool — only whether they're *well-formed* per this RFC's closed vocabulary. This is an inherent, named limitation of opening the door pre-partner-requirement (see GC-002).

## Alternatives

- **Fold layout constraints into **`rule`**, or wait for real in-language **`rule`** syntax to exist before adding this** — rejected: `rule` (residual DRC) is reserved for genuinely emergent connectivity-graph checks (RFC-004's classification), and layout constraints are neither emergent nor about connectivity correctness — they're declarative metadata about an orthogonal (physical) domain. Forcing them into `rule`'s shape would violate DR-006's core classification discipline in the other direction (using `rule` for something that isn't actually a `rule`-shaped check).
- **Attach layout constraints as more **`#[intent(...)]`**-style opaque strings only, no structured kinds at all** — rejected: unlike design rationale (where opaque prose is exactly the right level of structure, per RFC-012), a differential pair or a length-match group has real, checkable structure (arity, net existence) that's worth type-checking — reducing it to prose would throw away gradeability the redesign's whole thesis says is worth having wherever structure exists.
- **A fully general, extensible layout-constraint plugin system** (arbitrary constraint kinds defined by a partner tool's own schema) — rejected as premature and disproportionate: with no concrete partner requirement yet (per GC-002), designing an extensibility mechanism now would be speculative generality with no real use case to validate it against — exactly the "conceptual cost with no corresponding job" the Coherence Matrix exists to catch. A closed set of four kinds, extendable by a future RFC once real partner needs are known, is the more honest scope.

## Compatibility

Purely additive — no existing source, diagnostic, designator, or netlist/BOM byte is affected for any design without a `layout {}` block or `#[placement_hint(...)]`. The real compatibility risk is **forward**, not backward: per GC-002's disclosed design debt, the four constraint kinds may need revision once a real partner integration is scoped — this RFC's Decision explicitly flags that the constraint vocabulary is provisional and may be extended or reshaped by a follow-up RFC without that being a violation of this RFC's own stability (this is analogous to how RFC-001's ten-type unit set was explicitly closed-but-extensible-via-new-RFC).

## Tooling & operations

- Reserves a new error-code block, **E10xx** (following the established "block per mechanism, kind-of-mistake organizing principle" from RFC-011): E1001 unknown net in a layout constraint, E1002 duplicate `net_class` name, E1003 wrong `diff_pair` arity, E1004 `length_match` with fewer than two nets, E1005 `net_class` referenced before declaration.
- `cohdl build` gains a new output artifact (`layout.json` or a netlist-emitter addendum) alongside the existing `.net`/BOM — must be documented as a new, versioned output format the same way the netlist format itself is versioned (per the Constitution's compatibility promises).
- `cohdl check --json` (RFC-010) should surface layout-constraint diagnostics through the same `diagnostics` array (they're ordinary compile errors with codes/spans/messages) — no schema change needed, just new `code` values in the E10xx range.
- `cohdl fmt` (RFC-009) treats `layout { ... }` with the same block-formatting rules as `pins {}`/`spec {}`/`variants {}` — one statement per line, 4-space indent; `#[placement_hint(...)]` follows the existing single-line-attribute convention.

## Teaching cost

Medium. Four new constraint kinds to learn, but each is small, and the "this never affects your schematic's own correctness" guarantee (identical framing to `#[intent(...)]`) means an author can learn to use them incrementally without fear of breaking anything already working.

## Failure modes

- **A constraint kind is used but the eventual partner tool doesn't recognize/consume it** — since no concrete partner integration exists yet (GC-002's disclosed risk), this is a real possibility for all four kinds until a real integration validates them; mitigated only by the zero-impact guarantee (worst case: unused metadata, never corrupted schematic correctness).
- **An author expects a **`length_match`** tolerance to be enforced by CoHDL itself** — it is not; CoHDL has no geometry to check it against. The tolerance value is purely data passed through to whatever partner tool consumes `layout.json`. This must be stated plainly in any user-facing documentation, the same caveat pattern used for `#[intent(...)]`'s "this is not enforced" framing.
- **Scope creep**: a future request to make CoHDL itself validate a layout constraint against something (e.g. "warn if two length-matched nets differ by more than a computed estimate") would be a new, separately-justified RFC — not an incremental extension of this one, per the Operational risks section of GC-002's amendment.

## Migration path

N/A — purely additive, no existing source affected.

## Decision

Accepted — 2026-07-13, following GC-002's amendment (note 8) opening the layout door. Recorded as DR-011 (see note 7 — the next genuinely open decision-record number; distinct from RFC-011, the already-Accepted error-code registry RFC, which used its own number in the RFC track, not the DR track — same non-collision pattern already noted for DR-013/RFC-013 earlier in this backlog). Language Specification (note 10) gains a "Layout constraints (the door)" section. This RFC's constraint vocabulary is explicitly provisional — per GC-002's disclosed design debt, it should be revisited the moment a real partner layout-tool integration is scoped, and that revisitation is expected, not a failure of this RFC.
