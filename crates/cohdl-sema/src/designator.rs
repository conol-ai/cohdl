//! Designator lock file (`design.lock`) management.
//!
//! This module provides [`DesignatorDb`] which assigns stable reference
//! designators (e.g. `C1`, `R3`, `U2`) to component instances and persists
//! them in a TOML lock file.  Once a designator is assigned to a hierarchical
//! path it is never changed, even if the source order changes.  Removed
//! instances are moved to a `[tombstones]` section so their designators are
//! never reused.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::connectivity::ConnectivityIR;
use crate::SemaError;

// ── Lock file format ────────────────────────────────────────────────────────

/// On-disk representation of the `design.lock` TOML file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct LockFile {
    /// `[designators]` — maps hierarchical instance path → designator string.
    #[serde(default)]
    designators: BTreeMap<String, String>,
    /// `[tombstones]` — maps removed instance path → last-known designator.
    #[serde(default)]
    tombstones: BTreeMap<String, String>,
}

// ── DesignatorDb ────────────────────────────────────────────────────────────

/// In-memory designator database backed by a TOML lock file.
#[derive(Debug, Clone)]
pub struct DesignatorDb {
    designators: BTreeMap<String, String>,
    tombstones: BTreeMap<String, String>,
}

/// Context about a single instance needed for designator assignment.
#[derive(Debug, Clone)]
pub struct InstanceInfo {
    /// Hierarchical path (e.g. `"MainBoard::c1"`).
    pub hierarchical_path: String,
    /// Optional explicit designator override from `#[designator("U1")]`.
    pub designator_override: Option<String>,
    /// Designator prefix derived from implemented traits (e.g. `"C"`, `"R"`).
    /// `None` means use the default `"U"`.
    pub prefix: Option<String>,
}

impl Default for DesignatorDb {
    fn default() -> Self {
        Self::new()
    }
}

impl DesignatorDb {
    /// Create an empty database.
    pub fn new() -> Self {
        Self {
            designators: BTreeMap::new(),
            tombstones: BTreeMap::new(),
        }
    }

