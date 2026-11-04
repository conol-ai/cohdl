"""Drive the assembler's placement preview for one preview package, in Arc.

Usage: python3 -I tools/frame_boards/jlc_preview.py PACKAGE.zip [--run DIR] [--downloads DIR]

PACKAGE.zip is a ``preview_package.py`` output. Its SHA256SUMS is checked and it is
unpacked into the run directory (default ``~/.cache/cohdl-preview/runs/<stem>``,
outside the repository). Then, in the person's own Arc window, through Arc's
AppleScript ``execute javascript`` on the ``cart.jlcpcb.com`` tab only:

1. the Gerber archive goes into the quote page; the detected size is reported;
2. PCB Assembly is switched on, with Both Sides when the package has bottom parts;
3. NEXT twice, to the Bill of Materials step; the BOM and CPL go in, and
   "Process BOM & CPL" runs;
4. the BOM step must show the package's part numbers, every row selected, no
   stock warning, and the package's top and bottom designators;
5. NEXT to Component Placements; the 2D view; the preview's own whole-board
   exports of the top and the bottom land in the run directory.

It stops there. It never clicks anything that saves to the cart, quotes, orders or
pays, and it never ticks the assembly terms on the person's behalf. It reads the
page's DOM only, never a response of the site's own requests, and never issues a
request of its own. A sign-in page, a captcha or anything it cannot place stops it
with exit 2 and a message for the person. A check that fails exits 1: the preview is
then unverified, never passed. Measuring pin 1 on the exports is a separate step
(``measure_preview.py``).
"""
import argparse, base64, hashlib, json, re, shutil, subprocess, sys, time, zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import measure_preview

QUOTE = "https://cart.jlcpcb.com/quote"
TAB_PREFIX = "https://cart.jlcpcb.com/"
# Text the script must never click, whatever the element.
FORBIDDEN = re.compile(r"cart|order|pay|checkout|quote now|place|submit", re.I)
STOCK_WARNING = re.compile(r"insufficient|shortage|out of stock|not enough|unavailable|will not be assembled", re.I)


class Stop(Exception):
    """Exit 2: a person has to take over (sign-in, captcha, an unexpected page)."""


class Unverified(Exception):
    """Exit 1: a check failed; the preview is unverified."""


APPLESCRIPT = """on run argv
  with timeout of 20 seconds
    tell application id "company.thebrowser.Browser"
      repeat with t in tabs of front window
        if URL of t starts with "%s" then
          return execute t javascript (item 1 of argv)
        end if
      end repeat
      return "__no_tab__"
    end tell
  end timeout
end run""" % TAB_PREFIX


def js(code, attempts=4):
    """Run JavaScript in the cart.jlcpcb.com tab; its string result.

    A tab that is still loading does not answer, so a timeout is retried.
    """
    for _ in range(attempts):
        run = subprocess.run(["osascript", "-e", APPLESCRIPT, code], text=True, capture_output=True)
        if not (run.returncode and "-1712" in run.stderr):
            break
    else:
        raise Stop("Arc did not answer in time; check the tab for a dialog and rerun")
    if run.returncode:
        raise Stop("Arc did not answer (%s). Allow this terminal to control Arc under "
                   "System Settings > Privacy & Security > Automation, then rerun." % run.stderr.strip())
    out = run.stdout.strip()
    if len(out) >= 2 and out[0] == out[-1] == '"':
        out = json.loads(out)  # Arc returns a JavaScript string quoted and escaped
    if out == "__no_tab__":
        raise Stop("no %s tab in Arc's front window" % TAB_PREFIX)
    return out


def open_quote_tab():
    subprocess.run(["osascript", "-e", 'tell application id "company.thebrowser.Browser" to tell front window '
                    'to make new tab with properties {URL:"%s"}' % QUOTE], check=True, capture_output=True)


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


def expected_sides(upload):
    """{side: set of designators} from the package's supplier CPL."""
    sides = {"Top": set(), "Bottom": set()}
    cpl = next(upload.glob("*-cpl.csv"))
    for line in cpl.read_text(encoding="utf-8").splitlines()[1:]:
        ref, _, _, layer, _ = line.split(",")
        sides[layer].add(ref)
    return sides


def check_bom_step(text, part_numbers, sides):
    """The BOM step's text against the package; a list of problems."""
    problems = []
    m = re.search(r"(\d+)\s*parts detected\s*(\d+)\s*Parts confirmed", re.sub(r"\s+", " ", text))
    if not m or int(m.group(1)) != len(part_numbers) or int(m.group(2)) != len(part_numbers):
        problems.append("expected %d parts detected and confirmed, read %s" %
                        (len(part_numbers), m.group(0) if m else "no count"))
    words = set(re.findall(r"\b[A-Z]+\d+\b", text))
    missing = sorted(set(part_numbers) - words)
    if missing:
        problems.append("the BOM step does not list %s" % ", ".join(missing))
    for warning in STOCK_WARNING.findall(text):
        problems.append("stock warning: " + warning)
    for side, refs in sides.items():
        missing = refs - words
        if missing:
            problems.append("%s designators missing from the BOM step: %s" % (side, ", ".join(sorted(missing))))
    return problems


