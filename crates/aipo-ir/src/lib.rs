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
}
