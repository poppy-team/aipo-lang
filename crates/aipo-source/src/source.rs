//! Source file representation with line indexing and span resolution.

use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::error::SourceError;
use crate::span::{SourceLocation, SourceSpan};

static NEXT_SOURCE_ID: AtomicU32 = AtomicU32::new(1);

/// Unique identifier for a loaded source file within a compilation session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceId(pub u32);

impl SourceId {
    /// Generates a new unique `SourceId`.
    #[must_use]
    pub fn next() -> Self {
        Self(NEXT_SOURCE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl fmt::Display for SourceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "#{}", self.0)
    }
}

/// A loaded and normalized source file.
///
/// Immutably holds the UTF-8 text with BOM stripped, newlines normalized to LF,
/// and a precomputed index of line start byte offsets for O(log L) location queries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    id: SourceId,
    name: String,
    text: String,
    line_starts: Vec<usize>,
}

impl Source {
    /// Constructs a new `Source`, normalizing BOM and newlines.
    ///
    /// - UTF-8 Byte Order Mark (`\u{FEFF}`) at the beginning is stripped.
    /// - CRLF (`\r\n`) and lone CR (`\r`) are normalized to LF (`\n`).
    /// - Line start byte offsets are precalculated.
    #[must_use]
    pub fn new(id: SourceId, name: impl Into<String>, raw_text: &str) -> Self {
        let name = name.into();

        // 1. Strip leading UTF-8 BOM if present.
        let without_bom = raw_text.strip_prefix('\u{FEFF}').unwrap_or(raw_text);

        // 2. Normalize CRLF and lone CR to LF.
        let text = normalize_newlines(without_bom);

        // 3. Precalculate line starts (0-indexed byte offsets where each line begins).
        let mut line_starts = Vec::new();
        line_starts.push(0);

        for (offset, byte) in text.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(offset + 1);
            }
        }

        Self {
            id,
            name,
            text,
            line_starts,
        }
    }

    /// Returns the unique identifier of this source.
    #[must_use]
    pub const fn id(&self) -> SourceId {
        self.id
    }

    /// Returns the file name or virtual path of this source.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the normalized UTF-8 text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the length of the source text in bytes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Returns true if the source text is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    /// Returns the total number of lines in this source (at least 1).
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Slices the source text according to the provided span.
    ///
    /// Returns `None` if the span exceeds the bounds of the source or splits a UTF-8 character.
    #[must_use]
    pub fn slice(&self, span: SourceSpan) -> Option<&str> {
        if span.end > self.text.len() || span.start > span.end {
            return None;
        }
        self.text.get(span.start..span.end)
    }

    /// Validates that a span is within bounds, returning an error if it is not.
    pub fn validate_span(&self, span: SourceSpan) -> Result<(), SourceError> {
        if span.end > self.text.len() || span.start > span.end {
            Err(SourceError::SpanOutOfBounds {
                start: span.start,
                end: span.end,
                source_len: self.text.len(),
            })
        } else {
            Ok(())
        }
    }

    /// Resolves a byte offset to a 1-based `SourceLocation` (line, column).
    ///
    /// Column numbers count Unicode scalar values (characters) from the beginning of the line.
    #[must_use]
    pub fn location(&self, offset: usize) -> Option<SourceLocation> {
        if offset > self.text.len() {
            return None;
        }

        // Binary search to find the line containing the offset:
        // line_idx is the index into self.line_starts where line_starts[line_idx] <= offset.
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(exact) => exact,
            Err(insert_idx) => insert_idx.saturating_sub(1),
        };

        let line_start = self.line_starts[line_idx];
        let line_slice = self.text.get(line_start..offset)?;
        let column = line_slice.chars().count() + 1;
        let line = line_idx + 1;

        Some(SourceLocation {
            line,
            column,
            offset,
        })
    }

    /// Returns the byte span for a 1-based line number (including trailing newline if present).
    #[must_use]
    pub fn line_span(&self, line: usize) -> Option<SourceSpan> {
        if line == 0 || line > self.line_starts.len() {
            return None;
        }

        let line_idx = line - 1;
        let start = self.line_starts[line_idx];
        let end = if line_idx + 1 < self.line_starts.len() {
            self.line_starts[line_idx + 1]
        } else {
            self.text.len()
        };

        Some(SourceSpan::new(start, end))
    }

    /// Returns the text of a 1-based line, stripped of its trailing newline.
    #[must_use]
    pub fn line_content(&self, line: usize) -> Option<&str> {
        let span = self.line_span(line)?;
        let raw = self.slice(span)?;
        Some(raw.strip_suffix('\n').unwrap_or(raw))
    }
}

fn normalize_newlines(input: &str) -> String {
    // Quick scan to avoid allocation if already normalized
    if !input.contains('\r') {
        return input.to_string();
    }

    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            result.push('\n');
        } else {
            result.push(ch);
        }
    }

    result
}
