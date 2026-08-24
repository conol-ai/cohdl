#!/usr/bin/env python3
"""Generate the cohdl.org design-record pages from docs/design/.

Converts the language specification and every RFC (plus the GC-002 amendment)
from their Markdown sources into site chrome, VERBATIM — no paraphrase, no
build step at deploy time. Output is checked in; re-run after a design-repo
re-extract:

    python3 site/tools/gen_design_docs.py

Emits:
    site/public/docs/spec/index.html
    site/public/docs/rfcs/index.html
    site/public/docs/rfcs/<doc-stem>/index.html   (one per RFC + gc-002)

and prints the sitemap <url> entries for the generated pages.

The converter handles exactly the Markdown subset the design documents use —
headings, paragraphs, bullet/ordered lists, tables, fenced code, inline
code/bold/italic — and fails loudly on anything else, so a re-extract that
introduces a new construct cannot silently mis-render. Mentions of RFC-NNN /
GC-002 in prose (not code) are auto-linked to their pages; that is the one
deliberate addition to the verbatim text.
"""

from __future__ import annotations

import html
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
DESIGN = REPO / "docs" / "design"
PUB = REPO / "site" / "public"

SPEC_FILE = "10-language-specification.md"

# Order defines the index and the prev/next pager chain.
RFC_FILES = [
    "rfc-001-units-as-types.md",
    "rfc-002-pin-connection-obligation.md",
    "rfc-003-trait-satisfaction.md",
    "rfc-004-drc-reclassification.md",
    "rfc-005-designator-allocation.md",
    "rfc-006-nested-fn-calls.md",
    "rfc-007-generics-over-specs.md",
    "rfc-008-pattern-matching.md",
    "rfc-009-fmt.md",
    "rfc-010-check-json.md",
    "rfc-011-error-registry.md",
    "rfc-012-intent-annotations.md",
    "rfc-013-layout-constraint.md",
    "rfc-014-lsp.md",
    "rfc-015-ipc2581.md",
    "rfc-016-modules.md",
    "rfc-017-library-registry.md",
    "rfc-018-footprint-format.md",
    "rfc-019-vscode-extension.md",
    "rfc-020-board-outline-dxf.md",
    "rfc-021-ipc7351-footprint-naming.md",
    "rfc-022-mount-hole.md",
    "rfc-023-non-circular-mount-hole.md",
    "rfc-024-instance-arrays.md",
    "rfc-025-rotated-pad-placements.md",
    "rfc-026-back-side-placement.md",
    "rfc-027-quilter-physics-constraints.md",
    "rfc-028-phys-attrs-on-fn-pin-params.md",
    "rfc-029-package-dependency-versioning.md",
    "rfc-030-registry-service.md",
    "rfc-031-silkscreen-graphics.md",
    "gc-002-amended-layout-door.md",
]

