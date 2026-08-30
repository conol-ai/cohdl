# SF32 mini-board

This example is a buildable sensor-controller reference based on the
`SF32LB52EUB6`. It uses only parts whose pin assignments and PCB lands have
been audited:

- USB-C power and native USB 2.0 data through `USBLC6-2SC6` protection;
- an always-enabled `RT9080-33GQZ` 3.3V regulator;
- an `XC6206P182MR-G` 1.8V regulator and `RS0104YUTQH12` four-channel
  translator for the BHI260AP digital domain;
- external `W25Q128JVSIQ` QSPI flash on the SF32 MPI2 mapping;
- the OpenMicroKBD v2 `H0216F002AM` 2.16-inch AMOLED/touch interface, using
  QSPI on the SF32 LCDC1 pin group and I2C for touch;
- `BHI260AP` smart IMU and `BME280` environmental sensor on I2C1;
- manufacturer-qualified 48MHz and 32.768kHz crystals; and
- a six-pin debug/calibration socket.

The display connects through the 31-contact, 0.3mm-pitch OCN
`OK-F302-31115` FPC receptacle used by OpenMicroKBD v2. LCDC1 maps RESET, TE,
CS, CLK, and D0-D3 to PA00 and PA02-PA08; this is independent of the PA12-PA17
MPI2 flash bus. The CST9220 touch controller shares the board's 3.3V I2C1 bus
and has dedicated interrupt/reset GPIOs. The module's MIPI-DSI contacts and
normal-use programming pin remain open because the SF32LB52 uses QSPI here.

The H0216F002AM cannot be powered directly from either existing rail: 5V
exceeds its 4.6V absolute maximum and 3.3V is below its 3.7V operating
minimum. An adjustable `RT6150BGQW` therefore generates 3.987V for VBAT, and
an `SGM2554A` delays the 3.3V VCI/VCI_EN rail until IOVCC is stable. The panel
rail uses the Richtek-required 10uF input and 20uF output network, while the
module pins have local high-frequency and bulk decoupling. The module sheet
does not specify normal/maximum operating current, so the voltage design is
valid but the USB current budget is not claimed as qualified at full
brightness.

The SF32 power network follows SiFli's reference schematic. In particular,
`VDD_SIP` is a decoupling-only node for this E-series device; it is not tied to
3.3V. The internal buck uses the specified 4.7uH inductor and 4.7uF feedback
capacitor. Every required pin is connected, and every unused optional pin is
explicitly marked `nc`.

The 48MHz part is SiFli's certified Hosonic `E1SB48E001G00E`: its 22ohm
maximum ESR satisfies the MCU's 30ohm ceiling. Both selected crystals have
load capacitance below SiFli's no-fitted-capacitor threshold, so the netlist
contains no crystal load capacitors; the PCB layout should still reserve
unpopulated matching-capacitor lands as the hardware guide requests.

The BHI260AP is not a 3.3V-I/O device: its VDDIO/Fuser2 operating range is
1.71-1.89V and its logic pins must not exceed VDDIO + 0.3V. The board powers
both BHI260AP supply domains from 1.8V. I2C SDA/SCL, interrupt, and reset all
cross the `RS0104` translator; the BME280 remains on the 3.3V side. Both I2C
sides have their own 4.7kohm pull-ups, and reset has a 10kohm default-high
pull-up in each voltage domain. The translator starts disabled through a
10kohm OE pull-down; firmware enables it with PA25 after both rails are stable.

The former CH343P, microSD, and bare SX1262 circuits are intentionally absent.
The external flash already consumes PA12-PA17, which are the same pins used by
the SD interface. Native USB makes the UART bridge unnecessary. A bare SX1262
also needs a board-specific RF network and cannot safely share the SF32 RF
pin. Bluetooth is disabled here for the same RF-layout reason: `BRF_ANT` is
open until a controlled-impedance antenna path and any required matching
network are designed and qualified.

The board now carries a 60 x 40mm mechanical outline with 3mm corner radii in
`mechanical/sf32-miniboard-outline.dxf`, plus four M2 non-plated mounting
holes. USB-C, the display FPC, debug connector, MCU, flash, display-power, and
crystal clusters have seeded placements. The AMOLED panel remains off-board
in the enclosure. Before fabrication, verify the connector's stagger and
pin-1 orientation against a physical `OK-F302-31115`. The remaining parts
still require placement/routing, and the design does not claim a qualified RF
antenna implementation or production-ready enclosure.
