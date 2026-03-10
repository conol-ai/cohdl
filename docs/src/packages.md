# Packages

This page covers two uses of "packages" in cohdl: the **import system** for organizing code across files, and the **physical packages** that describe component form factors.

## Import system

### `use` declarations

Import items from modules into the current scope:

```cohdl
// Import a single item
use power::decoupling

// Import multiple items from one module
use passives::{res_10k, mlcc_100nF}

// Import from nested modules
use peripherals::usb::USBTypeC
```

### Grouped imports

Multiple items from the same module can be imported in a single `use` statement with brace syntax:

```cohdl
use passives::{BypassCap, mlcc_100nF, res_10k}
```

### Qualified paths

Items can always be referenced by their full path without importing:

```cohdl
design Board {
    inst c1: passives::BypassCap
    power::decoupling(vdd: VDD, gnd: GND, cap: passives::mlcc_100nF)
}
```

### `mod` declarations

Reference an external module defined in a separate file:

```cohdl
mod common
```

This imports the contents of `common.cohdl` as a module named `common`.

## Physical packages

The `Package` type represents a physical component package (footprint). Package values are used in device definitions and generic parameters:

```cohdl
device MLCC<pkg: Package = C0402> {
    package: pkg
    // ...
}

device STM32F103<pkg: Package = LQFP48> {
    package: pkg
    // ...
}
```

### Built-in package names

cohdl includes a default footprint table that maps common package names to KiCad footprints:

| Package name | KiCad footprint                        |
|-------------|----------------------------------------|
| `C0402`     | `Capacitor_SMD:C_0402_1005Metric`      |
| `C0603`     | `Capacitor_SMD:C_0603_1608Metric`      |
| `R0402`     | `Resistor_SMD:R_0402_1005Metric`       |
| `R0603`     | `Resistor_SMD:R_0603_1608Metric`       |
| `LQFP48`    | `Package_QFP:LQFP-48_7x7mm_P0.5mm`    |
| `LQFP64`    | `Package_QFP:LQFP-64_10x10mm_P0.5mm`  |
| `SOT-23`    | `Package_TO_SOT_SMD:SOT-23`           |

Unknown package names are emitted as `Unknown:PackageName` in the netlist. Custom footprint mappings can be configured -- see [KiCad Backend](backends/kicad.md).
