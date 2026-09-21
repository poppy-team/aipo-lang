//! Diagnostic UI goldens: user-visible rendering is a stable product surface.
//!
//! For high-frequency diagnostics the suite pins the full human rendering and
//! the JSONL representation, plus structural invariants that hold for every
//! diagnostic fixture: stdout stays empty on failure, no ANSI escapes appear
//! anywhere, JSONL always carries the complete schema, and cascading output
//! stays controlled (the nesting bound yields exactly one error).
//!
//! Regenerate goldens with `AIPO_UPDATE_SNAPSHOTS=1 cargo test -p aipo-cli`.

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

fn lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

fn snapshot_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/snapshots/ui")
}

fn run_cli(args: &[String]) -> (u8, String, String) {
    let mut out: Vec<u8> = Vec::new();
    let mut err: Vec<u8> = Vec::new();
    let code = aipo_cli::run_with(args, &mut out, &mut err);
    (
        code,
        String::from_utf8_lossy(&out).into_owned(),
        String::from_utf8_lossy(&err).into_owned(),
    )
}

fn check_human(relative: &str) -> (u8, String, String) {
    run_cli(&[
        "check".to_string(),
        relative.to_string(),
        "--message-format=human".to_string(),
    ])
}

fn check_jsonl(relative: &str) -> (u8, String, String) {
    run_cli(&[
        "check".to_string(),
        relative.to_string(),
        "--message-format=jsonl".to_string(),
    ])
}

fn run_human(relative: &str) -> (u8, String, String) {
    run_cli(&["run".to_string(), relative.to_string()])
}

fn golden(stem: &str, extension: &str) -> PathBuf {
    snapshot_dir().join(format!("{stem}.{extension}"))
}

fn expect_snapshot(stem: &str, extension: &str, actual: &str) {
    let path = golden(stem, extension);
    if std::env::var("AIPO_UPDATE_SNAPSHOTS").is_ok() {
        std::fs::create_dir_all(snapshot_dir()).expect("snapshot dir is writable");
        std::fs::write(&path, actual).expect("snapshot is writable");
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("snapshot {} exists", path.display()));
    assert_eq!(
        actual,
        expected,
        "golden drift in {} (re-run with AIPO_UPDATE_SNAPSHOTS=1 to review)",
        path.display()
    );
}

/// Check-time cases: fixture, expected code, expected `line:column`.
const CHECK_CASES: &[(&str, &str, &str)] = &[
    ("01_parse_missing_end", "AIPO_PARSE_UNEXPECTED_TOKEN", "3:1"),
    ("04_sem_unknown_name", "AIPO_SEM_UNKNOWN_NAME", "1:12"),
    (
        "15_sem_contract_violation",
        "AIPO_SEM_CONTRACT_VIOLATION_STATIC",
        "7:26",
    ),
];

/// Run-time cases (faults surface on execution, not on check).
const RUN_CASES: &[(&str, &str)] = &[
    ("09_runtime_div_zero", "AIPO_RT_DIV_ZERO"),
    (
        "07_runtime_index_out_of_range",
        "AIPO_RT_INDEX_OUT_OF_RANGE",
    ),
    ("10_runtime_failure_uncaught", "AIPO_RT_FAILURE_UNCAUGHT"),
];

#[test]
fn test_check_diagnostics_match_goldens() {
    let _guard = lock();
    for (stem, code, location) in CHECK_CASES {
        let relative = format!("../../docs/conformance/diagnostics/{stem}.aipo");
        let (exit, stdout, stderr) = check_human(&relative);
        assert_eq!(exit, 1, "{stem}: language failure exit");
        assert!(stdout.is_empty(), "{stem}: stdout stays empty on failure");
        assert!(
            stderr.contains(code),
            "{stem}: human rendering names {code}"
        );
        assert!(
            stderr.contains(location),
            "{stem}: human rendering locates {location}"
        );
        assert!(!stderr.contains('\u{1b}'), "{stem}: no ANSI escapes");
        expect_snapshot(stem, "human.txt", &stderr);

        let (exit, stdout, stderr) = check_jsonl(&relative);
        assert_eq!(exit, 1, "{stem}: jsonl exit agrees");
        assert!(stdout.is_empty(), "{stem}: jsonl keeps stdout empty");
        expect_snapshot(stem, "jsonl", &stderr);
        for line in stderr.lines() {
            assert!(
                line.contains("\"code\"")
                    && line.contains("\"severity\"")
                    && line.contains("\"message\"")
                    && line.contains("\"primary_span\"")
                    && line.contains("\"notes\"")
                    && line.contains("\"suggestions\""),
                "{stem}: every JSONL line carries the full schema"
            );
            assert!(!line.contains('\u{1b}'), "{stem}: JSONL has no escapes");
        }
    }
}

#[test]
fn test_run_faults_render_with_notes() {
    let _guard = lock();
    for (stem, code) in RUN_CASES {
        let relative = format!("../../docs/conformance/diagnostics/{stem}.aipo");
        let (exit, _stdout, stderr) = run_human(&relative);
        assert_eq!(exit, 1, "{stem}: fault exit");
        assert!(stderr.contains(code), "{stem}: names {code}");
        assert!(
            stderr.contains("note:"),
            "{stem}: carries the while-running note"
        );
        assert!(!stderr.contains('\u{1b}'), "{stem}: no ANSI escapes");
        expect_snapshot(stem, "human.txt", &stderr);
    }
}

#[test]
fn test_spanless_module_error_renders_without_location() {
    let _guard = lock();
    let (exit, _stdout, stderr) = check_human("../../docs/conformance/modules/cycle/entry.aipo");
    assert_eq!(exit, 1);
    assert!(stderr.contains("AIPO_SEM_IMPORT_CYCLE"));
    assert!(
        stderr.starts_with("error:"),
        "spanless diagnostics start at the message:\n{stderr}"
    );
    expect_snapshot("cycle_import", "human.txt", &stderr);
}

#[test]
fn test_nesting_cascade_is_controlled() {
    let _guard = sink_guard();
    let mut deep = String::new();
    for _ in 0..600 {
        deep.push_str("if true\n");
    }
    deep.push_str("io.println(1)\n");
    for _ in 0..600 {
        deep.push_str("end\n");
    }
    let path = std::env::temp_dir().join("aipo-ui-nesting.aipo");
    std::fs::write(&path, &deep).expect("scratch is writable");
    let args = vec![
        "check".to_string(),
        path.to_string_lossy().into_owned(),
        "--message-format=human".to_string(),
    ];
    let mut out: Vec<u8> = Vec::new();
    let mut err: Vec<u8> = Vec::new();
    let exit = aipo_cli::run_with(&args, &mut out, &mut err);
    let _ = std::fs::remove_file(&path);
    let stderr = String::from_utf8_lossy(&err).into_owned();
    assert_eq!(exit, 1);
    assert_eq!(
        stderr.matches("error:").count(),
        1,
        "exactly one error, no cascade:\n{}",
        &stderr[..stderr.len().min(600)]
    );
    assert!(stderr.contains("AIPO_PARSE_NESTING_TOO_DEEP"));
}

fn sink_guard() -> MutexGuard<'static, ()> {
    lock()
}
