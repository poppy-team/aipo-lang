//! Runtime contract matrix: every (value, contract) pair pins whether the
//! boundary check passes or faults, on both the parameter and return sides.
//!
//! The matrix pins *strictness* decisions the implementation owns (for example
//! that an `Int` value does not satisfy a `Float` contract): if any cell ever
//! changes, the diff is reviewable here instead of silent.

#![forbid(unsafe_code)]

use aipo_bytecode::compile;
use aipo_diagnostics::DiagnosticCode;
use aipo_hir::lower;
use aipo_ir::lower_to_ir;
use aipo_source::{Source, SourceId};
use aipo_syntax::parse;
use aipo_vm::{Vm, VmError};

fn check_program(source: &str) -> Result<String, String> {
    let src = Source::new(SourceId::next(), "matrix.aipo", source);
    let (program, diagnostics) = parse(&src);
    assert!(
        diagnostics.is_empty(),
        "matrix program parses: {diagnostics:?}"
    );
    let hir = lower(program);
    let ir = lower_to_ir(&hir);
    let bytecode = compile(&ir).expect("matrix program compiles");
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
    match vm.run(&bytecode) {
        Ok(value) => Ok(format!("{value}")),
        Err(error) => Err(match &error {
            VmError::Fault(fault) => fault.diagnostic_code().to_string(),
            VmError::UncaughtFailure(message) => {
                format!("{}:{message}", DiagnosticCode::AIPO_RT_FAILURE_UNCAUGHT)
            }
            // Internal scheduler signal; never escapes `run` in practice.
            VmError::Suspended => "AIPO_RT_INTERNAL_SUSPENDED".to_string(),
        }),
    }
}

fn param_case(contract: &str, argument: &str) -> String {
    format!("fn f(x: {contract})\nreturn x\nend\nf({argument})\n")
}

fn return_case(contract: &str, returned: &str) -> String {
    format!("fn f() -> {contract}\nreturn {returned}\nend\nf()\n")
}

#[test]
fn test_parameter_contract_matrix() {
    let cases = [
        ("Int", "5", true),
        ("Int", "\"s\"", false),
        ("Int", "none", false),
        ("Int", "5.0", false),
        ("Int?", "none", true),
        ("Int?", "5", true),
        ("Float", "2.5", true),
        ("Float", "5", false),
        ("Bool", "true", true),
        ("Bool", "1", false),
        ("String", "\"s\"", true),
        ("Byte", "Byte(5)", true),
        ("Byte", "5", false),
        ("List", "[1]", true),
        ("List", "5", false),
        ("Dict", "{\"k\": 1}", true),
        ("Function", "fn (x) return x end", true),
        ("Function", "5", false),
    ];
    for (contract, argument, passes) in cases {
        let outcome = check_program(&param_case(contract, argument));
        assert_eq!(
            outcome.is_ok(),
            passes,
            "param {contract} with {argument}: {outcome:?}"
        );
        if !passes {
            assert!(
                outcome.unwrap_err().contains("AIPO_RT_TYPE_MISMATCH"),
                "param {contract} with {argument} faults as contract violation"
            );
        }
    }

    let struct_cases = [
        ("S", "S{x = 1}", true),
        ("S", "5", false),
        ("S?", "none", true),
    ];
    for (contract, argument, passes) in struct_cases {
        let program =
            format!("struct S\nx\nend\nfn f(v: {contract})\nreturn v\nend\nf({argument})\n");
        let outcome = check_program(&program);
        assert_eq!(
            outcome.is_ok(),
            passes,
            "param {contract} with {argument}: {outcome:?}"
        );
    }
}

#[test]
fn test_return_contract_matrix() {
    let cases = [
        ("Int", "5", true),
        ("Int", "\"s\"", false),
        ("Int", "none", false),
        ("Int?", "none", true),
        ("Int?", "5", true),
        ("String", "\"s\"", true),
        ("String", "5", false),
    ];
    for (contract, returned, passes) in cases {
        let outcome = check_program(&return_case(contract, returned));
        assert_eq!(
            outcome.is_ok(),
            passes,
            "return {contract} with {returned}: {outcome:?}"
        );
        if !passes {
            assert!(
                outcome.unwrap_err().contains("AIPO_RT_TYPE_MISMATCH"),
                "return {contract} with {returned} faults as contract violation"
            );
        }
    }
}
