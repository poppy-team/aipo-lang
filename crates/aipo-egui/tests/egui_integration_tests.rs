//! Integration tests for `aipo-egui` host profile, capabilities, handles, and VM execution.

#![forbid(unsafe_code)]

use std::cell::RefCell;
use std::rc::Rc;

use aipo_bytecode::{BytecodeModule, compile};
use aipo_diagnostics::DiagnosticCode;
use aipo_egui::{
    EGUI_LOCK, EguiService, egui_begin_frame, egui_begin_window, egui_button, egui_checkbox,
    egui_create_context, egui_destroy_context, egui_end_frame, egui_end_window, egui_heading,
    egui_label, egui_progress_bar, egui_separator, egui_slider, egui_text_edit,
    egui_wants_pointer_input, install_egui, register_egui, revoke_egui,
};
use aipo_hir::lower;
use aipo_host::CapabilitySet;
use aipo_ir::lower_to_ir;
use aipo_runtime::NativeRegistry;
use aipo_source::{Source, SourceId};
use aipo_stdlib::register_stdlib;
use aipo_syntax::parse;
use aipo_vm::{DictMap, Value, Vm};

fn make_dict(entries: Vec<(Value, Value)>) -> Value {
    Value::Dict(Rc::new(RefCell::new(DictMap::from_entries(entries))))
}

fn compile_program(code: &str) -> BytecodeModule {
    let src = Source::new(SourceId::next(), "test_egui.aipo", code);
    let (ast, parse_diags) = parse(&src);
    assert!(parse_diags.is_empty(), "parse errors: {parse_diags:?}");
    let hir = lower(ast);
    let ir = lower_to_ir(&hir);
    compile(&ir).expect("program compiles")
}

fn create_test_vm(bytecode: &BytecodeModule) -> Vm {
    let mut vm = Vm::new();
    let mut registry = NativeRegistry::new();
    register_stdlib(&mut vm, &mut registry);
    register_egui(&mut vm);

    for decl in &bytecode.structs {
        let fields: Vec<(&str, bool)> = decl
            .fields
            .iter()
            .map(|(name, fixed)| (name.as_str(), *fixed))
            .collect();
        vm.register_struct(decl.name.clone(), fields);
    }
    for function in &bytecode.functions {
        if let Some((type_name, method)) = function.name.split_once('.') {
            vm.register_struct_method(
                type_name,
                method,
                function.entry_ip,
                function.params,
                function.is_async,
            );
        }
    }
    vm
}

#[test]
fn test_capability_denied_when_revoked() {
    let _lock = EGUI_LOCK.lock().unwrap();
    revoke_egui();

    let err = egui_create_context(&[]).expect_err("should be denied without capability");
    assert_eq!(
        err.diagnostic_code(),
        DiagnosticCode::AIPO_RT_CAPABILITY_DENIED
    );
}

#[test]
fn test_capability_denied_when_missing_permission() {
    let _lock = EGUI_LOCK.lock().unwrap();
    install_egui(EguiService::with_capabilities(CapabilitySet::none()));

    let err = egui_create_context(&[]).expect_err("should be denied without egui capability");
    assert_eq!(
        err.diagnostic_code(),
        DiagnosticCode::AIPO_RT_CAPABILITY_DENIED
    );
}

#[test]
fn test_handle_lifecycle_and_stale_detection() {
    let _lock = EGUI_LOCK.lock().unwrap();
    install_egui(EguiService::new());

    let handle_val = egui_create_context(&[]).expect("context created");
    assert!(matches!(handle_val, Value::HostHandle(_)));

    // Destroy the context
    egui_destroy_context(std::slice::from_ref(&handle_val)).expect("context destroyed");

    // Second destroy on stale handle must fault
    let err = egui_destroy_context(std::slice::from_ref(&handle_val)).expect_err("must be stale");
    assert_eq!(err.diagnostic_code(), DiagnosticCode::AIPO_RT_STALE_HANDLE);

    // Using stale handle in begin_frame must fault
    let err2 = egui_begin_frame(std::slice::from_ref(&handle_val)).expect_err("must be stale");
    assert_eq!(err2.diagnostic_code(), DiagnosticCode::AIPO_RT_STALE_HANDLE);
}

