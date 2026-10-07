//! Integration tests for the self-reference cycle guard in `aipo-vm`.
//!
//! Storing a collection into itself creates a reference cycle that the `Rc`
//! counter cannot collect. The VM guards both `SetIndex` and `SetField`
//! against pointer identity with the target, raising a recoverable `Failure`.

use aipo_bytecode::compile;
use aipo_hir::lower;
use aipo_ir::lower_to_ir;
use aipo_source::{Source, SourceId};
use aipo_syntax::parse;
use aipo_vm::{Value, Vm};

/// Drives the full pipeline from source text and returns the VM's globals.
fn run_program(source_text: &str) -> std::result::Result<(), aipo_vm::VmError> {
    let source = Source::new(SourceId::next(), "test.aipo", source_text);
    let (ast, diags) = parse(&source);
    assert!(diags.is_empty(), "parse diagnostics: {diags:?}");

    let hir = lower(ast);
    let ir = lower_to_ir(&hir);
    let bytecode = compile(&ir).expect("compilation to bytecode succeeds");

    let mut vm = Vm::new();
    let mut registry = aipo_runtime::NativeRegistry::new();
    aipo_stdlib::register_stdlib(&mut vm, &mut registry);
    for decl in &bytecode.structs {
        let fields: Vec<(&str, bool)> = decl
            .fields
            .iter()
            .map(|(name, fixed)| (name.as_str(), *fixed))
            .collect();
        vm.register_struct(decl.name.clone(), fields);
    }
    vm.run(&bytecode)?;
    Ok(())
}

/// Reads back a global after running a program.
fn run_and_read(source_text: &str, name: &str) -> Option<Value> {
    let source = Source::new(SourceId::next(), "test.aipo", source_text);
    let (ast, diags) = parse(&source);
    assert!(diags.is_empty(), "parse diagnostics: {diags:?}");

    let hir = lower(ast);
    let ir = lower_to_ir(&hir);
    let bytecode = compile(&ir).expect("compilation to bytecode succeeds");

    let mut vm = Vm::new();
    let mut registry = aipo_runtime::NativeRegistry::new();
    aipo_stdlib::register_stdlib(&mut vm, &mut registry);
    for decl in &bytecode.structs {
        let fields: Vec<(&str, bool)> = decl
            .fields
            .iter()
            .map(|(name, fixed)| (name.as_str(), *fixed))
            .collect();
        vm.register_struct(decl.name.clone(), fields);
    }
    vm.run(&bytecode).expect("program runs");
    vm.get_global(name).cloned()
}

/// Assigning a list into one of its own slots would form a cycle; the VM refuses
/// it, so the surrounding `attempt` catches the failure.
#[test]
fn test_storing_list_into_itself_raises_failure() {
    let code = r#"
var caught = false
var xs = [1, 2, 3]
attempt {
  xs[0] = xs
} failed err {
  caught = true
}
"#;
    run_program(code).expect("the cycle failure is caught");
    let caught = run_and_read(code, "caught").expect("global `caught` exists");
    assert!(
        matches!(caught, Value::Bool(true)),
        "the handler ran: {caught:?}"
    );
}

/// The failure carries a message naming the cycle.
#[test]
fn test_cycle_failure_message_names_the_cycle() {
    let code = r#"
var message = ""
var xs = [1]
attempt {
  xs[0] = xs
} failed err {
  message = err.message
}
"#;
    run_program(code).expect("the cycle failure is caught");
    let message = run_and_read(code, "message").expect("global `message` exists");
    let Value::String(text) = message else {
        panic!("message is a String: {message:?}");
    };
    assert!(
        text.contains("itself") || text.contains("cycle"),
        "the message names the self-reference: {text}"
    );
}

/// Assigning a dict into one of its own entries raises the same failure.
#[test]
fn test_storing_dict_into_itself_raises_failure() {
    let code = r#"
var caught = false
var d = { 1: 10 }
attempt {
  d[1] = d
} failed err {
  caught = true
}
"#;
    run_program(code).expect("dict self-cycle is caught");
    let caught = run_and_read(code, "caught").expect("global `caught` exists");
    assert!(
        matches!(caught, Value::Bool(true)),
        "the handler ran: {caught:?}"
    );
}

/// Storing a struct into its own field raises the failure.
#[test]
fn test_storing_struct_into_its_own_field_raises_failure() {
    let code = r#"
struct Node {
  var next
}

var caught = false
var n = Node{ next: 0 }
attempt {
  n.next = n
} failed err {
  caught = true
}
"#;
    run_program(code).expect("struct self-cycle is caught");
    let caught = run_and_read(code, "caught").expect("global `caught` exists");
    assert!(
        matches!(caught, Value::Bool(true)),
        "the handler ran: {caught:?}"
    );
}

/// An uncaught cycle guard surfaces as a program-level failure, not a silent leak.
#[test]
fn test_uncaught_cycle_failure_is_reported() {
    let code = r#"
var xs = [1, 2, 3]
xs[0] = xs
"#;
    let result = run_program(code);
    assert!(
        result.is_err(),
        "an uncaught cycle failure must be reported, not swallowed"
    );
}

/// A separate list with identical contents has a different allocation, so the
/// assignment is permitted — the guard tests identity, not equality.
#[test]
fn test_storing_equal_looking_copy_is_permitted() {
    let code = r#"
var a = [1, 2]
var b = [1, 2]
a[0] = b
"#;
    run_program(code).expect("a distinct copy has a distinct allocation");
}

/// Storing a different struct into the field is permitted.
#[test]
fn test_storing_distinct_struct_into_field_is_permitted() {
    let code = r#"
struct Node {
  var next
}

var a = Node{ next: 0 }
var b = Node{ next: 0 }
a.next = b
"#;
    run_program(code).expect("distinct struct succeeds");
}

/// Storing a scalar into a list slot is never a cycle and stays permitted.
#[test]
fn test_storing_scalar_into_list_is_permitted() {
    let code = r#"
var xs = [1, 2, 3]
xs[0] = 99
"#;
    run_program(code).expect("scalar assignment is unaffected");
}
