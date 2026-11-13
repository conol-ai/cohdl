# UNI-ROYAL thick-film chip resistors: part-number evidence

This is the source record for the UNI-ROYAL decode rule in `tools/gen_passive.py` (`uniroyal_decode`) and for the dry-run report `tools/passive_data/uniroyal_dry_run.json`. The rule is decode only: nothing is emitted, bound, mapped or aliased.

- Manufacturer: UNI-ROYAL. The cover names Uniroyal Electronics Global Co., Ltd., Kunshan, Jiangsu, China, with the manufacture plants Uniroyal Electronics Industry Co., Ltd., Aeon Technology Corporation, Royal Electronic Factory (Thailand) Co., Ltd. and Royal Technology (Thailand) Co., Ltd. The page headers carry the UNI-ROYAL, ROYALOHM and uniohm marks.
- Document: datasheet "Thick Film Chip Resistors", part name "Chip Series", file no. `SMD-SP-001`, V.11 dated Mar.17,2026 (page footers and the revision record on p.12). 12 pages, A4. PDF metadata: title "Microsoft Word - 1-CHIPSeries 第11版", created 2026-08-18.
- Source: <https://www.uni-royal.cn/images/userfile/file/1787191733c56505e6d9ab55c7.pdf>, the manufacturer's own site. It is the specification file linked from every row of the "普通厚膜晶片电阻器" (general thick-film chip resistor) product list, <https://www.uni-royal.cn/product.php?s=%E6%99%AE%E9%80%9A%E5%8E%9A%E8%86%9C%E6%99%B6%E7%89%87%E7%94%B5%E9%98%BB%E5%99%A8>. `uniohm.com` redirects to this site.
- Fetched 2026-11-11 with `tools/sourcing/http.py` (`cohdl-library-bot/0.1`), after the site's `robots.txt`, which does not disallow the path.
- Original PDF SHA-256: `d27e4cdcdf8fdb4c066ff6030cfc1f46b99efe2a38fe5673fa18a2ee4c7220d3` (736,886 bytes).
- Page numbers below are the document's printed ones ("Page 2/12"), which equal the PDF page indices.

The PDF is not redistributed by this repository, and no page or rendering of it is kept. Download it from the source above and verify the SHA-256 before auditing the rule. The values were read from the text layer (`pdftotext -layout`, poppler) and pp. 2 and 5 were also rendered at 110 dpi and compared; the two agree.

## The part number, as the document gives it

"Part No. includes 14 codes" (p.2 §2). The document's example (p.2 §3): "0805 1/8W ±5% 10KΩ T/R-5000" is `0805 W8 J 0103 T 5 E`.

| Codes | Field | What the document states | Page |
|---|---|---|---|
| 1–4 | size | "Part name. E.g.: 01005, 0201, 0402, 0603, 0805, 1206, 1210, 2010, 1812, 2512"; 01005 is written `0105` | p.2 §2.1, §3 |
| 5–6 | power rating | `WH` = 1/32 W, `WM` = 1/20 W, `WG` = 1/16 W, `WA` = 1/10 W, `W8` = 1/8 W, `W4` = 1/4 W, `W3` = 1/3 W, `W2` = 1/2 W, `07` = 3/4 W, `1W` = 1 W. "W=Normal Size", "“1~G” = “1~16”", and "If power rating is equal or lower than 1 watt, 5th code would be “W” and 6th code would be a number or letter" | p.2 §2.2, §3 |
| 7 | tolerance | `D` = ±0.5%, `F` = ±1%, `G` = ±2%, `J` = ±5% | p.2 (the second "2.2"), §3 |
| 8–11 | resistance | §3: "5% (E-24 series): The 1st digit will be “0”; the 2nd & 3rd digits are for the significant figures of the resistance and the 4th digit indicate the numbers of zeros following. ≤1% (E-96 series): The 1st to 3rd digits are for the significant figures of the resistance and 4th digit denotes number of zeros following." 11th code: `0`…`6` = 10⁰…10⁶, `J` = 10⁻¹, `K` = 10⁻², `L` = 10⁻³, `M` = 10⁻⁴, `N` = 10⁻⁵, `P` = 10⁻⁶ | p.2 §2.4, §3 |
| 12 | packaging | `T` = Tape/Reel (the only code given) | p.2 §2.5.1, §3 |
| 13 | quantity | `4` = 4,000, `5` = 5,000, `C` = 10,000, `D` = 20,000, `E` = 15,000. Per size: 01005 `D`; 0201 `E`; 0402 `C`; 0603, 0805, 1206, 1210 `5`; 2010, 1812, 2512 `4`. The reel table gives the same quantities | p.2 §2.5.2, §3; p.11 §12.3 |
| 14 | special feature | `E` = standard (the only code given) | p.2 §2.5.3, §3 |

