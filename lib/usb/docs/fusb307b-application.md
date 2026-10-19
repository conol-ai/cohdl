# FUSB307B application contract - legacy device

Source: [onsemi FUSB307B/D](https://www.onsemi.com/pdf/datasheet/fusb307b-d.pdf), Rev. 7, May 2026; retrieved unchanged on 2026-10-17 as `onsemi-fusb307b-d.pdf`. SHA-256: `f8adcc3df9f5ae85627e69c51a8635d2179dc6ace67a8afca53e6a36a45bb6ca`. Page numbers below are one-based PDF pages.

**Lifecycle:** the manufacturer PDF is watermarked discontinued; its revision table records document discontinuation on 2026-05-14 (p56). These definitions support existing/legacy designs. They are not a recommendation for new sourcing or a stock/availability claim.

## Exact parts and host interface

Table 1 p3 lists **FUSB307BMPX** (-40 to 85 C) and **FUSB307BVMPX** (automotive, -40 to 105 C), both 16-lead 3x3 mm QFN, case 510BS. Keep the exact order code and temperature qualification explicit. This library does not treat them as automatic alternates.

The device provides a PD 3.0 Rev. 1.1 / TCPCI Rev. 1.0 Ver. 1.2 controller interface and requires an external TCPM (p1). Pins 11/12 swap I2C functions depending on addressing configuration; their library names preserve both functions. Table 3 p7 gives:

| Pin 4 sampled level | Pin 11 | Pin 12 | 7-bit address |
|---|---|---|---|
| Low | SCL | SDA | 0x50 |
| High | SCL | SDA | 0x51 |
| Low | SDA | SCL | 0x52 |
| High | SDA | SCL | 0x53 |

Pin 4 is sampled with an external pull-up/down at startup, then becomes an orientation/general output; do not hard-strap it against that output. INT_N is active-low open drain; DBG_N is described as open-drain I/O. SCL/SDA and interrupt signals require appropriate pull-ups (pp5,7,26). Table 16 specifies 1.8-3.6 V I2C pull-up supply, while footnote 15 also constrains it to no higher than VDD. Use the intersection of those limits. CoHDL pin roles do not simulate open-drain or strap sequencing.

## Supplies and autonomous behavior

Table 6 p20 permits VDD 2.8-5.5 V for functional operation, but **all subsequent electrical parameters require at least 3.0 V**. VBUS supply operation is 4.0-21.5 V; VCONN is 2.7-5.5 V. Absolute maximum ratings are not operating ratings.

VBUS can power the internal LDO and TCPM during dead battery; when VDD exceeds VDDGOOD it bypasses the LDO (pp5-6,25). Figure 2 p3 shows 1 uF capacitors on VDD and LDO and 220 pF shunts on CC1/CC2. LDO is an output/bypass node, not another independent supply input. VBUS includes detection and discharge; SRC/SNK control external switches and do not carry the board's load current (pp12-17).

When unpowered, **both CC pins present Rd**. Dead-battery operation retains Rd and can enable the sink path (pp5-6). The controller **automatically sends GoodCRC before reporting a received valid packet** (p12); BIST receive also acknowledges. Firmware silence alone does not ensure passive capture. The library supplies no validated receive-only configuration, power-off isolation, orientation/VCONN architecture, timestamp service budget or sniffer firmware.

## Pin and land audit

Pin table/diagrams: pp1,4-5. GND joins lead 15 and the exposed pad, represented as library pad **17**. The top view on p1 places pin 1 upper-left; p4's top-through view is the same device rotated 180 degrees, alongside a bottom view. Do not mirror either physical binding.

Case 510BS, document 98AON13630G Issue O (2016-08-31), p57: 0.57 x 0.30 mm peripheral copper, 0.50 mm pitch, 3.30 mm outside span, 2.16 mm inside span. Pad-row centers are therefore +/-1.365 mm. EP copper is 1.75 x 1.75 mm. The library's 4.4 mm courtyard and pin-1 dot include clearance and are not manufacturer dimensions. Mask/paste/thermal-via treatment and assembly qualification remain board responsibilities. Manufacturer PDF notices remain intact; MIT covers the authored declarations and notes.
