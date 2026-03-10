# Types

## Type aliases

Type aliases create shorthand names for device instantiations with specific generic arguments. They reduce repetition when the same device configuration is used frequently.

```cohdl
type BypassCap = MLCC<C: 100nF, V: 10V, pkg: C0402>
```

After this declaration, `BypassCap` can be used anywhere `MLCC<C: 100nF, V: 10V, pkg: C0402>` would be used:

```cohdl
design Board {
    inst c1: BypassCap
    inst c2: BypassCap
}
```

## Parameterized type aliases

Type aliases can have their own generic parameters, creating partially-applied device types:

```cohdl
type SmallCap<C: Farads> = MLCC<C: C, V: 10V, pkg: C0402>
```

This creates a type that fixes the voltage and package but leaves capacitance configurable:

```cohdl
inst c1: SmallCap<C: 100nF>
inst c2: SmallCap<C: 10nF>
```

## Value types

cohdl has a built-in type system for electrical values. These types are used in generic parameter declarations and spec blocks:

| Type      | Description                    | Example values     |
|-----------|--------------------------------|--------------------|
| `Farads`  | Capacitance                    | `100nF`, `10uF`    |
| `Voltage` | Voltage                        | `3.3V`, `5V`       |
| `Ohms`    | Resistance                     | `10k`, `22R`, `4.7k` |
| `Amps`    | Current                        | `100mA`, `2A`      |
| `Package` | Physical package identifier    | `C0402`, `LQFP48`  |
| `Net`     | A net reference                | --                 |
| `Pin`     | A pin type (in trait pin decls)| --                 |
| `Bool`    | Boolean                        | `true`, `false`    |
| `Integer` | Integer number                 | `48`, `100`        |
| `Float`   | Floating-point number          | `0.8`, `3.14`      |
| `String`  | String literal                 | `"hello"`          |

## Engineering notation

cohdl supports engineering notation for numeric values with SI-style suffixes:

| Suffix | Meaning                     | Example              |
|--------|-----------------------------|----------------------|
| `nF`   | Nanofarads                  | `100nF`              |
| `uF`   | Microfarads                 | `4.7uF`              |
| `V`    | Volts                       | `3.3V`, `16V`        |
| `k`    | Kilohms                     | `10k`, `4.7k`        |
| `R`    | Ohms (explicit)             | `22R`, `100R`        |
| `m`    | Milliamps / milliohms       | `100m`, `100mA`      |

These suffixes can be combined with integer or decimal bases: `100nF`, `4.7uF`, `3.3V`, `10k`.

## `impl` constraints

Generic parameters can be constrained to require trait implementations:

```cohdl
fn decoupling<P: impl Capacitor>(vdd: Net, gnd: Net, cap: P) {
    inst c: cap
    net vdd: c.A
    net gnd: c.B
}
```

The `impl Capacitor` constraint means `P` must be a device that implements the `Capacitor` trait. Multiple trait bounds are combined with `+`:

```cohdl
device MyPart<C: impl TraitA + TraitB> {
    // ...
}
```
