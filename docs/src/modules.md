# Modules

Modules provide namespace grouping for organizing large designs into logical sections. They help keep related traits, devices, parts, functions, and types together.

## Inline modules

Define a module inline with its contents:

```cohdl
module power {
    pub fn decoupling<P: impl Capacitor>(vdd: Net, gnd: Net, cap: P) {
        inst c: cap
        net vdd: c.A
        net gnd: c.B
    }
}

module passives {
    pub type BypassCap = MLCC<C: 100nF, V: 10V, pkg: C0402>

    pub part mlcc_100nF: MLCC<C: 100nF, pkg: C0402> {
        primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }
    }
}
```

## External modules

Reference a module defined in a separate file:

```cohdl
module common
```

This tells the compiler to look for a file named `common.cohdl` in the same directory and include its contents. This works like Rust's `mod` keyword — the `module` declaration without braces loads the corresponding `.cohdl` file.

Multiple modules can be declared:

```cohdl
module devices
module connectors
module passives

design MainBoard {
    inst mcu: STM32F103<pkg: "LQFP48">
    inst c1: mlcc_100nF_0402
    // ...
}
```

## Visibility

Items inside a module are private by default. Use `pub` to make them accessible from outside the module:

```cohdl
module passives {
    pub type BypassCap = MLCC<C: 100nF, V: 10V, pkg: C0402>  // accessible
    type InternalType = MLCC<C: 10nF>                          // private
}
```

## Nested modules

Modules can contain other modules:

```cohdl
module peripherals {
    module usb {
        pub device USBTypeC: impl Connector {
            package: USB_C_SMD
            pins { VBUS: 1, DP: 4, DM: 5, GND: [6, 7] }
        }
    }

    module spi {
        // ...
    }
}
```

## Module contents

A module can contain any top-level item:

- `trait` definitions
- `device` definitions
- `part` declarations
- `type` aliases
- `fn` definitions
- `use` imports
- `module` references (external file modules)
- Nested `module` definitions

## Using items from modules

Import items from modules with `use` declarations:

```cohdl
use power::decoupling
use passives::{BypassCap, mlcc_100nF}
```

Or reference them directly with their qualified path:

```cohdl
design Board {
    inst c1: passives::BypassCap
    power::decoupling(vdd: VDD, gnd: GND, cap: passives::mlcc_100nF)
}
```

See [Packages](packages.md) for more on the import system.
