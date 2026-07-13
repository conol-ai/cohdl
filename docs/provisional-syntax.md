# Provisional constructs (MVP-pragmatic, pre-RFC)

The Language Specification (`docs/design/10-language-specification.md`) lists
constructs that are load-bearing for any real design but have **no Accepted
RFC yet** (its "Not yet specified" section). The MVP demo cannot run without
some of them, so this document pins down the smallest honest version of each.
Everything here is **provisional**: an Accepted RFC on the conol.ai design
repository supersedes this file, and each choice is written to be replaceable
without disturbing the accepted mechanisms (RFC-001…007).

Design discipline applied throughout: no string heuristics, no magic defaults,
nothing the type checker can't inspect (Constitution hard constraints). Where
v1 (branch `legacy`) had a convention-hack, v2 makes the fact explicit syntax.

## 1. Projects, files, and scope

- A **project** is a directory with a `cohdl.toml` manifest:

  ```toml
  [package]
  name = "demo-board"          # output file base name

  [design]
  top = "Board"                # the `design` to compile
  ```

- Source files are all `*.cohdl` under the project's `src/`, plus the std
  library (all `*.cohdl` under the compiler's `std/`; `--std <dir>` overrides,
  `--no-std` omits).
- **One flat global scope.** All top-level declarations across all files share
  one namespace; duplicate names are a compile error (`E201`). There are no
  `module`/`use` declarations in the MVP (modules are "Not yet specified" in
  note 10; RFC-003's "different module" organization is still expressible as
  different *files*). `pub` is accepted and recorded but not enforced (single
  flat scope has no visibility boundary yet).
- `cohdl check <dir>` / `cohdl build <dir>` also accept a single `.cohdl` file
  (treated as a one-file project named after the file, std included).

## 2. `part` — MPN binding (Conceptual Model's Part concept)

Carried forward from v1's real shape, adapted to v2's positional generics:

```cohdl
pub part MLCC_100nF_16V: MLCC<100nF, 16V, 10%> {
    primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC", footprint: "Capacitor_SMD:C_0402_1005Metric" }
    alt     { mfr: "Murata",  mpn: "GRM155R71C104KA88D" }
}
```

- A `part` binds a **fully-concrete device instantiation** (every generic
  argument a literal — no generic parameters may remain open) to an AVL.