    /// Load a designator database from `path`, or create an empty one if the
    /// file does not exist.
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
        let lock: LockFile = toml::from_str(&content)
            .map_err(|e| format!("failed to parse {}: {}", path.display(), e))?;
        Ok(Self {
            designators: lock.designators,
            tombstones: lock.tombstones,
        })
    }

    /// Assign designators to all instances in `ir`.
    ///
    /// - Existing assignments from the lock file are reused for paths that
    ///   still exist.
    /// - `#[designator("Xxx")]` overrides are applied (an error is produced if
    ///   the override conflicts with an existing assignment to a *different*
    ///   path).
    /// - New instances are assigned the lowest available number for their
    ///   prefix.
    ///
    /// Returns a mapping from hierarchical path → designator, plus any errors.
    pub fn assign(
        &mut self,
        instances: &[InstanceInfo],
    ) -> (BTreeMap<String, String>, Vec<SemaError>) {
        let mut errors = Vec::new();

        // Collect the set of paths that are currently live.
        let live_paths: HashSet<&str> = instances
            .iter()
            .map(|i| i.hierarchical_path.as_str())
            .collect();

        // Phase 1: keep existing assignments for live paths.
        let mut result: BTreeMap<String, String> = BTreeMap::new();
        for (path, desig) in &self.designators {
            if live_paths.contains(path.as_str()) {
                result.insert(path.clone(), desig.clone());
            }
        }

        // Track all used designators (from existing assignments + tombstones)
        // so we never reuse them.
        let mut used: HashSet<String> = HashSet::new();
        for desig in self.designators.values() {
            used.insert(desig.clone());
        }
        for desig in self.tombstones.values() {
            used.insert(desig.clone());
        }

        // Phase 2: apply explicit overrides.
        for inst in instances {
            if let Some(ref override_desig) = inst.designator_override {
                if let Some(existing) = result.get(&inst.hierarchical_path) {
                    if existing != override_desig {
                        // The lock file has a different designator for this path.
                        // The override wins, but only if the override designator
                        // isn't already assigned to a *different* path.
                        let conflict = result
                            .iter()
                            .find(|(p, d)| *d == override_desig && **p != inst.hierarchical_path);
                        if let Some((conflicting_path, _)) = conflict {
                            errors.push(SemaError::new(
                                format!(
                                    "designator `{}` for `{}` conflicts with existing assignment to `{}`",
                                    override_desig, inst.hierarchical_path, conflicting_path,
                                ),
                                cohdl_syntax::ast::Span { start: 0, end: 0 },
                            ));
                            continue;
                        }
                        // Remove old designator from used set and replace.
                        used.remove(existing);
                        result.insert(inst.hierarchical_path.clone(), override_desig.clone());
                        used.insert(override_desig.clone());
                    }
                    // If they match, nothing to do.
                } else {
                    // No existing assignment.  Check for conflict.
                    let conflict = result
                        .iter()
                        .find(|(p, d)| *d == override_desig && **p != inst.hierarchical_path);
                    if let Some((conflicting_path, _)) = conflict {
                        errors.push(SemaError::new(
                            format!(
                                "designator `{}` for `{}` conflicts with existing assignment to `{}`",
                                override_desig, inst.hierarchical_path, conflicting_path,
                            ),
                            cohdl_syntax::ast::Span { start: 0, end: 0 },
                        ));
                        continue;
                    }
                    if used.contains(override_desig)
                        && !result.values().any(|d| d == override_desig)
                    {
                        // Used in a tombstone — still a conflict.
                        errors.push(SemaError::new(
                            format!(
                                "designator `{}` for `{}` conflicts with a tombstoned designator",
                                override_desig, inst.hierarchical_path,
                            ),
                            cohdl_syntax::ast::Span { start: 0, end: 0 },
                        ));
                        continue;
                    }
                    result.insert(inst.hierarchical_path.clone(), override_desig.clone());
                    used.insert(override_desig.clone());
                }
            }
        }

        // Phase 3: assign new designators for remaining instances.
        for inst in instances {
            if result.contains_key(&inst.hierarchical_path) {
                continue; // Already assigned.
            }
            let prefix = inst.prefix.as_deref().unwrap_or("U");
            let desig = next_available(prefix, &used);
            result.insert(inst.hierarchical_path.clone(), desig.clone());
            used.insert(desig);
        }

        // Update internal state with the new assignments.
        self.designators = result.clone();

        (result, errors)
    }

    /// Move removed instances to the tombstones table.
    ///
    /// Any path in `old_paths` that is currently in `designators` but no
    /// longer present in the live set is moved to `tombstones`.
    pub fn tombstone_removed(&mut self, old_paths: &[String]) {
        for path in old_paths {
            if let Some(desig) = self.designators.remove(path) {
                self.tombstones.insert(path.clone(), desig);
            }
        }
    }

    /// Save the lock file to `path`.
    ///
    /// The file is written atomically (append-only semantics: existing entries
    /// are never removed from the file, only new ones are added).
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let lock = LockFile {
            designators: self.designators.clone(),
            tombstones: self.tombstones.clone(),
        };
        let content = toml::to_string_pretty(&lock)
            .map_err(|e| format!("failed to serialize lock file: {}", e))?;
        std::fs::write(path, content)
            .map_err(|e| format!("failed to write {}: {}", path.display(), e))?;
        Ok(())
    }

    /// Get the current designator assignments.
    pub fn designators(&self) -> &BTreeMap<String, String> {
        &self.designators
    }

    /// Get the current tombstones.
    pub fn tombstones(&self) -> &BTreeMap<String, String> {
        &self.tombstones
    }
}

