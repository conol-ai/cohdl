# RFC-027: Quilter physics-constraint hints and CSV export

## Status

Redesigned same day, per Tony's direct correction. The first draft added seven brand-new bare statement keywords inside layout {} (ground_net, high_current_net, single_ended_impedance_signal, bypass_capacitor, crystal_oscillator, switching_converter, bga_component) — real, permanent grammar growth, seven new reserved words the lexer must recognize forever. Tony's correction: use attribute-style syntax instead, and do not add this many keywords. This revision attaches every constraint as an #[...] attribute directly on the net/inst declaration the fact is actually about — reusing the bracket syntax already established by #[intent(...)] (RFC-012), #[placement_hint(...)] (RFC-013), and #[designator(...)] (RFC-005) — rather than inventing new statement-introducer keywords. Replaces this RFC's own first draft entirely.

## Problem

Grounded directly in eight real CSV files Tony supplied, matching a real, documented Quilter mechanism: "Physics Constraints" — high-level electrical/proximity facts a layout tool uses to generate and validate layout, beyond bare connectivity. Confirmed against Quilter's own docs (`docs.quilter.ai/physics-constraints/*`): seven of the eight are directly documented Quilter constraint kinds (power/high-current nets, single-ended impedance, differential pairs, bypass capacitors, crystal oscillators, switching converters), each with a real, small, fixed field schema; the eighth (`bga_components`) has a real CSV schema (`component, generate_fanout`) but no public Quilter doc describing it further, so it is scoped conservatively (see Design).

Confirmed against real source (src/ast.rs): LayoutBlock.constraints: Vec<LayoutConstraint> (RFC-013) is the existing home for net-pair/multi-net facts (net_class, diff_pair, length_match) that don't attach naturally to one single declaration. But six of the seven remaining CSVs are each fundamentally a fact about one specific net or inst (this net is high-current; this capacitor bypasses that pin) — not a relationship needing its own free-standing statement. Confirmed against real source (src/ast.rs): the generic Attr struct (name: Ident, args: Vec<(String, Span)>) already carries exactly one opaque string per attribute today (the shape RFC-012's #[intent("...")] established) — real prior art for exactly this "looks like #[name(...)]" syntax, though its arguments today are opaque strings, not the structured, unit-typed/reference-typed arguments these new attributes need (see Design for how this RFC resolves that).

One of the eight CSVs (differential_pairs) is not a wholly new fact — it's RFC-013's existing diff_pair(net_p, net_n) layout{} statement, which stays a layout{} statement (it is inherently about a pair of nets, not one single declaration) but gains the three additional numeric fields (differential_impedance, single_ended_impedance, frequency) Quilter's real form carries and RFC-013's version doesn't.

## Goals

- Let an author state each of the seven remaining real Quilter physics-constraint facts, attached directly to the net/inst declaration each fact is actually about — no new bare statement keywords, no separate cross-reference by name from inside layout {}.
- Reuse the existing #[name(...)] attribute bracket syntax (RFC-005/012/013's precedent) for every new fact — zero new reserved words added to the lexer.
- Extend `diff_pair` (already Accepted, RFC-013) with the three numeric fields Quilter's real form requires, rather than adding a second, competing construct.
- Emit each constraint kind as its own CSV file at `cohdl build`, with headers and column order matching the real files Tony supplied exactly.

## Non-goals

- Not automatic constraint inference. Quilter's own docs describe most of these as auto-detected from naming/topology. CoHDL does not replicate this inference — every constraint in this RFC is an explicit author-written attribute. An author who wants Quilter's own auto-detection to run can simply omit the corresponding CoHDL attribute — Quilter's detection still operates on the plain netlist CoHDL already emits, independent of this RFC.
- Not a generic opaque-string attribute reuse. Unlike #[intent(...)]/#[placement_hint(...)] (deliberately opaque, zero-parsed prose), the seven new attributes here carry real, structured, checked arguments (unit-typed values, pin/instance references) — a genuinely new attribute-argument grammar, not a repurposing of the existing opaque-string Attr shape (see Design, Alternatives).
- Not a general "component group" or "proximity constraint" abstraction. Bypass capacitors, crystal oscillators, and switching converters each have a different real arity of related instances/pins/values — collapsing them into one generic, optional-field attribute was considered and rejected (see Alternatives).
- Not board-level layer stackup or trace-width computation, and not validating physical achievability — CoHDL states the fact; Quilter computes/validates it, the same "declared fact, no physics computed by CoHDL" discipline every prior layout-adjacent RFC (013/020/025/026) already established.
- bga_components's scope is deliberately minimal — its only real, confirmed field is a boolean-shaped flag (generate_fanout); no further Quilter-side documentation of this constraint was found.

