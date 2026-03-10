# KiCad Backend

cohdl compiles designs to KiCad-compatible output files for layout, manufacturing, and procurement.

## Output files

The `cohdl build` command generates three output files:

### Netlist (`.net`)

A KiCad legacy netlist in XML format (compatible with KiCad 5, 6, and 7). Contains:

- **Components** -- each instance with its reference designator, value, footprint, and MPN
- **Nets** -- named connections between component pins

Import into KiCad via **File > Import Netlist** in the PCB editor.

### Simple BOM (`-bom.csv`)

A CSV file with columns `RefDes,MPN,Qty`:

```csv
RefDes,MPN,Qty
"C1,C2","CL05B104KO5NNNC",2
"R1","RC0402FR-0710KL",1
"U1","<UNSPECIFIED>",1
```

Instances sharing the same MPN are grouped into a single row with combined reference designators and a quantity count. Instances without a bound `part` appear with MPN `<UNSPECIFIED>`.

### AVL BOM (`-bom-avl.csv`)

A CSV file with columns `RefDes,Primary MPN,Alt 1,Alt 2,...`:

```csv
RefDes,Primary MPN,Alt 1
"C1","CL05B104KO5NNNC","GRM155R71C104KA88D"
"R1","RC0402FR-0710KL",""
```

Each instance gets its own row. The number of `Alt N` columns equals the maximum number of alternate MPNs across all instances. Rows are sorted by reference designator.

## Footprint mapping

cohdl maps package names to KiCad footprint library identifiers. The default mappings are:

| Package name | KiCad footprint                        |
|-------------|----------------------------------------|
| `C0402`     | `Capacitor_SMD:C_0402_1005Metric`      |
| `C0603`     | `Capacitor_SMD:C_0603_1608Metric`      |
| `R0402`     | `Resistor_SMD:R_0402_1005Metric`       |
| `R0603`     | `Resistor_SMD:R_0603_1608Metric`       |
| `LQFP48`    | `Package_QFP:LQFP-48_7x7mm_P0.5mm`    |
| `LQFP64`    | `Package_QFP:LQFP-64_10x10mm_P0.5mm`  |
| `SOT-23`    | `Package_TO_SOT_SMD:SOT-23`           |

Devices with unrecognized package names get a fallback footprint of `Unknown:PackageName`.

## Controlling output

Use the `--emit` flag to select which files to generate:

```bash
# Generate only the netlist
cohdl build --emit netlist

# Generate only BOMs
cohdl build --emit bom-simple,bom-avl

# Generate everything (default)
cohdl build --emit all
```

Use `--out-dir` to change the output directory (default: `out`):

```bash
cohdl build --out-dir build/output
```

## Netlist structure

The generated `.net` file follows the KiCad legacy netlist format:

```xml
(export (version D)
  (components
    (comp (ref "U1") (value "STM32F103")
      (footprint "Package_QFP:LQFP-48_7x7mm_P0.5mm")
      (fields (field (name "MPN") "STM32F103C8T6"))
    )
    (comp (ref "C1") (value "100nF")
      (footprint "Capacitor_SMD:C_0402_1005Metric")
    )
  )
  (nets
    (net (code 1) (name "VDD")
      (node (ref "U1") (pin "VDD_IO"))
      (node (ref "C1") (pin "A"))
    )
    (net (code 2) (name "GND")
      (node (ref "C1") (pin "B"))
    )
  )
)
```

Each component includes:
- `ref` -- the reference designator
- `value` -- the component value (from `value` generic substitution, or the device name)
- `footprint` -- the KiCad footprint identifier
- `fields` -- optional MPN field if a part is bound

## Importing into KiCad

1. Open your KiCad project
2. Open the **PCB Editor** (Pcbnew)
3. Go to **File > Import Netlist**
4. Browse to the generated `.net` file
5. Click **Update PCB**
6. Place the components and route traces

The netlist can be re-imported after changes to update component placement and net connections.
