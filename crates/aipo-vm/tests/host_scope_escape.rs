//! Scoped-escape enforcement at the VM's heap-publication points (Wave 4, `P03-G01`).
//!
//! Canon fixes the rule and the points where it is enforced: a binding created inside a scoped
//! host callback cannot leave it, and a leak is caught at `SetGlobal`, `Return`, `SetField`,
//! `SetIndex`, `BuildList` and `BuildDict` — not at some later read, where the scope it belonged
//! to is already gone from the machine's state.
//!
//! Every test here drives the real pipeline (parse → HIR → IR → bytecode → VM) rather than
//! poking at a helper, because the claim being made is about the opcodes.

use aipo_bytecode::BytecodeModule;
use aipo_bytecode::compile;
use aipo_hir::lower;
use aipo_ir::lower_to_ir;
use aipo_runtime::NativeRegistry;
use aipo_source::{Source, SourceId};
use aipo_stdlib::register_stdlib;
use aipo_syntax::parse;
use aipo_vm::{Value, Vm};

fn compile_program(code: &str) -> BytecodeModule {
    let src = Source::new(SourceId::next(), "escape.aipo", code);
    let (ast, parse_diags) = parse(&src);
    assert!(parse_diags.is_empty(), "parse errors: {parse_diags:?}");
    let hir = lower(ast);
    let ir = lower_to_ir(&hir);
    compile(&ir).expect("program compiles")
}

/// Registers the struct types and methods a module declares.
///
/// The VM executes bytecode and does not read declarations out of the source, so a harness that
/// drives the pipeline itself has to do what the CLI does before a run; without this a program
/// using a struct faults on an undefined type name rather than reaching the code under test.
fn register_declarations(vm: &mut Vm, module: &BytecodeModule) {
    for decl in &module.structs {
        let fields: Vec<(&str, bool)> = decl
            .fields
            .iter()
            .map(|(name, fixed)| (name.as_str(), *fixed))
            .collect();
        vm.register_struct(decl.name.clone(), fields);
    }
    for function in &module.functions {
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
}

/// Runs a program whose global `leaked` holds a host handle minted in the given scope.
///
/// The scope is closed before the run, so the handle in the global is exactly the binding canon
/// says cannot be published any further.
fn run_with_escaped_handle(code: &str, close_scope: bool) -> Result<Vm, aipo_vm::VmError> {
    let bytecode = compile_program(code);
    let mut vm = Vm::new();
    let mut registry = NativeRegistry::new();
    register_stdlib(&mut vm, &mut registry);
    register_declarations(&mut vm, &bytecode);

    let scope = vm.host_context().open_scope();
    let handle = vm
        .host_context()
        .hand_out(aipo_host::HostValue::string("entity"));
    if close_scope {
        vm.host_context().close_scope(scope);
    }
    vm.define_global("leaked", Value::HostHandle(handle));
    vm.run(&bytecode)?;
    Ok(vm)
}

fn assert_scope_escape(result: Result<Vm, aipo_vm::VmError>, site: &str) {
    let error = match result {
        Ok(_) => panic!("the escaped binding must be caught"),
        Err(error) => error,
    };
    assert_eq!(
        error.diagnostic_code(),
        aipo_diagnostics::DiagnosticCode::AIPO_RT_SCOPE_ESCAPE
    );
    let rendered = error.to_string();
    assert!(
        rendered.contains(site),
        "the fault must name the publication site {site}: {rendered}"
    );
}

#[test]
fn test_set_global_catches_an_escaped_binding() {
    assert_scope_escape(
        run_with_escaped_handle("var copy = leaked\n", true),
        "global 'copy'",
    );
}

#[test]
fn test_return_catches_an_escaped_binding() {
    assert_scope_escape(
        run_with_escaped_handle(
            "fn leak()\n    return leaked\nend\nvar held = leak()\n",
            true,
        ),
        "a return value",
    );
}

#[test]
fn test_build_list_catches_an_escaped_binding() {
    // The list literal is itself a publication point: the handle has left its scope the moment
    // it is inside the list, whether or not the list is ever stored.
    assert_scope_escape(
        run_with_escaped_handle("var items = [leaked]\n", true),
        "a list element",
    );
}

#[test]
fn test_build_dict_catches_an_escaped_binding() {
    assert_scope_escape(
        run_with_escaped_handle("var entry = {\"held\": leaked}\n", true),
        "a dict",
    );
}

#[test]
fn test_set_index_catches_an_escaped_binding() {
    assert_scope_escape(
        run_with_escaped_handle("var items = [none]\nitems[0] = leaked\n", true),
        "an indexed element",
    );
}

#[test]
fn test_set_field_catches_an_escaped_binding() {
    assert_scope_escape(
        run_with_escaped_handle(
            "struct Box\n    held\nend\nvar b = Box{held = none}\nb.held = leaked\n",
            true,
        ),
        "field 'held'",
    );
}

#[test]
fn test_a_binding_used_inside_its_own_scope_is_not_caught() {
    // The rule constrains what *leaves* the scope. An open scope changes nothing, so the same
    // program that faults above must run to completion here.
    let vm = run_with_escaped_handle("var copy = leaked\n", false).expect("scope still open");
    let published = vm.get_global("copy").expect("copy was stored");
    assert!(
        matches!(published, Value::HostHandle(_)),
        "the handle is what got stored, not a copy of the host value"
    );
}

#[test]
fn test_a_program_without_host_bindings_is_unaffected() {
    // The check must cost nothing and change nothing for a program that never had a host
    // binding: this is the ordinary path.
    let bytecode = compile_program("var items = [1, 2, 3]\nvar total = 0\n");
    let mut vm = Vm::new();
    let mut registry = NativeRegistry::new();
    register_stdlib(&mut vm, &mut registry);
    vm.run(&bytecode).expect("runs");
    assert!(!vm.host().has_escapes());
}

#[test]
fn test_closing_a_scope_releases_the_slot_before_anything_is_published() {
    // A second handle cannot reuse the first one's identity, so a stale handle can never be
    // mistaken for the live value that took its slot — the escape check builds on that.
    let mut vm = Vm::new();
    let scope = vm.host_context().open_scope();
    let first = vm.host_context().hand_out(aipo_host::HostValue::Int(1));
    vm.host_context().close_scope(scope);
    let second = vm.host_context().hand_out(aipo_host::HostValue::Int(2));

    assert_ne!(first, second, "a released slot mints a new generation");
    assert!(
        vm.host().check_publication(first, "global 'x'").is_err(),
        "the released handle stays escaped"
    );
    assert!(
        vm.host().check_publication(second, "global 'y'").is_ok(),
        "the live handle is publishable"
    );
}
