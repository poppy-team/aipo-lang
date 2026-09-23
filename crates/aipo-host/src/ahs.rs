//! The Aipo Host Schema (AHS): a host surface described as data.
//!
//! Canon (Fechamento Arquitetural §14) has the host *emit* its surface — modules, types,
//! host values, handles, functions, params, return contracts, mutability, async, docs,
//! capabilities and deprecation/version — so the compiler, the LSP and code agents consume
//! it without knowing which host produced it. Nothing in this module names an engine, and
//! nothing in the IR or bytecode ever learns about one.
//!
//! A description is data before it is truth: [`HostSchema::validate`] rejects a surface that
//! is internally inconsistent (duplicate names, a subject that is not a parameter, a
//! malformed capability) so a host cannot half-register and leave the boundary guessing.
//! Whether the *running profile* grants what a function declares is a separate question,
//! answered by [`HostSchema::missing_capabilities`] and [`HostSchema::require_capabilities`].

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::capability::{Capability, CapabilitySet};
use crate::fault::{HostFault, SchemaProblem};

/// A whole host surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostSchema {
    /// Host name, for diagnostics (`poppy`, `headless-test-host`).
    pub host: String,
    /// Host version, so a surface can be pinned.
    pub version: String,
    /// Modules the host exposes.
    #[serde(default)]
    pub modules: Vec<ModuleSchema>,
}

/// One module of the host surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleSchema {
    /// Module name as a script imports it.
    pub name: String,
    /// Optional documentation.
    #[serde(default)]
    pub docs: Option<String>,
    /// Capabilities every function of the module requires unless it declares its own.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Types the module exposes.
    #[serde(default)]
    pub types: Vec<TypeSchema>,
    /// Opaque host objects the module hands out.
    #[serde(default)]
    pub handles: Vec<HandleSchema>,
    /// Plain host values the module exposes.
    #[serde(default)]
    pub values: Vec<ValueSchema>,
    /// Functions the module exposes.
    #[serde(default)]
    pub functions: Vec<FunctionSchema>,
}

/// A type the host exposes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeSchema {
    /// Type name.
    pub name: String,
    /// Optional documentation.
    #[serde(default)]
    pub docs: Option<String>,
    /// Fields, when the host exposes a struct-like type.
    #[serde(default)]
    pub fields: Vec<FieldSchema>,
}

/// A field of an exposed type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldSchema {
    /// Field name.
    pub name: String,
    /// Declared contract.
    #[serde(rename = "type")]
    pub ty: TypeRef,
}

/// A reference to a type from a signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeRef {
    /// Contract name as it would be written in source (`Int`, `Entity`, `Task`).
    pub name: String,
    /// Whether the contract admits `none`.
    #[serde(default)]
    pub nullable: bool,
    /// Parametric arguments, when the host describes a parametric type.
    ///
    /// V1 source has no parametric contracts (ADP-006 §G rejects them with a dedicated
    /// diagnostic); the description keeps them so a future version can consume the same
    /// surface instead of re-emitting it.
    #[serde(default)]
    pub args: Vec<TypeRef>,
}

/// An opaque host object, reachable only through a handle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandleSchema {
    /// Handle type name.
    pub name: String,
    /// Optional documentation.
    #[serde(default)]
    pub docs: Option<String>,
    /// Operations the handle supports.
    #[serde(default)]
    pub operations: Vec<String>,
    /// Capabilities the handle's operations require unless one declares its own.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Whether a stale handle resolves to `none` rather than faulting.
    ///
    /// Canon allows both resolutions and requires the contract to say which one applies.
    #[serde(default)]
    pub stale_is_none: bool,
}

/// A plain value the host exposes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValueSchema {
    /// Value name.
    pub name: String,
    /// Declared contract.
    #[serde(rename = "type")]
    pub ty: TypeRef,
    /// Optional documentation.
    #[serde(default)]
    pub docs: Option<String>,
}

