//! Runtime faults and error representation for the Aipo VM.

use aipo_diagnostics::DiagnosticCode;
use std::fmt;

/// Unrecoverable runtime faults representing programming or contract violations.
#[derive(Debug, Clone, PartialEq)]
pub enum VmFault {
    /// Integer arithmetic exceeded safe range ±(2^53 - 1).
    Overflow {
        /// Explanation of operation that caused overflow.
        details: String,
    },
    /// Float operation resulted in NaN or Infinity.
    NonFiniteFloat,
    /// Division or modulo by zero.
    DivisionByZero,
    /// Index out of range for collection or string.
    IndexOutOfRange {
        /// Attempted index.
        index: i64,
        /// Length of collection.
        len: usize,
    },
    /// Key not found in strict dictionary lookup.
    KeyNotFound {
        /// Missing key name.
        key: String,
    },
    /// Target is not callable.
    NotCallable {
        /// Uncallable type name.
        type_name: String,
    },
    /// Collection was structurally modified while being iterated.
    MutationDuringIteration,
    /// Runtime operand or contract type mismatch.
    TypeMismatch {
        /// Expected type name.
        expected: String,
        /// Actual encountered type name.
        actual: String,
    },
    /// Stack underflow occurred during bytecode execution.
    StackUnderflow,
    /// Global variable is not defined.
    UndefinedGlobal {
        /// Name of the missing global.
        name: String,
    },
    /// Field does not exist on the struct instance.
    NoSuchField {
        /// Struct type name.
        type_name: String,
        /// Name of the missing field.
        field: String,
    },
    /// Attempt to reassign a `fixed` field after construction.
    FixedFieldMutation {
        /// Struct type name.
        type_name: String,
        /// Name of the fixed field.
        field: String,
    },
    /// A signature contract (`name: Type`, `name!: Type`, `-> T`, `T?`) was violated at a
    /// call boundary.
    ///
    /// Canon classifies a contract violation discovered only at runtime as a programming
    /// fault, not a recoverable `Failure`, so it is not capturable by `attempt`.
    ContractViolation {
        /// Where the contract is written (`parameter \`x\``, `return`).
        position: String,
        /// Declared contract, including `?` when `none` is accepted.
        expected: String,
        /// Runtime category of the offending value.
        actual: String,
    },
    /// Struct invariant validation failed.
    InvariantViolation {
        /// Struct type name.
        type_name: String,
        /// Validation failure message.
        message: String,
    },
    /// Cancelled task driven or awaited: cancellation is a fault, never a
    /// recoverable `Failure`, so it is not capturable by `attempt`.
    Cancelled {
        /// Description of what was cancelled.
        details: String,
    },
    /// Task awaiting itself, directly or transitively: depth-first driving
    /// would never complete, so the cycle faults instead of hanging.
    AwaitCycle {
        /// Chain of task ids forming the cycle.
        chain: Vec<u64>,
    },
    /// Blocking operation (`await`, `sleep`, join) executed inside a
    /// synchronous callback: `invoke` runs the callback on the host Rust
    /// stack, which the scheduler cannot suspend and resume. Await the task
    /// outside the callback instead.
    AwaitInCallback {
        /// Which blocking operation was attempted.
        operation: String,
    },
    /// Host operation attempted without the capability it requires.
    ///
    /// Canon: capabilities are deny-by-default for a sandboxed host and a denial is a
    /// fault, not a recoverable `Failure`, so it is not capturable by `attempt`.
    CapabilityDenied {
        /// The capability that was required, as a dotted path.
        capability: String,
        /// The host operation that required it.
        operation: String,
    },
    /// Host handle addressed after its slot was released or reused.
    ///
    /// Canon forbids using freed host state, and the declared contract decides whether a
    /// stale handle is `none` or this fault; the fault path is not capturable.
    StaleHandle {
        /// The handle that no longer addresses a live host object.
        handle: String,
    },
    /// A binding created inside a scoped host callback reached a heap-publication point
    /// outside the scope it belongs to.
    ScopeEscape {
        /// The binding that tried to escape.
        binding: String,
    },
    /// Corrupted bytecode instruction encountered.
    CorruptedBytecode {
        /// Offset where error occurred.
        offset: usize,
        /// Description of the corruption.
        reason: String,
    },
}

impl VmFault {
    /// Maps the runtime fault to its normative diagnostic code.
    #[must_use]
    pub fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            Self::Overflow { .. } => DiagnosticCode::AIPO_RT_OVERFLOW,
            Self::NonFiniteFloat => DiagnosticCode::AIPO_RT_NON_FINITE_FLOAT,
            Self::DivisionByZero => DiagnosticCode::AIPO_RT_DIV_ZERO,
            Self::IndexOutOfRange { .. } => DiagnosticCode::AIPO_RT_INDEX_OUT_OF_RANGE,
            Self::KeyNotFound { .. } => DiagnosticCode::AIPO_RT_KEY_NOT_FOUND,
            Self::NotCallable { .. } => DiagnosticCode::AIPO_RT_NOT_CALLABLE,
            Self::MutationDuringIteration => DiagnosticCode::AIPO_RT_MUTATION_DURING_ITERATION,
            Self::TypeMismatch { .. }
            | Self::StackUnderflow
            | Self::UndefinedGlobal { .. }
            | Self::NoSuchField { .. }
            | Self::FixedFieldMutation { .. }
            | Self::InvariantViolation { .. }
            | Self::ContractViolation { .. }
            | Self::CorruptedBytecode { .. } => DiagnosticCode::AIPO_RT_TYPE_MISMATCH,
            Self::Cancelled { .. } => DiagnosticCode::AIPO_RT_CANCELLED,
            Self::AwaitCycle { .. } => DiagnosticCode::AIPO_RT_AWAIT_CYCLE,
            Self::AwaitInCallback { .. } => DiagnosticCode::AIPO_RT_AWAIT_IN_CALLBACK,
            Self::CapabilityDenied { .. } => DiagnosticCode::AIPO_RT_CAPABILITY_DENIED,
            Self::StaleHandle { .. } => DiagnosticCode::AIPO_RT_STALE_HANDLE,
            Self::ScopeEscape { .. } => DiagnosticCode::AIPO_RT_SCOPE_ESCAPE,
        }
    }
}

