# RFC-0XX (number to be allocated): `mask: none` on a pad definition

> **PROPOSAL — not accepted; no number allocated.** Design note, 2026-11-25, written at the maintainer's request. The RFC number, the decision-record number and any error code are placeholders pending central allocation in the conol.ai design repository; this file is a working copy in the repository snapshot. Nothing here is implemented. Builds on RFC-018 (pad and footprint format).

## Problem

A pad definition cannot describe copper that the solder mask covers. A surface pad is always copper plus a mask opening, and the opening is the copper grown by `mask_expansion`:

- `mask_expansion` is one `Length`, applied to every side.
- It must be nonnegative; a negative value is E805 ("use a nonnegative expansion").
- The only layer a pad can drop is paste, with `paste: none`.

So a solder-mask-defined land, where the metal is larger than the opening, has no spelling.

## Worked example: TI DDA0008B thermal pad

`dcdc::TI_DDA0008B` (TI drawing 4214849/B, `lib/dcdc/docs/ti-lmr36520.pdf` p39) draws the thermal pad as mask-defined:

| | Size |
|---|---|
| Metal (NOTE 9: "Size of metal pad may vary") | 2.95 x 4.9 mm |
| Solder mask opening | 2.71 x 3.4 mm |

The opening is inside the metal by 0.12 mm per side in x and 0.75 mm per side in y. No single expansion value produces that, negative or not: the two axes need different amounts.

What the library ships today (option A): the land is the 2.71 x 3.4 mm opening as copper, with `mask_expansion: 0mm`. Exposed metal and paste are exactly TI's. The copper under the mask, a 0.24 x 1.5 mm shortfall in total extent, is left to layout. The dcdc README records this.

**The tempting alternative is a trap (option B).** Making the copper 2.95 x 4.9 mm today also makes the opening 2.95 x 4.9 mm. That trades the copper shortfall for exposed metal up to 0.75 mm beyond the package pad at each end, which is worse: solder is no longer confined to the pad the package has.

## Proposal

Add `mask: none` to the pad-definition fields, mirroring `paste: none`. A pad with `mask: none` contributes copper and no mask opening.

A mask-defined land is then two placements with the same pad number, the technique the library already uses for split paste apertures:

```cohdl
pub pad P_TI_DDA0008B_EP_METAL {
    shape: rect
    size: (2.95mm, 4.9mm)
    layer: top_copper
    plating: smd
    mask: none
    paste: none
}

pub pad P_TI_DDA0008B_EP {
    shape: rect
    size: (2.71mm, 3.4mm)
    layer: top_copper
    plating: smd
    mask_expansion: 0mm
}

pub footprint TI_DDA0008B {
    // ...
    pad 9: P_TI_DDA0008B_EP_METAL at (0mm, 0mm)
    pad 9: P_TI_DDA0008B_EP at (0mm, 0mm)
}
```

The metal land carries the copper; the smaller land carries the opening and the paste. Both are pad 9, so the electrical terminal is unchanged.

Rules:

- `mask: none` together with `mask_expansion` on the same pad is an error: there is no opening to expand.
- `mask: none` is for surface pads. Whether it is allowed on a plated through-hole pad (a tented via in pad) is an open question below.
- A pad number whose every placement is `mask: none` has no exposed metal and cannot be soldered; that should be a diagnostic, not a silent land.

## Why not the alternatives

- **Negative `mask_expansion`.** Still one value for all sides. It cannot express 0.12 mm in x and 0.75 mm in y.
- **Per-axis `mask_expansion: (x, y)`.** Expresses this case, but only for an opening concentric with the copper and of the same shape. It also changes the type of an existing field.
- **An explicit `mask_opening: (w, h)` size.** One pad instead of two, but a second geometry on a pad definition, with its own shape, offset and corner questions.
- **`mask: none`** adds one closed keyword, reuses same-number placement, and lets the opening be any pad shape at any position inside the metal.

## Surface it touches

- Parser, formatter and LSP field lists: one more pad field, canonical position next to `paste`.
- Validation in `resolve.rs` (the E805 block): the rules above.
- `emit::kicad_mod` and `kicad_pcb`: the pad layer set gains copper-only (`"F.Cu"`) and copper-with-paste-without-mask variants beside today's `"F.Cu" "F.Paste" "F.Mask"` and `"F.Cu" "F.Mask"`; the back-side mapping follows RFC-026.
- `emit::ipc2581`: the mask opening is derived from copper plus the expansion today; a `mask: none` pad must emit no mask feature.
- `emit::docsjson` (package API docs) and the registry footprint preview: carry and draw the field.
- The library geometry audit (`tests/library/geometry.rs`): overlapping copper of the same pad number is already allowed; nothing new is expected, but it should be pinned by a fixture.

Existing sources do not use the field, so every current artifact stays byte-identical.

## Open questions

1. Through-hole pads: should `mask: none` be allowed there to tent a thermal via, or stay surface-only in the first version?
2. Should the "no exposed metal on this pad number" case be an error or a warning?
3. Error-code allocation: a new code, or more E805 cases?
