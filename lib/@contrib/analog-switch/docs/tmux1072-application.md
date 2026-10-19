# TMUX1072 application contract

Source: [TI SCDS382C](https://www.ti.com/lit/ds/symlink/tmux1072.pdf), August 2019; retrieved 2026-10-16. Unmodified PDF: `ti-tmux1072-scds382c.pdf`, SHA-256 `a54e8a9336f4ac62fb9a3e1f045499077558bb6ebadaca3004574099c9318a4f`. Pages below are one-based PDF pages.

- Exact parts: TMUX1072DGSR (DGS, 10 pins), TMUX1072RUTR (RUT, 12 pins). Only RUT exposes active-low OE and open-drain FLT (pp3,21).
- VCC: 2.3-5.5 V; normal signal pass range: 0-5.5 V, including above VCC. COM-only 18 V fault tolerance is not a pass-through operating voltage or a 20 V VBUS rating (pp4,14-16).
- Continuous signal current is limited to +/-50 mA; use only a signal branch, not the 5 A power path (p4).
- On/off capacitance reaches 6.2 pF at the specified 240 MHz test. Supply-loss turn-off is up to 250 us under the specified ramp/load; include switch loading and transition exposure in the circuit analysis (p8).
- At VCC=0, I/Os isolate; leakage maximum is 15 uA at the specified 5.5 V test condition. Compare actual leakage/loading against the application's budget (p6).
- Default SEL=low selects NC. OE has an internal pulldown, so RUT enables by default; an external pull-up is needed for disabled startup. A powered-off guarantee does not prove every intermediate supply-ramp/reset state (pp6,8,16-17).
- OE high disconnects both channels. FLT needs a pull-up when used; unused FLT may float. Fit 100 nF at VCC (pp3,16-18).
- OVP can briefly expose downstream pins to 9.6 V under the specified transient test; it is not a universal low-voltage clamp (p7).

Library limits: these are signal switches, not PD controllers. No passive-sniffer architecture, firmware sequence, USB-C orientation/VCONN continuity, protection qualification, JLC stock, or manufacturing readiness is established.

Land source: DGS p25 and RUT p28; pin numbering p3. Copper rectangles preserve drawing dimensions; source corner radii, solder-mask and stencil geometry are not encoded. Courtyards add library margins, not TI dimensions. Verify the assembler's process before release.