- Exactly one `primary`, zero or more `alt`. `primary` **must** carry `mpn`
  and `footprint` (checked the moment the `part` is declared — the Conceptual
  Model's "MPN binding is non-optional at declaration" promise). `alt` must
  carry `mpn`. `mfr` is optional metadata. `footprint` is a verbatim KiCad
  footprint string (the MVP's one codegen target).
- An instance binds to a part in one of two ways:
  1. **By name:** `inst c1: MLCC_100nF_16V` — the part is used as the type.
  2. **By exact match:** `inst c1: MLCC<100nF, 16V, 10%>` — after
     monomorphization, if a part exists whose device and resolved spec values
     match exactly, the instance binds to it. If several match, the
     lexicographically-smallest part name wins (deterministic; noted in the
     build output).
- `cohdl build` (emitting netlist + BOM) **requires every instance to be
  part-bound** (`E801` otherwise) — this is what makes "the BOM lies"
  structurally impossible. `cohdl check` does not require part binding.

## 3. Pin roles — superseded by RFC-008 (now Accepted, no longer provisional)

This section originally made the role annotation optional with a documented
`passive` default. **RFC-008 (Accepted 2026-07-13) retired that default**:
every device pin now carries an explicit role annotation from the closed
six-value set, and an unannotated pin is a compile error (`E901`). Package
variants (`variants {}`, `pins[VARIANT]`, `spec[VARIANT]`, `[VARIANT]`
selectors) are likewise RFC-008-governed. See
`docs/design/rfc-008-pattern-matching.md` and the Language Specification's
"Structural variants" section — those are normative; nothing here overrides
them.

```cohdl
pub device AP2112K_3V3 {
    pins {
        required VIN:  1 [power_in]
        required GND:  2 [power_in]
        required EN:   3 [input]
        optional NC:   4 [passive]     // explicit — no unannotated pins
        required VOUT: 5 [power_out]
    }
}
```

- Role vocabulary (closed): `input`, `output`, `bidirectional`, `passive`,
  `power_in`, `power_out`. Trait pins stay abstract (`required A: pin`) and
  take no role annotation.
- **Driver-type pins** = `output` and `power_out`. Only the two driver DRC
  rules (D003/D004) consume roles.

## 4. Net annotations (needed by voltage-exceed / polarity DRC)

v1 inferred net voltage and "is GND" from net-name string parsing, which
failed on real designs. v2 annotates explicitly, in brackets after the name:

```cohdl
net VDD_3V3 [3.3V]: ldo.VOUT, mcu.VDD, c1.A
net GND     [gnd]:  usb.GND, mcu.GND, c1.B
net USB_DM:         usb.DM, mcu.USB_DM        // un-annotated is fine
```

- At most one annotation per `net` declaration: a `Voltage` literal (the net's
  nominal voltage) or the marker `gnd`. A non-Voltage unit literal here is a
  unit-type error (RFC-001's comparison discipline).
- When nets merge (below), conflicting voltage annotations (different values)
  are a compile error; `gnd` + a voltage annotation on the same merged net is
  likewise contradictory. Merging identical annotations is fine.
- DRC reads **only** these annotations — never net names.

## 5. Net semantics and merging

- `net NAME: member, member, …` — members are instance-pin references
  (`inst.PIN`), fn-parameter pins, or trait-role pins on generic-typed
  parameters (`target.A`, resolved through the impl mapping at
  monomorphization, per RFC-003/007).
- The same pin appearing in two `net` declarations merges them into one
  electrical net (union-find, as in v1) — this is also how a `fn`'s internal
  nets join the caller's nets through pin arguments.
- The merged net's emitted name is the lexicographically-smallest
  design-body-level name among its declarations; if none (all from expanded
  `fn` bodies), the smallest call-chain-qualified name. Deterministic.
- `net _: …` — anonymous; gets its call-chain name (RFC-006). Anonymous nets
  are the normal form inside `fn` bodies.
- A `net` declaration must name at least one member (grammar) and, after
  expansion, at least one instance pin (`E601` — RFC-004's W002
  reclassification: floating net is a compile-time structural error).
- `nc: pin, pin, …` — allowed in `design` **and** `fn` bodies (RFC-002 defines
  it as syntactically parallel to `net`; allowing it where `net` is allowed is
  the reading that lets a sub-circuit `fn` resolve its own instances' unused
  pins). The exhaustiveness check itself still runs once, at final design
  assembly, per RFC-002.

## 6. `designator_prefix` on traits (RFC-005's prefix mapping)

Carried forward from v1 verbatim (RFC-001's example already shows it):

```cohdl
pub trait Capacitor: TwoTerminal {
    designator_prefix: "C"
    spec { capacitance: Capacitance, voltage_rating: Voltage, tolerance: Tolerance }
}
```

- Only traits carry `designator_prefix`. A device's prefix = the prefix of the
  lexicographically-smallest trait name among its implemented traits that
  declares one; default `"U"` when none does (v1-compatible).
- `#[designator("U7")]` on an `inst` statement overrides the full designator
  (prefix + number), resolved before fresh assignment per RFC-005.

## 7. Residual-DRC rule concretions (RFC-004's four rules)

| Code | Rule | Fires when |
|---|---|---|
| `D001` | voltage-exceed (error) | an instance has a `voltage_rating` spec (that exact field name, `Voltage`-typed) and is connected to a net whose voltage annotation exceeds it |
| `D002` | polarity-mismatch (error) | a device with `impl Polarized` (std trait: pin roles `Anode`/`Cathode`) has its Anode-mapped pin on a `[gnd]`-annotated net |
| `D003` | single-driver (warning) | a net has exactly one connected instance pin — likely unfinished wiring (v1-faithful count check; the pin's role is reported in the message) |
| `D004` | multi-driver (error) | a net has two or more driver-type pins (`output`/`power_out`) |

Nothing beyond these four — a fifth "structural" rule request means re-running
the type-system-first test (RFC-004 Tooling & operations).

## 8. fn parameters and call arguments (completing RFC-006/007's surface)

- Parameter types: `Pin` (a pin reference), a generic type parameter name
  (`target: D`), or `impl Trait` (desugared per RFC-007).
- Call arguments: a pin reference (`mcu.VDD`, `ferrite.OUT`, or a `Pin`
  parameter being passed through) for `Pin` parameters; an instance name for
  generic/`impl Trait` parameters.
- Calls: `name::<generic-args>(args)` (turbofish, per RFC-006's examples) or
  `name(args)` when the fn has no generic parameters. Generic args are
  positional, as everywhere in v2.

## 9. What the MVP deliberately leaves out (beyond the MVP cut list)

No `module`/`use`, no `type` aliases, no inline AVL on `inst`, no
`footprint_alias`/`footprint_override`/`no_footprint`, no `rule` blocks in
source (the four DRC rules are engine-builtin; in-language `rule` syntax
returns with a future RFC), no bare `Ident` external net endpoints (v1's
"external" nets — every net member must be a real pin). Package/footprint
variants, formerly on this list, landed via RFC-008.
