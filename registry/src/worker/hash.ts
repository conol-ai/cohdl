// RFC-029's package content hash, recomputed SERVER-SIDE on every publish —
// the registry's own hash is the authoritative identity cohdl.lock verifies
// against (RFC-030: the publisher's local computation is never trusted).
//
// Recipe (must stay byte-identical to src/hash.rs::package_content_hash):
// files sorted by `/`-joined relative path; each contributes
// `path NUL <decimal length> NUL <content bytes>`; sha256 over the whole.

export async function packageContentHash(
  files: Map<string, Uint8Array>,
): Promise<string> {
  const enc = new TextEncoder();
  const parts: Uint8Array[] = [];
  const nul = new Uint8Array([0]);
  for (const path of [...files.keys()].sort()) {
    const content = files.get(path)!;
    parts.push(enc.encode(path), nul, enc.encode(String(content.length)), nul, content);
  }
  const total = parts.reduce((n, p) => n + p.length, 0);
  const buf = new Uint8Array(total);
  let off = 0;
  for (const p of parts) {
    buf.set(p, off);
    off += p.length;
  }
  const digest = await crypto.subtle.digest("SHA-256", buf);
  const hex = [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, "0")).join("");
  return `sha256:${hex}`;
}
