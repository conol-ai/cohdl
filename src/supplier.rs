//! RFC-034 §3.2 supplier selection, shared by every supplier-aware emitter.
//!
//! Selection is deliberately per IR instance, never per plain-BOM group. It
//! reads (but never mutates) the part binding established by `bind_parts`.

use crate::ast::{AvlEntry, BottomRotation, Ident, PartDef, SupplierBinding};
use crate::diag::{Diagnostic, Diagnostics};
use crate::ir::DesignIr;
use crate::resolve::{short, World};
use crate::units::UnitValue;
use std::collections::{BTreeMap, BTreeSet};

/// Which entry of the instance's own part declaration was selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvlEntryIndex {
    Primary,
    /// Zero-based index into `PartDef::alts`.
    Alt(usize),
}

/// The selected entry's own footprint-frame mapping. Missing `at`/`rotate`
/// clauses have their RFC-defined identity values.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SupplierMapping {
    pub at: (i128, i128),
    pub rotate: u16,
}

/// Complete purchase identity and mapping for one selected instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedSupplierEntry {
    pub entry: AvlEntryIndex,
    pub mfr: String,
    pub mpn: String,
    pub part_number: String,
    pub mapping: SupplierMapping,
}

/// Selection for one real IR instance, selected or explicitly unselected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceSelection {
    pub instance: String,
    pub designator: String,
    pub part: String,
    /// The primary footprint used by all existing physical emitters.
    pub footprint: String,
    pub selected: Option<SelectedSupplierEntry>,
}

/// All instances for one requested supplier, keyed by full IR instance path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupplierSelection {
    pub supplier: String,
    pub instances: BTreeMap<String, InstanceSelection>,
}

impl SupplierSelection {
    pub fn get(&self, instance_path: &str) -> Option<&InstanceSelection> {
        self.instances.get(instance_path)
    }
}

/// Requested suppliers in first-occurrence order. No supplier files or output
/// wrappers are created here: cards 2b/2c/3c consume this shared selection.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SupplierSelections {
    pub selections: Vec<SupplierSelection>,
}

impl SupplierSelections {
    pub fn is_empty(&self) -> bool {
        self.selections.is_empty()
    }

    pub fn get(&self, supplier: &str) -> Option<&SupplierSelection> {
        self.selections
            .iter()
            .find(|selection| selection.supplier == supplier)
    }
}

/// E816: validate declarations after resolution has indexed the package set,
/// from `run_declaration_checks` beside the related `(supplier, PN)` checks.
pub fn check_declarations(world: &World, diags: &mut Diagnostics) {
    for (id, declaration) in &world.suppliers {
        let context = format!("supplier declaration `{id}`");
        if !valid_supplier_id(id) {
            diags.push(Diagnostic::error(
                "E816",
                declaration.supplier.span,
                format!("invalid supplier identifier in {context}: expected a lowercase ASCII letter followed by lowercase letters, digits, or `_`, at most 32 bytes"),
            ));
        }
        if matches!(id.as_str(), "mpn" | "mfr") {
            diags.push(Diagnostic::error(
                "E816",
                declaration.supplier.span,
                format!("invalid supplier identifier in {context}: reserved: its upper-case form collides with the MPN/MFR netlist field"),
            ));
        }
        for key in &declaration.other_keys {
            diags.push(Diagnostic::error(
                "E816",
                key.span,
                format!(
                    "unknown key `{}` in {context}; expected `bottom_rotation`",
                    key.name
                ),
            ));
        }
        for key in &declaration.repeated_keys {
            diags.push(Diagnostic::error(
                "E816",
                key.span,
                format!("duplicate key `{}` in {context}", key.name),
            ));
        }
        if let Some((value, _)) = &declaration.bottom_rotation {
            if !matches!(value.name.as_str(), "kicad" | "mirrored") {
                diags.push(Diagnostic::error(
                    "E816",
                    value.span,
                    format!(
                        "unknown bottom_rotation `{}` in {context}; expected `kicad` or `mirrored`",
                        value.name
                    ),
                ));
            }
        }
    }
}

