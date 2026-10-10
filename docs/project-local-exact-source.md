# Exact Source Extensions

These project-local extensions were migrated to upstream revision `9b05275`
from the accepted SF32/Air780 exact-source patches. Existing declarations and
generated output remain unchanged when the extensions are not used.

## Polygon Pads

```cohdl
pub pad P {
    shape: polygon
    size: (1mm, 1mm)
    vertices: [(-0.5mm, -0.5mm), (0.5mm, -0.5mm), (0mm, 0.5mm)]
    layer: top_copper
    plating: smd
}
```

Vertices are ordered local coordinates relative to each placed pad center.
`size` preserves the supplier's declared positive extent metadata; it does not
recenter, clip, or replace the polygon. Native supplier centers can be rounded
or offset from the polygon bounding-box center. Consecutive duplicate points
and a repeated closing point are preserved in the AST, footprint, and model.
Validation removes those duplicates only in a temporary geometry view, then
rejects fewer than three distinct points, zero area, overlapping adjacent
edges, nonconsecutive repeated vertices, and intersecting/touching edges.

Coordinates must be Length literals within +/-100mm; at most 4096 authored
vertices are accepted. These bounds keep all exact i128 cross products and
signed-area sums in range. Nonzero mask expansion is rejected because an exact
polygon offset is not implemented. Chamfer and corner-radius controls are
incompatible with polygon geometry. Ordinary pad size, plating, drill, and
paste checks still apply.

KiCad uses a custom filled `gr_poly`, preserving all local coordinates and pad
rotations. Its 1nm anchor is proven contained by the polygon using exact integer
geometry, so it contributes no extra copper. Polygons that cannot contain this
anchor at their declared center fail with an explicit unsupported-export cause.
IPC-2581
uses exact contours with the established Y-axis conversion and bottom-side
mirror. Explorer emits optional `vertices: [[x, y], ...]` in mm only for
polygons and renders the actual rotated outline with bounds derived from its
vertices.

## Source-Empty Procurement Fields

```cohdl
#[source_unspecified("mpn")]
pub part P: Device {
    primary { mfr: "Known manufacturer", mpn: "", footprint: Footprint }
}
```

The marker names only explicitly empty primary `mpn` and/or `mfr` fields.
It cannot cover omitted fields, whitespace-only values, nonempty guesses,
unknown or repeated field names, alternative AVL entries, or other declarations.
Duplicate markers fail. Ordinary unmarked procurement validation and all
footprint/device checks remain active. Source labels and provenance are separate
from procurement identity. Compiler BOM, netlist, IPC-2581 characteristics, and
Explorer preserve the empty strings. An unknown MPN is grouped by the bound
part declaration as well as procurement fields so unrelated unknown parts
cannot lose their footprint identity through BOM grouping.

## Source-Unspecified Hole Plating and Logical Builds

```cohdl
pub pad SurfaceCopper {
    shape: annulus, size: (4mm, 2mm), layer: top_copper,
    plating: smd, paste: none
}
pub footprint Screw {
    pad 1: SurfaceCopper at (0mm, 0mm)
    mount_hole 1: source_unspecified at (0mm, 0mm) diameter 2mm
}
```

The undrilled annulus preserves observed surface copper mapped to an electrical
contact. Its `smd` value describes this copper primitive; it makes no claim about
the screw package's mounting technology. A separate co-located mount hole
preserves the observed aperture and its explicitly unknown plating. Normal
hole shape, dimensional units and positive-dimension checks still apply.

`cohdl check` and Explorer extraction accept this exact logical geometry. The
Explorer hole field `plated` is `null` for unknown, `true` for plated and `false`
for non-plated. The preview draws annulus copper with its actual outer and inner
diameters and exposes source-unspecified plating in the hole tooltip.
Every extracted pad also exposes its actual declared `layer` as `top_copper`,
`bottom_copper`, `top_paste`, `bottom_paste`, or `through_all`; incomplete declarations expose `null` rather
than guessing a copper side. This additive field permits independent comparison
of observed surface and through-hole copper with the compiler model.

`cohdl build --logical-only` writes normal logical netlist, BOM, designator lock
and applicable layout metadata, while suppressing KiCad footprints, IPC-2581
and Quilter physics CSVs. It is valid only on `build` and cannot accompany
`--emit`. Normal physical builds reject referenced primary or alternative
footprints with unknown hole plating before creating or changing artifacts;
unused unknown footprints do not block an otherwise known design. The public
KiCad footprint, KiCad board and IPC exporter APIs return `PhysicalExportError` for the same boundary,
so callers cannot silently reinterpret unknown as non-plated. Existing known
annulus physical output is unchanged.

## Regression Fixtures

`tests/fixtures/native_polygon_pads.json` contains unchanged native PAD strings
for all nine U1 contact61 placements from C2980299 and U10 contact2 from
C7465551. Native coordinates use 10mil = 0.254mm. Original supplier response
SHA-256 values:

- C2980299: `32ba999be1c6bcf190df041f0a7ef3de1a18e7288bf5ca7f0f5a010721280ac4`
- C7465551: `f8f95af41016e7a28b122808c1831ff204ced33637a339b063565e0009266c6c`

Native fixture tests exercise actual CLI check/build/extract, ordered vertices,
contact multiplicity, formatter roundtrips, geometry rejections, and IPC-2581
schema validation. Source-identity tests include asymmetric empty fields and
invalid markers. Language-service tests exercise polygon hover and diagnostics.
Existing extractor example tests prepare disposable projects against the
archived local library using structured dependency discovery; stale historical
example manifests and locks are retained unchanged.
Unknown-hole regressions exercise actual logical and physical CLI paths,
no-write retries, normal known geometry, alternate and unused footprints,
nullable extraction through the real CLI, normal geometry errors, formatter
roundtrips and language-service help.
