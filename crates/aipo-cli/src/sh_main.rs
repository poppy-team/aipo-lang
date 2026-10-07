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
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    let (args, reg) = split_engine_flag(raw_args);

    if args.is_empty() {
        return run_repl(reg);
    }

    match args[0].as_str() {
        "-h" | "--help" => {
            println!("aipo-sh — lightweight Aipo shell and script runner");
            println!();
            println!("USAGE:");
            println!("    aipo-sh <script.aipo> [args...]");
            println!("    aipo-sh -c <code>");
            println!("    aipo-sh [--engine=<vm|reg>]");
            println!("    aipo-sh --version");
            println!("    aipo-sh --help");
            println!();
            println!("FLAGS:");
            println!("    --engine=<vm|reg>   Execution engine: stack VM (default) or register VM");
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
            let code_u8 = if reg {
                aipo_cli::eval_source_reg(code, &mut io::stdout(), &mut io::stderr())
            } else {
                aipo_cli::eval_source(code, &mut io::stdout(), &mut io::stderr())
            };
            ExitCode::from(code_u8)
        }
        _ => {
            let mut run_args = vec!["run".to_string()];
            if reg {
                run_args.push("--engine=reg".to_string());
            }
            run_args.extend(args);
            let code_u8 = aipo_cli::run_with(&run_args, &mut io::stdout(), &mut io::stderr());
            ExitCode::from(code_u8)
        }
    }
}

/// Extracts `--engine=<vm|reg>` / `--engine <vm|reg>` / `--reg` from shell args.
///
/// Returns the remaining args and `true` when the register VM was selected.
fn split_engine_flag(args: Vec<String>) -> (Vec<String>, bool) {
    let mut kept = Vec::with_capacity(args.len());
    let mut reg = false;
    let mut index = 0;
    while index < args.len() {
        let arg = &args[index];
        if arg == "--reg" {
            reg = true;
        } else if let Some(value) = arg.strip_prefix("--engine=") {
            match value {
                "reg" => reg = true,
                "vm" | "bytecode" | "stack" => reg = false,
                _ => {
                    eprintln!("error: unrecognized --engine '{value}': expected 'vm' or 'reg'");
                    std::process::exit(2);
                }
            }
        } else if arg == "--engine" {
            index += 1;
            match args.get(index).map(String::as_str) {
                Some("reg") => reg = true,
                Some("vm" | "bytecode" | "stack") => reg = false,
                _ => {
                    eprintln!("error: --engine requires 'vm' or 'reg'");
                    std::process::exit(2);
                }
            }
        } else {
            kept.push(arg.clone());
        }
        index += 1;
    }
    (kept, reg)
}

fn run_repl(reg: bool) -> ExitCode {
    if reg {
        println!(
            "Aipo Shell (aipo-sh) v{} [engine=reg]",
            env!("CARGO_PKG_VERSION")
        );
    } else {
        println!("Aipo Shell (aipo-sh) v{}", env!("CARGO_PKG_VERSION"));
    }
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
                if reg {
                    aipo_cli::eval_source_reg(trimmed, &mut stdout, &mut stderr);
                } else {
                    aipo_cli::eval_source(trimmed, &mut stdout, &mut stderr);
                }
            }
            Err(e) => {
                eprintln!("error reading input: {e}");
                return ExitCode::FAILURE;
            }
        }
    }

    ExitCode::SUCCESS
}
