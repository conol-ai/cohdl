# FUSB302B MLP application contract

Scope: FUSB302BMPX, FUSB302B01MPX, FUSB302B10MPX and FUSB302B11MPX,
14-lead MLP/WQFN, onsemi case 510BR. This is a library-authored summary of
manufacturer facts and explicit application limitations; it is not approval of
any complete USB-PD sniffer or evidence of current supplier availability.

Source: [onsemi FUSB302B/D, Rev. 5, August 2021](https://www.onsemi.com/pdf/datasheet/fusb302b-d.pdf),
retrieved 2026-10-16, stored unchanged as `docs/onsemi-fusb302b-d.pdf`.
SHA-256: `712b6db97692f283b450b1f15d9018da0a3f83af35b1bdff792c0ef4ca81812f`. Page numbers below are one-based PDF pages.
The downloaded 33-page file includes the case drawing at page 31.

## Exact identities and wiring

- Table 1, p2 and Table 15, p18: 7-bit I2C addresses are 0x22 for
  FUSB302BMPX, 0x23 for FUSB302B01MPX, 0x24 for FUSB302B10MPX and
  0x25 for FUSB302B11MPX. The final address bit in Table 15 is R/W;
  do not use an 8-bit transfer address as a 7-bit device address.
- Figure 5/Table 3, p5: CC2 = pins 1,14; VBUS = 2; VDD = 3,4;
  INT_N = 5; SCL = 6; SDA = 7; GND = 8,9 and exposed thermal pad;
  CC1 = 10,11; VCONN = 12,13. Connect every physical pin in a group.
  The library names the otherwise unnumbered exposed ground pad 15.
- SDA and INT_N are open-drain. SCL is an input. Use external pull-ups,
  sized against the actual bus capacitance and required edge timing.

## Operating envelope and externals

- Table 8, p13: VDD functional range 2.7–5.5 V; electrical parameters
  require at least 3.0 V (note 3). VCONN supply is 2.7–5.5 V;
  VBUS detection input is 4–21 V. These are separate pin domains.
  The CC absolute maximum of 6 V is a stress limit, not a recommended
  operating or transient-protection target. Do not apply monitored 20 V
  to CC, VDD or VCONN.
- Figure 18/Table 43, p30: VDD bypass values are 100 nF and 1 uF;
  VCONN bypass is 100 nF. I2C and INT_N reference pull-ups are 4.7 kohm;
  INT_N minimum is 1 kohm. I2C pull-up voltage is 1.71 V through VDD.
  VCONN-source bulk is 10–220 uF when that source function is used.
  Those source-port values do not authorize adding bulk or terminations
  to a monitored CC conductor.
- Table 9, p14: receiver input impedance minimum 1 Mohm and receiver
  capacitance typical 50 pF, with transmitter off. They do not establish
  unpowered loading or complete-board loading with protection and switches.
- Table 8/Table 1: these four MPX variants operate at -40 to +85 C.

## Receive-only use: constraints, not a validated sniffer recipe

- Table 18, p20: SWITCHES0 (0x02) resets to 0x03: PDWN1 and PDWN2
  are enabled. Clearing PU_EN1/2, PDWN1/2 and VCONN_CC1/2 disables
  the programmed terminations/source switches after configuration.
  A directly attached chip is therefore NOT proven passive during reset,
  before initialization or after loss of host power. A passive design needs
  evidence-backed isolation/startup behavior, including failure states.
- Table 19, p20: leave AUTO_CRC and TXCC1/TXCC2 disabled; AUTO_CRC
  otherwise transmits GoodCRC. Control2 (Table 24, p22) must not toggle
  roles. Control3 (Table 25, p23) must not send/retry packets or resets.
  Never enqueue transmit tokens. PWR controls receiver activation (Table 27,
  p23); select the receive path per the documented CC orientation.
- These facts do not establish capture of both CC conductors simultaneously,
  all USB PD 3.0 packet/extended-message cases, FIFO servicing latency, or
  lossless firmware timestamping. Prove the architecture and firmware budget.
- VCONN is a supply input to the cable-power switch, not a second CC monitor.
  This source does not state a grounded/NC disposition for unused VCONN.
  Do not invent one. Either keep it in its specified supplied operating
  range with source switches disabled and prove the resulting isolation, or
  obtain additional manufacturer guidance for the proposed unused treatment.
- Connector/cable orientation, e-marker VCONN continuity, host-to-monitored
  power isolation, 5 A connector ratings and ESD loading remain system-level
  checks. None follows from selecting this registry part.

## Qualified land pattern

Page 31, case 510BR issue O (2016-08-31), document 98AON13629G:
manufacturer-recommended copper lands are 0.30 x 0.58 mm, 0.50 mm pitch,
with opposing land-row centres 2.43 mm apart, and a 1.45 x 1.45 mm exposed
pad. Figure 5 p5 establishes top-view pin numbering. Both pages were
rendered and visually inspected. `QFN14N50P250X250_1EP145X145` uses those
coordinates in CoHDL's component-side, y-down frame. Ground pad 15 is a
library numbering convention for the unnumbered exposed pad. The 3.51 mm
square courtyard adds 0.25 mm beyond outer copper; it is a library margin,
not a dimension claimed by onsemi. Mask, stencil segmentation, thermal vias
and assembler process qualification remain layout/manufacturing work.
