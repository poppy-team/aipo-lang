//! `aipo-lexer` provides tokenization, Unicode normalization,
//! and lexical diagnostics for the Aipo language.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod lexer;
pub mod number;
pub mod token;

pub use lexer::Lexer;
pub use number::{
    number_is_well_formed, parse_float_literal, parse_int_literal, strip_base_prefix,
};
pub use token::{StringPrefix, Token, TokenKind};

#[cfg(test)]
mod tests {
    use super::*;
    use aipo_diagnostics::DiagnosticCode;
    use aipo_source::{Source, SourceId};

    #[test]
    fn test_keywords_and_identifiers() {
        let code = "let var fixed fn struct impl interface satisfy init invariant if elif else then end match when loop while repeat as each in break continue return fail or_else attempt failed import export is not and or true false none self self! do div div=";
        let source = Source::new(SourceId(1), "test.aipo", code);
        let (tokens, diags) = Lexer::new(&source).tokenize();
        assert!(diags.is_empty());

        let kinds: Vec<_> = tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::Let,
                TokenKind::Var,
                TokenKind::Fixed,
                TokenKind::Fn,
                TokenKind::Struct,
                TokenKind::Impl,
                TokenKind::Interface,
                TokenKind::Satisfy,
                TokenKind::Init,
                TokenKind::Invariant,
                TokenKind::If,
                TokenKind::Elif,
                TokenKind::Else,
                TokenKind::Then,
                TokenKind::End,
                TokenKind::Match,
                TokenKind::When,
                TokenKind::Loop,
                TokenKind::While,
                TokenKind::Repeat,
                TokenKind::As,
                TokenKind::Each,
                TokenKind::In,
                TokenKind::Break,
                TokenKind::Continue,
                TokenKind::Return,
                TokenKind::Fail,
                TokenKind::OrElse,
                TokenKind::Attempt,
                TokenKind::Failed,
                TokenKind::Import,
                TokenKind::Export,
                TokenKind::Is,
                TokenKind::Not,
                TokenKind::And,
                TokenKind::Or,
                TokenKind::True,
                TokenKind::False,
                TokenKind::None,
                TokenKind::SelfVal,
                TokenKind::SelfMut,
                TokenKind::Do,
                TokenKind::Div,
                TokenKind::DivEq,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_numbers_and_ranges() {
        let code = "42 1_000_000 0xFF 0b1010 3.14 1.5e-3 1..10";
        let source = Source::new(SourceId(1), "test.aipo", code);
        let (tokens, diags) = Lexer::new(&source).tokenize();
        assert!(diags.is_empty());

        let kinds: Vec<_> = tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::IntLiteral("42".into()),
                TokenKind::IntLiteral("1_000_000".into()),
                TokenKind::IntLiteral("0xFF".into()),
                TokenKind::IntLiteral("0b1010".into()),
                TokenKind::FloatLiteral("3.14".into()),
                TokenKind::FloatLiteral("1.5e-3".into()),
                TokenKind::IntLiteral("1".into()),
                TokenKind::DotDot,
                TokenKind::IntLiteral("10".into()),
                TokenKind::Eof,
            ]
        );
    }

    /// Canon keeps number handling lexical, so a malformed literal is reported by the lexer
    /// and never reaches a later stage as a plausible number.
    #[test]
    fn test_malformed_numbers_report_invalid_number() {
        for code in [
            "0x", "0b102", "0o8", "1_", "1__0", "1e", "1e_5", "0x_FF", "1_.5", "1._5", "1.e5",
        ] {
            let source = Source::new(SourceId(1), "test.aipo", code);
            let (_, diags) = Lexer::new(&source).tokenize();
            assert!(
                diags
                    .iter()
                    .any(|d| d.code == DiagnosticCode::AIPO_LEX_INVALID_NUMBER),
                "'{code}' must report AIPO_LEX_INVALID_NUMBER, got {diags:?}"
            );
        }
        for code in [
            "0xFF",
            "0b1010",
            "0o17",
            "1_000_000",
            "1.5e-3",
            "0xFF_FF",
            "1_0.0_1",
            "1..10",
        ] {
            let source = Source::new(SourceId(1), "test.aipo", code);
            let (_, diags) = Lexer::new(&source).tokenize();
            assert!(diags.is_empty(), "'{code}' must lex cleanly, got {diags:?}");
        }
    }

    #[test]
    fn test_string_prefixes_and_multiline() {
        let code = r#"
"hello"
f"value: {x}"
r"C:\path"
fr"raw format: {y}"
"""line 1
line 2"""
"#;
        let source = Source::new(SourceId(1), "test.aipo", code);
        let (tokens, diags) = Lexer::new(&source).tokenize();
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");

        let filtered: Vec<_> = tokens
            .into_iter()
            .filter(|t| !matches!(t.kind, TokenKind::Newline | TokenKind::Eof))
            .map(|t| t.kind)
            .collect();

        assert_eq!(
            filtered,
            vec![
                TokenKind::StringLiteral {
                    content: "hello".into(),
                    prefix: StringPrefix::Normal,
                    is_multiline: false,
                },
                TokenKind::StringLiteral {
                    content: "value: {x}".into(),
                    prefix: StringPrefix::Format,
                    is_multiline: false,
                },
                TokenKind::StringLiteral {
                    content: r"C:\path".into(),
                    prefix: StringPrefix::Raw,
                    is_multiline: false,
                },
                TokenKind::StringLiteral {
                    content: "raw format: {y}".into(),
                    prefix: StringPrefix::FormatRaw,
                    is_multiline: false,
                },
                TokenKind::StringLiteral {
                    content: "line 1\nline 2".into(),
                    prefix: StringPrefix::Normal,
                    is_multiline: true,
                },
            ]
        );
    }

    /// Canon makes NFC an invariant of `String`, and literal decoding is the boundary where
    /// source text becomes a `String` value, so an escape that composes is stored composed.
    #[test]
    fn test_string_literals_are_nfc_normalized() {
        let code = "\"e\\u{0301}\"\nr\"e\\u{0301}\"\n";
        let source = Source::new(SourceId(1), "test.aipo", code);
        let (tokens, diags) = Lexer::new(&source).tokenize();
        assert!(diags.is_empty(), "unexpected diagnostics: {diags:?}");

        let literals: Vec<_> = tokens
            .into_iter()
            .filter_map(|t| match t.kind {
                TokenKind::StringLiteral {
                    content, prefix, ..
                } => Some((content, prefix)),
                _ => None,
            })
            .collect();

        assert_eq!(
            literals[0],
            ("é".to_string(), StringPrefix::Normal),
            "a decoded escape pair must be stored as the composed character"
        );
        assert_eq!(
            literals[1],
            (r"e\u{0301}".to_string(), StringPrefix::Raw),
            "a raw literal keeps the escape text, which is already NFC"
        );
    }

    #[test]
    fn test_operators_and_pipeline() {
        let code = "x |> filter |> sort += -= == != <= >= ?. ..";
        let source = Source::new(SourceId(1), "test.aipo", code);
        let (tokens, diags) = Lexer::new(&source).tokenize();
        assert!(diags.is_empty());

        let kinds: Vec<_> = tokens
            .into_iter()
            .filter(|t| !matches!(t.kind, TokenKind::Eof))
            .map(|t| t.kind)
            .collect();

        assert_eq!(
            kinds,
            vec![
                TokenKind::Identifier("x".into()),
                TokenKind::Pipeline,
                TokenKind::Identifier("filter".into()),
                TokenKind::Pipeline,
                TokenKind::Identifier("sort".into()),
                TokenKind::PlusEq,
                TokenKind::MinusEq,
                TokenKind::EqualEqual,
                TokenKind::BangEqual,
                TokenKind::LessEqual,
                TokenKind::GreaterEqual,
                TokenKind::QuestionDot,
                TokenKind::DotDot,
            ]
        );
    }

    #[test]
    fn test_diagnostics_on_invalid_tokens() {
        let code = "\"unterminated";
        let source = Source::new(SourceId(1), "test.aipo", code);
        let (_tokens, diags) = Lexer::new(&source).tokenize();
        assert!(!diags.is_empty());
        assert_eq!(
            diags[0].code,
            aipo_diagnostics::DiagnosticCode::AIPO_LEX_UNTERMINATED_STRING
        );
    }

    #[test]
    fn test_integer_division_tokens() {
        let code = "10 // 2 //= 5";
        let source = Source::new(SourceId(1), "test.aipo", code);
        let (tokens, diags) = Lexer::new(&source).tokenize();
        assert!(diags.is_empty());
        let kinds: Vec<_> = tokens
            .into_iter()
            .filter(|t| !matches!(t.kind, TokenKind::Eof))
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::IntLiteral("10".into()),
                TokenKind::SlashSlash,
                TokenKind::IntLiteral("2".into()),
                TokenKind::SlashSlashEq,
                TokenKind::IntLiteral("5".into()),
            ]
        );
    }
}