/// One function of the host surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionSchema {
    /// Function name.
    pub name: String,
    /// Optional documentation.
    #[serde(default)]
    pub docs: Option<String>,
    /// Parameters, in call order.
    #[serde(default)]
    pub params: Vec<ParamSchema>,
    /// Return contract, absent when the function returns nothing.
    #[serde(default)]
    pub returns: Option<TypeRef>,
    /// Whether the function is asynchronous (a `Task` handle at the call site).
    #[serde(default)]
    pub is_async: bool,
    /// Whether the function mutates its receiver.
    ///
    /// V1 module functions have no receiver, so this flag is informational; the AHS keeps
    /// it for handle methods a future version may describe (auditoria N-7).
    #[serde(default)]
    pub mutates_receiver: bool,
    /// Capabilities this function requires, on top of the module's.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Trailing-block subject: the parameter a block form would bind.
    #[serde(default)]
    pub subject: Option<String>,
    /// Deprecation note, when the function is on its way out.
    #[serde(default)]
    pub deprecated: Option<String>,
}

/// A parameter of an exposed function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParamSchema {
    /// Parameter name.
    pub name: String,
    /// Declared contract.
    #[serde(rename = "type")]
    pub ty: TypeRef,
    /// Whether the parameter is called `name!`, i.e. the callee may mutate it.
    #[serde(default)]
    pub is_mut: bool,
    /// Whether the parameter may be omitted.
    #[serde(default)]
    pub is_optional: bool,
    /// Optional documentation.
    #[serde(default)]
    pub docs: Option<String>,
}

impl HostSchema {
    /// Parses a surface description from JSON.
    ///
    /// # Errors
    ///
    /// [`HostFault::InvalidHostValue`] when the text is not a well-formed description, and
    /// [`HostFault::Schema`] when it parses but is internally inconsistent.
    pub fn from_json(text: &str) -> Result<Self, HostFault> {
        let schema: Self =
            serde_json::from_str(text).map_err(|error| HostFault::InvalidHostValue {
                detail: format!("host surface is not valid JSON: {error}"),
            })?;
        schema.validate()?;
        Ok(schema)
    }

    /// Checks the description is internally consistent.
    ///
    /// # Errors
    ///
    /// [`HostFault::Schema`] listing every problem found, so a host author fixes them in one
    /// pass instead of one per run.
    pub fn validate(&self) -> Result<(), HostFault> {
        let mut problems = Vec::new();
        if self.host.trim().is_empty() {
            problems.push(SchemaProblem::new("host", "name is empty"));
        }
        if self.version.trim().is_empty() {
            problems.push(SchemaProblem::new("version", "version is empty"));
        }

        let mut seen_modules = BTreeSet::new();
        for module in &self.modules {
            if !seen_modules.insert(module.name.as_str()) {
                problems.push(SchemaProblem::new(
                    &module.name,
                    "module is declared more than once",
                ));
            }
            module.validate(&mut problems);
        }

        if problems.is_empty() {
            Ok(())
        } else {
            Err(HostFault::Schema { problems })
        }
    }

    /// Every capability the surface declares, module-wide or per function.
    ///
    /// The result is a [`CapabilitySet`], so a surface that declares both `poppy` and
    /// `poppy.ecs` reports the single covering grant: the question this answers is what a
    /// profile must grant, and a redundant entry would invite a wrong "missing" reading.
    ///
    /// Only well-formed capabilities are reported: a malformed name is a [`HostSchema::validate`]
    /// problem, so this method assumes a validated surface (which [`HostSchema::from_json`]
    /// guarantees) and silently ignores leftovers instead of inventing a grant (auditoria N-5).
    #[must_use]
    pub fn declared_capabilities(&self) -> CapabilitySet {
        let mut declared = CapabilitySet::none();
        for module in &self.modules {
            collect_capabilities(&module.capabilities, &mut declared);
            for handle in &module.handles {
                collect_capabilities(&handle.capabilities, &mut declared);
            }
            for function in &module.functions {
                collect_capabilities(&function.capabilities, &mut declared);
            }
        }
        declared
    }

