# W25Q128JVSIQ — 128 Mbit 3 V SPI/Dual/Quad NOR flash, SOIC-8 208 mil

## Identity

```json
{
  "schema": "part-note/1",
  "device": "flash::W25Q128JV",
  "variants": ["SOIC8"],
  "parts": ["flash::FLASH_W25Q128JVSIQ"],
  "identities": [
    {"mfr": "Winbond", "mpn": "W25Q128JVSIQ", "lcsc": "C97521"}
  ],
  "sources": [
    {"id": "ds", "title": "W25Q128JV 3V 128M-bit Serial Flash Memory with Dual/Quad SPI", "revision": "Revision G, 2019-04-08",
     "url": "https://www.winbond.com", "sha256": "5e2f6a77e0225a5c61b8f4b5b9680b57e0e1fe68cfc011465e30f665fe282057", "local": "lib/flash/docs/w25q128jv.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages (1-based); the printed page number is one lower.

## Summary

128 Mbit (16 MB) serial NOR flash with standard, Dual and Quad SPI, 256-byte pages and uniform
4 KB / 32 KB / 64 KB erase, on a single 2.7 V to 3.6 V supply (ds p. 5). Use it for MCU/FPGA
code storage, XIP or data logging: up to 133 MHz clock at VCC 3.0 V to 3.6 V (ds p. 65), 24-bit
addressing covers the whole array (ds p. 29). The "IQ" ordering option ships with QE = 1 fixed,
so pins 3 and 7 act as IO2/IO3 and /HOLD is disabled (ds p. 17, p. 75). It is a 3 V part only
(absolute max VCC 4.6 V, ds p. 61); choose another part for 1.8 V I/O or > 16 MB.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Clock frequency FR (all instructions except 03h) | D.C. | — | 133 | MHz | VCC 3.0 V to 3.6 V | ds p. 65 |
| Clock frequency FR (all instructions except 03h) | D.C. | — | 104 | MHz | VCC 2.7 V to 3.0 V | ds p. 65 |
| Clock frequency fR, Read Data (03h) | D.C. | — | 50 | MHz | — | ds p. 65 |
| Equivalent Dual / Quad I/O clock rate | — | 266 / 532 | — | MHz | Fast Read Dual/Quad I/O at 133 MHz | ds p. 5 |
| Standby current ICC1 | — | 10 | 60 | µA | /CS = VCC, VIN = GND or VCC | ds p. 63 |
| Power-down current ICC2 | — | 1 | 20 | µA | /CS = VCC, VIN = GND or VCC | ds p. 63 |
| Read current ICC3 | — | 8 | 15 | mA | Read Data / Dual / Quad, 50 MHz, DO open | ds p. 63 |
| Read current ICC3 | — | 12 | 20 | mA | Read Data / Dual / Quad, 104 MHz, DO open | ds p. 63 |
| Program / erase / write-status current ICC4–ICC7 | — | 20 | 25 | mA | /CS = VCC | ds p. 63 |
| Page program time tPP | — | 0.4 | 3 | ms | Up to 256 bytes | ds p. 66 |
| Sector erase time tSE (4 KB) | — | 45 | 400 | ms | — | ds p. 66 |
| Block erase time tBE1 (32 KB) | — | 120 | 1,600 | ms | — | ds p. 66 |
| Block erase time tBE2 (64 KB) | — | 150 | 2,000 | ms | — | ds p. 66 |
| Chip erase time tCE | — | 40 | 200 | s | — | ds p. 66 |
| Write Status Register time tW | — | 10 | 15 | ms | Non-volatile bits | ds p. 66 |
| Program/erase endurance | 100,000 | — | — | cycles | Per sector | ds p. 5 |
| Data retention | 20 | — | — | years | "More than 20-year" | ds p. 5 |
| JEDEC ID (9Fh) | — | EFh 40h 18h | — | — | -IQ/JQ: MF = EFh, ID15–ID0 = 4018h | ds p. 22 |
| Device ID (ABh/90h/92h/94h) | — | 17h | — | — | — | ds p. 22 |
| VCC(min) to /CS low tVSL | 20 | — | — | µs | Characterized only | ds p. 62 |
| Delay before write instruction tPUW | 5 | — | — | ms | Characterized only | ds p. 62 |
| Write-inhibit threshold VWI | 1.0 | — | 2.0 | V | Characterized only | ds p. 62 |
| Software reset time tRST | — | — | 30 | µs | After 66h + 99h | ds p. 66 |
| /CS high to power-down tDP / to standby tRES1 | — | — | 3 | µs | — | ds p. 66 |
| Input low / high voltage VIL / VIH | –0.5 / VCC×0.7 | — | VCC×0.3 / VCC+0.4 | V | — | ds p. 63 |
| Output low / high voltage VOL / VOH | VCC–0.2 (VOH) | — | 0.2 (VOL) | V | IOL = 100 µA / IOH = –100 µA | ds p. 63 |
| Input / output capacitance | — | — | 6 / 8 | pF | TA = 25 °C, VCC = 3.0 V, sampled | ds p. 63 |
| Input / I/O leakage | — | — | ±2 | µA | — | ds p. 63 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Supply voltage VCC | –0.6 to 4.6 | V | ds p. 61 |
| Voltage on any pin VIO (relative to ground) | –0.6 to VCC+0.4 | V | ds p. 61 |
| Transient voltage on any pin VIOT (< 20 ns) | –2.0 to VCC+2.0 | V | ds p. 61 |
| Storage temperature | –65 to +150 | °C | ds p. 61 |
| Lead temperature | J-STD-20C (Note 2) | — | ds p. 61 |
| ESD, human body model | –2000 to +2000 | V | ds p. 61 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VCC for FR = 133 MHz, fR = 50 MHz | 3.0 | 3.6 | V | ds p. 61 |
| VCC for FR = 104 MHz, fR = 50 MHz | 2.7 | 3.0 | V | ds p. 61 |
| Ambient temperature TA, Industrial ("I", as in SIQ) | –40 | +85 | °C | ds p. 61, p. 75 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | CS | /CS | Input | Chip select, active low; must track VCC at power-up/down | ds p. 6, p. 10 |
| 2 | DO | DO (IO1) | I/O | Data output (IO1 in Dual/Quad) | ds p. 6 |
| 3 | WP | /WP (IO2) | I/O | Write protect input, active low; IO2 when QE = 1 | ds p. 6, p. 10 |
| 4 | GND | GND | Ground | Ground | ds p. 6 |
| 5 | DI | DI (IO0) | I/O | Data input (IO0 in Dual/Quad) | ds p. 6 |
| 6 | CLK | CLK | Input | Serial clock; SPI mode 0 or 3 | ds p. 6, p. 12 |
| 7 | HOLD | /HOLD or /RESET (IO3) | I/O | Hold input, active low; IO3 when QE = 1 | ds p. 6, p. 10 |
| 8 | VCC | VCC | Power | Power supply | ds p. 6 |

## Typical application

- Connect CLK, /CS, DI (IO0), DO (IO1) to the host SPI; add IO2 (pad 3) and IO3 (pad 7) for Quad
  instructions such as 6Bh/EBh/32h (ds p. 12, p. 24).
- /CS: add a pull-up to VCC if the host cannot hold /CS high while VCC ramps (ds p. 10, p. 13).
- QE = 1 is factory-fixed on -IQ parts (ds p. 17, p. 75): pad 7 is IO3 and /HOLD is not
  available; pad 3 is IO2. Without a Quad host, tie pads 3 and 7 high to VCC (pull-up) so
  they do not float (inference; the source gives no value, ds p. 6).
- /WP locks the Status Register only when SRP = 1 and /WP is low (ds p. 16); with the factory
  default SRL = SRP = 0, /WP has no control (ds p. 16).
- Firmware: wait tVSL ≥ 20 µs after VCC(min) before the first command and tPUW ≥ 5 ms before
  Write Enable/program/erase (ds p. 62); issue 06h before every program, erase or status write
  (ds p. 13).

## Layout notes

- SOIC-8 208 mil body: E1 5.13–5.33 mm, H 7.70–8.10 mm, pitch 1.27 mm BSC, A ≤ 2.16 mm
  (ds p. 68); library footprint `soic::SOIC8P127X790X216N` matches this span and height.
- Winbond gives no land pattern and no decoupling or routing guidance in this datasheet.
- At 104–133 MHz keep CLK and IO traces short; AC limits assume CL = 30 pF and 5 ns input edges
  (ds p. 64).

## Pitfalls

- Do not use /HOLD on this part: the IQ suffix fixes QE = 1 and disables /HOLD (ds p. 17, p. 75
  Note 5). Use the -IM/JM part (QE = 0, JEDEC ID 7018h) if /HOLD is needed (ds p. 22, p. 75).
- Pad 7 is labelled "/HOLD or /RESET"; a hardware /RESET pin exists only on SOIC-16 and TFBGA
  (ds p. 5, p. 10). Reset the SOIC-8 with 66h then 99h, then wait tRST 30 µs (ds p. 60, p. 66).
- Check BUSY and SUS before software reset; reset during erase/program can corrupt data (ds p. 60).
- Read Data (03h) is limited to 50 MHz; above that use Fast Read (0Bh) with 8 dummy clocks
  (ds p. 30, p. 65). 133 MHz requires VCC ≥ 3.0 V (ds p. 61).
- Commands sent before tPUW (5 ms) that write or erase are ignored (ds p. 13, p. 62).
- If WPS = 1, all individual block lock bits are 1 at power-up; erase/program is ignored until
  39h or 98h unlocks (ds p. 13, p. 18, p. 59). Factory default WPS = 0 (ds p. 13).
- Page Program wraps within a 256-byte page; more than 256 bytes overwrite earlier data
  (ds p. 37).
- Erase times are long: 64 KB block up to 2 s and chip erase up to 200 s (ds p. 66); poll BUSY,
  do not use fixed short delays.
- Output drive defaults to 25 % (DRV1:0 = 11) (ds p. 18); raise it if edges are slow at high clock.
- In power-down (B9h) only ABh is recognized, not even Read Status Register (ds p. 45).

## Not established by the sources

- Decoupling capacitor value or placement, and any land pattern for package S.
- Pull-up values for /CS, /WP (IO2) or /HOLD (IO3); whether pads 3 and 7 have internal pull-ups.
- Whether /WP hardware protection still acts in Standard/Dual SPI when QE = 1; p. 16 ties /WP to
  "Standard/Dual SPI operations" while p. 10 says /WP becomes IO2 when QE = 1.
- Function of Status Register-3 bit S23 (HOLD/RST): the bit is drawn on p. 18 but not described.
- Hold timing (tHLCH etc.): no /HOLD timing appears in the AC tables (ds p. 65–66).
- Retention or endurance conditions (temperature) behind the p. 5 feature-list figures.
- Thermal resistance of the SOIC-8 package; maximum junction temperature.
- Official per-document URL: the repository records only <https://www.winbond.com>.
