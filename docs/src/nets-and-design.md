# Nets and Design

## Design blocks

A `design` is the top-level entry point for a board or assembly. It contains component instances and net connections:

```cohdl
design MyBoard {
    inst mcu: STM32F103<pkg: LQFP48>
    inst c_vbus: MLCC<C: 100nF, V: 16V>

    net VDD: mcu.VDD_IO, c_vbus.A
    net GND: mcu.GND, c_vbus.B
}
```

The design named in `cohdl.toml` under `[design] top` is the one compiled by `cohdl build`.

## Instance statements

The `inst` keyword creates a component instance:

```cohdl
inst name: DeviceType<generic_args>
```

Examples:

```cohdl
inst mcu: STM32F103<pkg: LQFP48>
inst c1: MLCC<C: 100nF, V: 10V>
inst usb: USBTypeC
```

Generic arguments use named syntax (`param: value`). Parameters with defaults can be omitted.

### Inline AVL

Instances can include inline AVL (Approved Vendor List) entries:

```cohdl
inst r_sense: Resistor<R: 100m, pkg: R2512> {
    primary { mfr: "Vishay", mpn: "WSL2512R1000FEA" }
    alt { mfr: "Bourns", mpn: "CSS2512FT0R100" }
}
```

## Net statements

The `net` keyword declares a named electrical connection between pins:

```cohdl
net NetName: endpoint1, endpoint2, ...
```

### Pin references

Endpoints are dot-path references to instance pins:

```cohdl
net VDD: mcu.VDD_IO, c1.A
net GND: mcu.GND, c1.B, c2.B
net USB_DP: mcu.USB_DP, usb.DP
```

The syntax `instance_name.pin_name` references a specific pin on an instance. A single net can connect any number of pins.

### Multi-pin connections

When a device has a pin mapped to multiple physical pins (like `GND: [8, 23, 35, 47]`), referencing `mcu.GND` connects all of those pins to the net.

### Net names

Net names serve multiple purposes:
- **Identification** in the netlist output
- **DRC voltage inference** -- names like `3V3`, `5V`, or `1V8` are automatically parsed to determine net voltage for design rule checking
- **GND detection** -- names starting with `GND` or equal to `VSS` are recognized as ground nets by DRC rules

## Function calls

Designs can call functions to instantiate sub-circuits:

```cohdl
design Board {
    inst mcu: STM32F103<pkg: LQFP64>

    // Call a function with named arguments
    decoupling(vdd: mcu.VDD_IO, gnd: GND, cap: mlcc_100nF)

    // Call a function from a module
    power::ldo(vin: VIN, vout: VCC_3V3, gnd: GND)
}
```

## Attributes

Design statements can have attributes:

```cohdl
design Board {
    #[designator("U1")]
    inst mcu: STM32F103<pkg: LQFP64>

    #[allow(unconnected_pin)]
    inst debug_header: PinHeader_2x5
}
```

See [Designators](designators.md) for the `#[designator]` attribute and [DRC](drc.md) for the `#[allow]` attribute.

## Complete example

```cohdl
design USBBoard {
    // Component instances
    inst mcu: STM32F103<pkg: LQFP48>
    inst usb: USBTypeC
    inst c_vbus: MLCC<C: 100nF, V: 16V>
    inst c_bypass: MLCC<C: 100nF, V: 10V>

    // Power nets
    net VDD: mcu.VDD_IO, c_bypass.A
    net VBUS: usb.VBUS, c_vbus.A
    net GND: mcu.GND, usb.GND, c_vbus.B, c_bypass.B

    // Signal nets
    net USB_DP: mcu.USB_DP, usb.DP
    net USB_DM: mcu.USB_DM, usb.DM

    // Reusable sub-circuit
    decoupling(vdd: mcu.VDD_CORE, gnd: GND, cap: mlcc_100nF)
}
```
