# USB4105-GF-A application contract

This entry covers GCT USB4105-GF-A only: gold-flash contacts, tape/reel,
0.95 mm shell stakes. It is a USB 2.0 Type-C receptacle, not a PD controller.

## Manufacturer evidence

- [Drawing B4, 18 December 2023](https://gct.co/files/drawings/usb4105.pdf),
  sheet 1: pin table, ordering grid and component-side PCB pattern.
  Retrieved 2026-10-17; SHA-256
  `fb331fbabee8392ed2937ed757c1610cb0f174b84625147c0b580a18eea8c0e5`.
- [Product specification A3, 27 February 2023](https://gct.co/files/specs/usb4105-spec.pdf),
  page 2, section 4: VBUS contacts together 5 A; ground contacts together
  6.25 A; A5/B5 VCONN 1.25 A; other contacts 0.25 A each. Rated voltage
  is 48 V DC; operating range is -40 to +85 C.
  Retrieved 2026-10-17; SHA-256
  `372fe1bc0e0b1b4ce7e18b61514e967c7c2f883f0c7fe1f4586b567785ee9cd2`.

The PDFs remain at their manufacturer URLs. The drawing carries a copying
restriction and is not redistributed in this package. The hashes identify
the captured revisions; recheck the revision if the URLs return different bytes.

## Connection and geometry

CC1=A5, CC2=B5; DP=A6/B6, DN=A7/B7; SBU1=A8, SBU2=B8.
Ground is A1/A12/B1/B12 and VBUS A4/A9/B4/B9. Four shield slots are
represented as SH1..SH4. Shell-to-ground treatment remains a board decision.
VBUS and ground use passive pin roles: the connector does not generate power.

There are twelve copper contact lands serving sixteen contact identities.
Four paired power pads coincide. The copper widths are 0.60/0.30 mm and
length 1.15 mm. Rear shell copper/slot is 1.00 x 2.10 / 0.60 x 1.70 mm;
front shell copper/slot is 1.00 x 1.80 / 0.60 x 1.40 mm. Peg holes are
0.65 mm. The source specifies component-side pattern tolerance +/-0.05 mm.

Library coordinates put the recommended PCB edge at y=0. Derived centers:
front slots y=-2.60; rear slots y=-6.78; contact lands y=-7.355;
pegs y=-6.28. The 0.25 mm courtyard clearance around copper, body and the
explicit pin-1 dot is library-authored and
excludes the mating plug's insertion envelope. Confirm board-edge placement,
enclosure clearance, PCB thickness/stake fit and assembly process separately.

## Limits for an inline PD instrument

The aggregate rating does not permit 5 A through one VBUS contact. Route all
power contacts with suitable copper and verify current sharing, contact losses
and temperature rise. This entry does not establish a complete 5 A path.

A receptacle alone does not establish a passive sniffer topology. Resolve cable
orientation, CC continuity, VCONN and cable-marker communication for the actual
plug/receptacle arrangement. Do not infer USB 3.x lane pass-through, galvanic
isolation, EPR support, or compliant negotiation from the connector rating.

No LCSC stock capture, JLC assembly eligibility, pricing or manufacturing
qualification is asserted. Obtain current exact-MPN sourcing evidence before
locking this part. Compiler checks validate the binding and pad consistency,
not those electrical, mechanical or sourcing claims.
