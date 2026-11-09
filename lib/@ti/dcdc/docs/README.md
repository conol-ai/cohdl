# Official documentation

Retrieved 2026-07-30 from Texas Instruments.

## `tps59650-datasheet.pdf`

- Covers: `TPS59650RSLR` in the 48-pin RSL VQFN package.
- Document identity: SLUSAV7, January 2012.
- Official URL:
  <https://www.ti.com/lit/ds/symlink/tps59650.pdf>
- SHA-256:
  `5afb9e1c3ff9da14259014c3315bc56cb099ed725d6c8f20aadcca3b1463cc68`
- Footprint authority: the appended official RSL mechanical, thermal-pad,
  and land-pattern drawings 4207548, 4207841, and 4209046.

The pin table was checked on PDF pages 10-12. The package and land-pattern
drawings were visually inspected on PDF pages 48-51.

## Changelog

### 0.2.0 — 2026-11-07

- Breaking (footprint geometry): `CTRL_TPS59650` now uses the package-owned
  `TI_RSL_S_PVQFN_N48` (`src/footprints.cohdl`) instead of
  `qfn::QFN48N40P600X600_1EP405X360`. The 48 perimeter lands are unchanged.
  The thermal land (pad 49) was 4.05 x 3.60mm centred at (0, 0), with the
  4.05mm side along X, perpendicular to the pin 1-12 column. It is now
  3.60 x 4.05mm centred at (0, -0.225mm): the 4.05mm side runs parallel to
  pins 1-12 as in TI's RSL "Example Board Layout" (PDF p. 51, drawing
  4209046-2/O), and the land sits 0.225mm toward pins 37-48, as set by the
  0.30/0.75mm land-to-land gaps on p. 51 and the 1.20mm NOM. pad-to-edge
  dimension on p. 50 (drawing 4207841-3/P). Boards that placed the old land
  must re-route around the new thermal pad.
- The EP pin comment now records TI's grounding rule: the thermal pad is the
  analog ground and must not be connected directly to PGND (pin 42)
  (PDF pp. 12, 40, 43).
- The `qfn` dependency is dropped; the package now depends on `std` only.
- Why the land moved here rather than being fixed in `qfn`: TPS59650 was the
  only user of `qfn::QFN48N40P600X600_1EP405X360`, and the RSL land is
  TI-specific (off-centre thermal pad), which LIBRARY.md keeps with the
  manufacturer package. Fixing it in `qfn` would need a breaking `qfn` release
  and new versions and locks for every package and example that pins
  `qfn = "0.2.1"` (`@espressif/esp32`, `@raspberrypi/mcu`, `@richtek/dcdc`,
  `flash`, and the examples built on them). `qfn` 0.2.1 is left as is; its
  `QFN48N40P600X600_1EP405X360` still has the turned, centred thermal land,
  has no remaining user, and should be removed or corrected in the next
  breaking `qfn` release.

The thermal-pad orientation and offset were re-checked on PDF pages 49-51
rendered at 300-600 dpi on 2026-11-07.
