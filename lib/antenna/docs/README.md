# Antenna source manifest

Audited on 2026-08-07. The PDF is stored byte-for-byte as downloaded; it was
not regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `2450at18b100.pdf` | `CeramicAntenna2450` / `ANT_2450AT18B100E` | Johanson Technology 2450AT18B100 detail specification, Ver. 4.0 (2018-11-05) | <https://www.johansontechnology.com/datasheets/2450AT18B100/2450AT18B100.pdf> | `63a97b13585ba8f45408e746669e4cc9899fe0956d3ba8251932a1206ce3a2c7` |

SHA-256 checksums:

```text
63a97b13585ba8f45408e746669e4cc9899fe0956d3ba8251932a1206ce3a2c7  2450at18b100.pdf
```

The footprint represents only the two terminal copper lands shown on the
manufacturer evaluation board: 0.80 x 1.60 mm lands, 2.60 mm inner gap, and
4.20 mm total copper span.

The antenna is not production-ready from footprint placement alone. The host
board must reproduce or re-engineer the reference RF environment: place the
antenna at the PCB edge, preserve the 6.5 x 6.5 mm no-ground region, route a
50 ohm feed, provide the recommended matching network, and tune the assembled
board with a VNA. Enclosure, stack-up, nearby copper, batteries, and cables can
all change the required match.
