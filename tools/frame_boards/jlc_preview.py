"""Drive the assembler's placement preview for one preview package, in Arc.

Usage: python3 -I tools/frame_boards/jlc_preview.py PACKAGE.zip [--board BOARD.json] [--run DIR] [--downloads DIR]
       python3 -I tools/frame_boards/jlc_preview.py --recheck TOP.png BOTTOM.png --board BOARD.json
                                                    [--cpl CPL.csv] [--attribution N] [--body-method N]

The second form touches no browser: it prints the ``checks.json`` of exports that
were already taken (BOTTOM.png may be ``-``), for the CPL's designators or, without
``--cpl``, every footprint of the board. ``--attribution 1`` and ``--body-method 1``
judge them as the records made before attribution 2 and body method 2 were.

PACKAGE.zip is a ``preview_package.py`` output. Its SHA256SUMS is checked and it is
unpacked into the run directory (default ``~/.cache/cohdl-preview/runs/<stem>``,
outside the repository). Then, in the person's own Arc window, through Arc's
AppleScript ``execute javascript``:

1. the run opens its own quote tab and works in that tab only, found by its
   window and tab ids; other tabs are never read or navigated, and if this one
   is closed, moved or leaves ``cart.jlcpcb.com`` the run stops;
2. the Gerber archive goes into the quote page; the detected size is reported;
3. PCB Assembly is switched on, with Both Sides when the package has bottom parts;
4. NEXT twice, to the Bill of Materials step; the BOM and CPL go in, and
   "Process BOM & CPL" runs;
5. the BOM step is read row by row: each designator must appear exactly once,
   on its CPL side, in a selected row showing its BOM part number; the count of
   parts detected and confirmed must match, with no stock warning;
6. NEXT to Component Placements; the 2D view; the preview's own whole-board
   exports of the top and the bottom land in the run directory. Each instance of
   the side (``board.json``) needs its own evidence: one pin 1 mark of its own,
   at its pad 1 (``measure_preview.owners``, ``mark_ok``), or, for a
   non-polar chip part (``measure_preview.is_chip``), its drawn body along its
   pads, mod 180, and centred on them, read from the drawn model alone
   (``measure_preview.body``, method 2: silkscreen beside the part is not part
   of the reading). While any instance lacks it, the export
   is taken again. After the last attempt, an instance whose model is not drawn
   is "unverified", and a drawn one without its evidence is for "manual review";
   neither passes. ``preview/checks.json`` records each instance's verdict.

It stops there. It never clicks anything that saves to the cart, quotes, orders or
pays, and it never ticks the assembly terms on the person's behalf. It reads the
page's DOM only, never a response of the site's own requests, and never issues a
request of its own. A sign-in page, a captcha or anything it cannot place stops it
with exit 2 and a message for the person. A check that fails exits 1: the preview is
then unverified, never passed. Exit 3: both exports are taken, but some instances
are unverified or for manual review. Measuring pin 1 on the exports is a separate
step (``measure_preview.py``).

A package built before ``preview_package.py`` added ``board.json`` takes
``--board``: ``measure_preview.py board`` of the package's ``.kicad_pcb``.
"""
import argparse, base64, csv, hashlib, json, math, re, shutil, subprocess, sys, tempfile, time, zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import measure_preview

QUOTE = "https://cart.jlcpcb.com/quote"
TAB_PREFIX = "https://cart.jlcpcb.com/"
# Text the script must never click, whatever the element.
FORBIDDEN = re.compile(r"cart|order|pay|checkout|quote now|place|submit", re.I)
STOCK_WARNING = re.compile(r"insufficient|shortage|out of stock|not enough|unavailable|will not be assembled", re.I)
# The run's own tab, (window id, tab id), from open_tab().
TAB = None


class Stop(Exception):
    """Exit 2: a person has to take over (sign-in, captcha, an unexpected page)."""


class Unverified(Exception):
    """Exit 1: a check failed; the preview is unverified."""


APPLESCRIPT = """on run argv
  with timeout of 20 seconds
    tell application id "company.thebrowser.Browser"
      repeat with w in windows
        if id of w is item 1 of argv then
          repeat with t in tabs of w
            if id of t is item 2 of argv then
              if URL of t does not start with "%s" then return "__left__"
              return execute t javascript (item 3 of argv)
            end if
          end repeat
        end if
      end repeat
      return "__gone__"
    end tell
  end timeout
end run""" % TAB_PREFIX

