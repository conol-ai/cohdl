# Devices

Devices are concrete component definitions. They describe the physical package, pin mappings, and electrical specifications of a real component (or family of components).

## Basic syntax

```cohdl
device DeviceName {
    package: PackageName
    pins { /* pin mappings */ }
}
```

## Generic parameters

Devices can be parameterized with generics, making them reusable across different values and packages:

```cohdl
device MLCC<C: Farads, V: Voltage = 10V, pkg: Package = C0402>: impl Capacitor {
    package: pkg
    pins { A: 1, B: 2 }
    spec {
        capacitance: C
        voltage_rating: V
    }
}
```

Generic parameters have the form `name: Type`, where `Type` specifies the kind of value (`Farads`, `Voltage`, `Ohms`, `Package`, etc.). Parameters can have default values using `= value`.

When instantiating a device, generic arguments are passed with named syntax:

```cohdl
inst c1: MLCC<C: 100nF, V: 16V, pkg: C0603>
inst c2: MLCC<C: 100nF>  // uses defaults: V = 10V, pkg = C0402
```

## Implementing traits

Devices can implement traits using the `: impl` clause:

```cohdl
device MLCC<C: Farads, V: Voltage = 10V, pkg: Package = C0402>: impl Capacitor {
    // must provide all pins, specs, and satisfy all rules from Capacitor
}
```

Multiple traits can be implemented:

```cohdl
device MyDevice: impl TraitA + TraitB {
    // ...
}
```

When a device implements a trait, it must:
1. Provide all pins declared by the trait
2. Provide all spec fields declared by the trait
3. Pass all DRC rules defined in the trait

## Package declaration

The `package` field specifies the physical package for the device:

```cohdl
device MLCC<pkg: Package = C0402> {
    package: pkg
}
```

The package determines which KiCad footprint the component maps to. See [KiCad Backend](backends/kicad.md) for the default footprint mappings.

## Pin mappings

### Single pins

Map a logical pin name to a physical pin number:

```cohdl
pins {
    A: 1
    B: 2
    VDD_CORE: 1
    VDD_IO: 24
}
```

### Pin lists

Map a logical pin name to multiple physical pins (for components with multiple ground or power pins):

```cohdl
pins {
    GND: [8, 23, 35, 47]
}
```

### Pin ranges

Map a logical pin name to a contiguous range of physical pins:

```cohdl
pins {
    DATA: [0..7]
}
```

### Pin bus macro

The `pin_bus!` macro generates a series of numbered pin names:

```cohdl
pins {
    pin_bus!(PA, 10, 8)  // creates PA0..PA7 starting at physical pin 10
}
```

`pin_bus!(prefix, start_pin, count)` generates `count` pins named `prefix0` through `prefix(count-1)`, mapped to physical pins `start_pin` through `start_pin + count - 1`.

### Package-qualified pins

For devices that support multiple packages, pin blocks can be qualified with a package name:

```cohdl
device STM32F103<pkg: Package = LQFP48> {
    package: pkg
    pins[LQFP48] {
        VDD_CORE: 1
        VDD_IO: 24
        GND: [8, 23, 35, 47]
        pin_bus!(PA, 10, 8)
    }
}
```

The qualifier `[LQFP48]` indicates that this pin mapping applies only when the device is instantiated with that package.

## Spec blocks

Devices provide values for spec fields declared in their traits:

```cohdl
device MLCC<C: Farads, V: Voltage>: impl Capacitor {
    spec {
        capacitance: C
        voltage_rating: V
    }
}
```

Spec values can reference generic parameters, allowing a single device definition to cover many value combinations.

## Device-level DRC rules

Devices can define their own DRC rules in addition to those inherited from traits:

```cohdl
device MyRegulator: impl VoltageRegulator {
    package: SOT_23

    rule output_load(level: Warning) {
        assert self.spec.output_current >= 100mA
        message: "Output current may be insufficient"
    }
}
```

## Complete example

```cohdl
device STM32F103<pkg: Package = LQFP48> {
    package: pkg
    pins[LQFP48] {
        VDD_CORE: 1
        VDD_IO: 24
        GND: [8, 23, 35, 47]
        pin_bus!(PA, 10, 8)
        USB_DM: 20
        USB_DP: 21
    }
}

device USBTypeC: impl Connector {
    package: USB_C_SMD
    pins {
        VBUS: 1
        CC1: 2
        CC2: 3
        DP: 4
        DM: 5
        GND: [6, 7]
    }
}
```
