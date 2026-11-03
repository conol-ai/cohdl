# Notice: third-party data in this directory

The repository's MIT license covers CoHDL's own tooling and the structure of
these files. It does not license the third-party facts and images recorded
here, and it grants no rights in them.

## What is third-party

- **Catalog facts** in `jlc_catalog/` and `lcsc_backfill/`: part numbers,
  manufacturer names, MPNs, packages, classes, stock counts and dates read from
  JLCPCB's public `partdetail` pages and sitemap. Only factual fields are
  recorded. No page is stored, and raw fetches stay in a local cache outside
  the repository (`docs/library-supplier-plan.md`, ground rules).
- **The candidate list** in `jlcparts/` is derived from the jlcparts project
  (<https://github.com/yaqwsx/jlcparts>), which compiles JLCPCB's public parts
  list. Its `provenance.json` records the snapshot and hashes. See that project
  for its terms.
- **Assembly-preview captures** in `jlc_previews/*/` (`top.jpg`, `bottom.jpg`)
  are JLCPCB's renders of this project's own frame boards, with JLCPCB's
  watermark and part models. They are kept as the primary evidence for the
  footprint-frame rows in `jlc_frames.json`, which cite them. JLCPCB's names,
  marks and renders remain theirs.

Part numbers such as `C2146` are JLCPCB/LCSC catalog identifiers, used here
only to name the parts they identify.

## If you redistribute

Keep this notice with the files. A rights holder who objects to an image or
record should open an issue; the evidence can be replaced by a textual
measurement record without changing any library output.