# One-line summaries: the index blurb and each page's meta description.
SUMMARIES = {
    SPEC_FILE: "The compiled statement of what the language is — every Accepted RFC folded into one document.",
    "rfc-001-units-as-types.md": "Engineering values are typed by unit: a closed set of primitive unit types, zero coercion, bare numbers rejected.",
    "rfc-002-pin-connection-obligation.md": "Every pin carries a connection obligation; required pins must be wired or explicitly declared nc by final assembly.",
    "rfc-003-trait-satisfaction.md": "Devices satisfy traits through explicit impl blocks with pin and spec mapping, checked at impl time — never structurally.",
    "rfc-004-drc-reclassification.md": "Structural mistakes move into the type system; residual DRC shrinks to four genuinely emergent whole-graph rules.",
    "rfc-005-designator-allocation.md": "Collision-free designator allocation as a pure function with checked injectivity, held stable across edits by design.lock.",
    "rfc-006-nested-fn-calls.md": "Sub-circuit fns compose by nesting; expansion is deterministic, with cycle detection.",
    "rfc-007-generics-over-specs.md": "Generic devices and fns parameterized over spec values, bounded by traits.",
    "rfc-008-pattern-matching.md": "Exhaustive pattern matching over a device family's structural variants.",
    "rfc-009-fmt.md": "One canonical rendering of every source file; cohdl fmt rewrites, and --check gates.",
    "rfc-010-check-json.md": "Structured, machine-readable diagnostics: one JSON document per check.",
    "rfc-011-error-registry.md": "A formal registry of stable diagnostic codes; a code is issued once and never reused.",
    "rfc-012-intent-annotations.md": "#[intent(\"...\")] attaches one opaque rationale string per declaration, with guaranteed zero effect on compilation.",
    "rfc-013-layout-constraint.md": "The door to physical design: layout {} constraints and #[placement_hint], emitted as layout.json.",
    "rfc-014-lsp.md": "cohdl lsp — the language server: live diagnostics, hover, definitions, references over stdio.",
    "rfc-015-ipc2581.md": "The IPC-2581 handoff document emitted by build --emit ipc2581 for layout partners.",
    "rfc-016-modules.md": "File-tree module paths, use imports, and cross-package pub enforcement.",
    "rfc-017-library-registry.md": "#[doc] reference documents, and footprint as a resolvable declaration kind rather than a string.",
    "rfc-018-footprint-format.md": "The pad/footprint split: a closed pad vocabulary, footprint bodies, and geometry projected into .kicad_mod and IPC-2581.",
    "rfc-019-vscode-extension.md": "A real installable VS Code extension: TextMate grammar plus a language client over cohdl lsp.",
    "rfc-020-board-outline-dxf.md": "Board outlines extracted from a scoped DXF profile, and place gains orientation.",
    "rfc-021-ipc7351-footprint-naming.md": "Footprints in a closed six-family set carry IPC-7351-derived names, cross-checked against their own pad geometry.",
    "rfc-022-mount-hole.md": "Mechanical locating holes in footprints, numbered disjointly from pads.",
    "rfc-023-non-circular-mount-hole.md": "mount_hole grows shape and size — rectangular and oval holes reusing the pad shape vocabulary.",
    "rfc-024-instance-arrays.md": "One array-typed instance declares N fully real elements, addressed NAME[i] everywhere a reference is valid.",
    "rfc-025-rotated-pad-placements.md": "Pad placements gain rotate ANGLE; KiCad's three-argument (at x y angle), size never swapped.",
    "rfc-026-back-side-placement.md": "place gains side top|bottom, with mirror semantics matching pcbnew's flip-then-orient exactly.",
    "rfc-027-quilter-physics-constraints.md": "Seven structured physics attributes plus diff_pair brackets, exported as the Quilter CSV set.",
    "rfc-028-phys-attrs-on-fn-pin-params.md": "Physics attributes on fn Pin and Instance parameters, resolved per call site.",
    "rfc-029-package-dependency-versioning.md": "Exact-version [dependencies], a sha256 content-hash lock file — and std becomes an ordinary package.",
    "rfc-030-registry-service.md": "The external contract of registry.cohdl.org: trust-tiered packages, server-authoritative hashes, and stable package-and-part search.",
    "rfc-031-silkscreen-graphics.md": "Silkscreen primitives plus semantic pin-1 and polarity markers that expand to checked geometry.",
    "gc-002-amended-layout-door.md": "The governance amendment that admitted layout constraints into the conceptual model.",
}

STATUS = {f: "Accepted · implemented in the compiler" for f in RFC_FILES}
STATUS["gc-002-amended-layout-door.md"] = "Governance change · amended"

# Curated per-page notes rendered as an extra callout after the banner.
ROTATE_DEVIATION = (
    "Live deviation: the shipped compiler accepts <code>rotate</code> at any "
    "whole degree in 0–359 — on <code>place</code> and on <code>pad</code> alike — "
    "superseding the closed {0, 90, 180, 270} set this text states, pending a "
    'follow-up RFC. See <a href="/docs/layout/">Layout &amp; fabrication</a>.'
)

NOTES = {
    "rfc-020-board-outline-dxf.md": ROTATE_DEVIATION,
    "rfc-025-rotated-pad-placements.md": ROTATE_DEVIATION,
    "rfc-026-back-side-placement.md": ROTATE_DEVIATION,
    "10-language-specification.md": ROTATE_DEVIATION,
}

RFC_BANNER = (
    "This is the accepted text of the proposal, mirrored verbatim from the CoHDL "
    "design repository — a historical, normative record. Later RFCs can amend "
    "earlier ones; the compiled current statement of the language is the "
    '<a href="/docs/spec/">language specification</a>. Deliberate implementation '
    "deviations are recorded in "
    '<a href="https://github.com/conol-ai/cohdl/blob/main/docs/compliance-report.md">'
    "the compliance ledger</a>."
)