    /// The capabilities a profile must grant for the whole surface to be usable.
    #[must_use]
    pub fn missing_capabilities(&self, granted: &CapabilitySet) -> Vec<Capability> {
        self.declared_capabilities()
            .iter()
            .filter(|capability| !granted.allows(capability))
            .cloned()
            .collect()
    }

    /// Fails unless the profile grants every declared capability.
    ///
    /// This is the pre-flight check canon asks for ("the compiler/tooling must be able to
    /// explain when a target/host does not offer a required capability"), so the answer is a
    /// list of what is missing rather than a fault per call site.
    ///
    /// # Errors
    ///
    /// [`HostFault::CapabilityDenied`] naming the first missing capability and the host it
    /// belongs to.
    pub fn require_capabilities(&self, granted: &CapabilitySet) -> Result<(), HostFault> {
        let missing = self.missing_capabilities(granted);
        if missing.is_empty() {
            return Ok(());
        }
        let first = missing.into_iter().next().expect("non-empty");
        Err(HostFault::CapabilityDenied {
            capability: first,
            operation: format!("load host surface '{}'", self.host),
        })
    }

    /// Looks a function up by module and name.
    #[must_use]
    pub fn function(&self, module: &str, function: &str) -> Option<&FunctionSchema> {
        self.modules
            .iter()
            .find(|candidate| candidate.name == module)?
            .functions
            .iter()
            .find(|candidate| candidate.name == function)
    }
}

impl ModuleSchema {
    /// Collects this module's consistency problems.
    ///
    /// Duplicate detection is intentionally namespaced per category (auditoria H-4):
    /// a type named `Foo` and a handle named `Foo` do not collide because they
    /// live in distinct declaration lists consumed independently by hosts.
    fn validate(&self, problems: &mut Vec<SchemaProblem>) {
        if self.name.trim().is_empty() {
            problems.push(SchemaProblem::new("modules[]", "module name is empty"));
        }
        check_capabilities(&self.name, &self.capabilities, problems);

        let mut names = BTreeSet::new();
        for ty in &self.types {
            let ty_path = format!("{}.{}", self.name, ty.name);
            if ty.name.trim().is_empty() {
                problems.push(SchemaProblem::new(
                    format!("{}.types[]", self.name),
                    "type name is empty",
                ));
            }
            if !names.insert(("type", ty.name.as_str())) {
                problems.push(SchemaProblem::new(
                    ty_path.clone(),
                    "type is declared more than once",
                ));
            }
            let mut field_names = BTreeSet::new();
            for field in &ty.fields {
                if field.name.trim().is_empty() {
                    problems.push(SchemaProblem::new(
                        format!("{ty_path}.fields[]"),
                        "field name is empty",
                    ));
                }
                if !field_names.insert(field.name.as_str()) {
                    problems.push(SchemaProblem::new(
                        format!("{ty_path}.{}", field.name),
                        "field is declared more than once",
                    ));
                }
                check_type_ref(&format!("{ty_path}.{}", field.name), &field.ty, problems);
            }
        }
        for handle in &self.handles {
            if !names.insert(("handle", handle.name.as_str())) {
                problems.push(SchemaProblem::new(
                    format!("{}.{}", self.name, handle.name),
                    "handle is declared more than once",
                ));
            }
            check_capabilities(
                &format!("{}.{}", self.name, handle.name),
                &handle.capabilities,
                problems,
            );
            if handle.name.trim().is_empty() {
                problems.push(SchemaProblem::new(
                    format!("{}.handles[]", self.name),
                    "handle name is empty",
                ));
            }
            let mut operations = BTreeSet::new();
            for operation in &handle.operations {
                if operation.trim().is_empty() {
                    problems.push(SchemaProblem::new(
                        format!("{}.{}.operations[]", self.name, handle.name),
                        "operation name is empty",
                    ));
                }
                if !operations.insert(operation.as_str()) {
                    problems.push(SchemaProblem::new(
                        format!("{}.{}.{}", self.name, handle.name, operation),
                        "operation is declared more than once",
                    ));
                }
            }
        }
        for value in &self.values {
            if value.name.trim().is_empty() {
                problems.push(SchemaProblem::new(
                    format!("{}.values[]", self.name),
                    "value name is empty",
                ));
            }
            if !names.insert(("value", value.name.as_str())) {
                problems.push(SchemaProblem::new(
                    format!("{}.{}", self.name, value.name),
                    "value is declared more than once",
                ));
            }
            check_type_ref(
                &format!("{}.{}", self.name, value.name),
                &value.ty,
                problems,
            );
        }
        for function in &self.functions {
            if !names.insert(("function", function.name.as_str())) {
                problems.push(SchemaProblem::new(
                    format!("{}.{}", self.name, function.name),
                    "function is declared more than once",
                ));
            }
            function.validate(&self.name, problems);
        }
    }
}

