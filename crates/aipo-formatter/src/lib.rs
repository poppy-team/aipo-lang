//! `aipo-formatter` renders Aipo source in the canonical style.
//!
//! Canonical rules come from concrete canon (`Aipo V1 — Language Reference`,
//! `Aipo Language — Especificação Viva`):
//!
//! - indentation carries no semantic meaning and the official formatter uses **4 spaces
//!   per level**;
//! - blocks are explicitly closed with `end`, so indentation is derived from the block
//!   keywords themselves, never inferred from the input's original spacing;
//! - `import`/`export` may appear anywhere in the top level; the formatter only
//!   normalizes whitespace and never reorders declarations.
//!
//! The formatter is a whitespace normalizer over the **token stream**: token contents
//! (identifiers, literal spelling, string escapes, comments) are copied from the
//! original text, so formatting can never change a program's meaning or lose a comment.
//! Formatting is idempotent: `format(format(x)) == format(x)`.
//!
//! ```
//! use aipo_formatter::format_text;
//!
//! let formatted = format_text("main.aipo", "let x=1\nif x>0\nio.println(\"positive\")\nend")
//!     .expect("valid source formats");
//! assert_eq!(formatted, "let x = 1\nif x > 0\n    io.println(\"positive\")\nend\n");
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod tokens;

use aipo_lexer::{Lexer, Token, TokenKind};
use aipo_source::{Source, SourceId};
use std::fmt;
use tokens::{
    Class, classify, group_delta, is_branch_keyword, minus_is_prefix, opens_block, text_of,
};

/// Indentation width used by the canonical formatter.
pub const INDENT: &str = "    ";

/// Failure to format a source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatError {
    /// Human-readable reason, including the source position when known.
    pub message: String,
    /// Byte offset of the offending token, when the failure is positional.
    pub offset: Option<usize>,
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for FormatError {}

/// Formats Aipo source text, returning the canonical rendering.
///
/// # Errors
/// Returns [`FormatError`] when the input cannot be tokenized, so that `aipo fmt` never
/// rewrites a file it cannot fully understand.
pub fn format_text(name: &str, text: &str) -> Result<String, FormatError> {
    let source = Source::new(SourceId::next(), name, text);
    format_source(&source)
}

/// Formats an already-constructed [`Source`].
///
/// # Errors
/// Returns [`FormatError`] when the input cannot be tokenized.
pub fn format_source(source: &Source) -> Result<String, FormatError> {
    let (tokens, diagnostics) = Lexer::new(source).tokenize();
    if let Some(first) = diagnostics.first() {
        return Err(FormatError {
            message: first.message.clone(),
            offset: first.primary_span.as_ref().map(|span| span.start),
        });
    }

    let lines = split_lines(&tokens);
    Ok(render(source, &lines))
}

/// A logical source line together with the blank-line information carried by the
/// newline token that precedes it.
struct Line<'a> {
    tokens: Vec<&'a Token>,
    blank_before: bool,
}

fn split_lines<'a>(tokens: &'a [Token]) -> Vec<Line<'a>> {
    let mut lines: Vec<Line<'a>> = Vec::new();
    let mut current: Vec<&Token> = Vec::new();
    let mut pending_blank = false;
    let mut started = false;

    for token in tokens {
        match &token.kind {
            TokenKind::Eof => break,
            TokenKind::Newline => {
                // The lexer collapses runs of newlines into one token; the raw span tells
                // us whether the author left a blank line between statements.
                let width = token.span.end.saturating_sub(token.span.start);
                let newlines = width.max(1);
                if started && current.is_empty() && newlines >= 2 {
                    pending_blank = true;
                    continue;
                }
                if !current.is_empty() {
                    lines.push(Line {
                        tokens: std::mem::take(&mut current),
                        blank_before: pending_blank,
                    });
                    pending_blank = false;
                }
                if newlines >= 2 {
                    pending_blank = true;
                }
                started = true;
            }
            _ => {
                started = true;
                current.push(token);
            }
        }
    }

    if !current.is_empty() {
        lines.push(Line {
            tokens: current,
            blank_before: pending_blank,
        });
    }

    lines
}

