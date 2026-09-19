//! Fuzz smoke over the whole pipeline.
//!
//! The MVP gate requires that no Rust panic escapes as a user-facing error. This test drives
//! the real `aipo check` and `aipo run` paths with two kinds of garbage:
//!
//! 1. uniformly random bytes, which stress the lexer and the early parser;
//! 2. single-byte mutations of the conformance programs, which reach semantics, IR, bytecode
//!    verification and execution.
//!
//! The generator is a deterministic xorshift PRNG, so a failure is always reproducible from
//! the seed printed in the assertion message.
//!
//! This is a smoke test, not a proof: it samples the input space instead of covering it.

use std::io::Write;
use std::panic::{self, AssertUnwindSafe};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

/// Deterministic PRNG state; xorshift64* is enough for input sampling.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            return 0;
        }
        #[allow(clippy::cast_possible_truncation)]
        let value = (self.next_u64() % bound as u64) as usize;
        value
    }

    fn byte(&mut self) -> u8 {
        #[allow(clippy::cast_possible_truncation)]
        let value = (self.next_u64() & 0xFF) as u8;
        value
    }
}

fn sink_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[derive(Clone)]
struct NullSink(Arc<Mutex<Vec<u8>>>);

impl Write for NullSink {
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

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/conformance")
}

fn seed_programs() -> Vec<String> {
    let mut sources = Vec::new();
    for subdirectory in ["programs", "diagnostics"] {
        let directory = corpus_dir().join(subdirectory);
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.to_string_lossy().ends_with(".aipo"))
            .collect();
        paths.sort();
        for path in paths {
            if let Ok(text) = std::fs::read_to_string(&path) {
                sources.push(text);
            }
        }
    }
    assert!(
        !sources.is_empty(),
        "fuzz seeds need at least one conformance program to mutate"
    );
    sources
}

/// Writes `source` to a scratch file and runs the CLI against it, asserting no panic.
fn drive_pipeline(source: &str, seed: u64, iteration: usize, scratch: &PathBuf) {
    std::fs::write(scratch, source).expect("scratch file is writable");

    let captured = Arc::new(Mutex::new(Vec::new()));
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut out = NullSink(Arc::clone(&captured));
        let mut err = NullSink(Arc::clone(&captured));
        let args = vec![
            "check".to_string(),
            scratch.to_string_lossy().into_owned(),
            "--message-format=jsonl".to_string(),
        ];
        aipo_cli::run_with(&args, &mut out, &mut err)
    }));

    let code = result.unwrap_or_else(|payload| {
        let message = payload
            .downcast_ref::<&str>()
            .map(|text| (*text).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "<non-string panic payload>".to_string());
        panic!("panic escaped the pipeline (seed {seed}, iteration {iteration}): {message}\ninput:\n{source}");
    });

    assert!(
        code <= 2,
        "seed {seed}, iteration {iteration} produced exit code {code}\ninput:\n{source}"
    );
}

#[test]
fn test_random_bytes_never_panic_the_pipeline() {
    let _serialized = sink_lock();
    aipo_cli::set_output_sink(Some(Box::new(NullSink(Arc::new(Mutex::new(Vec::new()))))));
    let scratch = std::env::temp_dir().join("aipo-fuzz-random.aipo");
    let seed = 0xA1F0_C0DE_u64 ^ 0x5EED;

    let mut rng = Rng::new(seed);
    let mut accepted = 0usize;
    for iteration in 0..256 {
        let len = rng.below(384);
        let source: Vec<u8> = (0..len).map(|_| rng.byte()).collect();
        let source = String::from_utf8_lossy(&source).into_owned();
        drive_pipeline(&source, seed, iteration, &scratch);
        if !source.trim().is_empty() {
            accepted += 1;
        }
    }

    let _ = std::fs::remove_file(&scratch);
    assert!(accepted > 0, "the generator must actually produce input");
}

