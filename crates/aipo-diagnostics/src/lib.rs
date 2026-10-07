//! `aipo-diagnostics` provides stable diagnostic models, catalog codes,
//! and JSONL serialization for compiler errors, warnings, and runtime faults.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod code;
pub mod diagnostic;
pub mod emitter;
pub mod locale;
pub mod severity;

pub use code::DiagnosticCode;
pub use diagnostic::{Diagnostic, PrimarySpan, Suggestion};
pub use emitter::{DiagnosticEmitter, MessageFormat};
pub use locale::Locale;
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

        let human = diag.render_human_with_locale(Some(&map), Locale::En);
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

    #[test]
    fn test_i18n_locale_pt_br() {
        let text = "let x = 10\n";
        let source = Source::new(SourceId(1), "teste.aipo", text);
        let span = SourceSpan::new(4, 5);

        let diag = Diagnostic::error(
            DiagnosticCode::AIPO_SEM_UNKNOWN_NAME,
            "variável 'x' desconhecida",
        )
        .with_primary_span(&source, span)
        .with_note("verifique a declaração")
        .with_suggestion(Suggestion {
            message: "declare com let".to_string(),
            replacement: "let x".to_string(),
            start: 4,
            end: 5,
        });

        let rendered_pt = diag.render_human_with_locale(None, Locale::PtBr);
        assert!(
            rendered_pt.contains(
                "teste.aipo:1:5: erro: [AIPO_SEM_UNKNOWN_NAME] variável 'x' desconhecida"
            )
        );
        assert!(rendered_pt.contains("= nota: verifique a declaração"));
        assert!(rendered_pt.contains("= dica: declare com let: let x"));

        assert_eq!(
            DiagnosticCode::AIPO_SEM_UNKNOWN_NAME.title(Locale::En),
            "unknown identifier or unresolved name"
        );
        assert_eq!(
            DiagnosticCode::AIPO_SEM_UNKNOWN_NAME.title(Locale::PtBr),
            "identificador ou nome desconhecido"
        );
    }

    #[test]
    fn test_locale_detection() {
        assert_eq!(Locale::from_str("pt-BR"), Locale::PtBr);
        assert_eq!(Locale::from_str("pt_BR.UTF-8"), Locale::PtBr);
        assert_eq!(Locale::from_str("en_US.UTF-8"), Locale::En);
        assert_eq!(Locale::from_str("unknown"), Locale::En);
    }
}
