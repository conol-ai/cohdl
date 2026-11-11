# qfn

QFN, DFN and SON land patterns shared by several manufacturer packages.
`esp32-generated-footprints.md` documents the generated ESP32-family lands.

## Changelog

### 0.3.0 (2026-11-10)

- Removed (breaking): `QFN48N40P600X600_1EP405X360` and its three pads
  `P_QFN48N40P600X600_LEAD_H`, `P_QFN48N40P600X600_LEAD_V` and
  `P_QFN48N40P600X600_EP405X360`. The footprint was drawn for the TI
  TPS59650 RSL package and has a wrong thermal land. TI's 4.05 x 3.60 mm
  land runs with its 4.05 mm side parallel to pins 1-12 and sits 0.225 mm
  towards pins 37-48 (TPS59650 datasheet SLUSAV7 pp. 50-51). Here the
  land was turned 90 degrees and centred at (0, 0). The 48 perimeter lands
  were correct.
- It has no user. `@ti/dcdc` 0.2.0 (PR #94) moved `CTRL_TPS59650` to its
  own `ti_dcdc::TI_RSL_S_PVQFN_N48`, which has the correct land. No other
  package, example or fixture names the footprint. The CoHDL language has
  no deprecation marker, so the footprint is removed rather than left in
  place. A design that still names it now fails to resolve and cannot
  reuse it by accident. `@ti/dcdc` 0.2.0 asked for this in its changelog.
- Published `qfn` 0.2.1 is unchanged and still contains the footprint. A
  design pinned to 0.2.1 that places it must move to
  `ti_dcdc::TI_RSL_S_PVQFN_N48` (for TPS59650) or another verified land.
- All other footprints and pads are byte-identical to 0.2.1.