pub fn bottom_rotation(world: &World, supplier: &str) -> BottomRotation {
    world
        .suppliers
        .get(supplier)
        .map_or(BottomRotation::Kicad, |decl| {
            match decl
                .bottom_rotation
                .as_ref()
                .map(|(value, _)| value.name.as_str())
            {
                Some("mirrored") => BottomRotation::Mirrored,
                _ => BottomRotation::Kicad,
            }
        })
}

/// E817 is emitted at the physical-output boundary, not during declaration
/// checking: `--logical-only` has no CPL whose convention needs confirmation.
pub fn report_undeclared(
    world: &World,
    design_name: &str,
    supplier: &str,
    diags: &mut Diagnostics,
) {
    let span = world.designs[design_name].name.span;
    if !world.suppliers.contains_key(supplier) {
        diags.push(Diagnostic::warning(
            "E817",
            span,
            format!(
                "supplier `{supplier}` has no declaration; its CPL uses bottom_rotation: kicad"
            ),
        ));
    }
}

/// Resolve a mapping after E813 has accepted the source. Invalid source is
/// gated before this function runs, so an unrepresentable clause is not used to
/// construct procurement output.
fn mapping(binding: &SupplierBinding) -> SupplierMapping {
    let coordinate = |value: &Option<UnitValue>| value.as_ref().map_or(0, |v| v.femto);
    let at = binding.at.as_ref().map_or((0, 0), |(x, y)| {
        (coordinate(&x.value), coordinate(&y.value))
    });
    let rotate = binding
        .rotate
        .as_ref()
        .and_then(|(text, _)| text.parse::<u16>().ok())
        .filter(|angle| *angle < 360)
        .unwrap_or(0);
    SupplierMapping { at, rotate }
}

fn field(entry: &AvlEntry, name: &str) -> String {
    entry
        .field(name)
        .map(|field| field.value.clone())
        .unwrap_or_default()
}

/// Return the canonical identity of a resolved footprint reference. Looking
/// it up in the world is essential: `Ident` equality also compares spans, and
/// a raw spelling comparison would miss aliases resolved to the same fq key.
fn resolved_footprint_identity<'a>(world: &'a World, reference: &Ident) -> Option<&'a str> {
    world
        .footprints
        .get_key_value(&reference.name)
        .map(|(identity, _)| identity.as_str())
}

/// RFC-034 §3.2 alt eligibility, shared by selection and its E814 explanation.
/// Missing primary and effective alt footprints compare equal, preserving the
/// language's `None == None` inheritance semantics. The primary itself is not
/// subject to this comparison: a requested primary binding is always chosen.
fn same_primary_footprint(world: &World, part: &PartDef, entry: &AvlEntry) -> bool {
    let primary = part
        .primary
        .footprint
        .as_ref()
        .and_then(|reference| resolved_footprint_identity(world, reference));
    entry
        .effective_footprint(&part.primary)
        .and_then(|reference| resolved_footprint_identity(world, reference))
        == primary
}

fn selected_entry<'a>(
    world: &World,
    part: &'a PartDef,
    supplier: &str,
) -> Option<(AvlEntryIndex, &'a AvlEntry, &'a SupplierBinding)> {
    if let Some(binding) = part
        .primary
        .suppliers
        .iter()
        .find(|binding| binding.supplier.name == supplier)
    {
        return Some((AvlEntryIndex::Primary, &part.primary, binding));
    }
    part.alts.iter().enumerate().find_map(|(index, alt)| {
        if !same_primary_footprint(world, part, alt) {
            return None;
        }
        alt.suppliers
            .iter()
            .find(|binding| binding.supplier.name == supplier)
            .map(|binding| (AvlEntryIndex::Alt(index), alt, binding))
    })
}

