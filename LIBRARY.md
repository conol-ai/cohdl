# Component library authoring pattern

This playbook records the pattern established while completing
[`@espressif/esp32`](lib/@espressif/esp32/) and extracting its reusable QFN56
land pattern into [`qfn`](lib/qfn/). Apply the same process when adding or
revising an official component library.

The goal is a library whose part inventory, primary-source documents, pin
models, package ownership, dependency locks, and emitted footprints can all be
audited without relying on undocumented assumptions.

## Reference outcome

| Concern | ESP32 implementation | Reusable rule |
|---|---|---|
| Manufacturer devices and parts | [`lib/@espressif/esp32/src/`](lib/@espressif/esp32/src/) | Keep manufacturer-specific component models in `lib/@manufacturer/family`. |
| Manufacturer-specific module land pattern | `FP_ESP32_S3_WROOM_1` in [`footprints.cohdl`](lib/@espressif/esp32/src/footprints.cohdl) | Keep module, connector, and other vendor-specific geometry with the component library. |
| Reusable package land pattern | `qfn::QFN56N40P700X700_1EP400X400` in [`lib/qfn`](lib/qfn/) | Put reusable QFN, DFN, and SON geometry in a dedicated package library and reference it by a qualified name. |
| Primary-source documents | [`lib/@espressif/esp32/docs/`](lib/@espressif/esp32/docs/) | Save every applicable official datasheet locally and record provenance and checksums. |
| Cross-library dependency | `qfn = "0.1.0"` in the ESP32 manifest | Pin exact versions and commit generated `cohdl.lock` files. |
| Shipped-library validation | [`tests/library.rs`](tests/library.rs) | Load every dependency declared by the package before checking part/footprint consistency. |

The reference structure is:

```text
lib/
  @manufacturer/family/
    cohdl.toml
    cohdl.lock
    docs/
      README.md
      official-datasheet.pdf
    src/
      chips/
      modules/
      footprints.cohdl
  qfn/
    cohdl.toml
    cohdl.lock
    src/
      footprints.cohdl
```

Only create directories that the library actually needs. A simple component
family can keep all declarations directly under `src/`.

## 1. Define inventory and ownership first

The manifest's `[package] name` must match its `lib/` path, use an exact
`X.Y.Z` version, and include the standard license, description, repository,
and exact `std` dependency metadata. Files directly under `src/` export at the
package root; nested directories add module-path segments.

List every `pub part` and `alt` that the library will ship before downloading
documents or creating footprints. For each purchasable MPN, identify:

- the device and exact pin map;
- the official datasheet that covers it;
- the package or module land pattern it uses;
- whether it is truly interchangeable with another MPN.

An `alt` is valid only when the pin map, electrical connection obligations,
and footprint are compatible. Do not group variants merely because their
marketing names are similar. For ESP32-S3, the base bare SoC was added as its
own part; integrated-memory variants were not declared as alternates because
they reserve different pins, change supply behavior, or use different exposed
pad geometry.

Ownership follows the geometry:

- reusable standardized package geometry belongs in a focused package library
  such as `qfn`;
- manufacturer-specific modules and unusual packages remain with the
  manufacturer library;
- a footprint has one owner and consumers reference it instead of copying it.

## 2. Preserve official primary sources

Use the manufacturer datasheet and manufacturer CAD as the authority. Prefer
the official DXF, PADS/ASC, or equivalent land-pattern file over a third-party
footprint. Use the package drawing in the datasheet as an independent
cross-check. Reject arbitrary mirrors and HTML/error responses masquerading
as document or CAD downloads. If a manufacturer endpoint blocks automated
binary retrieval, a manufacturer-authored document served by an authorized
distributor or manufacturer-branded mirror is an acceptable fallback only
when `docs/README.md` records both URLs, explains the exception, and verifies
the document identity, revision, page content, and checksum after download.

Save unmodified PDFs under the manufacturer library's `docs/` directory.
Create `docs/README.md` containing, for every file:

- local filename;
- exact parts or variants it covers;
- official download URL;
- document version and release date;
- SHA-256 checksum;
- official CAD URLs used for footprint geometry;
- the CAD retrieval date and checksum when the vendor does not publish a
  versioned CAD artifact.

