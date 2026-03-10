# Quick Start

This guide walks you through creating a minimal cohdl project and compiling it to a KiCad netlist.

## 1. Create a project directory

```bash
mkdir my-board && cd my-board
```

## 2. Create `cohdl.toml`

Every cohdl project needs a manifest file:

```toml
[package]
name = "my-board"
version = "0.1.0"

[design]
root = "main.cohdl"
top = "MyBoard"
```

- `root` — the source file to compile
- `top` — the name of the top-level `design` to compile

## 3. Write your first design

Create `main.cohdl`:

```cohdl
// Define a two-terminal trait for passive components
trait TwoTerminal {
    pins {
        A: Pin
        B: Pin
    }
}

// Define a capacitor trait with specs and DRC
trait Capacitor: TwoTerminal {
    designator_prefix: "C"
    spec {
        capacitance: Farads
        voltage_rating: Voltage
    }
    rule voltage_exceed(level: Error) {
        assert net_voltage(self.A, self.B) <= self.spec.voltage_rating
        message: "Voltage exceeds capacitor rating"
    }
}

// A generic ceramic capacitor device
device MLCC<C: Farads, V: Voltage = 10V, pkg: Package = C0402>: impl Capacitor {
    package: pkg
    pins { A: 1, B: 2 }
    spec {
        capacitance: C
        voltage_rating: V
    }
}

// An MCU device
device STM32F103<pkg: Package = LQFP48> {
    package: pkg
    pins[LQFP48] {
        VDD_IO: 24
        GND: [8, 23, 35, 47]
        pin_bus!(PA, 10, 8)
    }
}

// Bind a device to a real manufacturer part
part mlcc_100nF: MLCC<C: 100nF, V: 10V> {
    primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }
    alt { mfr: "Murata", mpn: "GRM155R61C104KA88D" }
}

// Top-level board design
design MyBoard {
    inst mcu: STM32F103<pkg: LQFP48>
    inst c_bypass: MLCC<C: 100nF, V: 10V>

    net VDD: mcu.VDD_IO, c_bypass.A
    net GND: mcu.GND, c_bypass.B
}
```

## 4. Check your design

Run the compiler in check mode to validate without generating output:

```bash
cohdl check
```

If there are no errors, you'll see:

```
  No errors found.
```

## 5. Build

Generate output files:

```bash
cohdl build
```

This creates the `out/` directory containing:

- `my-board.net` — KiCad legacy netlist
- `my-board-bom.csv` — simple Bill of Materials
- `my-board-bom-avl.csv` — AVL Bill of Materials with alternates

## 6. Import into KiCad

1. Open KiCad and create a new project
2. Open the PCB editor
3. Go to **File > Import Netlist**
4. Select the generated `.net` file
5. Click **Update PCB**

Your components and nets are now in KiCad, ready for layout.

## Next steps

- Read the [Language Reference](traits.md) to learn about all cohdl constructs
- See [DRC](drc.md) for design rule checking
- See [CLI Reference](cli/reference.md) for all command-line options