impl FunctionSchema {
    /// Collects this function's consistency problems.
    fn validate(&self, module: &str, problems: &mut Vec<SchemaProblem>) {
        let path = format!("{module}.{}", self.name);
        if self.name.trim().is_empty() {
            problems.push(SchemaProblem::new(
                format!("{module}.functions[]"),
                "function name is empty",
            ));
        }
        check_capabilities(&path, &self.capabilities, problems);

        let mut param_names = BTreeSet::new();
        let mut optional_params: BTreeSet<String> = BTreeSet::new();
        let mut seen_optional = false;
        for param in &self.params {
            if param.name.trim().is_empty() {
                problems.push(SchemaProblem::new(
                    format!("{path}.params[]"),
                    "parameter name is empty",
                ));
            }
            if !param_names.insert(param.name.as_str()) {
                problems.push(SchemaProblem::new(
                    format!("{path}({})", param.name),
                    "parameter is declared more than once",
                ));
            }
            check_type_ref(&format!("{path}({})", param.name), &param.ty, problems);
            if param.is_optional {
                seen_optional = true;
                optional_params.insert(param.name.clone());
            } else if seen_optional {
                problems.push(SchemaProblem::new(
                    format!("{path}({})", param.name),
                    "a required parameter cannot follow an optional one",
                ));
            }
        }
        if let Some(returns) = &self.returns {
            check_type_ref(&format!("{path} -> {}", returns.name), returns, problems);
        }
        if let Some(subject) = &self.subject {
            // A block form binds a parameter, so the subject has to name one.
            if !param_names.contains(subject.as_str()) {
                problems.push(SchemaProblem::new(
                    &path,
                    format!("subject '{subject}' is not a parameter of this function"),
                ));
            } else if optional_params.contains(subject.as_str()) {
                problems.push(SchemaProblem::new(
                    &path,
                    format!(
                        "subject '{subject}' is optional; a block form needs a required parameter"
                    ),
                ));
            }
        }
        if self.subject.is_some() && self.is_async {
            problems.push(SchemaProblem::new(
                &path,
                "an async function cannot also take a trailing block",
            ));
        }
    }
}

/// Records every capability in `names`, reporting the malformed ones.
fn check_capabilities(path: &str, names: &[String], problems: &mut Vec<SchemaProblem>) {
    for name in names {
        if let Err(error) = Capability::parse(name) {
            problems.push(SchemaProblem::new(path, error.to_string()));
        }
    }
}

/// Maximum nesting of parametric contract arguments accepted by the validator.
///
/// V1 source has no parametric contracts at all, so anything deeper than this is a
/// malformed or adversarial description; the cap keeps `validate` total (auditoria N-4).
const MAX_TYPE_REF_DEPTH: usize = 32;

/// Validates a contract reference.
fn check_type_ref(path: &str, ty: &TypeRef, problems: &mut Vec<SchemaProblem>) {
    check_type_ref_at(path, ty, problems, 0);
}

