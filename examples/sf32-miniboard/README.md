# SF32 mini-board

This example is a buildable sensor-controller reference based on the
`SF32LB52EUB6`. It uses only parts whose pin assignments and PCB lands have
been audited:

- USB-C power and native USB 2.0 data through `USBLC6-2SC6` protection;
- an always-enabled `RT9080-33GQZ` 3.3V regulator;
- an `XC6206P182MR-G` 1.8V regulator and `RS0104YUTQH12` four-channel
  translator for the BHI260AP digital domain;
- external `W25Q128JVSIQ` QSPI flash on the SF32 MPI2 mapping;
- `BHI260AP` smart IMU and `BME280` environmental sensor on I2C1;
- manufacturer-qualified 48MHz and 32.768kHz crystals; and
- a six-pin debug/calibration socket.

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
10kohm OE pull-down; firmware enables it with PA00 after both rails are stable.

The former CH343P, microSD, and bare SX1262 circuits are intentionally absent.
The external flash already consumes PA12-PA17, which are the same pins used by
the SD interface. Native USB makes the UART bridge unnecessary. A bare SX1262
also needs a board-specific RF network and cannot safely share the SF32 RF
pin. Bluetooth is disabled here for the same RF-layout reason: `BRF_ANT` is
open until a controlled-impedance antenna path and any required matching
network are designed and qualified.

This source describes the electrical design and routing constraints. It does
not claim a production-ready placement, antenna implementation, enclosure, or
board outline; those remain board-level engineering work.
