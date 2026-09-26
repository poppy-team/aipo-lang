//! Compilation errors for the Aipo WebAssembly backend (ADP-013).

use aipo_source::SourceSpan;
use std::fmt;

/// Errors that can occur during WebAssembly compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmCompileError {
    /// An AST or HIR item is not yet supported by the Wasm backend.
    UnsupportedItem {
        /// Explanation of the unsupported item.
        message: String,
        /// Source span where the error occurred.
        span: SourceSpan,
    },
    /// A statement is not yet supported by the Wasm backend.
    UnsupportedStmt {
        /// Explanation of the unsupported statement.
        message: String,
        /// Source span where the error occurred.
        span: SourceSpan,
    },
    /// An expression is not yet supported by the Wasm backend.
    UnsupportedExpr {
        /// Explanation of the unsupported expression.
        message: String,
        /// Source span where the error occurred.
        span: SourceSpan,
    },
    /// An identifier was referenced that has not been declared.
    UnknownVariable {
        /// Name of the undeclared variable.
        name: String,
        /// Source span where the error occurred.
        span: SourceSpan,
    },
    /// A numeric or string literal could not be parsed.
    InvalidLiteral {
        /// Error message.
        message: String,
        /// Source span where the error occurred.
        span: SourceSpan,
    },
    /// Type mismatch encountered during code emission.
    TypeMismatch {
        /// Expected type name.
        expected: String,
        /// Found type name.
        found: String,
        /// Source span where the error occurred.
        span: SourceSpan,
    },
}

impl WasmCompileError {
    /// Returns the source span where the error occurred.
    #[must_use]
    pub const fn span(&self) -> SourceSpan {
        match self {
            Self::UnsupportedItem { span, .. }
            | Self::UnsupportedStmt { span, .. }
            | Self::UnsupportedExpr { span, .. }
            | Self::UnknownVariable { span, .. }
            | Self::InvalidLiteral { span, .. }
            | Self::TypeMismatch { span, .. } => *span,
        }
    }
}

impl fmt::Display for WasmCompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedItem { message, span } => {
                write!(f, "unsupported item at {}: {}", span, message)
            }
            Self::UnsupportedStmt { message, span } => {
                write!(f, "unsupported statement at {}: {}", span, message)
            }
            Self::UnsupportedExpr { message, span } => {
                write!(f, "unsupported expression at {}: {}", span, message)
            }
            Self::UnknownVariable { name, span } => {
                write!(f, "unknown variable `{}` at {}", name, span)
            }
            Self::InvalidLiteral { message, span } => {
                write!(f, "invalid literal at {}: {}", span, message)
            }
            Self::TypeMismatch {
                expected,
                found,
                span,
            } => {
                write!(
                    f,
                    "type mismatch at {}: expected `{}`, found `{}`",
                    span, expected, found
                )
            }
        }
    }
}

impl std::error::Error for WasmCompileError {}
