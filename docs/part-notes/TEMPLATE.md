# <MPN or series> — <one-line function>

<!--
Part note template (plan 5b), schema part-note/1. Copy this file to
docs/part-notes/<package>/<PART_NAME>.md and fill every section.

Rules:
- Every number cites the page of a pinned source in "Sources": write (p. N).
  No page, no number. Do not fill a value from memory or from another part.
- Quote limits as the datasheet states them, with their conditions. Keep
  absolute maximum ratings apart from recommended operating conditions.
- The pin table lists every pad of the library footprint once, with the
  library pin name exactly as the device declares it. tools/part_notes.py
  check compares it with the device.
- Say what the source does not establish instead of guessing.
- Delete these comments.
-->

## Identity

```json
{
  "schema": "part-note/1",
  "device": "<package>::<DEVICE>",
  "parts": ["<package>::<PART>"],
  "identities": [
    {"mfr": "<library mfr>", "mpn": "<library mpn>", "lcsc": "<C number or null>"}
  ],
  "sources": [
    {"id": "ds", "title": "<document title>", "revision": "<revision, date>",
     "url": "<official URL>", "sha256": "<SHA-256 of that PDF>", "local": "<lib/... path or null>"}
  ],
  "status": "draft"
}
```

## Summary

Three to five sentences: what the part is, what it is for, the one or two
properties that decide whether to choose it, and when not to use it.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| | | | | | | ds p. N |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| | | | ds p. N |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| | | | | ds p. N |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | | | | | ds p. N |

## Typical application

- Required external parts, with values and the page that asks for them.
- Default connection of every configuration pin.

## Layout notes

- What the source asks of the layout (decoupling placement, thermal pad,
  keep-outs), with pages.

## Pitfalls

- Mistakes an agent is likely to make with this part, each tied to a page.

## Not established by the sources

- What a designer might expect to find here but the sources do not state.