# Arc's reference to a tab it has just made does not resolve, so the new tab is
# found as the front window's one tab that was not there before.
OPEN_TAB = """tell application id "company.thebrowser.Browser"
  set known to id of every tab of front window
  set w to id of front window
  tell front window to make new tab with properties {URL:"%s"}
  set made to {}
  repeat with t in tabs of front window
    if known does not contain (id of t) then set end of made to (id of t)
  end repeat
  if (count of made) is not 1 then return "__unclear__"
  return w & linefeed & (item 1 of made)
end tell""" % QUOTE

AUTOMATION = "Allow this terminal to control Arc under System Settings > Privacy & Security > Automation"


def open_tab():
    """Open the run's own quote tab in Arc's front window; js() then works in it alone."""
    global TAB
    run = subprocess.run(["osascript", "-e", OPEN_TAB], text=True, capture_output=True)
    ids = run.stdout.split()
    if run.returncode or len(ids) != 2:
        raise Stop("could not open a tab in Arc (%s). Open an Arc window. %s, then rerun."
                   % (run.stderr.strip(), AUTOMATION))
    TAB = tuple(ids)


def js(code, attempts=4):
    """Run JavaScript in the run's own tab; its string result.

    A tab that is still loading does not answer, so a timeout is retried.
    """
    if TAB is None:
        raise AssertionError("no tab of the run's own; call open_tab() first")
    for _ in range(attempts):
        run = subprocess.run(["osascript", "-e", APPLESCRIPT, *TAB, code], text=True, capture_output=True)
        if not (run.returncode and "-1712" in run.stderr):
            break
    else:
        raise Stop("Arc did not answer in time; check the tab for a dialog and rerun")
    if run.returncode:
        raise Stop("Arc did not answer (%s). %s, then rerun." % (run.stderr.strip(), AUTOMATION))
    out = run.stdout.strip()
    if len(out) >= 2 and out[0] == out[-1] == '"':
        out = json.loads(out)  # Arc returns a JavaScript string quoted and escaped
    if out == "__gone__":
        raise Stop("the run's tab was closed or moved to another window; rerun")
    if out == "__left__":
        raise Stop("the run's tab left %s (a sign-in page?); sign in in another tab and rerun" % TAB_PREFIX)
    return out


def wait(code, test, timeout=60, what="the page"):
    end = time.time() + timeout
    while True:
        value = js(code)
        if test(value):
            return value
        if time.time() > end:
            raise Stop("timed out waiting for %s (last read: %s)" % (what, value[:200]))
        time.sleep(2)


def body_text():
    return js("document.body.innerText")


def require_signed_in():
    text = body_text()
    if re.search(r"captcha|verify you are human", text, re.I):
        raise Stop("the page asks for a captcha; solve it in Arc and rerun")
    shown = js("""String([...document.querySelectorAll("a,button,span,div")].some(e => e.childElementCount === 0 &&
        e.offsetParent && /^(Sign In|Log In|Login)$/i.test(e.textContent.trim())))""")
    if shown != "false":
        raise Stop("not signed in; sign in to JLCPCB in Arc and rerun")


def click_button(label):
    """Click the one visible button whose whole text is LABEL."""
    if FORBIDDEN.search(label):
        raise AssertionError("refusing to click %r" % label)
    result = js("""(() => { const b = [...document.querySelectorAll("button")].filter(e => e.offsetParent &&
        e.textContent.trim().toLowerCase() === %s); if (b.length !== 1) return "found " + b.length;
        setTimeout(() => b[0].click(), 100); return "clicked"; })()""" % json.dumps(label.lower()))
    if result != "clicked":
        raise Stop("expected one visible %r button, %s" % (label, result))


def inject(index, path, mime):
    """Put PATH into the INDEX-th file input, as a person choosing it would."""
    data = base64.b64encode(path.read_bytes()).decode()
    result = js("""(() => { try { const b = atob(%s); const u = new Uint8Array(b.length);
        for (let i = 0; i < b.length; i++) u[i] = b.charCodeAt(i);
        const inp = document.querySelectorAll("input[type=file]")[%d]; if (!inp) return "no input";
        const dt = new DataTransfer(); dt.items.add(new File([u], %s, {type: %s}));
        inp.files = dt.files; inp.dispatchEvent(new Event("change", {bubbles: true}));
        return "set " + u.length; } catch (e) { return "err " + e; } })()""" % (
        json.dumps(data), index, json.dumps(path.name), json.dumps(mime)))
    if result != "set %d" % path.stat().st_size:
        raise Stop("could not set %s: %s" % (path.name, result))


