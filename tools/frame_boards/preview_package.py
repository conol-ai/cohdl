"""Build a frame board's assembler preview package with one command.

Usage: python3 -I tools/frame_boards/preview_package.py BOARD -o OUT.zip \
         --cohdl COHDL --kicad-cli KICAD_CLI --kicad-python KICAD_PYTHON [--allow-dirty]

BOARD names a directory under ``tools/frame_boards`` (``lcsc-batch-1``, ``c2-board-3``).  In
order, and stopping at the first failure:

1. the source tree is clean, unless ``--allow-dirty`` (the package then says so);
2. a board whose manifest is generated (``REGENERATE``) regenerates it, and the
   committed manifest must be the generator's output byte for byte;
3. ``jlc_frame_board.py`` builds the board, refuses a library row whose
   supplier BOM line lacks its part number, and plots Gerbers and drill files.
   A board directory with a ``cohdl.lock`` is a record: its packages are those
   of its ``inputs.json``, written from git, and the build must match the lock;
4. ``check_cpl.py`` reads the board three ways against the compiler's CPLs;
5. the BOM, both CPLs and the netlist are cross-checked: every placement once
   in each CPL, in the BOM and in the netlist; the BOM's part numbers are the
   manifest's; and the netlist's LCSC field is the BOM's part number where the
   BOM selected the primary entry, and absent where it selected an ``alt``.

Only when all pass is OUT.zip written.  It holds the three files the assembler
takes (``<name>-gerbers.zip``, ``<name>-lcsc-bom.csv``, ``<name>-lcsc-cpl.csv``),
the per-placement ``CHECKLIST.md``, the manifest and lock files, ``board.json``
(``measure_preview.py board``: each footprint and its pads, which the preview
script and the measurement read), ``verification.json`` and ``SHA256SUMS``.  Both ZIPs are written with fixed
timestamps; KiCad stamps a creation date into each Gerber, so the Gerber
archive differs between runs while the BOM and CPLs do not.

The package is for a placement preview only.  It is not uploaded here, and the
preview's verdict is not recorded here: a missing part, an ambiguous marker or
a failed measurement is "unverified", never a pass.
"""
import argparse, contextlib, csv, hashlib, io, json, os, re, subprocess, sys, tempfile, zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.insert(0, str(HERE))
sys.path.insert(0, str(ROOT / "tools"))
import check_cpl
import jlc_frame_board

# Boards whose manifest a generator writes from the repository's data.  C2
# frame boards 1 and 2 are records of their previews, pinned to the packages of
# their day, so they are not regenerated.  The W3 pilot is regenerated from its
# frozen inputs (w3-pilot-0402/inputs.json), so a later passive version leaves it as previewed.
REGENERATE = {"lcsc-batch-1": "lcsc_batch_1.py", "lcsc-batch-2-passive": "lcsc_batch_2_passive.py",
              "c2-board-3": "c2_board_3.py", "w3-pilot-0402": "w3_pilot_0402.py",
              "preview-board-4": "preview_board_4.py"}

# The layers the assembler fabricates from; courtyard, fab and user layers stay out.
GERBER_LAYERS = ("F_Cu", "B_Cu", "F_Mask", "B_Mask", "F_Paste", "B_Paste",
                 "F_Silkscreen", "B_Silkscreen", "Edge_Cuts")


class PackageError(ValueError):
    pass


def git(*argv):
    return subprocess.run(["git", "-C", str(ROOT), *argv], text=True, capture_output=True, check=True).stdout.strip()


def call(main, argv):
    """Run a tool's main() in-process; its output and whether it passed."""
    out = io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(out):
        code = main(argv)
    return code == 0, out.getvalue()


def fabrication_files(fabrication, name):
    """The Gerber and drill files to upload, by layer, in a fixed order."""
    files = []
    for layer in GERBER_LAYERS:
        found = sorted(fabrication.glob("%s-%s.*" % (name, layer)))
        if len(found) != 1:
            raise PackageError("expected one %s Gerber in %s, found %d" % (layer, fabrication, len(found)))
        files.append(found[0])
    drill = fabrication / (name + ".drl")
    if not drill.is_file():
        raise PackageError("no drill file %s" % drill)
    return files + [drill]


def write_zip(path, members):
    """A ZIP of (name, bytes) members with fixed timestamps and modes."""
    with zipfile.ZipFile(path, "w", zipfile.ZIP_DEFLATED) as archive:
        for name, data in sorted(members):
            info = zipfile.ZipInfo(name, date_time=(1980, 1, 1, 0, 0, 0))
            info.external_attr = 0o644 << 16
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, data)


def publish_zip(out_zip, members):
    """Write OUT.zip whole or not at all; an existing OUT.zip survives a failure."""
    out_zip.parent.mkdir(parents=True, exist_ok=True)
    fd, partial = tempfile.mkstemp(dir=out_zip.parent, prefix=out_zip.name + ".", suffix=".partial")
    try:
        with os.fdopen(fd, "wb") as handle:
            write_zip(handle, members)
        os.replace(partial, out_zip)
    finally:
        if os.path.exists(partial):
            os.unlink(partial)