Rated power and resistance range per size (p.5 §6). The ±1%, ±2% and ±5% columns are equal for every size below.

| Size | Rating | ±0.5% | ±1%, ±2%, ±5% |
|---|---|---|---|
| 0201 | 1/20 W | 1 Ω–10 MΩ | 1 Ω–10 MΩ |
| 0402 | 1/16 W | 1 Ω–10 MΩ | 0.1 Ω–10 MΩ |
| 0603 | 1/10 W | 1 Ω–10 MΩ | 0.01 Ω–10 MΩ |
| 0805 | 1/8 W | 1 Ω–10 MΩ | 0.1 Ω ≤ R ≤ 10 MΩ |
| 0805 | 1/4 W | — | 0.01 Ω ≤ R < 0.1 Ω |
| 1206 | 1/4 W | 1 Ω–10 MΩ | 0.1 Ω ≤ R ≤ 10 MΩ |
| 1206 | 1/3 W | — | 0.01 Ω ≤ R < 0.1 Ω |
| 1210 | 1/2 W | 1 Ω–10 MΩ | 0.01 Ω–10 MΩ |
| 2010 | 3/4 W | 1 Ω–10 MΩ | 0.01 Ω–10 MΩ |
| 2512 | 1 W | 1 Ω–10 MΩ | 0.01 Ω–10 MΩ |

(01005 at 1/32 W and 1812 at 3/4 W are in the table too; the library has neither size.)

## One place where the document is read, not copied

§2.4.1 and §2.4.2 (p.2) word the two value forms by value series: "If value belongs to standard value of E-24 series, the 8th code is zero…", "If value belongs to standard value of E-96 series, the 8th~10th codes are the significant figures…". §3 on the same page words them by tolerance: "5% (E-24 series)" and "≤1% (E-96 series)". The two readings differ for a 1% part whose value is in E-24: 10 kΩ ±1% is `1002` under §3 and would be `0103` under §2.4.1 read alone.

