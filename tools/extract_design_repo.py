#!/usr/bin/env python3
"""Extract markdown content from conol.ai share pages (Next.js RSC payload)."""
import re, json, sys, subprocess, os, tempfile, shutil

BASE = "https://conol.ai/share/note/tqrnb5h4l0dv8c14dkd17h4l"

PAGES = [
    ("", "00-root"),
    ("nssd0qpz0osiovfv203fn49t", "01-product-constitution"),
    ("lb94ql8w3qz3n6rs5ia28qyz", "02-conceptual-model"),
    ("zc04ruzizp1c3q8ha6xlht9k", "03-capability-architecture"),
    ("nxoipg3asjsp175ozyy5d1oy", "04-principle-constraint-mapping"),
    ("bppud8dqcjj98ra1c0mxk49k", "05-coherence-matrix"),
    ("x7ywj3w893y1946u7wv3yxxz", "06-rfc-process"),
    ("a5wwanan8ivgee7046cam8c3", "rfc-001-units-as-types"),
    ("vgd1lixpvf3v9hwgwzdzyyzw", "rfc-002-pin-connection-obligation"),
    ("n68etihsg24y3bn05j44sn9s", "rfc-003-trait-satisfaction"),
    ("v1ojvdn3z6g1s8zskrwsn5t0", "rfc-004-drc-reclassification"),
    ("d8sji1bc2t68w5gmu4lgs0op", "rfc-005-designator-allocation"),
    ("qwruqt4u7x5kk825l4462puz", "rfc-006-nested-fn-calls"),
    ("t3fyqa0jtybwxan1gmxpu9hf", "rfc-007-generics-over-specs"),
    ("ia8zki87haqo3gkuu8wu8uct", "rfc-008-pattern-matching"),
    ("e2nobir31c40nj0enpz8vs8d", "rfc-009-fmt"),
    ("oxda94xa4fqex2047ptae5cl", "rfc-010-check-json"),
    ("or92tinn1xe2e1ywre7e0ie3", "rfc-011-error-registry"),
    ("dpbn9l3f6lnigukmcs2uj9f0", "rfc-012-intent-annotations"),
    ("s37zw59ns3ie446giyd1m5c8", "rfc-013-layout-constraint"),
    ("k3b7n8le98xxe1qkvr1kwkrl", "rfc-014-lsp"),
    ("fbt1zl961m0gki2vegoucxiz", "rfc-015-ipc2581"),
    ("cdbgc70jo99mkek3szjdbecf", "rfc-016-modules"),
    ("e0avobmaiqrc4proh2og13r7", "rfc-017-library-registry"),
    ("um9yd84duu5ocdyyiie495wg", "rfc-018-footprint-format"),
    ("kvm12spp9cb0uq6x2fjvjmxo", "rfc-019-vscode-extension"),
    ("g3belxvx8z757g3hjr0uidvy", "rfc-020-board-outline-dxf"),
    ("nvrah0x566grszfdooa8ochf", "rfc-021-ipc7351-footprint-naming"),
    ("zuigdl4orqjy0mwqlxapdw0t", "rfc-022-mount-hole"),
    ("y6d3jnomfzqvtdla8bjbqv7r", "rfc-023-non-circular-mount-hole"),
    ("kkyfcl5asidwwkhajemg5ut9", "rfc-024-instance-arrays"),
    ("p4xsc5dfk1sr1gwno4p24mzh", "rfc-025-rotated-pad-placements"),
    ("ayndt05z51domjge37i5efxd", "rfc-026-back-side-placement"),
    ("w67wqi1pciql0arfd4frmcop", "rfc-027-quilter-physics-constraints"),
    ("zy4qcr7km8llb2eqtgrerlio", "rfc-028-phys-attrs-on-fn-pin-params"),
    ("amxp6mluwr78owoi2s8mldyf", "rfc-029-package-dependency-versioning"),
    ("dgwea9n35gjgijtpuo6o7b0l", "rfc-030-registry-service"),
    ("odwihsmzb8bn4yohjjoflq82", "rfc-031-silkscreen-graphics"),
    ("e2l63rezqlv4293ogmoeqvww", "rfc-032-subdesign"),
    ("qsgtxir405qq6g41cpewm8ox", "gc-002-amended-layout-door"),
    ("z88w8x6zxiorid9r9lktcroz", "07-decision-records"),
    ("lw67cajqh8ym8fonk8j9kljg", "08-evolution-governance"),
    ("z6zk0qws34whx1tpnvbopm06", "09-mvp-definition"),
    ("lkhejpgivnaw42iq2kwh16it", "10-language-specification"),
]


