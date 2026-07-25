// RFC-017 `#[doc("path")]` discovery for the web UI's docs rendering
// (RFC-030 names "README rendering from a package's own #[doc(...)]-
// referenced content" as web-UI parity scope).
//
// The scan is lexical: `//` line comments are stripped (the language's only
// comment form), then every `#[doc("…")]` string is collected. Paths are
// validated with the same package-relative grammar parse.rs enforces
// (component-wise: no absolute paths, no `\`, no empty/`.`/`..` components,
// no URI scheme in the first component), and only paths that actually exist
// in the published tar are kept — the tar in R2 stays the single source of
// truth; this list is just the derived index stored per immutable version.

const DOC_ATTR = /#\[\s*doc\s*\(\s*"([^"]*)"\s*\)\s*\]/g;

/// The same lexical package-relative-path validation as parse.rs take_docs.
export function validDocPath(path: string): boolean {
  if (path.trim() === "" || path.includes("\\") || path.startsWith("/")) return false;
  const components = path.split("/");
  if (components.some((c) => c === "" || c === "." || c === "..")) return false;
  if (components[0].includes(":")) return false;
  return true;
}

/// Every valid, tar-present doc path referenced by any `.cohdl` file in the
/// package — sorted and deduplicated (deterministic like everything else).
export function docPaths(files: Map<string, Uint8Array>): string[] {
  const found = new Set<string>();
  const decoder = new TextDecoder();
  for (const [name, content] of files) {
    if (!name.endsWith(".cohdl")) continue;
    for (const rawLine of decoder.decode(content).split("\n")) {
      const slash = rawLine.indexOf("//");
      const line = slash >= 0 ? rawLine.slice(0, slash) : rawLine;
      for (const m of line.matchAll(DOC_ATTR)) {
        const path = m[1];
        if (validDocPath(path) && files.has(path)) found.add(path);
      }
    }
  }
  return [...found].sort();
}

/// Content types for serving a doc file out of the tar. Everything is served
/// with `Content-Security-Policy: sandbox` (a doc is untrusted publisher
/// content on the registry's origin), so even SVG stays inert.
export function docContentType(path: string): string {
  const ext = path.slice(path.lastIndexOf(".") + 1).toLowerCase();
  const types: Record<string, string> = {
    md: "text/markdown; charset=utf-8",
    txt: "text/plain; charset=utf-8",
    pdf: "application/pdf",
    png: "image/png",
    jpg: "image/jpeg",
    jpeg: "image/jpeg",
    gif: "image/gif",
    svg: "image/svg+xml",
    webp: "image/webp",
  };
  return types[ext] ?? "application/octet-stream";
}
