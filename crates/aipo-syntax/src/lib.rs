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
    use aipo_source::SourceId;

    #[test]
    fn test_parse_simple_program() {
        let src = Source::new(SourceId::next(), "test.aipo", "let x = 42\nvar y = x + 10");
        let (prog, diags) = parse(&src);
        assert!(diags.is_empty(), "diags: {:?}", diags);
        assert_eq!(prog.statements.len(), 2);
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
}
