//! End-to-end tests for the `expr?` failure-propagation operator.
//!
//! `?` is verified through the CLI so the whole chain is covered at once: lexer, parser,
//! HIR, IR lowering to `PropagateFailure`, bytecode emission and VM execution.

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
    (code, program_output, String::from_utf8_lossy(&err).into_owned())
}

#[test]
fn test_try_operator_conformance_fixture_passes() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture = manifest.join("../../docs/conformance/programs/29_try_propagation_operator.aipo");
    let (code, stdout, stderr) = run_program(&fixture);
    assert_eq!(code, 0, "run must succeed, stderr: {stderr}");
    assert!(stderr.is_empty(), "must not warn: {stderr}");

    let expected = std::fs::read_to_string(
        manifest.join("../../docs/conformance/programs/29_try_propagation_operator.stdout"),
    )
    .expect("snapshot exists");

    assert_eq!(stdout, expected, "stdout must match snapshot");
}

#[test]
fn test_try_operator_in_subexpression() {
    // `?` works inside binary expressions, indexing, calls, etc.
    let temp = std::env::temp_dir().join("aipo_try_subexpr.aipo");
    let source = r#"
fn get_num(flag) {
    if flag {
        return 10
    }
    fail "not a number"
}

fn add_nums() {
    let result = (get_num(true)? * 2) + get_num(true)?
    return result
}

io.println(String(add_nums() or_else -1))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "30");
}

#[test]
fn test_try_operator_propagates_on_failure() {
    let temp = std::env::temp_dir().join("aipo_try_prop.aipo");
    let source = r#"
fn produce_failure() {
    fail "inner reason"
}

fn outer() {
    let _ = produce_failure()?
    return "unreachable"
}

attempt {
    io.println(outer())
} failed err {
    io.println("caught: " + err.message)
}
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "caught: inner reason");
}
