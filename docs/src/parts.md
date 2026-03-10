# Parts

Parts bind device instantiations to real-world manufacturer part numbers. They form the **Approved Vendor List (AVL)** — a list of acceptable components for procurement.

## Basic syntax

```cohdl
part part_name: DeviceType<generics> {
    primary { mfr: "Manufacturer", mpn: "PartNumber" }
}
```

## Primary and alternate sources

Every part must have exactly one `primary` entry and may have any number of `alt` entries:

```cohdl
part mlcc_100nF: MLCC<C: 100nF, V: 10V, pkg: C0402> {
    primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }
    alt { mfr: "Murata", mpn: "GRM155R61C104KA88D" }
    alt { mfr: "Yageo", mpn: "CC0402KRX5R9BB104" }
}
```

- **`primary`** -- the preferred manufacturer and part number
- **`alt`** -- alternative sources that are electrically equivalent

The `mfr` (manufacturer) and `mpn` (manufacturer part number) fields are strings. These values flow through to the BOM output.

## AVL fields

Each AVL entry contains key-value pairs with string values:

```cohdl
primary {
    mfr: "Samsung"
    mpn: "CL05B104KO5NNNC"
}
```

The standard fields are `mfr` and `mpn`, but you can include additional fields as needed for your procurement process.

## Inline AVL on instances

Parts can also be specified inline on `inst` statements within a design, without creating a separate `part` declaration:

```cohdl
design Board {
    inst r_sense: Resistor<R: 100m, pkg: R2512> {
        primary { mfr: "Vishay", mpn: "WSL2512R1000FEA" }
        alt { mfr: "Bourns", mpn: "CSS2512FT0R100" }
    }
}
```

This is useful for one-off components that don't need to be reused across designs.

## BOM output

Part information flows into the generated BOMs:

- **Simple BOM** -- groups instances by MPN with quantities
- **AVL BOM** -- lists each instance with its primary and alternate MPNs

Instances without a bound part appear as `<UNSPECIFIED>` in the BOM. See [KiCad Backend](backends/kicad.md) for details on BOM formats.

## Complete example

```cohdl
// Device definition
device MLCC<C: Farads, V: Voltage = 10V, pkg: Package = C0402>: impl Capacitor {
    package: pkg
    pins { A: 1, B: 2 }
    spec { capacitance: C, voltage_rating: V }
}

// Part binding — reusable across designs
part mlcc_100nF: MLCC<C: 100nF, V: 10V> {
    primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }
    alt { mfr: "Murata", mpn: "GRM155R61C104KA88D" }
}

part mlcc_10uF: MLCC<C: 10uF, V: 6.3V, pkg: C0603> {
    primary { mfr: "Samsung", mpn: "CL10A106KP8NNNC" }
}
```