## Design

Seven new structured attribute kinds — reusing #[name(...)] bracket syntax, zero new bare keywords — plus one additive extension to the already-Accepted diff_pair layout{} statement.

```cohdl
design SensorNode {
    #[high_current(500mA)]
    net V3V3 [3.3V]: ldo.VOUT, mcu.VDD, u2.VIN

    #[high_current(500mA)]
    net VBUS [5V]: usbc.VBUS

    #[ground(primary)]
    net GND: ldo.GND, mcu.GND, u2.GND

    net USBC_DP: usbc.DP, mcu.USB_DP
    net USBC_DM: usbc.DM, mcu.USB_DM

    #[impedance(50ohm, frequency: 1GHz)]
    net HDMI_CLK: mcu.HDMI_CLK, hdmi.CLK

    #[bypass(mcu.VDD, 100nF)]
    inst c1: MLCC<100nF, 16V>

    #[crystal_oscillator(mcu, XTAL_IN, XTAL_OUT)]
    inst y1: Crystal_8MHz

    #[switching_converter(inductor: l1, input_capacitor: c_in, output_capacitor: c_out)]
    inst u2: BuckConverter
    inst l1: Inductor_2_2uH
    inst c_in: MLCC<10uF, 16V>
    inst c_out: MLCC<22uF, 16V>

    #[bga_fanout]
    inst mcu: MCU_BGA_256

    layout {
        diff_pair(USBC_DP, USBC_DM) [
            differential_impedance: 100ohm,
            single_ended_impedance: 50ohm,
            frequency: 1GHz
        ]
    }
}
```

