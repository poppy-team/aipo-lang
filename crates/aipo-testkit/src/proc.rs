//! Portable subprocess execution with a deadline.
//!
//! The historical fuzz harness shelled out to the Unix `timeout` command, which
//! does not exist on Windows. This helper polls the child and kills it through
//! the portable [`std::process::Child::kill`] API, so hang protection works on every platform
//! Rust and Node run on — without weakening it.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Outcome of a bounded execution.
pub struct BoundedOutput {
    /// Exit code when the child exited by itself (`None` = killed on timeout
    /// or terminated by signal).
    pub code: Option<i32>,
    /// Captured stdout (complete when `timed_out` is false).
    pub stdout: String,
    /// Captured stderr (complete when `timed_out` is false).
    pub stderr: String,
    /// `true` when the watchdog killed the child.
    pub timed_out: bool,
}

/// Spawns `program` with `args` in `current_dir`, killing it after `deadline`.
///
/// Standard output/error are piped (never inherited), so concurrent harnesses
/// cannot contaminate each other's captures. A program that floods its output
/// pipe is still killed on deadline: the watchdog does not wait for EOF.
pub fn run_bounded(
    program: &str,
    args: &[&str],
    current_dir: &std::path::Path,
    deadline: Duration,
) -> BoundedOutput {
    let mut child = Command::new(program)
        .args(args)
        .current_dir(current_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("child process starts");
    let start = Instant::now();
    loop {
        match child.try_wait().expect("child is pollable") {
            Some(status) => {
                let mut stdout = String::new();
                let mut stderr = String::new();
                if let Some(pipe) = child.stdout.as_mut() {
                    let _ = pipe.read_to_string(&mut stdout);
                }
                if let Some(pipe) = child.stderr.as_mut() {
                    let _ = pipe.read_to_string(&mut stderr);
                }
                return BoundedOutput {
                    code: status.code(),
                    stdout,
                    stderr,
                    timed_out: false,
                };
            }
            None => {
                if start.elapsed() >= deadline {
                    let _ = child.kill();
                    let status = child.wait().expect("child is reaped");
                    let mut stdout = String::new();
                    let mut stderr = String::new();
                    if let Some(pipe) = child.stdout.as_mut() {
                        let _ = pipe.read_to_string(&mut stdout);
                    }
                    if let Some(pipe) = child.stderr.as_mut() {
                        let _ = pipe.read_to_string(&mut stderr);
                    }
                    return BoundedOutput {
                        code: status.code(),
                        stdout,
                        stderr,
                        timed_out: true,
                    };
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }
}
