//! Entry point for the `aipo` command-line tool.
//!
//! All behavior lives in `aipo_cli::run_with`; this binary only forwards the process
//! arguments and maps the returned exit code.

#![forbid(unsafe_code)]

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    aipo_cli::run(&args)
}
