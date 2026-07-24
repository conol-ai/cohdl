// RFC-030's closed three-tier namespace — structural: the name's own shape
// is its trust tier, never a metadata flag. Mirrors src/registry.rs.

export type Tier = "official" | "brand" | "contrib";

const SEGMENT = /^[A-Za-z0-9_][A-Za-z0-9_-]*$/;

export function nameTier(name: string): { tier: Tier; brand?: string } | { error: string } {
  if (name.startsWith("@")) {
    const rest = name.slice(1);
    const slash = rest.indexOf("/");
    if (slash < 0) {
      return { error: `\`${name}\` is not a valid package name — a scoped name is @scope/name` };
    }
    const scope = rest.slice(0, slash);
    const pkg = rest.slice(slash + 1);
    if (!SEGMENT.test(scope) || !SEGMENT.test(pkg)) {
      return {
        error: `\`${name}\` is not a valid package name — each segment uses letters, digits, _, -`,
      };
    }
    return scope === "contrib" ? { tier: "contrib" } : { tier: "brand", brand: scope };
  }
  if (!SEGMENT.test(name)) {
    return { error: `\`${name}\` is not a valid package name` };
  }
  return { tier: "official" };
}

export interface PublisherGrants {
  isOfficial: boolean;
  verifiedBrands: string[];
}

/// The authoritative publish rule (the client-side check is a fast-fail
/// convenience, never a substitute — RFC-030 Gradeability).
export function publishRejection(name: string, grants: PublisherGrants): string | null {
  const t = nameTier(name);
  if ("error" in t) return t.error;
  switch (t.tier) {
    case "official":
      return grants.isOfficial
        ? null
        : "bare names are reserved for CoHDL's official account — they are never first-come-first-served; publish under @contrib/… or request brand verification";
    case "brand":
      return grants.verifiedBrands.includes(t.brand!)
        ? null
        : `\`@${t.brand}/…\` requires a verified manufacturer account for that brand — verification is human-gated; see /docs#brands`;
    case "contrib":
      return null;
  }
}