SPEC_BANNER = (
    "Mirrored verbatim from the CoHDL design repository: the compiled statement "
    "of what the language <em>is</em> — Accepted RFCs only. The proposals behind "
    'it, with their rationale and rejected alternatives, are in the <a href="/docs/rfcs/">'
    "RFC index</a>."
)

GENERATED_MARK = "<!-- GENERATED by site/tools/gen_design_docs.py — do not hand-edit; edit docs/design/ and re-run. -->"


# --------------------------------------------------------------- chrome

HEAD = """<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <meta name="theme-color" content="#070b11" />
    <title>@@TITLE@@</title>
    <meta name="description" content="@@DESCRIPTION@@" />
    <link rel="canonical" href="@@CANONICAL@@" />

    <meta property="og:type" content="article" />
    <meta property="og:site_name" content="CoHDL" />
    <meta property="og:title" content="@@OGTITLE@@" />
    <meta property="og:description" content="@@DESCRIPTION@@" />
    <meta property="og:url" content="@@CANONICAL@@" />
    <meta name="twitter:card" content="summary" />

    <link rel="icon" href="/favicon.svg" type="image/svg+xml" />
    <link rel="stylesheet" href="/css/style.css" />
    <link rel="stylesheet" href="/css/prose.css" />

    <!-- Google tag (gtag.js) — bootstrap lives in /js/analytics.js so the CSP
         needs no inline-script allowance. -->
    <script async src="https://www.googletagmanager.com/gtag/js?id=G-R73M37GXP7"></script>
    <script src="/js/analytics.js"></script>
  </head>
  <body>
    <div class="shell">
      <header class="masthead">
        <a class="brand" href="/">
          <svg
            class="brand-mark"
            width="34"
            height="34"
            viewBox="0 0 40 40"
            fill="none"
            aria-hidden="true"
          >
            <rect x="7.5" y="7.5" width="25" height="25" rx="7" />
            <path
              d="M14 16.5h4.5l3-4.5M14 23.5h4.5l3 4.5M21.5 12v16M21.5 16.5H27M21.5 23.5H27"
            />
            <circle cx="14" cy="16.5" r="1.3" />
            <circle cx="14" cy="23.5" r="1.3" />
            <circle cx="27" cy="16.5" r="1.3" />
            <circle cx="27" cy="23.5" r="1.3" />
          </svg>
          <span class="brand-word">CoHDL</span>
        </a>
        <nav class="site-nav" aria-label="Site">
          <a class="site-nav-link" href="/docs/" aria-current="page">Docs</a>
          <a class="site-nav-link" href="/blog/">Blog</a>
          <a class="site-nav-link" href="/use-cases/">Use cases</a>
          <a class="site-nav-link" href="https://registry.cohdl.org">Registry</a>
        </nav>
      </header>

      <main>
"""

FOOT = """      </main>

      <footer class="footer">
        <p class="footer-line">
          <span class="footer-brand">CoHDL</span>
          <span class="footer-sep" aria-hidden="true">·</span>
          <a href="/docs/">Docs</a>
          <span class="footer-sep" aria-hidden="true">·</span>
          <a href="/blog/">Blog</a>
          <span class="footer-sep" aria-hidden="true">·</span>
          <a href="/use-cases/">Use cases</a>
          <span class="footer-sep" aria-hidden="true">·</span>
          <a href="https://registry.cohdl.org">Package registry</a>
          <span class="footer-sep" aria-hidden="true">·</span>
          <a
            href="https://discord.gg/x7DXPvK66"
            target="_blank"
            rel="noopener noreferrer"
            aria-label="Join on Discord (opens in a new tab)"
          >Join on Discord <span aria-hidden="true">↗</span></a>
        </p>
        <p class="footer-fine">A project by Conol.</p>
      </footer>
    </div>
  </body>
</html>
"""


# ----------------------------------------------------- markdown subset

SLUG_BY_REF: dict[str, str] = {}
for f in RFC_FILES:
    stem = f[:-3]
    if stem.startswith("rfc-"):
        SLUG_BY_REF[f"RFC-{stem[4:7]}"] = stem
    elif stem.startswith("gc-"):
        SLUG_BY_REF[f"GC-{stem[3:6]}"] = stem

REF_RE = re.compile(r"\b(RFC-\d{3}|GC-\d{3})\b")


def autolink(html_text: str, self_stem: str) -> str:
    """Link RFC-NNN / GC-NNN mentions in already-escaped prose HTML."""

    def repl(m: re.Match[str]) -> str:
        ref = m.group(1)
        stem = SLUG_BY_REF.get(ref)
        if stem is None or stem == self_stem:
            return ref
        return f'<a href="/docs/rfcs/{stem}/">{ref}</a>'

    return REF_RE.sub(repl, html_text)


