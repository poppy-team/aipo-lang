//! `aipo-diagnostics` provides stable diagnostic models, catalog codes,
//! and JSONL serialization for compiler errors, warnings, and runtime faults.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod code;
pub mod diagnostic;
pub mod emitter;
pub mod severity;

pub use code::DiagnosticCode;
pub use diagnostic::{Diagnostic, PrimarySpan, Suggestion};
pub use emitter::{DiagnosticEmitter, MessageFormat};
pub use severity::Severity;

#[cfg(test)]
mod tests {
    use super::*;
    use aipo_source::{Source, SourceId, SourceSpan};

    #[test]
    fn test_diagnostic_jsonl_schema() {
        let text = "let x = 10\nlet y = 20\n";
        let source = Source::new(SourceId(1), "main.aipo", text);
        let span = SourceSpan::new(11, 16);

        let diag = Diagnostic::error(
            DiagnosticCode::AIPO_PARSE_UNCLOSED_BLOCK,
            "this block is not closed",
        )
        .with_primary_span(&source, span)
        .with_note("expected 'end' before EOF");

        let jsonl = diag.to_jsonl().unwrap();
        assert!(jsonl.contains("\"code\":\"AIPO_PARSE_UNCLOSED_BLOCK\""));
        assert!(jsonl.contains("\"severity\":\"error\""));
        assert!(jsonl.contains("\"file\":\"main.aipo\""));
        assert!(jsonl.contains("\"line\":2"));
        assert!(jsonl.contains("\"column\":1"));
    }

    #[test]
    fn test_human_rendering() {
        let text = "let x = 10\nlet y = 20\n";
        let mut map = aipo_source::SourceMap::new();
        map.add_source("main.aipo", text);
        let source = map.get_by_name("main.aipo").unwrap();

        let span = SourceSpan::new(11, 16);
        let diag = Diagnostic::error(
            DiagnosticCode::AIPO_SEM_UNKNOWN_NAME,
            "unknown variable 'y'",
        )
        .with_primary_span(source, span);

        let human = diag.render_human(Some(&map));
        assert!(
            human.contains("main.aipo:2:1: error: [AIPO_SEM_UNKNOWN_NAME] unknown variable 'y'")
        );
        assert!(human.contains("2 | let y = 20"));
    }

    #[test]
    fn test_emitter_jsonl() {
        let diag = Diagnostic::fault(DiagnosticCode::AIPO_RT_OVERFLOW, "integer overflow");
        let emitter = DiagnosticEmitter::new(MessageFormat::Jsonl, None);

        let mut buffer = Vec::new();
        emitter.emit(&mut buffer, &diag).unwrap();

        let output = String::from_utf8(buffer).unwrap();
        assert!(output.ends_with('\n'));
        assert!(output.contains("\"code\":\"AIPO_RT_OVERFLOW\""));
        assert!(output.contains("\"severity\":\"fault\""));
    }
}
