# Fenghua General Series MLCC: part-numbering evidence

`tools/fenghua_b1_report.py` decodes Fenghua MLCC part numbers by the rule below. The rule is transcribed from Fenghua's "General Series Multi-layer Ceramic Capacitors" product specification, 1005～1812 TYPE, 27 pages. The PDF is not in the repository: it was fetched to check the rule, and only its identity and the rule are kept here.

| Field | Value |
|---|---|
| Fetched | 2026-11-23, by `tools/sourcing/http.py` (identifying User-Agent, robots.txt honoured) |
| URL | https://datasheet.lcsc.com/datasheet/pdf/09fe2daf22580a99fa7fea93ccb20fb7.pdf?productCode=C1523 |
| Linked from | the LCSC datasheet pages of C1523 (0402B102K500NT), C1546 (0402CG101J500NT) and C1594 (0603B151K500NT), all three to the same file |
| SHA-256 | `705023d3be0454148a96fc2715d249a32d8ff442e70b64d1116d3490ed1d8136` |
| Size | 3,187,974 bytes |

The document's version and issue-date fields are blank. Its PDF metadata title belongs to an unrelated Word template, so the SHA-256 is the only identifier.

## How To Order (p. 2)

Example: `0805 CG 102 K 500 N T`.

| Position | Field | Codes |
|---|---|---|
| 1 | Size code | 1005 (EIA 01005, 0.40×0.20 mm), 0201, 0402, 0603, 0805, 1206, 1210, 1808, 1812 |
| 2 | Dielectric | CG = C0G, X = X5R, B = X7R, BS = X7S, BT = X7T, DS = X6S, DT = X6T |
| 3 | Nominal capacitance, pF | first two digits significant, third the number of zeros; R is the decimal point (0R5 = 0.5, 1R0 = 1.0, 102 = 1000) |
| 4 | Tolerance | A ±0.05 pF, B ±0.10 pF, C ±0.25 pF, D ±0.50 pF (A–D only for ≤ 10 pF), F ±1%, G ±2%, J ±5%, K ±10%, M ±20% |
| 5 | Rated voltage, V | same code as capacitance (6R3 = 6.3, 500 = 50, 201 = 200) |
| 6 | Termination | N = nickel barrier (three-layer plated) |
| 7 | Packing | T = 7-inch reel, D = 13-inch reel |

## What the report does with it

The report matches a part number only to a combination the library already offers, and only with the tolerance the library pairs with its dielectric (C0G J, X5R and X7R K), as `samsung_b1_alternates.py` does for Samsung. The ±0.25 pF (C) parts of 10 pF and below are wider than the library's ±5% at those values, so they are not alternates. The X7S, X7T, X6S and X6T codes are recognised but never matched, because the library has no such dielectric.
