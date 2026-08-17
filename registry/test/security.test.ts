import { describe, expect, it } from "vitest";
import { webJsonWriteRejection, withProductionSecurity } from "../src/worker/index";

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
    const policy = response.headers.get("Content-Security-Policy") ?? "";
    expect(policy).toContain("default-src 'self'");
    expect(policy).toContain("object-src 'none'");
    expect(policy).toContain("base-uri 'none'");
    expect(policy).toContain("form-action 'self'");
    expect(policy).toContain("frame-ancestors 'none'");
  });

  it("allows the analytics tag without opening up inline script", () => {
    const policy =
      withProductionSecurity(
        new Response("<!doctype html>", { headers: { "Content-Type": "text/html" } }),
      ).headers.get("Content-Security-Policy") ?? "";

    // The gtag loader and its beacons. The bootstrap is served from
    // /analytics.js precisely so that neither 'unsafe-inline' nor a snippet
    // hash is needed — tightening this back would silently kill analytics.
    expect(policy).toContain("https://www.googletagmanager.com");
    expect(policy).toContain("https://www.google-analytics.com");
    expect(policy).not.toContain("'unsafe-inline'; script-src");
    expect(policy).toMatch(/script-src [^;]*'self'/);
    expect(policy).not.toMatch(/script-src [^;]*'unsafe-inline'/);
  });

  it("preserves an existing document sandbox", () => {
    const response = withProductionSecurity(
      new Response("document", {
        headers: { "Content-Security-Policy": "sandbox" },
      }),
    );

    expect(response.headers.get("Content-Security-Policy")).toBe(
      "sandbox; frame-ancestors 'none'; object-src 'none'; base-uri 'none'",
    );
  });
});

describe("cookie-authenticated web writes", () => {
  const url = new URL("https://registry.cohdl.org/api/session");

  it("requires the exact origin", () => {
    for (const origin of [null, "https://evil.example", "https://docs.cohdl.org"]) {
      const headers = new Headers({ "Content-Type": "application/json" });
      if (origin) headers.set("Origin", origin);
      const request = new Request(url, { method: "POST", headers, body: "{}" });
      expect(webJsonWriteRejection(request, url)?.status).toBe(403);
    }
  });

  it("requires application/json, including for no-cors-compatible bodies", () => {
    for (const contentType of ["text/plain", "application/x-www-form-urlencoded", ""]) {
      const headers = new Headers({ Origin: url.origin });
      if (contentType) headers.set("Content-Type", contentType);
      const request = new Request(url, { method: "POST", headers, body: "{}" });
      expect(webJsonWriteRejection(request, url)?.status).toBe(415);
    }
  });

  it("accepts same-origin JSON with parameters", () => {
    const request = new Request(url, {
      method: "POST",
      headers: {
        Origin: url.origin,
        "Content-Type": "application/json; charset=utf-8",
      },
      body: "{}",
    });
    expect(webJsonWriteRejection(request, url)).toBeNull();
  });
});
