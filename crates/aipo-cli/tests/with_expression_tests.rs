//! End-to-end tests for the `base with { field: value, ... }` functional update.
//!
//! Exercised through the CLI so the entire chain runs together: lexer, parser,
//! HIR, IR lowering (`CloneStruct` + `SetField`), bytecode emission and VM.

use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

fn run_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[derive(Clone)]
struct CaptureSink(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for CaptureSink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn run_program(path: &Path) -> (u8, String, String) {
    let _serialized = run_lock();
    let captured = Arc::new(Mutex::new(Vec::new()));
    aipo_cli::set_output_sink(Some(Box::new(CaptureSink(Arc::clone(&captured)))));
    let args: Vec<String> = vec!["run".into(), path.to_string_lossy().into()];
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = aipo_cli::run_with(&args, &mut out, &mut err);
    aipo_cli::set_output_sink(None);

    let program_output = {
        let buffer = captured
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        String::from_utf8_lossy(&buffer).into_owned()
    };
    (
        code,
        program_output,
        String::from_utf8_lossy(&err).into_owned(),
    )
}

fn check_program(path: &Path) -> (u8, String, String) {
    let _serialized = run_lock();
    let args: Vec<String> = vec!["check".into(), path.to_string_lossy().into()];
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = aipo_cli::run_with(&args, &mut out, &mut err);
    (
        code,
        String::from_utf8_lossy(&out).into_owned(),
        String::from_utf8_lossy(&err).into_owned(),
    )
}

#[test]
fn test_with_expression_basic_functional_update() {
    let temp = std::env::temp_dir().join("aipo_with_basic.aipo");
    let source = r#"
struct Point {
    var x
    var y
}

let p1 = Point{x = 1, y = 2}
let p2 = p1 with { y: 99 }

io.println(String(p1.x) + "," + String(p1.y))
io.println(String(p2.x) + "," + String(p2.y))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "1,2\n1,99");
}

#[test]
fn test_with_expression_multiple_overrides_and_chaining() {
    let temp = std::env::temp_dir().join("aipo_with_multi.aipo");
    let source = r#"
struct Point {
    var x
    var y
    var z
}

let p = Point{x = 1, y = 2, z = 3}
let updated = p with { x: 10, z: 30 }
io.println(String(updated.x) + "," + String(updated.y) + "," + String(updated.z))

let chained = p with { x: 5 } with { y: 6 } with { z: 7 }
io.println(String(chained.x) + "," + String(chained.y) + "," + String(chained.z))

# Order-independence: the block is a set of overrides.
let a = p with { x: 9, y: 8 }
let b = p with { y: 8, x: 9 }
io.println(String(a == b))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "10,2,30\n5,6,7\ntrue");
}

#[test]
fn test_with_expression_on_call_result() {
    let temp = std::env::temp_dir().join("aipo_with_call.aipo");
    let source = r#"
struct Point {
    var x
    var y
}

fn make_point(x, y) {
    return Point{x = x, y = y}
}

let p = make_point(3, 4) with { y: 40 }
io.println(String(p.x) + "," + String(p.y))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "3,40");
}

#[test]
fn test_with_expression_static_error_on_unknown_field() {
    let temp = std::env::temp_dir().join("aipo_with_err_unknown.aipo");
    let source = r#"
struct Point {
    var x
    var y
}

let p = Point{x = 1, y = 2} with { nope: 9 }
io.println(p)
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, _stdout, stderr) = check_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_ne!(code, 0, "must reject unknown field statically");
    assert!(
        stderr.contains("AIPO_SEM_UNKNOWN_NAME"),
        "must report AIPO_SEM_UNKNOWN_NAME, got: {stderr}"
    );
    assert!(
        stderr.contains("has no field 'nope'"),
        "must name the offending field, got: {stderr}"
    );
}

#[test]
fn test_with_expression_static_error_on_fixed_field() {
    let temp = std::env::temp_dir().join("aipo_with_err_fixed.aipo");
    let source = r#"
struct Account {
    fixed id
    balance
}

let a = Account{id = 1, balance = 100} with { id = 999 }
io.println(a)
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, _stdout, stderr) = check_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_ne!(code, 0, "must reject fixed field replacement statically");
    assert!(
        stderr.contains("AIPO_SEM_READONLY_MUTATION"),
        "must report AIPO_SEM_READONLY_MUTATION, got: {stderr}"
    );
}

#[test]
fn test_with_expression_runtime_fault_on_non_struct_base() {
    let temp = std::env::temp_dir().join("aipo_with_non_struct.aipo");
    let source = r#"
fn get_val() {
    return 42
}

let bad = get_val() with { y: 1 }
io.println(bad)
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, _stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_ne!(code, 0, "must fail at runtime on non-struct base");
    assert!(
        stderr.contains("AIPO_RT_TYPE_MISMATCH"),
        "must report type mismatch, got: {stderr}"
    );
}