/// E813's declaration-level ID grammar, reused at the CLI boundary. The two
/// entry points must not disagree about which IDs can name a binding.
pub fn valid_supplier_id(id: &str) -> bool {
    let mut bytes = id.bytes();
    id.len() <= 32
        && bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

/// Deduplicate requested IDs without sorting away their first-occurrence order.
pub fn requested_suppliers(ids: &[String]) -> Vec<String> {
    let mut seen = BTreeSet::new();
    ids.iter()
        .filter(|id| seen.insert((*id).clone()))
        .cloned()
        .collect()
}

fn part_designators(ir: &DesignIr) -> BTreeMap<&str, Vec<String>> {
    let mut result: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for instance in ir.instances.values() {
        if let Some(part) = instance.part.as_deref() {
            result.entry(part).or_default().push(
                instance
                    .designator
                    .clone()
                    .unwrap_or_else(|| "?".to_string()),
            );
        }
    }
    for designators in result.values_mut() {
        designators.sort_by_key(|designator| crate::emit::designator_sort_key(designator));
    }
    result
}

fn report_unbound(
    world: &World,
    ir: &DesignIr,
    selections: &SupplierSelections,
    diags: &mut Diagnostics,
) {
    let instantiated = part_designators(ir);
    for selection in &selections.selections {
        let bound_ids: BTreeSet<&str> = instantiated
            .keys()
            .filter_map(|part_name| world.parts.get(*part_name))
            .flat_map(|part| {
                std::iter::once(&part.primary)
                    .chain(part.alts.iter())
                    .flat_map(|entry| entry.suppliers.iter())
            })
            .map(|binding| binding.supplier.name.as_str())
            .collect();
        if !bound_ids.contains(selection.supplier.as_str()) {
            let ids = if bound_ids.is_empty() {
                "none".to_string()
            } else {
                bound_ids.into_iter().collect::<Vec<_>>().join(", ")
            };
            let span = world.designs.get(&ir.name).map_or_else(
                || ir.instances.values().next().unwrap().span,
                |design| design.name.span,
            );
            diags.push(Diagnostic::warning(
                "E814",
                span,
                format!(
                    "supplier `{}` has no bindings on any part instantiated by this design; bound supplier IDs: {ids}",
                    selection.supplier
                ),
            ));
            continue;
        }

        for (part_name, designators) in &instantiated {
            if selection
                .instances
                .values()
                .any(|instance| instance.part == *part_name && instance.selected.is_some())
            {
                continue;
            }
            let part = &world.parts[*part_name];
            let mut message = format!(
                "part `{}` has no eligible `{}` binding for designators: {}",
                short(part_name),
                selection.supplier,
                designators.join(", ")
            );
            if part.alts.iter().any(|alt| {
                !same_primary_footprint(world, part, alt)
                    && alt
                        .suppliers
                        .iter()
                        .any(|binding| binding.supplier.name == selection.supplier)
            }) {
                message.push_str(
                    "; a binding whose effective footprint differs from the primary footprint is skipped",
                );
            }
            diags.push(Diagnostic::warning("E814", part.name.span, message));
        }
    }
}

/// Select all requested suppliers after successful `bind_parts`, and collect
/// E814 from precisely the instantiated part declarations.
pub fn select(
    world: &World,
    ir: &DesignIr,
    requested: &[String],
    diags: &mut Diagnostics,
) -> SupplierSelections {
    let requested = requested_suppliers(requested);
    let mut selections = SupplierSelections::default();
    for supplier in &requested {
        let mut instances = BTreeMap::new();
        for (path, instance) in &ir.instances {
            let Some(part_name) = instance.part.as_ref() else {
                continue;
            };
            let Some(part) = world.parts.get(part_name) else {
                continue;
            };
            let selected =
                selected_entry(world, part, supplier).map(|(entry_index, entry, binding)| {
                    SelectedSupplierEntry {
                        entry: entry_index,
                        mfr: field(entry, "mfr"),
                        mpn: field(entry, "mpn"),
                        part_number: binding.part_number.clone(),
                        mapping: mapping(binding),
                    }
                });
            instances.insert(
                path.clone(),
                InstanceSelection {
                    instance: path.clone(),
                    designator: instance
                        .designator
                        .clone()
                        .unwrap_or_else(|| "?".to_string()),
                    part: part_name.clone(),
                    footprint: part
                        .primary
                        .footprint
                        .as_ref()
                        .map(|footprint| footprint.name.clone())
                        .unwrap_or_default(),
                    selected,
                },
            );
        }
        selections.selections.push(SupplierSelection {
            supplier: supplier.clone(),
            instances,
        });
    }
    report_unbound(world, ir, &selections, diags);
    selections
}