One family datasheet may cover several parts, but the mapping must be explicit.
Attach the local document to both the device and part declarations:

```cohdl
#[doc("docs/vendor-family-datasheet.pdf")]
pub device DEVICE_NAME {
    // ...
}

#[doc("docs/vendor-family-datasheet.pdf")]
pub part PART_NAME: DEVICE_NAME {
    // ...
}
```

Prefer the current manufacturer revision and compare it with any legacy sheet
already used by the library. A stable family name does not guarantee stable
pin numbering or ordering suffixes. If a revision changes either, update the
device pin map, footprint pad numbers, exact purchasable MPN, provenance
manifest, and downstream locks together; record the superseded mapping in the
manifest so the change is auditable.

Validate each downloaded PDF before treating it as source material:

```sh
file lib/@manufacturer/family/docs/*.pdf
pdfinfo lib/@manufacturer/family/docs/example.pdf
pdftotext lib/@manufacturer/family/docs/example.pdf -
shasum -a 256 lib/@manufacturer/family/docs/*.pdf
```

Render and visually inspect the pin-table, package-drawing, and recommended
land-pattern pages. Text extraction alone will not reveal mirrored geometry,
diagram orientation, or drawing-layer mistakes.

## 3. Model devices and purchasable parts

Transcribe the complete official pin table, including exposed pads. Preserve
the manufacturer's primary pin names and put useful aliases in comments.

Use `required` and `optional` to express connection obligations, not whether a
feature is interesting to a particular design. Record configurable or unusual
power-pin behavior explicitly. The ESP32-S3 model, for example, documents
`VDD_SPI` as a configurable supply whose default behavior is a 3.3 V output.

Every purchasable part must have:

- manufacturer;
- exact MPN;
- a resolvable footprint symbol;
- a local `#[doc(...)]` reference.

Use a fully qualified footprint for another package:

```cohdl
pub part PART_NAME: DEVICE_NAME {
    primary {
        mfr: "Manufacturer",
        mpn: "Exact-MPN",
        footprint: qfn::QFN56N40P700X700_1EP400X400
    }
}
```

Cross-check pin tables in both directions: source-to-model and model-to-source.
This caught the previously swapped WROOM `RXD0` and `TXD0` pins.

## 4. Build footprints from checked geometry

Never ship an empty or nominal placeholder for a referenced footprint. For
each real land pattern, verify:

- pad count and pad numbers exactly match the device;
- pad sizes, pitch, centres, shapes, rotations, and plating;
- exposed-pad number and envelope;
- top-view numbering and pin-1 location;
- silkscreen pin-1 marker;
- courtyard and reference-text placement;
- any antenna, component, or copper keepout.

State the coordinate conversion in the source. Manufacturer CAD commonly uses
Cartesian `+Y` upward, while CoHDL/KiCad board coordinates use `+Y` downward.
Negate the CAD Y coordinates deliberately and then check numbering chirality.
This review caught vertically mirrored first drafts of both ESP32 footprints.

Keep the official CAD URL and the exact source dimensions beside the footprint.
Use the IPC-style identifier when one is available, such as
`QFN56N40P700X700_1EP400X400`.

When CoHDL cannot express a manufacturing detail, preserve electrical
correctness and document the required layout action prominently. Current
examples are:

- repeated same-number exposed-pad copper islands;
- independent segmented paste apertures;
- thermal-via arrays;
- footprint-level antenna keepouts.

For the QFN56 and WROOM exposed pads, the library preserves the full copper
envelope and explicitly requires paste segmentation and thermal vias during
layout. The WROOM antenna outline is only a silkscreen guide, so the board must
enforce the official keepout.

## 5. Extract reusable package footprints

When a land pattern is reusable, finish the dedicated dependency before
locking the consumer:

1. Create `lib/<package-family>/cohdl.toml` with an exact version and a pinned
   `std` dependency.
2. Move the pad and footprint declarations together with their CAD provenance.
3. Finalize and format the dependency, run `cohdl update` for it, and then
   check it so its own `cohdl.lock` is current.
4. Add the exact dependency version to the manufacturer manifest.
5. Replace the local symbol with a qualified reference.
6. Run `cohdl update` for the manufacturer package and then check it, updating
   the consumer's lock entry through the CLI.
