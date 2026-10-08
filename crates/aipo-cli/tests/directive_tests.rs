//! End-to-end tests for `#!` compiler directives.
//!
//! `#!` is an annotation, not a macro: it never introduces code, never takes an expression,
//! and never expands. A directive attaches to the item that follows it and carries a fact
//! the compiler reads (`#!test`), or an annotation the toolchain reports (`#!todo`,
//! `#!deprecated`).

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
fn test_directive_does_not_break_the_following_function() {
    let temp = std::env::temp_dir().join("aipo_dir_basic.aipo");
    let source = r#"
#!todo
fn soma(a, b) {
    return a + b
}

io.println(String(soma(2, 3)))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "5");
}

#[test]
fn test_hash_bang_is_a_directive_and_hash_alone_stays_a_comment() {
    let temp = std::env::temp_dir().join("aipo_dir_hash.aipo");
    let source = r#"
# this is a plain comment and must not be read as a directive
#!todo
fn f() {
    return 1
}

io.println(String(f()))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "1");
}

#[test]
fn test_deprecated_directive_with_message_parses() {
    let temp = std::env::temp_dir().join("aipo_dir_deprecated.aipo");
    let source = r#"
#!deprecated("use soma_v2")
fn soma_v1(a, b) {
    return a + b
}

io.println(String(soma_v1(1, 1)))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "2");
}

#[test]
fn test_directive_on_struct_parses() {
    let temp = std::env::temp_dir().join("aipo_dir_struct.aipo");
    let source = r#"
#!todo
struct Ponto {
    x
}

let p = Ponto{x = 7}
io.println(String(p.x))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "7");
}

#[test]
fn test_directive_without_following_item_is_rejected() {
    let temp = std::env::temp_dir().join("aipo_dir_dangling.aipo");
    let source = r#"
#!todo
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, _stdout, stderr) = check_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_ne!(code, 0, "a directive with no item must be an error");
    assert!(
        stderr.contains("trailing directive"),
        "must report the dangling directive, got: {stderr}"
    );
}

#[test]
fn test_directive_before_statement_is_rejected() {
    let temp = std::env::temp_dir().join("aipo_dir_stmt.aipo");
    let source = r#"
#!todo
io.println("x")
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, _stdout, stderr) = check_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_ne!(code, 0, "a directive on a statement must be an error");
    assert!(
        stderr.contains("only be attached to items"),
        "must report the misplaced directive, got: {stderr}"
    );
}

#[test]
fn test_multiple_directives_on_one_item() {
    let temp = std::env::temp_dir().join("aipo_dir_multi.aipo");
    let source = r#"
#!todo
#!deprecated("use outra")
fn f(a) {
    return a * 2
}

io.println(String(f(21)))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "42");
}

#[test]
fn test_satisfies_directive_verifies_interface_success() {
    let temp = std::env::temp_dir().join("aipo_dir_satisfies_ok.aipo");
    let source = r#"
interface Desenhável {
    desenhar(self) -> String
}

#!satisfies Desenhável
struct Círculo {
    raio: Int
}

Círculo:desenhar(self) -> String {
    return "círculo de raio " + String(self.raio)
}

let c = Círculo{ raio: 5 }
io.println(c.desenhar())
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "círculo de raio 5");
}

#[test]
fn test_satisfies_directive_fails_when_interface_not_satisfied() {
    let temp = std::env::temp_dir().join("aipo_dir_satisfies_err.aipo");
    let source = r#"
interface Desenhável {
    desenhar(self) -> String
}

#!satisfies Desenhável
struct Círculo {
    raio: Int
}

io.println("x")
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, _stdout, stderr) = check_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_ne!(code, 0, "must reject unsatisfied interface promise");
    assert!(
        stderr.contains("missing method") || stderr.contains("AIPO_SEM_UNKNOWN_NAME"),
        "must report contract failure, got: {stderr}"
    );
}

#[test]
fn test_deprecated_call_emits_warning() {
    let temp = std::env::temp_dir().join("aipo_dir_deprecated_warn.aipo");
    let source = r#"
#!deprecated("use soma_v2")
fn soma_v1(a, b) {
    return a + b
}

let res = soma_v1(10, 20)
io.println(String(res))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);

    assert_eq!(code, 0, "deprecated warning does not prevent execution");
    assert_eq!(stdout.trim(), "30");
    assert!(
        stderr.contains("AIPO_SEM_DEPRECATED") || stderr.contains("deprecated"),
        "must emit deprecation warning, got: {stderr}"
    );
    assert!(
        stderr.contains("use soma_v2"),
        "must include deprecation message, got: {stderr}"
    );
}

#[test]
fn test_todo_directive_emits_warning() {
    let temp = std::env::temp_dir().join("aipo_dir_todo_warn.aipo");
    let source = r#"
#!todo
fn pendente() {
}

io.println("ok")
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, _stdout, stderr) = check_program(&temp);
    let _ = std::fs::remove_file(&temp);

    assert_eq!(code, 0, "todo is a warning, not a fatal error");
    assert!(
        stderr.contains("AIPO_SEM_TODO") || stderr.contains("marked with #!todo"),
        "must emit todo warning, got: {stderr}"
    );
}