# Reject a share landing page or the wrong note even if it has a valid heading.
EXPECTED_TITLES = {
    "00-root": "# cohdl.dev — Coherent Design Repository",
    "01-product-constitution": "# 1. Product Constitution",
    "02-conceptual-model": "# 2. Conceptual Model",
    "03-capability-architecture": "# 3. Capability Architecture",
    "04-principle-constraint-mapping": "# 4. Principle → Constraint Mapping",
    "05-coherence-matrix": "# 5. Coherence Matrix",
    "06-rfc-process": "# 6. Feature Proposal Process (RFC)",
    "rfc-001-units-as-types": "# RFC-001: Units-as-types",
    "rfc-002-pin-connection-obligation": "# RFC-002: Pin connection-obligation typing",
    "rfc-003-trait-satisfaction": "# RFC-003: Trait-satisfaction-at-impl-time checking",
    "rfc-004-drc-reclassification": "# RFC-004: DRC/type-system reclassification pass",
    "rfc-005-designator-allocation": "# RFC-005: Collision-free designator allocation",
    "rfc-006-nested-fn-calls": "# RFC-006: Nested fn call semantics",
    "rfc-007-generics-over-specs": "# RFC-007: Generics-over-specs and generic trait bounds",
    "rfc-008-pattern-matching": "# RFC-008: Exhaustive pattern-matching over structural variants",
    "rfc-009-fmt": "# RFC-009: cohdl fmt canonical form",
    "rfc-010-check-json": "# RFC-010: cohdl check --json schema",
    "rfc-011-error-registry": "# RFC-011: Error-code registry (formal v2 baseline)",
    "rfc-012-intent-annotations": "# RFC-012: #[intent(...)] annotations (pure metadata)",
    "rfc-013-layout-constraint": "# RFC-013: Layout-constraint concept (the door)",
    "rfc-014-lsp": "# RFC-014: Language Server Protocol support",
    "rfc-015-ipc2581": "# RFC-015: IPC-2581 codegen backend (Quilter handoff)",
    "rfc-016-modules": "# RFC-016: Module system (package::module::submodule::name)",
    "rfc-017-library-registry": "# RFC-017: Library registry (cohdl source + docs + footprint symbols)",
    "rfc-018-footprint-format": "# RFC-018: Footprint format — pad/footprint, Cadence-style pad/footprint split",
    "rfc-019-vscode-extension": "# RFC-019: VS Code extension for CoHDL",
    "rfc-020-board-outline-dxf": "# RFC-020: Board outline (scoped DXF profile extraction) + oriented placement",
    "rfc-021-ipc7351-footprint-naming": "# RFC-021: IPC-7351 as the canonical footprint naming practice",
    "rfc-022-mount-hole": "# RFC-022: Mechanical locating holes in footprints (mount_hole)",
    "rfc-023-non-circular-mount-hole": "# RFC-023: Non-circular locating holes (mount_hole shape/size)",
    "rfc-024-instance-arrays": "# RFC-024: Array-typed instances and indexed references",
    "rfc-025-rotated-pad-placements": "# RFC-025: Rotated pad placements in footprints",
    "rfc-026-back-side-placement": "# RFC-026: Component placement on the board's back side",
    "rfc-027-quilter-physics-constraints": "# RFC-027: Quilter physics-constraint hints and CSV export",
    "rfc-028-phys-attrs-on-fn-pin-params": "# RFC-028: Physics-constraint attributes on fn Pin parameters",
    "rfc-029-package-dependency-versioning": "# RFC-029: Package dependency versioning — std becomes a versioned registry library",
    "rfc-030-registry-service": "# RFC-030: registry.cohdl.org — the package registry service",
    "rfc-031-silkscreen-graphics": "# RFC-031: Silkscreen graphics for footprints (pin-1 markers, polarity/direction indicators)",
    "rfc-032-subdesign": "# RFC-032: Typed logical composition (subdesign)",
    "gc-002-amended-layout-door": "# GC-002 (amended): Admit layout constraints into the conceptual model",
    "07-decision-records": "# 7. Decision Records",
    "08-evolution-governance": "# 8. Evolution Governance & Design Regression",
    "09-mvp-definition": "# 9. MVP Definition",
    "10-language-specification": "# 10. Language Specification",
}


