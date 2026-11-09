# Bosch Sensortec BHI260AP — programmable smart sensor (Fuser2 MCU + 6-axis IMU), I²C / SPI host interface, LGA-44 3.6 × 4.1 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/imu::BHI260AP",
  "parts": ["@contrib/imu::IMU_BHI260AP"],
  "identities": [
    {"mfr": "Bosch Sensortec", "mpn": "BHI260AP", "lcsc": "C3288651"}
  ],
  "sources": [
    {"id": "ds", "title": "BHI260AP Datasheet — Ultra-low power, high performance, self-learning AI smart sensor with integrated accelerometer and gyroscope", "revision": "BST-BHI260AP-DS000-02, revision 1.1, 15 April 2021",
     "url": "https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bhi260ap-ds000.pdf", "sha256": "1db636a1dcdc6cf2c5da3bd4b324aa210043d6e340327f2ccaa068f39b97b5c9", "local": "lib/@contrib/imu/docs/bst-bhi260ap-ds000.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. They equal Bosch's printed "N | 173". The URL is the one in `lib/@contrib/imu/docs/README.md`. Figure numbers are the captions on the pages; the PDF's list of figures is off by one from p. 159 on (it calls the footprint "Figure 36", the page caption says "Figure 37", ds p. 9, p. 162).

## Summary

The BHI260AP combines Bosch's programmable 32-bit Fuser2 microcontroller (ARC EM4, 20 or 50 MHz, 256 KB SRAM, 144 KB ROM) with a 16-bit 3-axis accelerometer and gyroscope in a 3.6 × 4.1 × 0.83 mm LGA-44 (ds p. 2). It must be given firmware at each start, either uploaded by a host over I²C/SPI or booted from an optional QSPI flash (ds p. 35). The host interface starts in I²C (address 0x28 or 0x29, up to 3.4 MHz) and switches to SPI (up to 50 MHz) when HCSB is pulled low (ds p. 2, p. 26, p. 27). Three master interfaces connect further sensors such as a magnetometer (ds p. 16, p. 17). The I/O and core supply VDDIO is limited to 1.71–1.89 V; only VDD (sensor analog) accepts up to 3.6 V (ds p. 146).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Brown-out detection level, rising | 1.3 | — | 1.6 | V | TA −40 to 85 °C | ds p. 146 |
| VIL | — | — | 0.3 × VDDIO | V | — | ds p. 147 |
| VIH | 0.7 × VDDIO | — | — | V | — | ds p. 147 |
| VOL / VOH, aux IMU pads (ASDX, ASCX, OCSB, OSDO) | — / 0.8 × VDDIO | — | 0.2 × VDDIO / — | V | VDDIO 1.71 V, 3 mA | ds p. 147 |
| VOL / VOH, other outputs | — / 0.7 × VDDIO | — | 0.3 × VDDIO / — | V | VDDIO 1.71 V, 4 mA (LDS) or 6 mA (HDS) | ds p. 147 |
| Internal pull-up | 71 | 121 | 214 | kΩ | — | ds p. 147 |
| Internal pull-down (HSDO) | 62 | 109 | 208 | kΩ | — | ds p. 147 |
| System oscillator, Long Run / Turbo | 18.4 / 46 | 20 / 50 | 21.6 / 54 | MHz | 25 °C | ds p. 147 |
| Total current IDD + IDDIO | — | 7.3 | — | µA | Sensors suspended, Fuser2 deep sleep, 32 KB RAM on | ds p. 147 |
| Total current IDD + IDDIO | — | 184 | — | µA | Accel full operation, gyro suspended, Fuser2 deep sleep | ds p. 147, p. 148 |
| Total current IDD + IDDIO | — | 929 | — | µA | Gyro and accel full operation, Fuser2 deep sleep | ds p. 148 |
| CPU current, Long Run 20 MHz / Turbo 50 MHz | — | 0.95 / 2.8 | — | mA | Running CoreMark | ds p. 2 |
| Host I²C | — | — | 3.4 | MHz | High-speed mode | ds p. 2, p. 27 |
| Host SPI | — | — | 20 / 50 | Mbit/s | Long Run / Turbo mode | ds p. 28 |
| I²C address | — | 0x28 / 0x29 | — | — | HSDO low / high | ds p. 27 |
| RESETN pulse duration | 100 | — | — | ns | Active low | ds p. 146 |
| VREG capacitor Creg | 0.1 | 0.22 | 0.5 | µF | ESR ≤ 0.5 Ω | ds p. 146 |
| External QSPI flash size | 0.5 | — | 8 | MB | — | ds p. 16 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD | −0.3 to 4.25 | V | ds p. 146 |
| VDDIO | −0.3 to 2.75 | V | ds p. 146 |
| Any logic (non-supply) pin | −0.3 to VDDIO + 0.3 | V | ds p. 146 |
| Passive storage temperature, ≤ 65 % RH | −50 to 150 | °C | ds p. 146 |
| Mechanical shock, 200 µs half sine | 10 000 | g | ds p. 146 |
| Mechanical shock, 1.0 ms half sine | 2 000 | g | ds p. 146 |
| Free fall onto hard surfaces | 1.8 | m | ds p. 146 |
| ESD, HBM | 2 | kV | ds p. 146 |
| ESD, CDM | 500 | V | ds p. 146 |
| ESD, MM | 200 | V | ds p. 146 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD, IMU analog domain (typ 1.8 V) | 1.71 | 3.6 | V | ds p. 146 |
| VDDIO, I/O domain and Fuser2 core (typ 1.8 V) | 1.71 | 1.89 | V | ds p. 146 |
| Operating temperature | −40 | 85 | °C | ds p. 146 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | M3SDA | M3SDA | in, pull-up at reset | Master 3 I²C SDA | ds p. 19 |
| 2 | M3SCL | M3SCL | in, pull-up at reset | Master 3 I²C SCL | ds p. 19 |
| 3 | HOSTBOOT | HOSTBOOT | in, pull-up | Boot mode: high = host boot, low = standalone (flash) | ds p. 19, p. 26 |
| 4 | QSPI_D0 | QSPI_D0 | in, pull-up at reset | QSPI data 0 | ds p. 19 |
| 5 | QSPI_CLK | QSPI_CLK | in, pull-up at reset | QSPI clock | ds p. 19 |
| 6 | VREG | VREG | supply | Internal regulator output; capacitor | ds p. 19 |
| 7 | VDDIO | VDDIO | supply | Digital I/O and Fuser2 supply | ds p. 19 |
| 8 | QSPI_D3 | QSPI_D3 | in, pull-up at reset | QSPI data 3 | ds p. 19 |
| 9 | RESETN | RESETN | in, pull-up | Reset, active low | ds p. 19 |
| 10 | HIRQ | HIRQ | output (pull-up at reset) | Host interrupt | ds p. 19 |
| 11 | HSDX | HSDX | in, pull-up at reset | Host SPI MOSI / I²C SDA | ds p. 19 |
| 12 | VDDIO | VDDIO | supply | Digital I/O and Fuser2 supply | ds p. 19 |
| 13 | M2SCX | M2SCX | in, pull-up at reset | Master 2 SPI SCK / I²C SCL | ds p. 19 |
| 14 | QSPI_CSN | QSPI_CSN | in, pull-up at reset | QSPI chip select | ds p. 20 |
| 15 | QSPI_D1 | QSPI_D1 | in, pull-up at reset | QSPI data 1 | ds p. 20 |
| 16 | MCSB3 | MCSB3 | in, pull-up at reset | SPI chip select 3 / GPIO | ds p. 20 |
| 17 | GNDIO | GNDIO | ground | Digital I/O and Fuser2 ground | ds p. 20 |
| 18 | MCSB2 | MCSB2 | in, pull-up at reset | SPI chip select 2 / GPIO | ds p. 20 |
| 19 | MCSB4 | MCSB4 | in, pull-up at reset | SPI chip select 4 / GPIO | ds p. 20 |
| 20 | QSPI_D2 | QSPI_D2 | in, pull-up at reset | QSPI data 2 | ds p. 20 |
| 21 | OCSB | OCSB | Hi-Z at reset | Aux (OIS) chip select | ds p. 20 |
| 22 | ASCX | ASCX | Hi-Z at reset | Aux OIS clock / I²C SCL | ds p. 20 |
| 23 | JTAG_CLK | JTAG_CLK | in, pull-up at reset | Debug clock | ds p. 20 |
| 24 | JTAG_DIO | JTAG_DIO | in, pull-up at reset | Debug data | ds p. 20 |
| 25 | GND | GND | ground | Analog sensor ground | ds p. 20 |
| 26 | GND | GND | ground | Analog sensor ground | ds p. 20 |
| 27 | GND | GND | ground | Analog sensor ground | ds p. 20 |
| 28 | VDD | VDD | supply | Analog sensor supply | ds p. 20 |
| 29 | M1SCX | M1SCX | output low at reset | Master 1 SPI SCK / I²C SCL | ds p. 20 |
| 30 | ASDX | ASDX | Hi-Z at reset | Aux OIS MOSI / I²C SDA | ds p. 20 |
| 31 | RESV3 | RESV3 | output high at reset | Reserved, do not connect (internal IMU chip select) | ds p. 20 |
| 32 | HSDO | HSDO | in, pull-down at reset | Host SPI MISO / I²C address select | ds p. 20 |
| 33 | HSCX | HSCX | in, pull-up at reset | Host SPI SCK / I²C SCL | ds p. 20 |
| 34 | HCSB | HCSB | in, pull-up at reset | Host SPI chip select; hold high for I²C | ds p. 20 |
| 35 | M2SDX | M2SDX | in, pull-up at reset | Master 2 SPI MOSI / I²C SDA | ds p. 20 |
| 36 | GNDIO | GNDIO | ground | Digital I/O and Fuser2 ground | ds p. 20 |
| 37 | M2SDI | M2SDI | in, pull-up at reset | Master 2 SPI MISO | ds p. 20 |
| 38 | MCSB1 | MCSB1 | in, pull-up at reset | SPI chip select 1 | ds p. 20 |
| 39 | OSDO | OSDO | Hi-Z at reset | Aux OIS MISO | ds p. 20 |
| 40 | RESV2 | RESV2 | in, pull-up at reset | Reserved, do not connect (internal IMU interrupt) | ds p. 20 |
| 41 | RESV1 | RESV1 | in, pull-up at reset | Reserved, do not connect (internal IMU interrupt) | ds p. 21 |
| 42 | VDDIO | VDDIO | supply | Digital I/O and Fuser2 supply | ds p. 21 |
| 43 | M1SDI | M1SDI | in, pull-up at reset | Master 1 SPI MISO | ds p. 21 |
| 44 | M1SDX | M1SDX | output low at reset | Master 1 SPI MOSI / I²C SDA | ds p. 21 |

Pad positions, checked pad by pad: the bottom view (Figure 1, ds p. 19; Figure 33, ds p. 159) has pad 1 at the lower left, pads 1–8 up the left side, 9–14 along the top, 15–22 down the right side, 23–28 along the bottom, and an inner ring 29–33 / 34–36 / 37–41 / 42–44. Mirrored to top view and turned 180°, that is Figure 37's top view (ds p. 162): pad 1 at the upper left, 1–8 down the left side, 9–14 along the bottom, 15–22 up the right side, 23–28 along the top, inner ring 29 (upper left) down to 33, 34–36 left to right at the bottom, 37 up to 41 on the right, 42–44 right to left at the top. `LGA44P40_360X410` places every pad in that order. Positions agree; land sizes do not (see "Not established").

## Typical application

- Connection diagram (Figure 3 and Table 4, ds pp. 25–26): C1 100 nF on VDD (AVDD); C2 100 nF and C3 1 µF on VDDIO; C4 220 nF on VREG. Creg must be 0.1–0.5 µF with ESR ≤ 0.5 Ω (ds p. 146).
- I²C host mode: 4.7 kΩ pull-ups R1/R2 on HSDX and HSCX to VDDIO, R3 0 Ω from HCSB to VDDIO; do not connect the host to HCSB or HSDO; HSDO selects the address (low 0x28, high 0x29) (ds p. 25, p. 27). The host interface starts in I²C after power-up or reset; any low on HCSB switches it to SPI until the next reset (ds p. 26).
- SPI host mode: HCSB, HSCX, HSDX, HSDO; CPOL/CPHA 0/0 or 1/1; 3-wire SPI is selectable in the Host Control register (ds p. 29).
- HOSTBOOT high = host boot (firmware uploaded over the host interface); low = standalone boot from QSPI flash after power-on, external or watchdog reset (Table 11, ds p. 36). Connect HOSTBOOT and RESETN to VDDIO if not driven (Figure 3 note 1, ds p. 25; ds p. 35). Leave the JTAG pads open if unused (Figure 3 note 2, ds p. 25).
- HIRQ: active high, push-pull, level, low drive strength with the default firmware; configurable (ds p. 26).
- Master interface 1 carries the integrated IMU (SPI at 10 MHz by default) and can host further sensors (ds p. 16). M3 is I²C only (ds p. 17); its pull-ups R4/R5 are 4.7 kΩ (ds p. 25).
- RESV1–RESV3 are internal connections between the IMU and Fuser2; do not connect them (ds p. 21).

## Layout notes

- Package (Figure 33, ds p. 159): 3.6 ± 0.05 × 4.1 ± 0.05 mm, 0.83 ± 0.04 mm tall. Terminals are 0.25 mm long (perpendicular to the body edge) and 0.2 mm wide at 0.4 mm pitch, 0.09 ± 0.05 mm in from the body edge; outer terminal centres span 3.17 mm across the 3.6 mm side and 3.67 mm across the 4.1 mm side; 0.55 mm separates the outer terminals from the inner ring.
- Footprint (Figure 37, ds p. 162), measured on the embedded drawing: centre spans 3.17 mm and 3.67 mm, 0.4 mm pitch, copper lands 0.35 mm long and 0.20 mm wide (the "0.35" and "0.2" dimensions sit on the solid outline of pad 22), each inside a dashed solder-mask opening of 0.45 × 0.30 mm ("SOLDERMASK FRAME AROUND EACH PAD: 50 µm"); the inner-ring corner lands have 0.05 mm × 45° chamfers.
- Library: lands centred at x = ±1.585 mm and y = ±1.835 mm, side pads on y = ±0.2 … ±1.4 mm, inner ring on a 0.4 mm grid at ±0.8 / ±1.2 mm. The centres agree with the 3.17 / 3.67 mm spans and with the package drawing; the land size does not.
- Reflow profile: Figure 40 (ds p. 166).

## Pitfalls

- VDDIO is 1.71–1.89 V operating and 2.75 V absolute (ds p. 146). Every logic pin is limited to VDDIO + 0.3 V (ds p. 146). A 3.3 V host needs level translation.
- Firmware is not in ROM as an application: after each power-up the host must upload it, or a QSPI flash must hold it (ds p. 35).
- HOSTBOOT has an internal pull-up (ds p. 19), but Bosch recommends tying it to VDDIO if unused (ds p. 25). Pulling it low makes every power-on attempt a flash boot (ds p. 36).
- Any glitch low on HCSB after reset switches the host interface to SPI (ds p. 26).
- RESV1–RESV3 must stay unconnected (ds p. 21).

## Not established by the sources

Pad numbers, names and positions agree with the data sheet (Table 1, ds pp. 19–21; Figure 33, ds p. 159; Figure 37, ds p. 162). Disagreements and gaps, most serious first:

- **Land size (`LGA44P40_360X410`):** the library comment reads Figure 37 as "Horizontal lands are 0.45 x 0.35mm, vertical lands 0.35 x 0.45mm" and adds a 50 µm mask expansion, giving 0.55 × 0.45 mm mask openings. On the embedded Figure 37 raster (ds p. 162), the "0.35" extension lines fall on the solid copper edges of pad 22 and the "0.2" extension lines on its top and bottom copper edges; the dashed outline around each land is 0.45 × 0.30 mm, which is that 0.35 × 0.20 mm copper plus the stated 50 µm mask frame. Measured in raster pixels relative to the 0.4 mm pitch (53 px), the copper is 45 × 26 px ≈ 0.34 × 0.20 mm and the dashed opening 59 × 39 px ≈ 0.44 × 0.29 mm. The package terminals are 0.25 × 0.2 mm (Figure 33, ds p. 159). The library lands are therefore 0.10 mm longer and 0.15 mm wider than Bosch's, leave only 0.05 mm of copper between neighbours at 0.4 mm pitch (Bosch: 0.20 mm), and their 0.45 mm-wide mask openings overlap at 0.4 mm pitch, so there is no solder-mask web between pads. `examples/sf32-miniboard` uses this part. The "0.45" and "0.3" dimension lines in Figure 37 are not drawn to clear features; the reading above rests on the "0.35" and "0.2" lines and on the mask-frame note.
- **Drawing scale:** the Figure 37 raster is not uniformly to scale (3.17 mm spans 410 px, while seven 0.4 mm pitches span 372 px, i.e. 133 px/mm vs 129 px/mm), so positions were taken from the stated dimensions, not from pixels.
- **"BMI260":** `bhi260ap.cohdl` calls the integrated IMU a BMI260. The data sheet never names it; it says "6-axis IMU" (ds p. 2).
- **Library comment on M1:** it attributes the M1/IMU wiring to "p21 note 1"; that note is about the RESV pads (ds p. 21). The IMU-on-M1 statement is on p. 16.
- **HIRQ pin type:** typed `output`; the pad is a GPIO configurable as push-pull or open-drain, active high or low (ds p. 26), and is an input with pull-up during reset (ds p. 19).
- **Required pins:** the library marks HOSTBOOT, RESETN, HSDO, HCSB `required`. That matches Bosch's advice to tie HOSTBOOT and RESETN to VDDIO (ds p. 25, p. 35), but Figure 3 note 3 says not to connect the host to HSDO and HCSB in I²C mode (ds p. 25); they still need a defined level.
- **TOC vs captions:** the list of figures numbers the footprint "Figure 36" and the outline "Figure 32"; the pages say Figure 37 and Figure 33 (ds p. 9, p. 159, p. 162).
- **Not given:** thermal resistance and a stencil recommendation.
