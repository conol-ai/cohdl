# Error-code registry (formal, v2 baseline — RFC-011)

A code is issued once and never repurposed. If a check's behavior changes
enough that its old meaning no longer applies, retire the code (mark
`[DEPRECATED]`, keep the row, keep the meaning documented) and issue a new one
— never edit an existing code's meaning in place.

**Organizing principle** (RFC-011): a block is chosen by **kind of mistake**,
not by which compiler pass happens to catch it. That is why unit-mismatch
diagnostics live in E1xx even when the mismatch is caught at a *generic*
substitution site — unit-mismatch is unit-mismatch regardless of call site.

Severities: all `Exxx` are errors; `Dxxx` severity is per-rule.

## Block ownership

| Block | Owner mechanism |
|---|---|
| E00x | CLI invocation (pre-pipeline — not a source diagnostic) |
| E0xx | Lexing & parsing |
| E1xx | Unit system (RFC-001) — all unit-mismatch/unit-literal diagnostics, regardless of call site |
| E2xx | Name resolution |
| E3xx | Trait satisfaction at impl (RFC-003) |
| E4xx | Generics (RFC-007), excluding unit-mismatch (that is E1xx) |
| E5xx | Sub-circuit fns (RFC-006) |
| E6xx | Design assembly & nets |
| E7xx | Pin connection obligations (RFC-002) |
| E8xx | Designators & parts (RFC-005) |
| E9xx | Structural variants (RFC-008) |
| E10xx | Layout constraints (RFC-013) |
| D00x | Residual DRC (RFC-004) — exactly four, never more |

**Enforcement**: `tests/error_registry.rs` runs the RFC-011 completeness check
in both directions on every build — every code literal in `src/` appears as a
row here, and every row here that is not `[DEPRECATED]` / `[RESERVED]` /
CLI-only has at least one real call site in `src/`.

## E00x — CLI invocation (not a source diagnostic)