def get_payload(html):
    chunks = re.findall(r'self\.__next_f\.push\(\[1,"((?:[^"\\]|\\.)*)"\]\)', html)
    return "".join(json.loads('"%s"' % c) for c in chunks)


def parse_flight(payload):
    """Parse RSC flight payload into {id: value} records."""
    records = {}
    dec = json.JSONDecoder()
    i = 0
    n = len(payload)
    while i < n:
        m = re.compile(r'([0-9a-fA-F]+):').match(payload, i)
        if not m:
            # skip to next newline and retry
            nl = payload.find('\n', i)
            if nl == -1:
                break
            i = nl + 1
            continue
        rid = m.group(1)
        j = m.end()
        c = payload[j] if j < n else ''
        if c == 'T':
            # text blob: T<hexlen>,<raw>
            comma = payload.index(',', j)
            length = int(payload[j + 1:comma], 16)
            # length is in bytes (utf-8)
            raw_bytes = payload[comma + 1:].encode('utf-8')[:length]
            text = raw_bytes.decode('utf-8', errors='replace')
            records[rid] = text
            i = comma + 1 + len(text)
        elif c == 'I':
            try:
                val, end = dec.raw_decode(payload, j + 1)
            except json.JSONDecodeError:
                nl = payload.find('\n', j)
                i = (nl + 1) if nl != -1 else n
                continue
            records[rid] = ('$MODULE', val)
            i = end
        else:
            try:
                val, end = dec.raw_decode(payload, j)
            except json.JSONDecodeError:
                nl = payload.find('\n', j)
                i = (nl + 1) if nl != -1 else n
                continue
            records[rid] = val
            i = end
        # skip trailing newline
        while i < n and payload[i] == '\n':
            i += 1
    return records


def resolve(node, records, depth=0):
    if depth > 200:
        return node
    if isinstance(node, str):
        if node.startswith('$L') or (node.startswith('$') and not node.startswith('$$')):
            key = node[2:] if node.startswith('$L') else node[1:]
            if key in records:
                return resolve(records[key], records, depth + 1)
            return None  # unresolvable ref like $undefined, $1
        return node
    return node


def find_viewer(node, records, seen=None):
    """DFS for element whose props.className contains conol-markdown-viewer."""
    if seen is None:
        seen = set()
    if isinstance(node, str):
        if node.startswith('$'):
            r = resolve(node, records)
            if r is not None and not isinstance(r, str):
                return find_viewer(r, records, seen)
        return None
    if isinstance(node, list):
        if len(node) == 4 and node[0] == '$':
            props = node[3]
            if isinstance(props, dict):
                cn = props.get('className', '')
                if isinstance(cn, str) and 'conol-markdown-viewer' in cn:
                    return node
                res = find_viewer(props.get('children'), records, seen)
                if res is not None:
                    return res
            return None
        for child in node:
            res = find_viewer(child, records, seen)
            if res is not None:
                return res
    return None


def text_of(node, records):
    """Inline text extraction."""
    if node is None:
        return ''
    if isinstance(node, bool):
        return ''
    if isinstance(node, str):
        if node.startswith('$'):
            r = resolve(node, records)
            return '' if r is None or isinstance(r, tuple) else (text_of(r, records) if not isinstance(r, str) else r)
        return node
    if isinstance(node, (int, float)):
        return str(node)
    if isinstance(node, list):
        if len(node) == 4 and node[0] == '$':
            tag = node[1]
            props = node[3] if isinstance(node[3], dict) else {}
            inner = text_of(props.get('children'), records)
            if tag == 'strong':
                return f'**{inner}**'
            if tag == 'em':
                return f'*{inner}*'
            if tag == 'del':
                return f'~~{inner}~~'
            if tag == 'code':
                return f'`{inner}`'
            if tag == 'a':
                href = props.get('href', '')
                return f'[{inner}]({href})' if href else inner
            if tag == 'br':
                return '\n'
            if tag == 'input':
                return '[x]' if props.get('checked') else '[ ]'
            return inner
        return ''.join(text_of(c, records) for c in node)
    return ''


