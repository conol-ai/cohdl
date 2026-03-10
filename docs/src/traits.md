# Traits

Traits define electrical behavior contracts that devices can implement. They are the primary abstraction mechanism in cohdl — think of them as interfaces that specify what pins a component must have, what electrical specs it must declare, and what design rules apply.

## Basic syntax

```cohdl
trait TraitName {
    // body items: pins, spec, designator_prefix, rules
}
```

## Pins

A trait can declare required pins. Any device implementing the trait must provide these pins:

```cohdl
trait TwoTerminal {
    pins {
        A: Pin
        B: Pin
    }
}
```

Pin declarations in traits use typed syntax (`name: Pin`) to declare that the pin must exist, without assigning physical pin numbers. The implementing device provides the actual pin mappings.

## Trait inheritance

Traits can extend other traits using the `:` syntax:

```cohdl
trait Capacitor: TwoTerminal {
    // inherits A and B pins from TwoTerminal
    designator_prefix: "C"
    spec {
        capacitance: Farads
        voltage_rating: Voltage
    }
}
```

Multiple parent traits are separated with `+`:

```cohdl
trait PoweredCapacitor: TwoTerminal + Powered {
    // inherits pins from both traits
}
```

## Designator prefix

The `designator_prefix` field sets the reference designator prefix for components implementing this trait:

```cohdl
trait Capacitor: TwoTerminal {
    designator_prefix: "C"
}

trait Resistor: TwoTerminal {
    designator_prefix: "R"
}

trait Connector {
    designator_prefix: "J"
}
```

Common prefixes follow industry convention: `C` for capacitors, `R` for resistors, `L` for inductors, `U` for ICs, `J` for connectors, `D` for diodes, etc.

## Spec blocks

Spec blocks declare electrical parameters that implementing devices must provide values for:

```cohdl
trait Capacitor: TwoTerminal {
    spec {
        capacitance: Farads
        voltage_rating: Voltage
    }
}
```

The type after the colon (`Farads`, `Voltage`, `Ohms`, etc.) specifies the kind of value expected. These types enable the compiler to validate that values have the correct units.

## DRC rules

Traits can define design rule checks that run against every device implementing the trait:

```cohdl
trait Capacitor: TwoTerminal {
    spec {
        capacitance: Farads
        voltage_rating: Voltage
    }

    rule voltage_exceed(level: Error) {
        assert net_voltage(self.A, self.B) <= self.spec.voltage_rating
        message: "Voltage {net_voltage(self.A, self.B)}V exceeds rating {self.spec.voltage_rating}V"
    }
}
```

A rule block contains:
- **`level`**: either `Error` (fails the build) or `Warning` (reported but doesn't fail)
- **`assert`**: a boolean expression that must be true
- **`message`**: an interpolated string displayed when the assertion fails

See [DRC](drc.md) for more details on design rules.

## Complete example

```cohdl
trait TwoTerminal {
    pins {
        A: Pin
        B: Pin
    }
}

trait Capacitor: TwoTerminal {
    designator_prefix: "C"

    spec {
        capacitance: Farads
        voltage_rating: Voltage
    }

    rule voltage_exceed(level: Error) {
        assert net_voltage(self.A, self.B) <= self.spec.voltage_rating
        message: "Voltage exceeds rating"
    }

    rule voltage_derating(level: Warning) {
        assert net_voltage(self.A, self.B) <= self.spec.voltage_rating * 0.8
        message: "Voltage exceeds 80% derating"
    }
}
```
