# Linx CONMHF4-SMD-G-T evidence

- Manufacturer: Linx Technologies (TE Connectivity).
- Exact order code: `CONMHF4-SMD-G-T`; the datasheet identifies the
  tape-and-reel suffix of the `CONMHF4-SMD-G` connector.
- Manufacturer document: local `conmhf4-smd-g-t.pdf`, sha256
  `63fedb5a916e68998c0ec4aafd5797065a19ffb404fb68fbbf44ad97910a7513`.
  Page 3, Figure 2, "Recommended PCB Dimensions for the CONMHF4-SMD".
- Footprint origin: the drawing's horizontal reference line (the one the two
  1.25 mm dimensions start from), centred between the side shield lands.
  The view is the figure as drawn, with the top shield land towards -y.

## Recommended lands and implementation

All dimensions and coordinates are in millimetres. Figure 2 dimensions:
2.5 overall span of the side shield lands, 1.34 gap between them, 1.58 their
height; top shield land 0.5 wide, its top 1.25 and its bottom 0.75 from the
reference line; signal land 0.60 wide, its top 0.65 and its bottom 1.25 from
the reference line. The 1.5 dimension is to the connector's ground legs, not
to copper. The side lands are centred on the reference line (measured from
the rendered figure: -0.79 to +0.79).

| Pad(s) | Figure 2 land | Implementation | Maximum deviation |
| --- | --- | --- | --- |
| 1 (signal) | 0.60 x 0.60 at (0, 0.95) | 0.60 x 0.60 at (0, 0.95) | 0.00 |
| 2 (top shield) | 0.50 x 0.50 at (0, -1.00) | 0.50 x 0.50 at (0, -1.00) | 0.00 |
| 2 (left shield) | 0.58 x 1.58 at (-0.96, 0) | 0.58 x 1.58 at (-0.96, 0) | 0.00 |
| 2 (right shield) | 0.58 x 1.58 at (0.96, 0) | 0.58 x 1.58 at (0.96, 0) | 0.00 |

Figure 2 is not to scale for one dimension: the signal land's 0.65 top edge
measures 0.70 on the rendered drawing. The dimension governs. The three
shield lands share electrical pin 2 (`GND`). The courtyard is the copper
envelope (2.5 x 2.5, which contains the connector outline drawn in the
figure) plus 0.25 per side: 3.0 x 3.0 at the origin. RF feed geometry,
stack-up and via fencing remain the consuming board's responsibility.
