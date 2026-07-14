# RFC-017: Library registry (cohdl source + docs + footprint symbols)

## Status

Revised 2026-07-14 (same day as original acceptance), per Tony's direct follow-up correction. The original draft made footprint: a bare package-relative path string to a .cfp file, with the .cfp format's grammar defined inline in this RFC. Tony corrected two things at once: (1) footprints must resolve through RFC-016's module/symbol-resolution infrastructure — like every other named thing in CoHDL — so one library can reuse another library's footprint by name, not just by copying a path; (2) the footprint format itself (what's inside a footprint declaration) is out of scope for this RFC and belongs in a later, dedicated RFC. This revision reflects both corrections. The original inline .cfp grammar sketch is removed from this RFC entirely — it was scope this RFC was never supposed to own.

## Problem

Getting started with CoHDL today means writing every trait/device/part from scratch — even the MVP's own std library exists only because "the demo board needs it" (note 9's own stated discipline), not as a general-purpose library. There is no way to publish, discover, or depend on a manufacturer's or community's pre-built set of devices, and — per RFC-015's own named future work — no way to resolve a part's footprint reference to actual geometry, because CoHDL owns no footprint format and no open, portable one exists industry-wide (confirmed by research: IPC-7351 is a naming/calculation methodology, not a file format; every CAD tool's footprint format is its own proprietary/de-facto standard).

Tony's directive: build a centralized library registry, containing four candidate content kinds — .cohdl source (traits/devices/parts/fns), reference documents (datasheets/manuals/app notes), manufacturer "best practice" skills, and footprints — to dramatically lower the cost of getting started. This RFC scopes three of those four kinds (skills explicitly deferred, per direct decision) — but for footprints specifically, this RFC scopes only making footprint a named, resolvable symbol in the module system, not the geometry format inside it (deferred, per Tony's direct correction, to a later RFC).

Who this is for: library authors (manufacturers, community members publishing reusable parts, footprints, and documents) and library consumers (anyone starting a new board who wants use adafruit::motors::TB6612 — and its footprint — to just work) — directly serving the Constitution's stated AI-native mission at the ecosystem layer, not just the language layer.

## Goals

- Define a **Library** as a distributable package (RFC-016's package concept) that may additionally carry two new content kinds alongside its `.cohdl` source: **reference documents** and **footprint symbols — both attached to / expressed as declarations, not floating loose paths**.
- Introduce footprint as a new top-level declaration kind, resolved through the exact same module-path/use/pub infrastructure RFC-016 already built for device/trait/fn/part — so a footprint authored in one library is a first-class, importable, cross-library-reusable symbol (use sparkfun::footprints::QFN10_3x3;), not a private file only its own library's parts can see.
- Change part's footprint: field to hold a symbol reference (a path resolved by RFC-016's rules) instead of today's KiCad-library-reference string — making footprint binding subject to the same visibility/resolution rules as every other cross-library reference.
- Make a library's content discoverable and resolvable through RFC-016's module-path mechanism — sparkfun::power::buck::TPS62840's datasheet is reached through the same path as the device declaration itself; its footprint is reached the same way a device reaches a trait impl, not a separate lookup system.

## Non-goals