def pin1_marks(png):
    """How many magenta pin 1 marks an export shows (a BGA shows two)."""
    bmp = png.with_suffix(".bmp")
    subprocess.run(["sips", "-s", "format", "bmp", str(png), "--out", str(bmp)], check=True, capture_output=True)
    try:
        return len(measure_preview.dots(str(bmp), (0, 0, 1, 1), False))
    finally:
        bmp.unlink()


def export_capture(side, downloads, out, parts, attempts=3, settle=15, timeout=60):
    """The preview's own export of SIDE, once its models show a mark for each of PARTS.

    Right after a side switch the preview can draw the lands before the parts,
    and an export then shows no pin 1 marks; it is retried after a pause.
    """
    js("""(() => { const b = [...document.querySelectorAll("[class*=_nav-bar-item_]")].find(e => e.textContent.trim() === %s);
        b.click(); return "ok"; })()""" % json.dumps(side))
    for attempt in range(attempts):
        time.sleep(settle)
        wait("String(!/Rendering/.test(document.body.innerText))", lambda v: v == "true", 120, "the models")
        before = time.time()
        js('(() => { document.querySelector("[class*=_download-icon_]").click(); return "ok"; })()')
        end = time.time() + timeout
        while True:
            fresh = [p for p in downloads.glob("*JLCPCB*.png") if p.stat().st_mtime >= before - 1]
            if fresh:
                shutil.move(str(max(fresh, key=lambda p: p.stat().st_mtime)), out)
                break
            if time.time() > end:
                raise Stop("the %s export did not reach %s. If Arc asks to allow multiple downloads, allow it "
                           "and rerun" % (side.lower(), downloads))
            time.sleep(1)
        marks = pin1_marks(out)
        if marks >= parts:
            return out
        print("%s export shows %d pin 1 marks for %d parts; exporting again" % (side, marks, parts))
    raise Unverified("the %s export never showed a pin 1 mark for each part" % side.lower())


def run_preview(package, run, downloads):
    upload, verification = unpack(package, run)
    name = next(upload.glob("*-gerbers.zip")).name.removesuffix("-gerbers.zip")
    sides = expected_sides(upload)
    part_numbers = verification["part_numbers"]

    try:
        # Deferred: a navigation inside the call keeps Arc from answering it.
        js("setTimeout(() => location.assign(%s), 100), 'ok'" % json.dumps(QUOTE))
    except Stop:
        open_quote_tab()
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
    pick = """(() => { const b = [...document.querySelectorAll(".btn.btnlg")].find(e => e.textContent.trim() === %s);
        return b ? b.className : ""; })()""" % json.dumps(side)
    js(pick.replace("return b ? b.className", "if (b) b.click(); return b ? b.className"))
    wait(pick, lambda v: "cur" in v.split(), 20, "the %s selection" % side)
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
    text = wait("document.body.innerText", lambda v: "Parts confirmed" in v, 120, "the BOM match")
    time.sleep(3)
    text = body_text()
    unchecked = js("""String([...document.querySelectorAll("table .el-checkbox")].filter(b => !b.classList.contains("is-checked")).length)""")
    problems = check_bom_step(text, part_numbers, sides)
    if unchecked != "0":
        problems.append("%s BOM rows are not selected" % unchecked)
    if problems:
        raise Unverified("; ".join(problems))
    print("BOM step: %d part numbers detected and confirmed, %d top and %d bottom placements, no stock warning"
          % (len(part_numbers), len(sides["Top"]), len(sides["Bottom"])))

    click_button("NEXT")
    wait("String(!!document.querySelector('[class*=_download-icon_]'))", lambda v: v == "true", 60, "Component Placements")
    js('(() => { document.querySelector("[class*=_contentLeft_]").click(); return "ok"; })()')
    wait('document.querySelector("[class*=_switchButton_]").textContent.trim()', lambda v: v == "2D", 20, "the 2D view")
    preview = run / "preview"
    preview.mkdir(exist_ok=True)
    for side_name in ("Top", "Bottom"):
        if sides[side_name]:
            path = export_capture(side_name, downloads, preview / (side_name.lower() + ".png"), len(sides[side_name]))
            print("%s export: %s (sha256 %s)" % (side_name, path, hashlib.sha256(path.read_bytes()).hexdigest()))
    print("Stopped at Component Placements; nothing was added to the cart.")


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("package", type=Path)
    ap.add_argument("--run", type=Path)
    ap.add_argument("--downloads", type=Path, default=Path.home() / "Downloads")
    args = ap.parse_args(argv)
    run = args.run or Path.home() / ".cache/cohdl-preview/runs" / args.package.stem
    try:
        run_preview(args.package, run, args.downloads)
    except Stop as stop:
        print("jlc_preview.py: a person has to take over: %s" % stop, file=sys.stderr)
        return 2
    except Unverified as failure:
        print("jlc_preview.py: unverified: %s" % failure, file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
