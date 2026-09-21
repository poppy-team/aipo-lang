//! Resource-exhaustion suite: hostile-but-terminating inputs in-process,
//! non-terminating inputs through the portable watchdog.
//!
//! Aipo source is untrusted input. Every case asserts the same contract:
//! termination (directly, or via watchdog kill), no panic, an exit code in
//! {0, 1, 2}, and a finite diagnostic count. Wall-clock bounds are generous
//! (seconds, not milliseconds) — these are termination tests, not benchmarks.

#![forbid(unsafe_code)]

use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

fn sink_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

#[derive(Clone)]
struct NullSink;

impl Write for NullSink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn scratch(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("aipo-resource-{name}.aipo"))
}

/// Runs `check` in-process under `catch_unwind`, asserting the contract.
fn assert_check_contract(label: &str, source: &str, budget: Duration) {
    let _guard = sink_lock();
    let path = scratch(label);
    std::fs::write(&path, source).expect("scratch is writable");
    let start = Instant::now();
    let result = std::panic::catch_unwind(|| {
        let mut out = NullSink;
        let mut err = NullSink;
        let args = vec![
            "check".to_string(),
            path.to_string_lossy().into_owned(),
            "--message-format=jsonl".to_string(),
        ];
        aipo_cli::run_with(&args, &mut out, &mut err)
    });
    let elapsed = start.elapsed();
    let _ = std::fs::remove_file(&path);
    let code = result.unwrap_or_else(|_| panic!("{label}: panic escaped"));
    assert!(code <= 2, "{label}: exit code {code} outside {{0,1,2}}");
    assert!(
        elapsed < budget,
        "{label}: took {elapsed:?}, budget {budget:?}"
    );
}

/// Runs `run` in-process (only for inputs that terminate by construction).
fn assert_run_contract(label: &str, source: &str, budget: Duration) {
    let _guard = sink_lock();
    let path = scratch(label);
    std::fs::write(&path, source).expect("scratch is writable");
    aipo_cli::set_output_sink(Some(Box::new(NullSink)));
    let start = Instant::now();
    let result = std::panic::catch_unwind(|| {
        let mut out = NullSink;
        let mut err = NullSink;
        let args = vec!["run".to_string(), path.to_string_lossy().into_owned()];
        aipo_cli::run_with(&args, &mut out, &mut err)
    });
    let elapsed = start.elapsed();
    aipo_cli::set_output_sink(None);
    let _ = std::fs::remove_file(&path);
    let code = result.unwrap_or_else(|_| panic!("{label}: panic escaped"));
    assert!(code <= 2, "{label}: exit code {code} outside {{0,1,2}}");
    assert!(
        elapsed < budget,
        "{label}: took {elapsed:?}, budget {budget:?}"
    );
}

fn deep_nesting(open: &str, close: &str, depth: usize) -> String {
    let mut source = String::new();
    for _ in 0..depth {
        source.push_str(open);
        source.push('\n');
    }
    source.push_str("io.println(1)\n");
    for _ in 0..depth {
        source.push_str(close);
        source.push('\n');
    }
    source
}

#[test]
fn test_deep_nesting_terminates() {
    // Past the ADP-005 bounds these must fail with the dedicated diagnostic —
    // and, critically, terminate (they previously aborted the host process).
    for (label, source) in [
        ("nest-if", deep_nesting("if true", "end", 300)),
        ("nest-fn", deep_nesting("fn f()\nreturn 1", "end", 200)),
        (
            "nest-paren",
            format!(
                "io.println({})\n",
                "(".repeat(2000) + "1" + &")".repeat(2000)
            ),
        ),
    ] {
        let _guard = sink_lock();
        let path = scratch(label);
        std::fs::write(&path, &source).expect("scratch is writable");
        let mut err = Vec::<u8>::new();
        let mut out = NullSink;
        let args = vec![
            "check".to_string(),
            path.to_string_lossy().into_owned(),
            "--message-format=jsonl".to_string(),
        ];
        let code = aipo_cli::run_with(&args, &mut out, &mut err);
        let _ = std::fs::remove_file(&path);
        assert_eq!(code, 1, "{label}: deep nesting is a language failure");
        let stderr = String::from_utf8_lossy(&err);
        assert!(
            stderr.contains("AIPO_PARSE_NESTING_TOO_DEEP"),
            "{label}: reports the nesting bound"
        );
        assert!(!stderr.contains("panicked"), "{label}: no panic");
    }
}

#[test]
fn test_giant_tokens_terminate() {
    let long_ident = "v".repeat(20_000);
    assert_check_contract(
        "long-ident",
        &format!("let {long_ident} = 1\nio.println({long_ident})\n"),
        Duration::from_secs(30),
    );
    let long_string = "a".repeat(500_000);
    assert_check_contract(
        "long-string",
        &format!("io.println(\"{long_string}\")\n"),
        Duration::from_secs(30),
    );
    let big_comment = format!("#{}\nlet x = 1\n", "z".repeat(500_000));
    assert_check_contract("big-comment", &big_comment, Duration::from_secs(30));
    let big_number = format!("io.println({})\n", "9".repeat(5000));
    assert_check_contract("big-number", &big_number, Duration::from_secs(30));
}

#[test]
fn test_mass_declarations_terminate() {
    let mut source = String::new();
    for index in 0..5_000 {
        source.push_str(&format!("let v{index} = {index}\n"));
    }
    source.push_str("io.println(v4999)\n");
    assert_check_contract("many-decls", &source, Duration::from_secs(60));
}

