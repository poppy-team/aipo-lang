//! `aipo-cli` implements the stable command surface documented in
//! `docs/reference/cli.md`:
//!
//! ```text
//! aipo run <path> [--message-format=<human|jsonl>]
//! aipo check <path> [--message-format=<human|jsonl>]
//! aipo build <path> [--out <dir>] [--message-format=<human|jsonl>]
//! aipo fmt <paths...> [--check]
//! aipo --version
//! aipo --help
//! ```
//!
//! Exit codes are part of the contract:
//!
//! - `0` — success (the program ran to completion, `check` found no error, or formatting
//!   either wrote the files or verified them unchanged);
//! - `1` — language failure (lexical/parse/semantic diagnostic, bytecode verification
//!   error, uncaught runtime fault, or a `--check` run that would rewrite a file);
//! - `2` — usage error (unknown command or flag, missing argument, unreadable file).
//!
//! The command surface is a thin orchestrator over the existing pipeline stages:
//! `aipo-source → aipo-syntax → aipo-hir → aipo-sema → aipo-ir → aipo-bytecode → aipo-vm`,
//! with `aipo-stdlib` registered before execution.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use aipo_diagnostics::{Diagnostic, DiagnosticEmitter, MessageFormat, Severity};
use aipo_runtime::NativeRegistry;
use aipo_sema::PreludeSurface;
use aipo_source::{Source, SourceId, SourceMap};
use aipo_vm::{Vm, VmError};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

mod modules;

/// Redirects the standard library's `io` output away from the process streams.
///
/// Program output goes to standard output by default; an embedder (or an integration test)
/// can install a sink here to capture it.
pub use aipo_stdlib::io::set_output_sink;

/// Successful execution.
pub const EXIT_SUCCESS: u8 = 0;
/// Language failure: diagnostics, runtime fault, or `fmt --check` drift.
pub const EXIT_LANGUAGE_FAILURE: u8 = 1;
/// Usage failure: bad command line or unreadable input.
pub const EXIT_USAGE: u8 = 2;

/// Help text printed for `aipo --help`.
pub const USAGE: &str = "\
aipo — Aipo language toolchain

USAGE:
    aipo run <path> [--message-format=<human|jsonl>]
    aipo check <path> [--message-format=<human|jsonl>]
    aipo build <path> [--out <dir>] [--message-format=<human|jsonl>]
    aipo fmt <paths...> [--check]
    aipo --version
    aipo --help

COMMANDS:
    run      Compile and execute an Aipo source file
    check    Run the frontend, semantic analysis and bytecode verification
    build    Emit a JavaScript bundle (app.js + aipo-runtime.js + app.js.map)
    fmt      Format source files in place; --check reports drift without writing

EXIT CODES:
    0  success
    1  language failure (diagnostics, runtime fault, or formatting drift)
    2  usage error (bad arguments or unreadable file)
";

/// Runs the CLI for an argument vector that excludes the program name.
pub fn run(args: &[String]) -> ExitCode {
    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut out = stdout.lock();
    let mut err = stderr.lock();
    ExitCode::from(run_with(args, &mut out, &mut err))
}

/// Runs the CLI with explicit output streams, returning the process exit code.
///
/// Keeping the streams injectable makes the whole command surface testable without
/// spawning a process.
pub fn run_with(args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let parsed = match Command::parse(args) {
        Ok(command) => command,
        Err(usage) => {
            let _ = writeln!(err, "error: {usage}");
            let _ = writeln!(err, "\n{USAGE}");
            return EXIT_USAGE;
        }
    };

    match parsed {
        Command::Help => {
            let _ = write!(out, "{USAGE}");
            EXIT_SUCCESS
        }
        Command::Version => {
            let _ = writeln!(out, "aipo {}", env!("CARGO_PKG_VERSION"));
            EXIT_SUCCESS
        }
        Command::Run { path, format } => execute(&path, format, out, err, Action::Run),
        Command::Check { path, format } => execute(&path, format, out, err, Action::Check),
        Command::Build {
            path,
            out_dir,
            format,
        } => build_bundle(&path, out_dir.as_deref(), format, out, err),
        Command::Fmt { paths, check } => format_files(&paths, check, out, err),
    }
}

/// A parsed command line.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Help,
    Version,
    Run {
        path: PathBuf,
        format: MessageFormat,
    },
    Check {
        path: PathBuf,
        format: MessageFormat,
    },
    Build {
        path: PathBuf,
        out_dir: Option<PathBuf>,
        format: MessageFormat,
    },
    Fmt {
        paths: Vec<PathBuf>,
        check: bool,
    },
}

