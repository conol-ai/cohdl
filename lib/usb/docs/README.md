# USB source manifest

Retrieved and revalidated on 2026-07-30.

| Local file | Covered exact MPN | Document version/date | Official source | SHA-256 |
|---|---|---|---|---|
| `molex-1050170001-sales-drawing.pdf` | Molex `105017-0001` | Drawing `SD-105017-001`, Rev T, approved 2012-12-10 | [Molex](https://www.molex.com/content/dam/molex/molex-dot-com/products/automated/en-us/salesdrawingpdf/105/105017/1050170001_sd.pdf) | `4e9804b927788782b90d4ef38ad6bf72bf0ff4ffee361ebba1abd67083c22304` |
| `hro-type-c-31-m-12-family-drawing.pdf` | HRO `TYPE-C-31-M-12` (exact part on sheet 4; related `-12A/-12B/-12C` variants on the other sheets) | Unversioned family drawing; exact-MPN sheet dated 2020-12-08; newest included sheet dated 2026-03-09 | [HRO product page](https://en.krhro.com/Product-Details/726.html) and its [official attachment](https://omo-oss-file110.thefastfile.com/portal-saas/new2023011311465136174/cms/file/type-c-31-m-12&12a&12b&12c.pdf) | `c5a4131fb0432bfc24dae4597173d9a3d242c8c778506e2b3032181c75aaedd4` |
| `infineon-ez-pd-ccg6df-ccg6sf-datasheet.pdf` | Infineon `CYPD6227-96BZXI` | Document `002-27161` Rev `*K`, 2023-08-30 | [Infineon](https://www.infineon.com/assets/row/public/documents/24/49/infineon-ez-pd-ccg6df-ccg6sf-usb-type-c-port-controller-datasheet-en.pdf?fileId=8ac78c8c7d0d8da4017d0ee8e2c571be) | `9c3e7291e38d663c356b5107705f39254d95574720006f59fbf9f2b5a2c0d9ff` |

## Footprint and pin audit

- `FP_USB_Micro_B_SMD` follows Molex's recommended PCB pattern: five
  0.40 x 1.35 mm signal lands on 0.65 mm pitch, four SMD shell lands, two
  0.60 x 1.30 mm plated shell slots, and two 0.85 mm plated shell holes.
  The official [Molex product page](https://www.molex.com/en-us/products/part-detail/1050170001)
  is the part-status reference. The CoHDL pad model cannot independently
  suppress paste on selected shell-only SMD lands, so paste treatment for
  those lands remains a fabrication-review item.
- `FP_USB_C_Receptacle_HRO_TYPE_C_31_M_12` uses HRO sheet 4's
  component-side land pattern, including its 1.64 mm contact-land length,
  0.60 mm locating holes, and four plated slots. A/B contact designators and
  the four commoned shell lands were checked against the official pin table.
- `BGA96C50P11X11_600X600X100N` contains exactly the 96 populated balls in
  Infineon Figure 4, including six DNU balls and no pads at the 25 depopulated
  grid positions. It implements Figures 8 and 9: 10 mil round corner pads,
  10 x 14 mil oval non-corner perimeter pads, and 14 mil round inner pads.
  Figure 8 is the manufacturer's recommended medium-density-interconnect
  footprint. Escape vias, solder-mask-defined versus non-solder-mask-defined
  processing, and the Figure 9 warning to leave D8 unused when using large
  8/16 or 10/16 vias remain PCB-layout decisions rather than footprint
  primitives.


## FUSB302B MLP family — added 2026-10-16

Four address-specific MPX parts use the manufacturer case 510BR land.
Read [the application contract](fusb302b-application.md) for exact pins,
operating limits, register/reset constraints and unresolved sniffer concerns.
Source: [onsemi](https://www.onsemi.com/pdf/datasheet/fusb302b-d.pdf),
`onsemi-fusb302b-d.pdf`, Rev. 5, August 2021; SHA-256 `712b6db97692f283b450b1f15d9018da0a3f83af35b1bdff792c0ef4ca81812f`.
The PDF was retrieved unchanged on 2026-10-16; pin and land drawings were
visually checked on pages 5 and 31. This does not qualify a complete board.

## Additional USB PD controllers (usb 0.1.3)

- `ptn5110-application.md` and `nxp-ptn5110.pdf`: six exact NXP TCPC order codes, startup differences, host/supply requirements and the explicitly unreleased SOT1883-1 source land drawing.
- `stusb1602-application.md` and `st-stusb1602-rev5.pdf`: STUSB1602AQTR BMC transceiver, chip-relative SPI directions, required external MCU, power/termination limitations.
- `stusb4500-application.md` and `st-stusb4500-rev7.pdf`: STUSB4500QTR autonomous sink. The note records the newer Rev. 8 I2C power-loss warning and the bundled Rev. 7 provenance.

These are controller definitions, not passive-sniffer reference designs. Searchable part intents distinguish roles. Each note cites the manufacturer document and exact captured PDF hash; mirrored PDFs retain the original manufacturer content. PDF copyrights remain with their manufacturers.

## More USB PD controllers (usb 0.1.5)

| Family | Exact order codes added | Application information |
|---|---|---|
| TI TUSB422 | TUSB422IYFPR, TUSB422IYFPT | [Host-controlled TCPC, supply/clockless interface, startup and PD-message limitations](tusb422-application.md) |
| onsemi FUSB307B | FUSB307BMPX, FUSB307BVMPX | [Legacy/discontinued TCPC, address straps, LDO/dead-battery behavior and automatic replies](fusb307b-application.md) |

Each part, device and footprint links its application note and unmodified manufacturer PDF through `#[doc]`. The notes record exact source URLs, versions, hashes and page references. New definitions were checked against both pin tables and rendered land drawings. TUSB422's YFP land has nine 0.23 mm copper circles at 0.4 mm pitch; FUSB307B uses the 510BS land and a grounded exposed pad. These are controller models, not qualified passive-sniffer circuits. FUSB307B is explicitly marked for legacy use because its current manufacturer document is discontinued.
