//! WebAssembly JIT runtime execution engine for Aipo programs using Wasmtime (ADP-013).

use std::io::Write;
#[cfg(feature = "wasmtime")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "wasmtime")]
use wasmtime::{Caller, Config, Engine, Linker, Module, Store};

/// Optional execution limits for a single Wasmtime invocation.
#[derive(Debug, Clone, Copy, Default)]
pub struct WasmExecutionOptions {
    /// Wasmtime instruction fuel; `None` preserves unlimited execution.
    /// Fuel does not bound native host work, memory, output or wall-clock time.
    pub fuel: Option<u64>,
}

/// Errors encountered during WebAssembly execution.
#[derive(Debug)]
pub enum WasmRuntimeError {
    /// Failure during module instantiation.
    Instantiation(String),
    /// Failure during function execution (trap or runtime fault).
    Execution(String),
    /// Exported symbol was not found in the module.
    MissingExport(String),
}

impl std::fmt::Display for WasmRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Instantiation(msg) => write!(f, "Wasm instantiation error: {msg}"),
            Self::Execution(msg) => {
                if msg.contains("divide by zero") {
                    write!(
                        f,
                        "[AIPO_RT_DIV_ZERO] runtime fault [AIPO_RT_DIV_ZERO]: division by zero"
                    )
                } else {
                    write!(f, "Wasm execution error: {msg}")
                }
            }
            Self::MissingExport(name) => write!(f, "Wasm missing export: `{name}`"),
        }
    }
}

impl std::error::Error for WasmRuntimeError {}

/// Disassembles WebAssembly binary bytes into the standard WebAssembly Text format (WAT).
///
/// # Errors
///
/// Returns an error string if the bytes are not valid WebAssembly binary format.
pub fn disassemble_wasm(wasm_bytes: &[u8]) -> Result<String, String> {
    wasmprinter::print_bytes(wasm_bytes).map_err(|e| e.to_string())
}

/// State held by the Wasmtime store during execution.
#[cfg(feature = "wasmtime")]
#[derive(Clone)]
struct HostState {
    output: Arc<Mutex<Vec<u8>>>,
}

#[cfg(feature = "wasmtime")]
impl HostState {
    fn write_str(&self, text: &str) {
        if let Ok(mut buf) = self.output.lock() {
            buf.extend_from_slice(text.as_bytes());
        }
    }
}

/// Executes a compiled WebAssembly module, routing host I/O (`print`, `println`) to `stdout`.
///
/// Looks for an entrypoint function in the following priority order:
/// 1. `__top_level__`
/// 2. `run`
/// 3. `main`
///
/// # Errors
///
/// Returns a [`WasmRuntimeError`] if instantiation fails or execution traps.
#[cfg(feature = "wasmtime")]
pub fn execute_wasm(wasm_bytes: &[u8], stdout: &mut dyn Write) -> Result<i64, WasmRuntimeError> {
    execute_wasm_with_options(wasm_bytes, stdout, WasmExecutionOptions::default())
}

