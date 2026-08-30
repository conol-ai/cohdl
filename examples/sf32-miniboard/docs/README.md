# SF32 mini-board display and mechanical sources

The H0216F002AM interface follows the Huaxia RGB module specification already
preserved by `@contrib/display` as `docs/h0216f002am.pdf` (SHA-256
`db2dd91303f847463fdcb20decff7230ee3a6c215056d4c0d0d2936934cb7715`).
The board-local receptacle land follows OCN Technology's
`OK-F302-**115` series specification/drawing, revisions A/A1:

- source: `main-soul.com/datasheet/ocn/Spec_Draw_OK-F302.pdf`
- retrieved copy SHA-256:
  `ebc73a00c6a5649198b732a27986f3c60f22b84de33d67a612e1c79863c5a069`
- n=31 dimensions: A=10.80mm, B=8.40mm, C=9.00mm

The row counts and dimensions fix the 16-odd/15-even pad split, but the source
drawing's contact view is easy to mirror. Confirm the stagger and pin-1 end
against a physical `OK-F302-31115` before fabrication.

The 3.987V panel rail uses the Richtek RT6150A/RT6150B datasheet carried by
`@richtek/dcdc`; the switched VCI rail uses SGMICRO's SGM2554A datasheet
carried by `load-switch`. The H0216 sheet leaves operating-current fields
blank, so this design does not certify the panel's full-brightness USB power
budget.

`mechanical/sf32-miniboard-outline.dxf` is a board-authored 60 x 40mm rounded
rectangle with 3mm corner radii. It contains one closed `LWPOLYLINE` on
`Edge.Cuts`; the four M2 NPTH features are placed components rather than
additional outline loops.
