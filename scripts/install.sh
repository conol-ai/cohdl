#!/bin/bash
set -e

# ── Build & install CLI + LSP ────────────────────────────────────────────────

cargo install --path crates/cohdl-cli
cargo install --path crates/cohdl-lsp

# ── Install std library ──────────────────────────────────────────────────────

rm -rf ~/.cohdl/lib/std
mkdir -p ~/.cohdl/lib
cp -r std ~/.cohdl/lib/

# ── Build & install VS Code extension ────────────────────────────────────────

EXT_DIR="editors/vscode"
VSIX="${EXT_DIR}/cohdl-lang.vsix"

install_extension() {
    local cmd="$1"
    local name="$2"

    echo "  Installing cohdl extension for ${name}..."
    "$cmd" --install-extension "$VSIX" --force
}

if [ -d "$EXT_DIR" ]; then
    echo "Building VS Code extension..."
    (cd "$EXT_DIR" && npm ci && npx @vscode/vsce package -o cohdl-lang.vsix)

    # Install into VS Code if available
    if command -v code &>/dev/null; then
        install_extension code "VS Code"
    fi

    # Install into Cursor if available
    if command -v cursor &>/dev/null; then
        install_extension cursor "Cursor"
    fi

    # Clean up .vsix
    rm -f "$VSIX"
fi
