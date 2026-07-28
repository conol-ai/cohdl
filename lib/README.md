# `lib/` — the libraries that ship with the compiler

Every subdirectory here is a package **family dir** (RFC-029): either a
package itself (`cohdl.toml` + `src/`) or a container of side-by-side
versions in arbitrarily-named subdirectories that each are one. Nothing is
privileged — `lib/std/` resolves by exactly the rule `lib/passives/` would.

A dependency name resolves through `<project>/deps/<name>`, then
`<lib>/<name>`, then the RFC-030 cache (`~/.cohdl/registry`). The compiler
finds this directory by walking up from its own executable (then the current
directory) for a `lib/` that offers at least one readable package.

The libraries here today: `std` (traits + demo-board devices) and `passive`
(chip resistors and MLCCs — generated, see `tools/gen_passive.py`).

To add an official library: create `lib/<name>/` with a `cohdl.toml` whose
`[package] name` is `<name>` and whose `[package] version` is an exact
`X.Y.Z`, put its sources under `src/`, and publish it with
`cohdl publish lib/<name>`. Projects pin it under `[dependencies]` like any
other package. The resolver needs no change.

Two rules the layout enforces by being what it is: a package's version comes
only from its manifest (a directory name that spells a version is
convention), and a released version's content is immutable — changing it
under the same version is a hard `E1103` against every `cohdl.lock` that
pinned it.
