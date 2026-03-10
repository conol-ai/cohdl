# DRC (Design Rule Checking)

cohdl includes a design rule checking engine that catches electrical and structural errors before you open a layout tool. DRC runs automatically during `cohdl build` and `cohdl check`.

## Built-in rules

### Errors

| Rule ID | Name                 | Description |
|---------|----------------------|-------------|
| `E001`  | `voltage_exceed`     | A component's `voltage_rating` is less than the voltage on a connected net. |
| `E002`  | `polarity_mismatch`  | A polarized device has its anode (`A` pin) connected to a GND net. |
| `E003`  | `spec_not_satisfied` | A device is missing a required spec field declared by its trait. |
| `E004`  | `trait_not_impl`     | A generic argument doesn't implement a required trait. |
| `E005`  | `missing_spec_field` | A trait spec field is not provided in the device instantiation. |

### Warnings

| Rule ID | Name                 | Description |
|---------|----------------------|-------------|
| `W001`  | `unconnected_pin`    | A declared pin on an instance has no net connection. |
| `W002`  | `floating_net`       | A net exists but has no instance pins connected. |
| `W003`  | `single_driver`      | A net has only one instance pin connection (likely unfinished wiring). |
| `W004`  | `multi_driver`       | A net has multiple output-type pins connected (potential short). |

## Voltage inference

The DRC engine automatically infers net voltages from two sources:

1. **Instance annotations** -- if a connected instance has a `voltage` generic substitution, that value is used
2. **Net name parsing** -- names like `3V3`, `5V`, and `1V8` are parsed into voltage values:
   - `3V3` = 3.3V
   - `5V` = 5.0V
   - `1V8` = 1.8V

## GND net detection

Nets are identified as ground nets if their name:
- Equals `GND` or `VSS` (case-insensitive)
- Starts with `GND` (e.g., `GND_ANALOG`)

This affects rule E002 (polarity mismatch).

## User-defined rules

### Trait rules

Define DRC rules inside traits. These rules apply to every device implementing the trait:

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

    rule voltage_derating(level: Warning) {
        assert net_voltage(self.A, self.B) <= self.spec.voltage_rating * 0.8
        message: "Voltage exceeds 80% derating"
    }
}
```

### Device rules

Devices can define their own rules in addition to inherited trait rules. A device rule with the same name as a trait rule overrides it:

```cohdl
device HighVoltageMLCC<C: Farads, V: Voltage>: impl Capacitor {
    // Override the trait's voltage_derating rule with a stricter one
    rule voltage_derating(level: Warning) {
        assert net_voltage(self.A, self.B) <= self.spec.voltage_rating * 0.5
        message: "High-voltage cap: exceeds 50% derating"
    }
}
```

### Rule anatomy

A rule block has three parts:

```cohdl
rule rule_name(level: Error) {
    assert <boolean_expression>
    message: "<interpolated string>"
}
```

- **`rule_name`** -- identifier for the rule (used with `#[allow]`)
- **`level`** -- `Error` (fails the build) or `Warning` (reported only)
- **`assert`** -- expression that must evaluate to `true`
- **`message`** -- interpolated string shown on failure

### Expressions in rules

Rule assertions support:

- **Comparison operators**: `<=`, `>=`, `==`, `!=`
- **Arithmetic**: `+`, `-`, `*`, `/`
- **Unary**: `-`, `!`
- **Function calls**: `net_voltage(self.A, self.B)`
- **Spec access**: `self.spec.voltage_rating`
- **Dot paths**: `self.A`, `self.spec.capacitance`
- **Literals**: `0.8`, `100nF`, `true`

### Interpolated messages

Rule messages support `{expression}` interpolation:

```cohdl
message: "Voltage {net_voltage(self.A, self.B)}V exceeds rating {self.spec.voltage_rating}V"
```

## Suppressing diagnostics

Use the `#[allow]` attribute to suppress specific DRC warnings or errors on an instance:

```cohdl
design Board {
    #[allow(unconnected_pin)]
    inst debug_header: PinHeader_2x5
}
```

## Diagnostic output

DRC diagnostics are rendered with source context, similar to Rust compiler errors:

```
Error[E001]: Board::c_vbus
  --> main.cohdl:15:5
   |
15 |     inst c_vbus: MLCC<C: 100nF, V: 5V>
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   | voltage_rating 5V is less than net `12V` voltage 12V
```

Errors cause a non-zero exit code. Warnings are reported but do not fail the build.
