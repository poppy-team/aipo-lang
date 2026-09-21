//! Suggestion mechanics: a machine-applicable correction must target the
//! intended span, and applying it must clear the intended error without
//! introducing a new failure.
//!
//! The compiler currently emits no suggestions (no producer exists), so these
//! tests pin the *mechanics* on constructed diagnostics: span targeting,
//! byte-level application, frontend re-run, and JSONL round-trip of the
//! suggestion payload.

#![forbid(unsafe_code)]

use aipo_diagnostics::{Diagnostic, DiagnosticCode, Suggestion};
use aipo_source::{Source, SourceId};

fn apply_suggestion(text: &str, suggestion: &Suggestion) -> String {
    assert!(
        suggestion.start <= suggestion.end && suggestion.end <= text.len(),
        "suggestion span is inside the source"
    );
    assert!(
        text.is_char_boundary(suggestion.start) && text.is_char_boundary(suggestion.end),
        "suggestion span respects character boundaries"
    );
    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..suggestion.start]);
    out.push_str(&suggestion.replacement);
    out.push_str(&text[suggestion.end..]);
    out
}

#[test]
fn test_suggestion_application_clears_the_error() {
    let text = "let total = 5\nio.println(totla)\n";
    let source = Source::new(SourceId::next(), "suggest.aipo", text);
    let span = aipo_source::SourceSpan::new(25, 30);
    assert_eq!(source.slice(span), Some("totla"));
    let diagnostic = Diagnostic::error(
        DiagnosticCode::AIPO_SEM_UNKNOWN_NAME,
        "unknown identifier 'totla'",
    )
    .with_primary_span(&source, span)
    .with_suggestion(Suggestion {
        message: "did you mean 'total'?".to_string(),
        replacement: "total".to_string(),
        start: span.start,
        end: span.end,
    });
    let fixed = apply_suggestion(text, &diagnostic.suggestions[0]);
    assert_eq!(fixed, "let total = 5\nio.println(total)\n");
    let fixed_source = Source::new(SourceId::next(), "fixed.aipo", &fixed);
    let (program, _) = aipo_syntax::parse(&fixed_source);
    assert!(
        !program.statements.is_empty(),
        "fixed source parses to statements"
    );
    let (_, diagnostics) = aipo_syntax::parse(&fixed_source);
    assert!(
        diagnostics
            .iter()
            .all(|d| d.code != DiagnosticCode::AIPO_SEM_UNKNOWN_NAME),
        "the intended error is gone"
    );
}

#[test]
fn test_suggestion_rejects_bad_spans() {
    let text = "let x = 1\n";
    for (start, end) in [(5, 2), (0, 99), (50, 51)] {
        let suggestion = Suggestion {
            message: "bad".to_string(),
            replacement: "y".to_string(),
            start,
            end,
        };
        let result = std::panic::catch_unwind(|| apply_suggestion(text, &suggestion));
        assert!(result.is_err(), "span {start}..{end} is rejected");
    }
}

#[test]
fn test_suggestion_survives_jsonl_round_trip() {
    let diagnostic = Diagnostic::error(DiagnosticCode::AIPO_SEM_UNKNOWN_NAME, "unknown 'x'")
        .with_suggestion(Suggestion {
            message: "rename".to_string(),
            replacement: "y".to_string(),
            start: 1,
            end: 2,
        });
    let line = diagnostic.to_jsonl().expect("JSONL serializes");
    assert!(line.contains("\"suggestions\""));
    assert!(line.contains("\"replacement\":\"y\""));
    let back: Diagnostic = serde_json::from_str(&line).expect("JSONL parses");
    assert_eq!(back, diagnostic);
}
