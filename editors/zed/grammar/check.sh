#!/usr/bin/env bash
# The grammar's CI gate, runnable locally from this directory:
#   1. the committed generated parser is current (tree-sitter generate is
#      a no-op against src/),
#   2. the corpus tests pass,
#   3. every .cohdl file in the repository parses with ZERO ERROR/MISSING
#      nodes — the grammar's whole contract is that unknown constructs
#      degrade to plain tokens, never to error recovery.
set -euo pipefail
cd "$(dirname "$0")"

npx tree-sitter generate
if ! git diff --exit-code -- src; then
    echo "generated parser is stale: run 'npx tree-sitter generate' and commit src/" >&2
    exit 1
fi

npx tree-sitter test

repo_root="$(git rev-parse --show-toplevel)"
bad=0
total=0
while IFS= read -r f; do
    total=$((total + 1))
    n="$(npx tree-sitter parse "$f" 2>/dev/null | grep -c 'ERROR\|MISSING' || true)"
    if [ "$n" != "0" ]; then
        bad=$((bad + 1))
        echo "PARSE ERRORS ($n): $f" >&2
    fi
done < <(find "$repo_root/lib" "$repo_root/examples" -name '*.cohdl')

echo "parsed $total files, $bad with errors"
test "$bad" = 0
