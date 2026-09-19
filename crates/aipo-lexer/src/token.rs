//! Token representation and classification for Aipo.

use aipo_source::SourceSpan;
use serde::{Deserialize, Serialize};

/// Prefix variant for string literals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum StringPrefix {
    /// Standard string `"..."`
    #[default]
    Normal,
    /// Format string `f"..."`
    Format,
    /// Raw string `r"..."`
    Raw,
    /// Format raw string `fr"..."`
    FormatRaw,
}

/// Token classification covering the entire Aipo V1 surface.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenKind {
    // --- Keywords ---
    /// `let`
    Let,
    /// `var`
    Var,
    /// `fixed`
    Fixed,
    /// `fn`
    Fn,
    /// `struct`
    Struct,
    /// `impl`
    Impl,
    /// `interface`
    Interface,
    /// `satisfy`
    Satisfy,
    /// `init`
    Init,
    /// `invariant`
    Invariant,
    /// `if`
    If,
    /// `elif`
    Elif,
    /// `else`
    Else,
    /// `then`
    Then,
    /// `end`
    End,
    /// `match`
    Match,
    /// `when`
    When,
    /// `loop`
    Loop,
    /// `while`
    While,
    /// `repeat`
    Repeat,
    /// `as`
    As,
    /// `each`
    Each,
    /// `in`
    In,
    /// `break`
    Break,
    /// `continue`
    Continue,
    /// `return`
    Return,
    /// `fail`
    Fail,
    /// `or_else`
    OrElse,
    /// `attempt`
    Attempt,
    /// `failed`
    Failed,
    /// `import`
    Import,
    /// `export`
    Export,
    /// `is`
    Is,
    /// `not`
    Not,
    /// `and`
    And,
    /// `or`
    Or,
    /// `true`
    True,
    /// `false`
    False,
    /// `none`
    None,
    /// `self`
    SelfVal,
    /// `self!`
    SelfMut,
    /// `do`
    Do,

    // --- Identifiers & Literals ---
    /// Identifier with name (NFC normalized)
    Identifier(String),
    /// Wildcard / discard marker `_`
    Discard,
    /// Integer literal string representation
    IntLiteral(String),
    /// Floating point literal string representation
    FloatLiteral(String),
    /// String literal token with prefix and multiline status
    StringLiteral {
        /// Content inside the quotes (with escapes processed if applicable)
        content: String,
        /// String prefix (`Normal`, `Format`, `Raw`, `FormatRaw`)
        prefix: StringPrefix,
        /// True if enclosed in triple quotes `"""..."""`
        is_multiline: bool,
    },

    // --- Operators & Delimiters ---
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `div`
    Div,
    /// `%`
    Percent,
    /// `+=`
    PlusEq,
    /// `-=`
    MinusEq,
    /// `*=`
    StarEq,
    /// `/=`
    SlashEq,
    /// `div=`
    DivEq,
    /// `%=`
    PercentEq,
    /// `==`
    EqualEqual,
    /// `!=`
    BangEqual,
    /// `<`
    Less,
    /// `<=`
    LessEqual,
    /// `>`
    Greater,
    /// `>=`
    GreaterEqual,
    /// `=`
    Equal,
    /// `|>` (pipeline)
    Pipeline,
    /// `..` (range)
    DotDot,
    /// `.`
    Dot,
    /// `?.`
    QuestionDot,
    /// `!`
    Bang,
    /// `?`
    Question,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `{`
    LBrace,
    /// `}`
    RBrace,

    // --- Trivia / Control ---
    /// Significant newline terminating statements
    Newline,
    /// Comment `# ...`
    Comment(String),
    /// End of file
    Eof,
    /// Lexical error token with description
    Error(String),
}

/// A token positioned within a source text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    /// The classification of this token.
    pub kind: TokenKind,
    /// The half-open byte span `[start, end)` in the source.
    pub span: SourceSpan,
}

impl Token {
    /// Constructs a new token.
    #[must_use]
    pub const fn new(kind: TokenKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }
}