#[test]
fn test_mutated_programs_never_panic_the_pipeline() {
    let _serialized = sink_lock();
    aipo_cli::set_output_sink(Some(Box::new(NullSink(Arc::new(Mutex::new(Vec::new()))))));
    let scratch = std::env::temp_dir().join("aipo-fuzz-mutation.aipo");
    let seeds = seed_programs();
    let seed = 0x1234_5678_9ABC_DEF0_u64;

    let mut rng = Rng::new(seed);
    for iteration in 0..512 {
        let base = &seeds[rng.below(seeds.len())];
        let mut bytes = base.clone().into_bytes();
        if bytes.is_empty() {
            continue;
        }
        // One to three mutations: replace, insert or delete a byte.
        for _ in 0..=rng.below(3) {
            let position = rng.below(bytes.len());
            match rng.below(3) {
                0 => bytes[position] = rng.byte(),
                1 => bytes.insert(position, rng.byte()),
                _ => {
                    bytes.remove(position);
                }
            }
            if bytes.is_empty() {
                break;
            }
        }
        let source = String::from_utf8_lossy(&bytes).into_owned();
        drive_pipeline(&source, seed, iteration, &scratch);
    }

    let _ = std::fs::remove_file(&scratch);
}

#[test]
fn test_truncated_programs_never_panic_the_pipeline() {
    let _serialized = sink_lock();
    aipo_cli::set_output_sink(Some(Box::new(NullSink(Arc::new(Mutex::new(Vec::new()))))));
    let scratch = std::env::temp_dir().join("aipo-fuzz-truncation.aipo");
    let seeds = seed_programs();
    let seed = 0xDEAD_BEEF_CAFE_F00D_u64;

    for (base_index, base) in seeds.iter().enumerate() {
        for cut in [
            0,
            1,
            2,
            base.len() / 3,
            base.len() / 2,
            base.len().saturating_sub(1),
        ] {
            let truncated = base.chars().take(cut).collect::<String>();
            drive_pipeline(&truncated, seed, base_index * 8 + cut, &scratch);
        }
    }

    let _ = std::fs::remove_file(&scratch);
}

/// Token dictionary for grammar-aware mutations: keywords, operators and block
/// scaffolding that push mutated programs into parser recovery, HIR lowering,
/// semantic analysis and formatting instead of merely breaking the lexer.
fn token_dictionary() -> Vec<&'static str> {
    vec![
        "fn f()\nend\n",
        "if x\nend\n",
        "return ",
        "let ",
        "var ",
        "do ",
        "end\n",
        "  ",
        "\"",
        "f\"",
        "{",
        "}",
        "[",
        "]",
        "(",
        ")",
        "..",
        "|>",
        "or_else",
        "attempt\nfailed e\nend\n",
        "self!",
        "!",
        "?.",
        "\n\n\n",
        "struct T\nend\n",
        "impl T\nend\n",
    ]
}

/// Applies 1–3 grammar-level mutations to `base`: line replacement, deletion or
/// duplication with dictionary tokens, `end`-stripping, or splicing two seeds.
fn grammar_mutate(rng: &mut Rng, seeds: &[String], base: &str) -> String {
    let dict = token_dictionary();
    let mut lines: Vec<String> = base.lines().map(|line| format!("{line}\n")).collect();
    if lines.is_empty() {
        lines.push(String::new());
    }
    for _ in 0..=rng.below(3) {
        match rng.below(6) {
            0 => {
                let at = rng.below(lines.len());
                lines[at] = dict[rng.below(dict.len())].to_string();
            }
            1 => {
                if lines.len() > 1 {
                    lines.remove(rng.below(lines.len()));
                }
            }
            2 => {
                let at = rng.below(lines.len());
                lines.insert(at, dict[rng.below(dict.len())].to_string());
            }
            3 => {
                for line in lines.iter_mut() {
                    if line.trim() == "end" && rng.below(2) == 0 {
                        line.clear();
                    }
                }
            }
            4 => {
                let other = &seeds[rng.below(seeds.len())];
                let other_lines: Vec<&str> = other.lines().collect();
                let cut_self = rng.below(lines.len() + 1);
                let cut_other = rng.below(other_lines.len() + 1);
                let mut spliced: Vec<String> = lines[..cut_self.min(lines.len())].to_vec();
                for line in &other_lines[cut_other.min(other_lines.len())..] {
                    spliced.push(format!("{line}\n"));
                }
                lines = spliced;
            }
            _ => {
                let at = rng.below(lines.len());
                lines[at].push_str(dict[rng.below(dict.len())]);
            }
        }
        if lines.is_empty() {
            lines.push(String::new());
        }
    }
    lines.concat()
}

