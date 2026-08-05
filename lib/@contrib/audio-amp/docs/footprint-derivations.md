# Audio-amplifier footprint derivations

## NS4150B MSOP-8

Nsiway's current NS4150B product page identifies the orderable product as
`NS4150B` in MSOP-8 and hosts the December 2022 V1.4 data-sheet pages used for
the identity and pin-map cross-check:

<https://nsiway.com.cn/list_38/82.html>

The relevant manufacturer-hosted page-image SHA-256 values retrieved on
2026-08-05 are:

```text
1f99db83e9983c25b3b084546c6f2a6474bd2105af10c9522314cd18799e700f  page 1: identity and MSOP-8 package claim
9319f61835255db85a41b7914eed8b3c1dc2c48a1b30efdce3484050aeddc656  page 2: exact eight-pin assignment
d0e496e0f21de85eadbe4c19478638593dc2e603e8f614639243b59a72fbd30a  page 9: MSOP-8 package dimensions
```

V1.4 page 9 and checked-in `ns4150b.pdf` page 13 give the same complete
package envelope: 0.65 mm basic pitch, 4.70-5.10 mm lead span, 0.40-0.70 mm
terminal length, 0.29-0.38 mm terminal width, 2.90-3.10 mm body, and 1.10 mm
maximum height.

`SOP8P65X490X110N` applies IPC-7351 nominal-density gull-wing goals to the
nominal package values:

- nominal lead span `E = 4.90 mm`
- nominal terminal length `L = (0.40 + 0.70) / 2 = 0.55 mm`
- nominal inner lead gap `S = E - 2L = 3.80 mm`
- nominal toe, heel, and side goals `JT = 0.35 mm`, `JH = 0.35 mm`, and
  `JS = 0.03 mm`
- land outer span `Z = E + 2JT = 5.60 mm`
- land inner gap `G = S - 2JH = 3.10 mm`
- land length `(Z - G) / 2 = 1.25 mm`
- land center offset `(Z + G) / 4 = 2.175 mm`
- nominal land width is approximately 0.40 mm; it is rounded outward to
  0.45 mm to retain side margin at the 0.38 mm maximum terminal width.

The resulting `1.25 x 0.45 mm` lands retain positive worst-case margins across
the stated component tolerances: 0.25 mm toe at the 5.10 mm maximum span,
0.10 mm heel at the 4.70 mm span/0.70 mm terminal combination, and 0.035 mm
per side at the 0.38 mm maximum terminal width. Board-fabrication and placement
capability still need to support the 0.20 mm minimum gap between adjacent
lands.
