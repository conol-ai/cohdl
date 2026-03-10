//! Generic parameter instantiation and type checking.
//!
//! This module extends name resolution with:
//! - Monomorphization of devices and functions by substituting generic arguments
//! - Validation of generic argument kinds (`Farads`, `Voltage`, `Ohms`, `Package`, user-defined)
//! - Default value application for omitted generic parameters
//! - `impl Trait` constraint verification for fn parameters
//! - Device trait satisfaction checking (pins + spec completeness)
//! - Type alias well-formedness checking
//! - Production of [`TypedDesign`] IR

use std::collections::{HashMap, HashSet};

use cohdl_syntax::ast::{
    self, DesignBodyStmtKind, DesignDecl, DeviceBodyItem, DeviceDecl, Expr, ExprKind, FnDecl,
    FnParamKind, GenericParamKind, NetEndpointKind, PartDecl, PinEntryKind, SourceFile, Span,
    TopLevelItem, TopLevelItemKind, TraitBodyItem, TraitDecl, TypeAlias, TypeExpr,
};

use crate::{ResolvedSourceFile, SemaError};

// ── Value kinds ─────────────────────────────────────────────────────────────

/// The kind of value a generic parameter accepts.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ValueKind {
    Farads,
    Voltage,
    Ohms,
    Amps,
    Package,
    Net,
    Bool,
    Integer,
    Float,
    String,
    /// A user-defined type (trait or device name).
    UserDefined(std::string::String),
    /// An `impl Trait` constraint.
    ImplTrait(Vec<std::string::String>),
    /// Could not determine the kind.
    Unknown,
}

impl std::fmt::Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueKind::Farads => write!(f, "Farads"),
            ValueKind::Voltage => write!(f, "Voltage"),
            ValueKind::Ohms => write!(f, "Ohms"),
            ValueKind::Amps => write!(f, "Amps"),
            ValueKind::Package => write!(f, "Package"),
            ValueKind::Net => write!(f, "Net"),
            ValueKind::Bool => write!(f, "bool"),
            ValueKind::Integer => write!(f, "integer"),
            ValueKind::Float => write!(f, "float"),
            ValueKind::String => write!(f, "String"),
            ValueKind::UserDefined(name) => write!(f, "{}", name),
            ValueKind::ImplTrait(traits) => write!(f, "impl {}", traits.join(" + ")),
            ValueKind::Unknown => write!(f, "unknown"),
        }
    }
}

// ── TypedDesign IR ──────────────────────────────────────────────────────────

/// Unique identifier for a component instance within a design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InstanceId(pub u32);

/// A fully type-checked design: a flat list of component instances and nets.
#[derive(Debug, Clone, PartialEq)]
pub struct TypedDesign {
    /// Design name.
    pub name: std::string::String,
    /// All component instances (devices/parts) with resolved generics.
    pub instances: Vec<ComponentInstance>,
    /// All nets connecting instance pins.
    pub nets: Vec<TypedNet>,
}

/// A single component instance with fully resolved generic arguments.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentInstance {
    /// Unique instance identifier within the design.
    pub id: InstanceId,
    /// The instance name as written in source (e.g. `c1`, `mcu`).
    pub name: std::string::String,
    /// The resolved device name (e.g. `MLCC`).
    pub device: std::string::String,
    /// If this instance is backed by a `part`, the primary MPN.
    pub mpn: Option<std::string::String>,
    /// Generic parameter substitutions (param name → resolved value string).
    pub generic_substitutions: HashMap<std::string::String, std::string::String>,
}

/// A net in the typed design, connecting a set of `(instance_id, pin_name)` endpoints.
#[derive(Debug, Clone, PartialEq)]
pub struct TypedNet {
    /// Net name (from the `net` statement target).
    pub name: std::string::String,
    /// The connected endpoints. Each is `(instance_id, pin_name)`.
    /// External/global nets use `InstanceId(u32::MAX)`.
    pub endpoints: Vec<(InstanceId, std::string::String)>,
}

/// The global instance id used for external/global net endpoints (e.g. `GND`, `VDD`).
pub const EXTERNAL_INSTANCE: InstanceId = InstanceId(u32::MAX);

// ── Type check result ───────────────────────────────────────────────────────

/// Result of the type-checking pass.
#[derive(Debug, Clone)]
pub struct TypeCheckResult {
    /// Successfully type-checked designs.
    pub designs: Vec<TypedDesign>,
    /// All errors encountered during type checking.
    pub errors: Vec<SemaError>,
}

// ── Internal: collected definitions ─────────────────────────────────────────

/// A collected trait definition with its required pins and spec fields.
#[derive(Debug, Clone)]
struct TraitDef {
    /// Required pin names from `pins` blocks.
    required_pins: HashSet<std::string::String>,
    /// Required spec field names and their expected kinds.
    required_specs: Vec<(std::string::String, ValueKind)>,
    /// Parent trait names (super-traits).
    parents: Vec<std::string::String>,
}

/// A collected device definition.
#[derive(Debug, Clone)]
struct DeviceDef {
    name: std::string::String,
    /// Generic parameters: (name, expected kind, optional default expr source text).
    generic_params: Vec<GenericParamDef>,
    /// Traits this device claims to implement.
    impl_traits: Vec<std::string::String>,
    /// Pin names declared by this device.
    declared_pins: HashSet<std::string::String>,
    /// Spec fields declared by this device: (name, kind).
    declared_specs: Vec<(std::string::String, ValueKind)>,
}

/// A collected generic parameter definition.
#[derive(Debug, Clone)]
struct GenericParamDef {
    name: std::string::String,
    kind: ValueKind,
    has_default: bool,
    default_text: Option<std::string::String>,
}

/// A collected part definition.
#[derive(Debug, Clone)]
struct PartDef {
    /// The underlying device name.
    device_name: std::string::String,
    /// Generic arguments provided to the device.
    generic_args: HashMap<std::string::String, std::string::String>,
    /// Primary MPN from the AVL.
    primary_mpn: Option<std::string::String>,
}