/// Drives `check` and `fmt --check` (both in-process, neither writes nor executes).
fn drive_static_paths(source: &str, seed: u64, iteration: usize, scratch: &PathBuf) {
    std::fs::write(scratch, source).expect("scratch file is writable");
    for command in [
        vec![
            "check".to_string(),
            scratch.to_string_lossy().into_owned(),
            "--message-format=jsonl".to_string(),
        ],
        vec![
            "fmt".to_string(),
            scratch.to_string_lossy().into_owned(),
            "--check".to_string(),
        ],
    ] {
        let result = panic::catch_unwind(AssertUnwindSafe(|| {
            let mut out = NullSink(Arc::new(Mutex::new(Vec::new())));
            let mut err = NullSink(Arc::new(Mutex::new(Vec::new())));
            aipo_cli::run_with(&command, &mut out, &mut err)
        }));
        let code = result.unwrap_or_else(|payload| {
            let message = payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "<non-string panic payload>".to_string());
            panic!(
                "panic escaped the static path (seed {seed}, iteration {iteration}): {message}\ninput:\n{source}"
            );
        });
        assert!(
            code <= 2,
            "seed {seed}, iteration {iteration} produced exit code {code}\ninput:\n{source}"
        );
    }
}

#[test]
fn test_grammar_mutations_never_panic_static_paths() {
    let _serialized = sink_lock();
    aipo_cli::set_output_sink(Some(Box::new(NullSink(Arc::new(Mutex::new(Vec::new()))))));
    let scratch = std::env::temp_dir().join("aipo-fuzz-grammar.aipo");
    let seeds = seed_programs();
    let seed = 0x6A1F_0000_u64 ^ 0xF025 ^ 0xABCD;

    let mut rng = Rng::new(seed);
    for iteration in 0..512 {
        let base = &seeds[rng.below(seeds.len())];
        let mutated = grammar_mutate(&mut rng, &seeds, base);
        drive_static_paths(&mutated, seed, iteration, &scratch);
    }

    let _ = std::fs::remove_file(&scratch);
}

/// Executes mutated seeds in a subprocess under `timeout` so a mutated infinite
/// loop fails the test (exit 124) instead of hanging it. Asserts the VM never
/// panics (a panic prints `thread 'main' panicked` on stderr) and the exit code
/// is a contract code (0/1/2).
#[test]
fn test_mutated_programs_never_panic_at_run() {
    let binary = env!("CARGO_BIN_EXE_aipo");
    let seeds = seed_programs();
    let seed = 0x9E57_0000_u64 ^ 0xE1EC ^ 0x1234;

    let mut rng = Rng::new(seed);
    let scratch = std::env::temp_dir().join("aipo-fuzz-run.aipo");
    for iteration in 0..128 {
        let base = &seeds[rng.below(seeds.len())];
        let mut bytes = base.clone().into_bytes();
        for _ in 0..=rng.below(3) {
            if bytes.is_empty() {
                break;
            }
            let position = rng.below(bytes.len());
            match rng.below(3) {
                0 => bytes[position] = rng.byte(),
                1 => bytes.insert(position, rng.byte()),
                _ => {
                    bytes.remove(position);
                }
            }
        }
        let source = String::from_utf8_lossy(&bytes).into_owned();
        std::fs::write(&scratch, &source).expect("scratch file is writable");
        let output = std::process::Command::new("timeout")
            .arg("10")
            .arg(binary)
            .arg("run")
            .arg(&scratch)
            .output()
            .expect("timeout runs the binary");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !stderr.contains("panicked"),
            "VM panic at run (seed {seed}, iteration {iteration}):\n{stderr}\ninput:\n{source}"
        );
        assert!(
            output.status.success() || output.status.code().is_some_and(|c| c <= 2),
            "seed {seed}, iteration {iteration} exited with {:?}:\n{stderr}\ninput:\n{source}",
            output.status.code()
        );
    }

    let _ = std::fs::remove_file(&scratch);
}
