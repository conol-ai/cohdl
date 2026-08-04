# SF32 miniboard reference (fabrication disabled)

This example is intentionally reduced to an empty, checkable reference design.
It is **not fabrication-ready**.

The contrib audit quarantined the SF32LB52X, BHI260AP, and CH343P purchasable
bindings because their available manufacturer documents do not completely
specify safe PCB land patterns. The formerly selected RT9080 binding was also
incompatible with its source-backed pin variant; that library has since been
corrected, but this board has not been re-reviewed. The previous generated
BOM, netlist, footprints, design lock, and KiCad schematic were therefore
removed so stale manufacturing artifacts cannot be mistaken for approved
output.

The logical source may be rebuilt after all instantiated components have
audited part identities and manufacturer-backed footprints.
