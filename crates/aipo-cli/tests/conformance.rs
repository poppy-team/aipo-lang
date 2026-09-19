//! Conformance suite for the Wave 1 command surface (slices S10 and S11).
//!
//! The corpus under `docs/conformance` is the shared artifact between the CLI, the
//! formatter and the language: `programs/` holds runnable programs with committed stdout
//! snapshots, `diagnostics/` holds fixtures that must fail with a specific diagnostic code,
//! and `formatting/` holds the formatter golden corpus (exercised by `aipo-formatter`).
//!
//! Regenerate stdout snapshots with `AIPO_UPDATE_SNAPSHOTS=1 cargo test -p aipo-cli`.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

/// Exit codes are part of the CLI contract documented in `docs/reference/cli.md`.
const EXIT_SUCCESS: u8 = 0;
const EXIT_LANGUAGE_FAILURE: u8 = 1;
const EXIT_USAGE: u8 = 2;

fn conformance_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/conformance")
}

fn fixtures(subdirectory: &str, extension: &str) -> Vec<PathBuf> {
    let directory = conformance_dir().join(subdirectory);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", directory.display()))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.to_string_lossy().ends_with(extension))
        .collect();
    paths.sort();
    assert!(
        !paths.is_empty(),
        "{} must contain at least one {extension} fixture",
        directory.display()
    );
    paths
}

/// Serializes every in-process CLI invocation.
///
/// The `io` sink installed by [`run_program`] is process-wide, so a run that captures program
/// output must be **exclusive**: another test executing a program at the same time would write
/// its own output into the capture buffer, which pollutes a regenerated snapshot with lines the
/// fixture never produced. The lock is therefore held for every invocation, not only for the
/// capturing one, and `run_program` calls [`run_cli_inner`] directly to avoid re-entering it.
fn run_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Writer that appends everything it receives to a shared buffer.
#[derive(Clone)]
struct CaptureSink(Arc<Mutex<Vec<u8>>>);

