import { describe, expect, it } from "vitest";
import { metadataRejection, parsePackageManifest } from "../src/worker/manifest";

describe("parsePackageManifest", () => {
  it("reads identity and display metadata from [package]", () => {
    const m = parsePackageManifest(
      [
        "# a comment",
        "[package]",
        'name = "@contrib/vector"',
        'version = "1.0.0"',
        'description = "Vector-drive helpers."',
        'license = "MIT"',
        'repository = "https://example.com/vector"',
        "",
        "[dependencies]",
        'std = "0.1.0"',
      ].join("\n"),
    );
    expect(m).toEqual({
      name: "@contrib/vector",
      version: "1.0.0",
      description: "Vector-drive helpers.",
      license: "MIT",
      repository: "https://example.com/vector",
    });
  });

  it("leaves undeclared keys null", () => {
    const m = parsePackageManifest('[package]\nname = "std"\nversion = "0.1.0"\n');
    expect(m.description).toBeNull();
    expect(m.license).toBeNull();
    expect(m.repository).toBeNull();
  });

  it("only reads keys inside [package]", () => {
    // A `description` under another section is not the package's.
    const m = parsePackageManifest(
      '[package]\nname = "std"\n[design]\ndescription = "not this one"\n',
    );
    expect(m.description).toBeNull();
  });

  it("keeps `=` inside a value (splits on the first one only)", () => {
    const m = parsePackageManifest('[package]\ndescription = "a = b"\n');
    expect(m.description).toBe("a = b");
  });

  it("tolerates the CRLF and loose spacing a hand-edited manifest carries", () => {
    // Rust's `str::lines()` drops the `\r` and so does this reader's trim,
    // which is what keeps server and compiler reading the same manifest.
    const m = parsePackageManifest('[package]\r\n  name   =   "std"  \r\n');
    expect(m.name).toBe("std");
  });
});

describe("metadataRejection", () => {
  const manifest = (license: string | null) => ({
    name: "@contrib/p",
    version: "1.0.0",
    description: null,
    license,
    repository: null,
  });

  it("accepts any declared license, including proprietary terms", () => {
    for (const ok of ["MIT", "Apache-2.0", "LicenseRef-Acme-Proprietary", "see LICENSE.txt"]) {
      expect(metadataRejection(manifest(ok)), ok).toBeNull();
    }
  });

  it("refuses a version that declares no license", () => {
    expect(metadataRejection(manifest(null))).toContain("`[package] license`");
  });

  it("refuses a license that is present but empty", () => {
    // `license = ""` and `license = "   "` are silence with extra steps.
    for (const blank of ["", "   ", "\t"]) {
      expect(metadataRejection(manifest(blank)), JSON.stringify(blank)).toContain(
        "`[package] license`",
      );
    }
  });

  it("says what to do, naming the key and the file", () => {
    const msg = metadataRejection(manifest(null))!;
    expect(msg).toContain("cohdl.toml");
    expect(msg).toContain("license");
  });
});