7. Remove the old declaration completely; do not leave two owners.
8. Add the package library as a direct dependency of every design that
   instantiates the manufacturer part. CoHDL intentionally resolves only a
   project's direct dependency set; it does not traverse dependency manifests
   transitively.

For the ESP32 extraction:

```toml
[dependencies]
qfn = "0.1.0"
std = "0.3.0"
```

```cohdl
footprint: qfn::QFN56N40P700X700_1EP400X400
```

A board that instantiates this part also declares `qfn = "0.1.0"` alongside
`"@espressif/esp32" = "0.1.0"`. The manufacturer package needs `qfn` so it can
be checked independently, while the board needs `qfn` so the qualified
footprint symbol is present in its own compilation world.

Do not hand-edit `cohdl.lock`. A dependency's content hash covers its package
files, so finish the dependency before recording it in a consumer. Once a
version is released, changed content requires a new version rather than
silently replacing the locked geometry. An ordinary `check` creates missing
first-resolution rows, but it deliberately fails on a stale hash with E1103;
use `update` to refresh pre-release authoring locks explicitly.

Add every new shipped package to [`lib/README.md`](lib/README.md), including a
representative qualified symbol and a precise ownership description.

## 6. Validate the complete result

Run canonical formatting and package diagnostics independently for the shared
footprint library and every consumer:

```sh
cargo run --quiet -- fmt lib/qfn
cargo run --quiet -- update lib/qfn
cargo run --quiet -- fmt lib/qfn --check
cargo run --quiet -- check lib/qfn --json

cargo run --quiet -- fmt lib/@manufacturer/family
cargo run --quiet -- update lib/@manufacturer/family --dep qfn
cargo run --quiet -- fmt lib/@manufacturer/family --check
cargo run --quiet -- check lib/@manufacturer/family --json
```

For each new or changed footprint, also build a small consuming design and
inspect the emitted KiCad footprint. Confirm the emitted pad count, numbering,
pin-1 position, coordinates, sizes, rotations, exposed pad, courtyard, and
silkscreen rather than assuming successful parsing proves correct geometry.
Use a separate temporary consumer; do not rename the real package manifest to
make a QA build emit artifacts. Ensure each reusable footprint is bound by at
least one real consumer or test fixture so pad consistency cannot pass
vacuously.

Remove temporary QA designs and rendered files afterward. Finish with a scoped
artifact sweep: no temporary source, `out/`, `tmp/`, `obj/`, rendered PDF
pages, or QA-only `design.lock` should remain. Keep `design.lock` only when the
package intentionally ships a real `design`.

Finish with repository validation:

```sh
cargo fmt --all --check
cargo test --quiet every_shipped_component_library_has_consistent_part_footprints -- --exact
cargo test --quiet
```

The shipped-library consistency test must resolve the dependencies declared by
each manifest. Passing a manufacturer package with only `std` loaded can hide
or falsely report cross-library footprint references.

## Acceptance checklist

- [ ] Every shipped MPN appears in the inventory and maps to an official
      datasheet.
- [ ] Official PDFs are stored locally, readable, versioned in `docs/README.md`,
      and checksum-recorded.
- [ ] Official CAD provenance includes its URL and enough retrieval identity
      to detect vendor-side replacement.
- [ ] Devices and parts carry valid local `#[doc(...)]` references.
- [ ] Pin numbers, names, directions, aliases, and connection obligations match
      the primary source.
- [ ] Alternates are genuinely pin-, obligation-, and footprint-compatible.
- [ ] Every referenced footprint exists and has complete, source-backed
      geometry.
- [ ] Reusable footprints have one owner in a focused package library.
- [ ] Every standalone footprint is exercised by a real consumer or test
      fixture.
- [ ] Manufacturer-specific footprints remain with the manufacturer package.
- [ ] CAD coordinate orientation and top-view numbering were checked visually.
- [ ] Exposed-pad, paste, via, and keepout limitations are explicit.
- [ ] Manifests use exact dependency versions and generated locks are current.
- [ ] The package is listed in `lib/README.md`.
- [ ] Temporary designs, emitted outputs, renders, and QA-only locks have been
      removed.
- [ ] Formatter, package checks, emitted-footprint inspection, the
      shipped-library test, and the full test suite pass.
