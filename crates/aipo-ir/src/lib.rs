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
