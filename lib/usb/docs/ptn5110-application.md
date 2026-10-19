# PTN5110 application contract

Source: [NXP PTN5110 datasheet](https://www.nxp.com/docs/en/data-sheet/PTN5110.pdf), Rev. 1.6 (3 April 2020), captured 2026-10-16 as `nxp-ptn5110.pdf`. SHA-256: `f79743918861d2b8369bcfd69d62fca5bac0353290f77cdb278cf1e8d7a99e86`.

## Identity and role

This is a TCPC PHY/protocol controller. An external TCPM supplies PD policy (pp10-11). It is not a complete autonomous policy engine. The exact order codes in Table 3 p6 are separate parts because their POR roles and TCPC interfaces differ (Table 2 pp5-6):

| MPN | TCPC | POR role |
| --- | --- | --- |
| PTN5110HQZ | Rev1.0 v1.1 | DRP, initially sink |
| PTN5110DHQZ | Rev1.0 v1.1 | Sink |
| PTN5110THQZ | Rev1.0 v1.1 | Source |
| PTN5110NHQZ | Rev2.0 v1.0 | DRP, initially sink |
| PTN5110NDHQZ | Rev2.0 v1.0 | Sink |
| PTN5110NTHQZ | Rev2.0 v1.0 | Source |

## Connections and operating limits

Table 4 pp8-9 is the pin oracle; GND's unnumbered center pad is library pad 17. BYPASS is an internal regulator node, not a general-purpose output supply. Fit its specified 2.2 uF +/-20% capacitor. VDD and VBUS require 2.2 uF +/-10% decoupling; VCONN_IN has a 2.2 uF example. Keep the external VBUS power switches separate from these controller pins.

Table 10 p34: VDD 2.7-5.5 V, VBUS 3.9-25 V; CC operating input -0.3 to 5.5 V. The 28 V VBUS and 6 V CC limits on p33 are absolute maxima, not operating ratings. VCONN supply selection and switch current need their own design check.

I2C supports up to 1 MHz. Table 6 p15 encodes 7-bit addresses: SLV_ADDR at GND = 0x50; 10 kohm to BYPASS = 0x51; floating = 0x52; 100 kohm to BYPASS = 0x53. It is sampled only at POR. Do not strap it directly to VDD. I2C pull-ups go to VDD; ALERT_N and FAULT_N need external pull-ups (10 kohm example). GPIO logic is referenced to BYPASS, not VDD. Respect the 3.6 V operating limit on ALERT_N/FAULT_N and GPIOs even if VDD is higher. Table 4's general VDD pull-up wording does not override Table 10 limits.

## Passive-capture limitation

POR applies source/sink terminations according to the exact variant. Dead-battery behavior applies CC clamps (p17). Back-current protection does not establish high-impedance passive behavior. Direct attachment is therefore not qualified as an always-passive inline sniffer. A design needs evidence for isolation, startup, power loss, receive filtering/GoodCRC behavior, orientation and VCONN; no such architecture is supplied here. TCPC version is not interchangeable with USB PD revision.

## Land provenance and release limit

Fig. 2 p8 fixes top-view numbering. Fig. 21 p54 (SOT1883-1, EP17081) supplies 0.4 mm pitch, 0.9 x 0.24 mm copper leads, 3.6 mm overall copper span, and 1.1 x 1.0 mm exposed-pad land. Centers are derived as (3.6 - 0.9)/2 = 1.35 mm. The source drawing is visibly marked **UNRELEASED**; these transcribed lands require manufacturer confirmation before manufacturing release. The 4.1 mm courtyard adds a library-authored 0.25 mm copper clearance. Paste segmentation, mask expansion and thermal vias are not represented by this copper declaration. Compile success checks consistency, not fabrication readiness, PD compliance, or live assembly stock.
