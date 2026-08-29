# CoHDL language reference (for design generation)

CoHDL is a text language for PCB schematics. You write `.cohdl` source; the
compiler type-checks it and emits a KiCad netlist + BOM. This reference is
complete: do not assume any syntax that is not shown here.

## Unit types (closed set of ten — zero coercion)

| Unit type | Symbol | Allowed SI prefixes | Example literals |
|---|---|---|---|
| `Voltage` | `V` | p n u m k M G | `3.3V`, `5V` |
| `Capacitance` | `F` | p n u | `100nF`, `10uF` |
| `Resistance` | `ohm` (ASCII — never `Ω`) | p n u m k M G | `10kohm`, `330ohm` |
| `Current` | `A` | p n u m k M G | `500mA`, `2A` |
| `Frequency` | `Hz` | k M G | `16MHz`, `32kHz` |
| `Time` | `s` | p n u m | `10ms`, `1us` |
| `Inductance` | `H` | p n u m | `10uH`, `100nH` |
| `Power` | `W` | u m k | `250mW`, `1W` |
| `Temperature` | `C` (ASCII — never `°C`) | none | `85C`, `-40C` |
| `Tolerance` | `%` | none | `1%`, `0.5%` |

- A literal is a number immediately followed by its suffix, no space: `100nF`.
- A bare number where a unit is expected is a compile error. No defaults, no
  coercion between unit types. Only `Temperature` may be negative.

## Devices

```cohdl
pub device ChipResistor<R: Resistance, T: Tolerance = 1%> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { resistance: R, tolerance: T }
}
```

- `pins { ... }`: logical pin name → physical pin number(s). A pin may carry
  several numbers (`required GND: 1, 40, 41` — a pin bus). Obligation keyword
  `required` (default when omitted) or `optional`. **Every pin must carry a
  role annotation** in brackets — one of `[input]`, `[output]`,
  `[bidirectional]`, `[passive]`, `[power_in]`, `[power_out]`. There is no
  default; an unannotated pin is a compile error.
- `spec { ... }`: field → unit literal or one of the device's own generic
  parameters.

### Package variants

A device may declare a closed set of package variants; each variant needs its
own pin layout, and every instantiation must select one:

```cohdl
pub device ExampleMLCC<C: Capacitance, V: Voltage = 16V, T: Tolerance = 10%> {
    variants { C0402, C0603 }
    pins[C0402] { A: 1 [passive], B: 2 [passive] }
    pins[C0603] { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C, voltage_rating: V, tolerance: T }
}

inst c1: ExampleMLCC<100nF, 16V, 10%>[C0402]   // [VARIANT] required — no default
```

`spec[VARIANT] { ... }` optionally overrides/extends the base spec per
variant. Parts select their variant in the part declaration; instantiating a
part by name needs no selector.

## Traits and impls

```cohdl
pub trait TwoTerminal {
    pins { required A: pin, required B: pin }
}
pub trait Capacitor: TwoTerminal {
    designator_prefix: "C"
    spec { capacitance: Capacitance, voltage_rating: Voltage, tolerance: Tolerance }
}

impl TwoTerminal for ExampleMLCC {}     // names match — empty body
impl Capacitor for ExampleMLCC {}

impl TwoTerminal for TantalumCap {
    pins { A: Anode, B: Cathode }   // explicit mapping when names differ
}
```

- Devices never declare traits inline; `impl Trait for Device` is a
  free-standing statement, checked exhaustively where it is written.
- A sub-trait bound (`Capacitor: TwoTerminal`) requires a separate satisfying
  `impl TwoTerminal for Device` to exist.

## Parts (purchasable components)

```cohdl
pub part Example100n: ExampleMLCC<100nF, 16V, 10%>[C0402] {
    primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC", footprint: passive::CHIP_0402 }
    alt { mfr: "Murata", mpn: "GRM155R71C104KA88D" }
}
```

- `footprint:` references a `footprint` SYMBOL (RFC-017) — a declared
  `pub footprint FP_X {}`, reached by bare name, `use` import, or a
  qualified path. It is never a string. Footprints live with the component
  packages that own the corresponding parts; std declares none.

- A part whose device declares `variants {}` selects its variant with the
  `[VARIANT]` suffix on the device reference, as above.

- A part binds a fully-concrete device to real MPNs. **Every instance in a
  buildable design must resolve to a part.** Instantiate a qualified part from
  a declared dependency, for example
  `inst c1: passive::C_100n_16V_X7R_0402`. Std supplies traits, not parts.

## Sub-circuit fns