- Not the footprint format. This RFC does not define what is inside a footprint { ... } declaration — no pad/shape/layer grammar, no geometry model, no parser. That is explicitly deferred to a future RFC (working title: "Footprint format," unnumbered until proposed). This RFC only establishes that footprint is a named declaration kind that exists, is resolvable, and is what part binds to. Treat any geometry example in this RFC's Design section as illustrating the declaration shape only (name, visibility, module placement) — never as a proposal for the format RFC to inherit.
- **Not "skills" (manufacturer best-practice guidance)** — explicitly deferred per direct decision. This RFC's registry ships with exactly three content kinds: source, documents, footprint symbols.
- Not the actual centralized hosting infrastructure (a package index server, a cohdl publish CLI, versioning/yanking policy, a web UI for browsing) — this RFC defines what a library is and how its content resolves once available on disk (mirroring RFC-016's own scope boundary). Hosting/distribution mechanics are future work, likely their own RFC once real libraries exist to distribute.
- Not footprint geometry auto-generation, 3D models, silkscreen art, or courtyard/assembly-layer detail — none of this can be scoped until the format RFC exists.

## Design

### A Library is just a Package (RFC-016) with two new optional content kinds

No new top-level concept beyond what RFC-016 already defines — `cohdl.toml`'s existing `[package]` block is the library's identity.

### Reference documents: `#[doc(...)]`, following `#[intent(...)]`'s exact established shape

```cohdl
#[doc("datasheets/TPS62840.pdf")]
#[doc("app-notes/buck-converter-layout-guidelines.pdf")]
pub device TPS62840<...> {
    ...
}
```

- One or more `#[doc("relative/path")]` attributes per declaration (unlike `#[intent(...)]`'s at-most-one rule — a device legitimately has multiple reference documents: datasheet, app note, errata).
- Paths are relative to the library's package root, resolved at build/tooling time — the compiler itself never opens or parses the referenced file. Zero-compilation-impact, same discipline #[intent(...)] (RFC-012) and #[placement_hint(...)] (RFC-013) already established.

### footprint: a new top-level declaration kind, resolved like everything else

```cohdl
// sparkfun/src/footprints/qfn.cohdl → module path sparkfun::footprints::qfn

pub footprint QFN10_3x3 {
    // body format: OUT OF SCOPE for this RFC — deferred to the format RFC.
    // Whatever that RFC lands on, a `footprint` declaration is a named,
    // pub-able, module-path-resolvable symbol exactly like `device`/`trait`/`fn`/`part`.
}
```

```cohdl
// in a different library, or the same one:
use sparkfun::footprints::qfn::QFN10_3x3;

pub part TPS62840_QFN10: TPS62840<...> {
    primary {
        mfr: "Texas Instruments",
        mpn: "TPS62840DLCT",
        footprint: QFN10_3x3   // symbol reference, resolved via RFC-016 — not a path string
    }
}
```

- footprint joins device/trait/fn/part as a top-level declaration kind. It follows RFC-016's rules exactly: its module path is derived from its file location, pub controls cross-package visibility, use imports it by name.
- This is the concrete fix for Tony's correction: because a footprint is a real symbol in the module system (not a path string embedded in a part's footprint: field), any library can use any other library's footprint declaration and reference it in its own parts — footprint reuse across libraries works exactly the way device/trait/fn reuse already does, with the same visibility enforcement (a non-pub footprint can't be reached from outside its package) and the same collision rules (two libraries' QFN10_3x3 footprints never collide, because they live at different module paths).
- part's footprint: field changes meaning again relative to the original (now-superseded) draft of this RFC: it now holds a symbol reference — a bare name (if in scope via use or local declaration) or a fully-qualified path (sparkfun::footprints::qfn::QFN10_3x3) — resolved by the exact same name-resolution the rest of the language already uses for device/part references. It is never a file path and never a string literal.
- The footprint { ... } declaration's body is left completely unspecified by this RFC. Until the format RFC lands, a footprint declaration exists as a resolvable name with no defined internal content — real footprint authoring (and the pad/pin-count consistency check between a footprint and its bound device) cannot happen until that RFC is Accepted. This RFC is deliberately "symbol-resolution-complete, format-empty" — the same kind of honestly-partial phasing RFC-015 already established for IPC-2581 ("logical-complete, physical-minimal").

### Resolution: footprints and docs travel with the module path

Because footprint is a first-class declaration (not a path string), resolving a part's footprint: reference is identical in shape to resolving any other cross-library name under RFC-016: unqualified if used or declared locally, qualified otherwise, pub-gated across package boundaries. #[doc(...)] paths remain plain relative paths (not symbols — see Alternatives for why these two are treated differently) resolved against the declaration's own package root.

## Type-system-first test

The check this RFC anticipates (pad/pin-count consistency between a footprint symbol and the device its bound part implements) is structural and local to one part+device+footprint triple — exactly the "checkable from the declaration alone" shape RFC-002/003's pin/trait checks already established, not a rule/DRC candidate. However, this RFC cannot specify that check concretely, because it depends on the footprint format RFC defining what a footprint's pad set actually looks like. This RFC's job is narrower: guarantee the symbol that check will eventually run against already resolves correctly and unambiguously. The format RFC inherits this section's classification (type system, not DRC) as a settled precedent, not an open question it needs to re-litigate.

## Conceptual impact

Low. No new top-level concept beyond RFC-016's Package/Module (already Accepted). One new declaration kind (footprint) — conceptually a peer of device/trait/fn/part, not a new category of concept, since it reuses 100% of RFC-016's resolution machinery. One new attachment mechanism (#[doc(...)]), a repeat of an already-established pattern (#[intent(...)], #[placement_hint(...)]). No new sub-language — the .cfp-format sketch from the original draft is removed; there is nothing to parse yet.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Med | Low | Med | Med | High | High |

Compat (Med, down from the original draft's High): footprint:'s meaning changes from a KiCad-library-reference string to a symbol reference — still a real, disclosed breaking change to every existing part declaration, but the migration is now smaller in this RFC's scope: existing parts need their footprint: field converted to reference some footprint symbol, but authoring that symbol's actual content is explicitly deferred (not blocked on this RFC landing).

Trust (Med, down from High): this RFC makes footprint references unambiguous and cross-library-safe, but does not yet make "the footprint lies" structurally impossible — that guarantee is deferred to the format RFC, which is where the actual pad/pin-count check will be specified and enforced.

Grammar (Low, down from Med): adding one new top-level declaration keyword (footprint) with an empty, unspecified body is a small grammar addition — no new sub-language to parse in this RFC.

Diagnostics/Netlist (Low, down from Med): no new completeness check and no new emission target land in this RFC — both are deferred with the format.

## Gradeability

This RFC's own checkable surface is narrow: footprint symbol resolution follows RFC-016's existing resolution rules exactly (unresolved name, ambiguous import, or visibility violation are compile errors already specified by RFC-016 — no new diagnostic mechanism needed here). The pad/pin-count consistency check between a footprint and its device is explicitly not specified or enforced by this RFC — it is future work for the format RFC, named here so it isn't silently assumed solved (see Failure modes).

## AI-generatability

High for the part this RFC actually delivers: referencing another library's footprint (use sparkfun::footprints::qfn::QFN10_3x3; then footprint: QFN10_3x3) is exactly as easy to generate as referencing any other cross-library symbol under RFC-016 — no special-cased syntax to memorize. Authoring a new footprint's actual content remains ungraded by this RFC (deferred to the format RFC, which will need to address it directly, likely landing at Medium per the original draft's reasoning: real geometric knowledge is required, this RFC doesn't change that).

## Alternatives

- Keep footprint: as a bare package-relative path string to a .cfp file (the original draft of this RFC) — rejected per Tony's direct correction: a path string cannot be used, cannot be checked for cross-package visibility, and gives two libraries defining "the same" footprint no way to converge on reusing one declaration instead of independently duplicating it. Symbol resolution is exactly the problem RFC-016 already solved for every other named thing in the language; inventing a second, weaker mechanism (bare paths) for footprints specifically would be the "parallel mechanism instead of extending an existing concept" smell note 4 already warns against.
- Define the footprint format inline in this RFC (the original draft's .cfp grammar sketch) — rejected per Tony's direct correction: bundling "make footprints resolvable" with "define what's inside a footprint" conflates two independently-decidable questions. The resolution question is answerable now, cheaply, by reusing RFC-016 wholesale. The format question is a real, separate design problem (pad geometry, shape vocabulary, layer model) that deserves its own focused RFC pass rather than being rushed as a sub-section of the registry RFC.
- Treat #[doc(...)] paths the same way (as symbols, not paths) — considered, rejected: a reference document is an inert external artifact (a PDF) with no internal CoHDL-checkable structure, so there is nothing to gain from symbol resolution's collision/visibility machinery — a plain relative path is the right-sized mechanism, and treating every string-shaped reference as a symbol just because footprints needed to be one would be over-generalizing a fix that was actually specific to footprints' reuse requirement.
- Adopt KiCad's .kicad_mod as the library format instead of a native format — the alternative Tony explicitly considered and decided against for the (deferred) format RFC to pick up later, in favor of more control. Recorded here for continuity; this RFC does not re-open or resolve it — that decision belongs to the format RFC.
- Skip footprints entirely for this RFC, ship source + docs only — rejected: footprints were explicitly named by Tony as one of the four registry content kinds up front, and making them resolvable (this RFC's actual, now-narrower scope) is real, shippable progress even while the format itself waits for its own RFC — better than deferring the entire concept a second time.

## Compatibility

One real, disclosed breaking change, narrower than the original draft's:

footprint:'s meaning changes from a KiCad-library-reference string to a symbol reference. Every existing part declaration in the std library and example boards needs its footprint: field converted to reference a footprint symbol. Because the footprint format itself doesn't exist yet, this migration can, for now, only go as far as declaring empty/placeholder footprint symbols for each existing part — full migration (real pad content) waits for the format RFC. This is explicitly a two-stage migration, disclosed as such, not glossed over as one step.

Depends on RFC-016 (module system) landing first — unchanged from the original draft; this RFC's entire footprint-resolution mechanism is RFC-016 reused, not extended.

## Tooling & operations

- cohdl lsp (RFC-014) hover on a part declaration should surface its #[doc(...)] paths (e.g. "view datasheet") and its resolved footprint: symbol's declaration location (via the same goto-def/hover machinery RFC-014 already provides for device/trait/part references) — no new LSP capability, just RFC-014's existing capabilities applied to one more declaration kind.
- cohdl fmt (RFC-009) needs a formatting rule for the new footprint top-level keyword (parallel to device/trait/fn/part) and for #[doc(...)] (same single-line-attribute convention as #[intent(...)]/#[designator(...)]). No .cfp-specific formatting rules — there is no format yet.
- No new error codes reserved by this RFC beyond what RFC-016's existing resolution diagnostics (unresolved name, ambiguous import, visibility violation) already cover for the new footprint declaration kind. The format RFC will reserve its own error-code block when it lands (likely still E8xx, per RFC-011's "kind of mistake" organizing principle — a footprint/pin mismatch is a part-completeness question).

## Teaching cost

Low. footprint is a fifth top-level declaration kind, but it follows every rule device/trait/fn/part already follow under RFC-016 — nothing new to learn about visibility, resolution, or use. The genuinely new teaching cost (what goes inside a footprint) is entirely deferred to the format RFC, where it belongs.

## Failure modes

- A footprint symbol resolves but its (currently undefined) content doesn't actually match its bound device's pins — this RFC cannot catch this, by design; it is explicitly named, tracked future work for the format RFC, not silently assumed solved. This is the direct successor to the original draft's stronger Trust claim, now honestly downgraded (see Coherence matrix row).
- Two libraries both publish a footprint intending to represent "the same" real-world package (e.g. two different QFN10_3x3 declarations from two different authors) — this RFC does not deduplicate or canonicalize; each is a distinct symbol at a distinct module path, and a consumer must choose which to depend on. This mirrors how two libraries can already publish two different devices claiming to model the same real chip — an existing, accepted property of the module system, not a new gap this RFC introduces.
- A library ships #[doc(...)] paths pointing at files that don't exist in the package — the compiler never opens these files, so this is not currently caught; a future lint/tooling pass could check path existence, named here as a real, honest gap.

## Migration path

Existing part declarations' footprint: fields must be converted from KiCad-library-reference strings to footprint symbol references. Because the format RFC hasn't landed, this migration's first stage is mechanical-but-incomplete: declare an empty placeholder footprint symbol per existing part, point footprint: at it. The second stage (giving each placeholder real content) is blocked on the format RFC and is not part of this RFC's "ship with its check" completion bar.

## Decision

Accepted (revised) — 2026-07-14. Supersedes the original same-day acceptance's footprint scope (source + documents unchanged; footprint scope narrowed to symbol-resolution-only, per Tony's direct correction). Recorded as an amendment to DR-023 (see note 7). Language Specification (note 10) is updated: the "Library registry" section's footprint sub-section is rewritten to describe footprint as a resolvable declaration kind with an unspecified body, and the .cfp-format content is removed pending the future format RFC. Explicitly depends on RFC-016 (module system) landing first. Skills remain deferred, unchanged from the original decision. The footprint format is now also explicitly deferred, to a future, separately-numbered RFC — this registry, as accepted, ships with exactly three content kinds (source, documents, footprint symbols), with the footprint symbols' internal content as an acknowledged, tracked gap rather than something this RFC pretends to close.