| Code | Meaning |
|---|---|
| E000 | invocation-level failure — bad flags, an invalid flag for the command, missing path/project, design-selection failure, or nothing-to-build. Exit code 2, prose on stderr, never inside a `--json` diagnostics array; source diagnostics collected before the failure still render to stderr first. CLI-only; not a source diagnostic, so it has no `Diagnostic` call site by design. (Classifying post-collection selection failures here is a documented deviation from RFC-010's pre-collection wording, pending amendment.) |

## E0xx — lexing & parsing

| Code | Meaning |
|---|---|
| E001 | unexpected character (includes the targeted `°C` → `C` guidance; standalone `Ω` is E107) |
| E002 | unterminated string literal |
| E010 | unexpected token (expected …, found …) |

## E1xx — unit system (RFC-001)

| Code | Meaning |
|---|---|
| E101 | non-ASCII unit spelling (`Ω`, `°C`) directly after a number |
| E102 | negative bare number (only `Temperature` and `Length` literals may be negative) |
| E103 | unknown unit suffix |
| E104 | SI prefix not allowed for this unit (incl. any prefix on `Temperature`/`Tolerance`) |
| E105 | leading `-` on a unit literal whose type is not signed (only `Temperature` and `Length` may carry a sign) |
| E106 | literal not exactly representable (too precise / out of range) |
| E107 | standalone Unicode `Ω` (no preceding number) — narrower than E101, so the message can be maximally specific (RFC-011) |
| E110 | unit-type mismatch — always names expected vs. actual (e.g. "expected `Voltage`, found `Capacitance`") |
| E111 | bare number where a unit-typed value is required |
| E112 | unit-type generic argument has the wrong unit type (RFC-011: relocated from the retired E402 — unit-mismatch belongs in E1xx) |
| E113 | bare number as a unit-type generic argument (RFC-011: relocated from the retired E404) |

## E2xx — name resolution

| Code | Meaning |
|---|---|
| E201 | duplicate top-level declaration |
| E202 | unknown name |
| E203 | unknown pin on device/trait |
| E204 | `[RESERVED, not yet implemented]` unknown spec field — no call site yet |
| E205 | name is the wrong kind (e.g. a trait used where a device is required) |
| E206 | instance/net names beginning with `__` are reserved for compiler-generated expansion names |
| E207 | ambiguous unqualified name (RFC-016) — declared at more than one module path; names every candidate, suggests qualifying or `use` |
| E208 | `use` collision (RFC-016) — one local name imported from two different paths; names both |
| E209 | visibility violation (RFC-016) — a non-`pub` item referenced from another package; names the item and its declaring package |
| E210 | unspellable module-path segment (RFC-016) — the package root, a `src/`/`std/` subdirectory, or a nested-file name is a keyword or non-identifier, so its declarations cannot be referenced by any qualified path |

## E3xx — trait satisfaction at impl (RFC-003)

| Code | Meaning |
|---|---|
| E301 | `impl Trait for Device` unsatisfied — names the trait, the device, and the exact missing/mismatched pin role or spec field |
| E302 | missing sub-trait impl — names the required sibling `impl` (chain diagnostic) |
| E303 | duplicate `impl` for the same (trait, device) — points at the earlier one |
| E304 | impl mapping names a role/field the trait does not require |
| E305 | impl mapping target is not a pin/spec of the device |
| E306 | cyclic sub-trait bounds |

## E4xx — generics (RFC-007)

E4xx is generics-*specific* mistakes only; unit-mismatch at a generic site is
E1xx (E112/E113), not here — see the organizing principle above.

| Code | Meaning |
|---|---|
| E401 | wrong number of generic arguments |
| E402 | `[DEPRECATED → E112]` unit-type generic argument has the wrong unit type — retired name, relocated to E1xx by RFC-011 |
| E403 | trait-bound not satisfied at instantiation — names the missing trait and the concrete type |
| E404 | `[DEPRECATED → E113]` bare number as a unit-type generic argument — retired name, relocated to E1xx by RFC-011 |
| E405 | generic argument is not concrete after substitution |
| E406 | invalid generic parameter declaration (e.g. default on a trait-bound parameter) |

## E5xx — sub-circuit fns (RFC-006)

| Code | Meaning |
|---|---|
| E501 | cyclic fn call chain — the message shows the full cycle |
| E502 | wrong number of call arguments |
| E503 | call argument kind mismatch (pin vs. instance) |
| E504 | unknown fn |

## E6xx — design assembly & nets

| Code | Meaning |
|---|---|
| E601 | `[RESERVED, not yet implemented]` floating net — a net resolving to zero instance pins (planned reclassification of RFC-004's W002); no call site yet |
| E602 | net/nc member is not a known pin |
| E603 | contradictory annotations on a merged net (two voltages, or voltage + `gnd`) |

## E7xx — pin connection obligations (RFC-002)

| Code | Meaning |
|---|---|
| E701 | required pin unresolved — appears in neither `net` nor `nc` |
| E702 | required pin contradictory — appears in both `net` and `nc` |

## E8xx — designators & parts (RFC-005, provisional part binding)

| Code | Meaning |
|---|---|
| E801 | instance not part-bound at `build` (netlist/BOM would lie) |
| E802 | invalid `part` declaration (missing `mpn`/`footprint` on `primary`, missing `mpn` on `alt`, non-concrete device) |
| E803 | `#[designator]` override collision |
| E804 | invalid designator format (must be PREFIX + number, e.g. `U7`) |
| E805 | invalid `pad` declaration (RFC-018) — missing/unknown field, non-`Length` dimension, non-positive size/drill extent, size arity vs shape, or the `drill` ⇔ `plated_through_hole` biconditional |
| E806 | invalid `footprint` body (RFC-018) — duplicate pad number, malformed member, duplicate courtyard/silkscreen_ref, non-`Length` coordinate, non-positive courtyard extent |
| E807 | footprint/device pad mismatch at `build` (RFC-018) — the footprint's pad numbers must exactly match the bound device's physical pin numbers; names the missing/extra numbers |
| E808 | malformed IPC-7351 footprint name (RFC-021) — a `footprint` identifier whose prefix IS a closed IPC-7351 family (QFP/QFN/SOIC/SOP/SOT/BGA/CHIP/MELF) but does not parse against that family's grammar (missing/invalid density suffix `{N,L,M}`, non-numeric or misordered dimension field, trailing characters); names the specific parse failure. A name whose prefix is outside the closed set is a free-form identifier and is not checked. |
| E809 | IPC-7351 name-vs-geometry mismatch (RFC-021) — the footprint identifier's declared pin count or pitch disagrees with the footprint's own pad placements (pins = pad count, minus the `_1EP` exposed pad; pitch = closest pad-center spacing); names the footprint and both values |

## E9xx — structural variants (RFC-008)

| Code | Meaning |
|---|---|
| E901 | device pin has no role annotation — every pin needs an explicit role; the message lists the six valid roles |
| E902 | a declared variant has no `pins[VARIANT]` block (exhaustiveness at the device declaration, naming the missing variant) |
| E903 | a `[VARIANT]` selector names an undeclared variant — the message lists the valid set |
| E904 | `[VARIANT]` selector omitted on a device that declares variants (no implicit default) — the message lists the valid set |
| E905 | `[VARIANT]` selector on a device with no variants, or on a part (parts already select theirs) |
| E906 | duplicate variant name in `variants { }` (checked at parse) |
| E907 | `pins[X]`/`spec[X]` qualifier names an undeclared variant |
| E908 | unqualified `pins { }` block on a device that declares variants |

**Reconciliation with RFC-011's E9xx table — a deviation pending amendment,
not compliance.** RFC-011's Accepted text proposes a five-code block
(E901–E905) with E902 = "missing selector", E904 = "missing `pins[VARIANT]`",
E905 = "duplicate variant"; this repository issues the eight codes above
instead. The engineering reasons to keep them: the eight predate the accepted
table (RFC-011 was drafted against a snapshot where the variant diagnostics
were "not yet wired"), they have real call sites and fixture tests, and
RFC-011's own stability rule forbids repurposing an already-issued code —
renumbering E902/E904/E906 to match the table would violate that rule on day
one. But implemented-first is a reason to *amend the note*, not a claim that
the Accepted text is satisfied: until the RFC-011 table is amended to the
eight-code assignment, this block is a **documented deviation**
(docs/compliance-report.md tracks it).

## E10xx — layout constraints (RFC-013)

Structural validation of `layout {}` constraints against their own closed
vocabulary — never a connectivity/DRC check, never affecting the netlist bytes.

| Code | Meaning |
|---|---|
| E1001 | a layout constraint references a net that is not declared in the design |
| E1002 | duplicate `net_class` name |
| E1003 | `diff_pair` does not name exactly two nets |
| E1004 | `length_match` names fewer than two nets |
| E1005 | `[RESERVED, not yet implemented]` `net_class` referenced before declaration — activates only once a future constraint kind references a `net_class` by name (the four current kinds reference nets, not classes) |
| E1006 | invalid `board_outline: "path.dxf"` (RFC-020). Check-time sub-cases: the path is not project-relative (absolute, `..`-escaping, a URL, or a drive letter), more than one outline, or an outline inside a called `fn` rather than the design's own `layout {}` block. Build-time sub-cases (when the DXF is read): the file cannot be read, is not valid DXF, has no closed polyline on the `Edge.Cuts` layer, the outline polyline is not closed, or has fewer than 3 vertices. E10xx family |
| E1007 | invalid `place <inst> at (x, y) [rotate ANGLE]` (RFC-020): the named instance does not exist among the design's own top-level instances, a coordinate is not a `Length`/`mm` value or is out of geometry range, `rotate` is not one of the closed set {0, 90, 180, 270}, the instance is placed more than once, or the `place` appears inside a called `fn`. (Placing an instance declared inside a called `fn` is a disclosed, deferred gap — RFC-020 Non-goals.) E10xx family |

## D00x — residual DRC (RFC-004; exactly four, never more)

| Code | Severity | Rule |
|---|---|---|
| D001 | error | voltage-exceed: `voltage_rating` spec < annotated net voltage |
| D002 | error | polarity-mismatch: `Polarized` anode pin on a `[gnd]` net |
| D003 | warning | single-driver: net whose only connected pin is a driver (`output`/`power_out`) — the driver drives nothing |
| D004 | error | multi-driver: net with ≥2 driver-type pins (`output`/`power_out`) |
