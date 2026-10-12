# CoHDL Schematic Explorer

A read-only, interactive projection of a checked CoHDL design: every
instance, net and pin the compiler resolved, drawn as a schematic-style
board you can search, trace and inspect. It never edits a circuit — the
`.cohdl` source stays the single source of truth.

## Run

**macOS app:** download the signed, notarized DMG from the latest
[`explorer-v*` release](https://github.com/conol-ai/cohdl/releases?q=explorer&expanded=true),
drag CoHDL Explorer to Applications, open it, and pick a CoHDL project
directory. The app serves on a free loopback port, opens your browser, and
quits itself a few minutes after the last tab closes. It is built by
`.github/workflows/release-explorer.yml` from `app/` (Info.plist, icon,
`make_dmg.sh`) — tag `explorer-vX.Y.Z` matching `extractor/Cargo.toml` to
release.

**From source:**

```sh
cd explorer/web && npm install && npm run build      # once
cd ../extractor
COHDL_LIB=../../lib cargo run --release -- ../../examples/rpi-pico2 \
    --serve --dist ../web/dist --port 5199
```

Open <http://127.0.0.1:5199/> (the server binds loopback only — it serves
project datasheets and photos, which belong to the local user, not the LAN).
Editing any `.cohdl` in the project re-extracts and refreshes the browser
within ~500 ms; a source that fails to compile keeps the last good view and
reports the diagnostics inline. Add `?mode=sch` for the dark pin-level
schematic view.

Live updates accept a replacement circuit only after a passing check. Both
extraction errors and a returned IR with semantic errors keep the last passing
model and expose current diagnostics in `live_error`. A later valid edit clears
the error and publishes its new circuit. With no previous passing model, an
initial failed IR remains inspectable; an initial extraction error still refuses
startup. Standalone extraction keeps its own result and exit semantics.

The CLI supports `--help`/`-h` and `--version`. Extract a model with
`cohdl-explorer PROJECT [-o FILE]` (`--out FILE` is equivalent); omit the output
option to write JSON to stdout. Serve with
`cohdl-explorer PROJECT --serve [--port N] [--dist DIR]`. Ports must be integers
from 1 through 65535; the default is 5199. Unknown/repeated options, missing
values, multiple project paths and incompatible options exit 2 with an error.
`--port` and `--dist` require `--serve`; `--out` is extraction-only. Use `--`
before a project path beginning with a dash. App-bundle launch with no arguments
still opens the macOS folder picker.

## Project Views And HTTP

`GET /api/project` returns `{ "schema": 1, "id": "<sha256>", "design": "<name>" }`
for project-scoped browser preferences. The ID hashes the canonical project
directory's native path bytes with SHA256; the response does not return the
path. It stays stable across restart at the same directory and separates
same-name projects in different directories. It is a local persistence key,
not authentication or an anonymity guarantee. Moving the project starts a new
namespace. The endpoint is read-only; non-GET methods return404. Model and
standalone extraction bytes are unchanged. If the identity or browser storage
is unavailable, Important marks remain usable for the current session.

`GET /views/<Design>.view.json` first reads the project's
`views/<Design>.view.json`, then falls back to the bundled
`<dist>/views/<Design>.view.json` when no project override exists. A malformed
project override is reported directly. The JSON requires `schema_version: 1`,
a `design` matching the filename, and a nonempty `views` array. Every view has
a unique, nonempty, trimmed name and a `regions` array; each region has a
unique, nonempty, trimmed name within its view and a `members` string array.
`Other` (case-insensitive) is reserved for unmatched parts. Member rules are
trimmed nonempty designators, `path:<instance path>`, `agg:host:<IC path>`, or
`agg:<rail combination key>`; prefix rules require a nonempty suffix. Every
matching region contributes membership; the first match controls placement only.
Empty regions/member arrays are permitted. Optional per-view
`navigation: [{"name":"Subsystem","groups":["Core","Interface"]}]` provides a
two-level index referencing existing region names. Shared leaves can appear under
multiple parents; overlapping selections count each actual instance once.

Fresh sessions open a nonempty source-backed core Group in SCH view. Existing
inspection URLs and Back restore valid choices; legacy overviews offer View
inside. The persistent rail hides/restores Groups and the toolbar and switches
Circuit/BOM. BOM keeps its own search and scroll while consecutive part selections
open matching details. Issues and Proposals are in the secondary menu. Selected
network details precede Connections; Documents/Datasheet follows it. Connection
State describes the original model, independent of temporary tracing.

