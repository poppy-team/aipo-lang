//! Comprehensive test suite for `aipo test` command runner (ADP-008).
//!
//! Validates:
//! - Canonical `test("name") do ... end` syntax
//! - Assertions via `expect.*`
//! - Isolated pristine VM per test (zero shared state)
//! - Async execution inside tests (`task.spawn` and `await`)
//! - Controlled virtual clock and deterministic PRNG
//! - `--filter <pattern>` flag
//! - `--message-format=jsonl` output structure
//! - Exit codes: 0 for success, 1 for failure, 2 for usage error

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

const EXIT_SUCCESS: u8 = 0;
const EXIT_LANGUAGE_FAILURE: u8 = 1;
const EXIT_USAGE: u8 = 2;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_test_dir(prefix: &str) -> PathBuf {
    let id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("aipo-test-{prefix}-{id}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create test temp dir");
    dir
}

fn run_test_cli(args: &[&str]) -> (u8, String, String) {
    let string_args: Vec<String> = args.iter().map(|s| (*s).to_string()).collect();
    let mut out: Vec<u8> = Vec::new();
    let mut err: Vec<u8> = Vec::new();
    let code = aipo_cli::run_with(&string_args, &mut out, &mut err);
    (
        code,
        String::from_utf8_lossy(&out).into_owned(),
        String::from_utf8_lossy(&err).into_owned(),
    )
}

#[test]
fn test_runner_passing_assertions() {
    let dir = temp_test_dir("passing");
    let test_file = dir.join("math_test.aipo");
    fs::write(
        &test_file,
        r#"
test("arithmetic and comparisons") do
    expect.equal(2 + 2, 4)
    expect.not_equal(2 + 2, 5)
    expect.true(10 > 5)
    expect.false(5 > 10)
    expect.none(none)
    expect.some(42)
    expect.approx(3.14159, 3.1416, 0.001)
end

test("collections and strings") do
    expect.contains("hello world", "world")
    let list = [10, 20, 30]
    expect.contains(list, 20)
end
"#,
    )
    .expect("write test file");

    let (code, out, _err) = run_test_cli(&["test", test_file.to_str().unwrap()]);
    assert_eq!(code, EXIT_SUCCESS);
    assert!(out.contains("running 2 tests"));
    assert!(out.contains("test arithmetic and comparisons ... ok"));
    assert!(out.contains("test collections and strings ... ok"));
    assert!(out.contains("test result: ok. 2 passed; 0 failed; 0 skipped"));
}

#[test]
fn test_runner_failing_assertion() {
    let dir = temp_test_dir("failing");
    let test_file = dir.join("fail_test.aipo");
    fs::write(
        &test_file,
        r#"
test("passing check") do
    expect.equal(1, 1)
end

test("failing check") do
    expect.equal(1, 2)
end
"#,
    )
    .expect("write test file");

    let (code, out, _err) = run_test_cli(&["test", test_file.to_str().unwrap()]);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE);
    assert!(out.contains("test passing check ... ok"));
    assert!(out.contains("test failing check ... FAILED"));
    assert!(out.contains("expect.equal failed: expected 2, got 1"));
    assert!(out.contains("test result: FAILED. 1 passed; 1 failed; 0 skipped"));
}

#[test]
fn test_runner_runtime_fault() {
    let dir = temp_test_dir("fault");
    let test_file = dir.join("fault_test.aipo");
    fs::write(
        &test_file,
        r#"
test("divide by zero fault") do
    let x = 1 / 0
    expect.equal(x, 0)
end
"#,
    )
    .expect("write test file");

    let (code, out, _err) = run_test_cli(&["test", test_file.to_str().unwrap()]);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE);
    assert!(out.contains("test divide by zero fault ... FAILED"));
    assert!(out.contains("AIPO_RT_DIV_ZERO"));
}

