# RS2257 application limits

Source: bundled manufacturer `rs2257.pdf`, Run-IC RevC.1,
SHA-256 `2518049fa2c22b066be9c2f821a8df717c95c4c3d53970046e55c25f8254bc91`.
Pages are one-based PDF pages; original provenance is in `docs/README.md`.

- RS2257XH is SOT23-6; RS2257XC6 is SOT363/SC70-6 (p2).
- Operating supply: 1.8–5.5 V. Analog terminals operate between ground and
  VPLUS; a 3.3 V supply does not permit a 5 V analog signal (pp3–4).
- The terminals have diodes to the supply rails (p2, note 2). That note's
  10 mA current limit beyond the rails is a stress-protection instruction,
  not an operating or zero-backfeed guarantee.
- Consequently, selecting the unused throw does **not** establish isolation
  from a live CC/VCONN conductor when VPLUS is absent. The published leakage
  tests use VPLUS=1.8–5.5 V, not zero supply (p3).
- IN=0 selects NC; IN=1 selects NO (p1). Hold the control input at a defined
  logic level (p3).
- Typical capacitance is 80 pF off and 350 pF on under the stated tests;
  these are not maximum guarantees (p4). Include loading in any CC analysis.

These facts do not qualify a passive PD sniffer circuit. Check every powered,
startup, reset and unpowered state of the complete tap network.