The rule follows §3. The catalog agrees: all 289 decoded rows of the B1 snapshot are ±1%, they include E-24 values, and every one uses three significant digits (`0402WGF1002TCE` is 10 kΩ on JLCPCB's page). The other pairing, `F` with a zero-led code or `J` with three significant digits, is refused rather than guessed.

## What the document does not give

- **The zero-ohm jumper's part number.** No value code, tolerance letter or example for a jumper appears anywhere in the 12 pages. See the next section for what is stated about the jumper.
- **Resistor arrays.** The `4D02…`/`4D03…` numbers of the snapshot are not covered; the site lists chip arrays (晶片排列电阻器) as a separate product family, and nothing about it was fetched.
- Temperature coefficient (p.7 §10 gives it per size and value range) is not in the part number.

## The zero-ohm jumper

What the document states:

- p.5 §7 "Ratings", columns "Resistance Value of Jumper", "Rated Current of Jumper", "Max. Overload Current of Jumper":

  | Size | Resistance | Rated current | Max. overload current | Operating temperature |
  |---|---|---|---|---|
  | 0201 | <50 mΩ | 0.5 A | 1 A | −55 °C to 155 °C |
  | 0402 | <50 mΩ | 1 A | 2 A | −55 °C to 155 °C |
  | 0603 | <50 mΩ | 1 A | 2 A | −55 °C to 155 °C |
  | 0805 | <50 mΩ | 2 A | 5 A | −55 °C to 155 °C |
  | 1206, 1210, 2010, 2512 | <50 mΩ | 2 A | 10 A | −55 °C to 155 °C |

- p.5, note under the table: "The measurement position of the 0Ω resistance value should be based on the positive electrode".
- p.7–8 §10: for a 0 Ω part the limit after short-time overload, load life, load life in humidity, low-temperature storage and high-temperature exposure is "<50mΩ", tested with "max Overload current" or "rated current for 0Ω"; "The resistors of 0Ω only can do the characteristic noted of *".
- The manufacturer's product list (the page named under Source) carries a jumper row for each size reading "0Ω：＜50mΩ" with the power, temperature-coefficient and tolerance columns left as "/".

What it does not state: a tolerance for the jumper, what the tolerance letter means on a jumper's part number, or the part number itself. The `F` in `0402WGF0000TCE` is therefore not explained by this document. That spelling comes from the B1 snapshot (C17168, C21189, C17477, C17888) and from JLCPCB's pages for those four rows, and stays marked inferred in the rule.

Beside it, the library's part: `R_0R_J_<size>` is declared `ChipResistor<0ohm, 5%>` with the Yageo primary `RC<size>JR-070RL`. Yageo's datasheet (`lib/passive/docs/yageo_rc_series_datasheet_en.pdf`, RC_L V.14) says "J = ± 5.0% (for jumper ordering, use code of J)" and gives the jumper criteria per size: RC0402 "Jumper<50mΩ", rated current 1.0 A, maximum current 2.0 A; RC0603 the same; RC0805 2.0 A and 5.0 A; RC1206 2.0 A and 10.0 A. For 0402, 0603, 0805 and 1206 the two manufacturers state the same resistance limit and the same two currents. Whether a UNI-ROYAL jumper is listed as an alternate of the `R_0R_J` part, whose declaration says 5%, was the maintainer's decision. **Decided on 2026-11-13: the four `F`-lettered jumpers may be alternates of `R_0R_J_<size>`, with a note.** The note states what is above: the jumper's rating for its size, and that the document gives a jumper no tolerance and its letter no meaning. `tools/uniroyal_alternates.py` writes it on each `zero_ohm` row of the dry-run report.

## Rule status, field by field

| Field | Status |
|---|---|
| 14 codes; field order | from the document, p.2 §2, §3 |
| size codes | from the document, p.2 §2.1; decoded for the library's eight sizes |
| `W` and the power code | from the document, p.2 §2.2 |
| power rating per size | from the document, p.5 §6 |
| tolerance letters | from the document, p.2; `F` and `J` decoded |
| value: three digits at 1%, zero-led two digits at 5% | from the document, p.2 §3 (see the note on §2.4.1 above) |
| value: power-of-ten codes | from the document, p.2 §2.4.3 |
| value: allowed range | from the document, p.5 §6 |
| `T` | from the document, p.2 §2.5.1 |
| reel code per size | from the document, p.2 §2.5.2, §3; p.11 §12.3 |
| trailing `E` | from the document, p.2 §2.5.3 |
| zero-ohm `0000` after `F` | still inferred (catalog rows and JLCPCB pages); not in the document |
| arrays | not decoded; not in the document |

Checked by catalog rows as well as the document: sizes 0402, 0603, 0805 and 1206 with `F` (289 rows, each agreeing with its JLCPCB page on resistance, tolerance, rated power and package). Resting on the document alone, with no catalog row: sizes 0201, 1210, 2010 and 2512, and the `J` letter.
