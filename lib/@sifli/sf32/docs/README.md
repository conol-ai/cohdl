# MCU source manifest

Retrieved and audited on 2026-08-04. The PDFs are stored byte-for-byte as
downloaded; none was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `ds0052-sf32lb52x-cn.pdf` | `SF32LB52X` and public `SF32LB52EUB6` binding | SiFli DS0052-SF32LB52X V0.1, 2024 | <https://www.sifli.com> | `68ad87f5846c47908614e980eaa56b0eea5e4ae036d0a5cee9357f80db7357c2` |

SHA-256 checksums:

```text
68ad87f5846c47908614e980eaa56b0eea5e4ae036d0a5cee9357f80db7357c2  ds0052-sf32lb52x-cn.pdf
```

## Retrieval notes

The manufacturer endpoint blocked automated binary retrieval. The manufacturer-authored document was retrieved from an authorized source.

## Current official cross-check

The logical pin map was re-audited on 2026-08-05 against these primary SiFli
sources:

- [DS5202-SF32LB52X English datasheet V0.2.5](https://downloads.sifli.com/user%20manual/DS5202-SF32LB52X-Datasheet%20V0p2p5.pdf),
  Figure 5-1 and Tables 5-2, 5-3, and 5-7.
- [SF32LB52X hardware design guide](https://wiki.sifli.com/en/hardware/SF32LB52B-E-G-J-HW-Application.html),
  including the power table and Figure 5-3 PCB pad reference.
- [SiFli KiCad package source at commit `234f4de6`](https://github.com/OpenSiFli/kicad-libraries/blob/234f4de6ae7311f073473393ffb0d0ca48d92a2a/footprint/no_lead_qfn.yml).
- [SiFli SiliconSchema at commit `1c17be05`](https://github.com/OpenSiFli/SiliconSchema/blob/1c17be0504aec66c2a01dcf06e2bfefca4cc87a5/chips/SF32LB52_X/chip.yaml).

Pins 1-14 and 17-69 match the pinned SiliconSchema number-for-number. That
schema revision omits package pins 15 (`VDD_VOUT2`) and 16 (`VDD_VOUT1`), but
the current datasheet pinout and hardware-guide power table independently show
both as internal LDO output/decoupling nodes. The library therefore retains
them as required `power_out` pins. SiliconSchema's semantic pad names `NC22`
and `GND` correspond to datasheet names `NC` and `GND PAD`/`EPAD` here.

## Qualified package geometry

SiFli Figure 5-3 specifies 0.20 x 0.55 mm perimeter copper, 0.30 x
0.65 mm solder-mask openings, 0.20 x 0.55 mm paste openings, and 0.35 mm
pitch. The pinned official KiCad source supplies complete component tolerances
and generates the IPC pull-back thermal-via footprint as follows:

- 68 perimeter lands: 0.55 x 0.25 mm on the left/right sides and 0.25 x
  0.55 mm on the top/bottom sides; side-axis centers at +/-3.30 mm and
  along-edge centers from -2.80 through +2.80 mm in 0.35 mm steps.
- Pin 69 exposed copper: 5.49 x 5.49 mm at the origin on front copper and a
  matching 5.49 x 5.49 mm back-copper pad.
- Nine paste-only round-rectangle apertures: 1.42 x 1.42 mm with 0.25 mm
  corner radius, centered on the Cartesian product of `{-1.83, 0, +1.83}` mm
  in X and Y (about 60.2% exposed-pad coverage).
- Sixteen netted thermal vias: 0.50 mm pad diameter, 0.20 mm drill, centered
  on the Cartesian product of `{-2.495, -0.831667, +0.831667, +2.495}` mm in
  X and Y; every via is pin 69.

`MCU_SF32LB52EUB6` reproduces this geometry directly. The EP is one continuous
top-copper land with paste suppressed; nine same-number auxiliary lands wholly
inside that copper create only the segmented stencil union. Sixteen repeated
pin-69 PTH placements and a matching bottom-copper land remain on the same
electrical pin and net. The resulting KiCad and IPC-2581 physical layers match
the official geometry without a solid 5.49 mm paste opening.