/// A collected function definition.
#[derive(Debug, Clone)]
struct FnDef {
    /// Parameters: (name, kind).
    params: Vec<(std::string::String, ValueKind)>,
}

/// A collected type alias definition.
#[derive(Debug, Clone)]
struct TypeAliasDef {
    generic_params: Vec<GenericParamDef>,
    target_device: std::string::String,
    target_args: HashMap<std::string::String, std::string::String>,
}

// ── Type checker ────────────────────────────────────────────────────────────

/// The main type checker state.
struct TypeChecker {
    traits: HashMap<std::string::String, TraitDef>,
    devices: HashMap<std::string::String, DeviceDef>,
    parts: HashMap<std::string::String, PartDef>,
    fns: HashMap<std::string::String, FnDef>,
    type_aliases: HashMap<std::string::String, TypeAliasDef>,
    errors: Vec<SemaError>,
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            traits: HashMap::new(),
            devices: HashMap::new(),
            parts: HashMap::new(),
            fns: HashMap::new(),
            type_aliases: HashMap::new(),
            errors: Vec::new(),
        }
    }

    // ── Phase 1: Collect definitions from AST ───────────────────────────

    fn collect_definitions(&mut self, items: &[TopLevelItem], module_path: &str) {
        for item in items {
            match &item.kind {
                TopLevelItemKind::Trait(t) => self.collect_trait(t, module_path),
                TopLevelItemKind::Device(d) => self.collect_device(d, module_path),
                TopLevelItemKind::Part(p) => self.collect_part(p, module_path),
                TopLevelItemKind::Fn(f) => self.collect_fn(f, module_path),
                TopLevelItemKind::TypeAlias(ta) => self.collect_type_alias(ta, module_path),
                TopLevelItemKind::Module(m) => {
                    let child = qualify(module_path, &m.name.name);
                    self.collect_definitions(&m.items, &child);
                }
                TopLevelItemKind::Design(_)
                | TopLevelItemKind::Use(_)
                | TopLevelItemKind::Mod(_) => {}
            }
        }
    }

    fn collect_trait(&mut self, t: &TraitDecl, module_path: &str) {
        let qname = qualify(module_path, &t.name.name);
        let mut required_pins = HashSet::new();
        let mut required_specs = Vec::new();
        let mut parents = Vec::new();

        if let Some(parent_bound) = &t.parents {
            for b in &parent_bound.bounds {
                parents.push(type_expr_name(b));
            }
        }

        for body_item in &t.body {
            match body_item {
                TraitBodyItem::Pins(pins) => {
                    for entry in &pins.entries {
                        if let Some(name) = pin_entry_name(&entry.kind) {
                            required_pins.insert(name);
                        }
                    }
                }
                TraitBodyItem::Spec(spec) => {
                    for entry in &spec.entries {
                        let kind = expr_to_value_kind(&entry.value);
                        required_specs.push((entry.name.name.clone(), kind));
                    }
                }
                TraitBodyItem::Rule(_) | TraitBodyItem::DesignatorPrefix(_) => {}
            }
        }

        self.traits.insert(
            qname.clone(),
            TraitDef {
                required_pins,
                required_specs,
                parents,
            },
        );
    }

    fn collect_device(&mut self, d: &DeviceDecl, module_path: &str) {
        let qname = qualify(module_path, &d.name.name);

        let generic_params = match &d.generic_params {
            Some(gp) => gp.params.iter().map(generic_param_to_def).collect(),
            None => Vec::new(),
        };

        let impl_traits = match &d.impl_traits {
            Some(tb) => tb.bounds.iter().map(type_expr_name).collect(),
            None => Vec::new(),
        };

        let mut declared_pins = HashSet::new();
        let mut declared_specs = Vec::new();

        for body_item in &d.body {
            match body_item {
                DeviceBodyItem::Pins(pins) => {
                    for entry in &pins.entries {
                        if let Some(name) = pin_entry_name(&entry.kind) {
                            declared_pins.insert(name);
                        }
                    }
                }
                DeviceBodyItem::Spec(spec) => {
                    for entry in &spec.entries {
                        let kind = expr_to_value_kind(&entry.value);
                        declared_specs.push((entry.name.name.clone(), kind));
                    }
                }
                DeviceBodyItem::Package(_) | DeviceBodyItem::Rule(_) => {}
            }
        }

        self.devices.insert(
            qname.clone(),
            DeviceDef {
                name: qname,
                generic_params,
                impl_traits,
                declared_pins,
                declared_specs,
            },
        );
    }

    fn collect_part(&mut self, p: &PartDecl, module_path: &str) {
        let qname = qualify(module_path, &p.name.name);
        let device_name = type_expr_name(&p.device_type);
        let generic_args = extract_generic_args(&p.device_type);

        let primary_mpn = p
            .avl_entries
            .iter()
            .find(|e| e.kind == ast::AvlKind::Primary)
            .and_then(|e| {
                e.fields
                    .iter()
                    .find(|f| f.name.name == "mpn")
                    .map(|f| f.value.value.clone())
            });

        self.parts.insert(
            qname.clone(),
            PartDef {
                device_name,
                generic_args,
                primary_mpn,
            },
        );
    }

    fn collect_fn(&mut self, f: &FnDecl, module_path: &str) {
        let qname = qualify(module_path, &f.name.name);

        let _generic_params: Vec<GenericParamDef> = match &f.generic_params {
            Some(gp) => gp.params.iter().map(generic_param_to_def).collect(),
            None => Vec::new(),
        };

        let params = f
            .params
            .iter()
            .map(|p| {
                let kind = match &p.kind {
                    FnParamKind::Type(te) => type_name_to_kind(&type_expr_name(te)),
                    FnParamKind::ImplConstraint(tb) => {
                        let traits: Vec<_> = tb.bounds.iter().map(type_expr_name).collect();
                        ValueKind::ImplTrait(traits)
                    }
                };
                (p.name.name.clone(), kind)
            })
            .collect();

        self.fns.insert(qname.clone(), FnDef { params });
    }

    fn collect_type_alias(&mut self, ta: &TypeAlias, module_path: &str) {
        let qname = qualify(module_path, &ta.name.name);

        let generic_params = match &ta.generic_params {
            Some(gp) => gp.params.iter().map(generic_param_to_def).collect(),
            None => Vec::new(),
        };

        let target_device = type_expr_name(&ta.target);
        let target_args = extract_generic_args(&ta.target);

        self.type_aliases.insert(
            qname.clone(),
            TypeAliasDef {
                generic_params,
                target_device,
                target_args,
            },
        );
    }

    // ── Phase 2: Validate definitions ───────────────────────────────────

    fn validate_definitions(&mut self, items: &[TopLevelItem], module_path: &str) {
        for item in items {
            match &item.kind {
                TopLevelItemKind::Device(d) => {
                    self.validate_device_trait_satisfaction(d, module_path);
                }
                TopLevelItemKind::TypeAlias(ta) => {
                    self.validate_type_alias(ta, module_path);
                }
                TopLevelItemKind::Module(m) => {
                    let child = qualify(module_path, &m.name.name);
                    self.validate_definitions(&m.items, &child);
                }
                _ => {}
            }
        }
    }

    /// Verify that a device satisfies all traits it claims to implement.
    fn validate_device_trait_satisfaction(&mut self, d: &DeviceDecl, module_path: &str) {
        let qname = qualify(module_path, &d.name.name);
        let device = match self.devices.get(&qname) {
            Some(dev) => dev.clone(),
            None => return,
        };

        for trait_name in &device.impl_traits {
            // Resolve the trait—try as-is, then qualified relative to common paths.
            let tdef = self
                .traits
                .get(trait_name)
                .or_else(|| self.traits.get(&qualify(module_path, trait_name)));
            let tdef = match tdef {
                Some(t) => t.clone(),
                None => continue, // Name resolution already reported this.
            };

            // Check all required pins are present.
            for required_pin in &tdef.required_pins {
                if !device.declared_pins.contains(required_pin) {
                    self.errors.push(SemaError::new(
                        format!(
                            "device `{}` is missing pin `{}` required by trait `{}`",
                            qname, required_pin, trait_name
                        ),
                        d.name.span,
                    ));
                }
            }

            // Check all required spec fields are present.
            for (spec_name, _expected_kind) in &tdef.required_specs {
                let has_spec = device.declared_specs.iter().any(|(n, _)| n == spec_name);
                if !has_spec {
                    self.errors.push(SemaError::new(
                        format!(
                            "device `{}` is missing spec field `{}` required by trait `{}`",
                            qname, spec_name, trait_name
                        ),
                        d.name.span,
                    ));
                }
            }

            // Recursively check parent traits.
            self.check_parent_traits(&device, &tdef, d.name.span, module_path);
        }
    }

    fn check_parent_traits(
        &mut self,
        device: &DeviceDef,
        tdef: &TraitDef,
        span: Span,
        module_path: &str,
    ) {
        for parent_name in &tdef.parents {
            let parent = self
                .traits
                .get(parent_name)
                .or_else(|| self.traits.get(&qualify(module_path, parent_name)));
            let parent = match parent {
                Some(p) => p.clone(),
                None => continue,
            };

            for required_pin in &parent.required_pins {
                if !device.declared_pins.contains(required_pin) {
                    self.errors.push(SemaError::new(
                        format!(
                            "device `{}` is missing pin `{}` required by parent trait `{}`",
                            device.name, required_pin, parent_name
                        ),
                        span,
                    ));
                }
            }

            for (spec_name, _) in &parent.required_specs {
                let has_spec = device.declared_specs.iter().any(|(n, _)| n == spec_name);
                if !has_spec {
                    self.errors.push(SemaError::new(
                        format!(
                            "device `{}` is missing spec field `{}` required by parent trait `{}`",
                            device.name, spec_name, parent_name
                        ),
                        span,
                    ));
                }
            }
        }
    }

    /// Verify that a type alias is well-formed: the target device exists and
    /// all generic arguments match the target's parameters.
    fn validate_type_alias(&mut self, ta: &TypeAlias, module_path: &str) {
        let qname = qualify(module_path, &ta.name.name);
        let alias = match self.type_aliases.get(&qname) {
            Some(a) => a.clone(),
            None => return,
        };

        let target_device = self.devices.get(&alias.target_device).or_else(|| {
            self.devices
                .get(&qualify(module_path, &alias.target_device))
        });
        let target_device = match target_device {
            Some(d) => d.clone(),
            None => {
                // Could be a builtin or resolved elsewhere—skip.
                return;
            }
        };

        // Each alias generic param must be used in the target args.
        // Also check that target args reference valid device params.
        for arg_name in alias.target_args.keys() {
            let found = target_device
                .generic_params
                .iter()
                .any(|p| p.name == *arg_name);
            if !found {
                self.errors.push(SemaError::new(
                    format!(
                        "type alias `{}` passes unknown generic argument `{}` to device `{}`",
                        qname, arg_name, alias.target_device
                    ),
                    ta.name.span,
                ));
            }
        }

        // Check that all required (no-default) params of the target are either
        // provided by the alias target_args or exposed as alias generic params.
        for param in &target_device.generic_params {
            if !param.has_default && !alias.target_args.contains_key(&param.name) {
                // Is it exposed via the alias's own generic params?
                let exposed = alias.generic_params.iter().any(|p| p.name == param.name);
                if !exposed {
                    self.errors.push(SemaError::new(
                        format!(
                            "type alias `{}` does not provide required generic parameter `{}` of device `{}`",
                            qname, param.name, alias.target_device
                        ),
                        ta.name.span,
                    ));
                }
            }
        }
    }

    // ── Phase 3: Type-check designs, produce IR ─────────────────────────

    fn check_designs(
        &mut self,
        items: &[TopLevelItem],
        resolved: &ResolvedSourceFile,
        module_path: &str,
    ) -> Vec<TypedDesign> {
        let mut designs = Vec::new();
        for item in items {
            match &item.kind {
                TopLevelItemKind::Design(d) => {
                    if let Some(td) = self.check_design(d, resolved, module_path) {
                        designs.push(td);
                    }
                }
                TopLevelItemKind::Module(m) => {
                    let child = qualify(module_path, &m.name.name);
                    let mut child_designs = self.check_designs(&m.items, resolved, &child);
                    designs.append(&mut child_designs);
                }
                _ => {}
            }
        }
        designs
    }

    fn check_design(
        &mut self,
        d: &DesignDecl,
        resolved: &ResolvedSourceFile,
        module_path: &str,
    ) -> Option<TypedDesign> {
        let mut instances = Vec::new();
        let mut nets = Vec::new();
        let mut instance_map: HashMap<std::string::String, InstanceId> = HashMap::new();
        let mut next_id: u32 = 0;

        // First pass: collect instances.
        for stmt in &d.body {
            if let DesignBodyStmtKind::Inst(inst) = &stmt.kind {
                let id = InstanceId(next_id);
                next_id += 1;

                let type_name = type_expr_name(&inst.ty);

                // Resolve through type aliases.
                let (device_name, base_args) = self.resolve_type_name(&type_name, module_path);

                // Check if this is a part or a device.
                let (final_device, mpn, part_args) =
                    if let Some(part) = self.find_part(&type_name, module_path) {
                        (
                            part.device_name.clone(),
                            part.primary_mpn.clone(),
                            part.generic_args.clone(),
                        )
                    } else {
                        (device_name.clone(), None, HashMap::new())
                    };

                // Merge args: base_args (from alias) < part_args < inst args.
                let mut all_args = base_args;
                for (k, v) in part_args {
                    all_args.insert(k, v);
                }
                let inst_args = extract_generic_args(&inst.ty);
                for (k, v) in inst_args {
                    all_args.insert(k, v);
                }

                // Validate generic arguments against the device definition.
                if let Some(dev) = self.find_device(&final_device, module_path) {
                    let dev = dev.clone();
                    self.check_generic_args(
                        &dev.generic_params,
                        &all_args,
                        &type_name,
                        inst.ty.span,
                    );
                    // Apply defaults for missing params.
                    for param in &dev.generic_params {
                        if !all_args.contains_key(&param.name) {
                            if let Some(default) = &param.default_text {
                                all_args.insert(param.name.clone(), default.clone());
                            }
                        }
                    }
                }

                instance_map.insert(inst.name.name.clone(), id);

                instances.push(ComponentInstance {
                    id,
                    name: inst.name.name.clone(),
                    device: final_device,
                    mpn,
                    generic_substitutions: all_args,
                });
            }
        }

        // Second pass: collect nets.
        for stmt in &d.body {
            if let DesignBodyStmtKind::Net(net_stmt) = &stmt.kind {
                let net_name = match &net_stmt.target.kind {
                    NetEndpointKind::Ident(id) => id.name.clone(),
                    NetEndpointKind::DotPath(dp) => dp
                        .root
                        .segments
                        .iter()
                        .map(|s| s.name.as_str())
                        .collect::<Vec<_>>()
                        .join("."),
                };

                let mut endpoints = Vec::new();

                // Collect all endpoints (target + right-hand side).
                let all_ep = std::iter::once(&net_stmt.target).chain(net_stmt.endpoints.iter());
                for ep in all_ep {
                    match &ep.kind {
                        NetEndpointKind::Ident(id) => {
                            // Global/external net endpoint.
                            endpoints.push((EXTERNAL_INSTANCE, id.name.clone()));
                        }
                        NetEndpointKind::DotPath(dp) => {
                            let root_name = &dp.root.segments[0].name;
                            if let Some(&inst_id) = instance_map.get(root_name.as_str()) {
                                if let Some(field) = dp.fields.first() {
                                    endpoints.push((inst_id, field.name.clone()));
                                }
                            } else {
                                self.errors.push(SemaError::new(
                                    format!("unknown instance `{}` in net endpoint", root_name),
                                    ep.span,
                                ));
                            }
                        }
                    }
                }

                nets.push(TypedNet {
                    name: net_name,
                    endpoints,
                });
            }
        }

        // Third pass: check function calls.
        for stmt in &d.body {
            if let DesignBodyStmtKind::Call(call) = &stmt.kind {
                let fn_name = call
                    .path
                    .segments
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
                    .join("::");
                self.check_call(&fn_name, &call.args, call.path.span, module_path, resolved);
            }
        }

        Some(TypedDesign {
            name: d.name.name.clone(),
            instances,
            nets,
        })
    }

    /// Validate generic arguments against parameter definitions.
    fn check_generic_args(
        &mut self,
        params: &[GenericParamDef],
        args: &HashMap<std::string::String, std::string::String>,
        type_name: &str,
        span: Span,
    ) {
        for param in params {
            if let Some(arg_value) = args.get(&param.name) {
                // Check kind compatibility.
                let arg_kind = infer_value_kind(arg_value);
                if !kinds_compatible(&param.kind, &arg_kind) {
                    self.errors.push(SemaError::new(
                        format!(
                            "generic argument `{}` of `{}` expects `{}`, got `{}`",
                            param.name, type_name, param.kind, arg_kind
                        ),
                        span,
                    ));
                }
            } else if !param.has_default {
                self.errors.push(SemaError::new(
                    format!(
                        "missing required generic argument `{}` for `{}`",
                        param.name, type_name
                    ),
                    span,
                ));
            }
        }
    }

    /// Verify a function call's arguments satisfy `impl Trait` constraints.
    fn check_call(
        &mut self,
        fn_name: &str,
        call_args: &[ast::CallArg],
        span: Span,
        module_path: &str,
        _resolved: &ResolvedSourceFile,
    ) {
        let fndef = self.find_fn(fn_name, module_path).cloned();
        let fndef = match fndef {
            Some(f) => f,
            None => return, // Name resolution already reported.
        };

        for (param_name, param_kind) in &fndef.params {
            if let ValueKind::ImplTrait(required_traits) = param_kind {
                // Find the call argument matching this parameter.
                let arg = call_args.iter().find(|a| a.name.name == *param_name);
                if let Some(arg) = arg {
                    let arg_type_name = expr_to_type_name(&arg.value);
                    if let Some(arg_type_name) = arg_type_name {
                        // Check that the provided type implements all required traits.
                        for required_trait in required_traits {
                            if !self.type_implements_trait(
                                &arg_type_name,
                                required_trait,
                                module_path,
                            ) {
                                self.errors.push(SemaError::new(
                                    format!(
                                        "argument `{}` of `{}` requires `impl {}`, but `{}` does not implement `{}`",
                                        param_name, fn_name, required_trait, arg_type_name, required_trait
                                    ),
                                    span,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    /// Check whether a given type (device/part) implements a trait.
    fn type_implements_trait(&self, type_name: &str, trait_name: &str, module_path: &str) -> bool {
        // Check devices.
        if let Some(dev) = self.find_device(type_name, module_path) {
            return dev.impl_traits.iter().any(|t| t == trait_name)
                || dev.impl_traits.iter().any(|t| {
                    qualify(module_path, t) == trait_name || *t == qualify(module_path, trait_name)
                });
        }

        // Check parts—look up the underlying device.
        if let Some(part) = self.find_part(type_name, module_path) {
            return self.type_implements_trait(&part.device_name, trait_name, module_path);
        }

        false
    }

    // ── Helpers: find definitions with module-path fallback ──────────────

    fn find_device(&self, name: &str, module_path: &str) -> Option<&DeviceDef> {
        self.devices
            .get(name)
            .or_else(|| self.devices.get(&qualify(module_path, name)))
    }

    fn find_part(&self, name: &str, module_path: &str) -> Option<&PartDef> {
        self.parts
            .get(name)
            .or_else(|| self.parts.get(&qualify(module_path, name)))
    }

    fn find_fn(&self, name: &str, module_path: &str) -> Option<&FnDef> {
        self.fns
            .get(name)
            .or_else(|| self.fns.get(&qualify(module_path, name)))
    }

    /// Resolve a type name through aliases, returning (device_name, base_generic_args).
    fn resolve_type_name(
        &self,
        name: &str,
        module_path: &str,
    ) -> (
        std::string::String,
        HashMap<std::string::String, std::string::String>,
    ) {
        let alias = self
            .type_aliases
            .get(name)
            .or_else(|| self.type_aliases.get(&qualify(module_path, name)));
        if let Some(alias) = alias {
            (alias.target_device.clone(), alias.target_args.clone())
        } else {
            (name.to_string(), HashMap::new())
        }
    }
}

// ── Free helper functions ───────────────────────────────────────────────────

/// Join a module path and a name.
fn qualify(module_path: &str, name: &str) -> std::string::String {
    if module_path.is_empty() {
        name.to_string()
    } else {
        format!("{}::{}", module_path, name)
    }
}

/// Extract the simple name from a type expression.
fn type_expr_name(te: &TypeExpr) -> std::string::String {
    te.path
        .segments
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>()
        .join("::")
}

/// Extract generic arguments from a type expression into a name→value map.
fn extract_generic_args(te: &TypeExpr) -> HashMap<std::string::String, std::string::String> {
    let mut args = HashMap::new();
    if let Some(ga) = &te.generic_args {
        for arg in &ga.args {
            args.insert(arg.name.name.clone(), expr_to_string(&arg.value));
        }
    }
    args
}

/// Convert an expression to a source-like string representation.
fn expr_to_string(expr: &Expr) -> std::string::String {
    match &expr.kind {
        ExprKind::EngineeringNumber(en) => {
            let mut s = en.number.clone();
            if let Some(suffix) = &en.suffix {
                s.push_str(suffix);
            }
            s
        }
        ExprKind::Integer(n) => n.to_string(),
        ExprKind::String(s) => s.value.clone(),
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Type(te) => {
            let mut s = type_expr_name(te);
            if let Some(ga) = &te.generic_args {
                s.push('<');
                for (i, arg) in ga.args.iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&arg.name.name);
                    s.push_str(": ");
                    s.push_str(&expr_to_string(&arg.value));
                }
                s.push('>');
            }
            s
        }
        ExprKind::DotPath(dp) => {
            let mut s = dp
                .root
                .segments
                .iter()
                .map(|seg| seg.name.as_str())
                .collect::<Vec<_>>()
                .join("::");
            for field in &dp.fields {
                s.push('.');
                s.push_str(&field.name);
            }
            s
        }
        ExprKind::FnCall(fc) => {
            let name = fc
                .path
                .segments
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>()
                .join("::");
            format!("{}(...)", name)
        }
        ExprKind::Binary(b) => format!(
            "{} {:?} {}",
            expr_to_string(&b.lhs),
            b.op,
            expr_to_string(&b.rhs)
        ),
        ExprKind::Unary(u) => format!("{:?}{}", u.op, expr_to_string(&u.operand)),
        ExprKind::Paren(e) => format!("({})", expr_to_string(e)),
    }
}

/// Extract a type name from an expression (if it's a type or simple identifier).
fn expr_to_type_name(expr: &Expr) -> Option<std::string::String> {
    match &expr.kind {
        ExprKind::Type(te) => Some(type_expr_name(te)),
        ExprKind::DotPath(dp) if dp.fields.is_empty() => Some(
            dp.root
                .segments
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>()
                .join("::"),
        ),
        _ => None,
    }
}

/// Convert a GenericParam AST node to a GenericParamDef.
fn generic_param_to_def(p: &ast::GenericParam) -> GenericParamDef {
    let kind = match &p.kind {
        GenericParamKind::Type(te) => type_name_to_kind(&type_expr_name(te)),
        GenericParamKind::ImplConstraint(tb) => {
            let traits: Vec<_> = tb.bounds.iter().map(type_expr_name).collect();
            ValueKind::ImplTrait(traits)
        }
    };
    GenericParamDef {
        name: p.name.name.clone(),
        kind,
        has_default: p.default.is_some(),
        default_text: p.default.as_ref().map(expr_to_string),
    }
}

/// Map a type name to a ValueKind.
fn type_name_to_kind(name: &str) -> ValueKind {
    match name {
        "Farads" => ValueKind::Farads,
        "Voltage" => ValueKind::Voltage,
        "Ohms" => ValueKind::Ohms,
        "Amps" => ValueKind::Amps,
        "Package" => ValueKind::Package,
        "Net" => ValueKind::Net,
        "bool" => ValueKind::Bool,
        "u8" | "u16" | "u32" | "u64" | "i8" | "i16" | "i32" | "i64" => ValueKind::Integer,
        "f32" | "f64" => ValueKind::Float,
        "String" => ValueKind::String,
        other => ValueKind::UserDefined(other.to_string()),
    }
}

/// Infer the value kind from a string representation of a value.
fn infer_value_kind(value: &str) -> ValueKind {
    let starts_with_digit = value.chars().next().is_some_and(|c| c.is_ascii_digit());

    // Check for engineering number suffixes.
    if starts_with_digit && (value.ends_with('F') || value.ends_with('f')) {
        return ValueKind::Farads;
    }
    if starts_with_digit && (value.ends_with('V') || value.ends_with('v')) {
        return ValueKind::Voltage;
    }
    if starts_with_digit && (value.ends_with('R') || value.ends_with("ohm")) {
        return ValueKind::Ohms;
    }
    if starts_with_digit && value.len() > 1 && (value.ends_with('A') || value.ends_with('a')) {
        return ValueKind::Amps;
    }
    // Check for boolean.
    if value == "true" || value == "false" {
        return ValueKind::Bool;
    }
    // Check for integer.
    if value.parse::<u64>().is_ok() || value.parse::<i64>().is_ok() {
        return ValueKind::Integer;
    }
    // Check for float.
    if value.parse::<f64>().is_ok() {
        return ValueKind::Float;
    }
    // Otherwise treat as a Package/identifier.
    if value.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
        return ValueKind::Package;
    }

    ValueKind::Unknown
}

/// Determine ValueKind from an expression (used for spec entries and defaults).
fn expr_to_value_kind(expr: &Expr) -> ValueKind {
    match &expr.kind {
        ExprKind::EngineeringNumber(en) => {
            if let Some(suffix) = &en.suffix {
                suffix_to_kind(suffix)
            } else {
                ValueKind::Float
            }
        }
        ExprKind::Integer(_) => ValueKind::Integer,
        ExprKind::String(_) => ValueKind::String,
        ExprKind::Bool(_) => ValueKind::Bool,
        ExprKind::Type(te) => type_name_to_kind(&type_expr_name(te)),
        _ => ValueKind::Unknown,
    }
}

/// Map an engineering number suffix to a ValueKind.
fn suffix_to_kind(suffix: &str) -> ValueKind {
    if suffix.ends_with('F') || suffix.ends_with('f') {
        ValueKind::Farads
    } else if suffix.ends_with('V') || suffix.ends_with('v') {
        ValueKind::Voltage
    } else if suffix.ends_with('R') || suffix == "Ω" || suffix.ends_with("ohm") {
        ValueKind::Ohms
    } else if suffix.ends_with('A') || suffix.ends_with('a') {
        ValueKind::Amps
    } else {
        ValueKind::Unknown
    }
}

/// Check whether an argument kind is compatible with a parameter kind.
fn kinds_compatible(expected: &ValueKind, got: &ValueKind) -> bool {
    if expected == got {
        return true;
    }
    // Unknown is compatible with anything (we can't tell).
    if *got == ValueKind::Unknown || *expected == ValueKind::Unknown {
        return true;
    }
    // Integer/Float are compatible with Farads/Voltage/Ohms/Amps (bare numbers).
    if matches!(
        expected,
        ValueKind::Farads | ValueKind::Voltage | ValueKind::Ohms | ValueKind::Amps
    ) && matches!(got, ValueKind::Integer | ValueKind::Float)
    {
        return true;
    }
    // UserDefined is compatible with Package (identifiers like C0402).
    if *expected == ValueKind::Package && matches!(got, ValueKind::UserDefined(_)) {
        return true;
    }
    if matches!(expected, ValueKind::UserDefined(_)) && *got == ValueKind::Package {
        return true;
    }
    false
}

/// Get the pin name from a PinEntryKind.
fn pin_entry_name(kind: &PinEntryKind) -> Option<std::string::String> {
    match kind {
        PinEntryKind::Single { name, .. }
        | PinEntryKind::List { name, .. }
        | PinEntryKind::Range { name, .. }
        | PinEntryKind::BusMacro { name, .. }
        | PinEntryKind::Typed { name, .. } => Some(name.name.clone()),
    }
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Run type checking on a parsed source file with its resolved symbols.
///
/// Returns a [`TypeCheckResult`] containing type-checked designs and any errors.
pub fn type_check(source: &SourceFile, resolved: &ResolvedSourceFile) -> TypeCheckResult {
    let mut checker = TypeChecker::new();

    // Phase 1: Collect all definitions.
    checker.collect_definitions(&source.items, "");

    // Phase 2: Validate definitions (trait satisfaction, type alias well-formedness).
    checker.validate_definitions(&source.items, "");

    // Phase 3: Type-check designs and produce IR.
    let designs = checker.check_designs(&source.items, resolved, "");

    TypeCheckResult {
        designs,
        errors: checker.errors,
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve;
    use cohdl_parser::parse_source_file;

    /// Helper: parse, resolve, and type-check source text.
    fn check_src(src: &str) -> TypeCheckResult {
        let sf = parse_source_file(src).expect("parse failed");
        let resolved = resolve(&sf);
        // Name resolution errors are separate; we only care about type-check errors.
        type_check(&sf, &resolved)
    }

    /// Helper: check that the error list contains a message matching the given substring.
    fn has_error(result: &TypeCheckResult, substr: &str) -> bool {
        result.errors.iter().any(|e| e.message.contains(substr))
    }

    // ── Missing generic argument ─────────────────────────────────────────

    #[test]
    fn error_on_missing_generic_arg_no_default() {
        let src = r#"
            trait Capacitor {
                pins { A: Pin, B: Pin }
                spec { capacitance: Farads, voltage_rating: Voltage }
            }
            device MLCC<C: Farads, V: Voltage>: impl Capacitor {
                pins { A: 1, B: 2 }
                spec { capacitance: C, voltage_rating: V }
            }
            design Board {
                inst c: MLCC
            }
        "#;
        let result = check_src(src);
        assert!(
            has_error(&result, "missing required generic argument `C`"),
            "errors: {:?}",
            result.errors
        );
        assert!(
            has_error(&result, "missing required generic argument `V`"),
            "errors: {:?}",
            result.errors
        );
    }

    #[test]
    fn default_generic_arg_fills_omitted_param() {
        let src = r#"
            trait Capacitor {
                pins { A: Pin, B: Pin }
                spec { capacitance: Farads, voltage_rating: Voltage }
            }
            device MLCC<C: Farads, V: Voltage = 10V, pkg: Package = C0402>: impl Capacitor {
                pins { A: 1, B: 2 }
                spec { capacitance: C, voltage_rating: V }
            }
            design Board {
                inst c: MLCC<C: 100nF>
            }
        "#;
        let result = check_src(src);
        // No missing-arg errors—V and pkg have defaults.
        assert!(
            !has_error(&result, "missing required generic argument"),
            "errors: {:?}",
            result.errors
        );
        // The instance should have all three substitutions.
        let design = &result.designs[0];
        let inst = &design.instances[0];
        assert_eq!(inst.generic_substitutions.get("C"), Some(&"100nF".into()));
        assert_eq!(inst.generic_substitutions.get("V"), Some(&"10V".into()));
        assert_eq!(inst.generic_substitutions.get("pkg"), Some(&"C0402".into()));
    }

    // ── Wrong kind ───────────────────────────────────────────────────────

    #[test]
    fn error_on_wrong_generic_kind() {
        let src = r#"
            trait Capacitor {
                pins { A: Pin, B: Pin }
                spec { capacitance: Farads, voltage_rating: Voltage }
            }
            device MLCC<C: Farads, V: Voltage>: impl Capacitor {
                pins { A: 1, B: 2 }
                spec { capacitance: C, voltage_rating: V }
            }
            design Board {
                inst c: MLCC<C: 3.3V, V: 100nF>
            }
        "#;
        let result = check_src(src);
        assert!(
            has_error(&result, "expects `Farads`, got `Voltage`"),
            "errors: {:?}",
            result.errors
        );
        assert!(
            has_error(&result, "expects `Voltage`, got `Farads`"),
            "errors: {:?}",
            result.errors
        );
    }

    #[test]
    fn correct_generic_kinds_pass() {
        let src = r#"
            trait Capacitor {
                pins { A: Pin, B: Pin }
                spec { capacitance: Farads, voltage_rating: Voltage }
            }
            device MLCC<C: Farads, V: Voltage>: impl Capacitor {
                pins { A: 1, B: 2 }
                spec { capacitance: C, voltage_rating: V }
            }
            design Board {
                inst c: MLCC<C: 100nF, V: 10V>
            }
        "#;
        let result = check_src(src);
        assert!(
            !has_error(&result, "expects"),
            "errors: {:?}",
            result.errors
        );
    }

    // ── Unsatisfied trait constraint ─────────────────────────────────────

    #[test]
    fn error_on_unsatisfied_impl_trait_constraint() {
        let src = r#"
            trait Capacitor {
                pins { A: Pin, B: Pin }
                spec { capacitance: Farads }
            }
            device MLCC: impl Capacitor {
                pins { A: 1, B: 2 }
                spec { capacitance: Farads }
            }
            device Resistor {
                pins { A: 1, B: 2 }
            }
            fn decoupling(vdd: Net, gnd: Net, cap: impl Capacitor) {}
            design Board {
                decoupling(vdd: Net, gnd: Net, cap: Resistor)
            }
        "#;
        let result = check_src(src);
        assert!(
            has_error(&result, "does not implement `Capacitor`"),
            "errors: {:?}",
            result.errors
        );
    }

    #[test]
    fn satisfied_impl_trait_passes() {
        let src = r#"
            trait Capacitor {
                pins { A: Pin, B: Pin }
                spec { capacitance: Farads }
            }
            device MLCC: impl Capacitor {
                pins { A: 1, B: 2 }
                spec { capacitance: Farads }
            }
            fn decoupling(vdd: Net, gnd: Net, cap: impl Capacitor) {}
            design Board {
                decoupling(vdd: Net, gnd: Net, cap: MLCC)
            }
        "#;
        let result = check_src(src);
        assert!(
            !has_error(&result, "does not implement"),
            "errors: {:?}",
            result.errors
        );
    }

    // ── Device trait satisfaction (pins + spec) ──────────────────────────

    #[test]
    fn error_on_missing_trait_pin() {
        let src = r#"
            trait Capacitor {
                pins { A: Pin, B: Pin }
            }
            device BadCap: impl Capacitor {
                pins { A: 1 }
            }
        "#;
        let result = check_src(src);
        assert!(
            has_error(&result, "missing pin `B` required by trait `Capacitor`"),
            "errors: {:?}",
            result.errors
        );
    }

    #[test]
    fn error_on_missing_trait_spec() {
        let src = r#"
            trait Capacitor {
                pins { A: Pin, B: Pin }
                spec { capacitance: Farads, voltage_rating: Voltage }
            }
            device BadCap: impl Capacitor {
                pins { A: 1, B: 2 }
                spec { capacitance: Farads }
            }
        "#;
        let result = check_src(src);
        assert!(
            has_error(
                &result,
                "missing spec field `voltage_rating` required by trait `Capacitor`"
            ),
            "errors: {:?}",
            result.errors
        );
    }

    // ── Type alias well-formedness ───────────────────────────────────────

    #[test]
    fn error_on_type_alias_missing_required_param() {
        let src = r#"
            trait Capacitor {
                pins { A: Pin, B: Pin }
                spec { capacitance: Farads, voltage_rating: Voltage }
            }
            device MLCC<C: Farads, V: Voltage>: impl Capacitor {
                pins { A: 1, B: 2 }
                spec { capacitance: C, voltage_rating: V }
            }
            type SmallCap = MLCC<V: 10V>
        "#;
        let result = check_src(src);
        assert!(
            has_error(
                &result,
                "does not provide required generic parameter `C` of device `MLCC`"
            ),
            "errors: {:?}",
            result.errors
        );
    }

    #[test]
    fn type_alias_forwarding_generic_is_well_formed() {
        let src = r#"
            trait Capacitor {
                pins { A: Pin, B: Pin }
                spec { capacitance: Farads, voltage_rating: Voltage }
            }
            device MLCC<C: Farads, V: Voltage>: impl Capacitor {
                pins { A: 1, B: 2 }
                spec { capacitance: C, voltage_rating: V }
            }
            type SmallCap<C: Farads> = MLCC<C: C, V: 10V>
        "#;
        let result = check_src(src);
        assert!(
            !has_error(&result, "does not provide required generic parameter"),
            "errors: {:?}",
            result.errors
        );
    }

    // ── Correct full design ──────────────────────────────────────────────

    #[test]
    fn full_design_produces_typed_ir() {
        let src = r#"
            trait Capacitor {
                pins { A: Pin, B: Pin }
                spec { capacitance: Farads, voltage_rating: Voltage }
            }
            device MLCC<C: Farads, V: Voltage = 10V, pkg: Package = C0402>: impl Capacitor {
                pins { A: 1, B: 2 }
                spec { capacitance: C, voltage_rating: V }
            }
            part mlcc_100nF: MLCC<C: 100nF, V: 10V, pkg: C0402> {
                primary { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }
            }
            design Board {
                inst c1: mlcc_100nF
                inst c2: MLCC<C: 22nF, V: 16V>
                net vdd: c1.A, c2.A
                net gnd: c1.B, c2.B
            }
        "#;
        let result = check_src(src);
        assert!(
            result.errors.is_empty(),
            "unexpected errors: {:?}",
            result.errors
        );
        assert_eq!(result.designs.len(), 1);

        let design = &result.designs[0];
        assert_eq!(design.name, "Board");
        assert_eq!(design.instances.len(), 2);

        // c1 is from the part, should have MPN.
        let c1 = &design.instances[0];
        assert_eq!(c1.name, "c1");
        assert_eq!(c1.device, "MLCC");
        assert_eq!(c1.mpn, Some("CL05B104KO5NNNC".to_string()));

        // c2 is a direct device instantiation, no MPN.
        let c2 = &design.instances[1];
        assert_eq!(c2.name, "c2");
        assert_eq!(c2.device, "MLCC");
        assert_eq!(c2.mpn, None);
        assert_eq!(c2.generic_substitutions.get("C"), Some(&"22nF".into()));
        assert_eq!(c2.generic_substitutions.get("V"), Some(&"16V".into()));
        // pkg should have been filled from default.
        assert_eq!(c2.generic_substitutions.get("pkg"), Some(&"C0402".into()));

        // Nets.
        assert_eq!(design.nets.len(), 2);
        let vdd_net = &design.nets[0];
        assert_eq!(vdd_net.name, "vdd");
        assert_eq!(vdd_net.endpoints.len(), 3); // vdd (external) + c1.A + c2.A
        let gnd_net = &design.nets[1];
        assert_eq!(gnd_net.name, "gnd");
        assert_eq!(gnd_net.endpoints.len(), 3); // gnd (external) + c1.B + c2.B
    }

    #[test]
    fn part_resolves_mpn_through_avl() {
        let src = r#"
            trait Capacitor {
                pins { A: Pin, B: Pin }
                spec { capacitance: Farads }
            }
            device MLCC<C: Farads>: impl Capacitor {
                pins { A: 1, B: 2 }
                spec { capacitance: C }
            }
            part cap_100nF: MLCC<C: 100nF> {
                primary { mfr: "Murata", mpn: "GRM155R61C104KA88D" }
                alt { mfr: "Samsung", mpn: "CL05B104KO5NNNC" }
            }
            design Board {
                inst c: cap_100nF
            }
        "#;
        let result = check_src(src);
        assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
        let design = &result.designs[0];
        let inst = &design.instances[0];
        assert_eq!(inst.mpn, Some("GRM155R61C104KA88D".to_string()));
    }

    #[test]
    fn ohms_kind_check() {
        let src = r#"
            trait Resistor {
                pins { A: Pin, B: Pin }
            }
            device RES<R: Ohms>: impl Resistor {
                pins { A: 1, B: 2 }
            }
            design Board {
                inst r: RES<R: 10V>
            }
        "#;
        let result = check_src(src);
        assert!(
            has_error(&result, "expects `Ohms`, got `Voltage`"),
            "errors: {:?}",
            result.errors
        );
    }

    #[test]
    fn multiple_generic_errors_collected() {
        let src = r#"
            trait Cap {
                pins { A: Pin, B: Pin }
            }
            device MLCC<C: Farads, V: Voltage, pkg: Package>: impl Cap {
                pins { A: 1, B: 2 }
            }
            design Board {
                inst c: MLCC
            }
        "#;
        let result = check_src(src);
        // Should have three missing-arg errors.
        let missing_count = result
            .errors
            .iter()
            .filter(|e| e.message.contains("missing required generic argument"))
            .count();
        assert_eq!(missing_count, 3, "errors: {:?}", result.errors);
    }
}