impl fmt::Display for VmFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow { details } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_OVERFLOW]: integer overflow: {details}"
                )
            }
            Self::NonFiniteFloat => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_NON_FINITE_FLOAT]: non-finite float result (NaN or Inf)"
                )
            }
            Self::DivisionByZero => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_DIV_ZERO]: division or modulo by zero"
                )
            }
            Self::IndexOutOfRange { index, len } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_INDEX_OUT_OF_RANGE]: index {index} out of range (length {len})"
                )
            }
            Self::KeyNotFound { key } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_KEY_NOT_FOUND]: key '{key}' not found in dictionary"
                )
            }
            Self::NotCallable { type_name } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_NOT_CALLABLE]: value of type '{type_name}' is not callable"
                )
            }
            Self::MutationDuringIteration => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_MUTATION_DURING_ITERATION]: structural collection mutation during iteration"
                )
            }
            Self::TypeMismatch { expected, actual } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_TYPE_MISMATCH]: expected {expected}, found {actual}"
                )
            }
            Self::StackUnderflow => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_TYPE_MISMATCH]: operand stack underflow"
                )
            }
            Self::UndefinedGlobal { name } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_TYPE_MISMATCH]: undefined global variable '{name}'"
                )
            }
            Self::NoSuchField { type_name, field } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_TYPE_MISMATCH]: struct '{type_name}' has no field '{field}'"
                )
            }
            Self::FixedFieldMutation { type_name, field } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_TYPE_MISMATCH]: cannot mutate fixed field '{field}' of struct '{type_name}'"
                )
            }
            Self::InvariantViolation { type_name, message } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_TYPE_MISMATCH]: invariant violation for '{type_name}': {message}"
                )
            }
            Self::ContractViolation {
                position,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_TYPE_MISMATCH]: contract violation at {position}: expected {expected}, got {actual}"
                )
            }
            Self::CorruptedBytecode { offset, reason } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_TYPE_MISMATCH]: corrupted bytecode at offset {offset}: {reason}"
                )
            }
            Self::Cancelled { details } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_CANCELLED]: cancelled task: {details}"
                )
            }
            Self::AwaitCycle { chain } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_AWAIT_CYCLE]: task await cycle: {}",
                    chain
                        .iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<_>>()
                        .join(" -> ")
                )
            }
            Self::AwaitInCallback { operation } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_AWAIT_IN_CALLBACK]: {operation} inside a synchronous callback cannot suspend; await outside the callback"
                )
            }
            Self::CapabilityDenied {
                capability,
                operation,
            } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_CAPABILITY_DENIED]: operation '{operation}' requires capability '{capability}', which this host does not grant"
                )
            }
            Self::StaleHandle { handle } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_STALE_HANDLE]: {handle} is stale: the host object it referenced was released"
                )
            }
            Self::ScopeEscape { binding } => {
                write!(
                    f,
                    "runtime fault [AIPO_RT_SCOPE_ESCAPE]: binding '{binding}' was created in a scoped host callback and cannot leave it"
                )
            }
        }
    }
}

impl std::error::Error for VmFault {}

/// Complete error envelope returned by the VM.
#[derive(Debug, Clone, PartialEq)]
pub enum VmError {
    /// Non-recoverable programming fault.
    Fault(VmFault),
    /// Uncaught recoverable failure reached top-level execution.
    UncaughtFailure(String),
    /// Internal control signal: the running task suspended (await, sleep, join)
    /// and the machine must pick another runnable task.
    ///
    /// Blocking primitives suspend with the triggering instruction and the
    /// operand stack intact, so resuming re-executes the instruction once its
    /// target is terminal. The signal unwinds through internal `invoke` callers
    /// and is consumed by [`crate::Vm::run`]; it never surfaces to user code. A stray
    /// signal escaping `run` is converted to `CorruptedBytecode`.
    Suspended,
}

impl VmError {
    /// Maps the VM error to its normative diagnostic code.
    #[must_use]
    pub fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            Self::Fault(fault) => fault.diagnostic_code(),
            Self::UncaughtFailure(_) => DiagnosticCode::AIPO_RT_FAILURE_UNCAUGHT,
            // Internal control flow; mapped only so the envelope stays total.
            Self::Suspended => DiagnosticCode::AIPO_RT_FAILURE_UNCAUGHT,
        }
    }
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fault(fault) => write!(f, "{fault}"),
            Self::UncaughtFailure(msg) => {
                write!(
                    f,
                    "runtime failure [AIPO_RT_FAILURE_UNCAUGHT]: uncaught failure: {msg}"
                )
            }
            Self::Suspended => write!(f, "internal error: task suspension escaped the scheduler"),
        }
    }
}

impl std::error::Error for VmError {}

impl From<VmFault> for VmError {
    fn from(fault: VmFault) -> Self {
        Self::Fault(fault)
    }
}
