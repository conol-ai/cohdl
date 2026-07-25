// A small, deliberately incomplete Markdown renderer for published
// documents (RFC-017 `#[doc]` content, rendered on package pages).
//
// Published documents are untrusted publisher content, so this renders to
// React elements — never `dangerouslySetInnerHTML`, never a raw-HTML
// passthrough. Anything it does not understand (including inline HTML) shows
// as literal text, which is the safe failure direction. Link targets are
// restricted to http/https/mailto and relative document paths, so no
// `javascript:` URL can reach an anchor.
//
// Supported, because this is what datasheets and READMEs actually use:
// ATX headings, fenced and indented code blocks, unordered/ordered lists,
// blockquotes, horizontal rules, tables (leading-pipe form), paragraphs; and
// inline code, bold, italic, links, and images.

import React from "react";

/// Only schemes that cannot execute script, plus relative paths. `resolve`
/// turns a relative path into a same-version document URL when the caller
/// knows how (images/links inside a package's own docs). Exported for the
/// unit tests: this is the function standing between publisher-authored
/// document text and an anchor's `href`.
export function safeUrl(raw: string, resolve?: (path: string) => string | null): string | null {
  const url = raw.trim();
  if (/^(https?:|mailto:)/i.test(url)) return url;
  if (/^[a-z][a-z0-9+.-]*:/i.test(url)) return null; // any other scheme: refuse
  if (url.startsWith("#") || url.startsWith("//")) return null;
  return resolve ? resolve(url) : null;
}

interface InlineOpts {
  resolve?: (path: string) => string | null;
}

