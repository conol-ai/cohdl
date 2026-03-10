# Functions

Functions define reusable sub-circuit templates. They encapsulate common circuit patterns (like decoupling, voltage dividers, or filtering) that can be instantiated multiple times across a design.

## Basic syntax

```cohdl
fn function_name(param1: Type1, param2: Type2) {
    // body: inst, net, and call statements
}
```

## Parameters

Function parameters declare the inputs to the sub-circuit. Parameters are typically `Net` types (for connecting to the caller's nets) or device types:

```cohdl
fn decoupling(vdd: Net, gnd: Net) {
    inst c: MLCC<C: 100nF>
    net vdd: c.A
    net gnd: c.B
}
```

## Generic parameters

Functions can have generic parameters with type constraints:

```cohdl
fn decoupling<P: impl Capacitor>(vdd: Net, gnd: Net, cap: P) {
    inst c: cap
    net vdd: c.A
    net gnd: c.B
}
```

This allows the caller to specify which capacitor type to use:

```cohdl
design Board {
    inst mcu: STM32F103
    decoupling(vdd: mcu.VDD_IO, gnd: GND, cap: mlcc_100nF)
}
```

## Function body

The body of a function can contain the same statements as a design:

- **`inst`** -- instantiate a component
- **`net`** -- connect pins to nets
- **call statements** -- call other functions

```cohdl
fn voltage_divider<R1: Ohms, R2: Ohms>(vin: Net, vout: Net, gnd: Net) {
    inst r_top: Resistor<R: R1>
    inst r_bot: Resistor<R: R2>
    net vin: r_top.A
    net vout: r_top.B, r_bot.A
    net gnd: r_bot.B
}
```

## Calling functions

Functions are called from designs or other functions using named arguments:

```cohdl
design Board {
    inst mcu: STM32F103<pkg: LQFP64>

    // Call a local function
    decoupling(vdd: mcu.VDD_IO, gnd: GND, cap: mlcc_100nF)

    // Call a function from a module
    power::ldo(vin: VIN, vout: VCC_3V3, gnd: GND)
}
```

Arguments use named syntax (`param_name: value`) for clarity.

## Visibility

Functions can be marked `pub` to make them accessible from other modules:

```cohdl
module power {
    pub fn decoupling<P: impl Capacitor>(vdd: Net, gnd: Net, cap: P) {
        inst c: cap
        net vdd: c.A
        net gnd: c.B
    }
}
```

## Attributes on function body statements

Statements inside a function body can have attributes, just like design statements:

```cohdl
fn power_section(vdd: Net, gnd: Net) {
    #[designator("C10")]
    inst c_bulk: MLCC<C: 10uF, V: 10V, pkg: C0603>
    net vdd: c_bulk.A
    net gnd: c_bulk.B
}
```

## Complete example

```cohdl
fn voltage_divider<R1: Ohms, R2: Ohms>(vin: Net, vout: Net, gnd: Net) {
    inst r_top: Resistor<R: R1>
    inst r_bot: Resistor<R: R2>
    net vin: r_top.A
    net vout: r_top.B, r_bot.A
    net gnd: r_bot.B
}

design SensorBoard {
    inst adc: ADS1115
    voltage_divider(vin: SENSOR_OUT, vout: adc.AIN0, gnd: GND)
}
```
