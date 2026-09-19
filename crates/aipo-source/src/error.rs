//! Error types for source loading and processing.

use std::fmt;

/// An error encountered when loading or processing source files.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SourceError {
    /// The file could not be read due to an I/O error.
    Io {
        /// The path to the file that failed to load.
        path: String,
        /// Description of the I/O failure.
        detail: String,
    },
    /// The source bytes were not valid UTF-8.
    InvalidUtf8 {
        /// The path or identifier of the source with invalid UTF-8.
        path: String,
        /// Description of the UTF-8 error.
        detail: String,
    },
    /// A requested span falls outside the bounds of the source text.
    SpanOutOfBounds {
        /// The start offset of the out-of-bounds span.
        start: usize,
        /// The end offset of the out-of-bounds span.
        end: usize,
        /// The actual length of the source text in bytes.
        source_len: usize,
    },
}

impl fmt::Display for SourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, detail } => {
                write!(formatter, "failed to read source file '{path}': {detail}")
            }
            Self::InvalidUtf8 { path, detail } => {
                write!(formatter, "invalid UTF-8 in source '{path}': {detail}")
            }
            Self::SpanOutOfBounds {
                start,
                end,
                source_len,
            } => {
                write!(
                    formatter,
                    "span [{start}..{end}) is out of bounds for source of length {source_len}"
                )
            }
        }
    }
}

impl std::error::Error for SourceError {}
