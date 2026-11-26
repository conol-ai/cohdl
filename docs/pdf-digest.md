# Datasheet digests shipped with library packages

Status: proposal, revised 2026-11-26 with the owner's decisions recorded.
Nothing described here is built. This document creates no digest, changes
no package or version, and does not edit `LIBRARY.md` or any cohdl-ai file.
Proposed wording for those files is given as text in §9 and §12.

This is library and agent-tooling work, not language design, so it lives
beside [`library-supplier-plan.md`](library-supplier-plan.md) in `docs/` and
not in `docs/design/`. `docs/design/README.md` describes that directory as a
"Snapshot of the CoHDL Coherent Design Repository" and says to "re-extract
with `python3 tools/extract_design_repo.py docs/design` rather than
hand-edit".

Facts about this repository were checked against the files cited. Facts about
cohdl-ai were checked against its `origin/main` at `ccc37abd` (2026-11-26).
All cohdl-ai line numbers in this document are that commit's. The local
cohdl-ai checkout is on branch `fix/floorplan-context`, where
`apps/agent-worker/src/floorplan.ts`, `packages/skills/src/skills/pcb-floorplan.ts`
and `apps/studio/lib/services/chat-session.ts` differ from `main`. The usage
figures in §1 come from a read-only inventory of the cohdl-ai staging
database and two development databases, as relayed in the brief. They were not
re-derived here.

**Decisions recorded in this revision**

- The consumer is **Option 1, an explicit `digest` action** on `PdfView`
  (§8).
- Every digest carries **all 20 intent sections**. Numeric and table material
  is transcribed with a page per row. Advisory material is rewritten in our
  own words with a page, never quoted (§4).
- A digest passes only if it answers **at least 80%** of the replayed real
  queries in the top-20 intents (§10).
- **Publication, the PDF rule, distribution, and the merge with part notes**
  are decided (§6):
  - digests are public and contain facts only;
  - every package ships its datasheet PDFs unless the vendor explicitly
    prohibits redistribution (superseded the same day by Rule B, §6.3);
  - digests ship inside the package, and each digest change is a patch
    version bump;
  - digests and part notes merge into one schema.
- **The catalog-scale exception survives** (§6.2). Packages such as
  `@st/stm32` stay hash-only.
- **The 100 part-note drafts move into their packages, marked `draft`.** The
  consumer never serves a draft (§4.5, §8.3).
- **The evidence-only documents are not migrated until their size impact is
  known** (§10.5).
- **Digests and PDFs are decoupled** (§6.3), which supersedes the uniform PDF
  rule:
  - **Rule A:** a digest always ships.
  - **Rule B:** a PDF ships unless one of four conditions holds.
  - **Rule C:** a digest's validity is `local` when its PDF ships and
    `deferred` when it does not.

## 1. Problem

### 1.1 The tool

`PdfView` is defined in `apps/agent-worker/src/tools-documents/index.ts`, with
its request schema at lines 20–23 and its description at lines 64–66. It has
four actions:

| Action | What it does | Returns |
|---|---|---|
| `capture` | Downloads the PDF, keeps it, renders one page | one PNG |
| `view` | Renders another page of a kept PDF | one PNG |
| `read` | Replays a saved render | one PNG |
| `find` | Literal text search over a kept PDF | text excerpts |

`Fetch` refuses PDFs and redirects to `PdfView`. `CohdlDocs`
(`apps/agent-worker/src/tools-hdl/index.ts:1635`) already returns, for each
PDF bundled in a package, a registry URL and its SHA-256. It tells the agent
to "capture with PdfView using that url and expectedSha256".

### 1.2 What the inventory measured

| Source | Calls | capture | view | find | read | Sessions | Projects | Documents |
|---|---|---|---|---|---|---|---|---|
| staging | 159 | 57 | 52 | 50 | 0 | 18 | 6 | 45 |
| one dev DB | 1185 | 454 | 511 | 178 | 36 | 147 | 54 | 232 |

Combined, the tool returned 934 page images and ran 228 text searches.
Agents mostly look at rendered pages. They rarely search text.

**Repetition is per document.** There were 284 distinct documents by content
hash. Of these, 57 were read in more than one project and 66 in more than one
session. The most repeated:

| Document | Calls | Projects | Notes |
|---|---|---|---|
| HRO TYPE-C-31-M-12 family drawing | 34 | 20 | 28 of 32 page reads were page 1 |
| onsemi FUSB302B | 131 | 16 | |
| ST USBLC6-2 | 18 | 10 | |
| Richtek RT9080 | 23 | 8 | |
| Richtek RT6150A/B | 39 | 8 | |
| ST STM32F072CB | 58 | 8 | |

**Repetition is also per page.** Among the 20 most-read documents, 7 have one
page with at least 30% of that document's page reads:

- HRO drawing p1 (28/32);
- TPS25982 p1 and p4;
- RT9080 p1 and p5;
- TPD1E05U06 p7;
- USBLC6-2 p1–2;
- RS2257 p1–2.

**Failures.** The inventory reports 192 failures in total. Its eight named
classes:

| Count | Failure | Stage |
|---|---|---|
| 44 | URL returned HTML instead of a PDF | fetch |
| 29 | Renderer refused the page content | read |
| 28 | Malformed arguments | call |
| 21 | Download timeout | fetch |
| 19 | "no such file/run/snapshot", all on `view`; the message does not say which | read |
| 13 | Server refused (bot protection) | fetch |
| 8 | 404/410 | fetch |
| 3 | SHA-256 mismatch | fetch |
| **165** | **total of the named classes** | |

The named classes add up to 165, not 192. **27 failures are unclassified**
(§11.4). The fetch classes total 89 (44 + 21 + 13 + 8 + 3):

- 89 is 46% of all 192 failures, or 54% of the 165 classified ones.
- The draft's "roughly 34% of failures happen while fetching" matches only
  HTML-instead-of-PDF plus timeout: 65 of 192.

This revision uses 89 as the fetch-class count.

### 1.3 What the inventory cannot say

**No duration or latency is stored anywhere.** The only timing available is an
upper bound: the result's timestamp minus the timestamp of the message that
issued the call. That bound has a median of about 10 s and a p90 of about
33–35 s per call. It includes model time and queueing, so it is not a
measured duration, and this document makes no latency claim from it. §10.3
requires real start and end times from the pilot's first day.

### 1.4 What agents searched for

The 228 `find` queries fall into 57 intents. The top 20 by call count:

| # | Intent | Calls |
|---|---|---|
| 1 | Specific pin or signal-name lookup | 27 |
| 2 | PCB layout and keepout | 13 |
| 3 | Supply current in run/active mode | 13 |
| 4 | Land pattern and package dimensions | 12 |
| 5 | Numeric literal only | 11 |
| 6 | Supply or power consumption, no mode stated | 11 |
| 7 | Absolute maximum ratings | 10 |
| 8 | Pinout and pin description | 10 |
| 9 | Diode/TVS reverse ratings | 9 |
| 10 | Ordering and variant comparison | 8 |
| 11 | Output current and inrush | 8 |
| 12 | Strapping and boot mode | 7 |
| 13 | Supply current in low-power modes | 7 |
| 14 | Regulator output-voltage setting | 6 |
| 15 | RF transmit | 5 |
| 16 | Crystal and load capacitance | 5 |
| 17 | Regulator external components (output capacitor, ESR, inductor) | 5 |
| 18 | Supply voltage | 5 |
| 19 | Recommended operating conditions | 4 |
| 20 | Thermal | 4 |

These 20 intents account for 180 of the 228 queries. §4 uses this ranking as
the digest's section order. §10 replays the same queries as the acceptance
test.

### 1.5 The HRO drawing: which page is which

This repository ships the HRO drawing as
`lib/usb/docs/hro-type-c-31-m-12-family-drawing.pdf`, SHA-256
`c5a4131fb0432bfc24dae4597173d9a3d242c8c778506e2b3032181c75aaedd4`.

- `lib/usb/docs/README.md:8` records the same hash.
- The file has a single commit (`46bf90e`, 2026-07-30), whose blob hashes to
  the same value. The committed file has not drifted from the record.
- The README does not say whether the hash is a vendor-published checksum.
  Its line 3, "Retrieved and revalidated on 2026-07-30.", reads as our own
  download and hashing.

What each page holds, according to the repository's records:

- `lib/usb/docs/README.md:8`: "HRO `TYPE-C-31-M-12` (exact part on sheet 4;
  related `-12A/-12B/-12C` variants on the other sheets)".
- `docs/part-notes/usb/USB_C_HRO_TYPE_C_31_M_12.md:28–29`: "Only PDF page 4
  describes this exact MPN; pages 1–3 are the -12A/-12B/-12C variants and were
  not used for any fact here."

