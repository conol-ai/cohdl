# The native `.kicad_pcb` emitter — `cohdl build --emit kicad_pcb`

`cohdl build --emit kicad_pcb` writes `out/<name>.kicad_pcb`: a KiCad 10
board file produced directly by the compiler — no KiCad installation, no
pcbnew scripting, byte-stable. It replaces the `tools/kicad_board.py` flow
(pcbnew assembling a board from the netlist + IPC-2581 XML + `.kicad_mod`
projections) with a single emitter reading the same checked IR every other
artifact reads. The flag composes with `--emit ipc2581` (`--emit` is
repeatable with distinct values), so the Quilter handoff and the KiCad board
can come out of one build.

The emitted board is a **layout starting point**: placements, net-bound
footprints, and the board outline — no routing, no zones, no design rules.
KiCad draws the ratsnest from the pad net bindings.

## What the file contains

- KiCad 10 format (`(version 20260206)`, generator `"cohdl"`;
  `generator_version "10.0"` declares the format generation targeted and is
  fixed for byte stability). The layer table and `(setup …)` block are the
  stock two-copper-layer defaults pcbnew itself writes — every generated
  board is 2-layer, the status quo the pcbnew flow established.
- One `(footprint …)` per instance, in designator natural order, named by
  the RFC-018 projection identity (fq path with `::` → `-`, e.g.
  `passive-CHIP_0402` — the same base name as `out/footprints/*.kicad_mod`).
  Reference = the designator; Value = the same principal value the netlist
  and BOM use; hidden empty Datasheet/Description complete KiCad's expected
  property set.
- Pads carry nets **by name** (`(net "GND")` — the KiCad 10 dialect; the
  file has no numbered net table). Net names are exactly the `.net`
  emitter's names. A logical pin fans out to every one of its physical pad
  numbers, and every same-numbered pad copy (exposed land + thermal vias)
  carries the net. `nc` pins and mechanical pads are represented by the
  absence of a net clause.
- Footprint geometry — pads (all RFC-018 shapes, chamfers, annulus rings,
  paste overrides), courtyard, `window` cutouts, RFC-022/023 mount holes,
  and RFC-031 silkscreen — comes from the same shared derivation the
  `.kicad_mod` emitter renders, so the two projections cannot drift.
- The RFC-020 board outline (when the design declares one) as
  `gr_line`/`gr_arc` on Edge.Cuts at 0.1 mm. Arc midpoints honor the DXF
  winding (correct beyond 180°, unlike the retired script's bisector).
- Deterministic RFC-4122-shaped uuids derived from stable identity
  (package/refdes/element), never random — same design, same bytes.

## Coordinate frame and placement

CoHDL's authoring frame **is** KiCad's board frame (+y down): `place`
coordinates and footprint-local geometry pass through verbatim. (The
IPC-2581 document's y-negation is that format's +y-up requirement.)

A top-side instance: `(at x y R)` with the authored rotation, angle omitted
when 0, normalized to pcbnew's (−180, 180].

## RFC-026 back side — the on-disk representation

Pinned empirically against pcbnew-written boards (and the hard-won
LEFT_RIGHT-flip lesson): a `side: bottom` instance keeps its authored
(x, y) and takes `(layer "B.Cu")`; KiCad stores the left/right flip as its
canonical y-mirror + 180° decomposition:

- footprint angle = authored R + 180, normalized to (−180, 180];
- every footprint-local y negates (pads, graphics, text anchors, mount
  holes, custom-pad polygon vertices);
- every `F.*` layer becomes `B.*` (`*.Cu`/`*.Mask` and Edge.Cuts unchanged);
- texts gain `(justify mirror)`; property angles land back on R mod 360;
- a pad's own RFC-025 rotation **reverses** (reflection:
  `R·Mirror·r = (R−r)·Mirror`), pad angles normalized to [0, 360);
- an asymmetric chamfer corner swaps vertically (the horizontal half of the
  flip lives in the folded-in 180°).

The absolute pad delta this encodes is `Rot(R+180)·(lx, −ly)` — for a pad
at local (1, 2) on an unrotated bottom part: (−1, 2), the empirical
LEFT_RIGHT check. Verified semantically identical to pcbnew's own output
over both repo examples and both OpenMicroKBD hardware revisions (v1: 88
footprints / 33 bottom; v2: 140 footprints / 53 bottom).

## Unplaced instances

Instances without a `place` statement stage exactly where the IPC-2581
document stages them: a shelf-packed grid immediately outside the board
outline (one convention, two emitters). With no outline to stage against,
they take the plain 12 mm grid from (40, 40), eight per row, in designator
order — never stacked at (0, 0).

## Ownership and routed boards

The board participates in `out/.cohdl-manifest` like every artifact: only
written with the flag, refused if a **foreign** file (one CoHDL did not
write — e.g. a pcbnew-generated or hand-routed board) already sits at the
path, and swept as stale by a later build without the flag once CoHDL owns
it. Consequence: **never route in place under `out/`** — copy the board out
(the established `pcb/` convention) and route the copy. The refusal
protects a foreign routed board; it cannot protect one saved over a
CoHDL-owned path.

## Verification

Automated: byte determinism, the pinned back-side/rotation encodings, net
fan-out, staging, ownership, and zero impact on every other artifact
(tests/kicad_pcb.rs). Cross-emitter agreement with IPC-2581 pad math is
pinned by the shared worked examples (tests/side_rotate.rs commentary).
A real `pcbnew.LoadBoard` + semantic diff against pcbnew-assembled boards
was executed live with KiCad 10.0.4 for both examples and both OpenMicroKBD
hardware revisions. `tools/validate_kicad_pcb.py` makes that check repeatable:
it compares footprint placements/sides/orientations, every pad copy and net,
fields, footprint graphics, and Edge.Cuts while ignoring UUID/net-code/order
noise. An actual KiCad open remains a human checkpoint, as ever — a
wrong-but-valid orientation is invisible to check/build.
