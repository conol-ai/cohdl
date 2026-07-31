import { describe, expect, it } from "vitest";
import { formatDate, formatSize } from "../src/ui/components";

describe("registry UI formatting", () => {
  it("formats artifact sizes for compact metadata", () => {
    expect(formatSize(0)).toBe("0 B");
    expect(formatSize(1023)).toBe("1023 B");
    expect(formatSize(1024)).toBe("1.0 KB");
    expect(formatSize(15 * 1024 * 1024)).toBe("15 MB");
  });

  it("formats valid dates and preserves invalid source values", () => {
    expect(formatDate("2026-07-31T02:09:55.699Z")).toMatch(/Jul.*2026|2026.*Jul/);
    expect(formatDate("not-a-date")).toBe("not-a-date");
  });
});
