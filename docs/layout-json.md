# The layout-constraint artifact (`<name>-layout.json`)

RFC-013's layout constraints ride alongside the netlist in a **separate,
versioned artifact** — never merged into the `.net`/BOM connectivity data.
This document is the output contract a partner layout tool consumes.
Emitter: `src/emit/layout.rs`; conformance tests: `tests/layout.rs`.

## Lifecycle

- Written by `cohdl build` to `<out-dir>/<package-name>-layout.json` **only
  when** the design carries layout metadata (at least one `layout {}`
  constraint or `#[placement_hint]`).
- A design with no layout metadata emits **no** file, and a successful build
  **removes** a stale artifact left by an earlier build — the artifact
  directory always reflects current source.
- `cohdl build --json`'s `build` object carries the artifact path under
  `"layout"` (the key is absent when no artifact was emitted).
- Byte-stable: same source + same std ⇒ identical bytes (Constitution).

## Schema (version 1)

```json
{
  "schema_version": 1,
  "net_classes": [
    { "name": "HighSpeed", "nets": ["USB_DPX", "USB_DMX"] }
  ],
  "diff_pairs": [
    { "p": "USB_DPX", "n": "USB_DMX" }
  ],
  "length_matches": [
    { "nets": ["USB_DPX", "USB_DMX"], "tolerance": "0.15mm" }
  ],
  "placement_hints": [
    { "designator": "J3", "instance": "Pico2::usb", "hint": "board edge" }
  ]
}
```

- `schema_version` — integer; bumped only on a breaking change to this
  document's shape. Consumers must check it before parsing further.
- `net_classes[]` — `name` (string) and `nets` (string array). A class
  declared inside a `fn` body carries its call-chain-scoped name
  (`__fn0_routed_pair::HighSpeed`), mirroring RFC-006 net naming, so a
  layout-bearing fn called twice yields two distinct classes.
- `diff_pairs[]` — `p`/`n` in source order (the pair's polarity is the
  author's argument order). Always two **distinct** nets (compile-checked).
- `length_matches[]` — `nets` (≥ 2 distinct, compile-checked) and
  `tolerance`: a string or `null`. The value is **pass-through data** — CoHDL
  never enforces it (it has no geometry). A tolerance written as an RFC-001
  unit literal (`1ms`) passes through as its source text; the quoted-string
  form carries units CoHDL cannot represent (`"0.15mm"`).
- `placement_hints[]` — one entry per `#[placement_hint]`, in instance-path
  order: `designator` (the RFC-005-assigned reference, e.g. `J3`), `instance`
  (the full hierarchical path), `hint` (the opaque string).

## Identity rules

- **Net names are the final IR net names** — identical to the names in the
  emitted `.net` file. A source net that merged into a differently-named net
  (shared pin) is reported under the merged name, so this artifact and the
  netlist always agree.
- Ordering is deterministic: constraints in source/collection order,
  placement hints in instance-path order.
- Strings are JSON-escaped per RFC 8259 (the same escaper as `--json`).

## Guarantees and their boundary

Structural validity of the constraints themselves is compile-checked
(E1001–E1004 in `docs/error-codes.md`) — an *invalid* layout block is an
ordinary compile error and fails the build. The RFC-013 zero-impact guarantee
is therefore precisely this: **valid** layout metadata — present, absent, or
mutated — never changes the schematic verdict, any RFC-001–011 diagnostic,
designator assignment, or the `.net`/BOM bytes; only this artifact differs.

## Compatibility policy

Additive fields may appear in later versions without a `schema_version` bump;
consumers should ignore unknown keys. Any change to the meaning or shape of an
existing field bumps `schema_version`. The constraint vocabulary itself is
explicitly provisional per RFC-013's Decision — it is expected to be revisited
when a real partner layout-tool integration is scoped.
