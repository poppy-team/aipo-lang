//! `aipo-ir` defines the target-neutral Core Intermediate Representation (Core IR)
//! for the Aipo compiler.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod builder;
pub mod ir;

pub use aipo_ast::{BinaryOp, UnaryOp};
pub use builder::IrBuilder;
pub use ir::*;

use aipo_hir::HirProgram;

/// Lowers an `HirProgram` into a target-neutral `CoreModule`.
#[must_use]
pub fn lower_to_ir(program: &HirProgram) -> CoreModule {
    let builder = IrBuilder::new();
    builder.build(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aipo_hir::lower;
    use aipo_source::{Source, SourceId};
    use aipo_syntax::parse;

    #[test]
    fn test_lower_arithmetic_to_ir() {
        let src = Source::new(SourceId::next(), "test.aipo", "let x = 10 + 20");
        let (ast, diags) = parse(&src);
        assert!(diags.is_empty());
        let hir = lower(ast);
        let ir = lower_to_ir(&hir);

        // Const, Const, Add, PropagateFailure (statement boundary), Store, Return
        assert_eq!(ir.top_level.instructions.len(), 6);
        assert!(matches!(
            &ir.top_level.instructions[0],
            CoreInst::Constant(CoreConstant::Int(10), _)
        ));
        assert!(matches!(
            &ir.top_level.instructions[1],
            CoreInst::Constant(CoreConstant::Int(20), _)
        ));
        assert!(matches!(
            &ir.top_level.instructions[2],
            CoreInst::Binary(aipo_ast::BinaryOp::Add, _)
        ));
        assert!(matches!(
            &ir.top_level.instructions[3],
            CoreInst::PropagateFailure(_)
        ));
        assert!(matches!(
            &ir.top_level.instructions[4],
            CoreInst::Store(name, _) if name == "x"
        ));
    }

    #[test]
    fn test_lower_function_to_ir() {
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "fn square(n)\n  return n * n\nend",
        );
        let (ast, diags) = parse(&src);
        assert!(diags.is_empty());
        let hir = lower(ast);
        let ir = lower_to_ir(&hir);

        assert_eq!(ir.functions.len(), 1);
        assert_eq!(ir.functions[0].name, "square");
        assert_eq!(ir.functions[0].params, vec!["n"]);
    }

    fn first_constant(text: &str) -> CoreConstant {
        let src = Source::new(SourceId::next(), "test.aipo", text);
        let (ast, diags) = parse(&src);
        assert!(diags.is_empty(), "parse diagnostics: {diags:?}");
        let ir = lower_to_ir(&lower(ast));
        match &ir.top_level.instructions[0] {
            CoreInst::Constant(value, _) => value.clone(),
            other => panic!("first instruction is not a constant: {other:?}"),
        }
    }

    /// Literal text is classified once, in `aipo-lexer`: bases and `_` separators must reach
    /// the constant pool as their numeric value, never as `0`.
    #[test]
    fn test_literal_bases_and_separators_lower_to_values() {
        assert_eq!(first_constant("0xFF"), CoreConstant::Int(255));
        assert_eq!(first_constant("0b1010"), CoreConstant::Int(10));
        assert_eq!(first_constant("0o17"), CoreConstant::Int(15));
        assert_eq!(first_constant("1_000_000"), CoreConstant::Int(1_000_000));
        assert_eq!(first_constant("1_0.5"), CoreConstant::Float(10.5));
        assert_eq!(first_constant("1.5e-3"), CoreConstant::Float(0.0015));
    }

    /// `CompoundAssign` on `Index` must evaluate `base`/`idx` once (auditoria IR-1):
    /// `list[f()] += 1` calls `f` exactly once via hidden locals.
    #[test]
    fn test_compound_assign_index_evaluates_base_and_index_once() {
        let src = Source::new(SourceId::next(), "test.aipo", "list[f()] += 1");
        let (ast, diags) = parse(&src);
        assert!(diags.is_empty(), "parse diagnostics: {diags:?}");
        let ir = lower_to_ir(&lower(ast));
        let calls = ir
            .top_level
            .instructions
            .iter()
            .filter(|inst| matches!(inst, CoreInst::Call { .. }))
            .count();
        assert_eq!(
            calls, 1,
            "index call evaluated once: {:?}",
            ir.top_level.instructions
        );
        assert!(
            ir.top_level.instructions.iter().any(|inst| matches!(
                inst,
                CoreInst::Store(name, _) if name.contains("cai_base")
            )),
            "base spilled to hidden local"
        );
    }

    /// Invalid assignment targets fail loudly instead of being silently dropped
    /// (auditoria IR-2). Sema rejects these statically; IR is the backstop.
    #[test]
    fn test_invalid_assign_target_emits_fail() {
        use aipo_hir::{HirExpr, HirProgram, HirStmt};
        use aipo_source::SourceSpan;
        let span = SourceSpan::default();
        let target = HirExpr::Literal(aipo_ast::Literal::Int("1".to_string()), span);
        let value = HirExpr::Literal(aipo_ast::Literal::Int("2".to_string()), span);
        for stmt in [
            HirStmt::Assign(target.clone(), value.clone(), span),
            HirStmt::CompoundAssign(aipo_ast::BinaryOp::Add, target.clone(), value.clone(), span),
        ] {
            let program = HirProgram {
                items: vec![],
                statements: vec![stmt],
                span,
            };
            let ir = lower_to_ir(&program);
            assert!(
                ir.top_level
                    .instructions
                    .iter()
                    .any(|inst| matches!(inst, CoreInst::Fail(_))),
                "invalid target must emit Fail"
            );
        }
    }

    /// A literal outside `Int`'s range is never coerced: it becomes a sentinel the VM and the
    /// JS shim both reject with `AIPO_RT_OVERFLOW` when the constant is loaded. An overflowing
    /// float parses to infinity, which `AIPO_RT_NON_FINITE_FLOAT` rejects — again on both
    /// backends. The old `unwrap_or(0)`/`unwrap_or(0.0)` produced a bogus zero instead.
    #[test]
    fn test_literals_outside_the_value_range_are_rejected_at_load() {
        assert_eq!(
            first_constant("99999999999999999999"),
            CoreConstant::Int(i64::MAX)
        );
        assert_eq!(
            first_constant("0xFFFFFFFFFFFFFFFFFF"),
            CoreConstant::Int(i64::MAX)
        );
        assert!(matches!(
            first_constant("1e999"),
            CoreConstant::Float(f) if f.is_infinite()
        ));
    }
}