If the inventory's HRO document has this hash, then 28 of 32 page reads were
of a sheet for a different MPN. Two things remain unverified:

- **The inventory's hash.** It has not been compared with this one.
- **The page contents.** They were not inspected for this document. This
  machine has no PDF tool: `pdfinfo`, `pdftotext`, `pdftoppm`, `mutool`, `qpdf`,
  `gs`, `magick`/`convert`, `pypdf` and PyMuPDF are all absent. So the page
  assignment above rests on the two quoted records, not on a reading of the
  PDF.

## 2. What already exists in this repository

The digest builds on three existing conventions and does not need a new one.

**Vendor PDFs in package `docs/`.** `LIBRARY.md` §2 says "Save unmodified
PDFs under the manufacturer library's `docs/` directory". It requires
`docs/README.md` to record, for every file, the "local filename", "official
download URL", "document version and release date" and "SHA-256 checksum".
`find lib -name '*.pdf'` finds 106 PDFs. Most documents in the inventory's top
list are among them:

| Inventory document | Shipped file |
|---|---|
| HRO TYPE-C-31-M-12 | `lib/usb/docs/hro-type-c-31-m-12-family-drawing.pdf` |
| FUSB302B | `lib/usb/docs/onsemi-fusb302b-d.pdf` |
| USBLC6-2 | `lib/esd/docs/st-usblc6-2-datasheet.pdf` |
| RT9080 | `lib/@contrib/ldo/docs/rt9080.pdf` |
| RT6150A/B | `lib/@richtek/dcdc/docs/rt6150a-rt6150b-datasheet.pdf` |
| STM32F072CB | `lib/@st/stm32/docs/stm32f072cb-datasheet.pdf` |
| TPS25982 | `lib/@ti/power-switch/docs/tps25982.pdf` |
| RS2257 | `lib/@contrib/analog-switch/docs/rs2257.pdf` |

**`.evidence.md` records where a package does not ship the PDF.** There are
68 such files, for example in `lib/sod/docs/`, `lib/esd/docs/` and
`lib/connectors/docs/`. `lib/esd/docs/tpd1e05u06.evidence.md` records the
source URL, an "Original PDF SHA-256", and "Facts used" with page numbers. It
ends: "The third-party PDF is not redistributed by this repository."
`LIBRARY.md` never mentions `.evidence.md`. Under Rule B (§6.3), these
records become the hash-only form for any of its four conditions, and their
documents' digests still ship (Rule A).

**Part notes (plan 5b).** `docs/part-notes/README.md` defines a closely
related artifact:

> Part notes are short Markdown summaries of a library part's datasheet, written for agents that design boards with the library (plan 5b). A note gives the facts an agent needs to use a part correctly, and cites the page of a pinned source for each one. It does not replace the datasheet.

They have:
- a schema, `part-note/1` (`docs/part-notes/TEMPLATE.md`);
- a checker, `tools/part_notes.py`, whose limits are in §11.1;
- a draft/reviewed status.

There are 100 notes, and every status field in them reads `"draft"`. Their
location was left open on purpose: "Whether notes ship inside each package's
`docs/` (and so change its package hash) or live in a separate knowledge base
is still open."

**Package-local application notes.** Some packages ship hand-written Markdown
attached to declarations through RFC-017 `#[doc]`, for example
`lib/usb/docs/fusb302b-application.md`. 85 `.cohdl` files carry a
`#[doc("docs/….md")]` attribute. `CohdlDocs` returns the content of such
attached Markdown inline (`apps/compile-service/src/docs-references.ts`,
`attachPackageReferences`). §13 Q6 covers what that means for digests.

**The relationship.** Part notes and digests merge into one artifact and one
schema (§6.1, decision 4; mapping in §4.5). The merged artifact is:
- keyed by **document**, not by part;
- shipped **inside the package**;
- structured by the page map and the intent-ordered sections;
- carrying the part-note identity fields, which the checker already verifies
  against the library.

It is keyed by document because the `digest` action receives the same URL and
hash that `capture` would. `part-note/1` is retired.

## 3. Decision: the digest ships inside the library package

A digest lives at

```text
lib/<pkg>/docs/<source-basename>.digest.md
```

next to the PDF or `.evidence.md` record it describes, with the same basename.
For example:

- `lib/usb/docs/hro-type-c-31-m-12-family-drawing.digest.md`;
- `lib/esd/docs/tpd1e05u06.digest.md`.

### 3.1 Why the package and not a project or a central index

- **The repetition is per package.** One drawing was read in 20 projects, and
  FUSB302B in 16. Every one of those projects installed the same package. A
  package digest is written once and reaches every installer.
- **The package already carries what the digest depends on.** RFC-029's
  content hash covers "every regular file (dotfiles and dot-directories
  skipped)" (`src/hash.rs`). The registry archive is the same file set
  (`src/registry.rs`, `pack_tar`). So a digest in `docs/`:
  - travels in the published tar;
  - is pinned by `cohdl.lock`;
  - sits beside the PDF or evidence hash it must match.
- **No new serving path.** The registry's `GET /api/doc?pkg&version&path`
  (`registry/src/worker/index.ts:643`) serves any file in a published
  version's tar, and is cached as immutable. The `digest` action can therefore
  read `docs/<basename>.digest.md` from the same package version as the PDF
  URL `CohdlDocs` already hands out, with no registry change.
- **Review stays with the part.** A digest is reviewed in the same pull
  request as the pin map and land it describes.

### 3.2 What this costs, and why the cost was accepted

- **Each new or changed digest is a patch version bump** of its package
  (§6.1, decision 3). This follows `LIBRARY.md` §5: "Once a version is
  released, changed content requires a new version rather than silently
  replacing the locked geometry." Several digest commits can be batched into
  one patch release.
- **The cost was accepted on purpose.** In exchange, every digest is covered by
  `cohdl.lock`: a project sees exactly the digest that was reviewed for the
  version it pins, and a digest cannot change under it.
- **Projects only receive digests after `cohdl update`.**
- **The digest is public.** It contains facts only (§6.1, decision 1).

## 4. Content schema (`pdf-digest/1`)

The owner's requirement is that a digest contains enough for the agent not to
need the PDF in ordinary work. **All 20 intent sections are mandatory.** They
are not a menu.

A digest is still an index to a pinned document, and the PDF remains
available (§8). One case always needs the PDF: transcribing a land-pattern
drawing into a hand-authored footprint still needs a captured page.
`packages/skills/src/skills/cohdl-implementation.ts:281` requires the
footprint's provenance comment to name the capture id.

### 4.1 Rules for every section

1. **Every fact cites a page.** Write `(p. N)` with the PDF page number. Add the
   printed page number where it differs. No page, no number.
2. **Numeric and table material is transcribed, one page citation per row.**
   This covers ratings, electrical values, pin numbers and names, dimensions,
   order codes, formulas and component values. Each value is copied with its
   unit, its conditions, and which of min/typ/max it is, exactly as printed. Do
   not round, convert or combine values.
3. **Advisory and expressive material is rewritten in our own words, with a
   page citation, and never quoted.** This covers layout and keepout advice,
   component-selection guidance, application-note explanations and functional
   descriptions.
4. **Gaps are recorded; nothing is inferred.** If the source is ambiguous or
   silent, add an entry to the Gaps section (§4.3, section 21).
5. **No figures.** Never reproduce figures, drawings, curves or page images.
   The page map says where they are.

### 4.2 Header

The first block is a fenced JSON object. It is the merged identity block
(§4.5):

```json
{
  "schema": "pdf-digest/1",
  "device": "<package>::<DEVICE>",
  "parts": ["<package>::<PART>", "..."],
  "identities": [
    {"mfr": "<library mfr>", "mpn": "<library mpn>", "lcsc": "<C number or null>"}
  ],
  "source": {
    "title": "<document title as printed>",
    "revision": "<revision and date as printed>",
    "url": "<official URL>",
    "retrieved_from": "<fallback URL, or null>",
    "sha256": "<SHA-256 of the exact PDF bytes>",
    "pages": 0,
    "evidence": "<docs/... path of the shipped PDF or the .evidence.md record>"
  },
  "extracted": {"date": "YYYY-MM-DD", "extractor": "<model id or person, and tool version>"},
  "reviewed": {"date": "YYYY-MM-DD", "reviewer": "<person>"},
  "status": "reviewed"
}
```

`device`, `parts` and `identities` have the meanings they have in
`part-note/1`, including its series forms `parts_pattern` and `mpn_pattern`. A
digest that covers several devices (a family drawing, for example) lists one
identity block per device in a top-level `devices` array instead.

