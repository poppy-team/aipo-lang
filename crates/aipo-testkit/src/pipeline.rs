//! In-process pipeline runners: parse, check, compile and execute.
//!
//! The standard library output sink is process-global. Any test that captures
//! program output through [`run_capture`] must serialize against other sink
//! users (the CLI conformance suite documents the same lock discipline).

use aipo_bytecode::BytecodeModule;
use aipo_diagnostics::{Diagnostic, Severity};
use aipo_ir::CoreModule;
use aipo_sema::PreludeSurface;
use aipo_source::{Source, SourceId};
use aipo_vm::{Vm, VmError};

pub use aipo_vm::VmMetrics;

use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// `true` when no diagnostic has error severity.
#[must_use]
pub fn is_clean(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().all(|d| d.severity != Severity::Error)
}

/// Prelude surface derived from the real stdlib registration, so `check` and
/// `run` agree by construction (same derivation as the CLI).
#[must_use]
pub fn prelude_surface() -> PreludeSurface {
    let mut vm = Vm::new();
    let mut registry = aipo_runtime::NativeRegistry::new();
    aipo_stdlib::register_stdlib(&mut vm, &mut registry);
    let mut surface = PreludeSurface::fundamental();
    for name in vm.globals.keys() {
        match registry.get(None, name) {
            Some(meta) => surface.add_function(name, meta.arity, meta.arity),
            None => surface.add_variable(name),
        }
    }
    surface
}

/// Parses and checks `text`, returning the lowered HIR program on success.
pub fn check_text(
    name: &str,
    text: &str,
) -> Result<(Source, aipo_hir::HirProgram), Vec<Diagnostic>> {
    let source = Source::new(SourceId::next(), name, text);
    let (program, mut diagnostics) = aipo_syntax::parse(&source);
    if !is_clean(&diagnostics) {
        return Err(diagnostics);
    }
    let hir = aipo_hir::lower(program);
    let surface = prelude_surface();
    let (_, sema_diagnostics) = aipo_sema::check_with_prelude(&source, &hir, &surface);
    diagnostics.extend(sema_diagnostics);
    if !is_clean(&diagnostics) {
        return Err(diagnostics);
    }
    Ok((source, hir))
}

/// Full frontend through Core IR.
pub fn lower_to_ir(name: &str, text: &str) -> Result<(Source, CoreModule), Vec<Diagnostic>> {
    let (source, hir) = check_text(name, text)?;
    Ok((source, aipo_ir::lower_to_ir(&hir)))
}

/// Full pipeline through verified bytecode.
pub fn compile_text(name: &str, text: &str) -> Result<(Source, BytecodeModule), PipelineFailure> {
    let (source, hir) = check_text(name, text).map_err(PipelineFailure::Diagnostics)?;
    let ir = aipo_ir::lower_to_ir(&hir);
    let bytecode = aipo_bytecode::compile(&ir).map_err(PipelineFailure::Bytecode)?;
    Ok((source, bytecode))
}

/// Pipeline failure: either frontend diagnostics or bytecode verification errors.
#[derive(Debug)]
pub enum PipelineFailure {
    /// Frontend (parse/semantic) diagnostics.
    Diagnostics(Vec<Diagnostic>),
    /// Bytecode verifier messages.
    Bytecode(Vec<String>),
}

const MAX_CAPTURE_BYTES: usize = 1024 * 1024;

/// Writer adapter capturing stdlib output into shared bytes.
struct CaptureWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
    limit: usize,
}

impl Write for CaptureWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut output = self
            .buffer
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if output.len().saturating_add(buf.len()) > self.limit {
            return Err(std::io::Error::other("captured output exceeded the limit"));
        }
        output.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[allow(missing_docs)]
pub struct RunCaptureReport {
    pub value: aipo_vm::Value,
    pub stdout: String,
    pub setup: Duration,
    pub execution: Duration,
    pub metrics: VmMetrics,
}

/// Executes verified bytecode with output captured, returning `(result, stdout)`.
pub fn run_capture(module: &BytecodeModule) -> Result<(aipo_vm::Value, String), VmError> {
    let report = run_capture_internal(module, false)?;
    Ok((report.value, report.stdout))
}

#[allow(missing_docs)]
pub fn run_capture_split(module: &BytecodeModule) -> Result<RunCaptureReport, VmError> {
    run_capture_internal(module, false)
}

#[allow(missing_docs)]
pub fn run_capture_split_with_metrics(
    module: &BytecodeModule,
) -> Result<RunCaptureReport, VmError> {
    run_capture_internal(module, true)
}

fn run_capture_internal(
    module: &BytecodeModule,
    collect_metrics: bool,
) -> Result<RunCaptureReport, VmError> {
    let setup_start = Instant::now();
    let buffer = Arc::new(Mutex::new(Vec::<u8>::new()));
    aipo_stdlib::io::set_output_sink(Some(Box::new(CaptureWriter {
        buffer: buffer.clone(),
        limit: MAX_CAPTURE_BYTES,
    })));
    let mut vm = build_vm(module);
    if collect_metrics {
        vm.enable_metrics();
    }
    let setup = setup_start.elapsed();
    let execution_start = Instant::now();
    let outcome = vm.run(module);
    let execution = execution_start.elapsed();
    aipo_stdlib::io::set_output_sink(None);
    let bytes = buffer
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .clone();
    let stdout = String::from_utf8(bytes).unwrap_or_default();
    let value = outcome?;
    Ok(RunCaptureReport {
        value,
        stdout,
        setup,
        execution,
        metrics: vm.metrics(),
    })
}

fn build_vm(module: &BytecodeModule) -> Vm {
    let mut vm = Vm::new();
    let mut registry = aipo_runtime::NativeRegistry::new();
    aipo_stdlib::register_stdlib(&mut vm, &mut registry);
    for decl in &module.structs {
        let fields: Vec<(&str, bool)> = decl
            .fields
            .iter()
            .map(|(name, fixed)| (name.as_str(), *fixed))
            .collect();
        vm.register_struct(decl.name.clone(), fields);
    }
    for function in &module.functions {
        if let Some((type_name, method)) = function.name.split_once('.') {
            vm.register_struct_method(
                type_name,
                method,
                function.entry_ip,
                function.params,
                function.is_async,
            );
        }
    }
    vm
}

/// Compiles and runs `text`, returning the captured stdout.
///
/// The entry value matches `aipo run` semantics: an uncaught `Failure` is an
/// error, a fault is an error, anything else yields its printed output.
pub fn run_text(name: &str, text: &str) -> Result<String, PipelineFailure> {
    let (_, bytecode) = compile_text(name, text)?;
    run_capture(&bytecode)
        .map(|(_, stdout)| stdout)
        .map_err(|error| {
            PipelineFailure::Diagnostics(vec![aipo_diagnostics::Diagnostic::error(
                error.diagnostic_code(),
                error.to_string(),
            )])
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_writer_rejects_oversized_output() {
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let mut writer = CaptureWriter { buffer, limit: 4 };
        assert!(writer.write(b"12345").is_err());
    }
}