def unpack(package, run):
    with zipfile.ZipFile(package) as archive:
        members = {name: archive.read(name) for name in archive.namelist()}
    sums = dict(reversed(line.split("  ", 1)) for line in members["SHA256SUMS"].decode().splitlines())
    for name, data in members.items():
        if name != "SHA256SUMS" and sums.get(name) != hashlib.sha256(data).hexdigest():
            raise Unverified("%s does not match the package's SHA256SUMS" % name)
    upload = run / "upload"
    upload.mkdir(parents=True, exist_ok=True)
    for name, data in members.items():
        if "/" in name or name.startswith("."):
            raise Unverified("unexpected member %r in the package" % name)
        (upload / name).write_bytes(data)
    return upload, json.loads(members["verification.json"])


def natural(ref):
    m = re.match(r"(\D*)(\d*)", ref)
    return m.group(1), int(m.group(2) or 0), ref


def expected_placements(upload):
    """{designator: (part number, side)} from the package's supplier BOM and CPL."""
    sides = {}
    for line in next(upload.glob("*-cpl.csv")).read_text(encoding="utf-8").splitlines()[1:]:
        ref, _, _, layer, _ = line.split(",")
        sides[ref] = layer
    placements = {}
    with open(next(upload.glob("*-bom.csv")), encoding="utf-8", newline="") as f:
        for row in csv.DictReader(f):
            for ref in row["Designator"].split(","):
                placements[ref] = (row["LCSC Part #"], sides.get(ref))
    if set(placements) != set(sides):
        raise Unverified("the package's BOM and CPL list different designators")
    return placements


def on_side(placements, side):
    return {ref for ref, (_, s) in placements.items() if s == side}


# The BOM step's table: its column names and, per row, each cell's text, with
# the Select cell read as its checkbox's state.
BOM_TABLE = """JSON.stringify((() => { const t = [...document.querySelectorAll(".el-table.ComponentList")].filter(e => e.offsetParent);
    if (t.length !== 1) return {error: "the BOM step shows " + t.length + " part tables"};
    const heads = [...t[0].querySelectorAll(".el-table__header-wrapper th.is-leaf")].map(th => th.innerText.trim());
    const rows = [...t[0].querySelectorAll(".el-table__body-wrapper tbody tr.el-table__row")].map(tr => [...tr.cells].map(td => {
        const c = td.querySelector(".el-checkbox"); return c ? (c.classList.contains("is-checked") ? "__checked__" : "__unchecked__")
        : td.innerText; }));
    return {heads, rows}; })())"""
# A board with no bottom parts gets no Bottom Designator column (2026-11-07).
COLUMNS = ("Top Designator", "Matched Part Detail", "Select")


def check_bom_step(text, table, placements):
    """The BOM step's text and table against the package; a list of problems.

    Each designator must appear exactly once, in the column of its CPL side, in a
    selected row whose matched part is the BOM's part number for it.
    """
    problems = []
    part_numbers = sorted({pn for pn, _ in placements.values()})
    m = re.search(r"(\d+)\s*parts detected\s*(\d+)\s*Parts confirmed", re.sub(r"\s+", " ", text))
    if not m or int(m.group(1)) != len(part_numbers) or int(m.group(2)) != len(part_numbers):
        problems.append("expected %d parts detected and confirmed, read %s" %
                        (len(part_numbers), m.group(0) if m else "no count"))
    for warning in STOCK_WARNING.findall(text):
        problems.append("stock warning: " + warning)
    if "error" in table:
        return problems + [table["error"]]
    heads = table["heads"]
    sides = ("Top", "Bottom") if "Bottom Designator" in heads else ("Top",)
    needed = COLUMNS + (("Bottom Designator",) if any(side == "Bottom" for _, side in placements.values()) else ())
    if not all(name in heads for name in needed):
        return problems + ["the BOM table's columns are %s" % ", ".join(heads)]
    col = {name: heads.index(name) for name in COLUMNS + tuple(s + " Designator" for s in sides)}
    if len(table["rows"]) != len(part_numbers):
        problems.append("expected %d BOM rows, read %d" % (len(part_numbers), len(table["rows"])))
    seen = {}
    for number, cells in enumerate(table["rows"], 1):
        if len(cells) != len(heads):
            problems.append("BOM row %d has %d cells for %d columns" % (number, len(cells), len(heads)))
            continue
        matched = re.findall(r"^C\d+$", cells[col["Matched Part Detail"]], re.M)
        if not matched:
            # Some rows show the part number in one line with the part's name and
            # parameters, not on a line of its own.
            matched = sorted(set(re.findall(r"\bC\d{3,}\b", cells[col["Matched Part Detail"]])))
        pn = matched[0] if len(matched) == 1 else None
        if pn is None:
            problems.append("BOM row %d matches %s" % (number, ", ".join(matched) or "no part number"))
        if pn is None or cells[col["Select"]] != "__checked__":
            # Say what the row shows, so a person can see why it is unmatched or unselected.
            shown = "; ".join("%s: %s" % (h, re.sub(r"\s+", " ", c).strip()[:120])
                              for h, c in zip(heads, cells) if c.strip() and c != "__checked__")
            problems.append("BOM row %d shows %s" % (number, shown[:700]))
        if cells[col["Select"]] != "__checked__":
            problems.append("BOM row %d (%s) is not selected" % (number, pn))
        for side in sides:
            for ref in re.split(r"[\s,]+", cells[col[side + " Designator"]].strip()):
                if ref:
                    seen.setdefault(ref, []).append((pn, side, number))
    for ref in sorted(set(placements) | set(seen), key=natural):
        rows = seen.get(ref, [])
        if ref not in placements:
            problems.append("%s is in the BOM step but not in the package" % ref)
        elif not rows:
            problems.append("%s is missing from the BOM step" % ref)
        elif len(rows) > 1:
            problems.append("%s appears %d times in the BOM step (rows %s)" %
                            (ref, len(rows), ", ".join(str(r[2]) for r in rows)))
        elif rows[0][:2] != placements[ref]:
            problems.append("%s: the BOM step shows %s on the %s, the package %s on the %s" %
                            ((ref,) + rows[0][:2] + placements[ref]))
    return problems