# Italic must respect flanking, or literal asterisks pair up: the corpus
# contains `/* ... */` and footnote markers like `E004*` in prose, which
# CommonMark renders literally. An opener may not follow a word char or `*`
# and may not precede whitespace; a closer may not follow whitespace.
ITALIC_RE = re.compile(r"(?<![\w*])\*(?!\s)([^*\n]+?)(?<!\s)\*")


def stash_code(text: str, codes: list[str]) -> str:
    """Replace `code spans` with placeholders so later passes never touch them."""

    def stash(m: re.Match[str]) -> str:
        codes.append(f"<code>{html.escape(m.group(1))}</code>")
        return f"\x00{len(codes) - 1}\x00"

    return re.sub(r"`([^`]+)`", stash, text)


def finish_inline(text: str, codes: list[str], self_stem: str, link_refs: bool) -> str:
    """Render bold/italic/ref links on code-stashed text, then restore spans."""
    text = html.escape(text, quote=False)
    text = re.sub(r"\*\*([^*]+)\*\*", r"<strong>\1</strong>", text)
    text = ITALIC_RE.sub(r"<em>\1</em>", text)
    if link_refs:
        text = autolink(text, self_stem)
    return re.sub(r"\x00(\d+)\x00", lambda m: codes[int(m.group(1))], text)


def inline(text: str, self_stem: str, link_refs: bool = True) -> str:
    """Escape and render inline markup: code spans, bold, italic, ref links."""
    codes: list[str] = []
    return finish_inline(stash_code(text, codes), codes, self_stem, link_refs)


class Doc:
    def __init__(self, filename: str):
        self.filename = filename
        self.stem = filename[:-3]
        self.lines = (DESIGN / filename).read_text().splitlines()
        first = self.lines[0]
        if not first.startswith("# "):
            sys.exit(f"{filename}: first line is not an h1 title")
        self.title = first[2:].strip()

    def body_html(self) -> str:
        lines = self.lines[1:]
        # Normalize heading depth: shallowest remaining level renders as h2.
        # Scan fence-aware — a `# comment` inside a code block is not a heading.
        seen_levels: set[int] = set()
        in_fence = False
        for ln in lines:
            if ln.startswith("```"):
                in_fence = not in_fence
            elif not in_fence and (m := re.match(r"^(#{1,4}) ", ln)):
                seen_levels.add(len(m.group(1)))
        level_map = {lv: min(i + 2, 4) for i, lv in enumerate(sorted(seen_levels))}

        out: list[str] = []
        i = 0
        n = len(lines)
        while i < n:
            line = lines[i]

            if not line.strip():
                i += 1
                continue

            if line.startswith("```"):
                fence: list[str] = []
                i += 1
                while i < n and not lines[i].startswith("```"):
                    fence.append(lines[i])
                    i += 1
                if i >= n:
                    sys.exit(f"{self.filename}: unclosed code fence")
                i += 1  # closing fence
                out.append("<pre><code>" + html.escape("\n".join(fence)) + "</code></pre>")
                continue

            m = re.match(r"^(#{1,4}) (.+)$", line)
            if m:
                h = level_map[len(m.group(1))]
                out.append(f"<h{h}>{inline(m.group(2), self.stem, link_refs=False)}</h{h}>")
                i += 1
                continue

            if line.startswith("|"):
                # Stash code spans BEFORE splitting on `|`, so a cell holding a
                # backtick-wrapped pipe (`top|bottom`) can never mis-split; a
                # GFM-escaped \| survives as a literal pipe in the cell.
                codes: list[str] = []
                rows: list[list[str]] = []
                while i < n and lines[i].startswith("|"):
                    stashed = stash_code(lines[i].strip().strip("|"), codes)
                    stashed = stashed.replace("\\|", "\x01")
                    rows.append([c.strip().replace("\x01", "|") for c in stashed.split("|")])
                    i += 1
                if len(rows) < 2 or not all(re.fullmatch(r":?-+:?", c) for c in rows[1]):
                    sys.exit(f"{self.filename}: malformed table near line {i}")
                width = len(rows[0])
                for row in rows[1:]:
                    if len(row) != width:
                        sys.exit(
                            f"{self.filename}: table row width {len(row)} != header {width} near line {i}"
                        )
                cell = lambda c: finish_inline(c, codes, self.stem, True)
                head_cells = "".join(f"<th>{cell(c)}</th>" for c in rows[0])
                body_rows = "".join(
                    "<tr>" + "".join(f"<td>{cell(c)}</td>" for c in row) + "</tr>"
                    for row in rows[2:]
                )
                out.append(
                    '<div class="table-scroll"><table>'
                    f"<thead><tr>{head_cells}</tr></thead>"
                    f"<tbody>{body_rows}</tbody></table></div>"
                )
                continue

            m = re.match(r"^([-*]|\d+\.) ", line)
            if m:
                ordered = m.group(1) not in ("-", "*")
                items: list[str] = []
                while i < n:
                    im = re.match(r"^([-*]|\d+\.) (.*)$", lines[i])
                    if im and (im.group(1) not in ("-", "*")) == ordered:
                        items.append(im.group(2))
                        i += 1
                        # Indented continuation lines belong to this item.
                        while i < n and re.match(r"^\s+\S", lines[i]) and not re.match(
                            r"^\s+([-*]|\d+\.) ", lines[i]
                        ):
                            items[-1] += " " + lines[i].strip()
                            i += 1
                    elif re.match(r"^\s+([-*]|\d+\.) ", lines[i]):
                        sys.exit(f"{self.filename}: nested list near line {i} — unsupported")
                    else:
                        break
                tag = "ol" if ordered else "ul"
                out.append(
                    f"<{tag}>" + "".join(f"<li>{inline(it, self.stem)}</li>" for it in items) + f"</{tag}>"
                )
                continue

            if line.startswith(("> ", "![", "<")):
                sys.exit(f"{self.filename}: unsupported construct near line {i}: {line[:60]!r}")

            # Paragraph: gather until a blank line or block construct.
            para = [line]
            i += 1
            while i < n and lines[i].strip() and not re.match(
                r"^(#{1,4} |```|\||([-*]|\d+\.) )", lines[i]
            ):
                para.append(lines[i])
                i += 1
            out.append(f"<p>{inline(' '.join(para), self.stem)}</p>")

        return "\n".join(out)


