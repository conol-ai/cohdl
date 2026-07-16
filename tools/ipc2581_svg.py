#!/usr/bin/env python3
"""Render a CoHDL-emitted IPC-2581 document to a self-contained HTML/SVG so the
board can be visually inspected (board outline, every footprint's pads at their
absolute positions, locked-vs-staged components, ratsnest of the netlist).

Stdlib only (xml.etree), deterministic, no external deps — same discipline as
the emitter it inspects.

Usage:
    python3 tools/ipc2581_svg.py <file.xml> [out.html]
    # default out: <file>.html next to the input
"""
import sys
import xml.etree.ElementTree as ET

NS = "{http://webstds.ipc.org/2581}"


def tag(e):
    return e.tag.split("}", 1)[-1]


def find(e, name):
    return e.find(NS + name)


def findall(e, name):
    return e.iter(NS + name)


def num(s):
    return float(s) if s not in (None, "") else 0.0


def parse(path):
    root = ET.parse(path).getroot()
    step = None
    for s in root.iter(NS + "Step"):
        step = s
        break
    if step is None:
        sys.exit("no <Step> found — not a CoHDL IPC-2581 document?")

    # Primitive dictionary (Content/DictionaryStandard): id -> (shape, w, h).
    # Pins reference these via <StandardPrimitiveRef id=...> (the mainstream
    # encoding); inline primitives are still read as a fallback.
    prims = {}
    for entry in root.iter(NS + "EntryStandard"):
        pid = entry.get("id")
        rc = find(entry, "RectCenter")
        ci = find(entry, "Circle")
        ov = find(entry, "Oval")
        if rc is not None:
            prims[pid] = ("rect", num(rc.get("width")), num(rc.get("height")))
        elif ov is not None:
            prims[pid] = ("oval", num(ov.get("width")), num(ov.get("height")))
        elif ci is not None:
            d = num(ci.get("diameter"))
            prims[pid] = ("circle", d, d)

    # Board outline (Step/Profile/Polygon).
    outline = []
    prof = find(step, "Profile")
    if prof is not None:
        for p in prof.iter():
            if tag(p) in ("PolyBegin", "PolyStepSegment"):
                outline.append((num(p.get("x")), num(p.get("y"))))

    # Packages: name -> [ (num, x, y, shape, w, h) ].
    packages = {}
    for pkg in step.iter(NS + "Package"):
        # Only Packages that are direct Step children (skip none — all are).
        pins = []
        for pin in pkg.iter(NS + "Pin"):
            loc = find(pin, "Location")
            if loc is None:
                continue
            x, y = num(loc.get("x")), num(loc.get("y"))
            shape, w, h = "rect", 0.0, 0.0
            ref = find(pin, "StandardPrimitiveRef")
            rc = find(pin, "RectCenter")
            ci = find(pin, "Circle")
            ov = find(pin, "Oval")
            if ref is not None and ref.get("id") in prims:
                shape, w, h = prims[ref.get("id")]
            elif rc is not None:
                shape, w, h = "rect", num(rc.get("width")), num(rc.get("height"))
            elif ov is not None:
                shape, w, h = "oval", num(ov.get("width")), num(ov.get("height"))
            elif ci is not None:
                d = num(ci.get("diameter"))
                shape, w, h = "circle", d, d
            pins.append((pin.get("number"), x, y, shape, w, h))
        packages[pkg.get("name")] = pins

    # Components: refDes, packageRef, (x, y).
    comps = []
    for c in step.iter(NS + "Component"):
        loc = find(c, "Location")
        cx, cy = (num(loc.get("x")), num(loc.get("y"))) if loc is not None else (0.0, 0.0)
        comps.append((c.get("refDes"), c.get("packageRef"), cx, cy))

    # Netlist: net -> [ (refDes, pin) ] for the ratsnest.
    nets = []
    for net in step.iter(NS + "LogicalNet"):
        members = [(pr.get("componentRef"), pr.get("pin"))
                   for pr in net.iter(NS + "PinRef")]
        nets.append((net.get("name"), net.get("netClass"), members))
    return outline, packages, comps, nets


def bbox_of(pins, cx, cy):
    xs, ys = [], []
    for _, px, py, _, w, h in pins:
        xs += [cx + px - w / 2, cx + px + w / 2]
        ys += [cy + py - h / 2, cy + py + h / 2]
    if not xs:
        return cx - 0.5, cy - 0.5, cx + 0.5, cy + 0.5
    return min(xs), min(ys), max(xs), max(ys)