impl Command {
    fn parse(args: &[String]) -> Result<Self, String> {
        let Some(first) = args.first() else {
            return Err("missing command".to_string());
        };

        match first.as_str() {
            "--help" | "-h" | "help" => Ok(Self::Help),
            "--version" | "-V" | "version" => Ok(Self::Version),
            "run" | "check" => {
                let mut path = None;
                let mut format = MessageFormat::Human;
                let rest = &args[1..];
                let mut index = 0;
                while index < rest.len() {
                    let arg = &rest[index];
                    if let Some(value) = arg.strip_prefix("--message-format=") {
                        format = parse_format(value)?;
                    } else if arg == "--message-format" {
                        index += 1;
                        let value = rest
                            .get(index)
                            .ok_or_else(|| "--message-format requires a value".to_string())?;
                        format = parse_format(value)?;
                    } else if arg.starts_with('-') {
                        return Err(format!("unrecognized flag '{arg}'"));
                    } else if path.is_some() {
                        return Err(format!("unexpected argument '{arg}'"));
                    } else {
                        path = Some(PathBuf::from(arg));
                    }
                    index += 1;
                }

                let path = path.ok_or_else(|| format!("'{first}' requires a path argument"))?;
                if first == "run" {
                    Ok(Self::Run { path, format })
                } else {
                    Ok(Self::Check { path, format })
                }
            }
            "build" => {
                let mut path = None;
                let mut out_dir = None;
                let mut format = MessageFormat::Human;
                let rest = &args[1..];
                let mut index = 0;
                while index < rest.len() {
                    let arg = &rest[index];
                    if let Some(value) = arg.strip_prefix("--message-format=") {
                        format = parse_format(value)?;
                    } else if arg == "--message-format" {
                        index += 1;
                        let value = rest
                            .get(index)
                            .ok_or_else(|| "--message-format requires a value".to_string())?;
                        format = parse_format(value)?;
                    } else if arg == "--out" {
                        index += 1;
                        let value = rest
                            .get(index)
                            .ok_or_else(|| "'--out' requires a directory argument".to_string())?;
                        out_dir = Some(PathBuf::from(value));
                    } else if let Some(value) = arg.strip_prefix("--out=") {
                        out_dir = Some(PathBuf::from(value));
                    } else if arg.starts_with('-') {
                        return Err(format!("unrecognized flag '{arg}'"));
                    } else if path.is_some() {
                        return Err(format!("unexpected argument '{arg}'"));
                    } else {
                        path = Some(PathBuf::from(arg));
                    }
                    index += 1;
                }

                let path = path.ok_or_else(|| "'build' requires a path argument".to_string())?;
                Ok(Self::Build {
                    path,
                    out_dir,
                    format,
                })
            }
            "fmt" => {
                let mut paths = Vec::new();
                let mut check = false;
                for arg in &args[1..] {
                    if arg == "--check" {
                        check = true;
                    } else if arg.starts_with('-') {
                        return Err(format!("unrecognized flag '{arg}'"));
                    } else {
                        paths.push(PathBuf::from(arg));
                    }
                }
                if paths.is_empty() {
                    return Err("'fmt' requires at least one path argument".to_string());
                }
                Ok(Self::Fmt { paths, check })
            }
            other => Err(format!("unrecognized command '{other}'")),
        }
    }
}

fn parse_format(value: &str) -> Result<MessageFormat, String> {
    match value {
        "human" => Ok(MessageFormat::Human),
        "jsonl" => Ok(MessageFormat::Jsonl),
        other => Err(format!(
            "invalid message format '{other}': expected 'human' or 'jsonl'"
        )),
    }
}

/// What a pipeline invocation should do after analysis succeeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Action {
    Run,
    Check,
}

/// Result of compiling a source file down to verified bytecode.
struct Compiled {
    module: aipo_bytecode::BytecodeModule,
    diagnostics: Vec<Diagnostic>,
}