# ------------------------------------------------------------- pages


def page(canonical: str, title: str, og_title: str, description: str, main_html: str) -> str:
    head = (
        HEAD.replace("@@TITLE@@", html.escape(title, quote=False))
        .replace("@@OGTITLE@@", html.escape(og_title))
        .replace("@@DESCRIPTION@@", html.escape(description))
        .replace("@@CANONICAL@@", canonical)
    )
    return head.replace("<!doctype html>", f"<!doctype html>\n{GENERATED_MARK}") + main_html + FOOT


def crumbs(items: list[tuple[str, str | None]]) -> str:
    lis = []
    for label, href in items:
        esc = html.escape(label, quote=False)
        if href:
            lis.append(f'<li><a href="{href}">{esc}</a></li>')
        else:
            lis.append(f'<li aria-current="page">{esc}</li>')
    return '<ol class="crumbs">' + "".join(lis) + "</ol>"


def write(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content)
    print(f"wrote {path.relative_to(REPO)}")


def rfc_display_no(stem: str) -> str:
    return stem.split("-")[0].upper() + "-" + stem.split("-")[1]


def gen_rfc_pages() -> None:
    docs = [Doc(f) for f in RFC_FILES]
    for idx, doc in enumerate(docs):
        prev_doc = docs[idx - 1] if idx > 0 else None
        next_doc = docs[idx + 1] if idx + 1 < len(docs) else None

        pager = ['<nav class="pager" aria-label="More">']
        if prev_doc:
            pager.append(
                f'<a href="/docs/rfcs/{prev_doc.stem}/">← {html.escape(rfc_display_no(prev_doc.stem))}</a>'
            )
        else:
            pager.append('<a href="/docs/rfcs/">← All RFCs</a>')
        if next_doc:
            pager.append(
                f'<a href="/docs/rfcs/{next_doc.stem}/">{html.escape(rfc_display_no(next_doc.stem))} →</a>'
            )
        else:
            pager.append('<a href="/docs/rfcs/">All RFCs →</a>')
        pager.append("</nav>")

        note = NOTES.get(doc.filename)
        note_html = f'\n<div class="callout">{note}</div>' if note else ""

        main = f"""        <div class="page-head">
          {crumbs([("Docs", "/docs/"), ("RFCs", "/docs/rfcs/"), (rfc_display_no(doc.stem), None)])}
          <h1 class="page-title">{html.escape(doc.title, quote=False)}</h1>
          <p class="post-meta">{STATUS[doc.filename]}</p>
        </div>

        <article class="prose">
<div class="callout">{RFC_BANNER}</div>{note_html}
{doc.body_html()}
{"".join(pager)}
        </article>
"""
        write(
            PUB / "docs" / "rfcs" / doc.stem / "index.html",
            page(
                f"https://cohdl.org/docs/rfcs/{doc.stem}/",
                f"{doc.title} — CoHDL docs",
                doc.title,
                SUMMARIES[doc.filename],
                main,
            ),
        )