#[test]
fn test_wide_calls_terminate() {
    let params: Vec<String> = (0..150).map(|i| format!("p{i}")).collect();
    let args: Vec<String> = (0..150).map(|i| i.to_string()).collect();
    let source = format!(
        "fn wide({}) \n return 0\nend\nio.println(wide({}))\n",
        params.join(", "),
        args.join(", ")
    );
    assert_check_contract("wide-call", &source, Duration::from_secs(30));
}

#[test]
fn test_large_collection_literal_terminates() {
    let items: Vec<String> = (0..20_000).map(|i| i.to_string()).collect();
    let source = format!("io.println(len([{}]))\n", items.join(", "));
    assert_run_contract("big-list", &source, Duration::from_secs(60));
}

#[test]
fn test_pathological_strings_terminate() {
    assert_check_contract(
        "unterminated",
        &format!("io.println(\"{}\n", "q".repeat(100_000)),
        Duration::from_secs(30),
    );
    let mut interp = String::from("io.println(f\"");
    for index in 0..500 {
        interp.push_str(&format!("{{{index}}}"));
    }
    interp.push_str("\")\n");
    assert_check_contract("interp-heavy", &interp, Duration::from_secs(30));
}

#[test]
fn test_diagnostic_cascade_is_finite() {
    let mut source = String::new();
    for index in 0..500 {
        source.push_str(&format!("this is not valid line {index} !!!\n"));
    }
    let _guard = sink_lock();
    let path = scratch("cascade");
    std::fs::write(&path, &source).expect("scratch is writable");
    let mut err = Vec::<u8>::new();
    let mut out = NullSink;
    let args = vec![
        "check".to_string(),
        path.to_string_lossy().into_owned(),
        "--message-format=jsonl".to_string(),
    ];
    let code = aipo_cli::run_with(&args, &mut out, &mut err);
    let _ = std::fs::remove_file(&path);
    assert_eq!(code, 1);
    let lines = err.split(|b| *b == b'\n').filter(|l| !l.is_empty()).count();
    assert!(lines < 10_000, "cascade bounded: {lines} diagnostics");
    assert!(lines > 0, "cascade reports something");
}

#[test]
fn test_deep_import_chain_terminates() {
    let _guard = sink_lock();
    let dir = std::env::temp_dir().join("aipo-resource-chain");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("chain dir is writable");
    for index in 0..25 {
        let next = if index + 1 < 25 {
            format!("import m{}\n", index + 1)
        } else {
            String::new()
        };
        std::fs::write(
            dir.join(format!("m{index}.aipo")),
            format!("{next}export let x{index} = {index}\n"),
        )
        .expect("module is writable");
    }
    std::fs::write(dir.join("entry.aipo"), "import m0\nio.println(m0.x0)\n")
        .expect("entry is writable");
    let start = Instant::now();
    let result = std::panic::catch_unwind(|| {
        let mut out = NullSink;
        let mut err = NullSink;
        let args = vec![
            "run".to_string(),
            dir.join("entry.aipo").to_string_lossy().into_owned(),
        ];
        aipo_cli::run_with(&args, &mut out, &mut err)
    });
    let elapsed = start.elapsed();
    let _ = std::fs::remove_dir_all(&dir);
    let code = result.unwrap_or_else(|_| panic!("import chain: panic escaped"));
    assert!(code <= 2, "import chain exit {code}");
    assert!(elapsed < Duration::from_secs(60), "chain took {elapsed:?}");
}

/// Non-terminating programs (infinite loop, unbounded recursion, runaway
/// allocation) execute in a subprocess killed by the portable watchdog.
/// There is no in-VM fuel budget (see ADP-003); this test proves the harness
/// terminates anyway and the VM reports stack exhaustion as a fault.
#[test]
fn test_nonterminating_programs_are_bounded_externally() {
    let binary = env!("CARGO_BIN_EXE_aipo");
    let dir = std::env::temp_dir().join("aipo-resource-hang");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("hang dir is writable");
    std::fs::write(dir.join("loop.aipo"), "loop\nend\n").expect("writable");
    std::fs::write(
        dir.join("recursion.aipo"),
        "fn f(n)\nreturn f(n + 1)\nend\nf(0)\n",
    )
    .expect("writable");

    // Infinite loop: must be killed by the watchdog, never hang the suite.
    let looped = aipo_testkit::proc::run_bounded(
        binary,
        &["run", "loop.aipo"],
        &dir,
        Duration::from_secs(10),
    );
    assert!(looped.timed_out, "infinite loop is killed, not hung");
    assert!(
        !looped.stderr.contains("panicked"),
        "no panic in infinite loop"
    );

    // Unbounded recursion: the operand-stack limit faults quickly in-process.
    let rec = aipo_testkit::proc::run_bounded(
        binary,
        &["run", "recursion.aipo"],
        &dir,
        Duration::from_secs(30),
    );
    assert!(!rec.timed_out, "recursion faults instead of hanging");
    assert_eq!(rec.code, Some(1), "recursion is a language failure");
    assert!(
        rec.stderr.contains("AIPO_RT_OVERFLOW"),
        "recursion reports overflow:\n{}",
        rec.stderr
    );
    assert!(!rec.stderr.contains("panicked"), "no panic in recursion");
    let _ = std::fs::remove_dir_all(&dir);
}