A digest in a package has `status: "draft"` or `"reviewed"`. Only `"reviewed"` is ever served (§7, §8.3).

### 4.3 Sections, normative and in this order

The table gives each section's heading and its writing rule. **T** means
transcribe (§4.1 rule 2). **W** means our own words (§4.1 rule 3).

A section that cannot apply to the part (an RF section for a TVS diode, say)
holds `Not applicable: <reason> (p. N)`, citing the page that establishes the
part's class. "Not applicable" is never used for "not found". Not found is a
gap.

| # | Heading | What it holds | Rule |
|---|---|---|---|
| 0 | Page map | One row per PDF page: what it holds (table, drawing, variant or MPN covered, revision block) | W, page numbers |
| 1 | Pin and signal names | Every pin and signal name the document uses, with pin/ball numbers and aliases | T |
| 2 | PCB layout and keepout | Keepout and clearance dimensions, via and copper requirements | T for numbers; W for advice |
| 3 | Supply current, run/active mode | Each value with its mode, frequency, voltage and temperature conditions | T |
| 4 | Land pattern and package dimensions | Dimension symbols with nominal, min and max columns; pitch; exposed pad | T |
| 5 | Value index | Every value transcribed anywhere in the digest, sorted, with its parameter, unit, section and page. This answers the "numeric literal only" intent, which is a kind of query, not a topic | T, derived |
| 6 | Supply or power consumption, mode not stated by the source | Only values whose mode the source leaves unstated; never a guessed mode | T |
| 7 | Absolute maximum ratings | Kept separate from section 19 | T |
| 8 | Pinout and pin description | Pad, datasheet name and type as printed; function in our words | T for names and types; W for function |
| 9 | Diode/TVS reverse ratings | V_RWM, V_BR, V_C, I_PP, leakage and capacitance, with conditions | T |
| 10 | Ordering codes and variant comparison | Exact order codes and suffix meanings; which pages apply to which variant; differences in our words | T for codes; W for comparison |
| 11 | Output current and inrush | | T |
| 12 | Strapping and boot mode | Strap pins, sampled levels and timing; the resulting behaviour in our words | T for levels; W for behaviour |
| 13 | Supply current, low-power modes | Each mode by its datasheet name | T |
| 14 | Regulator output-voltage setting | The formula, reference voltage and fixed-output codes | T; W for guidance |
| 15 | RF transmit | Output power, frequency and current per mode | T |
| 16 | Crystal and load capacitance | C_L, ESR, drive level and recommended values | T; W for selection guidance |
| 17 | Regulator external components | Output capacitor, ESR range, inductor values | T; W for selection guidance |
| 18 | Supply voltage | | T |
| 19 | Recommended operating conditions | | T |
| 20 | Thermal | θJA, θJC, T_J and T_A ranges with their board conditions | T; W for thermal-layout advice |
| 21 | Gaps | Each entry: what was looked for, which pages were searched, and the result ("not stated", "ambiguous: p. 7 and p. 12 disagree", "only in a figure, values not printed"). An empty section says `None found.` | W |
| 22 | Usage notes | What the part is for, when not to use it, the typical connection, and likely mistakes, each tied to a page. This replaces the part-note Summary, Typical application and Pitfalls sections | W |

Sections 0, 5, 21 and 22 are structural. Sections 1–20 correspond one-to-one,
in order, to the intents in §1.4. In section 5's slot, the "numeric literal
only" intent is served by the value index.

### 4.4 What a digest never contains

- Quoted prose, quoted application-note text, or quoted design advice.
- Prose paragraphs lifted from the datasheet, even paraphrased sentence by
  sentence.
- Figures, drawings, curves, photographs or page renders.
- Design advice that does not come from the cited source. That belongs in a
  package application note.

### 4.5 Merge with part notes

The merge changes the schema (decision 4). `part-note/1` is retired and
`pdf-digest/1` is the single schema. The rules below decide which fields win.

**Header fields.**

| Field | Winner | Reason |
|---|---|---|
| `device`, `parts`, `identities` (and `parts_pattern`, `mpn_pattern`) | part note; replaces the draft's `covers` | `tools/part_notes.py` already checks them against the library declarations and `lcsc_map.json` |
| `sources[]` (several) versus `source` (one) | digest: one `source` per file | Validity (§5.1) and the `digest` action need exactly one hash per file. A part note that cites several documents is split into one file per document, each holding only the facts from that document |
| `source.pages`, `source.evidence`, `extracted`, `reviewed` | digest (new) | Needed for page-range checking (§11.3) and review accountability |
| `status` | both forms agree: `draft` or `reviewed` | §7 restricts where a draft may live |

**Body sections.**

| `part-note/1` section | Goes to |
|---|---|
| Identity | Header |
| Summary | 22 Usage notes |
| Key parameters | Sections 3, 6, 9, 11 and 13–20, by intent |
| Absolute maximum ratings | 7 |
| Recommended operating conditions | 19 |
| Pins (Pad, Library pin, Datasheet name, Direction, Function, Source) | 8. The part-note columns win: the checker compares "Library pin" with the device. Function is in our own words |
| Typical application | Values to 14 and 17; the advice to 22 |
| Layout notes | 2 |
| Pitfalls | 22 |
| Not established by the sources | 21 Gaps |

The device-pin, identity and LCSC checks in `tools/part_notes.py` carry over
unchanged to the merged checker (§7 step 3).