def render(path, out):
    outline, packages, comps, nets = parse(path)
    SCALE = 9.0        # px per mm
    PAD = 8.0          # mm margin

    # World bounds (board + every component's pads), y flipped for SVG.
    xs, ys = [], []
    if outline:
        xs += [p[0] for p in outline]
        ys += [p[1] for p in outline]
    comp_boxes = {}
    for ref, pkg, cx, cy in comps:
        b = bbox_of(packages.get(pkg, []), cx, cy)
        comp_boxes[ref] = b
        xs += [b[0], b[2]]
        ys += [b[1], b[3]]
    minx, maxx = min(xs) - PAD, max(xs) + PAD
    miny, maxy = min(ys) - PAD, max(ys) + PAD

    def X(x):
        return (x - minx) * SCALE

    def Y(y):
        return (maxy - y) * SCALE  # flip: +y up (IPC) -> down (SVG)

    W = (maxx - minx) * SCALE
    H = (maxy - miny) * SCALE

    # Is a component inside the board outline (locked) or outside (staged)?
    ob = None
    if outline:
        ob = (min(p[0] for p in outline), min(p[1] for p in outline),
              max(p[0] for p in outline), max(p[1] for p in outline))

    def locked(b):
        if not ob:
            return False
        return b[0] >= ob[0] - 1e-6 and b[2] <= ob[2] + 1e-6 and \
            b[1] >= ob[1] - 1e-6 and b[3] <= ob[3] + 1e-6

    # Pin absolute positions for the ratsnest (refDes,pin) -> (x,y).
    pinpos = {}
    for ref, pkg, cx, cy in comps:
        for pn, px, py, _, _, _ in packages.get(pkg, []):
            pinpos[(ref, pn)] = (cx + px, cy + py)

    s = []
    s.append(f'<svg viewBox="0 0 {W:.1f} {H:.1f}" xmlns="http://www.w3.org/2000/svg" '
             f'style="max-width:100%;height:auto;background:#0d1117">')
    s.append('<style>.ref{font:600 3px sans-serif;fill:#c9d1d9}'
             '.pin{fill:#e3b341}.pinp{fill:#f0b7c0}'
             '.rat{stroke:#3b5170;stroke-width:0.4;opacity:0.55}</style>')

    # Ratsnest (thin lines between the first pad and every other pad on a net).
    for name, cls, members in nets:
        pts = [pinpos[m] for m in members if m in pinpos]
        if len(pts) < 2:
            continue
        a = pts[0]
        for b in pts[1:]:
            s.append(f'<line class="rat" x1="{X(a[0]):.1f}" y1="{Y(a[1]):.1f}" '
                     f'x2="{X(b[0]):.1f}" y2="{Y(b[1]):.1f}"/>')

    # Board outline.
    if outline:
        pts = " ".join(f"{X(x):.1f},{Y(y):.1f}" for x, y in outline)
        s.append(f'<polyline points="{pts}" fill="#12341f" fill-opacity="0.5" '
                 f'stroke="#2ea043" stroke-width="1.5"/>')

    # Components: pads + refdes, colored by locked/staged.
    for ref, pkg, cx, cy in comps:
        pins = packages.get(pkg, [])
        b = comp_boxes[ref]
        lk = locked(b)
        col = "pinp" if lk else "pin"
        # component courtyard box (faint)
        s.append(f'<rect x="{X(b[0]):.1f}" y="{Y(b[3]):.1f}" '
                 f'width="{(b[2]-b[0])*SCALE:.1f}" height="{(b[3]-b[1])*SCALE:.1f}" '
                 f'fill="none" stroke="{"#f0b7c0" if lk else "#8b6914"}" '
                 f'stroke-width="0.3" stroke-opacity="0.5"/>')
        for pn, px, py, shape, w, h in pins:
            ax, ay = cx + px, cy + py
            if shape == "circle":
                s.append(f'<circle class="{col}" cx="{X(ax):.1f}" cy="{Y(ay):.1f}" '
                         f'r="{w/2*SCALE:.1f}"/>')
            elif shape == "oval":
                s.append(f'<ellipse class="{col}" cx="{X(ax):.1f}" cy="{Y(ay):.1f}" '
                         f'rx="{w/2*SCALE:.1f}" ry="{h/2*SCALE:.1f}"/>')
            else:
                s.append(f'<rect class="{col}" x="{X(ax-w/2):.1f}" y="{Y(ay+h/2):.1f}" '
                         f'width="{w*SCALE:.1f}" height="{h*SCALE:.1f}" rx="0.4"/>')
        s.append(f'<text class="ref" x="{X(cx):.1f}" y="{Y(cy):.1f}" '
                 f'text-anchor="middle" dominant-baseline="middle">{ref}</text>')
    s.append("</svg>")
    svg = "\n".join(s)

    n_locked = sum(1 for ref, _, _, _ in comps if locked(comp_boxes[ref]))
    dims = ""
    if ob:
        dims = f"{ob[2]-ob[0]:.1f} × {ob[3]-ob[1]:.1f} mm"
    html = f"""<!doctype html><html><head><meta charset="utf-8">
<title>IPC-2581 view — {path.split('/')[-1]}</title>
<style>body{{margin:0;background:#0d1117;color:#c9d1d9;font:14px system-ui,sans-serif;padding:16px}}
.stat{{display:inline-block;margin-right:24px}}.k{{color:#8b949e}}
.legend span{{display:inline-block;width:12px;height:12px;border-radius:2px;vertical-align:middle;margin:0 4px}}</style></head>
<body>
<h2 style="margin:0 0 8px">IPC-2581 — {path.split('/')[-1]}</h2>
<div style="margin-bottom:12px">
<span class="stat"><span class="k">board</span> {dims}</span>
<span class="stat"><span class="k">components</span> {len(comps)}</span>
<span class="stat"><span class="k">locked (inside outline)</span> {n_locked}</span>
<span class="stat"><span class="k">staged (to place)</span> {len(comps)-n_locked}</span>
<span class="stat"><span class="k">nets</span> {len(nets)}</span>
</div>
<div class="legend" style="margin-bottom:10px;font-size:13px">
<span style="background:#2ea043"></span>board outline
<span style="background:#f0b7c0"></span>locked / pre-placed pads
<span style="background:#e3b341"></span>staged pads
<span style="background:#3b5170"></span>ratsnest (net connectivity)
</div>
{svg}
</body></html>"""
    open(out, "w").write(html)
    print(f"wrote {out}  ({len(comps)} components, {n_locked} locked, {len(nets)} nets)")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    inp = sys.argv[1]
    out = sys.argv[2] if len(sys.argv) > 2 else inp.rsplit(".", 1)[0] + ".html"
    render(inp, out)
