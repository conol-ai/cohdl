# STM32 open pin data provenance

This reference covers the generated STM32 device declarations outside the C5
family.

- Publisher: STMicroelectronics
- Repository: <https://github.com/STMicroelectronics/STM32_open_pin_data>
- Release tag: `STM32CubeMX-DB.6.0.180`
- Commit: `7d1f1514ed5583ec5007ad91236b4e1d377295b1`
- Commit date: 2026-06-29
- Upstream identity: subset of STM32CubeMX 6.18.0
- Retrieved: 2026-08-27
- License: BSD-3-Clause; the notice is stored in
  `STM32_OPEN_PIN_DATA_LICENSE.txt`.
- Frozen normalized snapshot: `tools/stm32_data/pin_data.json` at repository
  root, SHA-256
  `c652aec5052601b99b9aacc066354a2f20450599e03d261e6fadebd95a48b682`.

ST describes this repository as the STM32CubeMX database subset needed for
STM32 MCU pin configuration. `tools/gen_stm32.py` reads every non-STM32MP MCU
descriptor, records its package and provided physical pin list, expands ST's
parenthesized product choices, and retains lowercase `x` wherever ST uses it
as an ordering-code wildcard.

The import preserves one CoHDL pin per physical package position. Repeated
supply names become one logical multi-pad pin. Oscillator and bracketed remap
annotations retain the primary GPIO name. Distinct names such as `PC2_C` stay
distinct. CubeMX `PINREMAP` records duplicate an existing physical position;
the primary entry is retained and its raw bracketed alias remains visible in
the source provenance. Same-role selectable names on very small packages use
an explicit `_OR_` composite name and a generated comment. Any position whose
aliases imply incompatible CoHDL roles or obligations is excluded fail-closed
and listed in `src/catalog_coverage.cohdl`. Every QFN/QFPN, `LQFP*-EP`, and
other model whose XML sets `HasPowerPad=true` is also excluded. The feed's
flag is not reliable: these package records enumerate only perimeter
positions even when the manufacturer datasheet requires an exposed ground
pad. Inventing its pad number would violate the complete-pin-map rule.

The upstream `RefName` is a package-specific product pattern, not necessarily
an exact purchasable MPN. This source also does not contain recommended land
geometry or enough product-specific electrical guidance to certify a part.
Accordingly the broad import emits devices, never guessed parts or footprints.
A separate checked `tools/stm32_data/parts.json` overlay may emit exact parts
only after a local manufacturer datasheet and a dependency-owned complete land
pattern close those gaps. Board authors must still consult the current
datasheet for power sequencing, decoupling, boot straps, and other product
requirements.

The pre-existing exact ST portfolio inventory in
`tools/stm32_data/order_codes.txt` is an additional repository-only offline
validation gate, not pinout authority. No redistribution license for that
webpage compilation is recorded, so its order-code rows and unmatched
identities are not copied wholesale into the publishable package sources.
The small audited overlay derives its exact codes from its recorded local
manufacturer ordering scheme and merely requires those codes to remain in the
validation inventory. Terminal `TR` is collapsed only for a proven packing
alternate; other suffix characters are never trimmed.
`src/catalog_coverage.cohdl` records aggregate validation/part counts and every
exclusion derived from the BSD-licensed pin data.

CubeMX's `Type` field is a configurability class rather than a full electrical
contract. The generator therefore applies a closed conservative policy:
recognized GPIO and USB data I/O is optional bidirectional; reviewed
oscillator, op-amp, antenna, and RF names receive explicit input/output roles;
reset is optional input; `BOOT0` is required input; no-connect names are
position-specific optional passive pins; and reviewed supply/ground names are
required power inputs. Regulator, capacitor, switch, feedback, and ambiguous
core-supply nodes are required passive pins. Special analog/RF nodes that do
not map safely onto CoHDL's digital role vocabulary are explicitly reviewed
as optional passive pins. Semantic control names are optional inputs
regardless of the CubeMX class used by a particular family. Every snapshot
row is audited, including incomplete packages; an unknown `I/O`, `MonoIO`, or
`Power` name stops generation instead of silently acquiring a role.

Regenerate deterministically from the frozen snapshot with:

```sh
python3 tools/gen_stm32.py --check
```

Refreshing the snapshot requires both pinned upstream checkouts and the
explicit `--import-sources` command documented by the generator.
