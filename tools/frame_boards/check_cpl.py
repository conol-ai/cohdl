"""Cross-check a generated frame board's CPLs before it goes to a preview.

Usage: python3 -I tools/frame_boards/check_cpl.py BOARD_DIR \
         --kicad-python KICAD_PYTHON --kicad-cli KICAD_CLI [--supplier lcsc]

BOARD_DIR is a ``jlc_frame_board.py`` output.  Three independent readings of
its ``out/<name>.kicad_pcb`` must agree with the compiler's CPLs:

1. ``smt_pos.py --neutral`` under KiCad's pcbnew equals ``<name>-cpl.csv``
   byte for byte;
2. ``smt_pos.py --map <name>-<ID>-map.json`` equals ``<name>-<ID>-cpl.csv``
   byte for byte;
3. ``kicad-cli pcb export pos`` from the drill/place origin agrees with the
   neutral CPL in position (0.05 um), side and top-side angle.  KiCad writes
   bottom angles in its own encoding, so those are compared by (1) only.

Exit 0 when all agree, 1 otherwise.  Nothing in BOARD_DIR is written.
"""
import argparse, csv, io, subprocess, sys, tempfile
from pathlib import Path

TOOLS = Path(__file__).resolve().parents[1]


def compare_positions(kicad_text, cpl_text):
    """Mismatch descriptions between a KiCad CSV position export and a neutral CPL."""
    kicad = {r["Ref"]: r for r in csv.DictReader(io.StringIO(kicad_text))}
    cpl = {r["Designator"]: r for r in csv.DictReader(io.StringIO(cpl_text))}
    bad = []
    for ref, row in cpl.items():
        k = kicad.get(ref)
        if k is None:
            bad.append(f"{ref}: not in the KiCad export")
            continue
        x, y = float(row["Mid X"].removesuffix("mm")), float(row["Mid Y"].removesuffix("mm"))
        if abs(x - float(k["PosX"])) > 5e-5 or abs(y - float(k["PosY"])) > 5e-5:
            bad.append(f"{ref}: position {x},{y} against KiCad {k['PosX']},{k['PosY']}")
        if row["Layer"].lower() != k["Side"]:
            bad.append(f"{ref}: side {row['Layer']} against KiCad {k['Side']}")
        elif k["Side"] == "top" and (float(k["Rot"]) - float(row["Rotation"])) % 360 > 1e-6:
            bad.append(f"{ref}: angle {row['Rotation']} against KiCad {k['Rot']}")
    bad.extend(f"{ref}: only in the KiCad export" for ref in sorted(set(kicad) - set(cpl)))
    return bad


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("board_dir", type=Path)
    ap.add_argument("--kicad-python", required=True)
    ap.add_argument("--kicad-cli", required=True)
    ap.add_argument("--supplier", default="lcsc")
    args = ap.parse_args(argv)
    out = args.board_dir / "out"
    boards = sorted(out.glob("*.kicad_pcb"))
    if len(boards) != 1:
        print(f"check_cpl.py: expected one .kicad_pcb in {out}", file=sys.stderr)
        return 1
    board, name = boards[0], boards[0].stem
    neutral, supplier = out / f"{name}-cpl.csv", out / f"{name}-{args.supplier}-cpl.csv"
    failures = []
    with tempfile.TemporaryDirectory() as td:
        for label, extra, expected in (("smt_pos.py --neutral", ["--neutral"], neutral),
                                       ("smt_pos.py --map", ["--map", str(out / f"{name}-{args.supplier}-map.json")], supplier)):
            actual = Path(td) / expected.name
            run = subprocess.run([args.kicad_python, "-I", str(TOOLS / "smt_pos.py"), str(board), *extra, str(actual)],
                                 text=True, capture_output=True)
            if run.returncode or actual.read_bytes() != expected.read_bytes():
                failures.append(f"{label} differs from {expected.name}\n{run.stdout}{run.stderr}")
            else:
                print(f"{label}: equals {expected.name}")
        pos = Path(td) / "kicad-pos.csv"
        run = subprocess.run([args.kicad_cli, "pcb", "export", "pos", "--format", "csv", "--units", "mm",
                              "--use-drill-file-origin", "--output", str(pos), str(board)], text=True, capture_output=True)
        if run.returncode:
            failures.append(f"kicad-cli pos export failed\n{run.stdout}{run.stderr}")
        else:
            bad = compare_positions(pos.read_text(encoding="utf-8"), neutral.read_text(encoding="utf-8"))
            rows = len(neutral.read_text(encoding="utf-8").splitlines()) - 1
            print(f"kicad-cli pos: {rows - len({b.split(':')[0] for b in bad})} of {rows} rows agree")
            failures.extend(bad)
    for failure in failures:
        print("check_cpl.py: " + failure, file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
