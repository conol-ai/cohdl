# AP2112K-3.3TRG1 application contract

Exact scope: fixed 3.3 V AP2112K-3.3TRG1, SOT25. Package-authored summary;
not a complete-board approval or live supplier/assembly-availability claim.

Source: [Diodes AP2112 DS39724 Rev. 2-2, June 2017](https://www.diodes.com/datasheet/download/AP2112.pdf),
shipped as `docs/diodes-ap2112.pdf`; recorded source audit 2026-07-30,
re-read 2026-10-16. SHA-256:
`ef8d376f2ec356e29172eb9e053819a0ebdcc576dba7fc9ab0505c568427920f`.
The pages below refer to that pinned 18-page revision.

## Wiring and limits

- Page 1: pin 1 VIN, 2 GND, 3 EN, 4 NC, 5 VOUT. Leave pin 4 unconnected.
- Page 3: recommended VIN 2.5–6.0 V, ambient -40 to +85 C. Fixed 3.3 V
  operation additionally requires headroom above VOUT. The 6.5 V absolute
  maximum is not the recommended upper input voltage. Never connect this
  regulator directly to a 20 V USB-PD rail.
- Page 8, 3.3 V electrical table: output 3.3 V +/-1.5% at its stated test
  conditions, 600 mA maximum load specification, dropout up to 400 mV at
  600 mA. Derive worst-case headroom and thermal budget for the actual load;
  do not treat the current headline as a board-level continuous-current rating.
- Pages 1–2: use input and output bypass capacitors of 1 uF; the reference
  circuit connects both to GND. Check effective capacitance at the rail voltage,
  tolerance, temperature and layout; a nominal marking alone is not evidence.
- Page 8: EN high at least 1.5 V and no more than 6 V; low at most 0.4 V.
  EN has a typical 3 Mohm internal pulldown. Drive or tie it to a defined
  valid level; do not assume a floating pin turns the regulator on.
- Pages 1/8: disabled output is discharged through a typical 60 ohm path.
  Auto-discharge is not reverse-current blocking or host power isolation.
  This source does not establish reverse-blocking performance: keep power
  domains separate or add a separately qualified isolation solution.
- Page 3 lists SOT25 thetaJA 184 C/W without a heatsink; the actual PCB
  copper and ambient determine temperature rise. Estimate dissipation from
  (VIN - VOUT) * IOUT plus quiescent loss, and verify the board's thermal case.

## Land and remaining work

Page 14 supplies the SOT25 land: 0.55 x 0.80 mm pads, 0.95 mm pitch,
2.40 mm opposing row-centre separation. The registry's
`ldo::SOT5P95X290X160N` rotates this pattern 90 degrees while preserving
pin identity. Reuse it; do not infer coordinates from package body dimensions.
The prior source audit visually checked pages 1 and 14. Pin currents, source
transients, thermal derating, capacitor selection and manufacturing review
remain checks on the actual design.
