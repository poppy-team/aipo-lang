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
}

impl VmError {
    /// Maps the VM error to its normative diagnostic code.
    #[must_use]
    pub fn diagnostic_code(&self) -> DiagnosticCode {
        match self {
            Self::Fault(fault) => fault.diagnostic_code(),
            Self::UncaughtFailure(_) => DiagnosticCode::AIPO_RT_FAILURE_UNCAUGHT,
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
        }
    }
}

impl std::error::Error for VmError {}

impl From<VmFault> for VmError {
    fn from(fault: VmFault) -> Self {
        Self::Fault(fault)
    }
}
