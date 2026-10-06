//! Integration tests for the `aipo-sh` shell and evaluation interface (Marco 4 / ADP-014).

use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

fn test_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[derive(Clone)]
struct TestSink(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for TestSink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_eval_source_simple_expression() {
    let _guard = test_lock();
    let captured = Arc::new(Mutex::new(Vec::new()));
    aipo_cli::set_output_sink(Some(Box::new(TestSink(Arc::clone(&captured)))));
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = "io.println(10 + 20)";
    let exit_code = aipo_cli::eval_source(code, &mut out, &mut err);
    aipo_cli::set_output_sink(None);

    assert_eq!(exit_code, 0);
    let output = String::from_utf8(captured.lock().unwrap().clone()).unwrap();
    assert_eq!(output, "30\n");
    assert!(err.is_empty());
}

#[test]
fn test_eval_source_pipeline() {
    let _guard = test_lock();
    let captured = Arc::new(Mutex::new(Vec::new()));
    aipo_cli::set_output_sink(Some(Box::new(TestSink(Arc::clone(&captured)))));
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = r#"
fn double(x: Int) -> Int {
    return x * 2
}
io.println(21 |> double())
"#;
    let exit_code = aipo_cli::eval_source(code, &mut out, &mut err);
    aipo_cli::set_output_sink(None);

    assert_eq!(exit_code, 0);
    let output = String::from_utf8(captured.lock().unwrap().clone()).unwrap();
    assert_eq!(output, "42\n");
}

#[test]
fn test_eval_source_syntax_error_reports_failure() {
    let _guard = test_lock();
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = "fn broken(";
    let exit_code = aipo_cli::eval_source(code, &mut out, &mut err);
    assert_eq!(exit_code, 1);
    assert!(!err.is_empty());
}
