//! AHS (Aipo Host Schema) definition for the `egui` host profile.

#![forbid(unsafe_code)]

use aipo_host::ahs::{
    FunctionSchema, HandleSchema, HostSchema, ModuleSchema, ParamSchema, TypeRef,
};

fn tr(name: &str) -> TypeRef {
    TypeRef {
        name: name.to_string(),
        nullable: false,
        args: Vec::new(),
    }
}

fn param(name: &str, ty: &str) -> ParamSchema {
    ParamSchema {
        name: name.to_string(),
        ty: tr(ty),
        is_mut: false,
        is_optional: false,
        docs: None,
    }
}

fn fn_schema(
    name: &str,
    docs: &str,
    params: Vec<ParamSchema>,
    ret: Option<&str>,
) -> FunctionSchema {
    FunctionSchema {
        name: name.to_string(),
        docs: Some(docs.to_string()),
        params,
        returns: ret.map(tr),
        is_async: false,
        mutates_receiver: false,
        capabilities: vec!["egui".to_string()],
        subject: None,
        deprecated: None,
    }
}

/// Constructs the canonical AHS for `egui`.
#[must_use]
pub fn egui_schema() -> HostSchema {
    HostSchema {
        host: "egui".to_string(),
        version: "0.1.0".to_string(),
        modules: vec![ModuleSchema {
            name: "egui".to_string(),
            docs: Some("Agnostic immediate-mode GUI bindings powered by egui".to_string()),
            capabilities: vec!["egui".to_string()],
            types: Vec::new(),
            handles: vec![HandleSchema {
                name: "Context".to_string(),
                docs: Some("Generational handle to an egui immediate-mode session".to_string()),
                operations: vec![
                    "begin_frame".to_string(),
                    "end_frame".to_string(),
                    "destroy".to_string(),
                ],
                capabilities: vec!["egui".to_string()],
                stale_is_none: false,
            }],
            values: Vec::new(),
            functions: vec![
                fn_schema(
                    "create_context",
                    "Creates a new agnostic egui session and returns its handle",
                    Vec::new(),
                    Some("Context"),
                ),
                fn_schema(
                    "destroy_context",
                    "Destroys an egui session, invalidating its handle",
                    vec![param("handle", "Context")],
                    None,
                ),
                fn_schema(
                    "begin_frame",
                    "Begins a new immediate-mode frame pass with input parameters",
                    vec![param("handle", "Context"), param("options", "Dict")],
                    None,
                ),
                fn_schema(
                    "end_frame",
                    "Finishes the frame pass and returns output shapes and interaction state",
                    vec![param("handle", "Context")],
                    Some("Dict"),
                ),
                fn_schema(
                    "begin_window",
                    "Opens a floating styled window container",
                    vec![
                        param("title", "String"),
                        param("x", "Float"),
                        param("y", "Float"),
                        param("width", "Float"),
                        param("height", "Float"),
                    ],
                    None,
                ),
                fn_schema(
                    "end_window",
                    "Closes the currently active window container",
                    Vec::new(),
                    None,
                ),
                fn_schema(
                    "label",
                    "Renders a static text label in the active UI container",
                    vec![param("text", "String")],
                    None,
                ),
                fn_schema(
                    "heading",
                    "Renders a bold heading title in the active UI container",
                    vec![param("text", "String")],
                    None,
                ),
                fn_schema(
                    "separator",
                    "Renders a horizontal visual divider line",
                    Vec::new(),
                    None,
                ),
                fn_schema(
                    "button",
                    "Renders a clickable button returning true if clicked this frame",
                    vec![param("text", "String")],
                    Some("Bool"),
                ),
                fn_schema(
                    "checkbox",
                    "Renders a toggleable checkbox returning the updated boolean state",
                    vec![param("text", "String"), param("checked", "Bool")],
                    Some("Bool"),
                ),
                fn_schema(
                    "slider",
                    "Renders a floating-point numeric slider returning the current value",
                    vec![
                        param("text", "String"),
                        param("value", "Float"),
                        param("min", "Float"),
                        param("max", "Float"),
                    ],
                    Some("Float"),
                ),
                fn_schema(
                    "text_edit",
                    "Renders an interactive single-line text input field",
                    vec![param("label", "String"), param("text", "String")],
                    Some("String"),
                ),
                fn_schema(
                    "progress_bar",
                    "Renders a determinate progress bar between 0.0 and 1.0",
                    vec![param("fraction", "Float")],
                    None,
                ),
                fn_schema(
                    "wants_pointer_input",
                    "Returns true if egui is currently hovering or interacting with pointer controls",
                    vec![param("handle", "Context")],
                    Some("Bool"),
                ),
                fn_schema(
                    "wants_keyboard_input",
                    "Returns true if egui is currently focused on an active text input",
                    vec![param("handle", "Context")],
                    Some("Bool"),
                ),
            ],
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_egui_schema_validates_cleanly() {
        let schema = egui_schema();
        let problems = schema.validate();
        assert!(problems.is_ok(), "schema problems: {problems:?}");
    }

    #[test]
    fn test_egui_schema_declared_capabilities() {
        let schema = egui_schema();
        let caps = schema.declared_capabilities();
        assert!(caps.allows(&aipo_host::Capability::parse("egui").unwrap()));
    }
}
