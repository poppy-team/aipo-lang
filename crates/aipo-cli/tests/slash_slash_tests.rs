//! Tests for the `//` trap: in Aipo `//` is integer division, not a comment.
//!
//! When written where an expression cannot follow (at statement start or prefix),
//! the diagnostic must explicitly suggest `#` instead of a cryptic parse error.

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
fn test_slash_slash_at_statement_start_suggests_hash() {
    let temp = std::env::temp_dir().join("aipo_slash_stmt.aipo");
    let source = r#"
// this is not a comment in Aipo
let x = 1
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, _stdout, stderr) = check_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_ne!(code, 0, "must reject `//` at statement start");
    assert!(
        stderr.contains("is integer division in Aipo, not a comment"),
        "must explain that // is integer division, got: {stderr}"
    );
    assert!(
        stderr.contains("use '#' for line comments"),
        "must suggest '#' for line comments, got: {stderr}"
    );
}

#[test]
fn test_slash_slash_as_prefix_suggests_hash() {
    let temp = std::env::temp_dir().join("aipo_slash_prefix.aipo");
    let source = r#"
let x = // trying to comment here
1
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, _stdout, stderr) = check_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_ne!(code, 0, "must reject `//` as expression prefix");
    assert!(
        stderr.contains("is integer division in Aipo, not a comment"),
        "must explain that // is integer division, got: {stderr}"
    );
}

#[test]
fn test_slash_slash_as_valid_integer_division_works() {
    let temp = std::env::temp_dir().join("aipo_slash_valid.aipo");
    let source = r#"
let a = 7 // 2
let b = 15 // 4
io.println(String(a) + " " + String(b))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "3 3");
}
