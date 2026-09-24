//! `aipo-syntax` implements the parser, Pratt expression parser,
//! syntax validation, and error recovery for the Aipo programming language.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod depth;
pub mod parser;
pub mod shift;

pub use depth::MAX_AST_DEPTH;
pub use parser::Parser;

use aipo_ast::Program;
use aipo_diagnostics::{Diagnostic, DiagnosticCode};
use aipo_lexer::Lexer;
use aipo_source::Source;

/// Parses an Aipo source file into a `Program` AST and a collection of diagnostics.
///
/// After parsing, the AST depth is measured iteratively (see [`depth`]): a tree
/// deeper than [`MAX_AST_DEPTH`] would overflow downstream recursive walkers,
/// so it is reported as `AIPO_PARSE_NESTING_TOO_DEEP` here instead.
#[must_use]
pub fn parse(source: &Source) -> (Program, Vec<Diagnostic>) {
    let lexer = Lexer::new(source);
    let (tokens, mut diagnostics) = lexer.tokenize();
    let parser = Parser::new(source, tokens);
    let (program, mut parse_diagnostics) = parser.parse();
    diagnostics.append(&mut parse_diagnostics);
    if let Some(span) = depth::over_limit_span(&program) {
        diagnostics.push(
            Diagnostic::error(
                DiagnosticCode::AIPO_PARSE_NESTING_TOO_DEEP,
                "expression tree is too deep (limit documented in ADP-005)",
            )
            .with_primary_span(source, span),
        );
    }
    (program, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aipo_source::{SourceId, SourceSpan};

    fn parse_import(text: &str) -> aipo_ast::ImportDecl {
        let src = Source::new(SourceId::next(), "test.aipo", text);
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        prog.items
            .into_iter()
            .find_map(|item| match item {
                aipo_ast::Item::Import(import) => Some(import),
                _ => None,
            })
            .expect("expected an import declaration")
    }

    #[test]
    fn test_parse_simple_program() {
        let src = Source::new(SourceId::next(), "test.aipo", "let x = 42\nvar y = x + 10");
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        assert_eq!(prog.statements.len(), 2);
    }

    #[test]
    fn test_parse_qualified_import_path() {
        let import = parse_import("import acme.http");
        assert_eq!(
            import
                .module_name
                .iter()
                .map(|segment| segment.name.as_str())
                .collect::<Vec<_>>(),
            vec!["acme", "http"]
        );
        assert_eq!(import.span, SourceSpan::new(0, "import acme.http".len()));
    }

    #[test]
    fn test_parse_qualified_import_alias() {
        let import = parse_import("import acme.http as h");
        assert_eq!(
            import.alias.as_ref().map(|alias| alias.name.as_str()),
            Some("h")
        );
    }

    #[test]
    fn test_parse_qualified_import_selective_names() {
        let import = parse_import("import acme.http: name, other");
        assert_eq!(
            import
                .names
                .iter()
                .map(|name| name.name.as_str())
                .collect::<Vec<_>>(),
            vec!["name", "other"]
        );
    }

    #[test]
    fn test_parse_unqualified_import_remains_supported() {
        let import = parse_import("import math");
        assert_eq!(
            import
                .module_name
                .iter()
                .map(|segment| segment.name.as_str())
                .collect::<Vec<_>>(),
            vec!["math"]
        );
    }

    #[test]
    fn test_parse_fn_item() {
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "fn add(a, b) -> Int\n  return a + b\nend",
        );
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        assert_eq!(prog.items.len(), 1);
    }

    #[test]
    fn test_parse_pipeline() {
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "let val = [1, 2, 3] |> filter(is_even) |> map(square)",
        );
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn test_parse_struct_and_impl() {
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "struct Point\n  fixed x\n  var y\nend\n\nimpl Point\n  fn move_by(dx, dy)\n    self.x = self.x + dx\n  end\nend",
        );
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        assert_eq!(prog.items.len(), 2);
    }

    #[test]
    fn test_parse_trailing_block() {
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "items.each() do item\n  print(item)\nend",
        );
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn test_parse_match_statement() {
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "match status\n  when 200 then \"ok\"\n  when 404, 500 then \"err\"\n  else \"unknown\"\nend",
        );
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn test_parse_if_elif_else() {
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "if x > 10 then\n  y = 1\nelif x == 10 then\n  y = 0\nelse\n  y = -1\nend",
        );
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn test_parse_loops() {
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "while x > 0\n  x = x - 1\nend\nrepeat 5 as i\n  print(i)\nend\neach item in list\n  print(item)\nend",
        );
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        assert_eq!(prog.statements.len(), 3);
    }

    #[test]
    fn test_parse_attempt_failed() {
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "attempt\n  risky_call()\nfailed err\n  handle(err)\nend",
        );
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        assert_eq!(prog.statements.len(), 1);
    }

    #[test]
    fn test_parse_interface_and_satisfy() {
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "interface Printable\n  fn print()\nend\n\nsatisfy Point: Printable",
        );
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        assert_eq!(prog.items.len(), 2);
    }

    #[test]
    fn test_parse_precedence() {
        use aipo_ast::{BinaryOp, Expr};
        let src = Source::new(SourceId::next(), "test.aipo", "let res = 1 + 2 * 3");
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        if let aipo_ast::Stmt::Let(_, expr, _) = &prog.statements[0] {
            if let Expr::Binary(op, _left, right, _) = expr {
                assert_eq!(*op, BinaryOp::Add);
                if let Expr::Binary(inner_op, _, _, _) = &**right {
                    assert_eq!(*inner_op, BinaryOp::Mul);
                } else {
                    panic!("expected inner binary Mul");
                }
            } else {
                panic!("expected outer binary Add");
            }
        }
    }

    #[test]
    fn test_parse_destructuring() {
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "let [head, tail] = numbers\nlet {x, y} = coords",
        );
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        assert_eq!(prog.statements.len(), 2);
    }

    #[test]
    fn test_parse_recovery_on_error() {
        let src = Source::new(SourceId::next(), "test.aipo", "let x = \nlet y = 10");
        let (prog, diags) = parse(&src);
        assert!(!diags.is_empty(), "expected diagnostics for missing expr");
        assert_eq!(
            prog.statements.len(),
            1,
            "expected recovery to parse second statement"
        );
    }

    #[test]
    fn test_elided_comparison_continuation() {
        use aipo_ast::{BinaryOp, Expr};
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "let ok = val >= 0 and <= 100",
        );
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        assert_eq!(prog.statements.len(), 1);

        if let aipo_ast::Stmt::Let(_, expr, _) = &prog.statements[0] {
            if let Expr::Binary(op, left, right, _) = expr {
                assert_eq!(*op, BinaryOp::And);
                assert!(matches!(
                    &**left,
                    Expr::Binary(BinaryOp::GreaterEqual, _, _, _)
                ));
                assert!(matches!(
                    &**right,
                    Expr::Binary(BinaryOp::LessEqual, _, _, _)
                ));
            } else {
                panic!("expected Binary and");
            }
        } else {
            panic!("expected let statement");
        }
    }

    #[test]
    fn test_multi_elided_comparison_continuation() {
        use aipo_ast::{BinaryOp, Expr};
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "let ok = val is Int and >= 0 and <= 100",
        );
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        assert_eq!(prog.statements.len(), 1);

        if let aipo_ast::Stmt::Let(_, expr, _) = &prog.statements[0] {
            if let Expr::Binary(op, left, right, _) = expr {
                assert_eq!(*op, BinaryOp::And);
                assert!(matches!(
                    &**right,
                    Expr::Binary(BinaryOp::LessEqual, _, _, _)
                ));
                if let Expr::Binary(inner_op, inner_l, inner_r, _) = &**left {
                    assert_eq!(*inner_op, BinaryOp::And);
                    assert!(matches!(&**inner_l, Expr::Binary(BinaryOp::Is, _, _, _)));
                    assert!(matches!(
                        &**inner_r,
                        Expr::Binary(BinaryOp::GreaterEqual, _, _, _)
                    ));
                } else {
                    panic!("expected nested Binary and");
                }
            } else {
                panic!("expected Binary and");
            }
        } else {
            panic!("expected let statement");
        }
    }

    #[test]
    fn test_chained_comparison_reports_diagnostic() {
        use aipo_diagnostics::DiagnosticCode;
        let src = Source::new(SourceId::next(), "test.aipo", "let bad = 1 < x < 10");
        let (_prog, diags) = parse(&src);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_PARSE_UNEXPECTED_TOKEN);
    }

    #[test]
    fn test_short_lambda_single_param() {
        use aipo_ast::{Expr, Stmt};
        let src = Source::new(SourceId::next(), "test.aipo", "let f = x => x * 2");
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {diags:?}");
        assert_eq!(prog.statements.len(), 1);
        if let Stmt::Let(_, Expr::Fn(f), _) = &prog.statements[0] {
            assert_eq!(f.params.len(), 1);
            assert_eq!(f.params[0].name.name, "x");
            assert_eq!(f.body.len(), 1);
            assert!(matches!(&f.body[0], Stmt::Return(Some(_), _)));
        } else {
            panic!("expected let f = Expr::Fn");
        }
    }

    #[test]
    fn test_short_lambda_multi_param() {
        use aipo_ast::{Expr, Stmt};
        let src = Source::new(SourceId::next(), "test.aipo", "let add = (a, b) => a + b");
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {diags:?}");
        assert_eq!(prog.statements.len(), 1);
        if let Stmt::Let(_, Expr::Fn(f), _) = &prog.statements[0] {
            assert_eq!(f.params.len(), 2);
            assert_eq!(f.params[0].name.name, "a");
            assert_eq!(f.params[1].name.name, "b");
            assert_eq!(f.body.len(), 1);
            assert!(matches!(&f.body[0], Stmt::Return(Some(_), _)));
        } else {
            panic!("expected let add = Expr::Fn");
        }
    }

    #[test]
    fn test_short_lambda_zero_param() {
        use aipo_ast::{Expr, Stmt};
        let src = Source::new(SourceId::next(), "test.aipo", "let get = () => 42");
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {diags:?}");
        assert_eq!(prog.statements.len(), 1);
        if let Stmt::Let(_, Expr::Fn(f), _) = &prog.statements[0] {
            assert_eq!(f.params.len(), 0);
            assert_eq!(f.body.len(), 1);
            assert!(matches!(&f.body[0], Stmt::Return(Some(_), _)));
        } else {
            panic!("expected let get = Expr::Fn");
        }
    }

    #[test]
    fn test_short_lambda_discard_param() {
        use aipo_ast::{Expr, Stmt};
        let src = Source::new(SourceId::next(), "test.aipo", "let sink = _ => 0");
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {diags:?}");
        assert_eq!(prog.statements.len(), 1);
        if let Stmt::Let(_, Expr::Fn(f), _) = &prog.statements[0] {
            assert_eq!(f.params.len(), 1);
            assert_eq!(f.params[0].name.name, "_");
            assert_eq!(f.body.len(), 1);
            assert!(matches!(&f.body[0], Stmt::Return(Some(_), _)));
        } else {
            panic!("expected let sink = Expr::Fn");
        }
    }

    #[test]
    fn test_short_lambda_discard_in_tuple() {
        use aipo_ast::{Expr, Stmt};
        let src = Source::new(SourceId::next(), "test.aipo", "let f = (a, _) => a");
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {diags:?}");
        assert_eq!(prog.statements.len(), 1);
        if let Stmt::Let(_, Expr::Fn(f), _) = &prog.statements[0] {
            assert_eq!(f.params.len(), 2);
            assert_eq!(f.params[0].name.name, "a");
            assert_eq!(f.params[1].name.name, "_");
            assert_eq!(f.body.len(), 1);
            assert!(matches!(&f.body[0], Stmt::Return(Some(_), _)));
        } else {
            panic!("expected let f = Expr::Fn");
        }
    }

    #[test]
    fn test_parse_inline_conditional_expression() {
        use aipo_ast::{Expr, Stmt};
        let src = Source::new(
            SourceId::next(),
            "test.aipo",
            "let x = if condition then 42 else 0",
        );
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {diags:?}");
        assert_eq!(prog.statements.len(), 1);
        if let Stmt::Let(_, Expr::If(cond, then_branch, else_branch, _), _) = &prog.statements[0] {
            assert!(matches!(**cond, Expr::Identifier(_)));
            assert!(matches!(**then_branch, Expr::Literal(_, _)));
            assert!(matches!(**else_branch, Expr::Literal(_, _)));
        } else {
            panic!("expected let x = Expr::If");
        }
    }

    #[test]
    fn test_parse_is_nullable() {
        use aipo_ast::{BinaryOp, Expr, Stmt};
        let src = Source::new(SourceId::next(), "test.aipo", "let ok = val is Int?");
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {diags:?}");
        assert_eq!(prog.statements.len(), 1);
        if let Stmt::Let(_, Expr::Binary(op, left, right, _), _) = &prog.statements[0] {
            assert_eq!(*op, BinaryOp::IsNullable);
            assert!(matches!(**left, Expr::Identifier(_)));
            assert!(matches!(**right, Expr::Identifier(_)));
        } else {
            panic!("expected BinaryOp::IsNullable");
        }
    }

    #[test]
    fn test_parse_multiple_subjects_is() {
        use aipo_ast::{BinaryOp, Expr, Stmt};
        let src = Source::new(SourceId::next(), "test.aipo", "let ok = a, b, c is Int");
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {diags:?}");
        assert_eq!(prog.statements.len(), 1);
        // Desugared: ((a is Int and b is Int) and c is Int)
        if let Stmt::Let(_, Expr::Binary(op, left, right, _), _) = &prog.statements[0] {
            assert_eq!(*op, BinaryOp::And);
            assert!(matches!(**right, Expr::Binary(BinaryOp::Is, _, _, _)));
            if let Expr::Binary(inner_op, inner_l, inner_r, _) = &**left {
                assert_eq!(*inner_op, BinaryOp::And);
                assert!(matches!(**inner_l, Expr::Binary(BinaryOp::Is, _, _, _)));
                assert!(matches!(**inner_r, Expr::Binary(BinaryOp::Is, _, _, _)));
            } else {
                panic!("expected nested Binary and");
            }
        } else {
            panic!("expected let ok = Expr::Binary");
        }
    }

    #[test]
    fn test_parse_multiple_subjects_is_nullable() {
        use aipo_ast::{BinaryOp, Expr, Stmt};
        let src = Source::new(SourceId::next(), "test.aipo", "let ok = a, b is String?");
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {diags:?}");
        assert_eq!(prog.statements.len(), 1);
        if let Stmt::Let(_, Expr::Binary(op, left, right, _), _) = &prog.statements[0] {
            assert_eq!(*op, BinaryOp::And);
            assert!(matches!(
                **left,
                Expr::Binary(BinaryOp::IsNullable, _, _, _)
            ));
            assert!(matches!(
                **right,
                Expr::Binary(BinaryOp::IsNullable, _, _, _)
            ));
        } else {
            panic!("expected let ok = Expr::Binary");
        }
    }

    #[test]
    fn test_is_not_produces_unexpected_token_diagnostic() {
        use aipo_diagnostics::DiagnosticCode;
        let src = Source::new(SourceId::next(), "test.aipo", "let bad = x is not Int");
        let (_prog, diags) = parse(&src);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_PARSE_UNEXPECTED_TOKEN);
        assert!(diags[0].message.contains("'is not' is not supported"));
    }
}