#[cfg(test)]
mod lowering_tests {
    use super::*;
    use aipo_hir::lower;
    use aipo_source::{Source, SourceId};
    use aipo_syntax::parse;

    fn lower_text(text: &str) -> CoreModule {
        let src = Source::new(SourceId::next(), "test.aipo", text);
        let (ast, diags) = parse(&src);
        assert!(diags.is_empty(), "parse diagnostics: {diags:?}");
        lower_to_ir(&lower(ast))
    }

    fn all_instructions(module: &CoreModule) -> Vec<&CoreInst> {
        module
            .top_level
            .instructions
            .iter()
            .chain(module.functions.iter().flat_map(|f| f.instructions.iter()))
            .collect()
    }

    fn count<F: Fn(&CoreInst) -> bool>(module: &CoreModule, predicate: F) -> usize {
        all_instructions(module)
            .into_iter()
            .filter(|inst| predicate(inst))
            .count()
    }

    #[test]
    fn test_or_else_lowers_to_a_lazy_handler() {
        // `a() or_else b()` must build the fallback behind a handler, so it runs only on a
        // `Failure` (auditoria IR-6), and the fallback propagates its own failure (IR-7).
        let module = lower_text("fn a()\nreturn 1\nend\nfn b()\nreturn 2\nend\na() or_else b()\n");
        assert!(
            count(&module, |inst| matches!(inst, CoreInst::PushHandler(..))) >= 1,
            "or_else registers a handler: {:?}",
            module.top_level.instructions
        );
        assert!(
            !module
                .top_level
                .instructions
                .iter()
                .any(|inst| matches!(inst, CoreInst::Binary(BinaryOp::OrElse, _))),
            "the eager OrElse opcode is no longer emitted"
        );
        // The fallback body ends with its own propagation check.
        assert!(
            count(&module, |inst| matches!(
                inst,
                CoreInst::PropagateFailure(_)
            )) >= 2
        );
    }

    #[test]
    fn test_safe_navigation_tests_the_receiver_once() {
        // `p?.name` compares the receiver against `none`, so it evaluates it into a hidden
        // local exactly once and only reads the field on the non-none path.
        let module = lower_text("fn label(p)\nreturn p?.name\nend\n");
        assert!(
            count(&module, |inst| matches!(
                inst,
                CoreInst::Binary(BinaryOp::Equal, _)
            )) >= 1,
            "the none test is an equality"
        );
        let stores = count(
            &module,
            |inst| matches!(inst, CoreInst::Store(name, _) if name.contains("safe_base")),
        );
        assert_eq!(stores, 1, "receiver stored once");
        assert!(
            count(
                &module,
                |inst| matches!(inst, CoreInst::GetField(name, _) if name == "name")
            ) == 1
        );
    }

