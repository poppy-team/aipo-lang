//! Span and source location definitions.

use std::fmt;

/// A half-open interval `[start, end)` representing a byte range in UTF-8 source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceSpan {
    /// The start byte offset (inclusive).
    pub start: usize,
    /// The end byte offset (exclusive).
    pub end: usize,
}

impl SourceSpan {
    /// Creates a new span covering `[start, end)`.
    ///
    /// If `start > end`, `end` is adjusted to equal `start` to preserve invariants.
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        if start > end {
            Self { start, end: start }
        } else {
            Self { start, end }
        }
    }

    /// Creates an empty span at a specific byte offset.
    #[must_use]
    pub const fn empty(offset: usize) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }

    /// Returns the length of this span in bytes.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns true if this span is empty (start == end).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Merges this span with another, returning the smallest span enclosing both.
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Checks if a byte offset falls within this span (`start <= offset < end`).
    #[must_use]
    pub const fn contains_offset(&self, offset: usize) -> bool {
        offset >= self.start && offset < self.end
    }
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}..{}", self.start, self.end)
    }
}

/// A human-readable 1-based source location (line, column) derived from byte offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceLocation {
    /// 1-based line number.
    pub line: usize,
    /// 1-based column number (Unicode scalar count from line start).
    pub column: usize,
    /// Byte offset within the source.
    pub offset: usize,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.line, self.column)
    }
}