- #[ground(PRIMARY [, region_pour])] on a net declaration — PRIMARY closed to {primary, secondary} (at most one primary ground net per design — checked). region_pour a bare optional flag, defaults absent (⇒ false). Maps directly to ground_nets.csv's three columns.
- #[high_current(CURRENT [, power_pour])] on a net declaration — CURRENT a Current-typed value (RFC-001). power_pour a bare optional flag. Maps directly to high_current_nets.csv. This is Quilter's documented "Power Nets" constraint.
- #[impedance(IMPEDANCE, frequency: FREQ)] on a net declaration — IMPEDANCE a Resistance-typed value, FREQ a Frequency-typed value (both RFC-001). Maps directly to single_ended_impedance_signals.csv.
- #[bypass(INST.PIN, CAPACITANCE)] on the bypass capacitor's own inst declaration — INST.PIN an already-declared instance + pin (RFC-002); CAPACITANCE a Capacitance-typed value. Note this attaches to the capacitor's own inst line (the CSV's own row-subject), not to INST. Maps directly to bypass_capacitors.csv's four columns (the capacitor's own designator is read off the inst it's attached to).
- #[crystal_oscillator(PARENT_INST, PIN_1, PIN_2)] on the crystal's own inst declaration — PARENT_INST an already-declared instance; PIN_1/PIN_2 two of PARENT_INST's declared pins. Maps directly to crystal_oscillators.csv.
- #[switching_converter(inductor: INST [, input_capacitor: INST] [, output_capacitor: INST])] on the converter's own inst declaration — inductor required; the two capacitor arguments each optional (Quilter's own docs mark these optional), all already-declared instances. Maps directly to switching_converters.csv.
- #[bga_fanout] on a BGA's own inst declaration — a bare attribute, no arguments; presence ⇒ generate_fanout: true. Maps directly to bga_components.csv's one real confirmed column.
- diff_pair(net_p, net_n) [differential_impedance: IMPEDANCE, single_ended_impedance: IMPEDANCE, frequency: FREQ] — stays a layout{} statement (RFC-013), since it is inherently about a pair of nets, not attachable to one single declaration the way the other six facts are. Gains an optional trailing bracket carrying the three new fields; omitting the bracket preserves RFC-013's original, unannotated form exactly.
- Grammar note: these seven attributes are structurally distinct from the existing generic, opaque-string Attr (#[intent(...)]/#[placement_hint(...)]/#[designator(...)], all exactly-one-string-literal) — each carries its own real, closed argument grammar (unit-typed literals, bare flags, pin/instance references, named optional arguments), parsed and structurally checked, not opaque prose. They share the surface #[name(...)] bracket syntax with the existing attributes (so no new bracket/lexer token is introduced) but are recognized as their own closed set of attribute names, each with its own fixed argument shape — the same "looks like an attribute, is actually its own structurally-checked field" pattern RFC-013 already established for #[placement_hint(...)] (split out from the generic attrs: Vec<Attr> at parse time).

## Type-system-first test

Not a rule/DRC proposal — every check below is structural and local, resolved once, at the point each attribute is parsed on its net/inst declaration (the same discipline RFC-013 already established for its own layout{} kinds):

1. Reference resolution — every pin/instance reference argument must resolve to an already-declared instance/pin; unresolved is a compile error naming what wasn't found.
2. Closed-set/arity checks per attribute — #[ground(...)]'s PRIMARY two-value set and the at-most-one-primary-per-design rule; #[switching_converter(...)]'s required-vs-optional argument arity; unit-type checks on every numeric argument (RFC-001's zero-coercion rule, unchanged).
3. At most one of each attribute kind per declaration — the same "at most one" discipline #[intent(...)]/#[placement_hint(...)] already enforce.
4. No new emergent/cross-cutting check — every check is local to one attribute's own declaration and its referenced names. Never a residual-DRC candidate.

## Conceptual impact

Low. Zero new keywords — the correction this revision makes. Seven new attribute names, recognized inside the existing #[...] bracket syntax, each with its own small, closed argument grammar. No new top-level declaration kind, no new statement-introducer token. diff_pair's extension reuses RFC-013's existing LayoutConstraint mechanism, additive only.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Med | Low | Med | Med | Low | High |

Grammar (Low, revised down from the withdrawn draft's Med): zero new keywords — every fact rides the existing #[...] attribute bracket; the real new surface is seven closed argument grammars, not seven new statement shapes.
Diagnostics (Med): reference-resolution and arity/closed-set checks across seven attribute kinds — real, new surface, though structurally identical in kind to checks RFC-013 already established for layout{}.
Netlist (Med): a genuinely new emitted artifact class (eight CSV files) — real, new emitter work, though each is a direct, lossless re-projection of already-validated attribute data.
Trust (High): zero schematic-correctness impact, by construction — none of these attributes are read by the type checker, residual DRC, designator allocator, or .net/BOM emitters.
Compat (Low): purely additive; every existing inst/net/diff_pair(...) statement (with no new attribute) is unaffected.

## Gradeability

- Reference resolution and per-attribute arity/closed-set checks run once, at the point each attribute is parsed — the same stage RFC-013's own layout{} checks already run at.
- Unit-typed arguments reuse RFC-001's existing zero-coercion unit system unchanged.
- None of this runs in residual DRC — see Type-system-first test above.

## AI-generatability

High. Every attribute name and argument shape maps directly to Quilter's own publicly documented constraint schema, attached exactly where a model would naturally look for it (on the net/inst the fact is about) — no separate cross-referencing-by-name step the model could get wrong, unlike the withdrawn first draft's bypass_capacitor c1: bypasses mcu.VDD, ... shape, which restated c1's name redundantly from inside layout{}.

## Alternatives

- Seven new bare layout {} statement keywords (this RFC's own withdrawn first draft) — rejected per Tony's direct correction: real, permanent grammar growth (seven new reserved words) for facts that are each naturally a property of one already-existing declaration, not a free-standing relationship needing its own statement.
- Reuse the existing generic, opaque-string Attr shape verbatim (i.e., #[bypass("mcu.VDD, 100nF")] as a single string, unparsed) — rejected: these facts have real, checkable structure (a real pin reference, a real unit-typed value) that RFC-001's/RFC-002's existing type-checking machinery can and should validate; discarding that into an opaque string would be a real regression in gradeability for no benefit, and inconsistent with this project's own "push structure into the type system wherever real structure exists" thesis (DR-005).
- A single generic "proximity group" attribute covering bypass capacitors, crystal oscillators, and switching converters at once — rejected: the three have genuinely different real arities (one target pin + one value; two target pins + no value; three related components + no pins/value) — forcing one shape to cover all three would need optional/variadic arguments whose validity depends on which "kind" was chosen, the same ambiguous, convention-dependent shape this project's discipline (RFC-008, RFC-017) has consistently rejected.
- Auto-inferring these constraints from the netlist — rejected for CoHDL specifically: silent inference is exactly the class of risk this project avoids everywhere (RFC-008, RFC-016) — an author should always see, in source, exactly which facts CoHDL asserts to a downstream tool.
- A single flat CSV with a kind column instead of eight separate files — rejected: each Quilter constraint kind has a genuinely different, fixed column schema (confirmed against the real supplied CSVs); a single polymorphic file would be strictly harder to validate and would diverge from the real file set Tony supplied.

## Compatibility

Purely additive. Every existing inst/net/diff_pair(...) statement (with no new attribute, or an unannotated diff_pair) is completely unaffected, unchanged in meaning and in every emitted byte.

Depends on: RFC-013 (layout-constraint concept, already Accepted) for diff_pair's extension. Reuses RFC-001's unit-type system unchanged for every numeric argument. Reuses RFC-002's pin-obligation-declared pin names for pin-reference arguments. Reuses RFC-005/012/013's existing #[name(...)] attribute bracket syntax — no new bracket/lexer token.

## Tooling & operations

- cohdl build gains eight new emitted CSV artifacts (bga_components.csv, bypass_capacitors.csv, crystal_oscillators.csv, differential_pairs.csv, ground_nets.csv, high_current_nets.csv, single_ended_impedance_signals.csv, switching_converters.csv) — headers/column order matching the real supplied files exactly, one row per net/inst carrying the corresponding attribute (an empty file with just the header row when a design declares none, matching the real supplied header-only files).
- cohdl build --json's build object gains one new key per CSV file (path), present only when emitted — same pattern as the existing "layout"/"ipc2581" keys.
- cohdl fmt places each new attribute as a single-line prefix directly preceding its net/inst declaration — the exact existing convention #[intent(...)]/#[placement_hint(...)]/#[designator(...)] already use, no new formatting category.
- Reserves new E10xx sub-cases (layout constraints, RFC-013/020's existing home): unresolved pin/instance reference per attribute, invalid PRIMARY value, duplicate primary ground net, missing required argument (switching_converter's inductor), unit-type mismatch on any numeric argument, duplicate attribute of the same kind on one declaration — no new block, per RFC-011's "kind of mistake" organizing principle.

## Teaching cost

Low. Each attribute sits directly on the declaration it describes — an author already familiar with #[intent(...)]/#[placement_hint(...)]'s bracket convention needs only to learn seven new, small, closed argument shapes, each a direct transliteration of Quilter's own documented field names — no new statement-kind vocabulary, no cross-referencing-by-name step to learn.

## Failure modes

- An author references a pin/instance that doesn't exist — caught immediately, naming what wasn't found.
- An author declares two #[ground(primary)] nets — a compile error naming both.
- An author expects CoHDL to auto-detect these constraints — it does not (see Non-goals); Quilter's own detection runs independently on the plain netlist if the corresponding attribute is omitted.
- **An author expects a stated impedance/frequency to be validated as physically achievable** — CoHDL performs no such check; that is Quilter's own downstream validation.

## Migration path

No existing design requires migration — every attribute here is new, and diff_pair's extension is purely additive (bracket optional). A design wanting Quilter's real optimization benefit from these constraints needs real, non-mechanical authoring work to add the relevant attributes — genuine, disclosed follow-up work, not required by this RFC's completion bar.

## Decision

Accepted (redesigned same day) — 2026-07-20. Seven new structured attributes (#[ground(...)], #[high_current(...)], #[impedance(...)], #[bypass(...)], #[crystal_oscillator(...)], #[switching_converter(...)], #[bga_fanout]), each attached directly to the net/inst declaration it describes, reusing the existing #[name(...)] attribute bracket syntax — zero new bare keywords, correcting the withdrawn first draft's seven new statement-introducer tokens per Tony's direct correction. Plus an additive extension to the already-Accepted diff_pair layout{} statement (an optional [differential_impedance, single_ended_impedance, frequency] bracket), which stays a layout{} statement since it is inherently about a net pair. All map 1:1 to real, externally-documented Quilter physics-constraint schemas, grounded in eight real CSV files Tony supplied. cohdl build emits one CSV per kind, matching the real supplied files' headers/column order exactly. Zero schematic-correctness impact, by construction. Explicitly not auto-inferring any constraint. Recorded as a DR-033 revision (see note 7). Language Specification (note 10) gains a "Quilter physics-constraint hints" section, and diff_pair's existing entry is updated in place to document the new optional bracket.
