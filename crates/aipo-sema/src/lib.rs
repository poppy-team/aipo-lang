//! `aipo-sema` implements semantic analysis, lexical scope resolution,
//! mutability path checking, and contract verification for Aipo.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod analyzer;
pub mod prelude;
pub mod symbol;

pub use analyzer::{SemanticAnalyzer, SemanticFacts};
pub use prelude::PreludeSurface;
pub use symbol::{Mutability, Scope, ScopeTree, Symbol, SymbolKind};

use aipo_diagnostics::Diagnostic;
use aipo_hir::HirProgram;
use aipo_source::Source;

/// Runs semantic analysis on an `HirProgram` with only the language's own surface.
#[must_use]
pub fn check(source: &Source, program: &HirProgram) -> (SemanticFacts, Vec<Diagnostic>) {
    let analyzer = SemanticAnalyzer::new(source);
    analyzer.analyze(program)
}

/// Runs semantic analysis with an explicit global surface.
///
/// Embedders that register the standard library pass the surface derived from their
/// registration, so semantic analysis accepts exactly the names the VM can execute.
#[must_use]
pub fn check_with_prelude(
    source: &Source,
    program: &HirProgram,
    surface: &PreludeSurface,
) -> (SemanticFacts, Vec<Diagnostic>) {
    let analyzer = SemanticAnalyzer::with_prelude(source, surface);
    analyzer.analyze(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aipo_diagnostics::DiagnosticCode;
    use aipo_hir::lower;
    use aipo_source::{Source, SourceId};
    use aipo_syntax::parse;

    fn analyze_source(text: &str) -> Vec<Diagnostic> {
        let src = Source::new(SourceId::next(), "test.aipo", text);
        let (ast, diags) = parse(&src);
        assert!(diags.is_empty(), "parse diags: {:?}", diags);
        let hir = lower(ast);
        let (_, sema_diags) = check(&src, &hir);
        sema_diags
    }

    #[test]
    fn test_valid_program_passes() {
        let diags = analyze_source("let x = 1\nvar y = x + 2\ny = 5");
        assert!(diags.is_empty(), "expected 0 diags, found: {:?}", diags);
    }

    #[test]
    fn test_unknown_variable() {
        let diags = analyze_source("let x = unknown_name + 1");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_UNKNOWN_NAME);
    }

    #[test]
    fn test_redeclaration_in_same_scope() {
        let diags = analyze_source("let x = 1\nlet x = 2");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_REDECLARED_IN_SCOPE);
    }

    #[test]
    fn test_shadowing_in_nested_scope() {
        let diags = analyze_source("let x = 1\nif true then\n  let x = 2\nend");
        assert!(
            diags.is_empty(),
            "expected shadowing to succeed, found: {:?}",
            diags
        );
    }

    #[test]
    fn test_immutable_mutation() {
        let diags = analyze_source("let x = 10\nx = 20");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_READONLY_MUTATION);
    }

    #[test]
    fn test_arity_mismatch() {
        let diags = analyze_source("fn add(a, b)\n  return a + b\nend\nadd(1)");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_ARITY_MISMATCH);
    }

    #[test]
    fn test_satisfy_interface_success() {
        let code = "interface Greeter\n  fn greet(name)\nend\n\nstruct Bot\nend\n\nimpl Bot\n  fn greet(name)\n    return name\n  end\nend\n\nsatisfy Bot: Greeter";
        let diags = analyze_source(code);
        assert!(
            diags.is_empty(),
            "expected valid satisfy, found: {:?}",
            diags
        );
    }

    #[test]
    fn test_satisfy_missing_method() {
        let code =
            "interface Greeter\n  fn greet(name)\nend\n\nstruct Bot\nend\n\nsatisfy Bot: Greeter";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_UNKNOWN_NAME);
    }
}
