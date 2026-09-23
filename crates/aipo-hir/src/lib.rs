//! `aipo-hir` defines the High-Level Intermediate Representation (HIR)
//! and early semantics-preserving lowering passes for the Aipo compiler.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod hir;
pub mod lower;

pub use hir::*;
pub use lower::LoweringContext;

use aipo_ast::Program;

/// Lowers an AST `Program` into a `HirProgram` desugaring syntactic extensions.
#[must_use]
pub fn lower(program: Program) -> HirProgram {
    let mut ctx = LoweringContext::new();
    ctx.lower_program(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aipo_source::{Source, SourceId};
    use aipo_syntax::parse;

    #[test]
    fn test_lower_trailing_block() {
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "items.each() do item\n  print(item)\nend",
        );
        let (ast, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);

        let hir = lower(ast);
        assert_eq!(hir.statements.len(), 1);
        if let HirStmt::Expr(HirExpr::Call(callee, args, _)) = &hir.statements[0] {
            assert!(matches!(&**callee, HirExpr::Dot(_, _, _)));
            assert_eq!(args.len(), 1, "expected trailing block as argument");
            assert!(matches!(&args[0].value, HirExpr::Fn(_)));
        } else {
            panic!("expected Call statement");
        }
    }

    #[test]
    fn test_lower_pipeline() {
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "let res = [1, 2] |> map(square)",
        );
        let (ast, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);

        let hir = lower(ast);
        assert_eq!(hir.statements.len(), 1);
        if let HirStmt::Let(_, expr, _) = &hir.statements[0] {
            if let HirExpr::Call(callee, args, _) = expr {
                if let HirExpr::Identifier(name, _) = &**callee {
                    assert_eq!(name, "map");
                } else {
                    panic!("expected map callee");
                }
                assert_eq!(args.len(), 2);
                assert!(matches!(&args[0].value, HirExpr::List(_, _)));
                if let HirExpr::Identifier(arg1, _) = &args[1].value {
                    assert_eq!(arg1, "square");
                } else {
                    panic!("expected square arg");
                }
            } else {
                panic!("expected Call expr");
            }
        }
    }

    #[test]
    fn test_lower_destructuring_list() {
        let src = Source::new(SourceId::next(), "test.aipo", "let [a, b] = get_pair()");
        let (ast, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);

        let hir = lower(ast);
        // Desugars into: let __temp_0 = get_pair(); let a = __temp_0[0]; let b = __temp_0[1]
        assert_eq!(hir.statements.len(), 3);
        assert!(
            matches!(&hir.statements[0], HirStmt::Let(temp, _, _) if temp.starts_with("__temp_"))
        );
        assert!(
            matches!(&hir.statements[1], HirStmt::Let(name, HirExpr::Index(_, _, _), _) if name == "a")
        );
        assert!(
            matches!(&hir.statements[2], HirStmt::Let(name, HirExpr::Index(_, _, _), _) if name == "b")
        );
    }

    #[test]
    fn test_lower_destructuring_struct() {
        let src = Source::new(SourceId::next(), "test.aipo", "let {x, y} = point");
        let (ast, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);

        let hir = lower(ast);
        // Desugars into: let __temp_0 = point; let x = __temp_0.x; let y = __temp_0.y
        assert_eq!(hir.statements.len(), 3);
        assert!(
            matches!(&hir.statements[0], HirStmt::Let(temp, _, _) if temp.starts_with("__temp_"))
        );
        assert!(
            matches!(&hir.statements[1], HirStmt::Let(name, HirExpr::Dot(_, member, _), _) if name == "x" && member == "x")
        );
        assert!(
            matches!(&hir.statements[2], HirStmt::Let(name, HirExpr::Dot(_, member, _), _) if name == "y" && member == "y")
        );
    }

    #[test]
    fn test_lower_invariant_hook_merged_span() {
        let code = "struct Pos\n  x\n  y\nend\n\nimpl Pos\n  invariant()\n    x >= 0\n    y <= 100\n  end\nend";
        let src = Source::new(SourceId::next(), "test.aipo", code);
        let (ast, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);

        let hir = lower(ast);
        let impl_item = hir
            .items
            .iter()
            .find_map(|item| match item {
                HirItem::Impl(i) => Some(i),
                _ => None,
            })
            .expect("expected impl block");

        let inv = impl_item.invariant.as_ref().expect("expected invariant");
        assert_eq!(inv.body.len(), 1);
        if let HirStmt::Return(Some(HirExpr::Binary(op, left, right, span)), _) = &inv.body[0] {
            assert_eq!(*op, aipo_ast::BinaryOp::And);
            assert_eq!(span.start, left.span().start);
            assert_eq!(span.end, right.span().end);
        } else {
            panic!("expected binary and return in invariant");
        }
    }
}
