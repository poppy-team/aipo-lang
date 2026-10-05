//! CLI-level host ABI conformance, using the real executable and committed fixtures.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/conformance/host")
        .join(name)
}

fn cli(args: &[&str], path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aipo"))
        .args(args)
        .arg(path)
        .output()
        .expect("run CLI executable")
}

fn source(name: &str, text: &str, args: &[&str]) -> Output {
    let path =
        std::env::temp_dir().join(format!("aipo-headless-{}-{name}.aipo", std::process::id()));
    std::fs::write(&path, text).expect("write fixture");
    let output = cli(args, &path);
    std::fs::remove_file(path).expect("remove fixture");
    output
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("UTF-8 stderr")
}

fn assert_code(output: &Output, expected_exit: i32, expected_code: &str) {
    let error = stderr(output);
    assert_eq!(output.status.code(), Some(expected_exit), "{error}");
    assert!(error.contains(expected_code), "{error}");
    assert!(output.stdout.is_empty(), "{:?}", output.stdout);
}

#[test]
fn committed_fault_fixtures_pass_check_and_reach_the_exact_runtime_fault() {
    for name in ["capability_denied", "stale_handle", "scope_escape"] {
        let path = fixture(&format!("{name}.aipo"));
        let code =
            std::fs::read_to_string(fixture(&format!("{name}.code"))).expect("read committed code");
        let checked = cli(&["check", "--host=headless-test"], &path);
        assert!(checked.status.success(), "{}", stderr(&checked));
        assert!(checked.stdout.is_empty() && checked.stderr.is_empty());
        assert_code(
            &cli(&["run", "--host=headless-test"], &path),
            1,
            code.trim(),
        );
        let jsonl = cli(
            &["run", "--host=headless-test", "--message-format=jsonl"],
            &path,
        );
        assert_code(&jsonl, 1, code.trim());
        let error = stderr(&jsonl);
        assert_eq!(error.lines().count(), 1, "{error}");
        assert!(error.starts_with('{') && error.ends_with("}\n"), "{error}");
        assert!(
            error.contains(&format!("\"code\":\"{}\"", code.trim())),
            "{error}"
        );
        assert!(error.contains("\"severity\":\"error\""), "{error}");
        assert!(error.contains("while running"), "{error}");
    }
}