fn render(source: &Source, lines: &[Line<'_>]) -> String {
    let mut out = String::new();
    let mut block_level: i64 = 0;
    let mut group_depth: i64 = 0;

    for (index, line) in lines.iter().enumerate() {
        let first = line
            .tokens
            .first()
            .map(|token| token.kind.clone())
            .unwrap_or(TokenKind::Eof);

        let starts_with_branch = is_branch_keyword(&first);
        // A branch keyword closes the branch opened by the previous `when`/`elif`/`else`/
        // `failed` before opening its own body.
        let closes_previous_branch = matches!(
            first,
            TokenKind::When | TokenKind::Elif | TokenKind::Else | TokenKind::Failed
        );
        let starts_with_close = matches!(classify(&first), Class::Close);

        let mut line_block = block_level;
        if starts_with_branch {
            line_block -= 1;
        }
        let mut line_group = group_depth;
        if starts_with_close {
            line_group -= 1;
        }

        // An open group adds a single continuation level, not one per open bracket: a call
        // nested in a call is still written one step in, the way the block bodies inside it
        // are. Without this clamp, `f(g(fn () ... end))` would indent its body twice.
        let indent = (line_block.max(0) + line_group.clamp(0, 1)) as usize;
        let rendered = render_line(source, &line.tokens);

        if line.blank_before && index > 0 && !out.ends_with("\n\n") {
            out.push('\n');
        }
        out.push_str(&INDENT.repeat(indent));
        out.push_str(&rendered);
        out.push('\n');

        if closes_previous_branch {
            block_level -= 1;
        }

        for (position, token) in line.tokens.iter().enumerate() {
            group_depth += i64::from(group_delta(&token.kind));
            if opens_block_here(&token.kind, position, &line.tokens) {
                block_level += 1;
            }
            if matches!(token.kind, TokenKind::End) {
                block_level -= 1;
            }
        }
    }

    while out.ends_with('\n') {
        out.pop();
    }
    if !out.is_empty() {
        out.push('\n');
    }
    out
}

/// `fn` opens a block except in a type annotation, where it names a function type.
fn opens_block_here(kind: &TokenKind, position: usize, tokens: &[&Token]) -> bool {
    if !opens_block(kind) {
        return false;
    }
    if matches!(kind, TokenKind::Fn) {
        let previous = if position == 0 {
            None
        } else {
            Some(&tokens[position - 1].kind)
        };
        if matches!(previous, Some(TokenKind::Colon)) {
            return false;
        }
    }
    true
}

fn render_line(source: &Source, tokens: &[&Token]) -> String {
    let mut out = String::new();
    let mut previous: Option<Class> = None;
    let mut previous_kind: Option<&TokenKind> = None;

    for token in tokens {
        let class = if matches!(token.kind, TokenKind::Minus) && minus_is_prefix(previous_kind) {
            Class::PrefixTight
        } else {
            classify(&token.kind)
        };

        if needs_space(previous, previous_kind, class, &token.kind) {
            out.push(' ');
        }
        out.push_str(&text_of(token, source));
        previous = Some(class);
        previous_kind = Some(&token.kind);
    }

    out
}

/// Canonical spacing rule between two adjacent tokens of a line.
fn needs_space(
    previous: Option<Class>,
    previous_kind: Option<&TokenKind>,
    next: Class,
    next_kind: &TokenKind,
) -> bool {
    let Some(previous) = previous else {
        return false;
    };

    // Nothing may follow a comment on the same logical line.
    if previous == Class::Comment {
        return false;
    }
    if next == Class::Comment {
        return true;
    }

    // Tight on the left.
    if matches!(
        next,
        Class::Close | Class::Comma | Class::Colon | Class::Dot | Class::Postfix | Class::Range
    ) {
        return false;
    }

    // Tight on the right.
    if matches!(
        previous,
        Class::Dot | Class::Range | Class::Open | Class::PrefixTight
    ) {
        return false;
    }

    if matches!(next, Class::Open) {
        let binds_to_previous = matches!(
            previous_kind,
            Some(
                TokenKind::Identifier(_)
                    | TokenKind::IntLiteral(_)
                    | TokenKind::FloatLiteral(_)
                    | TokenKind::StringLiteral { .. }
                    | TokenKind::None
                    | TokenKind::True
                    | TokenKind::False
                    | TokenKind::SelfVal
                    | TokenKind::SelfMut
                    | TokenKind::Discard
                    | TokenKind::RParen
                    | TokenKind::RBracket
            )
        );
        // `f(x)`, `Type{...}` and `items[0]` are tight; `if (cond)` and `= (` are not.
        return !binds_to_previous;
    }

    if matches!(previous, Class::Comma | Class::Colon | Class::Operator) {
        return true;
    }

    if matches!(next_kind, TokenKind::Newline) {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format(text: &str) -> String {
        format_text("test.aipo", text).expect("source formats")
    }

    #[test]
    fn test_reindents_block_with_four_spaces() {
        let out = format("if x > 0\nio.println(\"yes\")\nend");
        assert_eq!(out, "if x > 0\n    io.println(\"yes\")\nend\n");
    }

    #[test]
    fn test_normalizes_operator_spacing() {
        let out = format("let total=a+b*2");
        assert_eq!(out, "let total = a + b * 2\n");
    }

    #[test]
    fn test_preserves_comments_and_strings() {
        let out = format("let path = r\"C:\\tmp\" # keep me\n");
        assert_eq!(out, "let path = r\"C:\\tmp\" # keep me\n");
    }

    #[test]
    fn test_is_idempotent() {
        let once = format("fn add(a,b)\nreturn a+b\nend\nif true\nlet x = [1,2,3]\nend");
        let twice = format(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn test_match_branches_align_with_match() {
        let out = format(
            "match status\nwhen \"a\"\nio.print(1)\nwhen \"b\"\nio.print(2)\nelse\nio.print(3)\nend",
        );
        assert_eq!(
            out,
            "match status\nwhen \"a\"\n    io.print(1)\nwhen \"b\"\n    io.print(2)\nelse\n    io.print(3)\nend\n"
        );
    }

    #[test]
    fn test_reports_unterminated_string() {
        let err = format_text("test.aipo", "let x = \"unterminated").unwrap_err();
        assert!(!err.message.is_empty());
    }
}
