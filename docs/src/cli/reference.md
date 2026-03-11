# CLI Reference

The `cohdl` command-line tool is the compiler and primary interface for working with cohdl projects.

## Global options

```
cohdl [OPTIONS] <COMMAND>
```

| Option          | Description                            | Default |
|-----------------|----------------------------------------|---------|
| `--color`       | Control colored output: `auto`, `always`, `never` | `auto` |
| `--version`     | Print version information              |         |
| `--help`        | Print help                             |         |

## Commands

### `cohdl build`

Full compilation pipeline: parse, semantic analysis, DRC, and code generation.

```bash
cohdl build [OPTIONS]
```

| Option          | Description                                        | Default |
|-----------------|----------------------------------------------------|---------|
| `--design`      | Name of the design to compile (overrides `cohdl.toml`) | from `cohdl.toml` |
| `--emit`        | Comma-separated list of output targets             | `all`   |
| `--out-dir`     | Output directory                                   | `out`   |

#### `--emit` targets

| Target       | Description                                |
|--------------|--------------------------------------------|
| `netlist`    | KiCad legacy netlist (`.net`)              |
| `bom-simple` | Simple BOM CSV (`RefDes,MPN,Qty`)          |
| `bom-avl`    | AVL BOM CSV with alternates                |
| `all`        | All of the above (default)                 |

#### Examples

```bash
# Build with defaults (all outputs to out/)
cohdl build

# Build a specific design
cohdl build --design AlternateBoard

# Generate only the netlist
cohdl build --emit netlist

# Generate BOMs to a custom directory
cohdl build --emit bom-simple,bom-avl --out-dir build/
```

### `cohdl check`

Validation only: parse, semantic analysis, and DRC without generating output files. Useful for fast feedback during development.

```bash
cohdl check
```

Reads the `cohdl.toml` manifest and validates the top-level design. Reports any parse errors, semantic errors, or DRC diagnostics. Exits with code 0 on success, 1 on any error.

#### Example

```bash
cohdl check
# Output on success:
#   No errors found.
```

### `cohdl fmt`

Format source files (not yet implemented).

```bash
cohdl fmt
```

## Project manifest (`cohdl.toml`)

Every project requires a `cohdl.toml` file in the working directory:

```toml
[package]
name = "my-board"
version = "0.1.0"

[design]
root = "main.cohdl"
top = "MyBoard"
```

| Section     | Field     | Description                                    |
|-------------|-----------|------------------------------------------------|
| `[package]` | `name`    | Project name (used for output file naming)     |
| `[package]` | `version` | Project version                                |
| `[design]`  | `root`    | Path to the root `.cohdl` source file. Any `module` declarations in this file are resolved to sibling `.cohdl` files in the same directory. |
| `[design]`  | `top`     | Name of the top-level `design` to compile      |

The `--design` flag overrides the `top` field from the manifest.

## Compilation pipeline

The compiler runs these stages in order:

```
1. Parse        Read root file, resolve `module` declarations, parse all sources
2. Resolve      Name resolution and symbol table construction
3. Type check   Generic instantiation and type validation
4. Connectivity Flatten design into instances and merged nets
5. DRC          Run built-in and user-defined design rules
6. Codegen      Emit netlist and BOM files (build only)
```

If any stage produces errors, the pipeline reports them and stops (with exit code 1). Warnings are reported but do not stop the pipeline.

## Exit codes

| Code | Meaning                                  |
|------|------------------------------------------|
| `0`  | Success                                  |
| `1`  | One or more errors (parse, sema, or DRC) |

## Diagnostic format

Errors and warnings are printed to stderr with source context:

```
Error[E001]: Board::c_vbus
  --> main.cohdl:15:5
   |
15 |     inst c_vbus: MLCC<C: 100nF, V: 5V>
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   | voltage_rating 5V is less than net `12V` voltage 12V
```

Each diagnostic shows:
- **Level and rule ID** (e.g., `Error[E001]`, `Warning[W001]`)
- **Instance path** (e.g., `Board::c_vbus`)
- **Source location** (file, line, column)
- **Source line** with an underline pointing to the relevant span
- **Message** describing the issue
