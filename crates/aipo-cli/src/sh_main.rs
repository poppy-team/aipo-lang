//! Entry point for `aipo-sh`, the standalone lightweight Aipo shell and script runner (Marco 4 / ADP-014).
//!
//! Features:
//! - Sub-millisecond cold start
//! - Direct in-memory string evaluation (`aipo-sh -c "code"`)
//! - Script execution (`aipo-sh script.aipo`)
//! - Interactive REPL (`aipo-sh`)

#![forbid(unsafe_code)]

use std::io::{self, BufRead, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        return run_repl();
    }

    match args[0].as_str() {
        "-h" | "--help" => {
            println!("aipo-sh — lightweight Aipo shell and script runner");
            println!();
            println!("USAGE:");
            println!("    aipo-sh <script.aipo> [args...]");
            println!("    aipo-sh -c <code>");
            println!("    aipo-sh");
            println!("    aipo-sh --version");
            println!("    aipo-sh --help");
            ExitCode::SUCCESS
        }
        "-v" | "--version" => {
            println!("aipo-sh {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        "-c" => {
            if args.len() < 2 {
                eprintln!("error: -c requires a code string argument");
                return ExitCode::from(2);
            }
            let code = &args[1];
            let code_u8 = aipo_cli::eval_source(code, &mut io::stdout(), &mut io::stderr());
            ExitCode::from(code_u8)
        }
        _ => {
            let mut run_args = vec!["run".to_string()];
            run_args.extend(args);
            let code_u8 = aipo_cli::run_with(&run_args, &mut io::stdout(), &mut io::stderr());
            ExitCode::from(code_u8)
        }
    }
}

fn run_repl() -> ExitCode {
    println!("Aipo Shell (aipo-sh) v{}", env!("CARGO_PKG_VERSION"));
    println!("Type 'exit' to quit.");

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();

    loop {
        print!("aipo> ");
        let _ = stdout.flush();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == "exit" || trimmed == "quit" {
                    break;
                }
                aipo_cli::eval_source(trimmed, &mut stdout, &mut stderr);
            }
            Err(e) => {
                eprintln!("error reading input: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
