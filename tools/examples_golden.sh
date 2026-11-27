#!/bin/bash
# usage: sh tools/examples_golden.sh [WORKTREE] [LABEL]
# Check and build every example that has a committed out/ against this tree's own lib/, offline
# (a fresh COHDL_HOME per example holding only lib/, COHDL_REGISTRY at a dead port), and diff the
# build with the committed out/. .xml/.enet/.html and the manifest are not compared. Needs
# target/debug/cohdl. Prints one line per example; "diff: NONE" means the golden output matches.
set -u
W=${1:-.}; L=${2:-golden}
cd "$W" || exit 1
export COHDL_REGISTRY=http://127.0.0.1:9
T=$(cd "$(mktemp -d "${TMPDIR:-/tmp}/cohdl-golden-$L.XXXX")" && pwd -P)  # cohdl refuses an out dir under a symlink (/var, /tmp on macOS)
for exdir in examples/*/out; do
  ex=$(basename "$(dirname "$exdir")")
  H="$T/home-$ex"; O="$T/build-$ex"; mkdir -p "$H"
  for d in lib/*/ lib/@*/*/; do d=${d%/}; [ -f "$d/cohdl.toml" ] || continue
    n=$(sed -n 's/^name = "\(.*\)"$/\1/p' "$d/cohdl.toml" | head -1); v=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$d/cohdl.toml" | head -1)
    mkdir -p "$H/registry/$n"; cp -R "$d" "$H/registry/$n/$v"; done
  COHDL_HOME=$H ./target/debug/cohdl check "examples/$ex" > "$O.check.log" 2>&1; c=$?
  COHDL_HOME=$H ./target/debug/cohdl build "examples/$ex" --out-dir "$O" > "$O.log" 2>&1; rc=$?
  d=$(/usr/bin/diff -rq "$O" "examples/$ex/out" 2>&1 | grep -v -E "\.(xml|enet|html)\$|cohdl-manifest")
  echo "[$L] $ex check exit $c ($(grep -c -E '^(error|warning)' "$O.check.log") diag lines) build exit $rc; files $(find "$O" -type f | wc -l | tr -d ' ') vs golden $(git ls-files "examples/$ex/out" | wc -l | tr -d ' '); diff: ${d:-NONE}"
done
rm -rf "${T:?}"
echo "[$L] git status: $(git status --short | tr '\n' ' ')"
