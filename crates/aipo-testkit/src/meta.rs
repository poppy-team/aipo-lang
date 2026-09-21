//! Metamorphic source transformations: validity- and semantics-preserving
//! rewrites whose observable behavior must be identical before and after.
//!
//! Every transform documents the exact equivalence argument. The oracle
//! (run both, compare) lives in the test files; this module only rewrites.

use crate::rng::Rng;

/// Appends comment and blank lines at deterministic positions.
///
/// Equivalence: comments and blank lines are not tokens with semantics.
pub fn insert_comments_blanks(rng: &mut Rng, text: &str) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let insertions = 1 + rng.below(3);
    for i in 0..insertions {
        let at = rng.below(lines.len() + 1);
        let comment = format!("# metamorphic comment {i}");
        lines.insert(at, String::new());
        lines.insert(at, comment);
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}

/// Swaps the newline convention of the whole file.
///
/// Equivalence: the source loader normalizes CRLF and lone CR to LF before
/// lexing, so the token stream is identical.
#[must_use]
pub fn to_crlf(text: &str) -> String {
    text.replace('\n', "\r\n")
}

/// Replaces whole-word occurrences of `from` with `to`.
///
/// The caller must guarantee `from` names exactly one binding (the AipoSmith
/// generator mints unique names, which makes this safe). Word boundaries are
/// ASCII alphanumerics plus `_`, matching identifier characters.
#[must_use]
pub fn rename_binding(text: &str, from: &str, to: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(pos) = rest.find(from) {
        let before_ok = pos == 0 || !is_word_char(rest[..pos].chars().next_back());
        let after = pos + from.len();
        let after_ok = rest[after..]
            .chars()
            .next()
            .is_none_or(|c| !is_word_char(Some(c)));
        if before_ok && after_ok {
            out.push_str(&rest[..pos]);
            out.push_str(to);
            rest = &rest[after..];
        } else {
            let next_boundary = pos + from.chars().next().map_or(1, |c| c.len_utf8());
            out.push_str(&rest[..next_boundary]);
            rest = &rest[next_boundary..];
        }
    }
    out.push_str(rest);
    out
}

fn is_word_char(ch: Option<char>) -> bool {
    ch.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Composed/decomposed spellings of the same NFC string, for Unicode
/// equivalence properties (precomposed `é` vs `e` + combining acute U+0301).
#[must_use]
pub fn nfc_equivalents() -> Vec<(String, String)> {
    vec![
        ("\u{e9}".to_string(), "e\u{301}".to_string()),
        ("\u{f1}".to_string(), "n\u{303}".to_string()),
        ("\u{fc}".to_string(), "u\u{308}".to_string()),
    ]
}
