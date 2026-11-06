# Part notes

Part notes are short Markdown summaries of a library part's datasheet, written for agents that design boards with the library (plan 5b). A note gives the facts an agent needs to use a part correctly, and cites the page of a pinned source for each one. It does not replace the datasheet.

**Location is provisional.** Whether notes ship inside each package's `docs/` (and so change its package hash) or live in a separate knowledge base is still open. Until that is decided, they live here, outside every package, so no package version or lock depends on them.

## Writing a note

1. Copy `TEMPLATE.md` to `<package>/<PART_NAME>.md`. A series note may cover a generated family instead (see `passive/YAGEO_RC_SERIES.md`).
2. Work from a pinned source. That is the manufacturer PDF the package already ships or records, with its SHA-256; `tools/sourcing_data/datasheet_links/` lists the links per identity. Do not use memory or another part's datasheet.
3. Cite a page for every number, keep absolute maximums apart from operating conditions, and say what the source does not establish.
4. Run `python3 -I tools/part_notes.py check` and fix what it reports. It checks:
   - the identity against the library;
   - the LCSC part number against `lcsc_map.json`;
   - each local source against its SHA-256;
   - the pin table against the device's pins;
   - that every table row cites a page.
5. A note starts as `"status": "draft"`. It becomes `"reviewed"` only after a person has checked it against the source.

## Pilots (2026-11-07)

| Note | Kind | Source |
|---|---|---|
| `@st/stm32/MCU_STM32F072CBT6.md` | MCU | ST datasheet, shipped in `lib/@st/stm32/docs/` |
| `ldo/LDO_AP2112K_3V3.md` | Power IC | Diodes DS39724 |
| `usb/FUSB302B_MLP.md` | Interface IC, four variants | onsemi FUSB302B/D Rev. 5 |
| `diode/D_1N4148W.md` | Discrete | Diodes DS30086 |
| `passive/YAGEO_RC_SERIES.md` | Passive series | Yageo RC_L V.14 |

The plan named STM32F103C8T6 as the MCU pilot. Its datasheet is not in the repository, so the pilot uses STM32F072CB, whose datasheet is.
