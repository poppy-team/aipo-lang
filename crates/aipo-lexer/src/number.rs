//! Numeric literal rules shared by the lexer and every later stage.
//!
//! Canon keeps number handling lexical: the lexer classifies a literal and carries its
//! text, and no stage evaluates what it does not own. Well-formedness is therefore a
//! property of the *text* and lives here, next to the scanner that produces it, so the
//! IR builder parses exactly the literals the lexer accepted — one definition, no drift
//! between "what lexes" and "what executes".

/// Digit portion of a `0x`/`0b`/`0o` literal, case-insensitively, or `None` for the
/// decimal form.
#[must_use]
pub fn strip_base_prefix(text: &str, base: char) -> Option<&str> {
    let mut chars = text.chars();
    if chars.next() != Some('0') {
        return None;
    }
    match chars.next() {
        Some(prefix) if prefix.eq_ignore_ascii_case(&base) => text.get(2..),
        _ => None,
    }
}

/// Whether `raw` is a lexically well-formed numeric literal.
///
/// The rule is: the digit set must match the declared base (`0x`/`0b`/`0o`, decimal
/// otherwise), `_` separators must sit *between* two digits, and the decimal form must be
/// `digits[.digits][(e|E)[+|-]digits]`. Range is deliberately **not** checked here — a
/// literal that does not fit `Int` is still well formed (canon keeps the value judgment
/// out of the lexer) and is rejected as an overflow fault when it is loaded.
#[must_use]
pub fn number_is_well_formed(raw: &str) -> bool {
    if let Some(digits) = strip_base_prefix(raw, 'x') {
        return radix_digits_ok(digits, 16);
    }
    if let Some(digits) = strip_base_prefix(raw, 'b') {
        return radix_digits_ok(digits, 2);
    }
    if let Some(digits) = strip_base_prefix(raw, 'o') {
        return radix_digits_ok(digits, 8);
    }
    decimal_is_well_formed(raw)
}

/// Parses a well-formed integer literal, honoring `0x`/`0b`/`0o` bases and `_` separators.
///
/// Returns `None` when the literal does not fit [`i64`]; callers turn that into an explicit
/// overflow fault instead of a silent `0`.
#[must_use]
pub fn parse_int_literal(raw: &str) -> Option<i64> {
    let cleaned;
    let s = if raw.contains('_') {
        cleaned = raw.replace('_', "");
        &cleaned
    } else {
        raw
    };
    if let Some(digits) = strip_base_prefix(s, 'x') {
        i64::from_str_radix(digits, 16).ok()
    } else if let Some(digits) = strip_base_prefix(s, 'b') {
        i64::from_str_radix(digits, 2).ok()
    } else if let Some(digits) = strip_base_prefix(s, 'o') {
        i64::from_str_radix(digits, 8).ok()
    } else {
        s.parse::<i64>().ok()
    }
}

/// Parses a well-formed float literal, honoring `_` separators.
///
/// Returns `None` for text that is not a number. Overflowing magnitudes parse to
/// infinity, which the runtime reports as `AIPO_RT_NON_FINITE_FLOAT` on both backends.
#[must_use]
pub fn parse_float_literal(raw: &str) -> Option<f64> {
    if raw.contains('_') {
        raw.replace('_', "").parse::<f64>().ok()
    } else {
        raw.parse::<f64>().ok()
    }
}

/// Validates a digit run: at least one digit, and `_` only between two digits of the base.
fn radix_digits_ok(digits: &str, radix: u32) -> bool {
    let bytes = digits.as_bytes();
    let mut cursor = 0;
    let consumed = consume_digit_run(bytes, &mut cursor, radix);
    consumed && cursor == bytes.len()
}

