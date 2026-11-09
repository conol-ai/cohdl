# Official source documents

`jscj_s8050.evidence.md` records the authorized manufacturer document, exact
ordering identity, pinout, electrical limits, SOT-23 dimensions and suggested
land used by `transistor::BJT_JSCJ_S8050_J3Y`.

## LCSC binding (transistor 0.2.1)

`BJT_JSCJ_S8050_J3Y` binds its primary to LCSC C2146 with a zero map: `supplier lcsc "C2146"`. The part number is in `tools/sourcing_data/lcsc_map.json` since the B2 full run, and its frame row `c2b1-sot23_s8050-c2146` cites C2 frame board 1 (`tools/sourcing_data/jlc_previews/2026-10-30-c2-board-1/`). That preview read a quarter turn on the land as it was before 0.2.0, which `FP_JSCJ_SOT23` has since been moved onto. No land changed from 0.2.0, so a plain `build` writes the same BOM and CPL. The three `transistor` frame rows now name 0.2.1.
