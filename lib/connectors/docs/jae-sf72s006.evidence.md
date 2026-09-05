# JAE SF72S006VBA(R2500) evidence

- Manufacturer: Japan Aviation Electronics Industry, Ltd. (JAE).
- Exact reel order code: `SF72S006VBA(R2500)`; base connector drawing:
  `SF72S006VBA` / `SF72S006`.
- Manufacturer product-information document: JAE `MB-0282-1`, "nano SIM Card
  Connector SF72 Series", August 2014, 5 pages. The manufacturer-authored PDF
  is mirrored at
  <https://datasheet.lcsc.com/datasheet/pdf/b13cbe906d2905e1a5778f79cd6e142e.pdf?productCode=C2977289>.
- Document SHA-256:
  `e55440fbf6ea70bbe5cf8821b1838ab011151fbc53f16dba835e3dd6224c6707`.
- Function: push-push nano-SIM (4FF) connector with detect switch.
- Document facts: page 1 identifies the 6-contact, 1.25 mm-high push-push
  connector and normally-open card-detect switch; page 2 gives the exact
  `SF72S006VBA(R2500)` ordering construction and 2,500-piece reel suffix;
  page 3 names individual drawing `SJ114535`; page 4 provides the applicable
  PWB dimensions and pin assignment; page 5 identifies reeled-product drawing
  `SJ114536`, specification `JACS-11019`, and handling document `JAHL-11019`.
- Contact naming follows ISO/IEC 7816 and the manufacturer drawing: C1 VCC,
  C2 RST, C3 CLK, C5 GND, C6 VPP/SWP, C7 I/O, plus `DSW` and `CSW` for the
  isolated card-detect switch and repeated `SH` shell lands.
- Geometry cross-check: KiCad's manufacturer-specific
  `Connector_JAE.pretty/JAE_SIM_Card_SF72S006.kicad_mod`, sha256
  `317edcf4ab309cc5bafbcc86482cab63c143f0569f6d4c3505e02a47cb27871a`.
  The public footprint reproduces all six card contacts, both detect contacts
  and every shell land from that drawing.
- Qualification boundary: the board must choose the detect polarity by wiring
  one of `DSW`/`CSW` to its pull-up input and the other to ground. `C6` is not
  required by ordinary UICC operation and may remain unconnected.

The manufacturer document is not redistributed by this repository.