/// Runs the frontend pipeline and reports diagnostics.
///
/// `analyze` never executes the program: `check` stops here, while `run` continues with
/// [`execute_module`].
fn analyze(source: &Source, path: &Path) -> Compiled {
    let (program, mut diagnostics) = aipo_syntax::parse(source);
    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        return Compiled {
            module: aipo_bytecode::BytecodeModule::new(),
            diagnostics,
        };
    }

    // Every module reachable from this file is merged first, so semantic analysis sees one
    // program with the imported declarations already in place.
    let hir = aipo_hir::lower(program);
    let resolved = modules::resolve(path, hir);
    diagnostics.extend(resolved.diagnostics);
    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        return Compiled {
            module: aipo_bytecode::BytecodeModule::new(),
            diagnostics,
        };
    }
    let hir = resolved.program;

    let mut surface = prelude_surface();
    // Imported module names and aliases are ordinary globals to the checker.
    for name in &resolved.imported_names {
        surface.add_variable(name);
    }
    let (_, sema_diagnostics) = aipo_sema::check_with_prelude(source, &hir, &surface);
    diagnostics.extend(sema_diagnostics);
    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        return Compiled {
            module: aipo_bytecode::BytecodeModule::new(),
            diagnostics,
        };
    }

    let ir = aipo_ir::lower_to_ir(&hir);
    match aipo_bytecode::compile(&ir) {
        Ok(module) => Compiled {
            module,
            diagnostics,
        },
        Err(errors) => {
            for error in errors {
                diagnostics.push(verification_diagnostic(&error));
            }
            Compiled {
                module: aipo_bytecode::BytecodeModule::new(),
                diagnostics,
            }
        }
    }
}

fn verification_diagnostic(reason: &str) -> Diagnostic {
    let fault = aipo_vm::VmFault::CorruptedBytecode {
        offset: 0,
        reason: reason.to_string(),
    };
    Diagnostic::error(fault.diagnostic_code(), fault.to_string())
}

fn execute(
    path: &Path,
    format: MessageFormat,
    out: &mut dyn Write,
    err: &mut dyn Write,
    action: Action,
) -> u8 {
    let text = match read_source(path) {
        Ok(text) => text,
        Err(message) => {
            let _ = writeln!(err, "error: {message}");
            return EXIT_USAGE;
        }
    };

    let source = Source::new(SourceId::next(), path.display().to_string(), &text);
    let compiled = analyze(&source, path);
    let has_errors = compiled
        .diagnostics
        .iter()
        .any(|diag| diag.severity == Severity::Error);

    emit_diagnostics(format, &source, &compiled.diagnostics, out, err);

    if has_errors {
        return EXIT_LANGUAGE_FAILURE;
    }
    if action == Action::Check {
        return EXIT_SUCCESS;
    }

    match execute_module(&compiled.module) {
        Ok(()) => EXIT_SUCCESS,
        Err(error) => {
            let diagnostic = runtime_diagnostic(&source, &error);
            emit_diagnostics(format, &source, std::slice::from_ref(&diagnostic), out, err);
            EXIT_LANGUAGE_FAILURE
        }
    }
}