```cohdl
fn decoupling_cap(pin: Pin, gnd: Pin) {
    inst c: passive::C_100n_16V_X7R_0402
    net _: pin, c.A
    net _: gnd, c.B
}

design Board {
    inst mcu: espressif_esp32::modules::wroom_s3::ESP32_S3_WROOM_1_N8
    decoupling_cap(mcu.VDD, mcu.GND)
}
```

- Parameters: `name: Pin` (a pin reference), `name: D` where `D` is a
  trait-bound generic parameter (`fn f<D: Capacitor>(target: D, ...)`), or
  `name: impl Trait`.
- Generic calls use turbofish for generic arguments: `f::<3.3V>(...)`.
  Arguments are pin references (`mcu.VDD`) or instance names.
- Nested calls (a fn calling another fn) are fully supported. Cyclic calls are
  a compile error.

## Designs

```cohdl
design SensorNode {
    inst usb: usb::connectors::type_c::USB_C_HRO_TYPE_C_31_M_12
    inst r_cc1: passive::R_5K1_F_0402

    net VBUS [5V]: usb.VBUS                // [5V] = net voltage annotation
    net GND [gnd]: usb.GND, usb.SHIELD, r_cc1.B
    net CC1: usb.CC1, r_cc1.A              // plain net

    nc: usb.CC2, usb.DP, usb.DN, usb.SBU1, usb.SBU2
}
```

- `inst name: PartOrDevice<args>` declares an instance.
- `net NAME [annotation]: member, member, …` connects pins. Members are
  `instance.PIN` references. The same pin in two nets merges them.
- Every `required` pin of every instance must appear in some `net` or in an
  `nc` declaration — leaving one unresolved is a compile error. A pin in both
  `net` and `nc` is a compile error.
- Optional pins may simply be left unmentioned.
- `#[designator("U7")]` on the line before an `inst` forces its designator.
- Annotate power nets with their voltage (`[5V]`, `[3.3V]`) and ground with
  `[gnd]` — the DRC checks voltage ratings and polarity against them.

## Residual DRC (runs after type checking)

- D001 voltage-exceed: instance `voltage_rating` < annotated net voltage.
- D002 polarity-mismatch: a `Polarized` device's anode on a `[gnd]` net.
- D003 single-driver (warning): a net whose only connected pin is a driver
  (`output`/`power_out`) — the driver drives nothing.
- D004 multi-driver: two or more `output`/`power_out` pins on one net.

## Rationale metadata (`#[intent]`)

`#[intent("why this choice")]` on the line before any declaration or
statement (except a `layout {}` block, which takes no attributes) attaches a
rationale string. It is exactly one string, at most one
per declaration, and is NEVER checked or compiled — it cannot change the
verdict, diagnostics, or netlist. Use a real `net`/`spec`/trait mechanism for
anything that must be enforced.

## Layout constraints (`layout {}`, `#[placement_hint]`)

```cohdl
design Board {
    net USB_DP: usb.DP, mcu.USB_DP
    net USB_DM: usb.DN, mcu.USB_DM
    layout {
        net_class HighSpeed { USB_DP, USB_DM }
        diff_pair(USB_DP, USB_DM)
        length_match(USB_DP, USB_DM) [tolerance: "0.15mm"]
    }
    #[placement_hint("board edge, near the connector")]
    inst usb: usb::connectors::type_c::USB_C_HRO_TYPE_C_31_M_12
}
```

- Four constraint kinds only: `net_class NAME { nets }`, `diff_pair(a, b)`
  (exactly two distinct nets), `length_match(nets…)` (two or more distinct
  nets, optional `[tolerance: 1ms]` unit literal or `[tolerance: "0.15mm"]`
  string), and inst-only `#[placement_hint("...")]`.
- Net references must name declared nets (checked); constraints never affect
  the netlist/BOM — they are emitted to a separate `<name>-layout.json`.
- CoHDL never enforces a tolerance; it is pass-through data for a layout tool.

## Comments

`// line comments only`

## Sensor-node libraries and qualified paths

Std is the implicit prelude and contains only these universal traits:
`TwoTerminal`, `Capacitor` (prefix C), `Resistor` (prefix R), `Polarized`
(pins Anode/Cathode), `Diode` (prefix D), `IC` (prefix U), and `Connector`
(prefix J). Std contains no devices, parts, pads, footprints, or helper
circuits.

Every other package must be pinned explicitly in the project manifest:

| Manifest dependency | CoHDL namespace | Representative declaration |
|---|---|---|
| `std = "0.3.0"` | implicit/unqualified | `IC` |
| `passive = "0.2.0"` | `passive` | `passive::MLCC` |
| `usb = "0.1.0"` | `usb` | `usb::connectors::type_c::USB_C_HRO_TYPE_C_31_M_12` |
| `esd = "0.1.0"` | `esd` | `esd::ESD_USBLC6` |
| `ldo = "0.1.0"` | `ldo` | `ldo::LDO_AP2112K_3V3` |
| `led = "0.1.0"` | `led` | `led::LED_RED_0603` |
| `mic = "0.1.0"` | `mic` | `mic::MIC_ICS43434` |
| `"@espressif/esp32" = "0.2.0"` | `espressif_esp32` | `espressif_esp32::modules::wroom_s3::ESP32_S3_WROOM_1_N8` |