**The 100 existing part notes.** They move into their packages as drafts (the
owner's decision):

- Each note is converted to `pdf-digest/1`, split per source where it cites
  more than one, remapped by the table above, and moved to
  `lib/<pkg>/docs/<source-basename>.digest.md` beside its PDF.
- `docs/part-notes/` is then removed, together with `TEMPLATE.md` and the
  `part-note/1` schema. There is no separate staging area.
- **Conversion does not review anything.** Every converted file keeps
  `status: "draft"`. Sections the old note did not have are filled with
  `Not yet extracted`, which is valid only in a draft.
- A draft becomes `reviewed` only through §7 steps 2–4, in its own commit and
  patch release.
- Moving the drafts in changes every affected package's content, so each
  affected package takes one patch version bump for the move (decision 3).

**The consumer never serves a draft.** This is a hard rule (§8.3). A draft in
a package is never presented as a value source.

**Cost of this choice, stated plainly.** Draft text now ships in every install
of those packages. Any error in a draft is on disk in every project that
installs the package, and readable by anyone who opens the file, even though
the `digest` action never serves it. That is the price of moving the drafts
into the packages. The rule that consumers ignore drafts limits the damage to
agent behaviour. It does nothing about what a person reads on disk.

## 5. Provenance, validity and staleness

### 5.1 Validity rule

A digest is **valid** only when all of these hold:

1. `schema` is a version the consumer understands, and `status` is
   `"reviewed"`.
2. `source.sha256` equals the package's current evidence hash for the file
   named in `source.evidence`:
   - if `evidence` is a shipped PDF, that is the SHA-256 of the PDF's bytes in
     the same package version;
   - if `evidence` is an `.evidence.md` record, it is the hash on that record's
     "Original PDF SHA-256" line.
3. The `docs/README.md` row for the same file, where one exists, records the
   same hash.

**An invalid digest is ignored completely.** No part of it is used, and there
is no fallback to an older digest.

Rule 2 is checked in one of two ways, depending on whether the PDF ships
(Rule C, §6.3). Every digest is in exactly one of these states, and the
`digest` action reports which one in every response (§8.3).

**`local`: the PDF ships in the package.** The digest's hash is checked
against a file that is present in every install:
- **at commit**, by the library's CI checker (§7);
- **on every install**, by RFC-029: `cohdl.lock` pins the package content
  hash, which covers the PDF and the digest together. An install whose bytes
  differ fails with E1103 before anything is read;
- **on every `digest` call**, by comparing the shipped PDF's hash with the
  `expectedSha256` the caller passes (§8.3).

No vendor download is involved.

**`deferred`: the PDF is not shipped** (Rule B, any of its four conditions).
Rule 2 can only compare two recorded strings: the digest's hash and the hash
in the `.evidence.md` record or catalog source index. The digest is trusted as
written, and the `digest` action serves it labelled `deferred`, until the
document's bytes are seen.

1. **First check.** The first `capture` of the document is called with
   `expectedSha256` equal to the digest's hash. The worker records the result
   against `(package, version, source.sha256)`.
2. **Match.** The digest stays served and is labelled `deferred, matched
   <date>`. A match is not permanent trust:
   - every later `capture` of the document hashes the download again (capture
     already computes the hash, so this costs nothing extra);
   - a recorded match expires after **30 days**, after which the label
     returns to `deferred` (unchecked) until the next capture.
3. **Mismatch.** The bytes at the recorded URL are not the bytes the digest
   was written from. That digest is invalid, and so is **every `deferred`
   digest in the same package version**. The consumer stops serving them
   until the package publishes a re-extracted digest in a new patch version.
   - `local` digests in that version are unaffected, because their bytes are
     verified locally.
   - The cause may be a vendor republishing (§5.2) rather than a wrong digest.
     The rule treats both the same way, because either way the pinned
     document can no longer be confirmed.
4. **No capture yet.** A digest that has never been checked against bytes is
   served with the label `deferred, not yet checked against the PDF`. The
   agent may use its values, but must cite them as digest values, never as
   inspected pages.

The invalid state is held by the cohdl-ai worker, keyed by package and
version, and is shared across projects and sessions. A new patch version
starts fresh.

### 5.2 Vendor republishing

The inventory saw one URL return two different files over time. The HRO family
PDF is unversioned, and `lib/usb/docs/README.md` notes its "newest included
sheet [is] dated 2026-03-09". Therefore:

- **A URL is never an identity.** The SHA-256 is. A URL only locates a
  candidate digest.
- **A digest describes the pinned bytes**, which the library's pin map and land
  were audited against. Its response says so: "describes revision X pinned by
  `<pkg>` `<version>`; the live URL may now serve a different revision".
- **If the caller's hash differs from the digest's hash**, the digest does not
  apply. The response is the same as for "no digest".
- **When a vendor revision changes a fact the library relies on**, the fix is
  the one in `LIBRARY.md` §2: update the model, the PDF or evidence record, and
  the hash together. A new digest is then reviewed against the new bytes. A
  digest is never edited to match a newer file without re-pinning the source.

### 5.3 Degradation

Each of these situations gives exactly today's agent experience:
- no digest;
- an older package version without digests;
- an invalid digest;
- an unknown schema;
- a `deferred` digest invalidated by a mismatched download (§5.1);
- a document outside every package;
- a digest with `status: "draft"`.

The `digest` action returns an empty result, and the agent proceeds to
`capture` with no error reported (§8.3, §9.4). `capture`, `view`, `read` and
`find` are unchanged in behaviour. The only addition to their output is the
one-line hint in §8.4.

## 6. Publication and distribution: decided

### 6.1 The owner's decisions (2026-11-26)

**Decision 1: what may be published.**
- Digests live in the public library repository, and contain facts only:
  - numeric and tabular material is transcribed exactly, with a page citation
    per row;
  - advisory or expressive material is rewritten in the repository's own
    words, with a page citation, and never quoted;
  - no prose paragraphs lifted from the datasheet, no figures, no page images
    (§4.1, §4.4).
- The vendor PDFs already committed stay committed. `LIBRARY.md` §2 requires
  them ("Save unmodified PDFs under the manufacturer library's `docs/`
  directory"), there are 106 of them, and that position does not change.
- The 68 `.evidence.md` records saying "The third-party PDF is not
  redistributed by this repository" are the exception case of decision 2 (now
  Rule B, §6.3). They are not a second policy.

**Decision 2: one rule for PDFs. Superseded on 2026-11-26 by Rule B (§6.3).**
It is kept here as history. The owner replaced it because the size
measurement in §10.5 showed that applying it uniformly would multiply small
packages by 100–500x for documents whose facts a digest already carries. The
original text:

Every package that has a datasheet ships the PDF in its `docs/`, unless the
vendor explicitly prohibits redistribution.
Only those documents keep a hash-only `.evidence.md` record. Today this rule
is unwritten, and 68 records follow the other convention without stating a
reason. The proposed `LIBRARY.md` text in §12 writes it down.

As a consequence, each `.evidence.md` record must name the vendor's
prohibition: where it is stated and on which page or URL. None of the 68 does
so today. A record that cannot name one no longer qualifies as the exception,
and its package ships the PDF instead. This is a migration step in applying
the rule, not a new policy. Under Rule B, this requirement survives as
condition 1.

**Decision 3: distribution.** Digests ship inside the package, distributed
exactly like the PDF. Changing a digest is a patch version bump of that
package. The cost, one version bump per digest edit, was accepted in
exchange for the lock-file guarantee (§3.2).

**Decision 4: one artifact.** The digest and the part note merge into one
schema. The 100 existing `draft` part notes move into their packages, still
marked `draft`. Nothing becomes reviewed by being moved, and the consumer never
serves a draft (§4.5).

### 6.2 The catalog-scale exception survives (decided 2026-11-26)

`LIBRARY.md` §2 already has an exception for "a catalog-scale generated
package where duplicating hundreds of official PDFs would make the immutable
registry archive operationally unusable". It stays alongside the
vendor-prohibition exception, and `@st/stm32` and its peers continue
hash-only:
- `lib/@st/stm32/docs/README.md` says its source index "replaces 1,240
  duplicate local PDFs";
- `lib/@espressif/esp32/docs/README.md` says its index "avoids duplicating
  hundreds of datasheet binaries".

**Reason.** A several-hundred-page catalog PDF in every install is a real
cost, and the exception already exists and is relied on.

**Accepted downside.** For these packages, a digest's hash cannot be validated
against a local file. Validity is checked only after a vendor download,
exactly as for an evidence-only document (state `deferred`, §5.1). That download is exposed to
the fetch failures in §1.2.

Under Rule B (§6.3), the catalog-scale exception is no longer a separate
rule. It is one case of condition 3: a catalog package that shipped its
documents (1,240 for `@st/stm32`) would far exceed the largest archive the
registry ships today. Its digests are `deferred` (Rule C).

### 6.3 Decoupling digests from PDFs: Rules A, B and C (decided 2026-11-26)

**Why the rule is split.** A digest is kilobytes of text. A PDF is megabytes
of bytes. The owner's goal is to stop agents re-reading datasheets, and the
digest alone serves it. Whether the PDF also ships is a size question, not a
correctness question: it changes only how a digest's hash is validated, not
what the agent can learn from it. The earlier rules decided both questions at
once, so they are separated here.

**Rule A: digests always ship.** A digest always ships inside its package,
with no size gate of any kind. Its only condition is that it is facts-only
(decision 1). A package that keeps its PDF out still ships its digest. This
is what makes the scheme work: the cheap artifact is always present, and the
expensive one is optional.

**Rule B: a PDF ships unless one of four conditions holds.** The figures come
from §10.5, measured 2026-11-26:

| # | Condition: the PDF stays out, hash-only | Threshold | Basis |
|---|---|---|---|
| 1 | The vendor explicitly prohibits redistribution | The `.evidence.md` record names where the prohibition is stated; if it cannot, condition 1 does not apply | Decision 2's exception, kept |
| 2 | The PDF alone is too large | More than 8 MiB (8,388,608 B) | The largest committed PDF is 18,176,869 B (18.2 MB, 17.3 MiB), an outlier; p90 is 4,446,023 B (4.45 MB, 4.24 MiB). Of the 106 committed PDFs, 4 exceed 8 MiB |
| 3 | The package archive would exceed the largest one shipped today | Archive with the PDFs more than 27,288,503 B (27.3 MB); that is `passive`, measured in this repository's `lib/` | Also covers catalog-scale packages (§6.2) |
| 4 | The package archive would grow more than 20x | Archive with the candidate PDFs ÷ archive without them > 20 | `do214` is 0.04 MB (43,852 B) today and would reach 5.5–19.9 MB, 100–500x, for a package of small discrete parts |

Notes on the conditions:
- "Archive" means the package's tar file set: every non-dot file
  (`src/registry.rs`, `pack_tar`).
- Condition 4's denominator is the archive without the candidate PDFs, so the
  test also works for a new package.
- The figure in condition 3 was measured from this repository's `lib/`. The
  registry's own largest published archive was not queried.
- **Thresholds 3 and 4 are provisional and measured, not principled.** They
  record what the library looks like today. They should be revisited once the
  real sizes of the 56 missing documents are known; §10.5 specifies how to
  obtain them.
- The 106 PDFs already committed stay committed (decision 1). Rule B applies
  when a PDF is added. The 4 committed PDFs above 8 MiB are not removed.

**Rule C: what hash-only means.** A hash-only package loses nothing except
local byte validation. Its digests ship and are served like any other. Their
validity state is `deferred` instead of `local`, as defined in §5.1: trusted
as written until the first `capture` checks the bytes, re-checked on every
capture, a match expiring after 30 days, and a mismatch invalidating the
package version's deferred digests until re-extraction.

Nothing in §6 remains open. Applying Rule B to the 56 missing documents waits
on the measurement in §10.5, which is a sequencing condition, not an open
policy question.

### 6.4 Facts versus expression

This is a working distinction, not legal advice.

| Datasheet content | Character | Digest rule |
|---|---|---|
| Numeric values, pin numbers and names, dimensions, order codes, ratings, formulas | Facts | Transcribed (§4.1 rule 2) |
| The selection and arrangement of a large table, copied whole | Arguably protected as a compilation. In the EU, substantial extraction from a database can be restricted | Transcribed values, laid out in the digest's own section structure |
| Descriptive prose, application guidance, layout advice | Expression | Our own words, never quoted (§4.1 rule 3) |
| Drawings, figures, curves, photographs | Expression | Never reproduced; located by the page map |
| A vendor's website terms of use | A contractual limit, per vendor | An explicit prohibition of redistribution keeps the PDF out (Rule B condition 1); its digest still ships (Rule A) |

## 7. Extraction process and quality bar

1. **Draft.** A model drafts the digest from the pinned bytes only: the shipped
   PDF, or a download whose SHA-256 matches the evidence record. It works from
   rendered pages and text extraction (`LIBRARY.md` §2: `pdfinfo`, `pdftotext`,
   `shasum -a 256`), and fills in `extracted`. A draft may be committed into
   the package with `status: "draft"`. It is never served (§8.3).
2. **Verify.** A person checks every number against its cited page in the
   rendered PDF. They check every own-words paragraph against its cited page
   for meaning, and confirm it is not a quotation. They check the page map row
   by row. Drawings are checked visually: `LIBRARY.md` warns that "Text
   extraction alone will not reveal mirrored geometry, diagram orientation, or
   drawing-layer mistakes". Only then does the reviewer set `reviewed` and
   `status: "reviewed"`.
3. **Check.** A checker run in CI enforces the following. It is
   `tools/part_notes.py`, extended to the merged schema, keeping its existing
   device-pin, identity and LCSC checks.
   - the header schema;
   - the §5.1 validity rule against the package's own files;
   - all 23 headings (sections 0–22) present and in order;
   - every table row cites a page;
   - **every cited page is within `1..pages`, and `pages` equals the real page
     count** of the shipped PDF (§11.3);
   - a non-empty Gaps section;
   - `device`, `parts` and `identities` resolve to real library declarations
     and AVL entries;
   - `Not yet extracted` appears only in a draft.
4. **Accept.** The digest passes the replay acceptance test in §10.
5. **Commit.** One digest per commit, which covers one source document; for a
   single-part datasheet that is one part. A family document covers its listed
   parts in the same commit. Each digest addition or change is a patch version
   bump of its package (decision 3). A patch release may batch several digest
   commits.

**A digest may be committed into a package with `status: "draft"`.** This
replaces the earlier rule that kept drafts out of packages. The owner chose to
move the 100 converted part notes into their packages, and a new digest draft
may be committed the same way.

Two rules make that safe for agents:
- **The consumer never serves a draft** (§8.3). A draft is never a value
  source.
- **A draft never counts toward acceptance coverage** (§11.2). Only a
  `reviewed` digest passes §7 step 4.

The cost is in §4.5: draft text travels to every install, and its errors are
visible on disk even though they are never served.

## 8. Consumer protocol (cohdl-ai)

### 8.1 Options considered

The draft and the discussion after it labelled these options with letters,
and the letters were used inconsistently. **The letters are retired.** The
options are named by mechanism:

- **Option 1: an explicit `digest` action.** A new `PdfView` action that
  takes the same `url` and `expectedSha256` as `capture`. It returns the
  digest, or a page map, as text.
- **Option 2: a `find` branch.** When `find` runs on a kept document with a
  valid digest, it returns digest rows first, then PDF text excerpts. This
  requires a successful `capture` first.
- **Option 3: resolution inside `capture`, before fetch.** `capture` looks up a
  valid digest before downloading and returns it in place of, or alongside,
  the render.

(Traceability only: in the draft, Option 1 was B, Option 2 was A, and Option 3
was C.)

### 8.2 Decision

**The owner chose Option 1, an explicit `digest` action.** His reasons:

- PDF reading must stay available whenever it is genuinely needed.
- The prompt and tool descriptions are being changed anyway, so teaching the
  agent one more call is acceptable.

Option 2 is not part of this design. Option 3 is not adopted, and `capture`'s
semantics do not change.

### 8.3 The `digest` action

**Request.**

```json
{"request": {"action": "digest", "url": "<url>", "expectedSha256": "<64 hex>"}}
```

`url` and `expectedSha256` are exactly what the agent would pass to
`capture`. For a bundled PDF, `CohdlDocs` already returns both. The action
takes no page and returns no image.

**Lookup.**

1. If `url` is a registry document URL
   (`<registry>/api/doc?pkg=…&version=…&path=docs/<basename>.pdf`, the form
   built by `registryPdfUrl` in `apps/compile-service/src/docs-references.ts`),
   fetch `docs/<basename>.digest.md` from the same package and version. Then
   validate it (§5.1) against `expectedSha256`.
2. If `url` is any other URL (a vendor URL for a hash-only document), look up
   `expectedSha256` among the `source.sha256` values of the digests in the
   project's locked package closure, read from `cohdl.lock` through the same
   registry path. Rule A guarantees that a hash-only document's digest is in
   its package, so this lookup is what makes `deferred` digests reachable. If
   no digest matches, the result is empty (§13 Q3).
3. Validate the digest under §5.1, and attach its state: `local`, `deferred`
   (with the date of the last match, if any), or invalid. An invalid digest
   gives shape `none`.

**Response.** Text only, one of three shapes:

| Shape | When | Content |
|---|---|---|
| `digest` | A valid digest exists | The digest; the package, version and digest path; the source SHA-256; the validity state (`local` or `deferred`, §5.1); the "describes revision X pinned by…" line (§5.2) |
| `page_map` | No valid digest, but the package ships the PDF | A machine-generated page map: each page's first lines of extracted text, plus any PDF outline entry for that page. It is marked "generated, unreviewed" |
| `none` | Neither, or the only digest is a draft | An empty result with no error. The agent proceeds to `capture`. For a draft, the response may carry one line: `A draft digest exists for this document but is unreviewed; it is not a value source.` |

**Drafts: a hard rule.** The `digest` action returns only digests whose
`status` is `"reviewed"`. For a draft it behaves exactly as if no digest
existed: shape `none` (or `page_map`, if the package ships the PDF), with at
most the one-line notice above. It never returns a draft's values, sections
or page map. A draft's page map is unreviewed, and the generated page map
serves that need without presenting draft text.

The `page_map` shape is the owner's first mitigation (§8.4). It needs the PDF
bytes from the registry, our own origin, not the vendor's, so bot protection
and HTML-instead-of-PDF responses do not apply. It is cached by SHA-256. It
uses the same text extraction that `find` already uses. A drawing with no text
layer yields empty rows, which the page map shows as "no text on this page".

**Requirements.**

- `capture` with an `expectedSha256` that matches a `deferred` digest records
  the match or mismatch under §5.1. This is bookkeeping on the existing hash
  check; it does not change what `capture` returns.
- `digest` returns no image and does not count against image caps. The
  floorplan's six-image cap (`apps/agent-worker/src/floorplan.ts:32`,
  `FLOORPLAN_PDF_IMAGES = 6`) counts only captures, views and re-reads.
