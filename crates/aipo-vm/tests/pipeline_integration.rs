//! End-to-end compiler pipeline integration test: Source -> AST -> HIR -> Core IR -> Bytecode -> VM.

use aipo_bytecode::compile;
use aipo_hir::lower;
use aipo_ir::lower_to_ir;
use aipo_source::{Source, SourceId};
use aipo_syntax::parse;
use aipo_vm::{Value, Vm};

#[test]
fn test_end_to_end_source_to_vm_execution() {
    let src = Source::new(
        SourceId::next(),
        "calc.aipo",
        "let a = 15\nlet b = 27\nlet total = a + b",
    );

    let (ast, parse_diags) = parse(&src);
    assert!(parse_diags.is_empty(), "parse errors: {:?}", parse_diags);

    let hir = lower(ast);
    let ir = lower_to_ir(&hir);
    let bytecode = compile(&ir).expect("compilation failed");

    let mut vm = Vm::new();
    let _ = vm.run(&bytecode).expect("vm execution failed");

    assert_eq!(vm.get_global("a"), Some(&Value::Int(15)));
    assert_eq!(vm.get_global("b"), Some(&Value::Int(27)));
    assert_eq!(vm.get_global("total"), Some(&Value::Int(42)));
}

#[test]
fn test_end_to_end_collections_and_control_flow() {
    let src = Source::new(
        SourceId::next(),
        "loop.aipo",
        "let items = [10, 20, 30]\nlet count = 3\nvar idx = 0\nvar acc = 0\nwhile idx < count\n    acc = acc + items[idx]\n    idx = idx + 1\nend",
    );

    let (ast, parse_diags) = parse(&src);
    assert!(parse_diags.is_empty(), "parse errors: {:?}", parse_diags);

    let hir = lower(ast);
    let ir = lower_to_ir(&hir);
    let bytecode = compile(&ir).expect("compilation failed");

    let mut vm = Vm::new();
    let _ = vm.run(&bytecode).expect("vm execution failed");

    assert_eq!(vm.get_global("acc"), Some(&Value::Int(60)));
    assert_eq!(vm.get_global("idx"), Some(&Value::Int(3)));
}

#[test]
fn test_end_to_end_attempt_failed_recovery() {
    let src = Source::new(
        SourceId::next(),
        "attempt.aipo",
        "var recovered = \"initial\"\nattempt\n    fail(\"network failure\")\nfailed err\n    recovered = err.message\nend",
    );

    let (ast, parse_diags) = parse(&src);
    assert!(parse_diags.is_empty(), "parse errors: {:?}", parse_diags);

    let hir = lower(ast);
    let ir = lower_to_ir(&hir);
    let bytecode = compile(&ir).expect("compilation failed");

    let mut vm = Vm::new();
    let _ = vm.run(&bytecode).expect("vm execution failed");

    assert_eq!(
        vm.get_global("recovered"),
        Some(&Value::String(std::rc::Rc::new(
            "network failure".to_string()
        )))
    );
}

#[test]
fn test_end_to_end_short_lambdas() {
    let src = Source::new(
        SourceId::next(),
        "lambdas.aipo",
        "let double = x => x * 2\nlet add = (a, b) => a + b\nlet get_42 = () => 42\nlet sink = _ => 99\nlet r1 = double(21)\nlet r2 = add(10, 32)\nlet r3 = get_42()\nlet r4 = sink(123)",
    );

    let (ast, parse_diags) = parse(&src);
    assert!(parse_diags.is_empty(), "parse errors: {:?}", parse_diags);

    let hir = lower(ast);
    let ir = lower_to_ir(&hir);
    let bytecode = compile(&ir).expect("compilation failed");

    let mut vm = Vm::new();
    let _ = vm.run(&bytecode).expect("vm execution failed");

    assert_eq!(vm.get_global("r1"), Some(&Value::Int(42)));
    assert_eq!(vm.get_global("r2"), Some(&Value::Int(42)));
    assert_eq!(vm.get_global("r3"), Some(&Value::Int(42)));
    assert_eq!(vm.get_global("r4"), Some(&Value::Int(99)));
}

#[test]
fn test_end_to_end_elided_comparisons() {
    let src = Source::new(
        SourceId::next(),
        "elided.aipo",
        "let x = 50\nlet in_range = x >= 0 and <= 100\nlet out_of_range = x < 0 or x > 100\nlet typed_in_range = x is Int and >= 0 and <= 100",
    );

    let (ast, parse_diags) = parse(&src);
    assert!(parse_diags.is_empty(), "parse errors: {:?}", parse_diags);

    let hir = lower(ast);
    let ir = lower_to_ir(&hir);
    let bytecode = compile(&ir).expect("compilation failed");

    let mut vm = Vm::new();
    let mut registry = aipo_runtime::NativeRegistry::new();
    aipo_stdlib::register_stdlib(&mut vm, &mut registry);
    let _ = vm.run(&bytecode).expect("vm execution failed");

    assert_eq!(vm.get_global("in_range"), Some(&Value::Bool(true)));
    assert_eq!(vm.get_global("out_of_range"), Some(&Value::Bool(false)));
    assert_eq!(vm.get_global("typed_in_range"), Some(&Value::Bool(true)));
}

#[test]
fn test_end_to_end_is_nullable() {
    let src = Source::new(
        SourceId::next(),
        "test_nullable.aipo",
        "let a = none\nlet b = 42\nlet c = \"text\"\nlet a_ok = a is Int?\nlet b_ok = b is Int?\nlet c_ok = c is Int?",
    );

    let (ast, parse_diags) = parse(&src);
    assert!(parse_diags.is_empty(), "parse errors: {:?}", parse_diags);

    let hir = lower(ast);
    let ir = lower_to_ir(&hir);
    let bytecode = compile(&ir).expect("compilation failed");

    let mut vm = Vm::new();
    let mut registry = aipo_runtime::NativeRegistry::new();
    aipo_stdlib::register_stdlib(&mut vm, &mut registry);
    let _ = vm.run(&bytecode).expect("vm execution failed");

    assert_eq!(vm.get_global("a_ok"), Some(&Value::Bool(true)));
    assert_eq!(vm.get_global("b_ok"), Some(&Value::Bool(true)));
    assert_eq!(vm.get_global("c_ok"), Some(&Value::Bool(false)));
}

#[test]
fn test_end_to_end_multiple_is() {
    let src = Source::new(
        SourceId::next(),
        "test_multi_is.aipo",
        "let x = 10\nlet y = 20\nlet z = 30\nlet ok = x, y, z is Int\nlet not_all = x, \"hello\", z is Int\nlet nullable_ok = x, none is Int?",
    );

    let (ast, parse_diags) = parse(&src);
    assert!(parse_diags.is_empty(), "parse errors: {:?}", parse_diags);

    let hir = lower(ast);
    let ir = lower_to_ir(&hir);
    let bytecode = compile(&ir).expect("compilation failed");

    let mut vm = Vm::new();
    let mut registry = aipo_runtime::NativeRegistry::new();
    aipo_stdlib::register_stdlib(&mut vm, &mut registry);
    let _ = vm.run(&bytecode).expect("vm execution failed");

    assert_eq!(vm.get_global("ok"), Some(&Value::Bool(true)));
    assert_eq!(vm.get_global("not_all"), Some(&Value::Bool(false)));
    assert_eq!(vm.get_global("nullable_ok"), Some(&Value::Bool(true)));
}