def load_board(upload, board, placements):
    """The footprints of BOARD, or of the package's board.json; each CPL row must be one, on its side."""
    path = board or upload / "board.json"
    if not path.is_file():
        raise Unverified("the package has no board.json; pass --board with `measure_preview.py board` of its .kicad_pcb")
    doc = json.loads(path.read_text(encoding="utf-8"))
    sides = {fp["ref"]: "Bottom" if fp["bottom"] else "Top" for fp in doc["fps"]}
    if {ref: sides.get(ref) for ref in placements} != {ref: side for ref, (_, side) in placements.items()}:
        raise Unverified("%s does not hold the CPL's designators on the CPL's sides" % path)
    return doc


def inspect(png, outline, bottom, fps, attribution=measure_preview.ATTRIBUTION, body_method=measure_preview.BODY_METHOD):
    """What an export shows of each of FPS: the pin 1 marks (``measure_preview.marks``;
    under attribution 1 every magenta cluster), and per part whether its model is
    drawn and, for a chip part, its body reading."""
    with tempfile.TemporaryDirectory() as td:
        bmp = Path(td) / "export.bmp"
        subprocess.run(["sips", "-s", "format", "bmp", str(png), "--out", str(bmp)], check=True, capture_output=True)
        view = measure_preview.View(str(bmp), outline, bottom)
        readings = {}
        for fp in fps:
            shown = measure_preview.drawn(view, fp)
            readings[fp["ref"]] = {"drawn": shown, "body": measure_preview.body(view, fp, body_method)
                                   if shown and measure_preview.is_chip(fp) else None}
        return (measure_preview.dots(view, outline, bottom) if attribution == 1 else measure_preview.marks(view)), readings


PASS = ("mark", "body")


def verdicts(marks, fps, readings, attribution=measure_preview.ATTRIBUTION):
    """Each instance of FPS on its own evidence: {ref: (status, detail)}.

    - a chip part (``measure_preview.is_chip``) has no pin 1 mark; its drawn body
      must lie along its pads, mod 180, and be centred on them: "body";
    - every other part needs exactly one mark of its own
      (``measure_preview.owners``), at its pad 1 within
      ``measure_preview.MARK_ANGLE_TOL``: "mark". A part with no mark, with
      several, with its mark elsewhere, or of a kind that has no pin 1
      (``measure_preview.expects_mark``) goes to "manual review";
    - a part whose model is not drawn (bare pads and the centroid marker) is
      "unverified", and a drawn part without its evidence goes to "manual review".
    Neither of the last two passes, and no instance's evidence covers another's.

    Under attribution 1, as the records before 2026-11-12 were judged, "mark" said
    only that a mark's nearest footprint origin was this part's (``unmarked``).
    """
    if attribution == 1:
        missing = set(unmarked(marks, fps))
    else:
        owned = measure_preview.owners(marks, [fp for fp in fps if readings[fp["ref"]]["drawn"]])
    out = {}
    for fp in fps:
        ref, reading = fp["ref"], readings[fp["ref"]]
        if not reading["drawn"]:
            out[ref] = ("unverified", "no model drawn: bare pads and the centroid marker")
        elif measure_preview.is_chip(fp):
            b = reading["body"]
            if measure_preview.body_ok(b):
                out[ref] = ("body", "along its pads, %+.1f deg mod 180; centred within %.3f mm (limit %.3f)"
                            % (b["delta_mod_180"], b["offset_mm"], b["offset_limit_mm"]))
            else:
                out[ref] = ("manual review", "chip body not along or not centred on its pads: %s" % json.dumps(b, sort_keys=True))
        elif attribution == 1:
            out[ref] = ("manual review", "drawn, with no pin 1 mark of its own") if ref in missing else ("mark", "")
        elif not measure_preview.expects_mark(fp):
            out[ref] = ("manual review", "drawn; a part of this kind has no pin 1 mark, and its body is not read here")
        elif not owned[ref]:
            out[ref] = ("manual review", "drawn, with no pin 1 mark of its own")
        elif len(owned[ref]) > 1:
            out[ref] = ("manual review", "drawn, with %d marks of its own: which is pin 1 is read by hand" % len(owned[ref]))
        else:
            mark, pad = owned[ref][0], measure_preview.pad_1(fp)
            delta = measure_preview.delta_ccw(fp, pad, mark)
            where = ("pin 1 mark %.1f deg counter-clockwise from pad 1 about the origin, seen from the top "
                     "(limit %g either way), %.2f mm from pad 1"
                     % (delta, measure_preview.MARK_ANGLE_TOL, math.hypot(mark[0] - pad["x"], mark[1] - pad["y"])))
            out[ref] = ("mark", where) if measure_preview.mark_ok(delta) else ("manual review", "drawn; " + where)
    return out