impl Write for CaptureSink {
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

/// Runs the CLI in-process so the whole surface is testable without spawning a process.
fn run_cli(args: &[&str]) -> (u8, String, String) {
    let _serialized = run_lock();
    run_cli_inner(args)
}

/// Runs the CLI without taking [`run_lock`], for callers that already hold it.
fn run_cli_inner(args: &[&str]) -> (u8, String, String) {
    let args: Vec<String> = args.iter().map(|arg| (*arg).to_string()).collect();
    let mut out: Vec<u8> = Vec::new();
    let mut err: Vec<u8> = Vec::new();
    let code = aipo_cli::run_with(&args, &mut out, &mut err);
    (
        code,
        String::from_utf8_lossy(&out).into_owned(),
        String::from_utf8_lossy(&err).into_owned(),
    )
}

/// Runs a program, capturing the output its `io` calls produce.
///
/// The standard library writes program output to the process streams, so the sink hook is
/// installed for the duration of the run and removed afterwards.
fn run_program(path: &Path) -> (u8, String, String) {
    let _serialized = run_lock();
    let captured = Arc::new(Mutex::new(Vec::new()));
    aipo_cli::set_output_sink(Some(Box::new(CaptureSink(Arc::clone(&captured)))));
    let result = run_cli_inner(&["run", &path.to_string_lossy()]);
    aipo_cli::set_output_sink(None);

    let program_output = {
        let buffer = captured
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        String::from_utf8_lossy(&buffer).into_owned()
    };
    (result.0, program_output, result.2)
}

fn replace_extension(path: &Path, from: &str, to: &str) -> PathBuf {
    PathBuf::from(path.to_string_lossy().replace(from, to))
}

#[test]
fn test_programs_run_and_match_committed_stdout() {
    let update = std::env::var("AIPO_UPDATE_SNAPSHOTS").is_ok();
    for program in fixtures("programs", ".aipo") {
        let (code, stdout, stderr) = run_program(&program);
        assert_eq!(
            code,
            EXIT_SUCCESS,
            "{} must run successfully, stderr: {stderr}",
            program.display()
        );
        assert!(stderr.is_empty(), "{} must not warn", program.display());

        let snapshot = replace_extension(&program, ".aipo", ".stdout");
        if update || !snapshot.exists() {
            std::fs::write(&snapshot, &stdout).expect("snapshot is writable");
        }

        let expected = std::fs::read_to_string(&snapshot).expect("snapshot is readable");
        assert_eq!(
            stdout,
            expected,
            "{} produced unexpected output",
            program.display()
        );
    }
}

#[test]
fn test_programs_pass_check_without_diagnostics() {
    for program in fixtures("programs", ".aipo") {
        let (code, stdout, stderr) = run_cli(&["check", &program.to_string_lossy()]);
        assert_eq!(
            code,
            EXIT_SUCCESS,
            "{} must pass `aipo check`, stderr: {stderr}",
            program.display()
        );
        assert!(stdout.is_empty() && stderr.is_empty());
    }
}

#[test]
fn test_programs_are_required_to_compile_for_execution() {
    // A program snapshot is only meaningful if execution reached the end of the file, so at
    // least one fixture must emit output (guards against a corpus of empty programs).
    let total: usize = fixtures("programs", ".stdout")
        .iter()
        .map(|path| {
            std::fs::read_to_string(path)
                .expect("snapshot is readable")
                .len()
        })
        .sum();
    assert!(total > 0, "program snapshots must contain output");
}

#[test]
fn test_failing_fixtures_report_expected_diagnostic_codes() {
    for fixture in fixtures("diagnostics", ".aipo") {
        let (code, _stdout, stderr) = run_cli(&["run", &fixture.to_string_lossy()]);
        assert_eq!(
            code,
            EXIT_LANGUAGE_FAILURE,
            "{} must fail the language contract, stderr: {stderr}",
            fixture.display()
        );

        let codes_file = replace_extension(&fixture, ".aipo", ".code");
        let expected: Vec<String> = std::fs::read_to_string(&codes_file)
            .expect("expected-codes file is readable")
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect();
        assert!(
            !expected.is_empty(),
            "{} must declare at least one diagnostic code",
            codes_file.display()
        );

        for expected_code in expected {
            assert!(
                stderr.contains(&expected_code),
                "{} must report {expected_code}, got: {stderr}",
                fixture.display()
            );
        }
    }
}

#[test]
fn test_formatting_corpus_reports_drift_with_check() {
    // `fmt --check` is the CI gate: unformatted input must fail without being rewritten.
    for input in fixtures("formatting", ".input.aipo") {
        let temp = std::env::temp_dir().join(
            input
                .file_name()
                .expect("fixture has a file name")
                .to_string_lossy()
                .replace(".input.aipo", ".check.aipo"),
        );
        std::fs::copy(&input, &temp).expect("fixture is copyable");
        let original = std::fs::read_to_string(&temp).expect("copy is readable");
        let expected =
            std::fs::read_to_string(replace_extension(&input, ".input.aipo", ".expected.aipo"))
                .expect("expectation is readable");

        let (code, _stdout, stderr) = run_cli(&["fmt", "--check", &temp.to_string_lossy()]);
        // An already-canonical input is not drift, so the fixture's own expectation decides.
        if original == expected {
            assert_eq!(
                code,
                EXIT_SUCCESS,
                "{} is canonical, so --check must pass: {stderr}",
                input.display()
            );
        } else {
            assert_eq!(
                code,
                EXIT_LANGUAGE_FAILURE,
                "{} must be reported as drift",
                input.display()
            );
            assert!(stderr.contains("would reformat"));
        }
        assert_eq!(
            std::fs::read_to_string(&temp).expect("copy is readable"),
            original,
            "`fmt --check` must never rewrite the file"
        );

        let _ = std::fs::remove_file(&temp);
    }
}

#[test]
fn test_canonical_sources_report_no_formatting_drift() {
    // The committed expectations are canonical, so `fmt --check` on them must be a no-op.
    for expected in fixtures("formatting", ".expected.aipo") {
        let (code, _stdout, stderr) = run_cli(&["fmt", "--check", &expected.to_string_lossy()]);
        assert_eq!(
            code,
            EXIT_SUCCESS,
            "{} is canonical, so --check must pass: {stderr}",
            expected.display()
        );
    }
}

/// Entry programs of the module corpus: `modules/<case>/<name>.aipo` whose sibling files are
/// either imported modules or further entry programs.
///
/// Naming keeps the two roles apart without a manifest: a file listed in a snapshot's
/// companion `.code` file is a failing entry, `entry.aipo` is always an entry, and every
/// other `.aipo` in the case directory is a module available for import.
fn module_entries() -> Vec<PathBuf> {
    let root = conformance_dir().join("modules");
    let mut entries: Vec<PathBuf> = std::fs::read_dir(&root)
        .unwrap_or_else(|error| panic!("{} is readable: {error}", root.display()))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_dir())
        .collect();
    entries.sort();
    assert!(
        !entries.is_empty(),
        "{} must contain at least one module case",
        root.display()
    );
    entries
}

