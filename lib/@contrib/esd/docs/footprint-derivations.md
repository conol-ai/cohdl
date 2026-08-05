# ESD footprint derivations

## ULC0511C DFN1006DN

The checked-in Tergy data sheet page 3 gives a complete DFN1006DN terminal
envelope: 0.65 mm terminal pitch, 0.15-0.35 mm terminal length, 0.40-0.60 mm
terminal width, 0.95-1.05 mm package length, 0.55-0.65 mm package width, and
0.50 mm maximum height. The bottom view also identifies terminal 1 at the
notched lower end.

`QFN2N65P60X100` applies nominal-density leadless-terminal goals to the
nominal terminal dimensions:

- nominal terminal length `L = (0.15 + 0.35) / 2 = 0.25 mm`
- nominal terminal width `b = (0.40 + 0.60) / 2 = 0.50 mm`
- toe, heel, and side goals `JT = 0.15 mm`, `JH = 0.10 mm`, and
  `JS = 0.05 mm`
- nominal land length `L + JT + JH = 0.50 mm`
- nominal land width `b + 2JS = 0.60 mm`; this is rounded outward to
  0.65 mm to retain side margin at the 0.60 mm maximum terminal width
- manufacturer terminal pitch is retained exactly, so land centers are at
  `y = +/-0.325 mm`

The resulting `0.65 x 0.50 mm` lands retain 0.075 mm at both the toe and heel
of a maximum-length 0.35 mm terminal and 0.025 mm per side at the maximum
0.60 mm terminal width. Opposing land edges are 0.15 mm apart. Fabricators
must confirm that copper and solder-mask capabilities support that gap.