def entries(side, found, fps, attribution=measure_preview.ATTRIBUTION):
    """``checks.json`` entries of one side's verdicts. An entry that rests on how marks
    are given to parts names the attribution; one without the key is attribution 1."""
    chip = {fp["ref"]: measure_preview.is_chip(fp) for fp in fps}
    return {ref: dict({"side": side.lower(), "status": status, "detail": detail},
                      **({"attribution": attribution} if attribution != 1 and status != "unverified" and not chip[ref] else {}))
            for ref, (status, detail) in found.items()}


def checks_text(checks):
    return json.dumps(dict(sorted(checks.items(), key=lambda kv: natural(kv[0]))), indent=1) + "\n"


def recheck(doc, exports, refs=None, attribution=measure_preview.ATTRIBUTION, body_method=measure_preview.BODY_METHOD):
    """The checks of exports already taken, as a run records them: EXPORTS is
    {"Top": png, "Bottom": png}, REFS the designators that were placed (all, if None)."""
    checks = {}
    for side, png in exports.items():
        fps = [fp for fp in doc["fps"] if fp["bottom"] == (side == "Bottom") and (refs is None or fp["ref"] in refs)]
        marks, readings = inspect(png, measure_preview.outline(doc), side == "Bottom", fps, attribution, body_method)
        checks.update(entries(side, verdicts(marks, fps, readings, attribution), fps, attribution))
    return checks


def unmarked(marks, fps):
    """Attribution 1: the designators in FPS that no mark falls on.

    A mark counts once, for the nearest footprint within whose reach it lies, so
    the second mark of a BGA cannot stand in for a part that was not drawn.
    """
    count = {fp["ref"]: 0 for fp in fps}
    for x, y, _ in marks:
        near = [(d, fp["ref"]) for fp in fps for d in [math.hypot(x - fp["x"], y - fp["y"])]
                if d <= measure_preview.reach(fp)]
        if near:
            count[min(near)[1]] += 1
    return sorted((ref for ref, n in count.items() if not n), key=natural)


EXPORT = '(() => { document.querySelector("[class*=_download-icon_]").click(); return "ok"; })()'


