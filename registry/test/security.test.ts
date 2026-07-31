import { describe, expect, it } from "vitest";
import { withProductionSecurity } from "../src/worker/index";

describe("production response security", () => {
  it("prevents framing and enables HSTS", () => {
    const response = withProductionSecurity(
      new Response("<!doctype html>", {
        headers: { "Content-Type": "text/html" },
      }),
    );

    expect(response.headers.get("Strict-Transport-Security")).toBe(
      "max-age=31536000; includeSubDomains",
    );
    expect(response.headers.get("X-Frame-Options")).toBe("DENY");
    expect(response.headers.get("Content-Security-Policy")).toBe("frame-ancestors 'none'");
  });

  it("preserves an existing document sandbox", () => {
    const response = withProductionSecurity(
      new Response("document", {
        headers: { "Content-Security-Policy": "sandbox" },
      }),
    );

    expect(response.headers.get("Content-Security-Policy")).toBe(
      "sandbox; frame-ancestors 'none'",
    );
  });
});
