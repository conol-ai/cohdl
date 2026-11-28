#!/usr/bin/env python3
"""Does a datasheet PDF ship in its package, or stay out with only its evidence record?

    python3 -I tools/datasheet_ship.py lib/<pkg> PDF [PDF ...]

Applies Rule B of docs/pdf-digest.md (section 6.3) to the PDFs a batch wants to add:

  2  a PDF larger than 8 MiB stays out;
  3  no PDF is added if the package's files would then exceed the largest package measured;
  4  no PDF is added if the package's files would then be more than 20 times what they are
     without the candidates.

Condition 1 (the vendor prohibits redistribution, and the record names where) is a person's
reading of the vendor's terms; this tool cannot decide it and says so. A copy LCSC serves is
not kept out by condition 1 (section 6.5, decided 2026-11-27).

"The package's files" is the byte sum of every file the registry packs (no dot file or dot
directory, `collect_files` in src/registry.rs, which follows a symbolic link to a directory),
the measure section 10.5 uses. Candidates with the same content count once.

Rule B applies when a PDF is added: a PDF already committed stays committed (section 6.3). A
candidate whose content is already in the package gets no verdict ("already in the package,
nothing to decide") and counts as part of the package, not as an addition.

Conditions 3 and 4 judge the candidates left after condition 2 as one set: the tool does not
choose a subset that would fit.

Prints one line per candidate and a summary. Exit 0 when no candidate stays out, 3 when any
does, 2 on a usage error. Reads files only; sends no request.
"""

import argparse
import hashlib
from pathlib import Path
import sys

MAX_PDF = 8 * 1024 * 1024  # condition 2
MAX_PACKAGE = 27_288_503   # condition 3: `passive`, measured 2026-11-26
MAX_GROWTH = 20            # condition 4


def packed_files(package: Path) -> list[Path]:
    """The files `collect_files` in src/registry.rs packs: no dot name, links followed."""
    out = []
    for entry in sorted(package.iterdir()):
        if entry.name.startswith("."):
            continue
        if entry.is_dir():
            out.extend(packed_files(entry))
        elif entry.is_file():
            out.append(entry)
    return out


def judge(package: Path, candidates: list[Path]) -> dict:
    """The verdict for each distinct candidate the package does not hold, and the numbers it rests on.

    `verdicts` maps a candidate's SHA-256 to 0 (ships) or the condition that keeps it out;
    `held` is the SHA-256 of each candidate already in the package, which gets no verdict.
    """
    distinct: dict[str, Path] = {}
    for pdf in candidates:
        distinct.setdefault(hashlib.sha256(pdf.read_bytes()).hexdigest(), pdf)
    files = packed_files(package)
    in_package = {hashlib.sha256(p.read_bytes()).hexdigest() for p in files if p.suffix.lower() == ".pdf"}
    held = {sha for sha in distinct if sha in in_package}
    base = sum(p.stat().st_size for p in files)
    verdicts = {sha: (2 if pdf.stat().st_size > MAX_PDF else 0) for sha, pdf in distinct.items() if sha not in held}
    added = sum(distinct[sha].stat().st_size for sha, v in verdicts.items() if v == 0)
    total = base + added
    set_condition = 3 if total > MAX_PACKAGE else 4 if added and total > MAX_GROWTH * base else 0
    if set_condition:
        verdicts = {sha: v or set_condition for sha, v in verdicts.items()}
    return {"base": base, "added": added, "total": total, "verdicts": verdicts, "paths": distinct, "held": held}


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("package", type=Path)
    parser.add_argument("pdf", type=Path, nargs="+")
    args = parser.parse_args(argv)
    if not (args.package / "cohdl.toml").is_file():
        parser.error(f"{args.package} is not a package (no cohdl.toml)")
    for pdf in args.pdf:
        if not pdf.is_file() or pdf.read_bytes()[:5] != b"%PDF-":
            parser.error(f"{pdf} is not a PDF")
    result = judge(args.package, args.pdf)
    for sha, pdf in result["paths"].items():
        if sha in result["held"]:
            verdict = "already in the package, nothing to decide"
        else:
            condition = result["verdicts"][sha]
            verdict = f"stays out (condition {condition})" if condition else "ships"
        print(f"{verdict}: {pdf.name}, {pdf.stat().st_size:,} B, sha256 {sha}")
    growth = f"{result['total'] / result['base']:.1f}x" if result["base"] else "no files without them"
    print(f"package files without the candidates: {result['base']:,} B; candidates under 8 MiB: {result['added']:,} B; "
          f"together {result['total']:,} B ({growth}; limits {MAX_PACKAGE:,} B and {MAX_GROWTH}x)")
    print("condition 1 (the vendor prohibits redistribution) is not checked here: read the vendor's terms")
    return 3 if any(result["verdicts"].values()) else 0


if __name__ == "__main__":
    sys.exit(main())
