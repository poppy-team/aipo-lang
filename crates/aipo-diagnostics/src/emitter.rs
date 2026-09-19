//! Diagnostic emitters for streaming output to writers.

use aipo_source::SourceMap;
use std::io::{self, Write};

use crate::diagnostic::Diagnostic;

/// Format for emitting diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MessageFormat {
    /// Human readable compiler-style text.
    #[default]
    Human,
    /// JSON Lines format (one JSON object per diagnostic line).
    Jsonl,
}

/// Emits diagnostics to any `Write` stream in either Human or JSONL format.
pub struct DiagnosticEmitter<'a> {
    source_map: Option<&'a SourceMap>,
    format: MessageFormat,
}

impl<'a> DiagnosticEmitter<'a> {
    /// Creates a new emitter with the specified format and optional source map.
    #[must_use]
    pub fn new(format: MessageFormat, source_map: Option<&'a SourceMap>) -> Self {
        Self { source_map, format }
    }

    /// Emits a single diagnostic to the provided writer.
    ///
    /// Accepts unsized writers such as `&mut dyn Write`, so callers can pass a
    /// type-erased stream (stdout/stderr handles, test buffers) directly.
    pub fn emit<W: Write + ?Sized>(
        &self,
        writer: &mut W,
        diagnostic: &Diagnostic,
    ) -> io::Result<()> {
        match self.format {
            MessageFormat::Human => {
                let rendered = diagnostic.render_human(self.source_map);
                writer.write_all(rendered.as_bytes())
            }
            MessageFormat::Jsonl => {
                let json = diagnostic
                    .to_jsonl()
                    .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))?;
                writeln!(writer, "{json}")
            }
        }
    }

    /// Emits a batch of diagnostics to the provided writer.
    pub fn emit_all<W: Write + ?Sized>(
        &self,
        writer: &mut W,
        diagnostics: &[Diagnostic],
    ) -> io::Result<()> {
        for diag in diagnostics {
            self.emit(writer, diag)?;
        }
        Ok(())
    }
}