def block_md(node, records, indent=0, ordered=False, out=None):
    """Convert block-level elements to markdown lines."""
    if out is None:
        out = []
    if node is None:
        return out
    if isinstance(node, str):
        if node.startswith('$'):
            r = resolve(node, records)
            if r is not None and not isinstance(r, (str, tuple)):
                return block_md(r, records, indent, ordered, out)
            if isinstance(r, str):
                out.append(r)
            return out
        out.append(node)
        return out
    if isinstance(node, list):
        if len(node) == 4 and node[0] == '$':
            tag, props = node[1], (node[3] if isinstance(node[3], dict) else {})
            children = props.get('children')
            if isinstance(props.get('code'), str):
                # lazy code-block component: {code, lang}; long code is a $ref to a T-blob
                code = props['code']
                if code.startswith('$'):
                    r = resolve(code, records)
                    code = r if isinstance(r, str) else ''
                lang = props.get('lang') or ''
                if isinstance(lang, str) and lang.startswith('$'):
                    r = resolve(lang, records)
                    lang = r if isinstance(r, str) else ''
                out.append('```' + (lang if isinstance(lang, str) else ''))
                out.extend(code.rstrip('\n').split('\n'))
                out.append('```')
                out.append('')
                return out
            if tag in ('h1', 'h2', 'h3', 'h4', 'h5', 'h6'):
                lvl = int(tag[1])
                out.append('#' * lvl + ' ' + text_of(children, records))
                out.append('')
            elif tag == 'p':
                t = text_of(children, records)
                if t.strip():
                    out.append('  ' * indent + t if indent else t)
                    out.append('')
            elif tag == 'blockquote':
                sub = block_md(children, records, 0, False, [])
                for line in sub:
                    out.append('> ' + line if line else '>')
                out.append('')
            elif tag in ('ul', 'ol'):
                items = children if isinstance(children, list) else [children]
                # find li children (may be nested one level in arrays)
                def walk_lis(n):
                    if isinstance(n, list):
                        if len(n) == 4 and n[0] == '$':
                            if n[1] == 'li':
                                yield n
                            return
                        for c in n:
                            yield from walk_lis(c)
                    elif isinstance(n, str) and n.startswith('$'):
                        r = resolve(n, records)
                        if r is not None and not isinstance(r, (str, tuple)):
                            yield from walk_lis(r)
                idx = 1
                for li in walk_lis(items):
                    lprops = li[3] if isinstance(li[3], dict) else {}
                    lchild = lprops.get('children')
                    # split inline text vs nested lists
                    inline_parts, nested = [], []
                    def split(n):
                        if isinstance(n, list):
                            if len(n) == 4 and n[0] == '$':
                                if n[1] in ('ul', 'ol', 'pre', 'table', 'blockquote'):
                                    nested.append(n)
                                elif n[1] == 'p':
                                    inline_parts.append(text_of(n[3].get('children') if isinstance(n[3], dict) else None, records))
                                else:
                                    inline_parts.append(text_of(n, records))
                            else:
                                for c in n:
                                    split(c)
                        elif n is not None:
                            inline_parts.append(text_of(n, records))
                    split(lchild)
                    marker = f'{idx}. ' if tag == 'ol' else '- '
                    out.append('  ' * indent + marker + ' '.join(p for p in inline_parts if p.strip()))
                    for nst in nested:
                        block_md(nst, records, indent + 1, tag == 'ol', out)
                    idx += 1
                if indent == 0:
                    out.append('')
            elif tag == 'pre':
                code = text_of(children, records)
                # detect language from inner code className
                lang = ''
                def find_code(n):
                    nonlocal lang
                    if isinstance(n, list):
                        if len(n) == 4 and n[0] == '$' and n[1] == 'code':
                            cn = (n[3] or {}).get('className', '') if isinstance(n[3], dict) else ''
                            m = re.search(r'language-([\w-]+)', cn or '')
                            if m:
                                lang = m.group(1)
                        else:
                            for c in n:
                                if isinstance(c, list):
                                    find_code(c)
                find_code(children)
                code = code.strip('`')
                out.append('```' + lang)
                out.extend(code.rstrip('\n').split('\n'))
                out.append('```')
                out.append('')
            elif tag == 'table':
                rows = []
                def walk_rows(n):
                    if isinstance(n, list):
                        if len(n) == 4 and n[0] == '$':
                            if n[1] == 'tr':
                                rows.append(n)
                                return
                            walk_rows(n[3].get('children') if isinstance(n[3], dict) else None)
                        else:
                            for c in n:
                                walk_rows(c)
                    elif isinstance(n, str) and n.startswith('$'):
                        r = resolve(n, records)
                        if r is not None and not isinstance(r, (str, tuple)):
                            walk_rows(r)
                walk_rows(children)
                first = True
                for tr in rows:
                    cells = []
                    def walk_cells(n):
                        if isinstance(n, list):
                            if len(n) == 4 and n[0] == '$':
                                if n[1] in ('td', 'th'):
                                    cells.append(text_of(n[3].get('children') if isinstance(n[3], dict) else None, records).replace('\n', ' '))
                                    return
                                walk_cells(n[3].get('children') if isinstance(n[3], dict) else None)
                            else:
                                for c in n:
                                    walk_cells(c)
                        elif isinstance(n, str) and n.startswith('$'):
                            r = resolve(n, records)
                            if r is not None and not isinstance(r, (str, tuple)):
                                walk_cells(r)
                    walk_cells(tr[3].get('children') if isinstance(tr[3], dict) else None)
                    out.append('| ' + ' | '.join(cells) + ' |')
                    if first:
                        out.append('|' + '---|' * len(cells))
                        first = False
                out.append('')
            elif tag == 'hr':
                out.append('---')
                out.append('')
            elif tag in ('div', 'section', 'article', 'span', 'main'):
                block_md(children, records, indent, ordered, out)
            elif tag == 'input':
                pass  # checkbox
            else:
                t = text_of(node, records)
                if t.strip():
                    out.append(t)
                    out.append('')
            return out
        for c in node:
            block_md(c, records, indent, ordered, out)
        return out
    return out


