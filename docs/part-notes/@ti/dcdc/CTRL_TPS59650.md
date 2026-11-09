# TPS59650RSLR — IMVP-7 SVID dual-channel (3-phase CPU / 2-phase GPU) D-CAP+ step-down controller, 48-pin VQFN 6×6

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@ti/dcdc::TPS59650",
  "parts": ["@ti/dcdc::CTRL_TPS59650"],
  "identities": [
    {"mfr": "Texas Instruments", "mpn": "TPS59650RSLR", "lcsc": "C129769"}
  ],
  "sources": [
    {"id": "ds", "title": "TPS51650, TPS59650 Dual-Channel (3-Phase CPU/2-Phase GPU) SVID, D-CAP+ Step-Down Controller for IMVP-7 VCORE with Two Integrated Drivers", "revision": "SLUSAV7, January 2012 (package option addendum dated 31-Oct-2025; RSL drawings 4207548/B, 4207841-3/P, 4209046-2/O)",
     "url": "https://www.ti.com/lit/ds/symlink/tps59650.pdf", "sha256": "5afb9e1c3ff9da14259014c3315bc56cb099ed725d6c8f20aadcca3b1463cc68", "local": "lib/@ti/dcdc/docs/tps59650-datasheet.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages. Pages 1–43 equal the printed page numbers. Pages 44–47 are the package option addendum and packing information, pages 48–51 the RSL package drawings (no printed page numbers).

## Summary

The TPS59650 is an Intel IMVP-7 VCORE controller: one CPU channel of one to three phases and one GPU channel of one or two phases, both set by an 8-bit SVID DAC from 0.250 V to 1.52 V (ds p. 1). It has two integrated gate drivers for CPU phases 1 and 2. CPU phase 3 and the GPU phases need external drivers such as the TPS51601 (ds p. 1, p. 34). The conversion input is 3–28 V, with 5 V and 3.3 V bias supplies (ds p. 1, p. 3). It is not a general-purpose buck: the output voltage comes from SVID commands, so it needs an IMVP-7 SVID host (ds p. 1, p. 23). The datasheet covers the TPS51650 too; the TPS59650 is the -40 °C to 105 °C version with a selectable SVID address (ds p. 1, p. 3).

## Key parameters

Over the recommended free-air temperature range, VV5 = VV5DRV = 5.0 V, VV3R3 = 3.3 V, unless stated (ds p. 4). Rows that differ between TPS51650 and TPS59650 give the TPS59650 values.

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| DAC output range | 0.250 | — | 1.52 | V | 8-bit SVID DAC | ds p. 1 |
| VID step | — | 5 | — | mV | — | ds p. 4 |
| xVFB tolerance | -7.5 | — | 7.5 | mV | 0.25–0.995 V, IxPU_CORE = 0 A, -40 to 105 °C | ds p. 4 |
| xVFB tolerance | -0.75 | — | 0.75 | % | 1.000–1.520 V, IxPU_CORE = 0 A, -40 to 105 °C | ds p. 4 |
| V5 supply current IV5 + IV5DRV | — | 7.5 | 11.0 | mA | CPU 3-phase and GPU 2-phase active, VR_ON high | ds p. 4 |
| V5 + V5DRV standby current | — | 10 | 20 | µA | VR_ON low | ds p. 4 |
| V5 UVLO OK threshold | 4.25 | 4.35 | 4.50 | V | Ramp up | ds p. 4 |
| V5 UVLO fault threshold | 3.95 | 4.10 | 4.30 | V | Ramp down | ds p. 4 |
| V3R3 supply current | — | 0.5 | 1.0 | mA | SVID bus idle, VR_ON high | ds p. 4 |
| V3R3 standby current | — | — | 10 | µA | VR_ON low | ds p. 4 |
| V3R3 UVLO OK threshold | 2.5 | 2.9 | 3.0 | V | Ramp up | ds p. 4 |
| V3R3 UVLO fault threshold | 2.4 | 2.7 | 2.8 | V | Ramp down | ds p. 4 |
| VREF output | 1.655 | 1.700 | 1.745 | V | 4.5 V ≤ VV5 ≤ 5.5 V, IVREF = 0 A | ds p. 4 |
| OCP valley voltage | 3.9 | 7.0 | 9.2 | mV | RxOCP-R = 20 kΩ (lowest of 8 settings) | ds p. 5 |
| OCP valley voltage | 45.6 | 49.0 | 51.9 | mV | RxOCP-R = 150 kΩ (highest of 8 settings) | ds p. 5 |
| IMAX value at VIMAX = 133 mV / 653 mV | — | 20 / 98 | — | A | VIMAX = VREF × IMAX / 255 | ds p. 5 |
| Droop amplifier transconductance | 480 | 497 | 518 | µS | xVFB = 1 V | ds p. 5 |
| Internal current-sense gain | 11.65 | 12.00 | 12.30 | V/V | CSPx − CSNx to PWM comparator | ds p. 5 |
| Start-up time | — | — | 5 | ms | VR_ON to SVID response, 12 mV/µs | ds p. 6 |
| Soft-start / soft-stop slew | 1.25 | 1.50 | 1.75 | mV/µs | SLEWRATE = 12 mV/µs | ds p. 6 |
| CPU on-time | 298 | 317 | 340 | ns | RCF = 20 kΩ (250 kHz), VBAT = 12 V, VDAC = 1.1 V | ds p. 6 |
| CPU on-time | 127 | 140 | 160 | ns | RCF = 150 kΩ (600 kHz), VBAT = 12 V, VDAC = 1.1 V | ds p. 6 |
| GPU on-time | 280 | 323 | 377 | ns | RGF = 20 kΩ (275 kHz), VBAT = 12 V, VDAC = 1.1 V | ds p. 7 |
| GPU on-time | 134 | 157 | 193 | ns | RGF = 150 kΩ (660 kHz), VBAT = 12 V, VDAC = 1.1 V | ds p. 7 |
| Minimum off time | — | 150 | 225 | ns | Fixed | ds p. 7 |
| Fixed OVP threshold | 1.67 | 1.72 | 1.77 | V | VCSN1 or VGCSN above it for 1 µs | ds p. 7 |
| xPGOOD high threshold | 190 | 220 | 245 | mV | At xVFB relative to the VID code; latches off | ds p. 7 |
| xPGOOD low threshold | -348 | -315 | -278 | mV | At xVFB relative to the VID code; latches off | ds p. 7 |
| Internal thermal shutdown / hysteresis | — | 155 / 20 | — | °C | Specified by design | ds p. 7 |
| VCLK, VDIO input logic high VIH | 0.65 | — | — | V | — | ds p. 8 |
| VCLK, VDIO input logic low VIL | — | — | 0.45 | V | — | ds p. 8 |
| VR_ON logic high | 0.8 | — | — | V | — | ds p. 8 |
| VR_ON logic low | — | — | 0.3 | V | — | ds p. 8 |
| SVID open-drain pull-down | 4 | 8 | 13 | Ω | VDIO, ALERT, VR_HOT at 0.31 V | ds p. 8 |
| xPGOOD pull-down | — | 36 | 50 | Ω | At 0.31 V | ds p. 8 |
| DRVH on-resistance, high / low state | — | 1.2 / 0.8 | 2.5 / 2.5 | Ω | (VCBSTx − VCSWx) = 5 V | ds p. 9 |
| DRVL on-resistance, high / low state | — | 0.9 / 0.4 | 2 / 1 | Ω | — | ds p. 9 |
| xPWM, xSKIP output high level | 4.2 | — | — | V | — | ds p. 9 |
| xPWM, xSKIP output low level | — | — | 0.3 | V | — | ds p. 9 |
| θJA / θJCbot, RSL 48 pins | — | 31.7 / 2.1 | — | °C/W | — | ds p. 2 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VBAT | -0.3 to 32 | V | ds p. 2 |
| CSW1, CSW2 | -6.0 to 32 | V | ds p. 2 |
| CDH1 to CSW1, CDH2 to CSW2, CBST1 to CSW1, CBST2 to CSW2 | -0.3 to 6.0 | V | ds p. 2 |
| CTHERM, CCOMP, CF-IMAX, GF-IMAX, GCOMP, GTHERM, V5DRV, V5 | -0.3 to 6.0 | V | ds p. 2 |
| COCP-R, CCSP1–3, CCSN1–3, CVFB, CGFB, V3R3, VR_ON, VCLK, VDIO, SLEWA, GGFB, GVFB, GCSN1, GCSP1, GOCP-R | -0.3 to 3.6 | V | ds p. 2 |
| PGND | -0.3 to 0.3 | V | ds p. 2 |
| VREF | -0.3 to 1.8 | V | ds p. 2 |
| Outputs CPGOOD, ALERT, VR_HOT, GPGOOD | -0.3 to 3.6 | V | ds p. 2 |
| Outputs CPWM3, GPWM1, GPWM2, GSKIP, CDL1, CDL2 | -0.3 to 6.0 | V | ds p. 2 |
| ESD, HBM | 2 | kV | ds p. 2 |
| ESD, CDM | 500 | V | ds p. 2 |
| Operating junction temperature (listed among absolute maximums) | -40 to 125 | °C | ds p. 2 |
| Storage temperature | -55 to 150 | °C | ds p. 2 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VBAT | -0.1 | 28 | V | ds p. 3 |
| CSW1, CSW2 | -3.0 | 30 | V | ds p. 3 |
| CDHx to CSWx, CBSTx to CSWx | -0.1 | 5.5 | V | ds p. 3 |
| V5DRV, V5 | 4.5 | 5.5 | V | ds p. 3 |
| V3R3 | 3.1 | 3.5 | V | ds p. 3 |
| CCOMP, GCOMP | -0.1 | 2.5 | V | ds p. 3 |
| CTHERM, GTHERM | 0.1 | 3.6 | V | ds p. 3 |
| CF-IMAX, GF-IMAX, COCP-R, GOCP-R | 0.1 | 1.7 | V | ds p. 3 |
| Current-sense and voltage-sense pins (CCSPx, CCSNx, CVFB, CGFB, GGFB, GVFB, GCSNx, GCSPx) | -0.1 | 1.7 | V | ds p. 3 |
| VR_ON, VCLK, VDIO, SLEWA | -0.1 | 3.5 | V | ds p. 3 |
| PGND | -0.1 | 0.1 | V | ds p. 3 |
| VREF | -0.1 | 1.72 | V | ds p. 3 |
| CPGOOD, ALERT, VR_HOT, GPGOOD output | -0.1 | VV3R3 | V | ds p. 3 |
| CPWM3, GPWM1, GSKIP, CDL1, CDL2 output | -0.1 | VV5 | V | ds p. 3 |
| Operating free-air temperature, TPS59650 | -40 | 105 | °C | ds p. 2, p. 3 |

## Pins

Top view: pins 1–12 down the left side, 13–24 along the bottom left to right, 25–36 up the right side, 37–48 along the top right to left, thermal pad in the centre (ds p. 10). The drawing is labelled TPS51650; the datasheet uses the same pinout for both devices (ds p. 10).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | CTHERM | CTHERM | bidirectional (I/O) | Thermal sensor for the CPU channel: resistor to VREF, NTC to GND | ds p. 10, p. 11 |
| 2 | COCP_R | COCP-R | input (I) | Resistor to GND selects 1 of 8 CPU OCP levels; its voltage sets CPU OSR/USR level | ds p. 10, p. 11 |
| 3 | CF_IMAX | CF-IMAX | input (I) | Resistor to GND sets CPU frequency; divider voltage from VREF sets CPU IMAX; latched at start-up | ds p. 10, p. 11 |
| 4 | CCSP1 | CCSP1 | input (I) | CPU phase 1 current sense, positive | ds p. 10 |
| 5 | CCSN1 | CCSN1 | input (I) | CPU phase 1 current sense, negative; has a secondary OVP comparator | ds p. 10 |
| 6 | CCSN2 | CCSN2 | input (I) | CPU phase 2 current sense, negative (order reversed against phase 1) | ds p. 10, p. 40 |
| 7 | CCSP2 | CCSP2 | input (I) | CPU phase 2 current sense, positive; tie to V3R3 to disable the phase | ds p. 10 |
| 8 | CCSP3 | CCSP3 | input (I) | CPU phase 3 current sense, positive; tie to V3R3 to disable the phase | ds p. 10 |
| 9 | CCSN3 | CCSN3 | input (I) | CPU phase 3 current sense, negative | ds p. 10 |
| 10 | CCOMP | CCOMP | output (O) | CPU GM error-amplifier output; resistor to VREF sets droop | ds p. 10 |
| 11 | CVFB | CVFB | input (I) | CPU voltage sense to VCORE; soft-stop transistor on this pin | ds p. 10, p. 11 |
| 12 | CGFB | CGFB | input (I) | CPU voltage-sense return; 10 Ω to GND when no processor is fitted | ds p. 10, p. 11 |
| 13 | GOCP_R | GOCP-R | input (I) | Resistor to GND selects 1 of 8 GPU OCP levels; its voltage sets GPU OSR/USR level | ds p. 10, p. 11 |
| 14 | VREF | VREF | output (O) | 1.7 V, 500 µA reference; bypass to GND | ds p. 10, p. 11 |
| 15 | V3R3 | V3R3 | input (I) | 3.3 V supply input; ≥ 1 µF to GND | ds p. 10, p. 11 |
| 16 | VR_ON | VR_ON | input (I) | IMVP-7 enable, 1 V I/O level; low = soft-stop | ds p. 10, p. 11 |
| 17 | CPGOOD | CPGOOD | output (O) | CPU power good, open drain | ds p. 10, p. 11 |
| 18 | VCLK | VCLK | input (I) | SVID clock, 1 V logic | ds p. 10, p. 11 |
| 19 | ALERT | ALERT | output (O) | SVID interrupt, open drain | ds p. 10 |
| 20 | VDIO | VDIO | bidirectional (I/O) | SVID data, 1 V logic | ds p. 10, p. 11 |
| 21 | VR_HOT | VR_HOT | output (O) | Thermal flag, open drain, active low | ds p. 10, p. 11 |
| 22 | SLEWA | SLEWA | input (I) | Voltage sets slew rate; resistor to GND sets the TPS59650 base SVID address; latched | ds p. 10, p. 11 |
| 23 | GPGOOD | GPGOOD | output (O) | GPU power good, open drain | ds p. 10, p. 11 |
| 24 | GF_IMAX | GF-IMAX | input (I) | Resistor to GND sets GPU frequency; divider voltage from VREF sets GPU IMAX; latched | ds p. 10, p. 11 |
| 25 | GGFB | GGFB | input (I) | GPU voltage-sense return; 10 Ω to GND when no processor is fitted | ds p. 10, p. 11 |
| 26 | GVFB | GVFB | input (I) | GPU voltage sense to VGFX; soft-stop transistor on this pin | ds p. 10, p. 11 |
| 27 | GCOMP | GCOMP | output (O) | GPU gM error-amplifier output; resistor to VREF sets droop | ds p. 10, p. 11 |
| 28 | GCSN1 | GCSN1 | input (I) | GPU phase 1 current sense, negative | ds p. 10, p. 11 |
| 29 | GCSP1 | GCSP1 | input (I) | GPU phase 1 current sense, positive | ds p. 10, p. 11 |
| 30 | GCSP2 | GCSP2 | input (I) | GPU phase 2 current sense, positive; tie to V3R3 to disable the phase | ds p. 10, p. 11 |
| 31 | GCSN2 | GCSN2 | input (I) | GPU phase 2 current sense, negative | ds p. 10, p. 11 |
| 32 | GTHERM | GTHERM | bidirectional (I/O) | Thermal sensor for the GPU channel: resistor to VREF, NTC to GND | ds p. 10, p. 11 |
| 33 | GSKIP | GSKIP | output (O) | Skip control to the external GPU driver, 5 V logic: high = FCCM, low = skip | ds p. 10, p. 11 |
| 34 | GPWM1 | GPWM1 | output (O) | PWM to external GPU driver, 5 V logic | ds p. 10, p. 11 |
| 35 | GPWM2 | GPWM2 | output (O) | PWM to external GPU driver, 5 V logic | ds p. 10, p. 11 |
| 36 | CPWM3 | CPWM3 | output (O) | PWM to external CPU phase-3 driver, 5 V logic | ds p. 10, p. 11 |
| 37 | VBAT | VBAT | input (I) | Input-voltage sense for the on-time circuits; 10 kΩ series resistor | ds p. 10, p. 11 |
| 38 | CDH2 | CDH2 | output (O) | CPU phase 2 high-side gate drive | ds p. 10 |
| 39 | CBST2 | CBST2 | input (I) | CPU phase 2 bootstrap | ds p. 10 |
| 40 | CSW2 | CSW2 | bidirectional (I/O) | CPU phase 2 high-side gate return (switch node) | ds p. 10, p. 11 |
| 41 | CDL2 | CDL2 | output (O) | CPU phase 2 low-side gate drive | ds p. 10 |
| 42 | PGND | PGND | ground (–) | Gate-driver ground; to the V5DRV decoupling capacitor, not directly to the thermal pad | ds p. 10, p. 11, p. 43 |
| 43 | V5DRV | V5DRV | input (I) | Gate-driver supply; ≥ 2.2 µF ceramic | ds p. 10, p. 11 |
| 44 | CDL1 | CDL1 | output (O) | CPU phase 1 low-side gate drive | ds p. 10 |
| 45 | CSW1 | CSW1 | bidirectional (I/O) | CPU phase 1 high-side gate return (switch node) | ds p. 10, p. 11 |
| 46 | CBST1 | CBST1 | input (I) | CPU phase 1 bootstrap | ds p. 10 |
| 47 | CDH1 | CDH1 | output (O) | CPU phase 1 high-side gate drive | ds p. 10 |
| 48 | V5 | V5 | input (I) | 5 V analog supply, through a resistor from the 5 V plane; ≥ 1 µF ceramic | ds p. 10, p. 11 |
| 49 | EP | PAD (thermal pad, named GND in the pin table) | ground | Thermal pad and analog ground; to a quiet ground island, not directly to PGND | ds p. 12, p. 40, p. 43 |

## Typical application

- Supplies: V5 through a resistor from the 5 V plane with ≥ 1 µF ceramic; V5DRV with ≥ 2.2 µF ceramic to PGND; V3R3 with ≥ 1 µF ceramic (ds p. 11, p. 43). VREF: 0.22 µF ceramic to GND (ds p. 11) or 0.33 µF to AGND (ds p. 43); see "Not established".
- VBAT: through a 10 kΩ series resistor from the conversion input; it feeds only the on-time circuits (ds p. 11).
- Power-up: apply 5 V and 3.3 V, then raise VR_ON to the VCCIO level (1 V I/O). Frequency, IMAX, OCP level, OSR/USR level, slew rate and base SVID address are latched then and cannot be changed during operation (ds p. 11, p. 24).
- Frequency: resistor RCF from CF-IMAX to GND (CPU) and RGF from GF-IMAX to GND (GPU), 20–150 kΩ in eight steps (ds p. 6, p. 7, p. 24).
- ICC(max): a resistor from VREF to CF-IMAX (or GF-IMAX) forms a divider with RCF (or RGF): ICC(max) = 255 × RCF / (RCF + RCIMAX) (ds p. 35).
- OCP: resistor from COCP-R (GOCP-R) to GND, 20–150 kΩ, selects one of eight valley limits; the pin voltage also sets the OSR/USR level (ds p. 5, p. 8, p. 11, p. 24).
- SLEWA: its voltage sets the SetVID-fast slew rate (4–26 mV/µs); for the TPS59650 a resistor to GND sets the base SVID address (ds p. 6, p. 11, p. 24).
- Unused phases: tie CCSP3 (then CCSP2) to V3R3 and the matching CCSN pins to GND; GCSP2 to V3R3 and GCSN2 to GND for one GPU phase; CCSP1 or GCSP1 to V3R3 to switch a whole channel off (ds p. 10, p. 11, p. 24).
- Sense lines: CVFB/GVFB to the processor's VCC sense and CGFB/GGFB to its VSS sense; add 10 Ω to VCORE/VGFX and to GND to close the loop when no processor is fitted (ds p. 11, p. 40).
- Thermal: an NTC from CTHERM (GTHERM) to GND and a resistor from VREF to the same pin; a small filter capacitor may be added (ds p. 11, p. 35).
- Open-drain outputs: VR_HOT is typically pulled up to the 1 V rail through 56 Ω (ds p. 11). CPGOOD, GPGOOD and ALERT are open drain (ds p. 10, p. 11).
- External drivers: CPWM3 drives a TPS51601 for CPU phase 3; GPWM1/GPWM2 and GSKIP drive the GPU drivers (ds p. 1, p. 11).
- Recommended power-stage parts (FETs, inductors, capacitors, NTCs, sense resistors) are listed in Table 1 (ds p. 23).

## Layout notes

- Separate the noisy driver interface lines from the sensitive analog interface lines; TI calls this the most critical layout rule (ds p. 40). The pinout groups the driver pins away from the analog and digital pins (Figure 52, ds p. 40).
- The thermal pad is the analog ground. Do not connect it directly to PGND (pin 42). Tie it to a ground island with at least four small vias or one large via, and join that island to a quiet spot of system ground at one point (ds p. 40, p. 43).
- Connect PGND directly to the ground terminal of the V5DRV decoupling capacitor (ds p. 40, p. 43).
- Route current-sense pairs differentially in a quiet layer with Kelvin connections at the sense element; place DCR-sense capacitors close to the CS pins (ds p. 41).
- Keep the low-side gate drive short (1 inch or less), width-to-length 1:10 or wider, and use at least two vias per layer change (ds p. 42).
- Place parts close to the device in this order: CS filters, COMP parts, VREF/V3R3/V5 decoupling, THERM filter, OCP-R resistors, F-IMAX resistors (ds p. 42).
- RSL land pattern (ds p. 51): 48 lands 0.85 × 0.20 mm on 0.40 mm pitch, 6.8 mm overall, 5.1 mm between opposite land rows; thermal land 4.05 × 3.60 mm with an example array of 16 Ø0.2 mm vias on a 1.0 mm grid; 0.125 mm stencil with 71% coverage on the thermal pad.
- Package: body 5.85–6.15 mm square, height 0.80–1.00 mm, terminal length 0.30–0.50 mm, width 0.14–0.26 mm (ds p. 49). Exposed pad 4.05 ± 0.10 mm by 3.60 ± 0.10 mm with a C0.30 pin-1 chamfer (ds p. 50).
- **Thermal-pad orientation.** The 4.05 mm side runs parallel to the pins 1–12 and 25–36 sides, and the pad sits 1.20 mm (nominal) from the package edge on the pins 13–24 side (ds p. 50). In the land pattern the gaps from the thermal land to the land rows are 0.30 mm on the pins 37–48 side and 0.75 mm on the other three sides (ds p. 51). The @ti/dcdc ≤0.1.x land does not follow this; @ti/dcdc 0.2.0 (PR #94) does. See "Not established".

## Pitfalls

- **Thermal pad and PGND are separate grounds.** The pad is the controller's only analog ground connection (ds p. 12, p. 43). The library calls it `EP` with type `passive`; connect it to the analog ground island, not straight to pin 42 (ds p. 40, p. 43). The @ti/dcdc ≤0.1.x comment says only "-> GND"; 0.2.0 (PR #94) carries this rule in the `EP` comment.
- **Phase-2 current-sense order is reversed.** CCSN2 is pin 6 and CCSP2 pin 7, unlike phases 1 and 3 (ds p. 10, p. 40). Check sense polarity for every phase (ds p. 40).
- **Settings are latched at VR_ON.** Changing the CF-IMAX, GF-IMAX, COCP-R, GOCP-R or SLEWA networks after enable has no effect until VR_ON is cycled (ds p. 24).
- **No UVLO on VBAT.** Only V5, V5DRV and V3R3 are monitored; the converter can run with VBAT as low as about 3 × VCORE (ds p. 34).
- **Logic levels.** VR_ON, VCLK and VDIO are 1 V level inputs (ds p. 8, p. 11). CPGOOD, ALERT, VR_HOT and GPGOOD must not be pulled above V3R3 (ds p. 3). CPWM3, GPWM1/2 and GSKIP are 5 V logic outputs (ds p. 11).
- **Faults latch.** UVP after OCP, OVP and thermal shutdown all hold the converter off until VR_ON is cycled (ds p. 34, p. 35).
- **Valley current limit.** The DC current at OCP is the OCP value plus half the ripple current (ds p. 34).
- **Two integrated drivers only.** CPU phase 3 and the GPU phases need external drivers (ds p. 1, p. 34).

## Not established by the sources

- **@ti/dcdc ≤0.1.x: thermal land rotated and centred (library footprint); fixed in @ti/dcdc 0.2.0 (PR #94).** In ≤0.1.x the part uses `qfn::QFN48N40P600X600_1EP405X360`, which places pins 1–12 down the left side and a 4.05 mm (x) × 3.6 mm (y) thermal land centred at (0, 0). With pins 1–12 vertical, the 4.05 mm side must run vertically (ds p. 50), and the land must be offset 0.225 mm towards the pins 37–48 row (ds p. 50, p. 51). So that land is rotated 90° and not offset. In the library's top-view coordinates, TI's land spans x ±1.80 mm and y -2.25 to +1.80 mm. The ≤0.1.x land spans x ±2.025 mm and y ±1.80 mm. It leaves a 0.45 × 3.6 mm strip of the package pad unlanded on the pins 37–48 side and overhangs the package pad by 0.225 mm towards pins 1–12 and 25–36, where the land-to-land gap shrinks from 0.75 to 0.525 mm. The comment says "Drawing +Y is negated" but does not mention a rotation. Checked on the rendered ds pp. 50–51. @ti/dcdc 0.2.0 moves the part to the package-owned `TI_RSL_S_PVQFN_N48`: the same 48 perimeter lands and a 3.60 (x) × 4.05 (y) mm thermal land centred at (0, −0.225) mm, matching TI's land above. `qfn::QFN48N40P600X600_1EP405X360` itself is unchanged in qfn 0.2.1 and has no other user; qfn 0.3.0 removes it.
- **Thermal-pad pin.** The datasheet names the pad "PAD" with pin "GND" and function "Thermal pad and analog circuit reference" (ds p. 12). The library pin is `EP` of type `passive`. In @ti/dcdc ≤0.1.x its comment is "exposed thermal PowerPAD -> GND", which does not say analog ground or warn against tying it to PGND (ds p. 40, p. 43); @ti/dcdc 0.2.0 (PR #94) states both.
- **Pin names.** The library spells COCP-R, CF-IMAX, GOCP-R and GF-IMAX with underscores. All 48 names otherwise match ds p. 10–11.
- **Pin types the library cannot express.** ALERT, CPGOOD, GPGOOD and VR_HOT are open-drain outputs (ds p. 10, p. 11). The library types them `output`.
- **Header comment.** `multiphase.cohdl` calls the part "the headline controller of TI reference design TIDA-00021". That design is not in the source.
- **Footprint omissions** (acknowledged in the library comment): the 16-via array and the segmented stencil of ds p. 51 are not encoded. The C0.30 pin-1 chamfer on the exposed pad (ds p. 50) is not drawn.
- **Datasheet conflicts:**
  - VREF bypass is 0.22 µF on ds p. 11 and 0.33 µF on ds p. 43.
  - Features and Simplified Application show a two-phase GPU with TPS51601 drivers (ds p. 1); the gate-driver section speaks of "the single-phase GPU channel" (ds p. 34).
  - The absolute-maximum list names GCSN1 and GCSP1 but not GCSN2 and GCSP2 (ds p. 2). The recommended output list names GPWM1 but not GPWM2 (ds p. 3).
  - VTHERM bit 6 prints min 462 mV, typ 455 mV, max 514 mV (ds p. 7), with typical below minimum; Table 5 gives 458 mV (ds p. 35).
  - The ICC(max) section names "The TPS51640 controller" (ds p. 35).
  - The pin drawing labels pin 21 "VR)HOT" and the die "TPS51650" (ds p. 10).
- **Not given:** limits of conversion efficiency, a reference schematic with values for a complete TPS59650 design (only a key-part list, ds p. 23), thermal-via plating and solder-mask tolerances (left to the fabricator, ds p. 51), and a recommended courtyard.
