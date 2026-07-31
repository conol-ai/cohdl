import { describe, expect, it } from "vitest";
import { sessionIdForRequest } from "../src/worker/auth";

describe("web session cookie parsing", () => {
  const id = "a".repeat(64);

  it("extracts an exact lowercase hexadecimal session cookie", () => {
    const request = new Request("https://registry.cohdl.org/api/me", {
      headers: { Cookie: `theme=dark; __Host-session=${id}; preference=compact` },
    });
    expect(sessionIdForRequest(request)).toBe(id);
  });

  it("rejects missing, malformed, and overlong session identifiers", () => {
    for (const cookie of [
      "",
      "__Host-session=short",
      `__Host-session=${"A".repeat(64)}`,
      `__Host-session=${id}a`,
      `not-__Host-session=${id}`,
      `session=${id}`,
    ]) {
      const request = new Request("https://registry.cohdl.org/api/me", {
        headers: { Cookie: cookie },
      });
      expect(sessionIdForRequest(request), cookie).toBeNull();
    }
  });
});