AI Groups in the official Groups panel reuse [Explorer Assist](assist/README.md).
Review shows actual members and original nets; Focus is temporary, Cancel restores
the preceding range, Save persists a validated display view, and Undo uses the
retained guarded receipt, including after reopening a saved proposal. Editing or
deleting a Group produces another reviewed proposal. Circuit source and BOM are
never modified by display-group actions.

Missing configs return HTTP 404 JSON; invalid JSON/schema returns HTTP 422
JSON with an `error` message. Config additions, content changes and deletions
in project and bundled `views/` directories notify `GET /api/events` through
the existing SSE version events. View-only edits preserve `/api/model` bytes
and never change source, pins or nets. Source edits still re-extract the model;
the watcher does not follow symlink directories.

`GET /api/model` returns the checked model; `GET /api/events` holds the live
SSE stream. `GET /api/photo?mpn=...` serves project photos, and
`GET /api/file?p=<encoded absolute path>` serves supported documents/images
inside the project or the configured CoHDL library checkout. Static paths are
served from `--dist`; only `/` maps to `index.html`. Unknown API/config/static
paths return 404 rather than HTML. Percent-encoded UTF-8 paths are supported;
invalid encoding, traversal and symlinks escaping the declared file root are
rejected. Ordinary HTTP responses use `Cache-Control: no-store` so reloaded
configs cannot retain stale content.

## Document Evidence And Source Excerpts

Project `explorer/documents.json` supplies an optional schema-1 registry:

```json
{"schema":1,"documents":[{"id":"manual","title":"Device manual","kind":"pdf","path":"docs/manual.pdf","revision":null,"sha256":null,"pages":null,"review":"unreviewed","uncertainty":"Original revision has not been established","applicability":[{"instances":["Board::device"],"groups":[],"relation":"exact","page":1}]}],"constraints":[]}
```

Replace the instance with its exact checked path and `sha256` with the actual
lowercase SHA-256 before opening local evidence. `kind` is `pdf` or `markdown`;
Markdown uses `section` citations and null `pages`. Unknown revision, expected
hash, total pages or local original path stays null. Each applicability record
names exact instance paths and/or `{ "view": "Functional", "region": "Core" }`
groups. `relation` is `exact` or `related`; related evidence requires explicit
uncertainty. `review` is `unreviewed`, `reviewed` or `rejected`. Optional
`source_url` must be HTTPS; optional `facts` retains literal string values,
including authored empty strings. Constraints contain `text`, `instances`,
`groups`, `citations` (`document` ID plus optional `page`/`section`) and optional
`uncertainty`. An absent original PDF and local supporting Markdown are separate
records.

```sh
curl http://127.0.0.1:5199/api/documents
curl http://127.0.0.1:5199/api/documents/manual
curl 'http://127.0.0.1:5199/api/source?path=src%2Fmain.cohdl&line=1&count=20'
```

The registry API derives `status`, `actual_sha256`, `issues` and `open_url` from
actual confined bytes. Missing registry returns 404; malformed/unsupported
registry returns 422. Invalid individual records remain inspectable beside
valid evidence and cannot open. Local opening requires matching declared hash,
valid applicability/metadata and accepted file format; unreviewed evidence is
labeled and rejected evidence is disabled. PDF links use `#page=N`, checked
against a known total; unknown totals remain explicit. Local evidence and source
paths cannot escape the project through traversal or symlinks. Document IDs
resolve through the registry rather than accepting arbitrary download paths.

Limits: registry and Markdown 4 MiB, PDF 64 MiB, source 1 MiB, excerpt 64 KiB;
source `line` starts at 1 and `count` is 1–200. Source excerpts include the actual
file SHA-256. Registry/file additions, edits and deletions refresh evidence via
the existing SSE stream without re-extracting or changing the checked circuit.
The inspector exposes citations, literal source facts and constraints for parts
and groups while keeping existing compiler documentation links available.

```sh
node explorer/web/test/browser-smoke.mjs --url http://127.0.0.1:5210 --stage documents
```

## Subdesigns

RFC-032 subdesign use sites appear as logical blocks with their declared
`Pin` ports. Double-click a block, use **Open subdesign** in its inspector,
or choose a path in **Hierarchy** to open it. Breadcrumbs return to enclosing
subdesigns or the board. Nested uses and each array element have their own
stable paths; function-expanded components stay within their owning scope.

Inside a subdesign, boundary ports connect to its real components and nested
blocks. The inspector shows required/optional obligations, outside connection
state, source location and final merged net names. Clicking a net highlights
its visible endpoints and lists its physical pins and subdesign ports. Empty
subdesigns and connections containing only ports remain inspectable.

