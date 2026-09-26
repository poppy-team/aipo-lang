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

// =========================================================================
// Marco 2: Structured Control Flow Tests (if/elif/else, while, loop, repeat)
// =========================================================================

#[test]
fn test_if_else_statement() {
    let code = r#"
fn abs_val(x: Int) -> Int {
  if x < 0 {
    return -x
  } else {
    return x
  }
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let abs_fn = instance
        .get_typed_func::<i64, i64>(&mut store, "abs_val")
        .expect("exported function `abs_val` exists");

    assert_eq!(abs_fn.call(&mut store, -42).unwrap(), 42);
    assert_eq!(abs_fn.call(&mut store, 10).unwrap(), 10);
    assert_eq!(abs_fn.call(&mut store, 0).unwrap(), 0);
}

#[test]
fn test_if_elif_else_cascade() {
    let code = r#"
fn classify(x: Int) -> Int {
  if x > 100 {
    return 3
  } elif x > 10 {
    return 2
  } elif x > 0 {
    return 1
  } else {
    return 0
  }
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let classify_fn = instance
        .get_typed_func::<i64, i64>(&mut store, "classify")
        .expect("exported function `classify` exists");

    assert_eq!(classify_fn.call(&mut store, 500).unwrap(), 3);
    assert_eq!(classify_fn.call(&mut store, 50).unwrap(), 2);
    assert_eq!(classify_fn.call(&mut store, 5).unwrap(), 1);
    assert_eq!(classify_fn.call(&mut store, -10).unwrap(), 0);
    assert_eq!(classify_fn.call(&mut store, 0).unwrap(), 0);
}

#[test]
fn test_inline_conditional_expression() {
    let code = r#"
fn max_val(a: Int, b: Int) -> Int {
  return if a > b then a else b
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let max_fn = instance
        .get_typed_func::<(i64, i64), i64>(&mut store, "max_val")
        .expect("exported function `max_val` exists");

    assert_eq!(max_fn.call(&mut store, (15, 27)).unwrap(), 27);
    assert_eq!(max_fn.call(&mut store, (100, 20)).unwrap(), 100);
}

#[test]
fn test_while_loop_sum() {
    let code = r#"
fn sum_to_n(n: Int) -> Int {
  var total = 0
  var i = 1
  while i <= n {
    total = total + i
    i = i + 1
  }
  return total
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let sum_fn = instance
        .get_typed_func::<i64, i64>(&mut store, "sum_to_n")
        .expect("exported function `sum_to_n` exists");

    assert_eq!(sum_fn.call(&mut store, 10).unwrap(), 55);
    assert_eq!(sum_fn.call(&mut store, 0).unwrap(), 0);
}

#[test]
fn test_while_loop_break() {
    let code = r#"
fn early_break(limit: Int) -> Int {
  var count = 0
  var i = 0
  while i < 100 {
    if i == limit {
      break
    }
    count = count + 1
    i = i + 1
  }
  return count
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let break_fn = instance
        .get_typed_func::<i64, i64>(&mut store, "early_break")
        .expect("exported function `early_break` exists");

    assert_eq!(break_fn.call(&mut store, 15).unwrap(), 15);
    assert_eq!(break_fn.call(&mut store, 0).unwrap(), 0);
}

#[test]
fn test_while_loop_continue() {
    let code = r#"
fn sum_odd_numbers(n: Int) -> Int {
  var sum = 0
  var i = 0
  while i < n {
    i = i + 1
    if i % 2 == 0 {
      continue
    }
    sum = sum + i
  }
  return sum
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let odd_fn = instance
        .get_typed_func::<i64, i64>(&mut store, "sum_odd_numbers")
        .expect("exported function `sum_odd_numbers` exists");

    // n = 5: i = 1 (odd, sum=1), 2 (skip), 3 (odd, sum=4), 4 (skip), 5 (odd, sum=9)
    assert_eq!(odd_fn.call(&mut store, 5).unwrap(), 9);
}

#[test]
fn test_unconditional_loop_break() {
    let code = r#"
fn loop_with_break() -> Int {
  var x = 0
  loop {
    x = x + 1
    if x == 10 {
      break
    }
  }
  return x
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let loop_fn = instance
        .get_typed_func::<(), i64>(&mut store, "loop_with_break")
        .expect("exported function `loop_with_break` exists");

    assert_eq!(loop_fn.call(&mut store, ()).unwrap(), 10);
}

#[test]
fn test_repeat_loop_with_index() {
    let code = r#"
fn repeat_indexed(n: Int) -> Int {
  var sum = 0
  repeat n as i {
    sum = sum + i
  }
  return sum
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let repeat_fn = instance
        .get_typed_func::<i64, i64>(&mut store, "repeat_indexed")
        .expect("exported function `repeat_indexed` exists");

    // n = 4: i = 0, 1, 2, 3 -> sum = 6
    assert_eq!(repeat_fn.call(&mut store, 4).unwrap(), 6);
}

#[test]
fn test_repeat_loop_without_index() {
    let code = r#"
fn repeat_simple(n: Int) -> Int {
  var count = 0
  repeat n {
    count = count + 2
  }
  return count
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let repeat_fn = instance
        .get_typed_func::<i64, i64>(&mut store, "repeat_simple")
        .expect("exported function `repeat_simple` exists");

    assert_eq!(repeat_fn.call(&mut store, 5).unwrap(), 10);
    assert_eq!(repeat_fn.call(&mut store, 0).unwrap(), 0);
}

#[test]
fn test_repeat_loop_with_continue() {
    let code = r#"
fn repeat_continue(n: Int) -> Int {
  var sum = 0
  repeat n as i {
    if i % 2 == 0 {
      continue
    }
    sum = sum + i
  }
  return sum
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let repeat_fn = instance
        .get_typed_func::<i64, i64>(&mut store, "repeat_continue")
        .expect("exported function `repeat_continue` exists");

    // n = 6: i in 0..5, odds are 1, 3, 5 -> sum = 9
    assert_eq!(repeat_fn.call(&mut store, 6).unwrap(), 9);
}

#[test]
fn test_nested_loops_matrix() {
    let code = r#"
fn nested_matrix(rows: Int, cols: Int) -> Int {
  var count = 0
  var r = 0
  while r < rows {
    var c = 0
    while c < cols {
      c = c + 1
      if c == 2 {
        continue
      }
      count = count + 1
    }
    r = r + 1
  }
  return count
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let matrix_fn = instance
        .get_typed_func::<(i64, i64), i64>(&mut store, "nested_matrix")
        .expect("exported function `nested_matrix` exists");

    // rows=3, cols=3: each row skips c=2, runs c=1 and c=3 (2 iterations per row) => 6
    assert_eq!(matrix_fn.call(&mut store, (3, 3)).unwrap(), 6);
}

#[test]
fn test_nested_loops_inner_break() {
    let code = r#"
fn nested_break(rows: Int, cols: Int) -> Int {
  var count = 0
  var r = 0
  while r < rows {
    var c = 0
    while c < cols {
      if c == 2 {
        break
      }
      count = count + 1
      c = c + 1
    }
    r = r + 1
  }
  return count
}
"#;
    let (mut store, instance) = instantiate_aipo(code);
    let break_fn = instance
        .get_typed_func::<(i64, i64), i64>(&mut store, "nested_break")
        .expect("exported function `nested_break` exists");

    // rows=3, cols=5: inner loop breaks at c=2 (runs c=0, 1 -> 2 per row) => 6
    assert_eq!(break_fn.call(&mut store, (3, 5)).unwrap(), 6);
}

#[test]
fn test_error_break_outside_loop() {
    let source = Source::new(SourceId::next(), "test.aipo", "fn bad() { break }");
    let (ast, diags) = parse(&source);
    assert!(diags.is_empty());
    let hir = lower(ast);
    let err = compile_hir(&hir).unwrap_err();
    assert!(matches!(
        err,
        aipo_wasm::WasmCompileError::UnsupportedStmt { ref message, .. } if message.contains("break outside of loop")
    ));
}

#[test]
fn test_error_continue_outside_loop() {
    let source = Source::new(SourceId::next(), "test.aipo", "fn bad() { continue }");
    let (ast, diags) = parse(&source);
    assert!(diags.is_empty());
    let hir = lower(ast);
    let err = compile_hir(&hir).unwrap_err();
    assert!(matches!(
        err,
        aipo_wasm::WasmCompileError::UnsupportedStmt { ref message, .. } if message.contains("continue outside of loop")
    ));
}
