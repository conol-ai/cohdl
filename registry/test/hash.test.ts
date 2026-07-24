// The server-side RFC-029 hash MUST be byte-identical to the compiler's
// (src/hash.rs) — the vector below was computed independently with the Rust
// implementation AND a reference Python implementation (both agree).
import { describe, expect, it } from "vitest";
import { packageContentHash } from "../src/worker/hash";

const enc = new TextEncoder();

describe("packageContentHash", () => {
  it("matches the Rust/Python cross-language vector", async () => {
    const files = new Map<string, Uint8Array>([
      ["cohdl.toml", enc.encode('[package]\nname = "@contrib/vector"\nversion = "1.0.0"\n')],
      ["src/lib.cohdl", enc.encode("pub device V { pins { A: 1 [passive] } }\n")],
    ]);
    expect(await packageContentHash(files)).toBe(
      "sha256:bd367483bb197980aa82858bbcdc93715b6fd871ed1ea40b9db770c485647752",
    );
  });

  it("is order-independent (sorted by path)", async () => {
    const a = new Map([
      ["b.cohdl", enc.encode("bb")],
      ["a.cohdl", enc.encode("aa")],
    ]);
    const b = new Map([
      ["a.cohdl", enc.encode("aa")],
      ["b.cohdl", enc.encode("bb")],
    ]);
    expect(await packageContentHash(a)).toBe(await packageContentHash(b));
  });
});
