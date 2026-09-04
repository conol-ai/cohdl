# LED source manifest

Retrieved and audited on 2026-07-30. The PDFs are stored byte-for-byte as
downloaded; none was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `liteon-ltst-c193krkt-5a.pdf` | `LTST-C193KRKT-5A` | Lite-On specification DS22-2005-077 Rev. D, effective 2021-10-22 | <https://optoelectronics.liteon.com/upload/download/DS22-2005-077/LTST-C193KRKT-5A.PDF> | `a735509f0638262d6d43851e13f42ca05bf37c70e9afedbf3173f71591e392ac` |
| `wurth-150060gs75000.pdf` | `150060GS75000` | Wurth Elektronik WL-SMCW datasheet Rev. 002.009, 2019-02-26 | <https://www.we-online.com/components/products/datasheet/150060GS75000.pdf> | `4e12ec7220fb4cb46c61b9f837bef372d82709fbba410caaf7aecae204abd60a` |
| `opsco-sk6812mini-e.pdf` | `SK6812MINI-E-012` | OPSCO file `SK6812MINI-E-012`; header Rev. B/1, cover Version B/2, 2025-11-22 | <https://www.opscoled.com/en/download/index.html> | `d78c60d7a163c7fdb69db78a2fccf0faa8104532fdee0b0472736aaeac427cc7` |
| `xinglight-xl-1005ubc.pdf` | `XL-1005UBC` | XINGLIGHT technical data sheet | [LCSC C22355736 manufacturer document](https://datasheet.lcsc.com/datasheet/pdf/7c777665dc6740ac42e09f3da6cdf7af.pdf) | `93b95a812e16cf3406b94bc55980e4baa170dbec322a10cd1e1e4b60b38a0811` |
| `nationstar-ncd0402r1.pdf` | `NCD0402R1` | NATIONSTAR product specification FGH/BD-0053.01-21B | [LCSC C130719 manufacturer document](https://datasheet.lcsc.com/datasheet/pdf/173b863487fbc96058d97a8db0e2fbb3.pdf) | `4b33f70b950e857eaa0388e2f3410de5d3aca906e3e8a3b788b58286439e7c24` |

## Retrieval notes

- Lite-On's official file endpoint rejected automated retrieval during this
  audit. The local manufacturer-authored Rev. D file was retrieved from
  authorized distributor Mouser:
  <https://www.mouser.cn/datasheet/2/239/LTST_C193KRKT_5A-1141706.pdf>.
- OPSCO's manufacturer portal did not expose a stable direct file URL. The
  current manufacturer-authored B/2 specification was retrieved from
  OPSCO-authorized distributor LCSC's `C5149201` record:
  <https://www.lcsc.com/datasheet/C5149201.pdf>.
- Wurth's local file came directly from the official manufacturer URL in the
  table.

Both distributor retrievals are recorded for reproducibility. Manufacturer
branding, exact MPN, revision, page count, and the relevant drawings were
checked visually after download.

## Coverage and geometry

- Lite-On page 1 establishes the cathode/anode orientation; page 7 specifies
  two 0.8 mm square lands with a 2.3 mm overall span.
- Wurth page 1 identifies pin 1 as cathode and pin 2 as anode, and specifies
  two 0.8 mm square lands with a 2.4 mm overall span.
- OPSCO pages 4-5 supply the complete 1 GND, 2 DIN, 3 VDD, 4 DOUT map and
  1.80 x 0.82 mm recommended lands. This supersedes the different numbering in
  the legacy 2019 Rev. 02 sheet. The footprint's 3.4 x 3.0 mm routed light
  window is a documented KiCad reverse-mount convention because B/2 does not
  dimension a PCB aperture.
- No separate manufacturer CAD files were used; copper geometry comes from
  the dimensioned manufacturer land-pattern drawings.
- XINGLIGHT page 7 gives pad 1 cathode / pad 2 anode and two 0.30 x 0.55 mm
  recommended lands on 0.80 mm centres. NATIONSTAR page 2 gives the same
  polarity convention and a 1.10 mm total land span with 0.40 mm inner gap,
  producing two 0.35 x 0.50 mm lands on 0.75 mm centres.

All PDFs passed `pdfinfo`, Poppler text extraction, and full-page rendering.
The identity, polarity/pinout, package, and land-pattern pages listed above
were visually inspected.
