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
the same source always yields the same drawing. AI only writes the
partition labels — `views/<Design>.view.json`, generated with
`skills/view-gen/SKILL.md`, which decides the page tabs and the region
each part belongs to. A bad view file changes grouping, never topology.

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
