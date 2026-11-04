# JSCJ S8050 evidence record

- Library coverage: Jiangsu Changjing Electronics Technology `S8050 J3Y(RANGE:200-350)` and `FP_JSCJ_SOT23`.
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C2146.html>.
- Manufacturer document served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/916f796ed66ea3166350dbd91c9a46f7.pdf?productCode=C2146>.
- Datasheet SHA-256: `02c34a6b6f6840adc94fbea81a7c6a738cc9f4c417dbf4c717df58f5a3e13271`.
- Document identity: JIANGSU CHANGJING ELECTRONICS TECHNOLOGY CO., LTD, S8050 SOT-23 Plastic-Encapsulate Transistors, Rev. 2.0.
- Pin facts: pin 1 Base, pin 2 Emitter, pin 3 Collector; package marking `J3Y`.
- Electrical facts: VCEO 25 V, collector current 500 mA, power dissipation 300 mW; J gain rank is hFE 200–350.
- Package facts: SOT-23 body D=2.8–3.0 mm, E=1.2–1.4 mm and E1=2.25–2.55 mm, 0.95 mm lead pitch.
- Geometry used: the manufacturer suggested land uses 0.6 × 0.8 mm pads, 1.90 mm horizontal centre spacing and 2.02 mm vertical centre spacing.
- Qualification boundary: this is an engineered selection for low-current reset, power-key and indicator sinks. Base drive, saturation voltage, thermal rise and assembly marking must be checked on the final board.

The third-party PDF is not redistributed by this repository. Download it from the authorized source and verify the recorded SHA-256 before auditing the model.

## Assembler frame (library plan W3)

Version 0.2.0 turns `FP_JSCJ_SOT23` 90 degrees clockwise onto the assembler's
frame, measured on the preview of 2026-10-30
(`tools/sourcing_data/jlc_previews/2026-10-30-c2-board-1/`). The delta from the
old frame to the new one is the RFC-034 section 2 triple `(0mm, 0mm, 90)`.
Pads, courtyard and silkscreen turn together; pad numbers and lands are
unchanged. A design that `place`s this footprint keeps its copper only after
each placement is composed with the delta, which `tools/realign_place.py` does.
