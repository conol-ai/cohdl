import { describe, expect, it } from "vitest";
import { docContentType, docPaths, validDocPath } from "../src/worker/docs";

function pkg(files: Record<string, string>): Map<string, Uint8Array> {
  const enc = new TextEncoder();
  return new Map(Object.entries(files).map(([k, v]) => [k, enc.encode(v)]));
}

describe("validDocPath", () => {
  it("accepts package-relative paths", () => {
    expect(validDocPath("README.md")).toBe(true);
    expect(validDocPath("docs/datasheet.pdf")).toBe(true);
  });

  it("refuses everything parse.rs refuses", () => {
    for (const bad of [
      "",
      "   ",
      "/etc/passwd",
      "../escape.md",
      "./readme.md",
      "docs//x.md",
      "docs/",
      "docs\\x.md",
      "file:/x.md",
      "https://example.com/x.md",
    ]) {
      expect(validDocPath(bad), bad).toBe(false);
    }
  });
});

describe("docPaths", () => {
  it("collects every #[doc] reference that exists in the archive", () => {
    const files = pkg({
      "cohdl.toml": '[package]\nname = "p"\nversion = "1.0.0"\n',
      "src/lib.cohdl": [
        '#[doc("README.md")]',
        '#[doc("docs/datasheet.pdf")]',
        "device Thing { pin A: Power }",
      ].join("\n"),
      "README.md": "# Thing",
      "docs/datasheet.pdf": "%PDF-1.4",
    });
    expect(docPaths(files)).toEqual(["README.md", "docs/datasheet.pdf"]);
  });

  it("skips references to files the archive does not contain", () => {
    const files = pkg({
      "src/lib.cohdl": '#[doc("missing.md")]\ndevice T { pin A: Power }',
    });
    expect(docPaths(files)).toEqual([]);
  });

  it("ignores commented-out references", () => {
    const files = pkg({
      "src/lib.cohdl": '// #[doc("README.md")] not active\ndevice T { pin A: Power }',
      "README.md": "# T",
    });
    expect(docPaths(files)).toEqual([]);
  });

  it("ignores strings outside .cohdl sources", () => {
    const files = pkg({
      "notes.txt": '#[doc("README.md")]',
      "README.md": "# T",
    });
    expect(docPaths(files)).toEqual([]);
  });

  it("deduplicates and sorts (deterministic like every other output)", () => {
    const files = pkg({
      "src/b.cohdl": '#[doc("b.md")]\n#[doc("a.md")]',
      "src/a.cohdl": '#[doc("a.md")]',
      "a.md": "a",
      "b.md": "b",
    });
    expect(docPaths(files)).toEqual(["a.md", "b.md"]);
  });

  it("reads the attribute through the spacing the grammar allows", () => {
    const files = pkg({
      "src/lib.cohdl": '#[ doc ( "README.md" ) ]\ndevice T { pin A: Power }',
      "README.md": "# T",
    });
    expect(docPaths(files)).toEqual(["README.md"]);
  });

  it("refuses an escaping path even when a matching file exists", () => {
    const files = pkg({
      "src/lib.cohdl": '#[doc("../escape.md")]',
      "../escape.md": "nope",
    });
    expect(docPaths(files)).toEqual([]);
  });
});

describe("docContentType", () => {
  it("names the types documents actually use", () => {
    expect(docContentType("README.md")).toBe("text/markdown; charset=utf-8");
    expect(docContentType("docs/ds.PDF")).toBe("application/pdf");
    expect(docContentType("fig.png")).toBe("image/png");
  });

  it("falls back to an inert type for anything else", () => {
    // Notably: no text/html — publisher content is never served as a document
    // the browser would treat as markup.
    expect(docContentType("index.html")).toBe("application/octet-stream");
    expect(docContentType("noext")).toBe("application/octet-stream");
  });
});