    #[test]
    fn test_each_uses_iteration_modes() {
        // One name projects the primary binding; two names project key and value.
        let one = lower_text("each x in items\nio.println(x)\nend\n");
        assert!(
            count(&one, |inst| matches!(
                inst,
                CoreInst::IterAt(IterMode::Primary, _)
            )) == 1
        );
        let two = lower_text("each k, v in items\nio.println(String(k) + String(v))\nend\n");
        assert!(
            count(&two, |inst| matches!(
                inst,
                CoreInst::IterAt(IterMode::Key, _)
            )) == 1
        );
        assert!(
            count(&two, |inst| matches!(inst, CoreInst::GetIndex(_))) >= 1,
            "the value side reads the collection"
        );
    }

    #[test]
    fn test_pipeline_evaluates_the_argument_before_the_callee() {
        // Source parsing desugars `a |> f` into `f(a)` at HIR level, so the Core IR
        // pipeline arm is exercised with a hand-built program; its argument must be
        // parked before the callee is evaluated (auditoria IR-16).
        use aipo_hir::{HirExpr, HirProgram, HirStmt};
        use aipo_source::SourceSpan;

        let span = SourceSpan::default();
        let program = HirProgram {
            items: vec![],
            statements: vec![HirStmt::Expr(HirExpr::Binary(
                BinaryOp::Pipeline,
                Box::new(HirExpr::Literal(
                    aipo_ast::Literal::Int("1".to_string()),
                    span,
                )),
                Box::new(HirExpr::Identifier("f".to_string(), span)),
                span,
            ))],
            span,
        };
        let module = lower_to_ir(&program);
        let instructions: Vec<&CoreInst> = all_instructions(&module);
        let arg_store = instructions
            .iter()
            .position(|inst| matches!(inst, CoreInst::Store(name, _) if name.contains("pip_arg")))
            .expect("argument parked in a hidden local");
        let first_call = instructions
            .iter()
            .position(|inst| matches!(inst, CoreInst::Call { .. }))
            .expect("pipeline call");
        assert!(
            arg_store < first_call,
            "the argument is evaluated before the callee"
        );
    }

    #[test]
    fn test_bare_return_under_a_contract_checks_none() {
        let module = lower_text("fn f() -> Int\nreturn\nend\n");
        let function = module
            .functions
            .iter()
            .find(|f| f.name == "f")
            .expect("function is lowered");
        assert!(
            function
                .instructions
                .iter()
                .any(|inst| matches!(inst, CoreInst::Constant(CoreConstant::None, _))),
            "bare return pushes none before the contract check: {:?}",
            function.instructions
        );
        assert!(
            function
                .instructions
                .iter()
                .any(|inst| matches!(inst, CoreInst::AssertContract { .. }))
        );
    }

    #[test]
    fn test_return_unwinds_frame_handlers_and_guards() {
        let module = lower_text(
            "fn f()\nattempt\neach x in items\nreturn x\nend\nfailed error\nreturn none\nend\nend\n",
        );
        let function = module
            .functions
            .iter()
            .find(|f| f.name == "f")
            .expect("function is lowered");
        let returns: Vec<usize> = function
            .instructions
            .iter()
            .enumerate()
            .filter(|(_, inst)| matches!(inst, CoreInst::Return { .. }))
            .map(|(index, _)| index)
            .collect();
        assert!(!returns.is_empty(), "the function returns");
        // Before the return inside the loop there must be a guard close and a handler pop,
        // so the frame does not leak machine-wide state.
        let loop_return = returns
            .iter()
            .copied()
            .find(|index| {
                function.instructions[..*index]
                    .iter()
                    .rev()
                    .take(4)
                    .any(|inst| matches!(inst, CoreInst::IterGuardEnd(_)))
            })
            .expect("return inside each closes the guard");
        let before: Vec<&CoreInst> = function.instructions[..loop_return]
            .iter()
            .rev()
            .take(4)
            .collect();
        assert!(
            before
                .iter()
                .any(|inst| matches!(inst, CoreInst::PopHandler(_)))
                || function.instructions[..loop_return]
                    .iter()
                    .rev()
                    .take(6)
                    .any(|inst| matches!(inst, CoreInst::PopHandler(_))),
            "return inside attempt pops the handler: {before:?}"
        );
    }
}
