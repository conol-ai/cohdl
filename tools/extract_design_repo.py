#!/usr/bin/env python3
"""Extract markdown content from conol.ai share pages (Next.js RSC payload)."""
import re, json, sys, subprocess, os

BASE = "https://conol.ai/share/note/od34sne4sa5ujuohyhldr21r"

PAGES = [
    ("", "00-root"),
    ("ql0w2zjodetyo7ikpu3azf3p", "01-product-constitution"),
    ("ejj7ue2zfoxizrdrs8u60v2b", "02-conceptual-model"),
    ("j9jy264k9rl2vd09gyqyp7pi", "03-capability-architecture"),
    ("mao945pcievy1753aphfbuoo", "04-principle-constraint-mapping"),
    ("u0g46pbhldab2v9rfx0bosrk", "05-coherence-matrix"),
    ("rpwb5os1m5hykf6j110qo408", "06-rfc-process"),
    ("falh4cffau5igrvl3vbs0wag", "rfc-001-units-as-types"),
    ("fnj4xqj77qxvoz54k4amox0j", "rfc-002-pin-connection-obligation"),
    ("dlt28nf62009b3rcu9atytef", "rfc-003-trait-satisfaction"),
    ("m8uh7d5o5kq1lrw6y488osi1", "rfc-004-drc-reclassification"),
    ("b1jxsdyp7yoix4dx0e4xwm1s", "rfc-005-designator-allocation"),
    ("tgl56zp1fp4dcekn39592yj3", "rfc-006-nested-fn-calls"),
    ("sdvm1pmfs8s2y3ovje4e9td2", "rfc-007-generics-over-specs"),
    ("z3vpfowxnku0w0phczzb977q", "rfc-008-pattern-matching"),
    ("ke8hndl2xb8f5u11shrvwxib", "rfc-009-fmt"),
    ("gh5vfukafa549owcr8uz15ra", "rfc-010-check-json"),
    ("r8oj9xfx4g7657dptne3p2uz", "rfc-011-error-registry"),
    ("xd3mrnbompcglnwl1ygn34cu", "rfc-012-intent-annotations"),
    ("mss6eauuemfswvbjflarwrvp", "rfc-013-layout-constraint"),
    ("f0prtp9282z9jtvcfmj3d77k", "rfc-014-lsp"),
    ("me94vtlxl1thydpcfiexokbw", "rfc-015-ipc2581"),
    ("dodhepqawslco0d6vcdo2is3", "rfc-016-modules"),
    ("m3lwb8usadcz42qlew56j7ma", "rfc-017-library-registry"),
    ("ddmxe9f1y4wbsdko8rqmh748", "rfc-018-footprint-format"),
    ("eaalgfqdqqp9wb9xtu80u11p", "07-decision-records"),
    ("g4mua3obv1qd14po8pgzjrfn", "08-evolution-governance"),
    ("rca8yxw24xn3o25c2rh7ovt6", "09-mvp-definition"),
    ("j49jyieotm07l3svn79se08y", "10-language-specification"),
]


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
    html = subprocess.run(['curl', '-sL', url], capture_output=True, text=True, check=True).stdout
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
    os.makedirs(outdir, exist_ok=True)
    for slug, name in PAGES:
        url = BASE + ('/' + slug if slug else '')
        try:
            md = extract_page(url)
        except Exception as e:
            print(f'FAIL {name}: {e}')
            continue
        path = os.path.join(outdir, name + '.md')
        with open(path, 'w', encoding='utf-8') as f:
            f.write(md)
        print(f'OK   {name}: {len(md)} chars')


if __name__ == '__main__':
    main()
