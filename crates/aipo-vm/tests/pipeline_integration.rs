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