#[test]
fn test_immediate_frame_and_shape_extraction() {
    let _lock = EGUI_LOCK.lock().unwrap();
    install_egui(EguiService::new());

    let ctx = egui_create_context(&[]).expect("context created");

    // Begin frame
    let opts = make_dict(vec![
        (
            Value::String(Rc::new("width".to_string())),
            Value::Float(800.0),
        ),
        (
            Value::String(Rc::new("height".to_string())),
            Value::Float(600.0),
        ),
        (
            Value::String(Rc::new("dt".to_string())),
            Value::Float(0.016),
        ),
    ]);

    egui_begin_frame(&[ctx.clone(), opts]).expect("begin_frame");

    // Build window and widgets
    egui_begin_window(&[
        Value::String(Rc::new("Configurações".to_string())),
        Value::Float(50.0),
        Value::Float(50.0),
        Value::Float(300.0),
        Value::Float(200.0),
    ])
    .expect("begin_window");

    egui_heading(&[Value::String(Rc::new("Painel Geral".to_string()))]).expect("heading");
    egui_separator(&[]).expect("separator");
    egui_label(&[Value::String(Rc::new("Status: Pronto".to_string()))]).expect("label");

    let btn_clicked = egui_button(&[Value::String(Rc::new("Salvar".to_string()))]).expect("button");
    assert_eq!(btn_clicked, Value::Bool(false));

    let chk = egui_checkbox(&[
        Value::String(Rc::new("Ativar Som".to_string())),
        Value::Bool(true),
    ])
    .expect("checkbox");
    assert_eq!(chk, Value::Bool(true));

    let slider_val = egui_slider(&[
        Value::String(Rc::new("Volume".to_string())),
        Value::Float(75.0),
        Value::Float(0.0),
        Value::Float(100.0),
    ])
    .expect("slider");
    assert_eq!(slider_val, Value::Float(75.0));

    let txt = egui_text_edit(&[
        Value::String(Rc::new("Nome".to_string())),
        Value::String(Rc::new("Jogador".to_string())),
    ])
    .expect("text_edit");
    assert_eq!(txt, Value::String(Rc::new("Jogador".to_string())));

    egui_progress_bar(&[Value::Float(0.65)]).expect("progress_bar");

    egui_end_window(&[]).expect("end_window");

    // End frame
    let output = egui_end_frame(std::slice::from_ref(&ctx)).expect("end_frame");

    if let Value::Dict(dict_rc) = output {
        let dict = dict_rc.borrow();
        let shapes = dict
            .get(&Value::String(Rc::new("shapes".to_string())))
            .expect("shapes list");
        if let Value::List(shapes_list) = shapes {
            assert!(
                !shapes_list.borrow().is_empty(),
                "should have generated visual shapes"
            );
        } else {
            panic!("shapes must be a list");
        }
    } else {
        panic!("output must be a dictionary");
    }

    egui_destroy_context(&[ctx]).expect("cleanup");
}

#[test]
fn test_interactive_pointer_hover_and_click() {
    let _lock = EGUI_LOCK.lock().unwrap();
    install_egui(EguiService::new());

    let ctx = egui_create_context(&[]).expect("context created");

    // Frame 1: Pointer hover over window area (100, 100)
    let opts1 = make_dict(vec![
        (
            Value::String(Rc::new("width".to_string())),
            Value::Float(800.0),
        ),
        (
            Value::String(Rc::new("height".to_string())),
            Value::Float(600.0),
        ),
        (
            Value::String(Rc::new("mouse_x".to_string())),
            Value::Float(100.0),
        ),
        (
            Value::String(Rc::new("mouse_y".to_string())),
            Value::Float(100.0),
        ),
        (
            Value::String(Rc::new("mouse_down".to_string())),
            Value::Bool(false),
        ),
    ]);

    egui_begin_frame(&[ctx.clone(), opts1]).expect("begin_frame frame 1");
    egui_begin_window(&[
        Value::String(Rc::new("Janela".to_string())),
        Value::Float(50.0),
        Value::Float(50.0),
        Value::Float(200.0),
        Value::Float(150.0),
    ])
    .expect("begin_window");
    let _ = egui_button(&[Value::String(Rc::new("Ação".to_string()))]).expect("btn");
    egui_end_window(&[]).expect("end_window");
    let _out1 = egui_end_frame(std::slice::from_ref(&ctx)).expect("end_frame frame 1");

    // Over window => wants_pointer_input should be true
    let wants_pointer =
        egui_wants_pointer_input(std::slice::from_ref(&ctx)).expect("wants_pointer_input");
    assert_eq!(wants_pointer, Value::Bool(true));

    egui_destroy_context(&[ctx]).expect("cleanup");
}

#[test]
fn test_aipo_script_execution_with_egui() {
    let _lock = EGUI_LOCK.lock().unwrap();
    install_egui(EguiService::new());

    let script = r#"
var ctx = egui.create_context()
egui.begin_frame(ctx, {
    "width": 800.0,
    "height": 600.0,
    "dt": 0.016
})
egui.begin_window("Aipo Test", 50.0, 50.0, 200.0, 150.0)
egui.heading("Immediate Mode")
egui.label("Hello from Aipo script!")
var clicked = egui.button("Clique")
egui.end_window()
var out = egui.end_frame(ctx)
egui.destroy_context(ctx)
var success = true
"#;

    let bytecode = compile_program(script);
    let mut vm = create_test_vm(&bytecode);
    vm.run(&bytecode).expect("script executed successfully");

    let success = vm.get_global("success").expect("success global exists");
    assert_eq!(success, &Value::Bool(true));
}

#[test]
fn test_run_example_28_egui_immediate_gui() {
    let _lock = EGUI_LOCK.lock().unwrap();
    install_egui(EguiService::new());

    let script = std::fs::read_to_string("../../examples/28_egui_immediate_gui.aipo")
        .or_else(|_| std::fs::read_to_string("examples/28_egui_immediate_gui.aipo"))
        .expect("read example 28");

    let bytecode = compile_program(&script);
    let mut vm = create_test_vm(&bytecode);
    vm.run(&bytecode).expect("example 28 executed successfully");
}
