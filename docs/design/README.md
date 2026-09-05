# Design repository snapshot

Snapshot of the CoHDL Coherent Design Repository, extracted 2026-07-13 from
<https://conol.ai/share/note/od34sne4sa5ujuohyhldr21r>.

The conol.ai note tree is the **source of truth**; this snapshot exists so the
implementation can reference the design offline and so changes to the design
are visible in review. If a document here disagrees with the live note, the
live note wins — re-extract with `python3 tools/extract_design_repo.py docs/design` rather than hand-edit.

| File | Note |
|---|---|
| `00-root.md` | 🧭 CoHDL — Coherent Design Repository (root) |
| `01-product-constitution.md` | 📜 1. Product Constitution |
| `02-conceptual-model.md` | 🧩 2. Conceptual Model |
| `03-capability-architecture.md` | 🏛️ 3. Capability Architecture |
| `04-principle-constraint-mapping.md` | ⚖️ 4. Principle → Constraint Mapping |
| `05-coherence-matrix.md` | 🔗 5. Coherence Matrix |
| `06-rfc-process.md` | 📋 6. Feature Proposal Process (RFC) |
| `rfc-001-units-as-types.md` | 📐 RFC-001: Units-as-types |
| `rfc-002-pin-connection-obligation.md` | 🔌 RFC-002: Pin connection-obligation typing |
| `rfc-003-trait-satisfaction.md` | 🧬 RFC-003: Trait-satisfaction-at-impl-time checking |
| `rfc-004-drc-reclassification.md` | 🔍 RFC-004: DRC/type-system reclassification pass |
| `rfc-005-designator-allocation.md` | 🏷️ RFC-005: Collision-free designator allocation |
| `rfc-006-nested-fn-calls.md` | 🧵 RFC-006: Nested fn call semantics |
| `rfc-007-generics-over-specs.md` | 🧬 RFC-007: Generics-over-specs and generic trait bounds |
| `rfc-008-pattern-matching.md` | RFC-008: Exhaustive pattern-matching over structural variants |
| `rfc-009-fmt.md` | RFC-009: cohdl fmt canonical form |
| `rfc-010-check-json.md` | RFC-010: cohdl check --json schema |
| `rfc-011-error-registry.md` | RFC-011: Error-code registry (formal v2 baseline) |
| `rfc-012-intent-annotations.md` | RFC-012: #[intent(...)] annotations (pure metadata) |
| `rfc-013-layout-constraint.md` | RFC-013: Layout-constraint concept (the door) |
| `rfc-014-lsp.md` | RFC-014: Language Server Protocol support |
| `rfc-015-ipc2581.md` | RFC-015: IPC-2581 codegen backend (Quilter handoff) |
| `rfc-016-modules.md` | RFC-016: Module system (package::module::submodule::name) |
| `rfc-017-library-registry.md` | RFC-017: Library registry (cohdl source + docs + footprint symbols) |
| `rfc-018-footprint-format.md` | RFC-018: Footprint format — Cadence-style pad/footprint split |
| `rfc-019-vscode-extension.md` | RFC-019: VS Code extension for CoHDL |
| `gc-002-amended-layout-door.md` | GC-002 (amended): Admit layout constraints into the conceptual model |
| `07-decision-records.md` | 🗂️ 7. Decision Records |
| `08-evolution-governance.md` | 🧬 8. Evolution Governance & Design Regression |
| `09-mvp-definition.md` | 🎯 9. MVP Definition |
| `10-language-specification.md` | 📖 10. Language Specification |

Extraction note: markdown checkboxes from the live notes render here as `[ ]` /
`[x]` list items.
