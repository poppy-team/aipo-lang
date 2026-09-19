//! `aipo-ast` provides strongly typed Abstract Syntax Tree (AST) structures
//! for statements, expressions, and declarations in Aipo V1.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod ast;

pub use ast::*;

#[cfg(test)]
mod tests {
    use super::*;
    use aipo_source::SourceSpan;

    #[test]
    fn test_ident_and_expr() {
        let span = SourceSpan::new(0, 5);
        let ident = Ident::new("alpha".into(), span);
        assert_eq!(ident.name, "alpha");

        let expr = Expr::Identifier(ident.clone());
        assert_eq!(expr.span(), span);
    }
}