/// Validates the decimal form `digits[.digits][(e|E)[+|-]digits]`.
fn decimal_is_well_formed(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut cursor = 0;
    if !consume_digit_run(bytes, &mut cursor, 10) {
        return false;
    }
    if bytes.get(cursor) == Some(&b'.') {
        cursor += 1;
        if !consume_digit_run(bytes, &mut cursor, 10) {
            return false;
        }
    }
    if matches!(bytes.get(cursor), Some(b'e' | b'E')) {
        cursor += 1;
        if matches!(bytes.get(cursor), Some(b'+' | b'-')) {
            cursor += 1;
        }
        if !consume_digit_run(bytes, &mut cursor, 10) {
            return false;
        }
    }
    cursor == bytes.len()
}

fn is_radix_digit(b: u8, radix: u32) -> bool {
    match radix {
        2 => matches!(b, b'0' | b'1'),
        8 => matches!(b, b'0'..=b'7'),
        10 => b.is_ascii_digit(),
        16 => b.is_ascii_hexdigit(),
        _ => false,
    }
}

/// Consumes digits and well-placed `_` separators, reporting whether a digit was seen.
///
/// An underscore is well placed when the character before and after it are digits of the
/// same base, so `1_000` and `0xFF_FF` are literal text while `_1`, `1_`, `1__0` and
/// `1e_5` are not.
fn consume_digit_run(bytes: &[u8], cursor: &mut usize, radix: u32) -> bool {
    let mut saw_digit = false;
    while let Some(&b) = bytes.get(*cursor) {
        if is_radix_digit(b, radix) {
            saw_digit = true;
            *cursor += 1;
        } else if b == b'_' {
            let before = cursor.checked_sub(1).and_then(|i| bytes.get(i)).copied();
            let after = bytes.get(*cursor + 1).copied();
            match (before, after) {
                (Some(before), Some(after))
                    if is_radix_digit(before, radix) && is_radix_digit(after, radix) =>
                {
                    *cursor += 1;
                }
                _ => return false,
            }
        } else {
            break;
        }
    }
    saw_digit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_well_formed_literals_are_accepted() {
        for raw in [
            "0",
            "42",
            "1_000_000",
            "0xFF",
            "0XFF_FF",
            "0b1010",
            "0B1010_1010",
            "0o17",
            "0O7_7",
            "3.14",
            "1.5e-3",
            "1_0.0_1e+2",
            "12e5",
        ] {
            assert!(number_is_well_formed(raw), "{raw} must be well formed");
        }
    }

    #[test]
    fn test_malformed_literals_are_rejected() {
        for raw in [
            "0x", "0b", "0o", "0b102", "0o8", "0xG", "_1", "1_", "1__0", "1_.5", "1._5", "1e",
            "1e_5", "1e+", "0x_FF", "1_.0e1",
        ] {
            assert!(!number_is_well_formed(raw), "{raw} must be malformed");
        }
    }

    #[test]
    fn test_int_literal_bases_and_separators_parse() {
        assert_eq!(parse_int_literal("42"), Some(42));
        assert_eq!(parse_int_literal("1_000_000"), Some(1_000_000));
        assert_eq!(parse_int_literal("0xFF"), Some(255));
        assert_eq!(parse_int_literal("0xff"), Some(255));
        assert_eq!(parse_int_literal("0b1010"), Some(10));
        assert_eq!(parse_int_literal("0o17"), Some(15));
        assert_eq!(
            parse_int_literal("9_007_199_254_740_991"),
            Some(9_007_199_254_740_991)
        );
    }

    #[test]
    fn test_out_of_range_int_literals_do_not_parse() {
        assert_eq!(parse_int_literal("99999999999999999999"), None);
        assert_eq!(parse_int_literal("0xFFFFFFFFFFFFFFFFFF"), None);
    }

    #[test]
    fn test_float_literal_parses_with_separators() {
        assert_eq!(parse_float_literal("1_000.5"), Some(1000.5));
        assert_eq!(parse_float_literal("1.5e-3"), Some(0.0015));
        assert_eq!(parse_float_literal("0.0"), Some(0.0));
        assert!(parse_float_literal("1e999").is_some_and(|f| f.is_infinite()));
        assert_eq!(parse_float_literal("nope"), None);
    }
}