def export_capture(side, downloads, out, fps, outline, attempts=3, settle=15, timeout=480, reclick=240):
    """The preview's own export of SIDE, and the verdict on each of FPS (``verdicts``).

    Right after a side switch the preview can draw the lands before the parts,
    and an export then shows no pin 1 marks; while any instance lacks its
    evidence, the export is retried after a pause. After the last attempt the
    export is kept with what it shows: the instances still without evidence are
    "unverified" or for "manual review", never passed.
    """
    js("""(() => { const b = [...document.querySelectorAll("[class*=_nav-bar-item_]")].find(e => e.textContent.trim() === %s);
        b.click(); return "ok"; })()""" % json.dumps(side))
    short = None
    for attempt in range(attempts):
        time.sleep(settle)
        wait("String(!/Rendering/.test(document.body.innerText))", lambda v: v == "true", 120, "the models")
        before = time.time()
        js(EXPORT)
        end, again = time.time() + timeout, time.time() + reclick
        while True:
            fresh = sorted((p for p in downloads.glob("*JLCPCB*.png") if p.stat().st_mtime >= before - 1),
                           key=lambda p: p.stat().st_mtime)
            if fresh:
                time.sleep(2)  # a second click's file may still be landing
                fresh = sorted((p for p in downloads.glob("*JLCPCB*.png") if p.stat().st_mtime >= before - 1),
                               key=lambda p: p.stat().st_mtime)
                shutil.move(str(fresh[-1]), out)
                for k, extra in enumerate(fresh[:-1]):  # duplicates of this export, out of the person's Downloads
                    shutil.move(str(extra), out.with_name("%s-extra%d.png" % (out.stem, k + 1)))
                break
            if time.time() > end:
                raise Stop("the %s export did not reach %s. If Arc asks to allow multiple downloads, allow it "
                           "and rerun" % (side.lower(), downloads))
            # A large board's export can take minutes to land; only a long silence earns one more click.
            if again and time.time() > again:
                js(EXPORT)
                again = None
            time.sleep(1)
        marks, readings = inspect(out, outline, side == "Bottom", fps)
        found = verdicts(marks, fps, readings)
        last, short = short, sorted((ref for ref, (status, _) in found.items() if status not in PASS), key=natural)
        if not short:
            return out, found
        if short == last:  # two exports alike: a model that is not drawn will not be drawn by a third
            break
        if attempt + 1 < attempts:
            print("%s export shows no evidence yet on %s; exporting again" % (side, ", ".join(short)))
    print("%s export: still no evidence on %s after %d exports" % (side, ", ".join(short), attempt + 1))
    return out, found


