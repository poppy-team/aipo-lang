//! End-to-end tests for `match` statements in the Wasm backend.

use aipo_hir::lower;
use aipo_source::{Source, SourceId};
use aipo_syntax::parse;
use aipo_wasm::compile_hir;
use wasmtime::{Engine, Instance, Module, Store};

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
fn test_match_on_int_arms() {
    let code = r#"
fn classify(v: Int) -> Int {
  match v {
    when 1
      return 100
    when 2
      return 200
    when 3
      return 300
    else
      return 0
  }
}
"#;
    let source = Source::new(SourceId::next(), "test.aipo", code);
    let (ast, diags) = parse(&source);
    assert!(diags.is_empty(), "parse diagnostics: {diags:?}");
    let hir = lower(ast);
    let wasm_bytes = compile_hir(&hir).expect("compilation to Wasm succeeds");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("valid module");
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).expect("instantiation");
    let f = instance
        .get_typed_func::<i64, i64>(&mut store, "classify")
        .expect("exported `classify`");

    assert_eq!(f.call(&mut store, 1).unwrap(), 100);
    assert_eq!(f.call(&mut store, 2).unwrap(), 200);
    assert_eq!(f.call(&mut store, 3).unwrap(), 300);
    assert_eq!(f.call(&mut store, 9).unwrap(), 0);
}

#[test]
fn test_match_with_multi_pattern_arm() {
    let code = r#"
fn bucket(v: Int) -> Int {
  match v {
    when 0, 1, 2
      return 10
    when 3, 4
      return 20
    else
      return 99
  }
}
"#;
    let source = Source::new(SourceId::next(), "test.aipo", code);
    let (ast, diags) = parse(&source);
    assert!(diags.is_empty(), "parse diagnostics: {diags:?}");
    let hir = lower(ast);
    let wasm_bytes = compile_hir(&hir).expect("compilation to Wasm succeeds");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("valid module");
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).expect("instantiation");
    let f = instance
        .get_typed_func::<i64, i64>(&mut store, "bucket")
        .expect("exported `bucket`");

    assert_eq!(f.call(&mut store, 0).unwrap(), 10);
    assert_eq!(f.call(&mut store, 2).unwrap(), 10);
    assert_eq!(f.call(&mut store, 3).unwrap(), 20);
    assert_eq!(f.call(&mut store, 4).unwrap(), 20);
    assert_eq!(f.call(&mut store, 7).unwrap(), 99);
}

#[test]
fn test_match_on_bool() {
    let code = r#"
fn describe(v: Bool) -> Int {
  match v {
    when true
      return 1
    else
      return 0
  }
}
"#;
    let source = Source::new(SourceId::next(), "test.aipo", code);
    let (ast, diags) = parse(&source);
    assert!(diags.is_empty(), "parse diagnostics: {diags:?}");
    let hir = lower(ast);
    let wasm_bytes = compile_hir(&hir).expect("compilation to Wasm succeeds");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("valid module");
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).expect("instantiation");
    let f = instance
        .get_typed_func::<i32, i64>(&mut store, "describe")
        .expect("exported `describe`");

    assert_eq!(f.call(&mut store, 1).unwrap(), 1);
    assert_eq!(f.call(&mut store, 0).unwrap(), 0);
}

#[test]
fn test_match_selects_first_matching_arm() {
    let code = r#"
fn first_wins(v: Int) -> Int {
  match v {
    when 5
      return 1
    when 5
      return 2
    else
      return 3
  }
}
"#;
    let source = Source::new(SourceId::next(), "test.aipo", code);
    let (ast, diags) = parse(&source);
    assert!(diags.is_empty(), "parse diagnostics: {diags:?}");
    let hir = lower(ast);
    let wasm_bytes = compile_hir(&hir).expect("compilation to Wasm succeeds");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("valid module");
    let mut store = Store::new(&engine, ());
    let instance = Instance::new(&mut store, &module, &[]).expect("instantiation");
    let f = instance
        .get_typed_func::<i64, i64>(&mut store, "first_wins")
        .expect("exported `first_wins`");

    assert_eq!(f.call(&mut store, 5).unwrap(), 1);
}
