#!/bin/bash
set -e

# ── Build & install CLI + LSP ────────────────────────────────────────────────

cargo install --path crates/cohdl-cli
cargo install --path crates/cohdl-lsp

# ── Install std library ──────────────────────────────────────────────────────

rm -rf ~/.cohal/lib/std
mkdir -p ~/.cohal/lib
cp -r std ~/.cohal/lib/

# ── Build & install VS Code extension ────────────────────────────────────────

EXT_DIR="editors/vscode"

install_extension() {
    local cmd="$1"
    local name="$2"

    echo "  Installing cohdl extension for ${name}..."
    "$cmd" --install-extension "${EXT_DIR}/cohdl-lang-0.1.0.vsix" --force
}

if [ -d "$EXT_DIR" ]; then
    echo "Building VS Code extension..."
    (cd "$EXT_DIR" && npm install && npm run compile)

    # Package the extension into a .vsix
    if ! command -v vsce &>/dev/null; then
        npm install -g @vscode/vsce
    fi
    (cd "$EXT_DIR" && vsce package --no-dependencies)

    # Install into VS Code if available
    if command -v code &>/dev/null; then
        install_extension code "VS Code"
    fi

    # Install into Cursor if available
    if command -v cursor &>/dev/null; then
        install_extension cursor "Cursor"
    fi

    # Clean up .vsix
    rm -f "${EXT_DIR}/cohdl-lang-0.1.0.vsix"
fi