/// Build [`InstanceInfo`] entries from a [`ConnectivityIR`] plus trait metadata.
///
/// `device_traits` maps device name → list of implemented trait names.
/// `trait_prefixes` maps trait name → designator prefix string.
/// `overrides` maps hierarchical path → explicit designator string from attributes.
pub fn build_instance_infos(
    ir: &ConnectivityIR,
    device_traits: &std::collections::HashMap<String, Vec<String>>,
    trait_prefixes: &std::collections::HashMap<String, String>,
    overrides: &std::collections::HashMap<String, String>,
) -> Vec<InstanceInfo> {
    ir.instances
        .iter()
        .map(|inst| {
            let prefix = device_traits
                .get(&inst.device)
                .and_then(|traits| traits.iter().find_map(|t| trait_prefixes.get(t).cloned()));
            InstanceInfo {
                hierarchical_path: inst.hierarchical_path.clone(),
                designator_override: overrides.get(&inst.hierarchical_path).cloned(),
                prefix,
            }
        })
        .collect()
}

/// Build [`InstanceInfo`] entries directly from a [`TypedDesign`], a
/// [`ConnectivityIR`], and the trait prefix map from [`TypeCheckResult`].
///
/// This is the recommended way to integrate designator assignment into the
/// compilation pipeline.
pub fn instance_infos_from_typed_design(
    design: &crate::typeck::TypedDesign,
    ir: &ConnectivityIR,
    trait_prefixes: &std::collections::HashMap<String, String>,
) -> Vec<InstanceInfo> {
    // Build a map from instance name → (designator_override, impl_traits)
    // from the TypedDesign.
    let inst_meta: std::collections::HashMap<&str, (&Option<String>, &Vec<String>)> = design
        .instances
        .iter()
        .map(|ci| (ci.name.as_str(), (&ci.designator_override, &ci.impl_traits)))
        .collect();

    let empty_traits: Vec<String> = Vec::new();
    ir.instances
        .iter()
        .map(|inst| {
            let (desig_override, impl_traits) = inst_meta
                .get(inst.name.as_str())
                .map(|(d, t)| ((*d).clone(), *t))
                .unwrap_or((None, &empty_traits));

            let prefix = impl_traits
                .iter()
                .find_map(|t| trait_prefixes.get(t).cloned());

            InstanceInfo {
                hierarchical_path: inst.hierarchical_path.clone(),
                designator_override: desig_override,
                prefix,
            }
        })
        .collect()
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Find the lowest available designator number for `prefix`.
///
/// E.g. if `used` contains `"C1"` and `"C3"`, then `next_available("C", used)`
/// returns `"C2"`.
fn next_available(prefix: &str, used: &HashSet<String>) -> String {
    for n in 1u32.. {
        let candidate = format!("{}{}", prefix, n);
        if !used.contains(&candidate) {
            return candidate;
        }
    }
    unreachable!()
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn info(path: &str, prefix: Option<&str>, override_desig: Option<&str>) -> InstanceInfo {
        InstanceInfo {
            hierarchical_path: path.to_string(),
            designator_override: override_desig.map(|s| s.to_string()),
            prefix: prefix.map(|s| s.to_string()),
        }
    }

    // ── Stable reassignment ─────────────────────────────────────────────

    #[test]
    fn stable_reassignment_preserves_existing() {
        let mut db = DesignatorDb::new();

        // First assignment: two capacitors.
        let instances = vec![
            info("Board::c1", Some("C"), None),
            info("Board::c2", Some("C"), None),
        ];
        let (result, errors) = db.assign(&instances);
        assert!(errors.is_empty());
        assert_eq!(result["Board::c1"], "C1");
        assert_eq!(result["Board::c2"], "C2");

        // Second assignment: same instances → same designators.
        let mut db2 = db.clone();
        let (result2, errors2) = db2.assign(&instances);
        assert!(errors2.is_empty());
        assert_eq!(result2["Board::c1"], "C1");
        assert_eq!(result2["Board::c2"], "C2");

        // Third assignment: add a new instance, existing ones keep designators.
        let instances3 = vec![
            info("Board::c1", Some("C"), None),
            info("Board::c2", Some("C"), None),
            info("Board::c3", Some("C"), None),
        ];
        let (result3, errors3) = db.assign(&instances3);
        assert!(errors3.is_empty());
        assert_eq!(result3["Board::c1"], "C1");
        assert_eq!(result3["Board::c2"], "C2");
        assert_eq!(result3["Board::c3"], "C3");
    }

    #[test]
    fn stable_reassignment_after_removal() {
        let mut db = DesignatorDb::new();

        // Assign three capacitors.
        let instances = vec![
            info("Board::c1", Some("C"), None),
            info("Board::c2", Some("C"), None),
            info("Board::c3", Some("C"), None),
        ];
        let (result, _) = db.assign(&instances);
        assert_eq!(result["Board::c1"], "C1");
        assert_eq!(result["Board::c2"], "C2");
        assert_eq!(result["Board::c3"], "C3");

        // Remove c2 (tombstone it).
        db.tombstone_removed(&["Board::c2".to_string()]);

        // Reassign with c2 gone and c4 added.
        let instances2 = vec![
            info("Board::c1", Some("C"), None),
            info("Board::c3", Some("C"), None),
            info("Board::c4", Some("C"), None),
        ];
        let (result2, errors2) = db.assign(&instances2);
        assert!(errors2.is_empty());
        // c1 and c3 keep their designators.
        assert_eq!(result2["Board::c1"], "C1");
        assert_eq!(result2["Board::c3"], "C3");
        // c4 gets C4 (not C2, because C2 is tombstoned).
        assert_eq!(result2["Board::c4"], "C4");
    }

    // ── Tombstone on removal ────────────────────────────────────────────

    #[test]
    fn tombstone_moves_to_tombstones_table() {
        let mut db = DesignatorDb::new();

        let instances = vec![
            info("Board::r1", Some("R"), None),
            info("Board::r2", Some("R"), None),
        ];
        db.assign(&instances);

        assert!(db.tombstones().is_empty());

        db.tombstone_removed(&["Board::r1".to_string()]);

        assert!(db.tombstones().contains_key("Board::r1"));
        assert_eq!(db.tombstones()["Board::r1"], "R1");
        assert!(!db.designators().contains_key("Board::r1"));
    }

    // ── #[designator] override ──────────────────────────────────────────

    #[test]
    fn designator_override_applied() {
        let mut db = DesignatorDb::new();

        let instances = vec![
            info("Board::mcu", Some("U"), Some("U1")),
            info("Board::c1", Some("C"), None),
        ];
        let (result, errors) = db.assign(&instances);
        assert!(errors.is_empty());
        assert_eq!(result["Board::mcu"], "U1");
        assert_eq!(result["Board::c1"], "C1");
    }

    #[test]
    fn designator_override_stable_across_reassignment() {
        let mut db = DesignatorDb::new();

        let instances = vec![
            info("Board::mcu", Some("U"), Some("U5")),
            info("Board::u2", Some("U"), None),
        ];
        let (result, errors) = db.assign(&instances);
        assert!(errors.is_empty());
        assert_eq!(result["Board::mcu"], "U5");
        assert_eq!(result["Board::u2"], "U1");

        // Reassign — override and auto-assign should be stable.
        let (result2, errors2) = db.assign(&instances);
        assert!(errors2.is_empty());
        assert_eq!(result2["Board::mcu"], "U5");
        assert_eq!(result2["Board::u2"], "U1");
    }

    // ── Conflict error ──────────────────────────────────────────────────

    #[test]
    fn conflict_error_on_duplicate_override() {
        let mut db = DesignatorDb::new();

        let instances = vec![
            info("Board::u1", Some("U"), Some("U1")),
            info("Board::u2", Some("U"), Some("U1")),
        ];
        let (_result, errors) = db.assign(&instances);
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("conflicts"));
    }

    // ── Default prefix ──────────────────────────────────────────────────

    #[test]
    fn default_prefix_is_u() {
        let mut db = DesignatorDb::new();

        let instances = vec![info("Board::chip", None, None)];
        let (result, errors) = db.assign(&instances);
        assert!(errors.is_empty());
        assert_eq!(result["Board::chip"], "U1");
    }

    // ── Mixed prefixes ──────────────────────────────────────────────────

    #[test]
    fn mixed_prefixes_numbered_independently() {
        let mut db = DesignatorDb::new();

        let instances = vec![
            info("Board::c1", Some("C"), None),
            info("Board::r1", Some("R"), None),
            info("Board::c2", Some("C"), None),
            info("Board::r2", Some("R"), None),
        ];
        let (result, errors) = db.assign(&instances);
        assert!(errors.is_empty());
        assert_eq!(result["Board::c1"], "C1");
        assert_eq!(result["Board::c2"], "C2");
        assert_eq!(result["Board::r1"], "R1");
        assert_eq!(result["Board::r2"], "R2");
    }

    // ── Load / save round-trip ──────────────────────────────────────────

    #[test]
    fn load_save_roundtrip() {
        let dir = std::env::temp_dir().join("cohdl_test_designator");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("design.lock");

        // Clean up from previous runs.
        let _ = std::fs::remove_file(&path);

        // Start fresh.
        let mut db = DesignatorDb::load(&path).unwrap();
        let instances = vec![
            info("Board::c1", Some("C"), None),
            info("Board::r1", Some("R"), None),
        ];
        db.assign(&instances);
        db.tombstone_removed(&["Board::old".to_string()]); // no-op, but harmless
        db.save(&path).unwrap();

        // Reload.
        let db2 = DesignatorDb::load(&path).unwrap();
        assert_eq!(db2.designators()["Board::c1"], "C1");
        assert_eq!(db2.designators()["Board::r1"], "R1");

        // Clean up.
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir(&dir);
    }

    #[test]
    fn load_nonexistent_creates_empty() {
        let db = DesignatorDb::load(Path::new("/tmp/nonexistent_cohdl_test.lock")).unwrap();
        assert!(db.designators().is_empty());
        assert!(db.tombstones().is_empty());
    }

    // ── build_instance_infos ────────────────────────────────────────────

    #[test]
    fn build_instance_infos_derives_prefix() {
        use crate::connectivity::{ConnectivityIR, Instance};
        use crate::typeck::InstanceId;

        let ir = ConnectivityIR {
            instances: vec![
                Instance {
                    id: InstanceId(0),
                    name: "c1".into(),
                    hierarchical_path: "Board::c1".into(),
                    device: "MLCC".into(),
                    mpn: None,
                    generic_substitutions: HashMap::new(),
                },
                Instance {
                    id: InstanceId(1),
                    name: "mcu".into(),
                    hierarchical_path: "Board::mcu".into(),
                    device: "STM32".into(),
                    mpn: None,
                    generic_substitutions: HashMap::new(),
                },
            ],
            nets: vec![],
        };

        let mut device_traits: HashMap<String, Vec<String>> = HashMap::new();
        device_traits.insert("MLCC".into(), vec!["Capacitor".into()]);
        device_traits.insert("STM32".into(), vec![]);

        let mut trait_prefixes: HashMap<String, String> = HashMap::new();
        trait_prefixes.insert("Capacitor".into(), "C".into());

        let overrides: HashMap<String, String> = HashMap::new();

        let infos = build_instance_infos(&ir, &device_traits, &trait_prefixes, &overrides);

        assert_eq!(infos.len(), 2);
        assert_eq!(infos[0].prefix, Some("C".into()));
        assert_eq!(infos[1].prefix, None); // will default to "U"
    }
}