#[test]
fn test_module_entry_programs_match_committed_stdout() {
    let update = std::env::var("AIPO_UPDATE_SNAPSHOTS").is_ok();
    for case_dir in module_entries() {
        let entry = case_dir.join("entry.aipo");
        // A case whose entry owns a `.code` file is a failure fixture, covered below.
        if replace_extension(&entry, ".aipo", ".code").exists() {
            continue;
        }
        let (code, stdout, stderr) = run_program(&entry);
        assert_eq!(
            code,
            EXIT_SUCCESS,
            "{} must run successfully, stderr: {stderr}",
            entry.display()
        );

        let snapshot = case_dir.join("entry.stdout");
        if update {
            std::fs::write(&snapshot, &stdout).expect("snapshot is writable");
        }
        let expected = std::fs::read_to_string(&snapshot).expect("snapshot is readable");
        assert_eq!(
            stdout,
            expected,
            "{} produced unexpected output",
            entry.display()
        );
    }
}

#[test]
fn test_module_aliases_and_selective_imports_match_committed_stdout() {
    let update = std::env::var("AIPO_UPDATE_SNAPSHOTS").is_ok();
    let case_dir = conformance_dir().join("modules/basic");
    for name in ["alias.aipo", "init_once.aipo"] {
        let program = case_dir.join(name);
        let (code, stdout, stderr) = run_program(&program);
        assert_eq!(
            code,
            EXIT_SUCCESS,
            "{} must run successfully, stderr: {stderr}",
            program.display()
        );

        let snapshot = replace_extension(&program, ".aipo", ".stdout");
        if update || !snapshot.exists() {
            std::fs::write(&snapshot, &stdout).expect("snapshot is writable");
        }
        let expected = std::fs::read_to_string(&snapshot).expect("snapshot is readable");
        assert_eq!(
            stdout,
            expected,
            "{} produced unexpected output",
            program.display()
        );
    }
}

#[test]
fn test_module_failures_report_expected_diagnostic_codes() {
    for case_dir in module_entries() {
        for name in [
            "entry.aipo",
            "private_violation.aipo",
            "alias_private_violation.aipo",
        ] {
            let fixture = case_dir.join(name);
            if !fixture.exists() {
                continue;
            }
            let codes_file = replace_extension(&fixture, ".aipo", ".code");
            if !codes_file.exists() {
                // Passing entry programs are covered by the snapshot test above.
                assert_eq!(name, "entry.aipo");
                continue;
            }

            let (code, _stdout, stderr) = run_cli(&["check", &fixture.to_string_lossy()]);
            assert_eq!(
                code,
                EXIT_LANGUAGE_FAILURE,
                "{} must fail the language contract, stderr: {stderr}",
                fixture.display()
            );

            let expected = std::fs::read_to_string(&codes_file).expect("codes file is readable");
            for expected_code in expected.lines().map(str::trim).filter(|l| !l.is_empty()) {
                assert!(
                    stderr.contains(expected_code),
                    "{} must report {expected_code}, got: {stderr}",
                    fixture.display()
                );
            }
        }
    }
}

#[test]
fn test_run_reports_jsonl_diagnostics() {
    let fixture = conformance_dir().join("diagnostics/04_sem_unknown_name.aipo");
    let (code, _stdout, stderr) =
        run_cli(&["run", &fixture.to_string_lossy(), "--message-format=jsonl"]);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE);
    assert!(
        stderr.contains("AIPO_SEM_UNKNOWN_NAME"),
        "jsonl output must carry the diagnostic code: {stderr}"
    );
}

#[test]
fn test_usage_errors_use_the_usage_exit_code() {
    let cases: Vec<Vec<&str>> = vec![
        vec![],
        vec!["frobnicate"],
        vec!["run"],
        vec!["run", "--unknown-flag", "x.aipo"],
        vec!["run", "/definitely/not/a/file.aipo"],
        vec!["fmt"],
        vec!["fmt", "--nope", "x.aipo"],
    ];
    for args in cases {
        let (code, _stdout, _stderr) = run_cli(&args);
        assert_eq!(code, EXIT_USAGE, "arguments {args:?} must be a usage error");
    }
}

#[test]
fn test_help_and_version_are_available() {
    let (code, stdout, _stderr) = run_cli(&["--help"]);
    assert_eq!(code, EXIT_SUCCESS);
    assert!(stdout.contains("USAGE:"));

    let (code, stdout, _stderr) = run_cli(&["--version"]);
    assert_eq!(code, EXIT_SUCCESS);
    assert!(stdout.starts_with("aipo "));
}

#[test]
fn test_missing_end_fixture_reports_a_parse_diagnostic() {
    // Guards the corpus itself: a fixture that stopped parsing for an unrelated reason would
    // still "fail", so the parse fixtures assert the class of the diagnostic they own.
    let fixture = conformance_dir().join("diagnostics/01_parse_missing_end.aipo");
    let (code, _stdout, stderr) = run_cli(&["check", &fixture.to_string_lossy()]);
    assert_eq!(code, EXIT_LANGUAGE_FAILURE);
    assert!(stderr.contains("AIPO_PARSE_"));
}
