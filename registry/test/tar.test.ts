import { describe, expect, it } from "vitest";
import { readTar } from "../src/worker/tar";

// A tar writer mirroring the CLI's (src/registry.rs::pack_tar) so the reader
// is tested against exactly what clients upload.
function header(name: string, size: number): Uint8Array {
  const h = new Uint8Array(512);
  const enc = new TextEncoder();
  h.set(enc.encode(name).subarray(0, 100), 0);
  h.set(enc.encode("0000644"), 100);
  h.set(enc.encode("0000000"), 108);
  h.set(enc.encode("0000000"), 116);
  h.set(enc.encode(size.toString(8).padStart(11, "0")), 124);
  h.set(enc.encode("00000000000"), 136);
  h[156] = 0x30;
  h.set(enc.encode("ustar"), 257);
  h.set(enc.encode("00"), 263);
  h.set(enc.encode("        "), 148);
  const sum = h.reduce((a, b) => a + b, 0);
  h.set(enc.encode(sum.toString(8).padStart(6, "0") + "\0 "), 148);
  return h;
}

function pack(files: [string, string][]): Uint8Array {
  const enc = new TextEncoder();
  const parts: Uint8Array[] = [];
  for (const [name, text] of files) {
    const content = enc.encode(text);
    parts.push(header(name, content.length), content);
    const pad = (512 - (content.length % 512)) % 512;
    parts.push(new Uint8Array(pad));
  }
  parts.push(new Uint8Array(1024));
  const total = parts.reduce((n, p) => n + p.length, 0);
  const out = new Uint8Array(total);
  let off = 0;
  for (const p of parts) {
    out.set(p, off);
    off += p.length;
  }
  return out;
}

describe("readTar", () => {
  it("round-trips a package archive", () => {
    const tar = pack([
      ["cohdl.toml", '[package]\nname = "x"\nversion = "1.0.0"\n'],
      ["src/lib.cohdl", "pub device D { pins { A: 1 [passive] } }\n"],
    ]);
    const files = readTar(tar);
    expect([...files.keys()].sort()).toEqual(["cohdl.toml", "src/lib.cohdl"]);
    expect(new TextDecoder().decode(files.get("src/lib.cohdl"))).toContain("pub device D");
  });

  it("rejects path traversal", () => {
    const tar = pack([["../evil.txt", "boom"]]);
    expect(() => readTar(tar)).toThrow(/escapes/);
  });

  it("rejects absolute paths", () => {
    const tar = pack([["/etc/hosts", "boom"]]);
    expect(() => readTar(tar)).toThrow(/escapes/);
  });
});
