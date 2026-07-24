import { describe, expect, it } from "vitest";
import { nameTier, publishRejection } from "../src/worker/namespace";

describe("the three-tier namespace (RFC-030)", () => {
  it("classifies structurally from the name alone", () => {
    expect(nameTier("std")).toEqual({ tier: "official" });
    expect(nameTier("@sparkfun/power")).toEqual({ tier: "brand", brand: "sparkfun" });
    expect(nameTier("@contrib/widgets")).toEqual({ tier: "contrib" });
    for (const bad of ["@nope", "@a b/x", "a b", "@contrib/"]) {
      expect(nameTier(bad)).toHaveProperty("error");
    }
  });

  it("bare names are never first-come-first-served", () => {
    const anon = { isOfficial: false, verifiedBrands: [] };
    expect(publishRejection("sensors", anon)).toMatch(/reserved for CoHDL/);
    expect(publishRejection("sensors", { isOfficial: true, verifiedBrands: [] })).toBeNull();
  });

  it("brand prefixes need verification for that exact brand", () => {
    const sparkfun = { isOfficial: false, verifiedBrands: ["sparkfun"] };
    expect(publishRejection("@sparkfun/power", sparkfun)).toBeNull();
    expect(publishRejection("@adafruit/motors", sparkfun)).toMatch(/verified manufacturer/);
  });

  it("contrib is open to any authenticated account", () => {
    expect(
      publishRejection("@contrib/widgets", { isOfficial: false, verifiedBrands: [] }),
    ).toBeNull();
  });
});
