# Designators

Reference designators (like `C1`, `R3`, `U2`) are the standard identifiers for component instances on a PCB. cohdl manages designator assignment automatically with stable, deterministic numbering.

## Automatic assignment

By default, cohdl assigns designators automatically based on the trait's `designator_prefix`:

- Devices implementing `Capacitor` (prefix `"C"`) get `C1`, `C2`, `C3`, ...
- Devices implementing `Resistor` (prefix `"R"`) get `R1`, `R2`, `R3`, ...
- Devices with no trait prefix default to `U1`, `U2`, `U3`, ...

Each prefix is numbered independently starting from 1.

## Explicit overrides

Use the `#[designator]` attribute to force a specific designator:

```cohdl
design Board {
    #[designator("U1")]
    inst mcu: STM32F103<pkg: LQFP48>

    inst c1: MLCC<C: 100nF>  // auto-assigned C1
    inst c2: MLCC<C: 10nF>   // auto-assigned C2
}
```

The compiler will report an error if two instances are assigned the same designator (whether by override or auto-assignment).

## Stability across edits

Designator assignments are tracked in a lock file (`design.lock`) so they remain stable across source edits:

- **Adding** a new component gets the next available number
- **Removing** a component tombstones its designator (never reused)
- **Reordering** source code does not change existing assignments

### Lock file format

The lock file is TOML with two sections:

```toml
[designators]
"Board::c1" = "C1"
"Board::c2" = "C2"
"Board::mcu" = "U1"

[tombstones]
"Board::old_cap" = "C3"
```

- `[designators]` maps hierarchical instance paths to their assigned designators
- `[tombstones]` preserves designators of removed instances so they are never reused

The lock file should be committed to version control alongside your source files.

## Conflict detection

The compiler detects and reports conflicts:

- Two `#[designator]` overrides with the same value
- An override that conflicts with a tombstoned designator
- An override that conflicts with an existing auto-assignment to a different instance

## Hierarchical paths

Designators are keyed by hierarchical path, which includes the design name and instance name:

```
Board::c1      -> C1
Board::mcu     -> U1
```

For components instantiated through function calls, the path includes the call hierarchy.

## Prefix conventions

Common industry-standard prefixes:

| Prefix | Component type |
|--------|---------------|
| `C`    | Capacitor     |
| `R`    | Resistor      |
| `L`    | Inductor      |
| `U`    | IC / default  |
| `J`    | Connector     |
| `D`    | Diode         |
| `Q`    | Transistor    |
| `F`    | Fuse          |
| `Y`    | Crystal       |
