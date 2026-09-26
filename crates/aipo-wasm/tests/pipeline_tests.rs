//! End-to-end integration tests for Aipo WebAssembly pipeline (Marco 1 / ADP-013).
//!
//! Verifies: Source (.aipo) -> Syntax -> HIR -> Wasm (.wasm) -> Wasmtime JIT Execution.

use aipo_hir::lower;
use aipo_source::{Source, SourceId};
use aipo_syntax::parse;
use aipo_wasm::compile_hir;
use wasmtime::{Engine, Instance, Module, Store};

/// Helper to compile Aipo source code into a live Wasmtime instance.
fn instantiate_aipo(source_code: &str) -> (Store<()>, Instance) {
    let source = Source::new(SourceId::next(), "test.aipo", source_code);
    let (ast, diags) = parse(&source);
    assert!(diags.is_empty(), "parse diagnostics: {diags:?}");

    let hir = lower(ast);
    let wasm_bytes = compile_hir(&hir).expect("compilation to Wasm succeeds");

    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("valid WebAssembly module");
    let mut store = Store::new(&engine, ());
    let instance =
        Instance::new(&mut store, &module, &[]).expect("WebAssembly instantiation succeeds");

    (store, instance)
}

#[test]
fn test_simple_arithmetic_function() {
    let code = r#"
fn add(a: Int, b: Int) -> Int {
  return a + b
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let add_fn = instance
        .get_typed_func::<(i64, i64), i64>(&mut store, "add")
        .expect("exported function `add` exists");

    assert_eq!(add_fn.call(&mut store, (15, 27)).unwrap(), 42);
    assert_eq!(add_fn.call(&mut store, (-10, 5)).unwrap(), -5);
}

#[test]
fn test_untyped_params_default_to_int() {
    let code = r#"
fn add_untyped(a, b) {
  return a + b
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let add_fn = instance
        .get_typed_func::<(i64, i64), i64>(&mut store, "add_untyped")
        .expect("exported function `add_untyped` exists");

    assert_eq!(add_fn.call(&mut store, (100, 250)).unwrap(), 350);
}

#[test]
fn test_compound_arithmetic_with_precedence() {
    let code = r#"
fn compute(x: Int, y: Int) -> Int {
  return (x * 2) + (y - 5)
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let compute_fn = instance
        .get_typed_func::<(i64, i64), i64>(&mut store, "compute")
        .expect("exported function `compute` exists");

    assert_eq!(compute_fn.call(&mut store, (10, 15)).unwrap(), 30);
    assert_eq!(compute_fn.call(&mut store, (0, 5)).unwrap(), 0);
}

#[test]
fn test_local_variables_let_var_and_reassignment() {
    let code = r#"
fn with_locals(x: Int) -> Int {
  let a = x * 2
  var b = a + 10
  b = b + 5
  return b
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let with_locals_fn = instance
        .get_typed_func::<i64, i64>(&mut store, "with_locals")
        .expect("exported function `with_locals` exists");

    // a = 7 * 2 = 14; b = 14 + 10 = 24; b = 24 + 5 = 29
    assert_eq!(with_locals_fn.call(&mut store, 7).unwrap(), 29);
}

#[test]
fn test_integer_division_and_modulo() {
    let code = r#"
fn div_mod(a: Int, b: Int) -> Int {
  let q = a // b
  let r = a % b
  return (q * 100) + r
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let div_mod_fn = instance
        .get_typed_func::<(i64, i64), i64>(&mut store, "div_mod")
        .expect("exported function `div_mod` exists");

    // 23 // 5 = 4; 23 % 5 = 3 => 4 * 100 + 3 = 403
    assert_eq!(div_mod_fn.call(&mut store, (23, 5)).unwrap(), 403);
}

#[test]
#[allow(clippy::approx_constant)]
fn test_float_arithmetic() {
    let code = r#"
fn circle_area(r: Float) -> Float {
  return 3.14159 * r * r
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let circle_area_fn = instance
        .get_typed_func::<f64, f64>(&mut store, "circle_area")
        .expect("exported function `circle_area` exists");

    let area = circle_area_fn.call(&mut store, 2.0).unwrap();
    let expected = 3.14159 * 4.0;
    assert!((area - expected).abs() < 1e-9);
}

#[test]
fn test_float_division_promotes_integers() {
    let code = r#"
fn half(x: Int) -> Float {
  return x / 2
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let half_fn = instance
        .get_typed_func::<i64, f64>(&mut store, "half")
        .expect("exported function `half` exists");

    assert_eq!(half_fn.call(&mut store, 7).unwrap(), 3.5);
    assert_eq!(half_fn.call(&mut store, 10).unwrap(), 5.0);
}

#[test]
fn test_unary_negation() {
    let code = r#"
fn negate_val(x: Int) -> Int {
  return -x
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let negate_fn = instance
        .get_typed_func::<i64, i64>(&mut store, "negate_val")
        .expect("exported function `negate_val` exists");

    assert_eq!(negate_fn.call(&mut store, 42).unwrap(), -42);
    assert_eq!(negate_fn.call(&mut store, -99).unwrap(), 99);
}

#[test]
fn test_inter_function_calls() {
    let code = r#"
fn square(n: Int) -> Int {
  return n * n
}

fn sum_of_squares(a: Int, b: Int) -> Int {
  return square(a) + square(b)
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let sum_fn = instance
        .get_typed_func::<(i64, i64), i64>(&mut store, "sum_of_squares")
        .expect("exported function `sum_of_squares` exists");

    // 3*3 + 4*4 = 9 + 16 = 25
    assert_eq!(sum_fn.call(&mut store, (3, 4)).unwrap(), 25);
}

#[test]
fn test_boolean_comparison() {
    let code = r#"
fn is_greater(a: Int, b: Int) -> Bool {
  return a > b
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let is_greater_fn = instance
        .get_typed_func::<(i64, i64), i32>(&mut store, "is_greater")
        .expect("exported function `is_greater` exists");

    assert_eq!(is_greater_fn.call(&mut store, (10, 5)).unwrap(), 1);
    assert_eq!(is_greater_fn.call(&mut store, (3, 8)).unwrap(), 0);
}

#[test]
fn test_top_level_entrypoint_script() {
    let code = r#"
let x = 10
let y = 20
return x + y
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let top_fn = instance
        .get_typed_func::<(), i64>(&mut store, "__top_level__")
        .expect("exported function `__top_level__` exists");

    assert_eq!(top_fn.call(&mut store, ()).unwrap(), 30);
}

#[test]
fn test_error_unknown_variable() {
    let source = Source::new(
        SourceId::next(),
        "test.aipo",
        "fn bad() -> Int { return undeclared_var + 1 }",
    );
    let (ast, diags) = parse(&source);
    assert!(diags.is_empty());
    let hir = lower(ast);
    let err = compile_hir(&hir).unwrap_err();
    assert!(
        matches!(err, aipo_wasm::WasmCompileError::UnknownVariable { name, .. } if name == "undeclared_var")
    );
}

#[test]
fn test_error_async_function_unsupported_in_marco_1() {
    let source = Source::new(
        SourceId::next(),
        "test.aipo",
        "async fn future_job() -> Int { return 42 }",
    );
    let (ast, diags) = parse(&source);
    assert!(diags.is_empty());
    let hir = lower(ast);
    let err = compile_hir(&hir).unwrap_err();
    assert!(matches!(
        err,
        aipo_wasm::WasmCompileError::UnsupportedItem { .. }
    ));
}
