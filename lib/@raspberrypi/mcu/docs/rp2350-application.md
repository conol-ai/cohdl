# RP2350A: support circuit and design limits

Applies to `raspberrypi_mcu::RP2350A` and
`raspberrypi_mcu::RP2350A_QFN60` in package `@raspberrypi/mcu`. The part is
RP2350A, QFN-60 with exposed ground pad 61; it has no embedded program flash.
This is an original engineering summary of the manufacturer sources below,
not a tested board, firmware implementation, or electrical signoff.

## Sources

- [D] Raspberry Pi, [RP2350 Datasheet](https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf),
  build 2025-07-29, `d126e9e-clean`; included as `rp2350-datasheet.pdf`.
- [H] Raspberry Pi, [Hardware design with RP2350](https://pip-assets.raspberrypi.com/categories/1214-rp2350/documents/RP-008280-DS-2-hardware-design-with-rp2350.pdf),
  Release 3, build 20/08/2026, `06a7f75e58a1`; included unchanged as
  `rp2350-hardware-design.pdf`. The release history is dated 24 August 2026;
  the URL's `DS-2` text does not identify the downloaded revision.

Page references below are **1-based PDF pages**, not the printed footer.
Document hashes and licensing are recorded in `docs/README.md`.

## Power connections: 3.3 V IO and internal 1.1 V core regulator

Use [D] Figure 148, PDF p.1335, for the QFN-60 pin map. The IOVDD row of
[D] Table 1432 (p.1339) conflicts with that diagram: it lists pin 54 instead
of pin 1. The library follows Figure 148: pin 1 is IOVDD; pin 54 is QSPI_IOVDD.
Do not substitute the RP2350B QFN-80 pin numbers.

| Net/function | RP2350A pins | Connection in the 3.3 V reference circuit |
| --- | --- | --- |
| IOVDD | 1, 11, 20, 30, 38, 45 | 3.3 V; connect every pin |
| QSPI_IOVDD / USB_OTP_VDD / ADC_AVDD | 54 / 53 / 44 | 3.3 V |
| DVDD | 6, 23, 39 | Regulated 1.1 V output; connect every pin |
| VREG_VIN | 49 | 3.3 V, with local 4.7 uF input capacitor to ground |
| VREG_LX | 48 | Switching node, through the qualified 3.3 uH inductor to 1.1 V |
| VREG_FB | 50 | Sense the 1.1 V output at its output capacitor |
| VREG_AVDD | 46 | 3.3 V through 33 ohm; 4.7 uF from this pin to ground |
| VREG_PGND / exposed GND | 47 / 61 | Ground with the regulator's specified return layout |

The 4.7 uF output capacitor connects **after the inductor**, from 1.1 V to
ground. Never short LX directly to DVDD. [H] Figure 3, p.6, shows the
regulator and the 33 ohm / 4.7 uF analog-supply filter; its capacitors are
6.3 V X5R. Check effective capacitance, tolerance and parasitics of the actual
parts. [D] section 6.3.8.2, p.456, requires input capacitance at least 4.7 uF
with resistance at most 50 milliohm, output capacitance 4.7 uF +/-20% with
resistance at most 250 milliohm and inductance at most 6 nH.

The regulator inductor is a critical selection, not a generic value match.
[D] section 6.3.8.2 and [H] section 2.1, pp.6-8, specify/recommend a fully shielded,
orientation-marked 3.3 uH +/-20% part, DC resistance at most 250 milliohm,
saturation current at least 1.5 A, and the prescribed winding orientation.
The recommended MPN is **Abracon AOTA-B201610S3R3-101-T**. Another library
inductor with the same nominal value is not thereby a qualified substitute.
Use [D] Figures 23-25 and [H] Figures 3-4 for the layout and orientation.
Keep switching-current loops short and away from VREG_AVDD; sense FB from
the output capacitor, not the LX trace. [D] also recommends an additional
4.7 uF on DVDD near pin 23, away from LX and the primary output capacitor.
These requirements must be preserved in the eventual PCB layout.

Place 100 nF close to each supply pin, including each DVDD pin. The example
in [H] section 2.2.1, pp.9-10, shares one 100 nF between adjacent pins 53/54
because of space; this is a specific placement compromise, not permission
to omit core decoupling. Connect and decouple the analog and USB/OTP supplies
in this reference circuit. The library groups all six physical IOVDD pins under
one logical `IOVDD` pin and all three DVDD pins under `DVDD`; this connects
the physical supply pins but does not instantiate their capacitors. Audit
the six IOVDD locations and three DVDD locations individually. A single
100 nF capacitor on a grouped net is not a per-pin bypass implementation.
USB_OTP_VDD must remain powered even without USB,
and VREG_AVDD also powers reset/brownout circuitry; see [D] section 6.1
for supply and sequencing requirements.

[D] Table 1441, pp.1344-1345: IOVDD and QSPI_IOVDD permit 1.62-3.63 V,
DVDD 1.05-1.16 V (nominal 1.1 V), VREG_VIN 2.7-5.5 V, and VREG_AVDD and
USB_OTP_VDD 3.135-3.63 V. ADC_AVDD permits 1.62-3.63 V, but ADC performance
degrades below 2.97 V. The reference circuit uses 3.3 V for all of these
except DVDD. VREG_VIN's wider range does not apply to the other supplies.

## External flash and boot/debug access

For normal execute-in-place boot, fit a compatible external QSPI flash.
RP2350A does not contain the flash present in RP2354 variants. Connect:
SS=60, SCLK=56, SD0=57, SD1=59, SD2=58, SD3=55; use the selected flash's
actual pinout and voltage, short traces, and its local supply decoupling.
[H] section 3.1 and Figure 8, pp.11-13, show W25Q128JVSIQ with 100 nF and
the supported boot-read commands. Different flash parts require their own
startup and command-compatibility checks.

Provide BOOTSEL access from QSPI_SS through **1 kohm** to a switch/test
connection to ground, sampled during boot. Keep that resistor: it allows
the MCU to drive SS after boot. Figure 8's optional 10 kohm SS pull-up is
DNF for the reference flash; decide whether to populate it using the actual
flash's power-up requirements. Do not assume all flash devices behave alike.

RUN pin 26 is active-low reset. Expose RUN and ground for reset access, and
SWCLK=24, SWDIO=25 plus ground for debugging. Buttons are one implementation;
accessible pads or a header can provide the same required access in a draft.
See [H] sections 5.3-5.4, pp.18-19. Determine connector geometry separately.

## Crystal and USB data interface

[H] Figure 10, p.14, uses **Abracon ABM8-272-T3, 12 MHz** (existing library
part `osc::XTAL_12MHZ`). XIN=21 connects to one crystal terminal; XOUT=22
connects through **1 kohm** to the other. Each crystal terminal has **15 pF
C0G** to ground; ground its case pads as shown. With the guide's estimated
3 pF board/parasitic capacitance, 15 pF / 2 + 3 pF = 10.5 pF versus the
crystal's 10 pF load rating. This is a starting circuit for the named crystal
and 3.3 V IO. Verify load, startup, drive and frequency error for the chosen
crystal/layout; the example is not a universal ppm guarantee.

USB_DP=52 and USB_DM=51 each need a **27 ohm series resistor near the MCU**.
The PHY supplies the USB data-line pull-up/pull-down functions internally;
do not add external USB data-line pull resistors. See [H] section 5.1 and
Figure 11, p.16. Full-speed is **12 Mbit/s**, not 12 Mbyte/s. Route for
90 ohm differential impedance over a continuous ground reference using the
actual stackup; the guide's trace widths are specific to its example PCB.
Connector CC termination, VBUS handling and ESD are separate circuit duties.

## Power budget: distinguish example measurements from limits

[D] Table 1446, p.1348, reports typical examples at room temperature, with
3.3 V supplies, internal 1.1 V DVDD, the specified external flash and USB
connected to a host. For `hello_usb`, the six reported input-rail currents
are 14.700 + 0.216 + 0.453 + 0.022 + 0.001 + 0.570 = **15.962 mA**
(52.7 mW). This is neither a guaranteed maximum nor a measurement of a
complete application board. The USB boot table's 6.050 mA peak is the
USB_OTP_VDD rail only, not the whole MCU.

Do not turn those examples into a claimed worst-case 30 mA MCU budget.
Budget the selected clocks, cores, firmware workload, flash, external GPIO
loads, receiver/interface ICs, conversion losses, temperature and margin.
The internal regulator's 200 mA normal-mode **output** limit ([D] Table 1442)
is not the MCU's current drawn from the 3.3 V board rail. A chosen upstream
LDO also needs its own input range, dissipation and stability review.

## ADC, powered-off behavior and capture firmware

ADC0-3 are GPIO26-29, package pins 40-43. They are digital/analog pads,
not the fault-tolerant digital pad category. [D] section 12.4, p.1069,
requires disabling their digital input/output functions (IE=0, OD=1) for
analog use. ADC input limits track **IOVDD**, not just ADC_AVDD: overvoltage
can inject current through protection structures. Respect the operating
input range, including when the MCU rail is absent; absolute maximum ratings
are stress limits. A high-value divider alone does not establish no-backfeed
or powered-off isolation for a separately powered VBUS monitor. Determine
divider tolerance, ADC acquisition/loading, protection and error budget for
the actual circuit. Do not connect USB PD VBUS or CC directly to these pins.

[D] erratum **RP2350-E9**, pp.1367-1369, affects **A2** silicon and is marked
fixed in **A3**. With a GPIO input buffer enabled, output disabled and the
pad in the undefined logic region, the faulty pad can source substantial
current (about 120 uA in the cited typical 3.3 V case). Disabling the digital
input removes that condition. An external pull of 8.2 kohm or less is one
documented workaround for digital inputs, but would load a high-impedance
measurement. The package MPN alone does not pin the silicon stepping:
confirm purchased silicon and review applicable errata. This workaround
does not prove safe powered-off sensing or a passive CC tap.

For a USB PD analyzer, RP2350's USB data PHY is not a CC/PD BMC receiver.
A separate qualified front end, its reset/unpowered behavior, and firmware
are required. RP2350 offers three PIO blocks / twelve state machines, but
that resource count alone proves neither capture timing nor lossless USB
streaming. Specify clocking, pin mux, interrupt/DMA service, buffering,
timestamping, decoding, overflow reporting and USB transport. Passive
non-negotiating behavior, orientation/VCONN handling and loading remain
design review obligations; a compiler pass cannot establish them.