def netlist_parts(net_text):
    """(ref, MPN field, LCSC field or None) for each component of a KiCad netlist."""
    try:
        root = jlc_frame_board.board_sexpr(net_text)
    except (IndexError, jlc_frame_board.BoardError):
        raise PackageError("the netlist is not a KiCad netlist")
    parts = []
    for components in jlc_frame_board.children(root, "components"):
        for comp in jlc_frame_board.children(components, "comp"):
            fields = {field[1][1]: field[2] for group in jlc_frame_board.children(comp, "fields")
                      for field in jlc_frame_board.children(group, "field") if len(field) > 2}
            parts.append((jlc_frame_board.children(comp, "ref")[0][1], fields.get("MPN"), fields.get("LCSC")))
    return parts


def audit_outputs(manifest, placements, bom_text, neutral_text, supplier_text, net_text):
    """Cross-check the supplier BOM, both CPLs and the netlist; a summary.

    The netlist carries an LCSC field exactly where the primary entry is bound
    (RFC-034 section 3.3).  So where the supplier BOM selected the primary, the
    field must be the BOM's part number; where it selected an ``alt``, the
    primary is unbound and the instance carries no field.
    """
    neutral = list(csv.DictReader(io.StringIO(neutral_text)))
    supplier = list(csv.DictReader(io.StringIO(supplier_text)))
    refs = [r["Designator"] for r in neutral]
    problems = []
    if len(refs) != placements:
        problems.append("the neutral CPL has %d placements, the manifest %d" % (len(refs), placements))
    if sorted(refs) != sorted(r["Designator"] for r in supplier) or len(set(refs)) != len(refs):
        problems.append("the neutral and supplier CPLs do not list the same placements once each")
    bom_refs, pns, selected = [], set(), {}
    for row in csv.DictReader(io.StringIO(bom_text)):
        bom_refs.extend(row["Designator"].split(","))
        pns.add(row["LCSC Part #"])
        selected.update((ref, (row["Comment"], row["LCSC Part #"])) for ref in row["Designator"].split(","))
    if sorted(bom_refs) != sorted(refs):
        problems.append("the supplier BOM does not list each placement once")
    if pns != {row["pn"] for row in manifest["rows"]}:
        problems.append("the supplier BOM names %s, the manifest %s" %
                        (", ".join(sorted(pns)), ", ".join(sorted({r["pn"] for r in manifest["rows"]}))))
    net = netlist_parts(net_text)
    if sorted(ref for ref, _, _ in net) != sorted(refs) or len({ref for ref, _, _ in net}) != len(net):
        problems.append("the netlist does not list each placement once")
    for ref, mpn, lcsc in net:
        if ref not in selected:
            continue
        chosen, pn = selected[ref]
        expected = pn if mpn == chosen else None
        if lcsc != expected:
            problems.append("%s: the netlist's LCSC field is %s, but the supplier BOM selected %s %s%s" %
                            (ref, lcsc or "absent", chosen, pn,
                             "" if expected else " as an alt of the unbound primary " + str(mpn)))
    if problems:
        raise PackageError("; ".join(problems))
    return {
        "placements": len(refs),
        "top": sum(r["Layer"] == "Top" for r in neutral),
        "bottom": sum(r["Layer"] == "Bottom" for r in neutral),
        "part_numbers": sorted(pns),
        "netlist_lcsc_fields": sum(lcsc is not None for _, _, lcsc in net),
    }


README = """{title} preview package, commit {commit}

Upload {name}-gerbers.zip as the Gerber, {name}-lcsc-bom.csv as the BOM and
{name}-lcsc-cpl.csv as the CPL. The Gerber archive holds copper, solder mask,
paste, silkscreen, outline and drill files.

The local checks in verification.json passed. The assembler preview has not
been run.

Expect {count} part numbers and {placements} placements: {top} top, {bottom} bottom.
Check pin 1 (ball A1 on a BGA) on every copy, with no manual rotation
correction. Record a missing part, an ambiguous marker or a failed measurement
as unverified, not passed.

Preview only; do not place an order.

CHECKLIST.md lists each placement. SHA256SUMS covers every other file here.
"""


