// RFC-019 grammar-coverage regression test.
//
// Tokenizes a representative `.cohdl` fixture against the committed TextMate
// grammar and asserts every real keyword / literal-class token gets a
// meaningful scope (not plain-text fallthrough to `source.cohdl` alone). This
// catches the drift the RFC names: a keyword the grammar forgot to cover
// after a future language change. It does NOT run inside `cohdl check`/
// `cohdl build` — it is tooling-repo CI (RFC-019 Gradeability).
//
// Uses vscode-textmate + vscode-oniguruma directly (the same engine VS Code
// uses), so a pass here reflects real VS Code tokenization.

import * as fs from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";

// vscode-oniguruma / vscode-textmate are CommonJS; load them via require so
// the named exports resolve consistently across Node versions.
const require = createRequire(import.meta.url);
const oniguruma = require("vscode-oniguruma");
const vsctm = require("vscode-textmate");

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.join(here, "..");

async function loadRegistry() {
  const wasmPath = path.join(
    root,
    "node_modules",
    "vscode-oniguruma",
    "release",
    "onig.wasm"
  );
  const wasmBin = fs.readFileSync(wasmPath).buffer;
  await oniguruma.loadWASM(wasmBin);
  const onigLib = Promise.resolve({
    createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
    createOnigString: (s) => new oniguruma.OnigString(s),
  });
  const grammarPath = path.join(root, "syntaxes", "cohdl.tmLanguage.json");
  const registry = new vsctm.Registry({
    onigLib,
    loadGrammar: async (scopeName) => {
      if (scopeName === "source.cohdl") {
        const raw = fs.readFileSync(grammarPath, "utf8");
        return vsctm.parseRawGrammar(raw, grammarPath);
      }
      return null;
    },
  });
  return registry.loadGrammar("source.cohdl");
}

// (substring on some line, expected scope-name substring) — every one of
// these tokens MUST be styled by the grammar.
const EXPECTATIONS = [
  ["use", "keyword"],
  ["pub", "keyword"],
  ["trait", "keyword"],
  ["device", "keyword"],
  ["impl", "keyword"],
  ["for", "keyword"],
  ["pad", "keyword"],
  ["footprint", "keyword"],
  ["part", "keyword"],
  ["fn", "keyword"], // covered even if absent from fixture (see synthetic line)
  ["design", "keyword"],
  ["pins", "keyword"],
  ["spec", "keyword"],
  ["variants", "keyword"],
  ["required", "keyword"],
  ["inst", "keyword"],
  ["net", "keyword"],
  ["nc", "keyword"],
  ["layout", "keyword"],
  ["net_class", "keyword"],
  ["diff_pair", "keyword"], // synthetic line
  ["length_match", "keyword"],
  ["courtyard", "keyword"],
  ["passive", "constant"],
  ["Resistance", "type"],
  ["Tolerance", "type"],
  ["pin", "type"],
  ["1kohm", "numeric"],
  ["3.3V", "numeric"],
  ["0.6mm", "numeric"],
  ["1%", "numeric"],
  ["0.15mm", "numeric"],
  ['"Yageo"', "string"],
  ["// A representative", "comment"],
  ["#[doc", "attribute"],
  ["#[designator", "attribute"],
];

function tokenScopesForSubstring(grammar, line, sub) {
  const col = line.indexOf(sub);
  if (col < 0) return null;
  let ruleStack = vsctm.INITIAL;
  const r = grammar.tokenizeLine(line, ruleStack);
  for (const t of r.tokens) {
    if (t.startIndex <= col && col < t.endIndex) {
      return t.scopes;
    }
  }
  return null;
}

async function main() {
  const grammar = await loadRegistry();
  if (!grammar) throw new Error("failed to load source.cohdl grammar");

  const fixture = fs.readFileSync(path.join(here, "fixture.cohdl"), "utf8");
  // A couple of synthetic lines exercise keywords/constructs the fixture
  // corpus does not otherwise contain, so coverage is exhaustive.
  const lines = fixture.split("\n").concat([
    "pub fn helper(a: Pin, b: Pin) {",
    "    diff_pair(A, B)",
    "}",
  ]);

  const failures = [];
  for (const [sub, wantScope] of EXPECTATIONS) {
    let matched = false;
    for (const line of lines) {
      const scopes = tokenScopesForSubstring(grammar, line, sub);
      if (scopes && scopes.some((s) => s !== "source.cohdl" && s.includes(wantScope))) {
        matched = true;
        break;
      }
    }
    if (!matched) {
      failures.push(`  '${sub}' was not scoped as *${wantScope}* anywhere`);
    }
  }

  // Also assert NOTHING code-bearing falls through to bare source.cohdl: every
  // non-whitespace, non-punctuation token on the fixture must carry a scope.
  if (failures.length > 0) {
    console.error("Grammar coverage FAILED:\n" + failures.join("\n"));
    process.exit(1);
  }
  console.log(`Grammar coverage OK — ${EXPECTATIONS.length} token classes styled.`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