- Any failure while resolving a digest or page map returns `none`. It is
  logged, never surfaced as a tool error, and never reported to the user.
- Each call records:
  - the shape returned;
  - the reason a digest was rejected;
  - the package, version and source SHA-256;
  - real start and end times.
- Facts the agent takes from a digest are cited as the source URL, SHA-256 and
  page, plus the digest path. They are not cited as a capture id.
  - `apps/studio/lib/prompts/system.ts:646–648` already accepts a datasheet
    citation that carries "url (https:// manufacturer document) and/or source
    = the PdfView capture id plus page".
  - Claims that need visual verification still need a capture. For example,
    `apps/agent-worker/src/tools-design/component-selection.ts:137` treats a
    branch with "zero successful PdfView observations" as unverified.
  - `component-evidence.ts` should record a digest result as evidence of kind
    "digest", distinct from a captured page.

### 8.4 Known risk of this choice and its mitigations

**Risk.** This was why Options 1 and 2 were the weaker candidates in the
draft. Agents may skip an optional step and call `capture` directly. The
inventory shows a strong habit of rendering pages (934 images against 228
searches). Nothing in Option 1 forces the agent to call `digest`. If it does
not, page images and fetch failures stay where they are.

**Mitigations.** None of these changes `capture`'s semantics.

1. **The `digest` action returns a page map even when no digest exists**, as
   long as a package ships the document (§8.3, shape `page_map`). A cheap call
   therefore always tells the agent something useful, such as which page names
   the exact MPN, before it spends a render.