def build(board, out_zip, cohdl, kicad_cli, kicad_python, allow_dirty, workdir):
    directory = HERE / board
    manifest_path = directory / "manifest.json"
    if not manifest_path.is_file():
        raise PackageError("%s has no manifest.json" % directory)
    commit = git("rev-parse", "HEAD")
    dirty = bool(git("status", "--porcelain"))
    if dirty and not allow_dirty:
        raise PackageError("the source tree has uncommitted changes; commit them or pass --allow-dirty")
    checks = {}

    generator = REGENERATE.get(board)
    if generator:
        regenerated = workdir / "manifest.json"
        run = subprocess.run([sys.executable, "-I", str(HERE / generator), str(regenerated)],
                             text=True, capture_output=True)
        if run.returncode or regenerated.read_bytes() != manifest_path.read_bytes():
            raise PackageError("%s is not what %s writes now; regenerate it\n%s%s" %
                               (manifest_path, generator, run.stdout, run.stderr))
        checks["manifest_regenerated"] = generator
    manifest = jlc_frame_board.load_manifest(manifest_path)
    name = manifest["name"]

    project = workdir / "board"
    pinned = []
    if (directory / "cohdl.lock").is_file():
        pinned = ["--packages", str(directory / "inputs.json"), "--lock", str(directory / "cohdl.lock")]
        checks["packages_pinned"] = "inputs.json, cohdl.lock"
    ok, log = call(jlc_frame_board.main, [str(manifest_path), "-o", str(project),
                                          "--cohdl", cohdl, "--kicad-cli", kicad_cli, *pinned])
    if not ok:
        raise PackageError("jlc_frame_board.py failed\n" + log)
    checks["library_bindings_in_supplier_bom"] = True

    ok, log = call(check_cpl.main, [str(project), "--kicad-python", kicad_python, "--kicad-cli", kicad_cli])
    if not ok:
        raise PackageError("check_cpl.py failed\n" + log)
    agree = re.search(r"kicad-cli pos: (\d+) of (\d+) rows agree", log)
    checks["smt_pos_neutral_equals_compiler"] = "smt_pos.py --neutral: equals" in log
    checks["smt_pos_mapped_equals_compiler"] = "smt_pos.py --map: equals" in log
    checks["kicad_position_side_top_angle_agree"] = "%s of %s" % agree.groups()
    checks["bottom_angles"] = "compared only by smt_pos.py against the compiler; KiCad encodes them its own way"

    out = project / "out"
    read = lambda path: path.read_text(encoding="utf-8")
    bom, cpl, neutral = (out / (name + "-lcsc-bom.csv"), out / (name + "-lcsc-cpl.csv"), out / (name + "-cpl.csv"))
    # jlc_frame_board.py lays the board out at its default 5 mm spacing.
    placements = len(jlc_frame_board.expand(manifest["rows"], 5)[0])
    summary = audit_outputs(manifest, placements, read(bom), read(neutral), read(cpl), read(out / (name + ".net")))
    checks["bom_cpl_netlist_cross_check"] = True

    board_json = workdir / "board.json"
    run = subprocess.run([kicad_python, "-I", str(HERE / "measure_preview.py"), "board",
                          str(out / (name + ".kicad_pcb")), str(board_json)], text=True, capture_output=True)
    if run.returncode:
        raise PackageError("measure_preview.py board failed\n" + run.stdout + run.stderr)

    gerbers = io.BytesIO()
    write_zip(gerbers, [(path.name, path.read_bytes()) for path in fabrication_files(project / "fabrication", name)])
    short = commit[:7] + ("-dirty" if dirty else "")
    verification = {
        "board": board,
        "commit": commit,
        "dirty": dirty,
        "cohdl_version": subprocess.run([cohdl, "--version"], text=True, capture_output=True).stdout.strip(),
        "kicad_cli_version": read(project / "fabrication/kicad-cli-version.txt").strip(),
        **summary,
        "checks": checks,
        "source_sha256": {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                          for path in (out / (name + ".kicad_pcb"), out / (name + "-lcsc-map.json"), neutral)},
        "supplier_preview": "not uploaded",
    }
    members = {
        name + "-gerbers.zip": gerbers.getvalue(),
        bom.name: bom.read_bytes(),
        cpl.name: cpl.read_bytes(),
        "CHECKLIST.md": (project / "CHECKLIST.md").read_bytes(),
        "manifest.json": manifest_path.read_bytes(),
        "cohdl.lock": (project / "cohdl.lock").read_bytes(),
        "design.lock": (project / "design.lock").read_bytes(),
        "board.json": board_json.read_bytes(),
        "verification.json": (json.dumps(verification, indent=2) + "\n").encode(),
        "README.txt": README.format(title=board, commit=short, name=name, count=len(summary["part_numbers"]),
                                    **summary).encode(),
    }
    members["SHA256SUMS"] = "".join("%s  %s\n" % (hashlib.sha256(data).hexdigest(), member)
                                    for member, data in sorted(members.items())).encode()
    publish_zip(out_zip, members.items())
    return verification


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("board")
    ap.add_argument("-o", "--output", type=Path, required=True)
    ap.add_argument("--cohdl", required=True)
    ap.add_argument("--kicad-cli", required=True)
    ap.add_argument("--kicad-python", required=True)
    ap.add_argument("--allow-dirty", action="store_true")
    args = ap.parse_args(argv)
    if args.output.suffix != ".zip":
        ap.error("-o must name a .zip file")
    try:
        with tempfile.TemporaryDirectory() as td:
            verification = build(args.board, args.output.resolve(), args.cohdl, args.kicad_cli,
                                 args.kicad_python, args.allow_dirty, Path(td))
    except (PackageError, jlc_frame_board.BoardError, subprocess.CalledProcessError, OSError) as exc:
        print("preview_package.py: " + str(exc), file=sys.stderr)
        return 1
    print("%s: %d placements, %d part numbers, KiCad positions %s" %
          (args.output, verification["placements"], len(verification["part_numbers"]),
           verification["checks"]["kicad_position_side_top_angle_agree"]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