Devices used by the sensor-node prompt:

| Device | Pins (name: number [role]) |
|---|---|
| `passive::MLCC<C: Capacitance, V: Voltage = 16V, T: Tolerance = 10%>` — variants `C0201`, `C0402`, `C0603`, `C0805`, `C1206`, `C1210` | A: 1 [passive], B: 2 [passive] |
| `passive::ChipResistor<R: Resistance, T: Tolerance = 1%>` — variants `R0201`, `R0402`, `R0603`, `R0805`, `R1206`, `R1210`, `R2010`, `R2512` | A: 1 [passive], B: 2 [passive] |
| `led::ChipLED` | Cathode: 1 [passive], Anode: 2 [passive] (implements Polarized, Diode) |
| `espressif_esp32::modules::wroom_s3::ESP32_S3_WROOM_1` | required GND: 1,40,41 [power_in]; required VDD: 2 [power_in]; required EN: 3 [input]; optional IO0–IO48 [bidirectional] (IO19 = USB D-, IO20 = USB D+); optional TXD0: 36 [output]; optional RXD0: 37 [input] |
| `ldo::AP2112K_3V3` (3.3V LDO, 600mA) | required VIN: 1 [power_in]; required GND: 2 [power_in]; required EN: 3 [input]; optional NC: 4 [passive]; required VOUT: 5 [power_out]; spec voltage_rating: 6V |
| `usb::connectors::type_c::USB_C_Receptacle_2_0` | required GND: A1,A12,B1,B12 [power_in]; required VBUS: A4,A9,B4,B9 [power_out]; required CC1: A5 [passive]; required CC2: B5 [passive]; required DP: A6,B6 [bidirectional]; required DN: A7,B7 [bidirectional]; optional SBU1: A8 [passive]; optional SBU2: B8 [passive]; required SHIELD: SH1,SH2,SH3,SH4 [passive] |
| `esd::USBLC6` | required IO1_A: 1 [bidirectional]; required GND: 2 [power_in]; required IO2_A: 3 [bidirectional]; required IO2_B: 4 [bidirectional]; required VBUS: 5 [power_in]; required IO1_B: 6 [bidirectional] |
| `mic::ICS43434` (I2S MEMS mic) | required WS: 1 [input]; required LR: 2 [input]; required SCK: 3 [input]; required SD: 4 [output]; required VDD: 5 [power_in]; required GND: 6 [power_in] |

Concrete parts for this prompt (instantiate these qualified names so the BOM
resolves):

- `passive::C_100n_16V_X7R_0402` — 100nF decoupling cap
- `passive::C_1u_10V_X5R_0402` — 1uF cap (LDO in/out)
- `passive::C_10u_10V_X5R_0603` — 10uF bulk cap
- `passive::R_5K1_F_0402` — 5.1kohm (USB-C CC pulldowns)
- `passive::R_10K_F_0402` — 10kohm (EN pull-up)
- `passive::R_1K_F_0402` — 1kohm (LED series)
- `led::LED_RED_0603` — red status LED
- `espressif_esp32::modules::wroom_s3::ESP32_S3_WROOM_1_N8` — ESP32-S3 module
- `ldo::LDO_AP2112K_3V3` — 3.3V LDO
- `usb::connectors::type_c::USB_C_HRO_TYPE_C_31_M_12` — USB-C receptacle
- `esd::ESD_USBLC6` — USB ESD protection
- `mic::MIC_ICS43434` — I2S microphone

Passive helper circuits are `passive::decoupling_100n(vdd: Pin, gnd: Pin)` and
`passive::bulk_10u(vdd: Pin, gnd: Pin)`. There is no `status_led` helper:
instantiate `passive::R_1K_F_0402` and `led::LED_RED_0603`, then wire
signal → resistor → LED anode and LED cathode → GND.

## Electrical design notes

- USB-C as a power sink needs 5.1kohm pulldowns from CC1 and CC2 to GND
  (separate resistors).
- The ESP32-S3-WROOM-1 EN pin needs a pull-up to 3.3V (10kohm).
- The AP2112K EN pin can be tied directly to VIN.
- The ICS43434 LR pin selects the I2S channel — tie it to GND or VDD.
- Give every IC its decoupling (100nF close to each supply pin, plus bulk
  capacitance on the rail).