2. **`capture` and `view` results carry a one-line, machine-generated hint**
   when a valid digest or a page map exists for that document:
   `A digest exists for this document; call digest to see values without rendering.`
   (or `A page map exists for this document; call digest to see it.`). This is
   a notice only: no behaviour change, no refusal, and no extra image.
3. **The `PdfView` description states the intended order** in one short
   sentence (§9.2): digest first for values; `capture`/`view` when the digest
   does not answer.

§10.2 measures whether these mitigations work: digest adoption, and captures
where the hint fired but the agent rendered anyway.

## 9. Prompt and tool-description changes

These are the text changes Option 1 needs, file by file. Paths and line
numbers are cohdl-ai `origin/main` at `ccc37abd`. Each change says why it is
worth its tokens. The changes are proposed, not applied.

### 9.1 The `digest` action: schema and description

**File:** `apps/agent-worker/src/tools-documents/index.ts`

- **Lines 20–23:** add a fourth request variant beside `capture`, `view`,
  `read` and `find`:
  `Type.Object({ action: Type.Literal("digest"), url: Type.String({ minLength: 1, maxLength: 4096 }), expectedSha256: Type.String({ pattern: "^[a-f0-9]{64}$" }) }, { additionalProperties: false })`.
  - `expectedSha256` is required here, although it is optional on `capture`,
    because validity is defined by the hash (§5.1).
- **Line 64:** the description string currently begins with the `find`
  paragraph. Prepend this paragraph:

  > Before rendering a datasheet page for a value, call {"request":{"action":"digest","url":"<url>","expectedSha256":"<sha256>"}} with the same url and sha256 you would give capture (CohdlDocs returns both for a bundled PDF). It returns text only, never an image: the library's reviewed digest of that exact PDF (values with their page numbers, gaps listed), or a generated page map, or nothing. A digest marked deferred has not yet been checked against the PDF's bytes: cite its values as digest values, not inspected pages. No result is normal and is not an error: go straight to capture and do not mention it.

  Why it earns its tokens:
  - The first sentence names the call and when to make it, which is what
    changes behaviour.
  - The argument note lets the agent reuse the arguments it already has from
    `CohdlDocs`, so the call costs no extra lookup.
  - "text only, never an image" says why it is cheaper.
  - The "deferred" sentence is the only place the agent learns that Rule C
    exists. Without it, a not-yet-checked value would be cited as if a page had
    been seen.
  - The last sentence is the rule in §9.4. Without it, agents tend to report
    the empty result to the user as a failure.

### 9.2 The one-line ordering rule

