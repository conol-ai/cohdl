// `[package]` metadata from a package's own cohdl.toml — the manifest is
// the sole identity authority (RFC-029), so publish verifies the declared
// name/version against the URL and records the display metadata the web UI
// shows (description / license / repository).
//
// The line-based parse mirrors src/deps.rs::read_package_identity exactly
// (sections by `[…]` line, `key = value` split on the first `=`, values
// trimmed of surrounding double quotes) so the server never accepts a
// manifest the compiler would read differently.

export interface PackageManifest {
  name: string | null;
  version: string | null;
  description: string | null;
  license: string | null;
  repository: string | null;
}

/// Registry policy: a published version MUST declare its license. A package
/// someone can pin into a board they manufacture is a package whose terms
/// they must be able to read, so an undeclared license is a refusal rather
/// than a blank field on the page. Returns the rejection message, or null
/// when the manifest satisfies the policy.
///
/// The value itself is not validated against a license list — proprietary
/// and custom terms are legitimate; what the registry refuses is silence.
export function metadataRejection(m: PackageManifest): string | null {
  if (!m.license || m.license.trim() === "") {
    return "publishing requires `[package] license` in cohdl.toml — every published version must declare its license";
  }
  return null;
}

/// Rust's `trim_matches('"')`: strip ALL leading and trailing double quotes.
function trimQuotes(v: string): string {
  return v.replace(/^"+/, "").replace(/"+$/, "");
}

export function parsePackageManifest(text: string): PackageManifest {
  const out: PackageManifest = {
    name: null,
    version: null,
    description: null,
    license: null,
    repository: null,
  };
  let section = "";
  for (const raw of text.split("\n")) {
    const line = raw.trim();
    if (line.startsWith("[") && line.endsWith("]")) {
      section = line.slice(1, -1).trim();
      continue;
    }
    const eq = line.indexOf("=");
    if (eq < 0 || section !== "package") continue;
    const key = line.slice(0, eq).trim();
    const value = trimQuotes(line.slice(eq + 1).trim());
    if (key === "name") out.name = value;
    else if (key === "version") out.version = value;
    else if (key === "description") out.description = value;
    else if (key === "license") out.license = value;
    else if (key === "repository") out.repository = value;
  }
  return out;
}