fn check_type_ref_at(path: &str, ty: &TypeRef, problems: &mut Vec<SchemaProblem>, depth: usize) {
    if depth > MAX_TYPE_REF_DEPTH {
        problems.push(SchemaProblem::new(
            path,
            format!("contract nesting exceeds the supported depth of {MAX_TYPE_REF_DEPTH}"),
        ));
        return;
    }
    if ty.name.trim().is_empty() {
        problems.push(SchemaProblem::new(path, "contract name is empty"));
    } else if ty.name.trim() != ty.name || ty.name.chars().any(char::is_whitespace) {
        problems.push(SchemaProblem::new(
            path,
            format!(
                "contract name '{}' has surrounding or embedded whitespace",
                ty.name
            ),
        ));
    }
    for argument in &ty.args {
        check_type_ref_at(path, argument, problems, depth + 1);
    }
}

/// Adds every well-formed capability in `names` to `declared`.
fn collect_capabilities(names: &[String], declared: &mut CapabilitySet) {
    for name in names {
        if let Ok(capability) = Capability::parse(name) {
            declared.grant(capability);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilitySet;

    const SURFACE: &str = r#"
{
  "host": "headless-test-host",
  "version": "0.1.0",
  "modules": [
    {
      "name": "world",
      "capabilities": ["poppy"],
      "handles": [
        { "name": "Entity", "operations": ["position", "set_position"], "stale_is_none": false }
      ],
      "types": [
        { "name": "Vec2", "fields": [ { "name": "x", "type": { "name": "Float" } },
                                      { "name": "y", "type": { "name": "Float" } } ] }
      ],
      "values": [ { "name": "origin", "type": { "name": "Vec2" } } ],
      "functions": [
        {
          "name": "spawn",
          "params": [ { "name": "at", "type": { "name": "Vec2" } } ],
          "returns": { "name": "Entity" },
          "subject": "at",
          "docs": "spawns an entity"
        },
        {
          "name": "tick",
          "params": [ { "name": "count", "type": { "name": "Int" } } ],
          "returns": { "name": "Int", "nullable": true },
          "is_async": true,
          "capabilities": ["poppy.ecs"]
        }
      ]
    }
  ]
}
"#;

    #[test]
    fn test_well_formed_surface_is_accepted() {
        let schema = HostSchema::from_json(SURFACE).expect("valid surface");
        assert_eq!(schema.host, "headless-test-host");
        assert_eq!(schema.modules.len(), 1);
        let spawn = schema.function("world", "spawn").expect("declared");
        assert_eq!(
            spawn.returns.as_ref().map(|ty| ty.name.as_str()),
            Some("Entity")
        );
        assert_eq!(spawn.subject.as_deref(), Some("at"));
        assert!(schema.function("world", "missing").is_none());
    }

    #[test]
    fn test_declared_capabilities_cover_modules_handles_and_functions() {
        let schema = HostSchema::from_json(SURFACE).expect("valid surface");
        let declared: Vec<String> = schema
            .declared_capabilities()
            .iter()
            .map(|capability| capability.name().to_string())
            .collect();
        // `poppy` covers `poppy.ecs`, and the set is minimal, so only the ancestor appears.
        assert_eq!(declared, vec!["poppy".to_string()]);
    }

    #[test]
    fn test_missing_capabilities_are_reported_before_running() {
        let schema = HostSchema::from_json(SURFACE).expect("valid surface");
        let denied = CapabilitySet::none();
        let missing = schema.missing_capabilities(&denied);
        assert_eq!(
            missing.iter().map(Capability::name).collect::<Vec<_>>(),
            vec!["poppy"]
        );
        let fault = schema
            .require_capabilities(&denied)
            .expect_err("denied by default");
        assert_eq!(
            fault.code(),
            aipo_diagnostics::DiagnosticCode::AIPO_RT_CAPABILITY_DENIED
        );

        let granted: CapabilitySet = [Capability::parse("poppy").expect("valid")]
            .into_iter()
            .collect();
        assert!(schema.require_capabilities(&granted).is_ok());
    }

    #[test]
    fn test_duplicate_names_are_rejected() {
        let text = r#"{ "host": "h", "version": "1", "modules": [
            { "name": "m", "functions": [ { "name": "f", "params": [],
              "returns": null }, { "name": "f", "params": [], "returns": null } ] } ] }"#;
        let fault = HostSchema::from_json(text).expect_err("duplicate");
        let message = fault.message();
        assert!(message.contains("declared more than once"), "{message}");
    }

    #[test]
    fn test_subject_must_name_a_parameter() {
        let text = r#"{ "host": "h", "version": "1", "modules": [
            { "name": "m", "functions": [ { "name": "f", "params": [
                { "name": "a", "type": { "name": "Int" } } ],
              "subject": "b" } ] } ] }"#;
        let fault = HostSchema::from_json(text).expect_err("bad subject");
        assert!(
            fault.message().contains("subject 'b'"),
            "{}",
            fault.message()
        );
    }

    #[test]
    fn test_async_function_cannot_take_a_trailing_block() {
        let text = r#"{ "host": "h", "version": "1", "modules": [
            { "name": "m", "functions": [ { "name": "f", "is_async": true,
              "params": [ { "name": "body", "type": { "name": "Function" } } ],
              "subject": "body" } ] } ] }"#;
        let fault = HostSchema::from_json(text).expect_err("async block");
        assert!(
            fault
                .message()
                .contains("cannot also take a trailing block"),
            "{}",
            fault.message()
        );
    }

    #[test]
    fn test_malformed_capability_is_rejected() {
        let text = r#"{ "host": "h", "version": "1", "modules": [
            { "name": "m", "capabilities": ["Clock.Wall"], "functions": [] } ] }"#;
        let fault = HostSchema::from_json(text).expect_err("bad capability");
        assert!(fault.message().contains("Clock"), "{}", fault.message());
    }

    #[test]
    fn test_empty_host_name_is_rejected() {
        let text = r#"{ "host": "", "version": "1", "modules": [] }"#;
        assert!(HostSchema::from_json(text).is_err());
    }

    #[test]
    fn test_invalid_json_is_a_boundary_fault() {
        let fault = HostSchema::from_json("{ not json").expect_err("bad json");
        assert_eq!(
            fault.code(),
            aipo_diagnostics::DiagnosticCode::AIPO_RT_TYPE_MISMATCH
        );
    }

    #[test]
    fn test_empty_names_are_rejected_across_categories() {
        let cases = [
            (
                r#"{ "host": "h", "version": "1", "modules": [ { "name": "m",
                    "types": [ { "name": "", "fields": [] } ] } ] }"#,
                "type name is empty",
            ),
            (
                r#"{ "host": "h", "version": "1", "modules": [ { "name": "m",
                    "values": [ { "name": "", "type": { "name": "Int" } } ] } ] }"#,
                "value name is empty",
            ),
            (
                r#"{ "host": "h", "version": "1", "modules": [ { "name": "m",
                    "functions": [ { "name": "f", "params": [
                        { "name": "", "type": { "name": "Int" } } ] } ] } ] }"#,
                "parameter name is empty",
            ),
            (
                r#"{ "host": "h", "version": "1", "modules": [ { "name": "m",
                    "types": [ { "name": "T", "fields": [
                        { "name": "", "type": { "name": "Int" } } ] } ] } ] }"#,
                "field name is empty",
            ),
        ];
        for (text, expected) in cases {
            let fault = HostSchema::from_json(text).expect_err("empty name");
            assert!(
                fault.message().contains(expected),
                "expected '{expected}' in: {}",
                fault.message()
            );
        }
    }

    #[test]
    fn test_duplicate_fields_and_operations_are_rejected() {
        let duplicate_field = r#"{ "host": "h", "version": "1", "modules": [ { "name": "m",
            "types": [ { "name": "T", "fields": [
                { "name": "x", "type": { "name": "Int" } },
                { "name": "x", "type": { "name": "Int" } } ] } ] } ] }"#;
        let fault = HostSchema::from_json(duplicate_field).expect_err("duplicate field");
        assert!(
            fault.message().contains("field is declared more than once"),
            "{}",
            fault.message()
        );

        let duplicate_operation = r#"{ "host": "h", "version": "1", "modules": [ { "name": "m",
            "handles": [ { "name": "E", "operations": ["pos", "pos"] } ] } ] }"#;
        let fault = HostSchema::from_json(duplicate_operation).expect_err("duplicate operation");
        assert!(
            fault
                .message()
                .contains("operation is declared more than once"),
            "{}",
            fault.message()
        );

        let empty_operation = r#"{ "host": "h", "version": "1", "modules": [ { "name": "m",
            "handles": [ { "name": "E", "operations": [""] } ] } ] }"#;
        let fault = HostSchema::from_json(empty_operation).expect_err("empty operation");
        assert!(
            fault.message().contains("operation name is empty"),
            "{}",
            fault.message()
        );
    }

    #[test]
    fn test_required_parameter_cannot_follow_an_optional_one() {
        let text = r#"{ "host": "h", "version": "1", "modules": [ { "name": "m",
            "functions": [ { "name": "f", "params": [
                { "name": "a", "type": { "name": "Int" }, "is_optional": true },
                { "name": "b", "type": { "name": "Int" } } ] } ] } ] }"#;
        let fault = HostSchema::from_json(text).expect_err("optional order");
        assert!(
            fault
                .message()
                .contains("required parameter cannot follow an optional"),
            "{}",
            fault.message()
        );
    }

    #[test]
    fn test_subject_cannot_be_optional() {
        let text = r#"{ "host": "h", "version": "1", "modules": [ { "name": "m",
            "functions": [ { "name": "f", "params": [
                { "name": "body", "type": { "name": "Function" }, "is_optional": true } ],
              "subject": "body" } ] } ] }"#;
        let fault = HostSchema::from_json(text).expect_err("optional subject");
        assert!(
            fault.message().contains("subject 'body' is optional"),
            "{}",
            fault.message()
        );
    }

    #[test]
    fn test_contract_name_shape_and_depth_are_bounded() {
        let spaced = r#"{ "host": "h", "version": "1", "modules": [ { "name": "m",
            "values": [ { "name": "v", "type": { "name": " Int" } } ] } ] }"#;
        let fault = HostSchema::from_json(spaced).expect_err("spaced contract");
        assert!(
            fault.message().contains("whitespace"),
            "{}",
            fault.message()
        );

        // Deeply nested parametric arguments must not recurse without bound. Built as a
        // struct literal because serde_json has its own recursion guard; the validator
        // must be total on any already-parsed description.
        let mut ty = TypeRef {
            name: "T".to_string(),
            nullable: false,
            args: vec![],
        };
        for _ in 0..(MAX_TYPE_REF_DEPTH + 8) {
            ty = TypeRef {
                name: "T".to_string(),
                nullable: false,
                args: vec![ty],
            };
        }
        let schema = HostSchema {
            host: "h".to_string(),
            version: "1".to_string(),
            modules: vec![ModuleSchema {
                name: "m".to_string(),
                docs: None,
                capabilities: vec![],
                types: vec![],
                handles: vec![],
                values: vec![ValueSchema {
                    name: "v".to_string(),
                    ty,
                    docs: None,
                }],
                functions: vec![],
            }],
        };
        let fault = schema.validate().expect_err("deep nesting");
        assert!(
            fault.message().contains("nesting exceeds"),
            "{}",
            fault.message()
        );
    }

    #[test]
    fn test_declared_capabilities_ignore_malformed_names() {
        // `declared_capabilities` assumes a validated surface; on a hand-built schema the
        // malformed entry grants nothing instead of inventing a capability (auditoria N-5).
        let schema = HostSchema {
            host: "h".to_string(),
            version: "1".to_string(),
            modules: vec![ModuleSchema {
                name: "m".to_string(),
                docs: None,
                capabilities: vec!["Clock.Wall".to_string(), "clock".to_string()],
                types: vec![],
                handles: vec![],
                values: vec![],
                functions: vec![],
            }],
        };
        assert!(schema.validate().is_err());
        let declared: Vec<String> = schema
            .declared_capabilities()
            .iter()
            .map(|capability| capability.name().to_string())
            .collect();
        assert_eq!(declared, vec!["clock".to_string()]);
    }
}