def extract_page(url):
    html = subprocess.run(['curl', '-fsSL', url], capture_output=True, text=True, check=True).stdout
    payload = get_payload(html)
    records = parse_flight(payload)
    # collect ALL viewer elements + the page h1 title
    title = ''
    for rid, val in records.items():
        v = val
        # find h1 title
        def find_h1(n, depth=0):
            nonlocal title
            if depth > 100 or title:
                return
            if isinstance(n, list):
                if len(n) == 4 and n[0] == '$' and n[1] == 'h1' and isinstance(n[3], dict):
                    cn = n[3].get('className', '') or ''
                    if 'font-bold' in cn and ('text-3xl' in cn or 'text-5xl' in cn):
                        title = text_of(n[3].get('children'), records)
                        return
                for c in (n if not (len(n) == 4 and n[0] == '$') else [n[3].get('children')] if isinstance(n[3], dict) else []):
                    find_h1(c, depth + 1)
        if not isinstance(v, tuple):
            find_h1(v)
    viewer = None
    for rid, val in records.items():
        if isinstance(val, tuple):
            continue
        viewer = find_viewer(val, records)
        if viewer is not None:
            break
    lines = [f'# {title}', ''] if title else []
    if viewer is not None:
        props = viewer[3]
        body = block_md(props.get('children'), records, 0, False, [])
        lines.extend(body)
    return '\n'.join(lines)


def main():
    outdir = sys.argv[1] if len(sys.argv) > 1 else 'design-repo'
    with tempfile.TemporaryDirectory(prefix='cohdl-design-') as staging:
        extracted = []
        failures = 0
        for slug, name in PAGES:
            url = BASE + ('/' + slug if slug else '')
            try:
                md = extract_page(url)
                title, _, body = md.partition('\n')
                if title != EXPECTED_TITLES[name]:
                    raise ValueError(f'expected {EXPECTED_TITLES[name]!r}, got {title!r}')
                if not body.strip():
                    raise ValueError('empty document body')
                with open(os.path.join(staging, name + '.md'), 'w', encoding='utf-8') as f:
                    f.write(md)
                extracted.append((name, len(md)))
            except Exception as e:
                print(f'FAIL {name} (page {slug or "<root>"}): {e}', file=sys.stderr)
                failures += 1
        if failures:
            print(f'Snapshot unchanged: {failures} page(s) failed.', file=sys.stderr)
            return 1

        os.makedirs(outdir, exist_ok=True)
        for name, _ in extracted:
            shutil.move(os.path.join(staging, name + '.md'), os.path.join(outdir, name + '.md'))
        for name, count in extracted:
            print(f'OK   {name}: {count} chars')
    return 0


if __name__ == '__main__':
    sys.exit(main())
