# CoHDL Schematic Explorer

A read-only, interactive projection of a checked CoHDL design: every
instance, net and pin the compiler resolved, drawn as a schematic-style
board you can search, trace and inspect. It never edits a circuit — the
`.cohdl` source stays the single source of truth.

## Run

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

## Develop (frontend hot-reload)

For work on the frontend itself, run Vite's dev server alongside the
extractor instead of rebuilding `dist`:

```sh
cd explorer/web && npm run dev        # http://localhost:5198/
```

It proxies `/api` (model, SSE, photos, files) to the extractor on 5199, so
both live loops compose: a `.cohdl` edit re-extracts and refreshes the view,
a `.tsx` edit hot-swaps modules in place without losing UI state.

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
