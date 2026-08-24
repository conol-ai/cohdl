# Package API documentation (`cohdl docs` → registry API explorer)

Repository tooling, user-directed 2026-08-19 — no RFC. This document is the
contract shared by three implementations, which must agree exactly:

1. the Rust emitter (`src/emit/docsjson.rs`, surfaced as the `cohdl docs` verb
   and the post-publish upload in `publish_command`),
2. the registry worker (`registry/src/worker/apidocs.ts` + routes in
   `registry/src/worker/index.ts`), including the most-recently-published public-part
   search index derived from this document,
3. the registry UI (the package page's API explorer and its SVG previews).

The goal is docs.rs parity for CoHDL packages: every declaration in a
published package browsable on registry.cohdl.org, with auto-drawn schematic
symbol previews for devices and exact footprint previews.

## Position relative to the RFCs

- RFC-030 originally fixed five normative registry endpoints and left
  browser search/page design as implementation freedom. Its 2026-08-24
  amendment adds one stable discovery endpoint, `GET /search`, while leaving
  the browser's existing package-only `/api/search` shape unchanged. The
  API-doc sidecar remains a tooling artifact, but its package-local public
  `part` items are now the authoritative input to the most-recently-published part
  search index.
- RFC-030 line 47 binds the published tar to "the same content RFC-029's hash
  covers". Therefore the docs artifact is **never** part of the tar and never
  affects the content hash. It is a *derived, re-generatable view* uploaded as
  a sidecar — unlike the tar it is NOT identity and MAY be replaced
  (owner-authenticated, last write wins), e.g. after a compiler upgrade.
- RFC-017's `#[doc]` reference documents are unchanged. The existing
  per-version document index and `/api/doc` serving stay as they are; item
  pages additionally link each declaration's own `#[doc]` documents.
- Publishing remains compile-free in its verdict (a package that fails
  `cohdl check` still publishes). The docs upload after a successful publish
  is best-effort: extraction failure (check errors, unresolvable deps) prints
  a warning and never fails the publish.

## CLI surface

```
cohdl docs [PATH] [--out FILE] [--publish]
```

- Loads the package exactly like `check` (`resolve_manifest_deps` →
  `load_project_with_deps` → `pipeline::check_files_in_with_deps`).
- If diagnostics contain errors: print them, exit 1, emit nothing.
- Default: write the JSON to stdout. `--out FILE` writes the file instead
  (parent directory must exist; not part of `build`'s `out/` ownership).
  `--publish` without `--out` keeps stdout empty — the registry is the
  document's destination.
- `--publish`: additionally `PUT` the JSON to the registry for the manifest's
  `name@version`. Requires a stored token; the version must already be
  published; the server enforces ownership. This is the backfill path for
  versions published before this feature existed.
- `cohdl publish` runs the same extraction after a successful upload and
  `PUT`s the result. On extraction or upload failure it prints
  `api docs: skipped (...)` / a registry diagnostic and still exits 0 for the
  publish itself.
- Error mapping for the PUT reuses the E12xx block: missing token / 401 →
  E1201, 400/403/409/413 → E1202 (server's `error` string is the message),
  404 → E1203 ("publish the version first"), unreachable → E1204, and no new
  codes are introduced.

## Registry endpoints

- `PUT /packages/{name}/{version}/docs` — authenticated (Bearer token), owner
  of the package only, `(name, version)` must exist. Body: the JSON document
  below, `Content-Type: application/json`, at most **16 MiB**. The server
  validates only: body parses as JSON, top level is an object,
  `schema_version` is `1`, and `package.name`/`package.version` equal the URL.
  Deep schema validation is deliberately not re-implemented server-side; the
  UI renders every field as inert text/SVG (no HTML path exists). New uploads
  are byte-preserved at fixed-length, content-addressed R2 keys
  `apidocs/sha256/{sha256}.json`. The validated document envelope carries the
  package name and version; D1 records `versions.api_docs_size` and
  `versions.api_docs_r2_key`. Advancing that
  pointer and replacing the derived search rows happen in one transaction, so
  concurrent uploads remain internally consistent. Pre-migration sidecars at
  `apidocs/{name}/{version}.json` remain readable as a legacy fallback.
  Responses: `200 {"name","version","size"}`, or
  `400/401/403/404/413` with `{"error": "..."}`.
- `GET /api/apidocs?pkg={name}&version={ver}` — public; serves the stored
  JSON with `Content-Type: application/json`,
  `Cache-Control: public, max-age=600`, `X-Content-Type-Options: nosniff`.
  404 when absent.
- `GET /api/packages/{name}` — each version row gains `"api_docs": boolean`.
- `GET /search?q={term}&kind={all|package|part}&limit={1..50}&offset={0..10000}`
  — public, read-only package and most-recently-published public-part discovery. The
  query is trimmed, contains no Unicode control characters, has at least three
  Unicode scalar values, and is at most 128 UTF-8 bytes. `kind` defaults to
  `all`, `limit` to 20, and `offset` to 0; offset and truncation apply
  independently to packages and parts. Each response family carries
  `results` plus `has_more`, never a total count. The exact response schema is
  specified in RFC-030.

## Search-index projection

The sidecar remains byte-preserved in R2 and remains outside the immutable
package hash. In addition, when docs are uploaded for the package's most
recently published version, the registry projects a deliberately narrow
search record from `items`:

- only package-local entries with `kind: "part"` and `pub: true`;
- the `fq` must be rooted under the server-derived module root for the
  uploading package and its final segment must equal `name`; a sidecar cannot
  advertise another package's path as its own;
- never non-public entries and never the `foreign` array, which would
  duplicate dependency-owned parts under every consumer;
- the owning package name plus the part's `fq`, `name`, `device`, optional
  `intent`, arguments and variant;
- primary and alternate `fields` names and values within the fixed per-part
  projection budgets, so ordinary primary and alternate manufacturer/MPN
  lookups work.

The PUT envelope rules do not change: malformed item entries do not reject an
otherwise valid sidecar. Search extraction safely skips malformed,
non-public, non-part, foreign, and ownership-inconsistent entries. Fixed
budgets bound items inspected, arguments, AVL entries/fields, encoded row
size, and D1 insert chunks; pathological excess projection data is
deterministically omitted rather than failing or exhausting the upload. The
stored sidecar remains available byte-for-byte to the API explorer.

Uploading docs for the most-recently-published version atomically replaces
that version's search rows, so removed and renamed parts do not survive.
Uploading docs for an older version stores and serves its sidecar normally but
never displaces the current search index. Publishing a newer version clears
the prior rows until that new version's best-effort docs upload succeeds.

The search-index migration cannot derive rows from D1 alone because existing
sidecars live in R2. After deploying the index, the most-recently-published
version of each existing package is backfilled by re-running
`cohdl docs --publish`; the upload is already
owner-authenticated, re-uploadable and idempotent, and does not alter the
package tar or its RFC-029 content hash.

D1 migration `0002-api-docs.sql`: `ALTER TABLE versions ADD COLUMN
api_docs_size INTEGER` (NULL = no docs uploaded). Search adds migration
`0003-part-search.sql`, which adds the content-addressed R2 pointer and the
`part_search` FTS5 table with its trigram tokenizer. `schema.sql` mirrors both
migrations.

## The document (schema_version 1)

Produced only by the Rust emitter. Deterministic: same source + same exact
dependency set → same bytes. Hand-rolled JSON following `src/emit/json.rs`
conventions — `json_str` escaping, fixed key order, 2-space indent, optional
keys present only when they have content, one trailing newline, inputs
pre-sorted (the emitter never sorts at print time).

Value conventions:

- **Lengths in geometry** are canonical mm decimal strings from `geom::mm`
  (minimal decimal, trailing zeros trimmed, never `-0`): `"-0.95"`, `"1.2"`.
  The UI parses them with `Number(...)` for display-only SVG (f64 is fine
  client-side; nothing byte-stable is derived from it).
- **Unit values outside geometry** (spec values, generic defaults, part args)
  are the preserved source text: `"100nF"`, `"6V"`, `"10%"`.
- **Angles** are whole-degree integers (counter-clockwise positive, the
  authoring convention; SVG must apply `rotate(-angle)` since screen y is
  down — same frame as authoring, so x/y are used verbatim).
- Enum spellings are the language's own: obligations `required|optional`;
  roles `input|output|bidirectional|passive|power_in|power_out`; pad shapes
  `rect|circle|oval|annulus`; layers `top_copper|bottom_copper|through_all`;
  plating `smd|plated_through_hole`; corners `top_left|top_right|bottom_left|
  bottom_right`; mount-hole plating `plated|non_plated`; marker shapes
  `dot|triangle|band|arrow`.

Top level, in exactly this key order:

```json
{
  "schema_version": 1,
  "generator": "cohdl 0.3.0",
  "package": {
    "name": "@st/stm32",
    "version": "0.1.0",
    "root": "st_stm32",
    "description": "…",
    "license": "MIT",
    "repository": "…"
  },
  "dependencies": [ { "name": "std", "version": "0.3.0", "root": "std" } ],
  "items": [ … ],
  "impls": [ … ],
  "foreign": [ … ]
}
```

- `package.root` / `dependencies[].root` — the sanitized module root
  (`pipeline::package_root`): `@st/stm32` → `st_stm32`. The UI uses this map
  to turn any foreign fq path into a link to the owning package's page.
- `description`/`license`/`repository` — present only when set in the
  manifest.
- `dependencies` — the resolved exact dependency set, std first then name
  order (the same order the pipeline uses).

### Items

`items` holds every top-level **named** declaration of this package (`use`
imports and `impl`s excluded — impls have their own section), sorted by `fq`
(designs, which are bare-named, sort by their bare name in the same list).
Common keys, in order:

```json
{
  "fq": "passive::resistors_0402::RC0402FR_0710KL",
  "name": "RC0402FR_0710KL",
  "kind": "part",
  "pub": true,
  "module": "passive::resistors_0402",
  "file": "src/resistors_0402.cohdl",
  "line": 17,
  "intent": "…",
  "docs": ["docs/yageo_rc_l.pdf"],
  "<kind>": { … }
}
```

- `module` — the fq minus its last segment; for designs (bare-named,
  project-global), the module of their defining file. Designs are neither
  importable nor privatable, so they always carry `"pub": true`.
- `file` — the pipeline display name (`src/...` for project files), which is
  exactly the path `GET /api/doc?path=` serves out of the published tar; the
  UI's "view source" fetches it and jumps to `line` (1-based, the name
  ident's line).
- `intent` — the item's `#[intent]` string, only when present.
- `docs` — the item's `#[doc]` paths in source order, only when non-empty;
  each is servable via `/api/doc`.
- `<kind>` — one key named exactly `trait|device|fn|part|pad|footprint|design`
  holding the kind payload.

#### `trait`

```json
{ "super_traits": ["std::TwoTerminal"],
  "designator_prefix": "C",
  "pins": [ { "name": "A", "obligation": "required" } ],
  "specs": [ { "name": "capacitance", "type": "Capacitance" } ] }
```

`designator_prefix` only when declared. Trait pins have no role/number by
design — no symbol preview for traits.

#### `device`

```json
{ "generics": [
    { "name": "C", "bound": { "unit": "Capacitance" } },
    { "name": "V", "bound": { "unit": "Voltage" }, "default": "16V" },
    { "name": "D", "bound": { "traits": ["std::Capacitor"] } } ],
  "variants": ["TSOT235"],
  "designator_prefix": "U",
  "pins": [
    { "variant": "TSOT235",
      "pins": [ { "name": "VIN", "obligation": "required",
                  "numbers": ["1"], "role": "power_in" } ] } ],
  "specs": [
    { "variant": "TSOT235",
      "fields": [ { "name": "voltage_rating", "value": "6V" },
                  { "name": "capacitance", "generic": "C" } ] } ]
}
```

- `variants` only when declared. For a variant-less device, `pins`/`specs`
  hold exactly one entry with no `variant` key. For a varianted device, one
  entry per variant in declaration order, with pins from `pins_for(variant)`
  and the **merged** spec view from `spec_fields_for(variant)` — the docs
  show what an instantiation of that variant sees.
- A spec field carries either `value` (literal, source text) or `generic`
  (parameter name), never both.
- `designator_prefix` — `World::designator_prefix` (the trait-derived prefix,
  `"U"` default), always present.

#### `fn`

```json
{ "generics": [ { "name": "V", "bound": { "unit": "Voltage" } } ],
  "params": [
    { "name": "vdd", "type": { "kind": "pin" } },
    { "name": "c", "type": { "kind": "generic", "name": "D" } },
    { "name": "x", "type": { "kind": "impl", "traits": ["std::Capacitor"] } } ],
  "insts": [ { "name": "c", "type": "passive::devices::MLCC",
               "args": ["100nF", "16V", "10%"] } ],
  "calls": ["passive::circuits::decoupling_100n"],
  "nets": 2 }
```

`insts`/`calls` in body order (`calls` deduplicated, sorted); `nets` counts
`net` statements (anonymous included). Same `insts`/`calls`/`nets` summary on
`design`. `args`/`variant` on an inst entry only when present; an RFC-024
array instance additionally carries `"array": N`. `insts`/`calls` are omitted
when empty (the optional-keys convention); `nets` is always present.

#### `part`

```json
{ "device": "passive::devices::MLCC",
  "args": ["100nF", "16V", "10%"],
  "variant": "C0402",
  "primary": { "fields": [ { "name": "mfr", "value": "Samsung" },
                            { "name": "mpn", "value": "CL05B104KO5NNNC" } ],
               "footprint": "passive::footprints::CHIP_0402" },
  "alts": [ { "fields": [ { "name": "mpn", "value": "GRM155R71C104KA88D" } ] } ]
}
```

`args`/`variant` only when the device reference carries them. `fields` in
source order (the AVL entry's own name/value pairs — `mpn`, `mfr`, …);
`footprint` (fq) only when the entry declares one.

#### `pad`

All geometry mm strings. Keys present only when the declaration carries them
(`shape`/`size` are practically always present):

```json
{ "shape": "rect", "size": ["0.825", "0.25"],
  "layer": "top_copper", "plating": "smd",
  "drill": { "round": "0.3" },
  "chamfer": { "corner": "top_left", "cut": "0.2" },
  "corner_radius": "0.1",
  "mask_expansion": "0.05",
  "paste": "none" }
```

`drill` is `{ "round": d }` or `{ "slot": [w, l] }`. `paste` is `"none"`,
`{ "rect": [w, h] }`, or `{ "segmented_annulus": [outer, inner, gap] }`.
`size` arity follows the shape (circle 1; rect/oval/annulus 2 — annulus is
`[outer_diameter, inner_diameter]`).

#### `footprint`

```json
{ "placeholder": false,
  "pads": [ { "number": "1", "pad": "passive::pads::P_CHIP_0402",
              "x": "-0.51", "y": "0", "rotate": 90 } ],
  "mount_holes": [ { "number": "1", "plating": "non_plated",
                     "shape": "rect", "x": "-6.75", "y": "0",
                     "size": ["2", "1.5"] } ],
  "courtyard": { "shape": "rect", "at": ["0", "0"], "size": ["1.54", "0.94"] },
  "window": { … same shape as courtyard … },
  "silkscreen_ref": { "at": ["0", "-2"] },
  "markers": [ { "kind": "pin_1_marker", "pad": "1", "shape": "dot" } ],
  "silk": [
    { "kind": "line", "from": ["-1", "0.5"], "to": ["1", "0.5"], "width": "0.12" },
    { "kind": "circle", "at": ["0", "0"], "radius": "0.2", "width": "0.1", "fill": true },
    { "kind": "arc", "at": ["0", "0"], "radius": "1", "start_angle": 0,
      "end_angle": 90, "width": "0.12" },
    { "kind": "polygon", "points": [["0","0"], ["1","0"], ["0","1"]], "fill": true } ]
}
```

- `placeholder` — the stage-one empty-body case; when `true` every other key
  is absent and the UI shows a placeholder note instead of a preview.
- `pads` in source placement order; `rotate` omitted when 0; the `pad` fq
  resolves inside `items` or `foreign`.
- `mount_holes`: circle geometry carries `diameter`, rect/oval carry `size`;
  `shape` is always present (default circle applied).
- `silk` is the **expanded** graphics list from `emit::silk::graphics` — the
  identical expansion the `.kicad_mod` and IPC-2581 emitters consume, so the
  semantic markers' computed geometry (standoffs, dot/triangle/band/arrow)
  can never drift from the shipped artifacts. `markers` additionally records
  the semantic markers as authored, for textual display (`pad` names the
  target pad number for both marker kinds). `fill` is a boolean.

#### `design`

```json
{ "insts": [ … ], "calls": [ … ], "nets": 4 }
```

### Impls

`impls` — the `impl` declarations of this package, sorted by
`(trait, device)`:

```json
{ "trait": "std::Capacitor", "device": "passive::devices::MLCC",
  "file": "src/devices.cohdl", "line": 40,
  "pin_map": [ { "role": "A", "pin": "P1" } ],
  "spec_map": [ { "field": "capacitance", "spec": "capacitance" } ] }
```

`pin_map`/`spec_map` are the RESOLVED maps (`World.resolved_impls`), present
only when non-empty; sorted by role/field name. Item-level `#[intent]` on
impls is not carried in v1 (the resolver drops it for unnamed items — noted
here as a disclosed gap).

### Foreign items

`foreign` — declarations that live in **dependency** packages but are needed
to render this package's pages self-contained, in the same item shape as
`items` (including `module`/`file`/`line` relative to their own package),
sorted by fq. Included exactly:

- every pad referenced by a local footprint's placements,
- every footprint referenced by a local part's AVL entries, plus every pad
  those footprints reference,
- every device referenced by a local part.

Traits and other references are NOT inlined — the UI links them via the
`dependencies` root map. A foreign item's `file` is only meaningful within
its own package's tar (`src/<rel>` for the ordinary layout; bare `<rel>`
for a src-less directory dependency); the UI does not offer "view source"
for foreign items. All `file` fields are `/`-separated on every platform.

## UI contract (summary)

- The package page gains an **API** tab (alongside the existing
  Documentation/Versions), driven by URL search params on the existing
  `/package/$` route: `view=api` selects the tab, `item=<fq>` deep-links one
  item's page, `q=` filters, `kind=` filters by declaration kind. Latest
  version stays canonicalized to no `?version=`.
- The explorer follows the `Documents`/`.document-layout` precedent: sticky
  left rail (kind groups + module tree + filter box), content pane with item
  list or item detail. Non-`pub` items are shown de-emphasized behind a
  "show private items" toggle — the package's public API is the `pub` set.
- **Symbol previews** (devices, and parts via their device): auto-drawn
  IC-style box, client-side SVG. Side assignment is a tooling-layer
  convention, not a language guarantee: `input`/`passive` left; `output`,
  `bidirectional`, `power_out` right; `power_in` top — except ground-looking
  names (`GND`, `VSS`, `AGND`, `DGND`, `PGND`, `EP`) bottom. Pin stubs carry
  the physical number(s); `optional` pins render dashed. Varianted devices
  get a variant selector.
- **Footprint previews**: exact SVG from the payload — pads by resolved shape
  (rect/circle/oval/annulus, roundrect via `corner_radius`, one-corner
  chamfer), drills and slots, plated/non-plated mount holes, courtyard,
  window, expanded silkscreen, `REF**` anchor. Authoring frame is y-down —
  identical to SVG — and pad rotation is applied as `rotate(-angle)` about
  the pad centre. Hovering a pad highlights it and shows its number; pads
  render their pin number, and signal names are drawn beside pads when a
  bound device is known (from the part context, or on footprint pages via
  the first part referencing the footprint); a scale bar shows mm.
- Everything renders as text/attributes through React — the no-raw-HTML rule
  of the Markdown renderer extends to every docs-JSON string.

## Determinism & zero impact

- The emitter follows the house rules: BTreeMap iteration or pre-sorted
  vectors only, shared `json_str` escaping, `geom::mm` for all lengths, no
  `f64` anywhere.
- Generating docs must not change any existing artifact or verdict:
  `check`/`build` outputs, diagnostics, designators, and lock bytes are
  untouched (pinned by test, like `tests/intent.rs`).
- `tests/apidocs.rs` pins: byte-stable output (double run), the schema shape,
  the zero-impact property, variant/generic/foreign coverage, and a scale run
  over `lib/passive` (~9k parts, including the legitimately empty generated
  module).