Search covers full paths, subdesign types and ports, component designators,
MPNs and nets. Finding a hidden part opens its containing subdesign.
**All parts (flattened)** shows every physical component; existing `view.json`
region tabs operate on that view. The same navigation works in overview and
SCH modes. Live reload retains the open path, returning to the board if that
use site is removed.

ExplorerModel v1 gains an additive `subdesigns` array: each entry carries
`path`, `definition`, optional `parent`, use-site `span`, and `ports`
(`name`, `obligation`, optional merged `net`, `connected` outside the node).
Old v1 snapshots without this field still render. Physical `instances`, `nets`
and designators remain separate; port-only net identities are carried by
the port metadata. `derived.fn_groups` contains only actual function groups.

Compact resistors, capacitors and inductors attach wires to their logical
terminals in both overview and SCH modes. Branches on the same terminal
share one attachment; a zero-ohm resistor still separates its two declared
nets. Hover a terminal to see its pin number and net.

## Physical layout

Choose **Layout** (or open `?mode=layout`) to inspect the authored physical
placement in millimetres. The view draws the referenced DXF board perimeter,
footprint pads, holes, courtyards and cutouts at the compiler's resolved
coordinates. Opening a subdesign defaults to **Subdesign local** coordinates:
its authored placements are visible even before the parent anchors it on the
board. Nested placements compose in the selected subdesign's frame. Switch
**Coordinates** to **Board coordinates** to see the final whole-subdesign
transforms and explicit child overrides. The board outline appears only in
the board frame. Top and bottom components use different colours;
both are viewed from above, so bottom components mirror local X before
rotation. The **Components** filter selects top, bottom or both.

Click a component to inspect exact X/Y, rotation, side and placement hints.
Search locates placed components; **Show in layout** links the schematic's
part inspector to this view. Pan, zoom or **Fit board** to inspect the board,
and ⌘/Ctrl-click two components to measure the distance between their
placement origins. Selecting a net highlights its pads. **Layout constraints**
lists net classes, differential pairs and length-match groups.

Parts with no `place` coordinate in the selected frame are listed under
**Without local/board placement**. Unanchored subdesign defaults are visible
in their local view and remain unplaced on the board. An empty board view
links to available local layouts. A placed part with no footprint geometry
shows an origin marker.
This is a read-only placement viewer: the drawing contains no routing or
clearance verdict. Edit the source to move components; source and DXF edits
reload the view. A missing/invalid outline is reported within the layout view
while the schematic remains available.

ExplorerModel v1 adds nullable `layout`, based on the compiler's `layout.json`
projection. Placements also carry canonical `at_mm` strings and SVG `matrix`
coefficients derived from `cohdl::trig`'s fixed-point table. No placement
rotation uses platform sine/cosine. The optional `outline_error` describes an
outline display failure without changing the check verdict. Old snapshots
without layout metadata offer a re-extraction prompt. Subdesign entries carry
additive `local_placements` in the same placement shape; these are retained
tooling metadata and never enter board outputs or designator allocation.

## Develop (frontend hot-reload)

Project group checkboxes display their union; no checked groups means the
whole circuit. Choosing a functional group opens the flattened part view,
so components inside subdesigns retain their actual group identity. Rail
aggregates split at group boundaries. Shown counts use represented source
instances and unique incident model nets, including rails.
Changing groups, view or hierarchy starts a fresh fitted inspection and clears
temporary part/pin/net selection, search and details. Back explicitly restores
the preceding inspection. Important marks persist independently of selection.

The net inspector lists every logical endpoint and all its physical numbers,
including endpoints outside the current groups. Following an endpoint keeps
the net selected; Back restores the preceding group, selection and camera
after layout completes. Pan and zoom (including zoom buttons) preserve the
camera on resize; Fit resumes automatic fitting. The physical Layout uses its
own millimetre camera for the same Back behavior. Clear filters restores the
complete circuit, including from an empty group.
Physical endpoint visibility also follows the Components side filter. Following
a hidden endpoint reveals its side; Back restores the preceding side and camera.

Group disclosures collapse real members into logical boundary blocks. Bulk
collapse/expand commands keep the represented part/net counts unchanged.
Fully collapsed Overview uses compact module packing with bounded orthogonal
corridors. Phone overview fits two columns and pans vertically. Detailed and SCH
layouts retain native layered placement; exact endpoints remain inspectable.
Overview uses compact boundary blocks; SCH and the selected block inspector
retain each boundary's actual net, logical pin and physical numbers. Internal
nets stay inside the group. No placement or circuit connection is synthesized.

Neutral connections bundle distinct real nets between the same displayed
objects. Multi-object networks use a shared bus with spokes, preserving their
full memberships. Click the path or count to inspect the network list, then
choose a net for its exact trace. Rails, hidden or internal members, single-ended
nets, NC and open pins retain separate explanations. Whole left/right pin rows
and inspector rows support pointer, Enter and Space activation.

