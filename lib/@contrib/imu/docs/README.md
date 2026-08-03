# IMU source manifest

Retrieved and audited on 2026-08-03. The PDF is stored byte-for-byte as
downloaded; it was not regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `bst-bhi260ap-ds000.pdf` | `BHI260AP` | Bosch Sensortec BST-BHI260AP-DS000-02 Revision 1.1 | <https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bhi260ap-ds000.pdf> | `1db636a1dcdc6cf2c5da3bd4b324aa210043d6e340327f2ccaa068f39b97b5c9` |

## Retrieval and lifecycle notes

- Downloaded from the official Bosch Sensortec URL above. The first attempt
  silently truncated at 3,439,947 of 5,104,295 bytes (curl exited 0); the
  file was completed with `curl -C -` resume and then structurally verified
  (opens, 173 pages, text extractable). Verify structure, not just size,
  when re-fetching.
- The PDF carries AES-256 encryption with an empty password. Tools that
  cannot open it directly (PyMuPDF) work after a `pikepdf` empty-password
  decrypt pass; the stored file is the original encrypted download.
- Bosch lists BHI260AP as not recommended for new designs in favour of
  BHI360/BHI380; the datasheet remains the authoritative document for this
  part.

## Coverage and geometry

- Pages 19-21 (Table 1, Figure 1) supply the complete 44-pad map with reset
  values; Figure 1 gives the bottom-view numbering and pin-1 corner.
- Page 159 (Figure 33) supplies the LGA-44 outline: 3.6 x 4.1 x 0.83 mm
  body, pad-array reference spans, and the pin-1 corner.
- Page 162 (Figure 37) supplies the PCB land-pattern recommendation used
  for `FP_Bosch_BHI260AP_LGA44_3_6x4_1mm`: 0.35 x 0.20 mm pads, 0.40 mm
  pitch, 3.17 mm column span, 3.67 mm row span, 0.45 mm inner-to-outer
  column gap, 0.30 mm inner-to-outer row gap, plus a 50 um soldermask frame
  around each pad (44x) and 0.05 x 45 deg pad corner chamfers, neither of
  which CoHDL can express (handle in layout; the chamfers are cosmetic at
  fab resolution).
- Figures 33 and 37 are embedded raster images with no extractable vector
  or text data. Dimensions printed in Figure 37 were read visually from
  high-resolution renders; pad centres that carry no printed dimension
  (the 0.47 mm centre gap between outer-column pads 4-5/18-19, and the
  regularized undimensioned centres) were measured from the native
  1165x893 px embedded raster calibrated against the four printed
  reference spans (129.9 px/mm; all printed dimensions reproduce within
  +/-1.5 px). The measurement method and full derivation are archived in
  the cohdl-doc repository under `reports/bhi260ap/`.
- Figure 37's top-view numbering was cross-checked pad-by-pad against the
  Figure 1 bottom view (mirror about the horizontal axis); all 44 positions
  agree, pin 1 upper-left in top view.
- No manufacturer CAD file is published for this package; the footprint is
  derived solely from the dimensioned land-pattern and outline drawings.
