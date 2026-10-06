//! `aipo-sema` implements semantic analysis, lexical scope resolution,
//! mutability path checking, and contract verification for Aipo.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod analyzer;
pub mod prelude;
pub mod symbol;

pub(crate) mod flow;

pub use analyzer::{SemanticAnalyzer, SemanticFacts};
pub use prelude::{HostFunction, HostParameter, PreludeSurface};
pub use symbol::{MethodSignature, Mutability, Scope, ScopeTree, Symbol, SymbolKind};

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

    /// Canon allows one `await` in statement, initializer and return position; a nested
    /// one cannot suspend the enclosing path, so it is reported instead of guessed.
    #[test]
    fn test_await_in_subexpression_is_reported() {
        let code = "async fn worker()\n  return 21\nend\n\nfn consume(value)\n  return value\nend\n\nfn caller()\n  let t = worker()\n  return consume(await t)\nend";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(
            diags[0].code,
            DiagnosticCode::AIPO_SEM_AWAIT_IN_SUBEXPRESSION
        );
    }

    /// A `Task` that no statement observes was started and can never be joined, which is
    /// canon's forgotten-task diagnostic (never silent).
    #[test]
    fn test_forgotten_task_is_reported() {
        let code = "async fn worker()\n  return 1\nend\n\nfn caller()\n  worker()\nend";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_FORGOTTEN_TASK);
    }

    /// Binding the handle instead of discarding it is the canonical way to keep the task
    /// observable, so it must stay silent.
    #[test]
    fn test_bound_task_is_not_reported() {
        let code = "async fn worker()\n  return 1\nend\n\nfn caller()\n  let t = worker()\n  return t\nend";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    /// `await do` is already a sequential-await block: an inner one adds no suspension
    /// point, so canon asks for a diagnostic.
    #[test]
    fn test_nested_await_do_is_reported() {
        let code = "async fn one()\n  return 1\nend\n\nasync fn outer()\n  await do\n    await do\n      let value = await one()\n    end\n  end\nend";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_NESTED_AWAIT_DO);
    }

    /// The single-block form is the sugar canon sanctions, so it must analyze cleanly.
    #[test]
    fn test_await_do_accepts_sequential_awaits() {
        let code = "async fn one()\n  return 1\nend\n\nasync fn outer()\n  await do\n    let value = await one()\n    return value\n  end\nend";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    /// ADP-006 §G: a `Task` only comes from an `async fn` call or a combinator, so a literal
    /// operand is provable before execution and reported instead of left to the runtime fault.
    #[test]
    fn test_await_of_a_literal_is_reported_statically() {
        let diags = analyze_source("let value = await 41");
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(
            diags[0].code,
            DiagnosticCode::AIPO_SEM_CONTRACT_VIOLATION_STATIC
        );
    }

    /// Awaiting a handle from an `async fn` call is the sanctioned shape and stays silent.
    #[test]
    fn test_await_of_a_call_is_not_reported() {
        let code =
            "async fn worker()\n  return 1\nend\n\nfn caller()\n  return await worker()\nend";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    #[test]
    fn test_assignment_await_not_reported() {
        let code = "async fn worker()\n  return 1\nend\n\nfn caller()\n  var x = 0\n  x = await worker()\nend";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    #[test]
    fn test_immutable_receiver_mutation() {
        let code = "struct Point\n  x\n  y\nend\n\nimpl Point\n  fn set_x(self, val)\n    self.x = val\n  end\nend";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_READONLY_MUTATION);
    }

    #[test]
    fn test_mutable_path_mutation_allowed() {
        let code = "var items = [1, 2, 3]\nitems[0] = 10";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    #[test]
    fn test_fixed_field_reassignment_reported() {
        let code = "struct User {\n  fixed id\n  name\n}\n\nimpl User {\n  fn change_id(var self, new_id) {\n    self.id = new_id\n  }\n}";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_FIXED_REASSIGN);
    }

    #[test]
    fn test_async_method_forgotten_task_reported() {
        let code = "struct Service\nend\n\nimpl Service\n  async fn fetch(self)\n    return 42\n  end\nend\n\nlet s = Service{}\ns.fetch()";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_FORGOTTEN_TASK);
    }

    #[test]
    fn test_immutable_binding_field_mutation_allowed() {
        let code = "struct Point\n  x\n  y\nend\n\nlet p = Point{x = 1, y = 2}\np.x = 10";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    #[test]
    fn test_immutable_binding_index_mutation_allowed() {
        let code = "let items = [1, 2, 3]\nitems[0] = 99";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    #[test]
    fn test_fixed_field_instance_mutation_reported() {
        let code = "struct User\n  fixed id\n  name\nend\n\nvar u = User{id = 1, name = \"Alice\"}\nu.id = 2";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_FIXED_REASSIGN);
    }

    #[test]
    fn test_satisfy_return_type_mismatch() {
        let code = "interface Getter\n  fn get() -> Int\nend\n\nstruct Boxed\nend\n\nimpl Boxed\n  fn get() -> String\n    return \"hi\"\n  end\nend\n\nsatisfy Boxed: Getter";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(
            diags[0].code,
            DiagnosticCode::AIPO_SEM_CONTRACT_VIOLATION_STATIC
        );
    }

    #[test]
    fn test_satisfy_receiver_mutability_mismatch() {
        let code = "interface Mutator {\n  fn mutate(var self)\n}\n\nstruct State {}\n\nimpl State {\n  fn mutate(self) {}\n}\n\nsatisfy State: Mutator";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(
            diags[0].code,
            DiagnosticCode::AIPO_SEM_CONTRACT_VIOLATION_STATIC
        );
    }

    #[test]
    fn test_satisfy_async_mismatch() {
        let code = "interface Fetcher\n  async fn fetch()\nend\n\nstruct Service\nend\n\nimpl Service\n  fn fetch()\n    return 1\n  end\nend\n\nsatisfy Service: Fetcher";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(
            diags[0].code,
            DiagnosticCode::AIPO_SEM_CONTRACT_VIOLATION_STATIC
        );
    }

    #[test]
    fn test_destructuring_list_valid() {
        let code = "let [a, b] = [1, 2]";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    #[test]
    fn test_modern_struct_immutable_by_default() {
        let code = "struct User {\n  id\n  var name\n}\n\nvar u = User{ id: 1, name: \"Alice\" }\nu.id = 2";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_FIXED_REASSIGN);
    }

    #[test]
    fn test_modern_interface_and_var_self() {
        let code = "interface Counter {\n  fn increment(var self)\n}\n\nstruct Ticker {\n  var count\n}\n\nimpl Ticker {\n  fn increment(var self) {\n    self.count += 1\n  }\n}\n\nsatisfy Ticker: Counter";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    // --- control-flow analysis ---

    /// A `return <value>` next to a bare `return` leaves the caller unable to know
    /// which shape it received, so it is reported instead of left to runtime.
    #[test]
    fn test_mixed_return_shapes_are_reported() {
        let code = "fn pick(flag: Bool) -> Int\n  if flag\n    return 1\n  end\n  return\nend";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_RETURN_VALUE_MISMATCH);
    }

    /// Every `return` carrying a value is the ordinary shape and must stay silent.
    #[test]
    fn test_uniform_value_returns_are_accepted() {
        let code = "fn pick(flag: Bool) -> Int\n  if flag\n    return 1\n  end\n  return 2\nend";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    /// Every bare `return` is consistent for a function with no value contract.
    #[test]
    fn test_uniform_void_returns_are_accepted() {
        let code = "fn stop(flag: Bool)\n  if flag\n    return\n  end\n  return\nend";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    /// A function that promises a value but can reach its end without producing
    /// one would hand the caller `none`, so the path is reported.
    #[test]
    fn test_declared_contract_with_falling_path_is_reported() {
        let code = "fn maybe(flag: Bool) -> Int\n  if flag\n    return 1\n  end\nend";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(
            diags[0].code,
            DiagnosticCode::AIPO_SEM_PATH_MISSING_RETURN_VALUE
        );
    }

    /// When every path returns a value, no path falls off the end and the check
    /// must stay silent.
    #[test]
    fn test_total_returns_are_not_reported_as_missing() {
        let code = "fn total(flag: Bool) -> Int\n  if flag\n    return 1\n  else\n    return 2\n  end\nend";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    /// `match` with an `else` that returns on every arm is total.
    #[test]
    fn test_match_with_total_arms_is_accepted() {
        let code = "fn classify(v: Int) -> Int\n  match v\n    when 1\n      return 10\n    else\n      return 0\n  end\nend";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    /// `match` without an `else` can fall out of the statement, so a value
    /// contract is not honoured on that path.
    #[test]
    fn test_match_without_else_can_fall_through() {
        let code = "fn classify(v: Int) -> Int\n  match v\n    when 1\n      return 10\n  end\nend";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(
            diags[0].code,
            DiagnosticCode::AIPO_SEM_PATH_MISSING_RETURN_VALUE
        );
    }

    /// `loop` without a reachable `break` never completes, so nothing falls off
    /// the end and the missing-value path must not be reported.
    #[test]
    fn test_infinite_loop_is_not_a_missing_value_path() {
        let code = "fn forever() -> Int\n  loop\n    return 1\n  end\nend";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    /// `fail` ends the path like `return`, so it satisfies a value contract.
    #[test]
    fn test_fail_ends_the_path() {
        let code = "fn checked(v: Int) -> Int\n  if v < 0\n    fail (\"negative\")\n  end\n  return v\nend";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    /// `break` exits only the loop, so the function can still fall off the end.
    #[test]
    fn test_break_does_not_end_the_function_path() {
        let code = "fn first(limit: Int) -> Int\n  var total = 0\n  var i = 0\n  loop\n    if i >= limit\n      break\n    end\n    total = total + i\n    i = i + 1\n  end\n  return total\nend";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    // --- non-Bool conditions ---

    /// An integer literal is provably not a `Bool`, so the condition is reported
    /// before execution rather than faulting at runtime.
    #[test]
    fn test_integer_literal_condition_is_reported() {
        let diags = analyze_source("if 42\n  let x = 1\nend");
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_NON_BOOL_CONDITION);
    }

    /// A `while` with a literal condition has the same problem.
    #[test]
    fn test_while_literal_condition_is_reported() {
        let diags = analyze_source("while 0\n  break\nend");
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_NON_BOOL_CONDITION);
    }

    /// `and` and `or` short-circuit on a `Bool`; a literal operand is reported.
    #[test]
    fn test_logical_operator_literal_operands_are_reported() {
        let diags = analyze_source("let flag = 1 and 2");
        assert_eq!(diags.len(), 2, "found: {diags:?}");
        assert!(diags
            .iter()
            .all(|d| d.code == DiagnosticCode::AIPO_SEM_NON_BOOL_CONDITION));
    }

    /// `not` also requires a `Bool`.
    #[test]
    fn test_not_of_literal_is_reported() {
        let diags = analyze_source("let flag = not 7");
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_NON_BOOL_CONDITION);
    }

    /// A `Bool` literal is the canonical condition and must stay silent.
    #[test]
    fn test_bool_literal_condition_is_accepted() {
        let diags = analyze_source("if true\n  let x = 1\nend");
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    /// A condition whose type is not statically known is left to the runtime,
    /// so a dynamic program keeps working.
    #[test]
    fn test_dynamic_condition_is_accepted() {
        let diags = analyze_source("var ready = 1\nif ready\n  let x = 1\nend");
        assert!(diags.is_empty(), "found: {diags:?}");
    }

    /// The diagnostic carries a replacement, so an editor can offer a quick fix.
    #[test]
    fn test_non_bool_condition_carries_a_suggestion() {
        let diags = analyze_source("if 42\n  let x = 1\nend");
        assert_eq!(diags[0].suggestions.len(), 1, "found: {diags:?}");
        assert_eq!(diags[0].suggestions[0].replacement, "42 != 0");
    }

    // --- struct construction ---

    /// A field the struct does not declare is a typo, caught at the call site.
    #[test]
    fn test_unknown_struct_field_is_reported() {
        let code = "struct Point\n  x\n  y\nend\n\nlet p = Point{x: 1, z: 3}";
        let diags = analyze_source(code);
        assert_eq!(diags.len(), 1, "found: {diags:?}");
        assert_eq!(diags[0].code, DiagnosticCode::AIPO_SEM_UNKNOWN_NAME);
    }

    /// A declared field supplied exactly once is the ordinary shape.
    #[test]
    fn test_struct_with_all_fields_is_accepted() {
        let code = "struct Point\n  x\n  y\nend\n\nlet p = Point{x: 1, y: 2}";
        let diags = analyze_source(code);
        assert!(diags.is_empty(), "found: {diags:?}");
    }
}