/// Executes a module with optional instruction fuel and checked output delivery.
///
/// # Errors
/// Returns an error on invalid modules, exhausted fuel, missing entrypoints or writer errors.
#[cfg(feature = "wasmtime")]
pub fn execute_wasm_with_options(
    wasm_bytes: &[u8],
    stdout: &mut dyn Write,
    options: WasmExecutionOptions,
) -> Result<i64, WasmRuntimeError> {
    let mut config = Config::new();
    config.consume_fuel(options.fuel.is_some());
    let engine =
        Engine::new(&config).map_err(|e| WasmRuntimeError::Instantiation(e.to_string()))?;
    let module = Module::new(&engine, wasm_bytes)
        .map_err(|e| WasmRuntimeError::Instantiation(e.to_string()))?;

    let output = Arc::new(Mutex::new(Vec::new()));
    let state = HostState {
        output: Arc::clone(&output),
    };
    let mut store = Store::new(&engine, state);
    if let Some(fuel) = options.fuel {
        store
            .set_fuel(fuel)
            .map_err(|e| WasmRuntimeError::Instantiation(e.to_string()))?;
    }
    let mut linker = Linker::new(&engine);

    // Register host I/O functions under module "aipo_host"
    linker
        .func_wrap(
            "aipo_host",
            "print_int",
            |caller: Caller<'_, HostState>, val: i64| {
                caller.data().write_str(&val.to_string());
            },
        )
        .map_err(|e| WasmRuntimeError::Instantiation(e.to_string()))?;

    linker
        .func_wrap(
            "aipo_host",
            "print_float",
            |caller: Caller<'_, HostState>, val: f64| {
                if val.fract() == 0.0 && val.abs() < 1e15 {
                    caller.data().write_str(&format!("{val:.1}"));
                } else {
                    caller.data().write_str(&val.to_string());
                }
            },
        )
        .map_err(|e| WasmRuntimeError::Instantiation(e.to_string()))?;

    linker
        .func_wrap(
            "aipo_host",
            "print_str",
            |mut caller: Caller<'_, HostState>, ptr: i32| {
                let mem = match caller.get_export("memory").and_then(|e| e.into_memory()) {
                    Some(m) => m,
                    None => return,
                };
                let data = mem.data(&caller);
                let Ok(ptr) = usize::try_from(ptr) else {
                    return;
                };
                let Some(start) = ptr.checked_add(4) else {
                    return;
                };
                let Some(header) = data.get(ptr..start) else {
                    return;
                };
                let Ok(header) = <[u8; 4]>::try_from(header) else {
                    return;
                };
                let Ok(len) = usize::try_from(u32::from_le_bytes(header)) else {
                    return;
                };
                let Some(end) = start.checked_add(len) else {
                    return;
                };
                if let Some(bytes) = data.get(start..end) {
                    if let Ok(s) = std::str::from_utf8(bytes) {
                        caller.data().write_str(s);
                    }
                }
            },
        )
        .map_err(|e| WasmRuntimeError::Instantiation(e.to_string()))?;

    linker
        .func_wrap(
            "aipo_host",
            "print_bool",
            |caller: Caller<'_, HostState>, val: i32| {
                caller
                    .data()
                    .write_str(if val != 0 { "true" } else { "false" });
            },
        )
        .map_err(|e| WasmRuntimeError::Instantiation(e.to_string()))?;

    linker
        .func_wrap("aipo_host", "println", |caller: Caller<'_, HostState>| {
            caller.data().write_str("\n");
        })
        .map_err(|e| WasmRuntimeError::Instantiation(e.to_string()))?;

    let result = (|| -> Result<i64, WasmRuntimeError> {
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| WasmRuntimeError::Instantiation(e.to_string()))?;
        if let Ok(func) = instance.get_typed_func::<(), i64>(&mut store, "__top_level__") {
            return func
                .call(&mut store, ())
                .map_err(|e| WasmRuntimeError::Execution(format!("{e:#}")));
        }
        if let Ok(func) = instance.get_typed_func::<(), ()>(&mut store, "__top_level__") {
            func.call(&mut store, ())
                .map_err(|e| WasmRuntimeError::Execution(format!("{e:#}")))?;
            return Ok(0);
        }
        if let Ok(func) = instance.get_typed_func::<(), i64>(&mut store, "run") {
            return func
                .call(&mut store, ())
                .map_err(|e| WasmRuntimeError::Execution(format!("{e:#}")));
        }
        if let Ok(func) = instance.get_typed_func::<(), ()>(&mut store, "run") {
            func.call(&mut store, ())
                .map_err(|e| WasmRuntimeError::Execution(format!("{e:#}")))?;
            return Ok(0);
        }
        if let Ok(func) = instance.get_typed_func::<(), i64>(&mut store, "main") {
            return func
                .call(&mut store, ())
                .map_err(|e| WasmRuntimeError::Execution(format!("{e:#}")));
        }
        if let Ok(func) = instance.get_typed_func::<(), ()>(&mut store, "main") {
            func.call(&mut store, ())
                .map_err(|e| WasmRuntimeError::Execution(format!("{e:#}")))?;
            return Ok(0);
        }
        Err(WasmRuntimeError::MissingExport(
            "__top_level__, run or main with signature () -> i64 or () -> ()".to_string(),
        ))
    })();

    // Attempt to deliver output even after a guest trap. Preserve the original
    // execution error when execution and output delivery both fail.
    let output_result = (|| {
        let buf = output
            .lock()
            .map_err(|e| WasmRuntimeError::Execution(e.to_string()))?;
        stdout
            .write_all(&buf)
            .map_err(|e| WasmRuntimeError::Execution(e.to_string()))?;
        stdout
            .flush()
            .map_err(|e| WasmRuntimeError::Execution(e.to_string()))
    })();
    match result {
        Err(error) => Err(error),
        Ok(value) => output_result.map(|()| value),
    }
}

/// Fallback execution function when `wasmtime` is disabled.
///
/// # Errors
/// Always returns [`WasmRuntimeError::Execution`] indicating that the execution engine is disabled.
#[cfg(not(feature = "wasmtime"))]
pub fn execute_wasm(_wasm_bytes: &[u8], _stdout: &mut dyn Write) -> Result<i64, WasmRuntimeError> {
    Err(WasmRuntimeError::Execution(
        "WebAssembly JIT execution is disabled in this build (wasmtime feature not enabled)"
            .to_string(),
    ))
}

/// Fallback for builds without the Wasmtime execution engine.
///
/// # Errors
/// Always reports that JIT execution is disabled, regardless of the requested fuel.
#[cfg(not(feature = "wasmtime"))]
pub fn execute_wasm_with_options(
    wasm_bytes: &[u8],
    stdout: &mut dyn Write,
    _options: WasmExecutionOptions,
) -> Result<i64, WasmRuntimeError> {
    execute_wasm(wasm_bytes, stdout)
}
