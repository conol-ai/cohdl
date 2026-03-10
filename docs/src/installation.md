# Installation

## Prerequisites

- **Rust toolchain** (1.70 or later) — install via [rustup](https://rustup.rs/)
- **Git** — for version control of your designs

If you plan to use the KiCad backend:

- **KiCad** (6.0 or later) — [kicad.org](https://www.kicad.org/)

## Building from source

Clone the repository and build with Cargo:

```bash
git clone https://github.com/conol/cohdl.git
cd cohdl
cargo build --release
```

The compiled binary will be at `target/release/cohdl`.

## Adding to your PATH

Copy or symlink the binary to a directory on your `PATH`:

```bash
# Option 1: install directly via cargo
cargo install --path crates/cohdl-cli

# Option 2: symlink
ln -s "$(pwd)/target/release/cohdl" ~/.local/bin/cohdl
```

## Verifying the installation

```bash
cohdl --version
cohdl --help
```

You should see the version number and a list of available subcommands (`build`, `check`, `fmt`).
