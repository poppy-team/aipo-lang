//! Parser totality: arbitrary token streams terminate without panic.
//!
//! The parser must make progress on hostile input: random token sequences
//! (including error tokens and unbalanced structure) always terminate, and the
//! only observable outcomes are a program plus diagnostics.

#![forbid(unsafe_code)]

use aipo_lexer::{Token, TokenKind};
use aipo_source::{Source, SourceId, SourceSpan};

fn token(kind: TokenKind) -> Token {
    Token::new(kind, SourceSpan::empty(0))
}

fn alphabet() -> Vec<TokenKind> {
    vec![
        TokenKind::Let,
        TokenKind::Var,
        TokenKind::Fn,
        TokenKind::If,
        TokenKind::Else,
        TokenKind::End,
        TokenKind::Return,
        TokenKind::Match,
        TokenKind::When,
        TokenKind::Loop,
        TokenKind::While,
        TokenKind::Break,
        TokenKind::Identifier("x".to_string()),
        TokenKind::IntLiteral("1".to_string()),
        TokenKind::StringLiteral {
            content: "s".to_string(),
            prefix: aipo_lexer::StringPrefix::Normal,
            is_multiline: false,
        },
        TokenKind::Plus,
        TokenKind::Equal,
        TokenKind::EqualEqual,
        TokenKind::LParen,
        TokenKind::RParen,
        TokenKind::LBracket,
        TokenKind::RBracket,
        TokenKind::Comma,
        TokenKind::Dot,
        TokenKind::Newline,
        TokenKind::Eof,
        TokenKind::Error("!".to_string()),
    ]
}

#[test]
fn test_random_token_streams_terminate() {
    let alphabet = alphabet();
    let mut state = 0x51ED_00DBu64;
    let mut next = move || {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        state = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
        state
    };
    let source = Source::new(SourceId::next(), "tokens.aipo", "x");
    for _ in 0..2000 {
        let len = (next() % 24) as usize;
        let mut tokens = Vec::with_capacity(len + 1);
        for _ in 0..len {
            let kind = alphabet[(next() % alphabet.len() as u64) as usize].clone();
            tokens.push(token(kind));
        }
        tokens.push(token(TokenKind::Eof));
        let parser = aipo_syntax::Parser::new(&source, tokens);
        let (_program, _diagnostics) = parser.parse();
    }
}

#[test]
fn test_deeply_nested_tokens_hit_the_bound_not_the_stack() {
    // 500 nested `if`s must end in the nesting diagnostic, never an abort.
    let mut tokens = Vec::new();
    for _ in 0..500 {
        tokens.push(token(TokenKind::If));
        tokens.push(token(TokenKind::True));
    }
    tokens.push(token(TokenKind::IntLiteral("1".to_string())));
    for _ in 0..500 {
        tokens.push(token(TokenKind::End));
    }
    tokens.push(token(TokenKind::Eof));
    let source = Source::new(SourceId::next(), "deep.aipo", "x");
    let parser = aipo_syntax::Parser::new(&source, tokens);
    let (_program, diagnostics) = parser.parse();
    assert!(
        diagnostics
            .iter()
            .any(|d| d.code == aipo_diagnostics::DiagnosticCode::AIPO_PARSE_NESTING_TOO_DEEP),
        "deep nesting reports the bound"
    );
}
