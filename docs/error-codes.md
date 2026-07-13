# Error-code registry (informal, stable)

RFC-011 (formal registry) is cut from the MVP; per the MVP Definition, codes
exist here as **stable strings** — a code is never repurposed, only
deprecated. Blocks are reserved per mechanism, mirroring the RFC structure.

Severities: all `Exxx` are errors; `Dxxx` severity is per-rule.

## E0xx — lexing & parsing

| Code | Meaning |
|---|---|
| E001 | unexpected character (includes targeted `Ω` → `ohm`, `°C` → `C` guidance) |
| E002 | unterminated string literal |
| E010 | unexpected token (expected …, found …) |

## E1xx — unit system (RFC-001)

| Code | Meaning |
|---|---|
| E101 | non-ASCII unit spelling (`Ω`, `°C`) directly after a number |
| E102 | negative bare number (only `Temperature` literals may be negative) |
| E103 | unknown unit suffix |
| E104 | SI prefix not allowed for this unit (incl. any prefix on `Temperature`/`Tolerance`) |
| E105 | leading `-` on a non-`Temperature` unit literal |
| E106 | literal not exactly representable (too precise / out of range) |
| E110 | unit-type mismatch — always names expected vs. actual (e.g. "expected `Voltage`, found `Capacitance`") |
| E111 | bare number where a unit-typed value is required |

## E2xx — name resolution

| Code | Meaning |
|---|---|
| E201 | duplicate top-level declaration |
| E202 | unknown name |
| E203 | unknown pin on device/trait |
| E204 | unknown spec field |
| E205 | name is the wrong kind (e.g. a trait used where a device is required) |

## E3xx — trait satisfaction at impl (RFC-003)

| Code | Meaning |
|---|---|
| E301 | `impl Trait for Device` unsatisfied — names the trait, the device, and the exact missing/mismatched pin role or spec field |
| E302 | missing sub-trait impl — names the required sibling `impl` (chain diagnostic) |
| E303 | duplicate `impl` for the same (trait, device) — points at the earlier one |
| E304 | impl mapping names a role/field the trait does not require |
| E305 | impl mapping target is not a pin/spec of the device |

## E4xx — generics (RFC-007)

| Code | Meaning |
|---|---|
| E401 | wrong number of generic arguments |
| E402 | unit-type generic argument has the wrong unit type |
| E403 | trait-bound not satisfied at instantiation — names the missing trait and the concrete type |
| E404 | bare number as a unit-type generic argument |
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
| E601 | floating net — a net resolving to zero instance pins (RFC-004's W002, reclassified as a type error) |
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

## D00x — residual DRC (RFC-004; exactly four, never more)

| Code | Severity | Rule |
|---|---|---|
| D001 | error | voltage-exceed: `voltage_rating` spec < annotated net voltage |
| D002 | error | polarity-mismatch: `Polarized` anode pin on a `[gnd]` net |
| D003 | warning | single-driver: net with exactly one connected pin |
| D004 | error | multi-driver: net with ≥2 driver-type pins (`output`/`power_out`) |