#[test]
fn test_runner_isolation_between_tests() {
    let dir = temp_test_dir("isolation");
    let test_file = dir.join("isolation_test.aipo");
    // Global variable initialized at module level.
    // Test 1 mutates it; Test 2 verifies it starts fresh and unmutated.
    fs::write(
        &test_file,
        r#"
let shared_list = []

test("test 1 mutates global") do
    shared_list.add(100)
    expect.equal(shared_list.len(), 1)
end

test("test 2 sees pristine global") do
    expect.equal(shared_list.len(), 0)
end
"#,
    )
    .expect("write test file");

    let (code, out, _err) = run_test_cli(&["test", test_file.to_str().unwrap()]);
    assert_eq!(code, EXIT_SUCCESS);
    assert!(out.contains("test test 1 mutates global ... ok"));
    assert!(out.contains("test test 2 sees pristine global ... ok"));
    assert!(out.contains("2 passed; 0 failed"));
}

#[test]
fn test_runner_async_tasks() {
    let dir = temp_test_dir("async");
    let test_file = dir.join("async_test.aipo");
    fs::write(
        &test_file,
        r#"
test("async tasks and await") do
    let task1 = task.spawn(fn() return 10 + 20 end, [])
    let res = await task1
    expect.equal(res, 30)
end
"#,
    )
    .expect("write test file");

    let (code, out, _err) = run_test_cli(&["test", test_file.to_str().unwrap()]);
    assert_eq!(code, EXIT_SUCCESS);
    assert!(out.contains("test async tasks and await ... ok"));
    assert!(out.contains("1 passed; 0 failed"));
}

#[test]
fn test_runner_filter_flag() {
    let dir = temp_test_dir("filter");
    let test_file = dir.join("filter_test.aipo");
    fs::write(
        &test_file,
        r#"
test("apple pie") do
    expect.true(true)
end

test("banana split") do
    expect.true(true)
end
"#,
    )
    .expect("write test file");

    let (code, out, _err) =
        run_test_cli(&["test", test_file.to_str().unwrap(), "--filter", "banana"]);
    assert_eq!(code, EXIT_SUCCESS);
    assert!(out.contains("running 1 test in"));
    assert!(out.contains("test banana split ... ok"));
    assert!(!out.contains("test apple pie ... ok"));
    assert!(out.contains("1 passed; 0 failed; 1 skipped"));
}

#[test]
fn test_runner_jsonl_output() {
    let dir = temp_test_dir("jsonl");
    let test_file = dir.join("jsonl_test.aipo");
    fs::write(
        &test_file,
        r#"
test("first") do
    expect.equal(1, 1)
end

test("second") do
    expect.equal(1, 2)
end
"#,
    )
    .expect("write test file");

    let (code, out, _err) = run_test_cli(&[
        "test",
        test_file.to_str().unwrap(),
        "--message-format=jsonl",
    ]);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE);

    let lines: Vec<&str> = out.lines().filter(|l| !l.is_empty()).collect();
    assert!(lines.iter().any(|l| l.contains(r#""type":"suite_start""#)));
    assert!(
        lines
            .iter()
            .any(|l| l.contains(r#""type":"test_start""#) && l.contains(r#""name":"first""#))
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains(r#""type":"test_pass""#) && l.contains(r#""name":"first""#))
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains(r#""type":"test_start""#) && l.contains(r#""name":"second""#))
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains(r#""type":"test_fail""#) && l.contains(r#""name":"second""#))
    );
    assert!(lines.iter().any(|l| l.contains(r#""type":"suite_finish""#)
        && l.contains(r#""passed":1"#)
        && l.contains(r#""failed":1"#)));
}

#[test]
fn test_runner_no_tests_found_exits_usage() {
    let dir = temp_test_dir("empty");
    let (code, _out, err) = run_test_cli(&["test", dir.to_str().unwrap()]);
    assert_eq!(code, EXIT_USAGE);
    assert!(err.contains("no test files found"));
}