#[test]
fn live_handles_run_deterministically_and_separate_host_flag_is_supported() {
    let expected = std::fs::read(fixture("live_handle.stdout")).expect("read stdout snapshot");
    for _ in 0..3 {
        let output = cli(
            &["run", "--host", "headless-test"],
            &fixture("live_handle.aipo"),
        );
        assert!(output.status.success(), "{}", stderr(&output));
        assert_eq!(output.stdout, expected);
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn scopes_fault_at_all_six_heap_publication_sites() {
    for (name, text, site) in [
        (
            "global",
            "let escaped = headless.scoped(1)",
            "global 'escaped'",
        ),
        (
            "return",
            "fn leak() { let h = headless.scoped(1)\nreturn h }\nleak()",
            "a return value",
        ),
        ("list", "[headless.scoped(1)]", "a list element"),
        (
            "dict",
            "let entry = {\"held\": headless.scoped(1)}",
            "a dict",
        ),
        (
            "index",
            "let items = [none]\nitems[0] = headless.scoped(1)",
            "an indexed element",
        ),
        (
            "field",
            "struct Box { var held }\nlet box = Box{held: none}\nbox.held = headless.scoped(1)",
            "field 'held'",
        ),
    ] {
        let output = source(name, text, &["run", "--host=headless-test"]);
        assert_code(&output, 1, "AIPO_RT_SCOPE_ESCAPE");
        assert!(
            stderr(&output).contains(site),
            "{name}: {}",
            stderr(&output)
        );
    }
}

#[test]
fn both_clock_operations_use_the_deny_by_default_context() {
    for operation in ["now", "monotonic"] {
        let output = source(
            operation,
            &format!("time.{operation}()"),
            &["run", "--host=headless-test"],
        );
        assert_code(&output, 1, "AIPO_RT_CAPABILITY_DENIED");
        assert!(stderr(&output).contains(&format!("time.{operation}")));
        assert!(stderr(&output).contains("clock"));
    }
}

#[test]
fn a_fault_cannot_be_caught_as_a_recoverable_failure() {
    let output = source(
        "uncatchable",
        "attempt { time.now() } failed err { io.println(\"caught\") }",
        &["run", "--host=headless-test"],
    );
    assert_code(&output, 1, "AIPO_RT_CAPABILITY_DENIED");
}

#[test]
fn profile_does_not_leak_between_in_process_invocations_or_embedding() {
    let profile_args = vec![
        "check".to_string(),
        "--host=headless-test".to_string(),
        fixture("live_handle.aipo").display().to_string(),
    ];
    let mut out = Vec::new();
    let mut err = Vec::new();
    assert_eq!(aipo_cli::run_with(&profile_args, &mut out, &mut err), 0);
    let plain_args = vec![
        "check".to_string(),
        fixture("live_handle.aipo").display().to_string(),
    ];
    assert_eq!(aipo_cli::run_with(&plain_args, &mut out, &mut err), 1);
    assert!(
        String::from_utf8(err)
            .expect("UTF-8")
            .contains("AIPO_SEM_UNKNOWN_NAME")
    );
    assert!(aipo_cli::compile_file(&fixture("live_handle.aipo"), None).is_err());

    // The VM-local denial must not revoke the ordinary CLI's process-global clock.
    let denied = vec![
        "run".to_string(),
        "--host=headless-test".to_string(),
        fixture("capability_denied.aipo").display().to_string(),
    ];
    assert_eq!(aipo_cli::run_with(&denied, &mut out, &mut Vec::new()), 1);
    let normal = vec![
        "run".to_string(),
        fixture("capability_denied.aipo").display().to_string(),
    ];
    assert_eq!(aipo_cli::run_with(&normal, &mut out, &mut Vec::new()), 0);
}

#[test]
fn ahs_alone_neither_installs_natives_nor_grants_capabilities() {
    let flag = format!("--ahs={}", fixture("headless_test_host.json").display());
    let path = fixture("live_handle.aipo");
    let checked = cli(&["check", &flag], &path);
    assert!(checked.status.success(), "{}", stderr(&checked));
    assert_code(&cli(&["run", &flag], &path), 1, "AIPO_RT_");
}

#[test]
fn host_signatures_reject_unknown_members_arity_and_literal_contracts() {
    for (name, text, expected) in [
        ("unknown", "headless.nonexistent()", "AIPO_SEM_UNKNOWN_NAME"),
        ("arity", "headless.create()", "AIPO_SEM_ARITY_MISMATCH"),
        (
            "contract",
            "headless.create(\"not an Int\")",
            "AIPO_SEM_CONTRACT_VIOLATION_STATIC",
        ),
    ] {
        assert_code(
            &source(name, text, &["check", "--host=headless-test"]),
            1,
            expected,
        );
    }
    assert_code(
        &source(
            "dynamic",
            "let value = \"dynamic\"\nheadless.create(value)",
            &["run", "--host=headless-test"],
        ),
        1,
        "AIPO_RT_TYPE_MISMATCH",
    );
    assert_code(
        &source(
            "invalid-handle",
            "headless.read(42)",
            &["run", "--host=headless-test"],
        ),
        1,
        "AIPO_RT_TYPE_MISMATCH",
    );
}

#[test]
fn invalid_host_flags_and_unsupported_combinations_are_usage_errors() {
    let path = fixture("live_handle.aipo");
    let ahs = format!("--ahs={}", fixture("headless_test_host.json").display());
    for args in [
        vec!["run", "--host="],
        vec!["check", "--host=unknown"],
        vec!["run", "--host", "--wasm"],
        vec!["run", "--host=headless-test", "--host=headless-test"],
        vec!["run", "--host=headless-test", "--wasm"],
        vec!["check", "--host=headless-test", "--target=wasm"],
        vec!["disasm", "--host=headless-test"],
        vec!["build", "--host=headless-test"],
        vec!["test", "--host=headless-test"],
        vec!["run", "--host=headless-test", &ahs],
    ] {
        assert_code(&cli(&args, &path), 2, "--host");
    }
    for path in [Path::new("precompiled.aibc"), Path::new("precompiled.wasm")] {
        assert_code(&cli(&["run", "--host=headless-test"], path), 2, "--host");
    }
    let missing = Command::new(env!("CARGO_BIN_EXE_aipo"))
        .args(["run", "--host"])
        .output()
        .expect("missing profile");
    assert_code(&missing, 2, "--host");
}
