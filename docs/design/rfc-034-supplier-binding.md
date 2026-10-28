# RFC-034: Supplier bindings with placement mapping

> Draft. Number provisional pending central registry allocation; error codes provisional pending central allocation. Baseline: main `329af05` (2026-10-21). Builds on the provisional `part`/AVL syntax (`docs/provisional-syntax.md` §2), RFC-001 (units), RFC-020/025/026 (placement, rotation, back side), RFC-029/030 (versioned packages, registry.cohdl.org) and the registry search amendment to RFC-030/DR-036. Also composes with two repository extensions that have no RFC (commit `f1f7acc`): the `#[source_unspecified("mpn"|"mfr")]` part marker, which lets a primary entry carry an explicitly empty `mpn` or `mfr`, and `cohdl build --logical-only`, which writes logical artifacts and no physical ones.

## Problem

A CoHDL part names what to buy by manufacturer and MPN. Assembly houses and distributors order and place by their own part numbers and their own footprint frames, and CoHDL has no way to say either.

- **No supplier part number.** An AVL entry carries `mfr`, `mpn` and `footprint` only (`src/check/generics.rs` rejects other keys with E802). One MPN can map to several part numbers even at a single supplier; the research behind this RFC found an RF connector MPN listed under two numbers by the same distributor. So a BOM (`src/emit/bom.rs`) cannot be uploaded to an assembler without someone matching parts by hand. The existing `.enet` netlist emitter leaves out the format's supplier properties because nothing could fill them (recorded in `docs/compliance-report.md`).
- **No placement mapping.** An assembler places each part using its own library footprint, whose origin and 0° orientation often differ from the design's footprint. A CPL that reports the design's own position and rotation then puts parts off-centre or turned. `tools/smt_pos.py` explicitly leaves this to "the fab's own correction table", which lives outside the design and so is never versioned with it.
  - Measured on the official library against one assembler's library, 14 of 33 parts used a different 0° orientation.
  - The offset and rotation belong to the pair (this footprint, that supplier's footprint for that supplier part number), so they cannot be a global table.
- **No way to see where a part can be bought.** registry.cohdl.org shows a part's MPN but cannot link to a supplier page or show whether the supplier stocks it.

Who this is for:
- AI authors choosing parts that can actually be ordered.
- Reviewers who want one click from a part to its supplier page.
- Anyone sending a design to an assembler, who needs a BOM and CPL that are correct for that assembler without hand edits.

## Goals

- **Bind suppliers to AVL entries.** Let any AVL entry name, for any number of suppliers, the supplier's part number and the transform from the entry's footprint to that supplier's footprint (offset and rotation).
- **Keep the compiler supplier-neutral.** It checks structure and units only and knows no particular supplier. Supplier-specific knowledge (part-number formats, product URLs, catalog classes) lives in registry data and tools.
- **Emit placement and per-supplier outputs.** `cohdl build` emits a supplier-neutral placement file (CPL) for any placed design. For each chosen supplier it also emits a BOM keyed by that supplier's part numbers, a placement map, and that supplier's CPL, all transformed by one defined composition rule.
- **Show suppliers in the registry.** For each part, the registry shows its suppliers, part numbers, links and, where a catalog snapshot exists, availability. It can search by supplier and by a supplier-defined class, and audit a project's bindings online.
- **Serve Constitution ranks** 1 (correctness/gradeability), 2 (AI-generatability) and 5 (faithful output).

## Non-goals

- **Any particular supplier in the language.** No supplier name, number format or URL is compiled into `cohdl`.
- **Live supplier data in packages.** Stock, price and class change daily. They live in the registry with an as-of date, never in `.cohdl` source.
- **Supplier geometry.** A binding records only how the supplier's footprint sits relative to the design's own footprint. The design's footprint geometry stays authored from manufacturer documents (`LIBRARY.md`).
- **Footprint-level default mappings.** Bindings are per AVL entry and supplier. Sharing one mapping across all parts with the same footprint is deferred (see Alternatives).
- **Automatic part selection, substitution or editing of source.**
- **Ordering or uploading to suppliers.**
- **The official library's supplier program**: backfilling supplier part numbers into existing official parts, aligning official footprints so a chosen supplier's mapping is (0, 0, 0°), and catalog-driven intake. That program uses this RFC's mechanism and is maintained in the library plan and `LIBRARY.md`, outside the language design.

## Design

### 1. The supplier clause

An AVL entry (`primary { … }` or `alt { … }`) may contain zero or more supplier clauses, comma-separated like its other items:

```text
supplier SUPPLIER "PART_NUMBER" [at (X, Y)] [rotate ANGLE]
```

```cohdl
pub part RP2350A_QFN60: RP2350A {
    primary { mfr: "Raspberry Pi", mpn: "RP2350A", footprint: qfn::QFN60N40P700X700_1EP340X340,
              supplier acme "A-104411" }
}

// Only the Samsung alternate is bound, so a supplier BOM for acme orders
// it; it carries its own (here identity) mapping.
pub part C_100n_16V_X7R_0402: MLCC<100nF, 16V, 10%>[C0402] {
    primary { mfr: "Yageo", mpn: "CC0402KRX7R7BB104", footprint: CHIP_0402 }
    alt { mfr: "Samsung", mpn: "CL05B104KO5NNNC", supplier acme "A-200157" }
    alt { mfr: "Murata", mpn: "GRM155R71C104KA88D" }
}

// Mapping values illustrative: the supplier's footprint for this part is
// turned half a revolution and its origin sits 1.2 mm off ours.
pub part CONN_BAT_2P: BAT_CONN_2P {
    primary { mfr: "JST", mpn: "S2B-XH-A(LF)(SN)", footprint: FP_BAT_2P_2P5,
              supplier acme "A-300931" at (0mm, -1.2mm) rotate 180 }
}

// Several suppliers on one entry, each with its own part number and
// mapping. `--supplier acme` and `--supplier globex` produce different
// BOMs and CPLs for the same board.
pub part USB_C_HRO_TYPE_C_31_M_12: USB_C_Receptacle_2_0 {
    primary { mfr: "HRO", mpn: "TYPE-C-31-M-12", footprint: FP_USB_C_Receptacle_HRO_TYPE_C_31_M_12,
              supplier acme "A-165948" at (0mm, 0.5mm) rotate 180,
              supplier globex "GX-31M12-T" rotate 90 }
}
```

Rules:

- **`SUPPLIER`** is an identifier: lowercase ASCII letter first, then lowercase letters, digits or `_`, at most 32 bytes. The set is **open**: the compiler attaches no meaning to any particular name. Names in conventional use are recorded in the registry's supplier table (§4), not in the compiler. `acme`, `globex` and their part numbers in this RFC are fictitious placeholders. The IDs `mpn` and `mfr` are reserved (E813): their upper-case forms would collide with the `MPN`/`MFR` fields of the KiCad netlist (§3.3).
- **`PART_NUMBER`** is a string: non-empty after trimming, at most 128 UTF-8 bytes, no control characters. The compiler checks no supplier-specific format.
- **`at (X, Y)`** is the mapping offset, two `Length` values (RFC-001; a bare number is the existing unit error). It may be negative. It defaults to `(0mm, 0mm)`.
- **`rotate ANGLE`** is the mapping rotation, a whole number of degrees in `0..=359`. This is the same range and diagnostic shape as `place … rotate` and pad rotation under the live arbitrary-angle deviation. It defaults to `0`.
- **Syntax reuse.** `at (…)` and `rotate …` are the existing placement clauses, reused with the meaning defined in §2. `supplier` is recognized only at the start of an item inside an AVL entry, where it is followed by an identifier and a string, never by `:`. So it is a contextual word, not a new reserved keyword; an existing identifier named `supplier` keeps working everywhere else.
- **Uniqueness:**
  - One entry may not name the same supplier twice.
  - Across the whole compiled package set (every `part`, primary and alternates, as the existing cross-part `(mfr, mpn)` identity check in `src/check/generics.rs` already scans), the same (supplier, part number), with the part number compared exactly as written (no trimming or case folding), may not appear on two entries whose (`mfr`, `mpn`) differ, because a supplier part number orders one item. An empty, source-unspecified `mpn` counts as differing from every non-empty one: sharing a part number with another entry would silently assert the unknown MPN.
  - Both are **E813**.
- **Limit.** At most 16 supplier clauses per AVL entry (E813). The registry index applies the same bound.
- **Unknown MPN.** A primary entry whose `mpn` or `mfr` is explicitly empty under `#[source_unspecified]` may carry supplier clauses like any other entry. For such a part the supplier part number may be the only orderable identity, and the binding records it without guessing the MPN. The marker's own rules are unchanged; a supplier clause never satisfies or replaces it.
- **Optional.** A part with no supplier clause is valid, and `check`/`build` without supplier output behave exactly as today.

Diagnostics:
- **Syntax** (a missing supplier identifier or part-number string, a stray token) is the parser's **E010**, like every other syntax error.
- **E813** owns the clause's semantics, naming the part, the entry and the supplier: an empty, over-long or control-character part number, an over-long supplier identifier, a duplicate supplier, a conflicting part number, more than 16 clauses, an out-of-range angle, and a non-`Length` value in `at` (a bare number or another unit). Each construct already owns its own unit errors (E805/E806 for pads, E1007 for `place`); E813 does the same here.
- Lexical unit errors (E101, E103, E104) apply as everywhere.
- Unknown `name:` keys remain E802.

### 2. Meaning of the mapping

A binding's mapping says where the supplier's footprint for that part number sits when drawn in the coordinate frame of the entry's footprint:

- `at (dx, dy)` is the position of the supplier footprint's origin, in the entry footprint's local frame.
- `rotate r` is the angle of the supplier footprint's 0° axis, in the same frame and with the same angular sense as pad rotation (RFC-025).

So a local point `q'` of the supplier footprint corresponds to the local point `q = d + Rot(r)·q'` of the entry's footprint, with `d = (dx, dy)`.

**Normative rule.** The supplier placement of an instance is

```text
supplier_place = compose_place(instance_place, PlaceData { at: d, rotate: r, side: Top })
```

where `compose_place` is the function that already composes a subdesign's internal placement with its use site (`src/check/expand.rs`). It moves to a shared module so that CPL emission and subdesign composition use one implementation and cannot drift. It covers both sides: on the bottom (RFC-026) it mirrors the offset's x, then rotates, and reverses the mapping rotation, exactly as for a placement inside a subdesign placed on the bottom. All arithmetic is in `src/trig.rs`, never platform floating-point trigonometry, so output bytes stay stable.

**Informal reading, top side.** For an instance placed at `P` with rotation `θ`:

```text
P_s = P + Rot(θ) · d
θ_s = (θ + r) mod 360
```

`P`, `d` and `Rot` are in the authoring frame (+y down), and positive angles turn counter-clockwise as seen from above, the sense of `place … rotate`. In that frame `Rot(θ)` is `trig::rotate(·, (360 − θ) mod 360)`, because `trig::rotate` is counter-clockwise in a +y-up frame. This is why the reference is `compose_place`, not the formula.

A mapping of `(0, 0, 0°)` means the supplier's footprint frame is identical to the entry's.

### 3. Build outputs

The CPL files below are **physical** outputs, like `.kicad_mod` and IPC-2581. They are written only by a physical build, after the existing physical-export gate (`emit::validate_physical_export`) has passed, so a design that gate rejects never gets a CPL. Under `cohdl build --logical-only` no CPL is written and E815 is not reported. Supplier BOMs and map files are procurement data, like the plain BOM, and are written in both modes.

**Ownership.** Every new file is written through the existing `write_artifact` path and recorded in `out/.cohdl-manifest`, like every other artifact. So:
- a pre-existing file with the same name that CoHDL did not write fails the build, as it does today;
- a file this build no longer produces is removed by the stale sweep, for example `-acme-*` files after a build without `--supplier acme`, or the CPL after the last `place` is deleted. No stale procurement file survives.

`build --json`'s `build` object gains the additive keys `cpl` (path) and `suppliers` (`{ "<ID>": { bom, map, cpl? } }`), present only when those files are written.

#### 3.1 Neutral placement file (no supplier)

`cohdl build` writes `out/<name>-cpl.csv` whenever the design places at least one instance with `place` (RFC-020/025/026, including placements composed through `subdesign`). This file is supplier-neutral: it reports the design's own footprint frames, with no mapping applied.

```text
Designator,Mid X,Mid Y,Layer,Rotation
```

- **Rows:** one per **machine-placed instance**: an instance that has a row in the BOM and whose footprint has at least one numbered `smd` pad in its KiCad pad projection (§3.2, map file). Through-hole-only and pad-less instances are not rows.
  - The test is made on the projection, not on the source pads, so that `tools/smt_pos.py --neutral`, which sees only the board, reaches the same answer. The two differ in two places: an annulus pad is always written as an SMD pad, and a paste aperture is an unnumbered SMD pad that does not count, even on a through-hole part with paste for pin-in-paste.
  - There are no exclusions by value or designator. A part that is bought is placed.
  - `tools/smt_pos.py`'s existing default output also drops `TouchSensor`/`TouchPad` values and `TP…` designators, by heuristic. Only that legacy output keeps those exclusions; `--neutral` applies this rule exactly.
  - A way to mark an instance as bought but not assembled is out of scope for this RFC.
  - Conformance tests cover an SMD part, a connector with both SMD and through-hole pads, a test point, and a bare-copper part.
- **Origin:** when the design declares an RFC-020 board outline, the lower-left corner of the outline's bounding box, with Y positive upward. This is the origin `tools/smt_pos.py` already uses. Without an outline, the authoring origin, with y negated.
- **Position:** `Mid X`/`Mid Y` are the **footprint origin** in millimetres, written with exactly four decimal places, rounded half away from zero, and an `mm` suffix (`12.3450mm`). They are not a pad centroid: the mapping in §2 is defined between footprint origins.
- **Layer:** `Top` or `Bottom`.
- **Rotation**, in `0..=359`:
  - Top side: the `place` rotation.
  - Bottom side: the footprint angle KiCad uses for that part, `(R + 180) mod 360` for `place … rotate R side bottom`. This is the value `--emit kicad_pcb` writes and the value a position export from that board reports. One convention keeps the compiler's CPL and a CPL derived from the board identical. Conformance tests pin it against `tools/smt_pos.py` output on the emitted board, top and bottom.
- **Unplaced instances:** machine-placed instances with no placement are left out, and the build reports **one E815** warning that gives their count and lists their designators. A partly placed design, which is common, gets one line, not one per part. A position the compiler only staged (as `--emit kicad_pcb` does for unplaced parts) is never reported.
- **Ordering:** rows are in designator natural order, and the output is byte-stable.
- A design with no `place` statement gets no CPL file and no warning. A design that places at least one instance gets the file even when it has no rows (header only), for example when every placed instance is through-hole.
- **Unplaced** means the instance has no position in the design's own layout (`place`, including placements composed through `subdesign`). Staging positions an emitter invents are not placements.

When final placement happens outside CoHDL, the CPL comes from the laid-out board instead. `tools/smt_pos.py` gains `--neutral`, which writes exactly these columns with this origin, position and rotation convention. Its existing default output, the 11-column file with a pad-centroid `Mid X/Y` and `T`/`B` layers, is unchanged for current users.

#### 3.2 Per-supplier outputs (`--supplier ID`)

`cohdl build --supplier ID` (repeatable, one set of files per ID) writes:

| File | Content |
|---|---|
| `out/<name>-<ID>-bom.csv` | `Comment,Designator,Footprint,<ID> Part #`, where the last header is the supplier ID in upper case (for `acme`: `ACME Part #`). One row per selected purchase identity (below). |
| `out/<name>-<ID>-map.json` | `schema_version: 1`. A `footprints` table and a `designators` table (described after this table). |
| `out/<name>-<ID>-cpl.csv` | Written when the neutral CPL (§3.1) is, so never under `--logical-only`. Same columns, rows, origin and bottom-side angle convention as the neutral file. Each row's placement is first transformed by §2 using that designator's mapping. |

The neutral `-cpl.csv` is still written alongside; `--supplier` adds files and never replaces one. `--supplier` combines with `--logical-only` (it is not an `--emit` value) and then writes the BOM and map files only.

**Map file.** The map identifies the footprint frame each mapping was written against, so a tool applying it to a laid-out board can tell whether that board still uses that frame.
- `footprints`: one entry per footprint of an instance listed in `designators`, keyed by its fully-qualified name. Each entry holds:
  - the owning package and version;
  - `pads`: the footprint's **KiCad pad projection**, as `[number, x, y]` per pad, canonical mm strings, sorted by number, then x, then y. The order is for canonical serialization and the `frame` hash only; the frame check below does not pair pads by it;
  - `frame`: the SHA-256 of that `pads` list in its canonical JSON form.

  The map also carries a top-level `pad_projection: 1`, the version of the projection rule below.

**Pad projection.** The `pads` list records the pads exactly as the board sees them, because that is what a tool reading a laid-out board can compare against. It is taken from the derivation `.kicad_mod` and `--emit kicad_pcb` already share (`kicad_mod::pad_plans`), so the map and the board cannot drift apart. The source pad list is not used: the projection differs from it in ways that would fail correctly exported boards.
- **Only numbered pads are listed.** The projection also writes unnumbered pads: paste-override apertures (RFC-018, including each sector of a segmented annulus) and mount holes (RFC-022/023). They carry no electrical identity and are left out on both sides of the comparison.
- **The position is the KiCad anchor**, in the footprint's front-side local frame (+y down). For most pads it is the authored centre. An annulus pad's anchor is moved onto its copper ring, `mid_radius` along the pad's own +x axis, and the map records that anchor, not the ring centre.
- **Repeated pad numbers** (several lands for one pin) each appear once, so the list is a multiset.
- A change to how `pad_plans` places anchors or which pads it numbers changes every affected map and board. It increments `pad_projection` and is listed in the compiler's release notes.
- `designators`: one entry per machine-placed instance, holding:
  - `part` and `footprint` (fully-qualified names);
  - `entry` (`primary` or the alt's index) and that entry's `mfr` and `mpn`;
  - `pn`, `at` (canonical mm strings) and `rotate`.

  An instance with no eligible binding is present with `"pn": null`. That records "explicitly unbound", which is different from "not in this map".

How the entry and rows are chosen:

- **Eligibility.** Every emitter draws the primary's footprint, and a mapping is relative to its own entry's footprint. So an entry is eligible only when its effective footprint (its own `footprint:`, else the primary's, the rule the cross-part identity check already uses) is the primary's footprint. A bound `alt` with a different footprint is skipped, and E814 says why when that leaves the group unbound. Its binding still appears in API docs and the registry.
- **Entry selection is per instance**, from the part that instance is bound to: the primary if it has a binding for `ID`, otherwise the first eligible `alt` in declaration order that has one.
  - Two part declarations may share a primary MPN but allow different alternates, part numbers or mappings. The existing identity check does not require their AVLs to match. Choosing once per plain-BOM group could then violate one declaration's constraints, so selection never happens at group level.
  - The mapping that is applied is the **selected entry's**, because an alternate's supplier footprint can differ from the primary's.
- **Supplier BOM rows** merge instances by their selected purchase identity: (part number, selected entry's `mfr`, selected entry's `mpn`). `Comment` is the selected entry's `mpn`. It is empty for a source-unspecified MPN, and the `<ID> Part #` cell then carries the part's identity.
  - Unbound instances are merged by the plain BOM's key instead: primary MPN and manufacturer, plus the bound part declaration when the MPN is empty.
  - The plain `-bom.csv` keeps its existing grouping.
- **The map and supplier CPL are per designator** and always carry that instance's own selection and mapping.
- **Unbound instances** get an empty part-number cell, identity mapping in the supplier CPL, and one **E814** warning per unbound part declaration, naming the part and its designators. Only parts the design instantiates are considered; a declared but unused part never produces E814. It appears only when `--supplier` is given, in human and RFC-010 `--json` output alike. An unbound part is still a correct design, so E814 never fails a build. If no entry of any part the design instantiates binds `ID`, which is usually a misspelled ID, the per-group warnings are replaced by one E814 that says so and lists the supplier IDs the design does bind.
- **Ordering:** BOM rows are sorted by part number, then MPN, with designators in natural order. Output is byte-stable.

**Laid-out boards.** `tools/smt_pos.py` gains `--map out/<name>-<ID>-map.json`. With a map, it applies §2 to each footprint of the laid-out board and writes the supplier CPL in the neutral column set. `--neutral` without a map writes the neutral file. `--map` implies the neutral columns. `--all` cannot be combined with `--neutral` or `--map` (usage error, exit 2). A board with no outline uses the board origin with y negated, which for a board written by `--emit kicad_pcb` is the authoring origin of §3.1. Before writing anything, it checks the board against the map and fails closed:
- **Frame check.** For each footprint, it applies the same filter as the projection: numbered pads only, so paste apertures and holes drop out. It then recovers each pad's anchor in the footprint's front-side local frame:
  - subtract the footprint position and undo the footprint orientation, which gives the pad's footprint-relative position as the board stores it;
  - on the bottom side, negate local y. This inverts the bottom-side encoding `--emit kicad_pcb` writes, which is also how pcbnew stores a footprint flipped in either flip direction (the direction only changes the footprint angle).

  The board's pads and the map's `pads` are then compared as **multisets within a tolerance**:
  - Both sides are grouped by pad number, and each number must have the same count on both sides.
  - Within each group there must be a one-to-one pairing of board pads with map pads in which every pair differs by at most 0.001 mm in x and in y. No pad may be used in two pairs.
  - Pairing must not rely on sort order. Two pads with one number and the same x can swap order under sub-tolerance rounding (x = +0.0000004 against −0.0000004), and an element-by-element comparison of sorted lists would then report a 2 mm error that is not there. The tool finds a perfect bipartite matching per group (for example Hopcroft–Karp; groups are small). A greedy pairing is not enough, because it can also reject a valid pairing.

  Moving, rotating or flipping a part on the board does not affect the check. A footprint whose frame changed, for example an older library version still on the board, fails it. A mismatch is an error naming the designator and the lowest failing pad number, with that group's board and map positions in sorted order, so the message is deterministic. Because a board exported under an older `pad_projection` fails the same way, the message names both causes.
- **Projection tests.** One fixture footprint set covers an annulus pad, a segmented annulus paste, rect and circle paste overrides, plated and non-plated mount holes, and repeated pad numbers, including two pads with one number, the same x and different y. Each is placed on the top and bottom sides at a cardinal and a non-cardinal rotation on a board written by `--emit kicad_pcb`. The check must pass on that board, on the same board after a pcbnew open-and-save, and with one footprint flipped by hand in pcbnew in each flip direction. It must fail when one pad is moved by 0.01 mm. A unit test of the matcher pairs the repeated-number case with x errors of opposite sign below the tolerance, and with board coordinates rounded to 1 nm after a 1° rotation, and must accept both.
- **Coverage.** A machine-placed footprint on the board with no `designators` entry is an error ("not in this map"). An entry with `"pn": null` is reported as unbound and placed with identity mapping. A map designator missing from the board is a warning.
- Any error means no output file is written.

#### 3.3 Other outputs

- `-bom.csv`: unchanged, byte for byte.
- KiCad `.net`: for each binding on the **primary** entry, adds `(field (name "<ID upper case>") "<pn>")` beside `MPN`/`MFR`, in supplier order. Instances whose primary has no binding emit the same bytes as today.
- `.enet` netlist: only when `--supplier ID` is given, and only for instances with a selected entry:
  - It adds `Supplier: "<ID upper case>"` and `Supplier Part: "<pn>"`.
  - It takes `Manufacturer` and `Manufacturer Part` from the **same selected entry**, so one component never carries a primary's manufacturer identity next to an alternate's supplier part number. Today the `.enet` emitter reads those two fields from the primary.
  - Unbound instances and builds without `--supplier` are unchanged.
  - With several `--supplier` values, the first one given is used.
  - A test with only an alternate bound (the C_100n_16V_X7R_0402 example in §1) pins the BOM, map and `.enet` to the same selected entry.
- API docs (`cohdl docs`): each AVL entry gains an additive `suppliers` array, `[{ supplier, pn, at: [x, y], rotate }]` with canonical mm strings, in declaration order. Every key is always present; defaults are written out (`["0mm", "0mm"]`, `0`). `schema_version` is not bumped: the key is additive under both version 1 and version 2 (the RFC-033 shape), and existing keys are unchanged.

### 4. Registry

**Supplier table.** A new D1 table `suppliers` (migration `0005`) has one row per known supplier ID:
- display name;
- product-URL template with a `{pn}` placeholder, for example `https://parts.example.com/{pn}`;
- the class vocabulary the supplier's catalog uses, if any.

It is maintained by official accounts. A binding to an unlisted supplier is shown without links.

**Manufacturer aliases.** A table `manufacturer_aliases` (same migration) maps an alias to a canonical manufacturer name. Official accounts maintain it through the admin API. §5 compares manufacturer names after Unicode case folding, collapsing runs of whitespace, and mapping both sides through this table. Normalization and alias mapping both happen in the registry, so every client compares the same keys (below).

**Catalog table.** A new table `supplier_catalog` is keyed by `(supplier, pn)`. Its columns are `mpn`, `manufacturer`, `package`, `class`, `stock`, extra `links` (label + URL, for supplier pages that cannot be derived from the template), and `fetched_at`.
- It is replaced per supplier from a snapshot uploaded through an admin endpoint. The Worker does not fetch supplier sites.
- What a snapshot contains and how it is gathered is outside this RFC.

**Index.** `registry/src/worker/search.ts` reads the API-docs `suppliers` array into the part index next to `mfr`/`mpn`, bounded like the other projections (a new `MAX_SUPPLIERS_PER_AVL = 16`, beside `MAX_AVL_ENTRIES_PER_PART`). Part numbers also join the searchable text, so searching for a part number finds the part.

**Public surface** (additive; all read-only and login-free like `/search`):

- `GET /search` part rows gain `suppliers: [{ supplier, pn, at, rotate, links, catalog|null }]`.
  - `catalog` carries `class`, `stock` and the snapshot date.
  - New optional parameters: `supplier=ID`, and `class=…` (repeatable, requires `supplier`).
  - A part matches when it has a binding for that supplier whose catalog row has one of the classes. Without these parameters, results are unchanged.
- New `GET /suppliers/{id}/parts`, with two forms (at most 100 values per request; every response carries the snapshot date):
  - `?pn=P1,P2,…`: each part number's catalog row, or `unknown`.
  - `?mpn=M1,M2,…`: for each MPN, the catalog rows whose MPN matches after normalization (case and whitespace), ranked by class order (as declared in the supplier table), then in stock, then higher stock.
  - Each catalog row also carries `mpn_key` and `manufacturer_key`: the normalized MPN, and the normalized manufacturer after alias mapping.
  - Values are matched as given. A value may repeat, and each occurrence gets its own result in input order. A part number is looked up exactly, so one that is not trimmed or not in NFC simply returns `unknown`. Empty values, control characters, values over 256 UTF-8 bytes and more than 100 values are rejected. Because the list is comma-separated, a value containing `,` cannot be queried.
- New `GET /manufacturers/normalize?mfr=…&mpn=…` (each parameter repeatable, at most 100 of each, same value rules): returns the registry's key for each value, in input order. Clients never normalize names themselves.
- `cohdl search` prints bindings, gains `--supplier ID` and `--class C` (repeatable), and `--json` passes `suppliers` through.
- The registry UI part view lists each supplier with its part number, mapping, links, class and snapshot date, and offers supplier and class filters on search.

### 5. `cohdl suppliers`: online audit and suggestions

`cohdl suppliers <dir> [--supplier ID]... [--json]` resolves the project offline exactly as `check` does, then queries the registry. A project that fails `check` exits 1 before any query, like `check`.

- **For each binding** (optionally limited to the given suppliers), it reports one of:
  - `ok`
  - `unknown`: part number not in that supplier's snapshot
  - `mismatch`: catalog MPN or manufacturer differs from the entry after normalization; manufacturer names are compared through a registry-maintained alias table. A source-unspecified (empty) field is never compared, so it cannot cause a `mismatch`; the report shows the catalog's value for that field as information only and never proposes writing it into source.
  - `out-of-stock`
- **For each AVL entry with no binding** for a requested supplier, it reports `unbound` with **suggestions**: that supplier's catalog rows matching the entry's `mpn`, ranked as in `/suppliers/{id}/parts?mpn=`. An entry with an empty `mpn` has nothing to match and is reported `unbound` with no suggestions.
  - Each suggestion shows part number, class, stock and catalog manufacturer, and whether the manufacturer agrees with the entry's `mfr`. Agreeing suggestions rank first.
  - Suggestions never include a mapping: the registry cannot know how the supplier's footprint sits relative to the design's.
  - The command never edits source. `--json` carries suggestions as structured data for a repair loop.
- **Scope.** The audit covers every AVL entry, primary and alternates, of the parts the design instantiates, as E814 does. Without `--supplier` it audits every supplier ID those entries bind and reports no `unbound`.
- **Comparison** uses the registry's keys (§4): the catalog row's `mpn_key`/`manufacturer_key` against the keys `/manufacturers/normalize` returns for the entry's `mpn` and `mfr`.
- **Values the registry cannot be asked about.** A part number containing `,` or longer than 256 bytes is reported `unknown` with that reason, without a query. An `mpn` of that kind is reported `unbound` with no suggestions. An `mfr` or `mpn` over 256 bytes is not compared, like an empty one.
- **Unlisted supplier.** When the registry has no catalog for a supplier ID (`404 supplier not found`), the report says so at supplier level and lists no per-binding status for it. This does not change the exit code, because §4 allows bindings to unlisted suppliers.
- **Exit codes:**
  - 0 when every audited binding is `ok`
  - 3 when any is `unknown` or `mismatch`
  - 1 on registry or protocol failure (E1204)
  - 2 on invalid arguments
  - `out-of-stock` and `unbound` are reported without changing the exit code.
  - Exit code 3 is new to the CLI, which uses only 0/1/2 today. It is documented in `cohdl --help` and the CLI exit-code documentation, and no other command uses it.
- **It never runs inside `check` or `build`.** Same sources and locks must give the same verdict and bytes, so a fact that changes daily cannot enter them. CI that wants this gate runs `cohdl suppliers` as its own step.
- `--json` is a report shape of its own, like `cohdl search --json`, not RFC-010 diagnostics.

## Type-system-first test

Not applicable: no `rule`/DRC check is proposed. All compiler checks are declaration-time well-formedness on the AVL entry (E813) or reuse unit checking.

## Conceptual impact

No new declaration kind. A **supplier binding** is a new property of an existing AVL entry: a supplier ID, a part number and a placement mapping. It extends the Part concept's purchasing identity from "who makes it" (`mfr`, `mpn`) to "who sells or places it, and how".

Vocabulary:
- "Supplier" and "supplier part number" are new and do not overlap existing terms.
- "Source" is avoided because it means source code and source documents throughout CoHDL.
- `at`/`rotate` keep their existing meaning of a placement transform, here applied between two footprint frames.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Med | Med | Low | Med | Low | Med |

- **Grammar (Med):** one new item form inside AVL entries, built from existing pieces (an identifier, a string, the existing `at`/`rotate` clauses) and recognized contextually. No reserved word is added, and `fmt` gains one canonical item form.
- **Oracle (Med):** the compiler can verify structure and units, but not whether a part number orders that MPN or whether a mapping is right. Addressed by:
  - `cohdl suppliers` (online, outside the verdict);
  - registry catalog status with an as-of date;
  - for official libraries, library CI and assembler previews under the library program.
- **Netlist (Med):** `.net` bytes change only for instances whose primary carries a binding. The neutral CPL is a new file, written only for placed designs. Supplier outputs and `.enet` supplier properties appear only with `--supplier`. Every existing output file keeps its bytes.
- **Trust (Med):** part numbers and mappings are author statements, like MPNs today. Everything that changes daily stays in the registry, dated.

## Gradeability

| Fact | Checked by | Stage |
|---|---|---|
| Clause syntax | E010 | parse |
| Part number well-formed; supplier unique per entry; at most 16 clauses; part number not shared across differing (`mfr`, `mpn`) anywhere in the package set; angle in range; offset has `Length` units | E813 | check |
| Supplier BOM complete | E814 per unbound part declaration, or one E814 when the ID is bound nowhere | build, only with `--supplier` |
| Map applied to the board it was made for | `smt_pos.py --map` frame and coverage checks | tool, before writing |
| Map pads and board pads come from one projection | shared `pad_plans`; projection fixtures (annulus, paste, holes, repeated numbers, both sides, pcbnew round trip) | test suite |
| CPL complete | one E815 listing unplaced machine-placed instances | build, when a CPL is written |
| Neutral CPL agrees with the KiCad board | conformance test against `tools/smt_pos.py --neutral`, top and bottom | test suite |
| Part number exists and orders that MPN | `cohdl suppliers`, exit 3 | on demand / CI, online |
| Mapping is physically right | assembler preview of the CPL; for official libraries, the library program's evidence | outside the compiler |

## AI-generatability

The clause is local to the entry it describes and reads like a placement. The supplier name is free text, so a model can bind any supplier without a special case.

There are two main risks, each with a mitigation:
- **Inventing a part number.** `cohdl search --supplier` returns real part numbers, and `cohdl suppliers` turns a wrong one into a named `unknown`/`mismatch` row and an unbound entry into ranked suggestions.
- **Inventing a mapping.** The mapping defaults to (0, 0, 0°), the documentation says to write a non-zero mapping only from the supplier's footprint, and suggestions never propose a mapping.

## Alternatives

1. **One flat key per supplier** (`acme: "A-104411"`). This was the previous draft. Rejected: it cannot carry a mapping, and it would make the compiler's key set grow with every supplier.
2. **Footprint-level default mapping** (`footprint F { supplier acme rotate 180 }`). Deferred: mappings depend on the supplier's footprint for a specific part number, and a family default could silently apply to a part where it is wrong. It can be added later as a default that per-entry clauses override.
3. **Registry-only bindings.** Rejected: outputs would not be reproducible offline or for local packages, and the registry would override immutable package content.
4. **A per-package sidecar file.** Rejected: it would be a second source of truth beside the AVL, with no place for per-`alt` bindings and no compile-time check.
5. **A closed, compiler-known supplier list with format checks.** Rejected: it would put supplier knowledge into the language and require a compiler release for every new supplier. Format checks belong in the registry's supplier table and `cohdl suppliers`.
6. **Rotation only, no offset.** Rejected: supplier footprints often place the origin elsewhere than the body centre (connectors, modules), and a CPL position error is as harmful as a rotation error.
7. **CPL only on request (`--emit cpl`).** It would avoid a new default file and would inherit `--emit`'s exclusion with `--logical-only` for free. Not chosen: a placed design's CPL is as basic an assembly output as its BOM, and the board author asked for it by default. The costs are contained instead: only placed designs get the file, a partly placed design gets one summary E815, and `--logical-only` suppresses it explicitly (§3).
8. **Pad-centroid position** (`smt_pos.py`'s current `Mid X/Y`). Rejected for the neutral file: the centroid moves when a land is redrawn, and §2's mapping is between footprint origins. The existing script output keeps the centroid for its current users.

## Compatibility

- **Language:** additive. Every existing `.cohdl` file keeps its meaning. E813 (error), E814 and E815 (warnings) are new, with numbers provisional.
- **Outputs:**
  - BOM CSV: unchanged.
  - `.net`: changes only for primaries with bindings.
  - `.enet`: changes only with `--supplier`.
  - `-cpl.csv`: new default output, written only by physical builds of designs that contain `place`; every existing output file keeps its bytes. `--logical-only` builds write exactly the files they write today.
  - `tools/smt_pos.py`: default output unchanged; `--neutral` and `--map` are new.
  - `build --json`: additive `cpl` and `suppliers` keys.
  - `-<ID>-bom.csv`, `-<ID>-map.json`, `-<ID>-cpl.csv`: new, only with `--supplier` (the CPL only in physical builds).
  - API docs: additive.
- **Registry and CLI:** additive (`suppliers` on part rows, `supplier=`/`class=` filters, `/suppliers/{id}/parts`, `cohdl suppliers`).
- **Libraries:** none required by this RFC.

## Tooling & operations

- **Documents updated once Accepted:**
  - Note 10: the supplier clause and mapping rule.
  - `docs/provisional-syntax.md` §2: the AVL item list.
  - `docs/error-codes.md`: E813, E814, E815.
  - `docs/apidocs.md`: the `suppliers` array.
  - The `.enet` netlist, BOM and CPL output documentation.
  - `tools/smt_pos.py` docs: `--neutral`, `--map`.
  - `cohdl --help` and the CLI exit-code documentation: exit code 3 of `cohdl suppliers`.
- **`fmt`:** canonical item form `supplier ID "PN" at (Xmm, Ymm) rotate N`. Written `at`/`rotate` are kept and absent ones stay absent. Supplier clauses always come after every `name:` field and `footprint` (which keep today's source-span order), and in source order among themselves; `fmt` moves a clause written earlier in the entry.
- **LSP:** offers `supplier` as an AVL item completion; hover shows the supplier's display name and link when the registry supplier table is cached.
- **`check --json` / `build --json`:** carry E813/E814/E815 as ordinary diagnostics; the RFC-010 schema is unchanged.
- **Zero-dependency rule:** unaffected. Compiler changes are parsing, checking, string emission and fixed-point transforms; all fetching lives in the registry and `tools/`.

## Teaching cost

- **Authors** learn one clause: supplier, part number, and an optional `at`/`rotate` that describes the supplier's footprint relative to ours.
- **Reviewers** learn which entry a supplier BOM and CPL use: per instance, the primary if bound, else the first bound `alt` that shares the primary's footprint. They also learn that the neutral CPL carries no mapping.
- **Tool builders** learn the §2 composition rule.

## Failure modes

- **Wrong part number.** E813 catches structural errors; `cohdl suppliers` catches unknown or mismatched numbers.
- **Mapping sign or direction reversed** (writing the transform from supplier to ours). The CPL comes out turned or shifted. §2 fixes one direction, and documentation gives a worked example. The assembler's placement preview is the final check.
- **Two parts share a primary but allow different alternates.** Handled: selection is per instance from its own part (§3.2). Supplier BOM rows merge only identical selected identities.
- **Map applied to a board with an older footprint frame.** For example, the library moved a footprint (a breaking release), the map was regenerated, but the routed board still holds the old footprint. The frame check fails with the designator and pad, and no CPL is written.
- **Board exported under an older pad projection.** The footprint frame is unchanged but anchors moved, for example after a change to annulus anchors. The frame check fails rather than guessing; the message names both possible causes, and re-exporting the board's footprints fixes it.
- **Alternate with its own footprint.** Every emitter draws the primary's footprint, so an alternate whose effective footprint differs is never selected for supplier outputs (§3.2). Otherwise its mapping would describe copper that is not on the board. Among eligible entries, the map file and supplier CPL carry the selected entry's mapping, never the primary's by default.
- **Bottom-side angle convention.** Assemblers differ in how they read bottom-side rotation. The neutral CPL fixes one convention (KiCad's, §3.1), and the conformance tests pin it against the board. Whether a given assembler needs another convention is supplier knowledge, settled by its placement preview and kept outside the compiler.
- **Stale supplier files.** Handled by the ownership manifest: files from an earlier `--supplier` run are removed when not produced again (§3).
- **Misspelled supplier ID.** One E814 says the ID is bound nowhere in the design and lists the IDs that are.
- **Neutral CPL sent to a supplier whose frames differ.** Parts come out turned or shifted. The neutral file is named without a supplier ID, and the supplier CPL exists next to it whenever `--supplier` is given.
- **Parts placed outside CoHDL.** The compiler's CPL lists only instances placed in source, and E815 names the rest. Boards laid out elsewhere take their CPL from `tools/smt_pos.py`.
- **Part with an unknown MPN.** A supplier part number on a source-unspecified entry is the author's only ordering claim, and nothing offline can check it. `cohdl suppliers` reports whether the part number exists but cannot report `mismatch` on the empty field, and the BOM row's `Comment` stays empty rather than borrowing the catalog's MPN.
- **CPL expected from a logical build.** `--logical-only` writes no CPL by design; a physical build that fails the physical-export gate writes none either. Neither writes a partial file.
- **Bottom-side parts.** The mapping goes through the existing mirror-then-rotate composition, and tests pin top and bottom cases against the geometry emitters.
- **Supplier renames or retires a part number.** The registry shows it as `unknown`, and `cohdl suppliers` exits 3. The package keeps its binding until a new version is published.

## Open questions

- **Mirroring in a mapping.** A mapping is rigid (offset and rotation). A supplier footprint that is the mirror image of ours cannot be expressed. The one known reverse-mount part (a side-view LED validated in production) needs only a rotation, so mirroring is left out until a real case needs it.
- **Bottom-side convention in practice.** §3.1 follows KiCad's convention, derived from how `--emit kicad_pcb` writes bottom-side angles. Before acceptance, a test board with bottom-side parts should confirm two things: that `tools/smt_pos.py --neutral` and the compiler CPL agree, and that one assembler's preview places them correctly.

## Migration path

None required. Designs adopt supplier output by adding clauses to their own parts, or by updating to library versions that carry them.

## Decision

Draft, 2026-10-23; revised 2026-10-27 and 2026-10-28 after four source reviews against `329af05` (the third defined the map's pad projection, the fourth its matching rule). Awaiting review; no decision record yet.

Implementation may proceed in stages before acceptance:
- The clause syntax, checks, `fmt`, per-instance selection, supplier BOM, registry and API docs do not depend on the map check and may start now.
- The map, the CPLs and the `smt_pos.py` options follow this revision's pad-projection rule.
- Official library bindings and footprint alignment ship only with each part number's confirmation and frame evidence (the library program's rules).

Implementation status, 2026-10-28: every stage is implemented on the local branch `feat/rfc-034` and passes the compiler, registry and tool test suites. Rulings made during implementation are folded into §1, §4 and §5 above. Two projection tests need pcbnew and are carried as a manual checklist (open-and-save, and a hand flip in each direction); they are part of acceptance. LSP completion and offline hover are implemented; hover with the registry's supplier display name and link waits for a local cache of the supplier table.

Acceptance still requires central allocation of the RFC number and error codes, and the bottom-side test board and assembler preview in Open questions. Until then this text is not frozen. This file is a working copy in the repository snapshot. The canonical RFC belongs in the conol.ai design repository, where its number and error codes are allocated.