**File:** `apps/agent-worker/src/tools-documents/index.ts`, line 65. This line
holds the existing ordering guidance ("After the first successful capture of a
document, use action view…"). Insert at its start:

> Order: digest first for values; capture/view only for what the digest does not answer or when you must see a drawing.

Why: one sentence states the sequence the risk in §8.4 depends on. The words
"or when you must see a drawing" keep the owner's reason, that PDF reading
stays available when genuinely needed, inside the rule itself.

### 9.3 The system-prompt line that makes it a habit

**File:** `apps/studio/lib/prompts/system.ts`.

- **Line 403**, the `PdfView` bullet in `COHDL_SYSTEM_PROMPT`. After "Reuse its
  captureId with action view to inspect more pages or crops from the same
  retained PDF; action read replays the saved image.", add:

  > For any datasheet value, call PdfView digest first with the url and sha256 you have; render pages only for what it leaves open or for drawings you must transcribe.

  Why: the tool description is read when a tool is chosen, but the system
  prompt sets habits before any choice is made. "For any datasheet value"
  makes the call the default rather than an option. "drawings you must
  transcribe" keeps `cohdl-implementation.ts:281`'s capture requirement
  intact.

- **Lines 164–165**, in the `CohdlDocs` bullet, change "For a bundled PDF, use
  its returned url and sha256 with PdfView capture (expectedSha256), then view
  the relevant pages." to:

  > For a bundled PDF, pass its returned url and sha256 to PdfView digest first; capture (expectedSha256) and view only the pages it does not answer.

  Why: this is the existing instruction that sends the agent straight to
  `capture`. Leaving it unchanged would contradict the new line at 403.

**The same capture-first instruction appears in four more places.** Each
replacement is one clause, so the contradiction is not repeated:

| File and line (origin/main) | Current text | Change |
|---|---|---|
| `apps/agent-worker/src/tools-hdl/index.ts:1636` (`CohdlDocs` description) | "Bundled PDFs include a registry url and sha256: capture with PdfView using that url and expectedSha256, then view the needed pages." | "…: call PdfView digest with that url and sha256 first, then capture/view only the pages it does not answer." |
| `apps/compile-service/src/docs-references.ts:70` (the `pdf_reference` note) | "…use PdfView with request:{action:"capture",url,expectedSha256:sha256,page:…}" | Prefix with `use PdfView request:{action:"digest",url,expectedSha256:sha256} first; then` |
| `packages/skills/src/skills/component-selection.ts:109` | "use PdfView capture with that url and expectedSha256, then view the relevant pages." | "use PdfView digest with that url and expectedSha256 first, then capture/view the pages it does not answer." |
| `packages/skills/src/skills/pcb-floorplan.ts:190` | "Capture each bundled PDF once with PdfView, using the url and sha256 it returns…" | Insert before it: "Call PdfView digest with the url and sha256 first; its page map names the drawing pages." |

`apps/agent-worker/src/floorplan.ts:34` tells the agent, when the image cap is
hit, that "PdfView find (text only) still works". It should read "PdfView
digest and find (text only) still work". Why: at that point the digest is the
only route to values left.

**Capability version.** `apps/studio/lib/services/chat-session.ts:130` defines
`COHDL_CAPABILITIES` as `{ version: 62, … systemPrompt: COHDL_SYSTEM_PROMPT }`.
It hashes the tools and system prompt together (line 131), and
`packages/agent-db/src/store.ts:1180` compares that hash. A prompt change
alters the hash, so the version is presumably bumped with it. The owner of
that file should confirm the convention.

### 9.4 Missing or invalid digests are never errors

This is stated in three places so that no single omission undoes it:
- the tool description (§9.1, last sentence);
- the tool itself, which returns shape `none` with no error field (§8.3);
- the system-prompt line, which says to "render pages" for what the digest
  "leaves open", treating absence as ordinary.

The agent proceeds to `capture` silently. It does not tell the user that no
digest existed, and does not record it as an open item.

## 10. Acceptance test and measurement

### 10.1 Acceptance test: replaying real queries

Before a digest is released, the `find` queries recorded in the inventory are
replayed against it:

- **Query set.** Every one of the 228 recorded `find` queries that was issued
  against this document's SHA-256, and whose intent is one of the top 20
  (§1.4).
- **Covered.** A query counts as covered only if the digest answers it with a
  cited value: a row in the matching section that contains the asked-for value
  and its page. A Gaps entry is not coverage. It is reported separately as
  "answered by gap". An answer found only in an unreviewed text, such as a
  converted part-note draft, is not coverage (§11.2).
- **Bar.** At least 80% of the replayed top-20-intent queries are covered.
  This is the owner's target. It is measured per digest and reported in
  aggregate.

This is a measurement on a real query set, not a judgement call. The query
set, the covered criterion and the bar are all fixed before extraction
begins. Scoring is a binary decision per query, and the scored sheet is kept
with the pilot results.

The replay covers only queries that were actually issued against a document.
A document that was rendered often but rarely searched has few or no
queries; for the HRO drawing, the count against its hash is not yet known.
For such a document the test is weak, and the page-image metrics in §10.2
carry more weight.

### 10.2 Metrics

Before and after windows, per document and per package version:

| Metric | Definition | Expected direction |
|---|---|---|
| **Digest adoption (primary)** | Share of datasheet lookups whose first `PdfView` call on that document in a session is `digest` rather than `capture` | Up. Option 1 depends on it |
| `PdfView` calls per document per session | All actions | Down |
| Page images returned per document per session | `capture` + `view` + `read` renders | Down |
| Fetch-class failures | HTML-instead-of-PDF 44, timeout 21, bot refusal 13, 404/410 8, hash mismatch 3: **89** in the inventory | Down, but see the note below |
| Hint ignored | Captures and views whose result carried the §8.4 hint and which the agent followed with another render of the same document, with no `digest` call in between | Down. If high, the mitigations are not working |
| Replay coverage | §10.1 | At least 80% |
| Call duration | Measured start and end per call (§10.3) | Reported once measured; no claim before that |

**About fetch-class failures:**

- They disappear only for packages that ship the PDF, and only when `capture`
  uses the registry URL. Option 1 still lets `capture` run against a vendor
  URL, and hash-only documents (Rule B) have no registry copy.
- The metric's denominator needs the 27 unclassified failures resolved first
  (§11.4).

### 10.3 Timing

Real start and end times for every `PdfView` call, `digest` included, are
recorded from the pilot's first day. None exist today (§1.3), so there is no
"before" duration. The first window of the pilot is the baseline for any later
latency claim.

### 10.4 Rollout order

1. **Pilot: the HRO TYPE-C-31-M-12 family drawing.** One document; 34 calls in
   20 projects; reads concentrated on one page.
   - All 20 sections are filled for the exact MPN, whose sheet is page 4 by
     the repository's records (§1.5).
   - The page map covers every page, so an agent heading for page 1 is told
     that it is a variant sheet.
   - Before extraction, compare the inventory's hash for this document with
     `c5a4131f…`, and inspect pages 1–4 on a machine that has a PDF tool.
2. **Small parts whose reads concentrate on one page:** TPD1E05U06 (p7),
   USBLC6-2 (p1–2), RS2257 (p1–2), RT9080 (p1, p5) and TPS25982 (p1, p4).
   TPD1E05U06 is hash-only today. Under Rule A its digest still ships, and the
   SHA-256 lookup in §8.3 step 2 makes it reachable from the vendor URL. It is
   the pilot's first `deferred` digest, and it exercises the first-capture
   check in §5.1. No generated page map exists for it, because there are no
   shipped bytes to extract from.
3. **Large datasheets with many calls:** FUSB302B (131 calls in 16 projects),
   STM32F072CB (58 in 8) and RT6150A/B (39 in 8). These go last, after the
   pilot shows adoption.

Each step needs a package release and `cohdl update` in the measured projects.

**Success.** The pilot succeeds when digest adoption is measurably above
zero, page images for the HRO document fall, and replay coverage meets the
bar. If adoption stays low despite the §8.4 mitigations, the next step is the
prompt wording in §9, not more digests.

### 10.5 Size measurement: the gate for applying Rule B

This section first measured what the uniform PDF rule (decision 2) would
cost. Its numbers produced the thresholds in Rule B (§6.3). It is now the gate
for applying Rule B to the 56 documents that the evidence-only packages
reference but do not ship. There is no blanket migration: each document ships
or stays hash-only according to Rule B, once its real size is known. Its
digest ships either way (Rule A).

The sections below separate what was measured locally, with no download, from
what needs a download.

**Measured locally (2026-11-26, this worktree).** Sizes are file bytes. MB
means 10^6 bytes, MiB means 2^20 bytes.

| Quantity | Value |
|---|---|
| PDFs committed under `lib/` | 106 files, 207,622,642 bytes (198.0 MiB) |
| Distinct by SHA-256 | 104. Two pairs are duplicates: `ti-tlv90x2-snosda3h.pdf` in `soic` and `@contrib/comparator`, and `rp2350-hardware-design.pdf` in `@raspberrypi/mcu` and `passive`. Distinct bytes: 192,781,049 (183.9 MiB) |
| Packages holding PDFs | 54 of 66 |
| Largest per package | `passive` 25.1 MB (14 files); `connectors` 24.0 MB (8); `usb` 17.0 MB (9); `@raspberrypi/mcu` 16.5 MB (2); `@contrib/display` 11.0 MB (2) |
| Size distribution, per file | min 12,952 B; p10 0.28 MB; p25 0.61 MB; **median 1.26 MB** (1,260,561 B); mean 1.96 MB; p75 2.21 MB; **p90 4.45 MB** (4,446,023 B) |
| Largest file | `lib/connectors/docs/raspberry-pi-pico-datasheet.pdf`, 18,176,869 B, in `connectors` |
| Git object store (shared repository) | 265.6 MiB packed (`git count-objects -v`: `size-pack` 271,999 KiB) |
| Largest package archives today (all non-dot files, which is the tar file set) | `passive` 27.3 MB; `connectors` 24.1 MB |

**The evidence-only set.** It is **68 files in 17 packages**, not 68
packages:

| Package | Files | Package | Files |
|---|---|---|---|
| `connectors` | 12 | `switches` | 3 |
| `do214` | 9 | `@contrib/imu` | 2 |
| `sod` | 8 | `load-switch` | 2 |
| `sot` | 8 | `led` | 2 |
| `esd` | 7 | `mic` | 1 |
| `@contrib/ldo` | 4 | `cellular` | 1 |
| `passive` | 3 | `transistor` | 1 |
| `diode` | 3 | `@contrib/fuel-gauge` | 1 |
| | | `@contrib/charger` | 1 |

- **Catalog-scale versus normal.** None of the 17 is catalog-scale. The two
  packages that use the catalog-scale source-index exception, `@st/stm32` and
  `@espressif/esp32` (their `docs/README.md` files say so), hold no
  `.evidence.md` file. All 17 are normal packages.
- **Documents actually missing.** The first SHA-256 in each record is the
  document hash. In the one multi-hash record checked
  (`connectors/jae-sf72s006`), the second hash is a KiCad footprint, not a
  PDF.
  - 4 of those document hashes match a PDF already committed. Three records
    (`connectors`: `jst-xh`, `conmhf4-smd-g-t`, `yzp0048-20048-04025-03`)
    already have a sibling PDF.
  - That leaves **56 distinct documents** not in the repository.
  - 8 records carry more than one hash. Counting every unshipped hash gives
    64, which is an upper bound, since at least one extra hash is CAD.

**Estimate: what the 56 would add.** This is an estimate, not a measurement.
The assumption is that each missing document is the size of an existing
committed PDF at the given percentile.

| Assumption: each missing PDF equals the existing | Added bytes |
|---|---|
| p25 (0.61 MB) | 34.3 MB |
| median (1.26 MB) | 70.6 MB |
| p75 (2.21 MB) | 123.9 MB |
| p90 (4.45 MB), a pessimistic bound | 249.0 MB |

The working range is **34–124 MB (p25–p75), central 71 MB**. The assumption
may be biased. 25 of the 56 are in `do214`, `sod` and `sot`, which are mostly
package-outline documents for discrete parts. Their real sizes are unknown
here and may be smaller than the median. The estimate is not adjusted for
this.

**Effect on (i) the git repository.** 34–124 MB is +13% to +47% of today's
265.6 MiB pack, about +27% at the median. Two assumptions apply:
- git's zlib saves little on PDFs, whose streams are usually already
  compressed;
- anything committed stays in history permanently, even if it is removed
  later.

**Effect on (ii) registry package archives.** The archive is an uncompressed
tar of the package's files (`src/registry.rs`, `pack_tar`), so each package
grows by the full size of its new PDFs. The largest relative changes (p25–p75):

| Package | Today | After (p25–p75) |
|---|---|---|
| `do214` | 0.04 MB | 5.5–19.9 MB |
| `sod` | 0.04 MB | 4.9–17.7 MB |
| `sot` | 0.05 MB | 4.9–17.7 MB |
| `esd` | 0.61 MB | 4.9–16.1 MB |
| `connectors` | 24.1 MB | 27.8–37.4 MB |
| `passive` | 27.3 MB | 29.1–33.9 MB |

At the median, `connectors` (31.7 MB) would become the largest package
archive. Every project that installs any `connectors` part downloads it.

**Not derivable locally:**
- **The real size of each of the 56 documents.** This needs a download.
- **Whether any vendor explicitly prohibits redistribution.** This needs each
  vendor's terms to be read. It decides how many of the 56 move at all.
- **The registry's upload ceiling.** The worker has no explicit publish-size
  limit in `registry/src/worker/`. `CLAUDE.md` states that Cloudflare's
  request-body ceiling is plan-dependent, and the account's plan is not
  visible here.

**The measurement that still has to run.** No migration happens before it.
1. Download each of the 56 documents into a scratch directory outside the
   repository. Commit nothing.
2. Record, per document: bytes, SHA-256 (it must equal the record's hash),
   page count, and whether the vendor's terms explicitly prohibit
   redistribution, with the place they say so.
3. Sum per package, and recompute the two effects above from real numbers.
4. Delete the scratch directory.

**Applying Rule B: headroom per package, measured.** The table gives, for
each of the 17 packages, how many PDF bytes it could add before condition 3
or condition 4 triggers. That is the minimum of (27,288,503 B − archive
today) and (19 × archive today). Condition 2 applies to each file
separately.

| Package | Archive today | Addable before condition 3 or 4 | Missing docs | One median PDF (1.26 MB) fits? |
|---|---|---|---|---|
| `passive` | 27,288,503 B | 0 (condition 3: already the largest) | 3 | no |
| `transistor` | 4,824 B | 0.09 MB (condition 4) | 1 | no |
| `@contrib/fuel-gauge` | 5,054 B | 0.10 MB (condition 4) | 1 | no |
| `switches` | 8,580 B | 0.16 MB (condition 4) | 3 | no |
| `sod` | 37,603 B | 0.71 MB (condition 4) | 8 | no |
| `do214` | 43,852 B | 0.83 MB (condition 4) | 9 | no |
| `sot` | 45,543 B | 0.87 MB (condition 4) | 8 | no |
| `connectors` | 24,068,413 B | 3.22 MB (condition 3) | 6 | yes, about two |
| `esd` | 606,220 B | 11.52 MB (condition 4) | 7 | yes |
| `diode` | 625,863 B | 11.89 MB (condition 4) | 3 | yes |
| `load-switch` | 721,158 B | 13.70 MB (condition 4) | 2 | yes |
| `mic` | 1,136,834 B | 21.60 MB (condition 4) | 1 | yes |
| `@contrib/imu` | 5,120,371 B | 22.17 MB (condition 3) | 2 | yes |
| `cellular` | 3,448,082 B | 23.84 MB (condition 3) | 1 | yes |
| `@contrib/ldo` | 2,918,513 B | 24.37 MB (condition 3) | 4 | yes |
| `@contrib/charger` | 1,313,206 B | 24.95 MB (condition 3) | 1 | yes |
| `led` | 2,172,943 B | 25.12 MB (condition 3) | 2 | yes |

This is a forecast from measured archive sizes, not a result:
- **Almost certainly hash-only:** 33 of the 56 documents. That is `passive`
  (any addition), plus the six packages that cannot take even 0.9 MB:
  `transistor`, `@contrib/fuel-gauge`, `switches`, `sod`, `do214` and `sot`.
- **Undecided until measured:** the other 23, in the remaining ten packages.
  Most would fit at median size; `connectors` can take about 3.2 MB in total.

Their digests ship in every case (Rule A). The real sizes from the
measurement above decide each document.

## 11. Known defects to fix before the pilot

### 11.1 `tools/part_notes.py` checks citation form only

The checker's "Citations" rule (docstring) is: "Every row of the parameter,
rating, condition and pin tables cites a page of a listed source, as
`<id> p. N`." In the code:

- It applies the regex `\b(\w+) pp?\. ?\d` to the `Source` cell of rows in four
  sections: `Key parameters`, `Absolute maximum ratings`, `Recommended
  operating conditions` and `Pins`.
- It requires a match, and that the cited id is a listed source.
- It **never checks that the page exists** in the document.
- It **never opens the PDF**. The only use of a local PDF is hashing it
  ("Sources: … A `local` file must exist and hash to that SHA-256"), and the
  docstring says "Nothing is fetched."
- Citations in bullet sections and prose are not checked.

It cannot verify that a cited page actually holds the cited value.

### 11.2 The 100 part notes are unreviewed and are not coverage

All 100 existing part notes pass this checker, and all are `"draft"`. None has
been verified by a person, and passing the checker shows only that their
citations are well formed. They move into their packages as drafts (§4.5),
so they ship to every install while remaining unreviewed.

**Requirements.**
- **Not coverage.** In the acceptance test (§10.1), a query answered only by a
  draft digest (a converted part note or any other), or by any unreviewed
  text, is **not covered**. Coverage is counted only against a `reviewed`
  digest. The pilot's coverage figures must not include the drafts, even
  where a draft already holds the answer.
- **Never served.** The `digest` action never returns a draft's content
  (§8.3), so a draft cannot raise adoption or lower page images either. A
  metric that moves because of a draft indicates a consumer bug.

### 11.3 Requirement on the merged checker

The digest checker (§7 step 3) must, at minimum, reject any
citation whose page is outside `1..pages`, and must confirm `pages` against
the real page count of the shipped PDF. For a hash-only document, the
`.evidence.md` record must also record the page count, which it does not today.
A citation that names an existing page can still be wrong; only the human
verification in §7 step 2 catches that.

### 11.4 The failure classification does not add up

The eight named failure classes total 165, but the inventory counts 192
failures. 27 are unclassified. In addition, the 19 "no such
file/run/snapshot" failures merge three different causes into one message.

**Requirement.** Before the pilot, classify the 27 remaining failures, and
split the "no such file/run/snapshot" message into its three causes in the
worker. Until then, the fetch-failure metric in §10.2 has no reliable
denominator, and its before-and-after comparison cannot be stated.

## 12. Proposed `LIBRARY.md` text

This text is proposed, not applied. It replaces the single PDF rule proposed
in the earlier revision with Rules A, B and C (§6.3). Each rule is five lines.
The supporting detail (header fields, the 23 sections, the merge with part
notes, the review process) stays in this document, and the rules point to it.

**In §2, replacing "Save unmodified PDFs under the manufacturer library's
`docs/` directory" and its catalog-scale exception paragraph (Rule B):**

> **PDFs.** Ship each datasheet PDF unmodified in `docs/` unless a condition
> in the table below holds; then keep only `docs/<basename>.evidence.md` with
> the URL, revision, SHA-256, page count, pages used and the condition met.
> Condition 1 applies only if the record names where the vendor prohibits it.
> Thresholds 3 and 4 are provisional (`docs/pdf-digest.md` §6.3, §10.5).
>
> | # | The PDF stays out when | Threshold |
> |---|---|---|
> | 1 | The vendor explicitly prohibits redistribution | The prohibition is named in the record |
> | 2 | The PDF is too large | More than 8 MiB (8,388,608 B) |
> | 3 | The package archive would exceed the largest shipped today | More than 27,288,503 B (`passive`, measured 2026-11-26) |
> | 4 | The package archive would grow too much | With the candidate PDFs ÷ without them > 20 |

**A new subsection after §2, "Datasheet digests" (Rules A and C):**

> **Digests.** A document's digest, `docs/<basename>.digest.md` (schema
> `pdf-digest/1`), always ships in its package, with or without the PDF, under
> no size limit. Facts only: values transcribed, one page per row; advice in our
> own words with a page, never quoted; no lifted prose, figures or page images;
> gaps recorded, never inferred. Edits are patch bumps; drafts are never served.

> **Validity.** A digest is `local` when its PDF ships: CI and the lock file
> check its SHA-256 against that file. Otherwise it is `deferred`: served as
> written until a capture downloads the PDF; every capture re-hashes it, a
> match lapses after 30 days, and a mismatch invalidates the version's deferred
> digests until a re-extracted patch release.

**The acceptance checklist would gain:**

> - [ ] Each datasheet PDF ships in `docs/`, or its `.evidence.md` record
>       names which Rule B condition keeps it out (and, for condition 1, where
>       the vendor prohibits redistribution).
> - [ ] Each digest ships regardless of its PDF; each reviewed digest matches
>       its hash, cites an existing page for every value, meets the replay
>       coverage bar, and passes the digest checker.

## 13. Open questions

1. **Who owns digest review?** Options are the package maintainer, a named
   library reviewer, or the author of the matching part. The `reviewer` field
   must name a person, which the checker cannot verify.
2. **Do digests version with the package or ship separately?**
   **Resolved (decision 3, §6.1):** with the package, as a patch bump per
   digest change.
3. **Documents in no library package.** These are web documents fetched at
   design time. Hash-only package documents are settled: Rule A ships their
   digests, and §8.3 step 2 finds them by SHA-256 in the project's locked
   closure. What remains open:
   - a document outside every package gets `none` today;
   - a document read in many projects that maps to no package is a signal that
     a package is missing.
4. **Should the tool refuse to render a page a valid digest covers?**
   **Resolved: no.** The owner's decision requires that PDF reading stay
   available whenever it is genuinely needed. The §8.4 hint is the most the
   tool does.
5. **Do digests and part notes merge?** **Resolved (decision 4, §6.1):** one
   schema. The mapping and the handling of the 100 drafts are in §4.5.
6. **Should a digest be attached to declarations through `#[doc]`?**
   `CohdlDocs` inlines attached Markdown into every lookup of that symbol,
   within a response byte budget. Attaching a 20-section digest would deliver
   it twice and could displace application notes from that budget. The
   recommendation is not to attach digests through `#[doc]` while `digest` is
   the access path.
7. **Venue and PDF rule.** **Resolved (decisions 1 and 2, §6.1; the
   catalog-scale exception, §6.2).**
8. **Page-map quality for drawings.** Drawings with no text layer produce
   empty generated page-map rows (§8.3). If the HRO pilot shows that, a
   reviewed page-map-only file may be worth allowing for documents that do not
   yet have a full digest.