/// Emits a JavaScript bundle for an entry file.
///
/// Diagnostics use the same frontend as `run`/`check`, so a program that fails
/// `check` fails `build` with the same codes and no files are written.
#[allow(clippy::needless_pass_by_value)]
fn build_bundle(
    path: &Path,
    out_dir: Option<&Path>,
    format: MessageFormat,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> u8 {
    let text = match read_source(path) {
        Ok(text) => text,
        Err(message) => {
            let _ = writeln!(err, "error: {message}");
            return EXIT_USAGE;
        }
    };

    let source = Source::new(SourceId::next(), path.display().to_string(), &text);
    let (program, mut diagnostics) = aipo_syntax::parse(&source);
    if !diagnostics.iter().any(|d| d.severity == Severity::Error) {
        let hir = aipo_hir::lower(program);
        let resolved = modules::resolve(path, hir);
        diagnostics.extend(resolved.diagnostics);
        if !diagnostics.iter().any(|d| d.severity == Severity::Error) {
            let mut surface = prelude_surface();
            for name in &resolved.imported_names {
                surface.add_variable(name);
            }
            let (_, sema_diagnostics) =
                aipo_sema::check_with_prelude(&source, &resolved.program, &surface);
            diagnostics.extend(sema_diagnostics);
            if !diagnostics.iter().any(|d| d.severity == Severity::Error) {
                let ir = aipo_ir::lower_to_ir(&resolved.program);
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("main.aipo");
                let bundle = aipo_js::emit_js(file_name, &text, &ir);
                let dir: PathBuf = match out_dir {
                    Some(dir) => dir.to_path_buf(),
                    None => path
                        .parent()
                        .map_or_else(|| PathBuf::from("dist"), Path::to_path_buf)
                        .join("dist"),
                };
                if let Err(error) = std::fs::create_dir_all(&dir) {
                    let _ = writeln!(err, "error: {}: {error}", dir.display());
                    return EXIT_USAGE;
                }
                for (name, contents) in [
                    ("app.js", bundle.app_js.as_str()),
                    ("aipo-runtime.js", bundle.runtime_js.as_str()),
                    ("app.js.map", bundle.source_map.as_str()),
                ] {
                    if let Err(error) = std::fs::write(dir.join(name), contents) {
                        let _ = writeln!(err, "error: {}: {error}", dir.join(name).display());
                        return EXIT_USAGE;
                    }
                }
                let _ = writeln!(out, "built 3 files to {}", dir.display());
                return EXIT_SUCCESS;
            }
        }
    }

    emit_diagnostics(format, &source, &diagnostics, out, err);
    EXIT_LANGUAGE_FAILURE
}

/// Registers the standard library and runs a verified module.}
///
/// # Errors
/// Returns the runtime error raised by the program.
fn execute_module(module: &aipo_bytecode::BytecodeModule) -> Result<(), VmError> {
    let (mut vm, _) = standard_environment();
    // Struct layouts travel with the module: without them the runtime cannot name the
    // fields of a user-declared instance, and every `p.x` would fault.
    for decl in &module.structs {
        let fields: Vec<(&str, bool)> = decl
            .fields
            .iter()
            .map(|(name, fixed)| (name.as_str(), *fixed))
            .collect();
        vm.register_struct(decl.name.clone(), fields);
    }
    // `impl Type` methods are compiled as `Type.method` functions; the runtime needs the
    // entry points to answer `value.method(...)` with a bound method.
    for function in &module.functions {
        if let Some((type_name, method)) = function.name.split_once('.') {
            // `self` is parameter 0 and arrives as the receiver, so the total arity the
            // frame expects equals the declared parameter count.
            vm.register_struct_method(
                type_name,
                method,
                function.entry_ip,
                function.params,
                function.is_async,
            );
        }
    }
    vm.run(module).map(|_| ())
}

/// Builds a VM with the standard library registered, plus the registry of its metadata.
fn standard_environment() -> (Vm, NativeRegistry) {
    let mut vm = Vm::new();
    let mut registry = NativeRegistry::new();
    aipo_stdlib::register_stdlib(&mut vm, &mut registry);
    (vm, registry)
}

/// Derives the semantic surface from the standard library registration.
///
/// `check` and `run` therefore agree by construction: a name is accepted exactly when
/// the VM that will execute the program can resolve it. Module names (`math`, `string`,
/// `io`) become globals, and Prelude natives carry their cataloged arity so member calls
/// can be arity-checked.
fn prelude_surface() -> PreludeSurface {
    let (vm, registry) = standard_environment();
    let mut surface = PreludeSurface::fundamental();

    for name in vm.globals.keys() {
        match registry.get(None, name) {
            Some(meta) => surface.add_function(name, meta.arity, meta.arity),
            None => surface.add_variable(name),
        }
    }

    surface
}

fn runtime_diagnostic(source: &Source, error: &VmError) -> Diagnostic {
    Diagnostic::error(error.diagnostic_code(), error.to_string())
        .with_note(format!("while running {}", source.name()))
}

/// Emits diagnostics in the requested format.
///
/// Human diagnostics go to standard error, matching compiler convention; JSONL keeps the
/// schema documented in `docs/reference/cli.md`.
fn emit_diagnostics(
    format: MessageFormat,
    source: &Source,
    diagnostics: &[Diagnostic],
    out: &mut dyn Write,
    err: &mut dyn Write,
) {
    if diagnostics.is_empty() {
        return;
    }

    let mut map = SourceMap::new();
    map.add_source(source.name(), source.text());
    let emitter = DiagnosticEmitter::new(format, Some(&map));
    let _ = emitter.emit_all(err, diagnostics);
    let _ = out.flush();
}

/// Formats files in place, or verifies that they are already canonical.
fn format_files(paths: &[PathBuf], check: bool, out: &mut dyn Write, err: &mut dyn Write) -> u8 {
    let mut exit = EXIT_SUCCESS;
    let mut changed = 0usize;

    for path in paths {
        let text = match read_source(path) {
            Ok(text) => text,
            Err(message) => {
                let _ = writeln!(err, "error: {message}");
                return EXIT_USAGE;
            }
        };

        let formatted = match aipo_formatter::format_text(&path.display().to_string(), &text) {
            Ok(formatted) => formatted,
            Err(error) => {
                let _ = writeln!(err, "error: {}: {error}", path.display());
                exit = EXIT_LANGUAGE_FAILURE;
                continue;
            }
        };

        if formatted == text {
            continue;
        }

        changed += 1;
        if check {
            let _ = writeln!(err, "would reformat {}", path.display());
            exit = EXIT_LANGUAGE_FAILURE;
        } else if let Err(error) = std::fs::write(path, &formatted) {
            let _ = writeln!(err, "error: {}: {error}", path.display());
            exit = EXIT_USAGE;
        }
    }

    if !check {
        let verb = if changed == 1 { "file" } else { "files" };
        let _ = writeln!(out, "formatted {changed} {verb}");
    }
    exit
}

fn read_source(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))
}
