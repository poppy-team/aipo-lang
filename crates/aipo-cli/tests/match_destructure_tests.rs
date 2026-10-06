//! End-to-end tests for `match` destructuring and guards.
//!
//! A `when { field, ... }` arm binds those fields into names visible in the guard and the
//! arm body. An optional `if guard` filters a matched arm: a `false` guard falls through to
//! the next arm. Exercised through the CLI so the whole chain runs together.

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

fn check_program(path: &Path) -> (u8, String, String) {
    let _serialized = run_lock();
    let args: Vec<String> = vec!["check".into(), path.to_string_lossy().into()];
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = aipo_cli::run_with(&args, &mut out, &mut err);
    (code, String::from_utf8_lossy(&out).into_owned(), String::from_utf8_lossy(&err).into_owned())
}

#[test]
fn test_match_destructure_binds_fields_in_arm_body() {
    let temp = std::env::temp_dir().join("aipo_match_dest.aipo");
    let source = r#"
struct User
    name
    age
end

let u = User{name = "ana", age = 30}
match u
    when { name, age } then
        io.println(name + " " + String(age))
end
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "ana 30");
}

#[test]
fn test_match_guard_accepts_and_rejects() {
    let temp = std::env::temp_dir().join("aipo_match_guard.aipo");
    let source = r#"
struct User
    name
    age
end

fn describe(u)
    match u
        when { name, age } if age >= 65 then
            return "senior " + name
        when { name, age } if age >= 18 then
            return "adult " + name
        else
            return "minor"
    end
end

let ana = User{name = "ana", age = 30}
let kid = User{name = "bo", age = 7}
io.println(describe(ana))
io.println(describe(kid))
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "adult ana\nminor");
}

#[test]
fn test_match_destructure_subset_of_fields() {
    let temp = std::env::temp_dir().join("aipo_match_subset.aipo");
    let source = r#"
struct User
    name
    age
end

let u = User{name = "ana", age = 30}
match u
    when { name } then
        io.println(name)
end
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "ana");
}

#[test]
fn test_match_guard_on_value_pattern() {
    let temp = std::env::temp_dir().join("aipo_match_guard_value.aipo");
    let source = r#"
match 5
    when 1, 2, 3 then
        io.println("small")
    when 4, 5, 6 if true then
        io.println("medium")
    else
        io.println("large")
end
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, stdout, stderr) = run_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_eq!(code, 0, "stderr: {stderr}");
    assert_eq!(stdout.trim(), "medium");
}

#[test]
fn test_match_destructure_is_scoped_to_its_arm() {
    // A bound name does not leak into sibling arms or past the statement.
    let temp = std::env::temp_dir().join("aipo_match_scoped.aipo");
    let source = r#"
struct User
    name
    age
end

let u = User{name = "ana", age = 30}
match u
    when { name } then
        io.println(name)
end
io.println(name)
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, _stdout, stderr) = check_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_ne!(code, 0, "must reject the leaked name");
    assert!(
        stderr.contains("AIPO_SEM_UNKNOWN_NAME"),
        "must report AIPO_SEM_UNKNOWN_NAME, got: {stderr}"
    );
}

#[test]
fn test_match_non_bool_guard_is_rejected() {
    // The guard is a condition, so a literal that is provably not a `Bool` is a
    // diagnostic before execution — the same rule `if` applies.
    let temp = std::env::temp_dir().join("aipo_match_guard_non_bool.aipo");
    let source = r#"
struct User
    name
    age
end

let u = User{name = "ana", age = 30}
match u
    when { name, age } if 1 then
        io.println(name)
end
"#;
    std::fs::write(&temp, source).unwrap();
    let (code, _stdout, stderr) = check_program(&temp);
    let _ = std::fs::remove_file(&temp);
    assert_ne!(code, 0, "must reject a non-Bool guard");
    assert!(
        stderr.contains("AIPO_SEM_NON_BOOL_CONDITION"),
        "must report AIPO_SEM_NON_BOOL_CONDITION, got: {stderr}"
    );
}
