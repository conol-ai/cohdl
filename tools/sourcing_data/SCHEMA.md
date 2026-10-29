# Supplier sourcing data schema

## English

These repository-owned files are written by the sourcing-data maintainers and
frame-verification tools. The library generators read both files before emitting
RFC-034 supplier bindings; `tests/library/supplier.rs` independently validates
both files against every checked library package. CI and generators must not
silently repair malformed rows.

### `lcsc_map.json`

The top level contains `schema_version: 1`, `supplier: "lcsc"`, and `rows`.
Every row has:

- `mfr`, `mpn`, `pn`: non-empty manufacturer, manufacturer part number, and
  LCSC part number strings. Together they identify one purchasing identity.
- `rule`: `exact`, `alias`, or `manual`.
- `category`: `basic`, `preferred`, or `extended`.
- `snapshot_sha256`: exactly 64 lowercase hexadecimal characters.
- `date`: a real calendar date in `YYYY-MM-DD` form.

A binding's `(mfr, mpn, pn)` must match a row. Across all library packages,
one `(supplier, pn)` must always identify the same `(mfr, mpn)`.

### `jlc_frames.json`

The top level contains `schema_version: 1` and `rows`. Every row has:

- `id`: a non-empty string unique within this file.
- `package`: the full package name, including any scope.
- `package_version`: the package's current `cohdl.toml` version. A row for any
  other version is stale and does not satisfy a binding.
- `footprint`: the footprint name within that package.
- `supplier` and `pn`: non-empty supplier and supplier part-number strings.
- `mapping`: the binding's required `dx_mm`, `dy_mm`, and `rotate_deg` numeric
  values. Omitted source `at`/`rotate` values mean zero. Rotation is an integer
  in the RFC-034 range `0..360` (0 inclusive, 360 exclusive).
- `measured`: an object with the same three numeric fields, or `null`.
- `exception`: a non-empty reason string exactly when `mapping` is non-zero;
  it must be `null` for an all-zero mapping.
- `evidence` and `evidence_ref`: exactly one must be non-null. `evidence` has
  `method` (`preview` or `product_page`), a real `YYYY-MM-DD` `date`, and a
  `record` path to an existing file inside the repository. `evidence_ref` has
  `row`, `reason` (`same_supplier_footprint` or `same_preview_board`), and a
  non-empty `note`. The referenced row must exist and carry its own evidence;
  chained references are invalid.

Each LCSC binding needs a frame row exactly matching `(package,
package_version, footprint, supplier, pn)` and its mapping. Validation covers
all rows, including rows not currently selected by a binding.

## 中文

这两个文件由 sourcing 数据维护者和封装验证工具写入；库生成器读取它们以
生成 RFC-034 supplier 绑定，`tests/library/supplier.rs` 则对全部已检查库包
独立复验。CI 与生成器不得静默修复格式错误的行。

`lcsc_map.json` 顶层为 `schema_version: 1`、`supplier: "lcsc"` 和 `rows`。
每行必须有非空的 `mfr`、`mpn`、`pn`；`rule` 为 `exact`、`alias`、`manual`
之一；`category` 为 `basic`、`preferred`、`extended` 之一；
`snapshot_sha256` 为 64 位小写十六进制；`date` 为真实的 `YYYY-MM-DD` 日期。
绑定的 `(mfr, mpn, pn)` 必须匹配一行，且全库同一 `(supplier, pn)` 只能对应
一个 `(mfr, mpn)`。

`jlc_frames.json` 顶层为 `schema_version: 1` 和 `rows`。每行包含全文件唯一
的非空 `id`、完整 `package`（含 scope）、当前清单版本 `package_version`、
包内 `footprint`、非空 `supplier` 与 `pn`。`mapping` 的 `dx_mm`、`dy_mm`、
`rotate_deg` 均为数值，分别对应绑定的 `at` 和 `rotate`（未写即零）；角度
为 RFC-034 的 `0..360` 整数。`measured` 为同结构对象或 `null`。非零映射
必须有非空 `exception` 理由，零映射的 `exception` 必须为 `null`。

`evidence` 与 `evidence_ref` 恰好一个非空。前者含 `method`
（`preview`/`product_page`）、真实日期和仓库内现存文件路径 `record`；后者
含目标 `row`、枚举 `reason`（`same_supplier_footprint`/
`same_preview_board`）及非空 `note`。目标行必须存在且自带证据，禁止链式
引用。每个 LCSC 绑定必须精确匹配当前包版本的 `(package, package_version,
footprint, supplier, pn)` 及映射；所有行（包括未使用行）都接受格式与证据
检查。
