//! Lexer properties: spans, termination and totality over arbitrary input.
//!
//! For every input — corpus text or random bytes — lexing must terminate,
//! never panic, keep every token span inside the source, emit spans in
//! non-decreasing order, and end the stream with a stable `Eof`.

#![forbid(unsafe_code)]

use aipo_lexer::{Lexer, TokenKind};
use aipo_source::{Source, SourceId};

fn lex(text: &str) -> (Vec<aipo_lexer::Token>, Vec<aipo_diagnostics::Diagnostic>) {
    let source = Source::new(SourceId::next(), "prop.aipo", text);
    let tokens = Lexer::new(&source).tokenize();
    // Rebuild the source-independent assertions from a fresh handle: spans are
    // byte offsets, so validity is checked against the same text.
    let check = Source::new(SourceId::next(), "prop.aipo", text);
    for token in &tokens.0 {
        assert!(
            check.validate_span(token.span).is_ok(),
            "token span {span:?} escapes the source of {text:?}",
            span = token.span
        );
    }
    (tokens.0, tokens.1)
}

#[test]
fn test_spans_cover_source_monotonically() {
    let texts = [
        "let x = 1 + 2\nio.println(x)\n",
        "fn f(a, b)\nreturn a\nend\n",
        "f\"hi {name}!\" r\"raw\\n\" \"\"\"multi\nline\"\"\"\n",
        "struct P\nfixed id\nend\n",
        "a |> f |> g(1, x = 2)\n",
        "# only a comment\n",
        "",
        "\n\n\n",
    ];
    for text in texts {
        let (tokens, _) = lex(text);
        let mut cursor = 0usize;
        for token in &tokens {
            assert!(token.span.start >= cursor, "spans go backwards in {text:?}");
            cursor = token.span.start;
        }
        assert!(
            matches!(tokens.last().map(|t| &t.kind), Some(TokenKind::Eof)),
            "stream ends with Eof in {text:?}"
        );
    }
}

#[test]
fn test_arbitrary_bytes_never_panic_and_terminate() {
    // Deterministic xorshift64*; same shape as the CLI fuzz smoke seed family.
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = move || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        state = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
        state
    };
    for _ in 0..2000 {
        let len = (next() % 96) as usize;
        let bytes: Vec<u8> = (0..len).map(|_| (next() & 0xFF) as u8).collect();
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let source = Source::new(SourceId::next(), "fuzz.aipo", &text);
        let (tokens, _) = Lexer::new(&source).tokenize();
        for token in &tokens {
            assert!(source.validate_span(token.span).is_ok());
        }
        assert!(matches!(
            tokens.last().map(|t| &t.kind),
            Some(TokenKind::Eof)
        ));
    }
}

#[test]
fn test_unicode_text_keeps_valid_spans() {
    let texts = [
        "let café = \"naïve\"\nio.println(café)\n",
        "io.println(\"é\")\n",
        "let 中文 = 1\nlet αβγ = 2\n",
        "io.println(\"🎉🎉\")\n",
        "\"a\u{200B}b\"\n",
    ];
    for text in texts {
        let (tokens, _) = lex(text);
        assert!(matches!(
            tokens.last().map(|t| &t.kind),
            Some(TokenKind::Eof)
        ));
    }
}
