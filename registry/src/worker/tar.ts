// Plain POSIX tar reader — the upload format the cohdl CLI produces
// (uncompressed, per RFC-030's ".tar.gz (or equivalent)"; see
// src/registry.rs::pack_tar). Path-traversal-safe: entries must be
// relative, `..`-free paths.

export function readTar(data: Uint8Array): Map<string, Uint8Array> {
  const out = new Map<string, Uint8Array>();
  const dec = new TextDecoder();
  let off = 0;
  while (off + 512 <= data.length) {
    const header = data.subarray(off, off + 512);
    if (header.every((b) => b === 0)) break; // end-of-archive
    const nameField = header.subarray(0, 100);
    const nameEnd = nameField.indexOf(0);
    let name = dec.decode(nameField.subarray(0, nameEnd < 0 ? 100 : nameEnd));
    // ustar prefix field (long paths).
    const prefixField = header.subarray(345, 500);
    const prefixEnd = prefixField.indexOf(0);
    const prefix = dec.decode(prefixField.subarray(0, prefixEnd < 0 ? 155 : prefixEnd));
    if (prefix.length > 0) name = `${prefix}/${name}`;

    const sizeField = dec.decode(header.subarray(124, 136)).split("\0")[0].trim();
    const size = parseInt(sizeField, 8);
    if (Number.isNaN(size)) throw new Error("malformed tar: bad size field");
    const kind = header[156];
    off += 512;

    if (kind === 0x30 /* '0' */ || kind === 0) {
      if (
        name.startsWith("/") ||
        name.split("/").some((seg) => seg === ".." || seg === "")
      ) {
        throw new Error(`tar entry \`${name}\` escapes the target`);
      }
      if (off + size > data.length) throw new Error("truncated tar entry");
      out.set(name, data.slice(off, off + size));
    }
    off += Math.ceil(size / 512) * 512;
  }
  return out;
}
