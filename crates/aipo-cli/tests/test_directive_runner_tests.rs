//! Tests for `#!test` discovery and execution via `aipo test`.
//!
//! Validates:
//! - Direct function tests marked with `#!test`
//! - Explicit names with `#!test("description")`
//! - Tagged tests `#!test[tag]` and filtering via `--filter`
//! - Failure reporting when a test function triggers `fail`
//! - Test discovery in any `.aipo` file containing `#!test`.

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

fn run_test_cli(args: &[&str]) -> (u8, String, String) {
    let _serialized = run_lock();
    let captured = Arc::new(Mutex::new(Vec::new()));
    aipo_cli::set_output_sink(Some(Box::new(CaptureSink(Arc::clone(&captured)))));
    let str_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = aipo_cli::run_with(&str_args, &mut out, &mut err);
    aipo_cli::set_output_sink(None);

    let program_output = {
        let buffer = captured
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        String::from_utf8_lossy(&buffer).into_owned()
    };
    let combined_out = if out.is_empty() {
        program_output
    } else {
        String::from_utf8_lossy(&out).into_owned()
    };
    (
        code,
        combined_out,
        String::from_utf8_lossy(&err).into_owned(),
    )
}

#[test]
fn test_hash_bang_test_discovery_and_pass() {
    let temp = std::env::temp_dir().join("math_sample.aipo");
    let source = r#"
fn soma(a, b) {
    return a + b
}

#!test
fn teste_soma_simples() {
    let res = soma(2, 3)
    if res != 5 {
        fail "soma incorreta"
    }
}

#!test("soma com zero")
fn teste_zero() {
    if soma(10, 0) != 10 {
        fail "zero incorreto"
    }
}
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_test_cli(&["test", temp.to_str().unwrap()]);
    let _ = std::fs::remove_file(&temp);

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("teste_soma_simples ... ok"),
        "got: {stdout}"
    );
    assert!(stdout.contains("soma com zero ... ok"), "got: {stdout}");
    assert!(stdout.contains("2 passed"), "got: {stdout}");
}

#[test]
fn test_hash_bang_test_failure_reporting() {
    let temp = std::env::temp_dir().join("fail_sample.aipo");
    let source = r#"
#!test
fn deve_falhar() {
    fail "resultado inesperado"
}
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, _stderr) = run_test_cli(&["test", temp.to_str().unwrap()]);
    let _ = std::fs::remove_file(&temp);

    assert_ne!(code, 0, "test suite with failure must return non-zero");
    assert!(stdout.contains("deve_falhar ... FAILED"), "got: {stdout}");
    assert!(stdout.contains("resultado inesperado"), "got: {stdout}");
    assert!(stdout.contains("1 failed"), "got: {stdout}");
}

#[test]
fn test_hash_bang_test_filtering() {
    let temp = std::env::temp_dir().join("filter_sample.aipo");
    let source = r#"
#!test[rapido]
fn teste_rapido() {
}

#!test[lento]
fn teste_lento() {
}
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) =
        run_test_cli(&["test", temp.to_str().unwrap(), "--filter", "rapido"]);
    let _ = std::fs::remove_file(&temp);

    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("teste_rapido[rapido] ... ok"),
        "got: {stdout}"
    );
    assert!(!stdout.contains("teste_lento"), "got: {stdout}");
}
