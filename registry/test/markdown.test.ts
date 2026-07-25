import { describe, expect, it } from "vitest";
import { safeUrl } from "../src/ui/markdown";

// Published documents are untrusted publisher content, so a link target only
// becomes an `href` if it cannot execute script. Relative paths reach the
// caller's resolver (same-version document URLs); everything else is refused
// and the renderer degrades to plain text.
const resolve = (path: string) => `/api/doc?path=${path}`;

describe("safeUrl", () => {
  it("passes the schemes that cannot execute", () => {
    expect(safeUrl("https://example.com/x")).toBe("https://example.com/x");
    expect(safeUrl("http://example.com/x")).toBe("http://example.com/x");
    expect(safeUrl("mailto:someone@example.com")).toBe("mailto:someone@example.com");
    expect(safeUrl("  https://example.com/x  ")).toBe("https://example.com/x");
  });

  it("refuses script-bearing and otherwise surprising schemes", () => {
    for (const bad of [
      "javascript:alert(1)",
      "JavaScript:alert(1)",
      "  javascript:alert(1)",
      "data:text/html;base64,PHNjcmlwdD4=",
      "vbscript:msgbox",
      "file:///etc/passwd",
      "//evil.example.com/x",
      "#anchor",
    ]) {
      expect(safeUrl(bad, resolve), bad).toBeNull();
    }
  });

  it("sends relative paths through the caller's resolver", () => {
    expect(safeUrl("docs/errata.md", resolve)).toBe("/api/doc?path=docs/errata.md");
    // With no resolver there is nothing safe to point at.
    expect(safeUrl("docs/errata.md")).toBeNull();
  });
});