Component details show source-backed identity, footprint and connections before
supporting datasheets; an unspecified MPN remains unspecified. Important marks
use bounded local browser storage scoped to canonical project identity and design.
Storage failure leaves session marks usable. Locate and unmark are available
from the Important list, and marking does not dim the other components.

Search includes logical pins and physical numbers, including alphanumeric pads
and qualified queries such as `U1.1` or `U1.A1`. A located part/pin reveals its
group and expands its block. Explicit part/pin/BOM/endpoint visits focus the completed native target;
net visits frame actual represented endpoint bounds, including native placed
footprints in Layout. Located cameras remain stable through resize, reload and
history; explicit Fit returns to whole-circuit fitting. NC/open pins retain their state without an invented
net. Circuit, BOM and Issues tabs share the same locate path; Issues includes
the original live-source error even when the last good model has no diagnostics.
An authored diagnostic span locates a matching part/net/subdesign; other source
locations remain explicit and focused, without selecting an unrelated part.

Inspection links encode validated context in the `inspect` query parameter.
Mode, hierarchy scope (root and flattened are distinct), view, groups, collapsed
blocks, selection, pin, net, tab, coordinate frame, side and native camera are
restored on reload and browser Back/Forward. Each mode/scope/view keeps a separate
session snapshot. Manual cameras take precedence over automatic fitting;
obsolete asynchronous layouts cannot restore another context's camera.

Run the desktop/mobile collapse, unified search, BOM/Issues, URL/history, native
camera and live-refresh regression:

```sh
node explorer/web/test/browser-smoke.mjs --url http://127.0.0.1:5210 --stage navigation
```

Run every browser stage in a separate browser/storage context:

```sh
node explorer/web/test/browser-smoke.mjs --url http://127.0.0.1:5210 --stage all
```

The aggregate stops on a failing stage and retains each stage's command, time,
exit, logs and screenshots under the supplied `--output` directory.

The navigation regression first exercises the real served circuit, then uses
declared synthetic hierarchy/diagnostic/placement fixtures for states absent
from PDFExact. A temporary local HTTP/SSE fixture uses the production frontend
and real EventSource requests to test model refresh; it closes after the check.

Run the desktop/mobile union, endpoint, camera and empty-recovery regression:

```sh
node explorer/web/test/browser-smoke.mjs --url http://127.0.0.1:5210 --stage union
```

The browser camera regression explicitly supplies two test-only placements on opposite sides
because the PDFExact snapshot contains no placement facts. It does not write
those coordinates to the model service or circuit source.

For work on the frontend itself, run Vite's dev server alongside the
extractor instead of rebuilding `dist`:

```sh
cd explorer/web && npm run dev        # http://localhost:5198/
```

It proxies `/api` (model, SSE, photos, files) to the extractor on 5199, so
both live loops compose: a `.cohdl` edit re-extracts and refreshes the view,
a `.tsx` edit hot-swaps modules in place without losing UI state.

Run `cargo test` in `extractor/` and `npm test && npm run build` in `web/`.
The dependency-free project at `extractor/tests/fixtures/subdesign/` exercises
nested generic arrays, function expansion, optional ports, empty subdesigns
and port-only connections; it can also be opened with `--serve` for UI checks.

## What is deterministic, what is AI

Extraction, display rules, layout and wire routing are ordinary code:
the same source always yields the same drawing. Project partition labels in
`views/<Design>.view.json` control views and group membership; changing them
does not change circuit topology. The existing `skills/view-gen/SKILL.md`
describes generating these configurations.

[Explorer Assist](assist/README.md) also provides source-cited AI grouping,
read-only interface planning and connection proposals through its CLI. It
retains actual provider output, exact diffs and compiler validation. The
Proposals tab displays those records; applying or reverting a checked change
is an explicit CLI operation. The browser remains a read-only circuit viewer.

## Layout

| Path | What |
| --- | --- |
| `extractor/` | Rust crate: calls the compiler pipeline, emits ExplorerModel JSON v1, serves it over HTTP + SSE |
| `web/` | TypeScript/React frontend: display-rule engine, ELK layout, custom node and wire renderers |
| `views/` | Per-design partition configs (AI-generated, validated against the model) |
| `skills/view-gen/` | The skill an agent follows to write a `view.json` |

`extractor` is a standalone crate with its own `Cargo.lock`. It depends on
`cohdl` by path and on serde for JSON, so it deliberately lives outside the
compiler crate, whose zero-dependency rule it does not share — the same
arrangement as `registry/` and `editors/vscode/`.