def gen_spec_page() -> None:
    doc = Doc(SPEC_FILE)
    note = NOTES.get(SPEC_FILE)
    note_html = f'\n<div class="callout">{note}</div>' if note else ""
    main = f"""        <div class="page-head">
          {crumbs([("Docs", "/docs/"), ("Language specification", None)])}
          <h1 class="page-title">The language specification</h1>
          <p class="page-lede">{SUMMARIES[SPEC_FILE]}</p>
        </div>

        <article class="prose">
<div class="callout">{SPEC_BANNER}</div>{note_html}
{doc.body_html()}
<nav class="pager" aria-label="More"><a href="/docs/">← Docs</a><a href="/docs/rfcs/">The RFCs →</a></nav>
        </article>
"""
    write(
        PUB / "docs" / "spec" / "index.html",
        page(
            "https://cohdl.org/docs/spec/",
            "The language specification — CoHDL docs",
            "The CoHDL language specification",
            SUMMARIES[SPEC_FILE],
            main,
        ),
    )


def gen_rfc_index() -> None:
    items = []
    for f in RFC_FILES:
        doc = Doc(f)
        no = rfc_display_no(doc.stem)
        # Strip the leading "RFC-NNN: " / "GC-002 (amended): " from the display title.
        short_title = re.sub(r"^(RFC-\d{3}|GC-\d{3}( \(amended\))?): ", "", doc.title)
        items.append(
            '<li class="rfc-item">'
            f'<span class="rfc-no">{html.escape(no)}</span>'
            f'<h2 class="rfc-item-title"><a href="/docs/rfcs/{doc.stem}/">{html.escape(short_title, quote=False)}</a></h2>'
            f'<p class="rfc-item-summary">{html.escape(SUMMARIES[f], quote=False)}</p>'
            "</li>"
        )
    main = f"""        <div class="page-head">
          {crumbs([("Docs", "/docs/"), ("RFCs", None)])}
          <h1 class="page-title">The RFCs</h1>
          <p class="page-lede">
            Every feature of CoHDL exists because a written proposal was accepted for
            it. These are the accepted texts, verbatim — the problem each one solves,
            the design, the rejected alternatives, and the decision. Together with the
            <a href="/docs/spec/">language specification</a> they are the language's
            complete normative record.
          </p>
        </div>

        <section aria-label="RFC index">
          <ul class="rfc-list">
{chr(10).join(items)}
          </ul>
        </section>
"""
    write(
        PUB / "docs" / "rfcs" / "index.html",
        page(
            "https://cohdl.org/docs/rfcs/",
            "RFCs — CoHDL docs",
            "The CoHDL RFCs",
            "The accepted proposals behind every CoHDL feature, published verbatim: 31 RFCs and the GC-002 governance amendment.",
            main,
        ),
    )


def print_sitemap_entries() -> None:
    urls = ["https://cohdl.org/docs/spec/", "https://cohdl.org/docs/rfcs/"] + [
        f"https://cohdl.org/docs/rfcs/{f[:-3]}/" for f in RFC_FILES
    ]
    print("\nsitemap entries:")
    for u in urls:
        print(f"  <url>\n    <loc>{u}</loc>\n    <changefreq>monthly</changefreq>\n    <priority>0.5</priority>\n  </url>")


if __name__ == "__main__":
    missing = [f for f in [SPEC_FILE, *RFC_FILES] if not (DESIGN / f).exists()]
    if missing:
        sys.exit(f"missing sources: {missing}")
    unknown = set(SUMMARIES) - {SPEC_FILE, *RFC_FILES}
    if unknown:
        sys.exit(f"summaries for unknown files: {unknown}")
    gen_spec_page()
    gen_rfc_pages()
    gen_rfc_index()
    print_sitemap_entries()
