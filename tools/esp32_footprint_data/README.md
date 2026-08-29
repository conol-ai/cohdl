# ESP32 footprint source snapshot

`footprints.json` is the reviewed, normalized input to
`tools/gen_esp32_footprints.py`. Normal regeneration and `--check` are fully
offline.

The snapshot joins three pinned evidence sets:

- Espressif's KiCad library 3.2.1 at commit
  `1dfc3110895c9cd62daf332f49c49ee0ee200831`;
- KiCad's generic footprint library at commit
  `819223b66f96508feaeaa305301b5e6bb5c1038b`, used only to cross-check the
  generic references in Espressif symbols;
- exact URL/SHA-256 facts from Espressif's downloadable PADS ASCII land
  patterns. Raw website CAD is not redistributed because those downloads do
  not carry the GitHub library's open-license notice.

Refresh is explicit and requires clean checkouts plus the separately downloaded
CAD tree:

```sh
python3 tools/gen_esp32_footprints.py \
  --import-sources /path/to/espressif-kicad /path/to/kicad-footprints \
  --direct-cad-root /path/to/esp32-cad
```

Review the complete JSON diff and update the snapshot SHA-256 pin in the
generator only after accepting every geometry/provenance change.