def run_preview(package, run, downloads, board=None, select_matched=False):
    upload, verification = unpack(package, run)
    name = next(upload.glob("*-gerbers.zip")).name.removesuffix("-gerbers.zip")
    placements = expected_placements(upload)
    if sorted({pn for pn, _ in placements.values()}) != sorted(verification["part_numbers"]):
        raise Unverified("the package's BOM and verification.json list different part numbers")
    doc = load_board(upload, board, placements)
    sides = {side: on_side(placements, side) for side in ("Top", "Bottom")}

    open_tab()
    wait("String(location.pathname.startsWith('/quote') && document.readyState === 'complete' && "
         "/Add gerber file|Sign In/.test(document.body.innerText))", lambda v: v == "true", 60, "a fresh quote page")
    require_signed_in()

    inject(0, upload / (name + "-gerbers.zip"), "application/zip")
    detected = wait("(document.body.innerText.match(/Detected[^\\n]*/) || [''])[0]", bool, 120, "the Gerber")
    print(detected)

    js("""(() => { const h = [...document.querySelectorAll("h2")].find(e => [...e.childNodes].some(n => n.nodeType === 3 &&
        n.textContent.trim() === "PCB Assembly")); let box = h; for (let k = 0; k < 3; k++) box = box.parentElement;
        if (!/Assembly Side/.test(document.body.innerText)) box.querySelector(".switch-box").click(); return "ok"; })()""")
    wait("String(/Assembly Side/.test(document.body.innerText))", lambda v: v == "true", 20, "the PCBA options")
    side = "Both Sides" if sides["Bottom"] and sides["Top"] else ("Bottom Side" if sides["Bottom"] else "Top Side")
    # Several option groups can carry a button with the same text (a multilayer board's
    # quote has more groups than a two-layer one), so take the one nearest the
    # "Assembly Side" label: the fewest ancestors up to an element whose text holds it.
    pick = """(() => { const depth = e => { let k = 0; for (let a = e; a && a !== document.body; a = a.parentElement, k++)
            if (/Assembly Side/.test(a.innerText || "")) return k; return 1e9; };
        const all = [...document.querySelectorAll(".btn.btnlg")].filter(e => e.textContent.trim() === %s);
        const b = all.slice().sort((p, q) => depth(p) - depth(q))[0];
        return b ? b.className : ""; })()""" % json.dumps(side)
    # On a multilayer board the quote starts on the Economic PCBA type, which offers one
    # side only: the wanted side is then disabled until the Standard type is chosen.
    if js(pick.replace("return b ? b.className", "return b ? String(!!b.disabled)")) == "true":
        kind = pick.replace(json.dumps(side), json.dumps("Standard")).replace("Assembly Side", "PCBA Type")
        js(kind.replace("return b ? b.className", "if (b) b.click(); return b ? b.className"))
        wait(kind, lambda v: "cur" in v.split(), 20, "the Standard PCBA type (%s is not offered on Economic)" % side)
        print("PCBA Type: Standard, because %s is not offered on Economic for this board" % side)
    js(pick.replace("return b ? b.className", "if (b) b.click(); return b ? b.className"))
    try:
        wait(pick, lambda v: "cur" in v.split(), 20, "the %s selection" % side)
    except Stop as stop:
        # Say what the page offers, so a person can see why the selection did not take.
        seen = js(pick.replace('return b ? b.className : "";', """return JSON.stringify({
            candidates: all.map(e => ({cls: e.className, depth: depth(e), disabled: !!e.disabled,
                pointer: getComputedStyle(e).pointerEvents, group: ((e.parentElement || e).innerText || "").slice(0, 80)})),
            dialogs: [...document.querySelectorAll("[role=dialog], .el-dialog__wrapper, .el-message-box__wrapper")]
                .filter(d => d.offsetParent !== null).map(d => (d.innerText || "").slice(0, 200)),
            options: (() => { const h = [...document.querySelectorAll("h2")].find(e => [...e.childNodes].some(n =>
                n.nodeType === 3 && n.textContent.trim() === "PCB Assembly")); let box = h;
                for (let k = 0; box && k < 3; k++) box = box.parentElement; if (!box) return [];
                return [...box.querySelectorAll(".btn.btnlg")].map(e => e.textContent.trim().slice(0, 40) +
                    (e.className.split(" ").includes("cur") ? " [cur]" : "") + (e.disabled ? " [disabled]" : "")); })(),
            text: (() => { const h = [...document.querySelectorAll("h2")].find(e => [...e.childNodes].some(n =>
                n.nodeType === 3 && n.textContent.trim() === "PCB Assembly")); let box = h;
                for (let k = 0; box && k < 3; k++) box = box.parentElement;
                return box ? (box.innerText || "").replace(/\\s+/g, " ").slice(0, 1500) : ""; })()});"""))
        raise Stop("%s; the page shows %s" % (stop, seen[:4000]))
    agreed = js("""(() => { const r = document.evaluate("//*[contains(normalize-space(.), 'JLCPCB assembly Service')]",
        document.body, null, XPathResult.ORDERED_NODE_SNAPSHOT_TYPE, null); for (let i = r.snapshotLength - 1; i >= 0; i--) {
        const cb = r.snapshotItem(i).querySelector("input[type=checkbox]"); if (cb) return String(cb.checked); } return "none"; })()""")
    if agreed != "true":
        raise Stop("the assembly terms are not ticked; a person has to decide that")

    click_button("NEXT")
    wait("location.pathname", lambda v: v.startswith("/smt-order"), 60, "the PCBA order page")
    time.sleep(3)
    click_button("NEXT")
    wait("String(document.querySelectorAll('input[type=file]').length)", lambda v: v == "2", 60, "the BOM/CPL inputs")
    labels = js("document.body.innerText.match(/Add BOM File[\\s\\S]*Add CPL File/) ? 'ok' : 'no'")
    if labels != "ok":
        raise Stop("the BOM and CPL inputs are not where expected")
    inject(0, upload / (name + "-lcsc-bom.csv"), "text/csv")
    inject(1, upload / (name + "-lcsc-cpl.csv"), "text/csv")
    click_button("Process BOM & CPL")
    wait("document.body.innerText", lambda v: "Parts confirmed" in v, 120, "the BOM match")
    time.sleep(3)
    table = json.loads(js(BOM_TABLE))
    problems = check_bom_step(body_text(), table, placements)
    forced = []
    if problems and select_matched and "error" not in table and "Select" in table["heads"]:
        # Opt-in, for reading orientation only: the assembler matched a part but left its
        # row unticked (no quantity to offer). Tick such rows so the part is drawn. This
        # changes nothing but the quote's part selection; the run still stops at
        # Component Placements.
        sel, det = table["heads"].index("Select"), table["heads"].index("Matched Part Detail")
        for index, cells in enumerate(table["rows"]):
            pns = sorted(set(re.findall(r"\bC\d{3,}\b", cells[det])))
            if cells[sel] == "__unchecked__" and len(pns) == 1:
                state = js("""(() => { const t = [...document.querySelectorAll(".el-table.ComponentList")].filter(e => e.offsetParent)[0];
                    const tr = t.querySelectorAll(".el-table__body-wrapper tbody tr.el-table__row")[%d];
                    const c = tr && tr.querySelector(".el-checkbox"); if (!c) return "no checkbox";
                    if (c.classList.contains("is-disabled")) return "disabled";
                    c.click(); return "clicked"; })()""" % index)
                if state != "clicked":
                    raise Unverified("BOM row %d (%s) is not selected and cannot be: %s" % (index + 1, pns[0], state))
                forced.append((index + 1, pns[0]))
        if forced:
            time.sleep(3)
            table = json.loads(js(BOM_TABLE))
            problems = check_bom_step(body_text(), table, placements)
            for number, pn in forced:
                print("BOM row %d (%s) was matched but not selected; selected by this run to read its placement" % (number, pn))
            # A row ticked this way has no stock behind it: the warning is expected and is reported, not fatal.
            warnings = [p for p in problems if p.startswith("stock warning: ") or p.startswith("expected ")]
            problems = [p for p in problems if p not in warnings]
            for warning in sorted(set(warnings)):
                print("BOM step note (orientation-only run): " + warning)
    if problems:
        raise Unverified("; ".join(problems))
    print("BOM step: %d part numbers detected%s; each of %d top and %d bottom designators once, "
          "on its side, with its part number, selected%s"
          % (len(verification["part_numbers"]), "" if forced else " and confirmed", len(sides["Top"]), len(sides["Bottom"]),
             "; %d row(s) selected by this run, so this is an orientation-only preview" % len(forced) if forced
             else "; no stock warning"))

    click_button("NEXT")
    wait("String(!!document.querySelector('[class*=_download-icon_]'))", lambda v: v == "true", 60, "Component Placements")
    # A large board can still be rendering in 3D when the switch is first clicked.
    for attempt in range(3):
        js('(() => { document.querySelector("[class*=_contentLeft_]").click(); return "ok"; })()')
        try:
            wait('document.querySelector("[class*=_switchButton_]").textContent.trim()', lambda v: v == "2D", 30,
                 "the 2D view")
            break
        except Stop:
            if attempt == 2:
                raise
    preview = run / "preview"
    preview.mkdir(exist_ok=True)
    checks = {}
    for side_name in ("Top", "Bottom"):
        if sides[side_name]:
            fps = [fp for fp in doc["fps"] if fp["ref"] in sides[side_name]]
            path, found = export_capture(side_name, downloads, preview / (side_name.lower() + ".png"), fps,
                                         measure_preview.outline(doc))
            checks.update(entries(side_name, found, fps))
            print("%s export: %s (sha256 %s)" % (side_name, path, hashlib.sha256(path.read_bytes()).hexdigest()))
    (preview / "checks.json").write_text(checks_text(checks), encoding="utf-8")
    count = {s: sorted((r for r, c in checks.items() if c["status"] == s), key=natural)
             for s in ("mark", "body", "unverified", "manual review")}
    print("Instances: %d with their pin 1 mark at pad 1, %d chip bodies along their pads; unverified: %s; manual review: %s"
          % (len(count["mark"]), len(count["body"]), ", ".join(count["unverified"]) or "none",
             ", ".join(count["manual review"]) or "none"))
    print("Stopped at Component Placements; nothing was added to the cart.")
    return checks


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("package", type=Path, nargs="?")
    ap.add_argument("--board", type=Path, help="board.json for a package that carries none")
    ap.add_argument("--run", type=Path)
    ap.add_argument("--downloads", type=Path, default=Path.home() / "Downloads")
    ap.add_argument("--select-matched", action="store_true",
                    help="tick BOM rows the assembler matched but left unselected, to read their placement (orientation only)")
    ap.add_argument("--recheck", nargs=2, metavar=("TOP.png", "BOTTOM.png"), help="judge exports already taken; no browser")
    ap.add_argument("--cpl", type=Path, help="with --recheck: the CPL whose designators were placed")
    ap.add_argument("--attribution", type=int, choices=(1, 2), default=measure_preview.ATTRIBUTION)
    ap.add_argument("--body-method", type=int, choices=sorted(measure_preview.BODY_METHODS), default=measure_preview.BODY_METHOD)
    args = ap.parse_args(argv)
    if args.recheck:
        if args.package or not args.board:
            ap.error("--recheck takes --board and no package")
        refs = args.cpl and {line.split(",")[0] for line in args.cpl.read_text(encoding="utf-8").splitlines()[1:]}
        exports = {side: Path(png) for side, png in zip(("Top", "Bottom"), args.recheck) if png != "-"}
        sys.stdout.write(checks_text(recheck(json.loads(args.board.read_text(encoding="utf-8")), exports, refs,
                                             args.attribution, args.body_method)))
        return 0
    if not args.package:
        ap.error("a package, or --recheck")
    run = args.run or Path.home() / ".cache/cohdl-preview/runs" / args.package.stem
    try:
        checks = run_preview(args.package, run, args.downloads, args.board, args.select_matched)
    except Stop as stop:
        print("jlc_preview.py: a person has to take over: %s" % stop, file=sys.stderr)
        return 2
    except Unverified as failure:
        print("jlc_preview.py: unverified: %s" % failure, file=sys.stderr)
        return 1
    open_ = sorted((ref for ref, c in checks.items() if c["status"] not in PASS), key=natural)
    if open_:
        print("jlc_preview.py: exports complete; %d instances unverified or for manual review: %s (see %s)"
              % (len(open_), ", ".join(open_), run / "preview/checks.json"), file=sys.stderr)
        return 3
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