/// Inline spans: `code`, ![image](src), [text](href), **bold**, *italic*.
/// Code wins over everything else inside it (so `**x**` in code stays
/// literal), which is why it is matched in the same pass.
const INLINE =
  /(`[^`]+`)|(!\[([^\]]*)\]\(([^)\s]+)\))|(\[([^\]]+)\]\(([^)\s]+)\))|(\*\*([^*]+)\*\*)|(\*([^*]+)\*)|(_([^_]+)_)/g;

function renderInline(text: string, opts: InlineOpts, keyBase: string): React.ReactNode[] {
  const out: React.ReactNode[] = [];
  let last = 0;
  let k = 0;
  for (const m of text.matchAll(INLINE)) {
    const at = m.index ?? 0;
    if (at > last) out.push(text.slice(last, at));
    const key = `${keyBase}-${k++}`;
    if (m[1]) {
      out.push(<code key={key}>{m[1].slice(1, -1)}</code>);
    } else if (m[2]) {
      const src = safeUrl(m[4], opts.resolve);
      // A refused image src degrades to its alt text — never a broken
      // request to somewhere unexpected.
      out.push(src ? <img key={key} src={src} alt={m[3]} /> : <em key={key}>{m[3] || m[4]}</em>);
    } else if (m[5]) {
      const href = safeUrl(m[7], opts.resolve);
      out.push(
        href ? (
          <a key={key} href={href} rel="noopener noreferrer nofollow" target="_blank">
            {m[6]}
          </a>
        ) : (
          <span key={key}>{m[6]}</span>
        ),
      );
    } else if (m[8]) {
      out.push(<strong key={key}>{m[9]}</strong>);
    } else if (m[10] || m[12]) {
      out.push(<em key={key}>{m[11] ?? m[13]}</em>);
    }
    last = at + m[0].length;
  }
  if (last < text.length) out.push(text.slice(last));
  return out;
}

function tableRow(line: string): string[] {
  const t = line.trim().replace(/^\|/, "").replace(/\|$/, "");
  return t.split("|").map((c) => c.trim());
}

const DIVIDER_ROW = /^\|?[\s:|-]+\|[\s:|-]*$/;

export function Markdown({
  source,
  resolve,
}: {
  source: string;
  resolve?: (path: string) => string | null;
}) {
  const lines = source.replace(/\r\n?/g, "\n").split("\n");
  const blocks: React.ReactNode[] = [];
  const opts: InlineOpts = { resolve };
  let i = 0;
  let key = 0;

  const paragraph: string[] = [];
  const flushParagraph = () => {
    if (paragraph.length === 0) return;
    const text = paragraph.join("\n");
    paragraph.length = 0;
    blocks.push(<p key={`p${key++}`}>{renderInline(text, opts, `p${key}`)}</p>);
  };

  while (i < lines.length) {
    const line = lines[i];
    const trimmed = line.trim();

    if (trimmed === "") {
      flushParagraph();
      i++;
      continue;
    }

    // Fenced code (``` or ~~~): everything to the closing fence is literal.
    const fence = /^(```|~~~)(.*)$/.exec(trimmed);
    if (fence) {
      flushParagraph();
      const marker = fence[1];
      const body: string[] = [];
      i++;
      while (i < lines.length && !lines[i].trim().startsWith(marker)) body.push(lines[i++]);
      i++; // closing fence (or EOF — an unclosed fence still renders)
      blocks.push(
        <pre className="md-code" key={`c${key++}`}>
          <code>{body.join("\n")}</code>
        </pre>,
      );
      continue;
    }

    // Indented code block (4 spaces / a tab), only outside a paragraph.
    if (paragraph.length === 0 && /^(\t| {4})/.test(line)) {
      const body: string[] = [];
      while (i < lines.length && (/^(\t| {4})/.test(lines[i]) || lines[i].trim() === "")) {
        if (lines[i].trim() === "" && !/^(\t| {4})/.test(lines[i + 1] ?? "")) break;
        body.push(lines[i].replace(/^(\t| {4})/, ""));
        i++;
      }
      blocks.push(
        <pre className="md-code" key={`c${key++}`}>
          <code>{body.join("\n")}</code>
        </pre>,
      );
      continue;
    }

    const heading = /^(#{1,6})\s+(.*)$/.exec(trimmed);
    if (heading) {
      flushParagraph();
      const depth = heading[1].length;
      // Headings inside a document start at <h3>: the page's own <h1>/<h2>
      // own the top of the outline.
      const Tag = `h${Math.min(6, depth + 2)}` as "h3";
      blocks.push(<Tag key={`h${key++}`}>{renderInline(heading[2], opts, `h${key}`)}</Tag>);
      i++;
      continue;
    }

    if (/^(-{3,}|\*{3,}|_{3,})$/.test(trimmed)) {
      flushParagraph();
      blocks.push(<hr key={`r${key++}`} />);
      i++;
      continue;
    }

    if (trimmed.startsWith(">")) {
      flushParagraph();
      const body: string[] = [];
      while (i < lines.length && lines[i].trim().startsWith(">")) {
        body.push(lines[i].trim().replace(/^>\s?/, ""));
        i++;
      }
      blocks.push(
        <blockquote key={`q${key++}`}>{renderInline(body.join("\n"), opts, `q${key}`)}</blockquote>,
      );
      continue;
    }

    // Table: a header row followed by a `|---|` divider.
    if (trimmed.includes("|") && DIVIDER_ROW.test(lines[i + 1]?.trim() ?? "")) {
      flushParagraph();
      const header = tableRow(trimmed);
      i += 2;
      const rows: string[][] = [];
      while (i < lines.length && lines[i].includes("|") && lines[i].trim() !== "") {
        rows.push(tableRow(lines[i]));
        i++;
      }
      blocks.push(
        <div className="md-table" key={`t${key++}`}>
          <table>
            <thead>
              <tr>
                {header.map((h, n) => (
                  <th key={n}>{renderInline(h, opts, `th${key}-${n}`)}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {rows.map((r, rn) => (
                <tr key={rn}>
                  {r.map((c, cn) => (
                    <td key={cn}>{renderInline(c, opts, `td${key}-${rn}-${cn}`)}</td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>,
      );
      continue;
    }

    const bullet = /^[-*+]\s+(.*)$/.exec(trimmed);
    const numbered = /^\d+[.)]\s+(.*)$/.exec(trimmed);
    if (bullet || numbered) {
      flushParagraph();
      const ordered = !!numbered;
      const items: string[] = [];
      while (i < lines.length) {
        const t = lines[i].trim();
        const b = ordered ? /^\d+[.)]\s+(.*)$/.exec(t) : /^[-*+]\s+(.*)$/.exec(t);
        if (b) {
          items.push(b[1]);
          i++;
        } else if (t !== "" && /^\s{2,}/.test(lines[i]) && items.length > 0) {
          items[items.length - 1] += `\n${t}`; // continuation line
          i++;
        } else {
          break;
        }
      }
      const List = ordered ? "ol" : "ul";
      blocks.push(
        <List key={`l${key++}`}>
          {items.map((it, n) => (
            <li key={n}>{renderInline(it, opts, `li${key}-${n}`)}</li>
          ))}
        </List>,
      );
      continue;
    }

    paragraph.push(trimmed);
    i++;
  }
  flushParagraph();

  return <div className="md">{blocks}</div>;
}
