//! Unicode security corpus: characterization tests, not policy.
//!
//! Aipo accepts precomposed non-ASCII identifiers and preserves emoji/ZWJ in
//! strings, while rejecting combining marks, zero-width and bidi controls in
//! identifier position with `AIPO_LEX_UNEXPECTED_CHARACTER`. Mixed scripts and
//! confusables are currently unrestricted. None of this is a security policy
//! yet: every unspecified behavior is recorded in ADP-004, and these tests pin
//! current behavior so a future policy change is reviewable.

#![forbid(unsafe_code)]

use aipo_testkit::pipeline;
use std::sync::{Mutex, MutexGuard, OnceLock};

fn vm_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

fn check(source: &str) -> Result<(), Vec<aipo_diagnostics::Diagnostic>> {
    pipeline::check_text("unicode.aipo", source).map(|_| ())
}

fn code_of(diagnostics: &[aipo_diagnostics::Diagnostic]) -> Vec<String> {
    diagnostics.iter().map(|d| d.code.to_string()).collect()
}

#[test]
fn test_precomposed_identifiers_work() {
    let _guard = vm_lock();
    let source = "let caf\u{e9} = 1\nlet \u{4e2d} = 2\nlet \u{3b1}\u{3b2} = 3\nio.println(caf\u{e9} + \u{4e2d} + \u{3b1}\u{3b2})\n";
    let stdout = pipeline::run_text("unicode.aipo", source).expect("runs");
    assert_eq!(stdout, "6\n");
}

#[test]
fn test_combining_mark_in_identifier_is_rejected() {
    // Decomposed `e` + U+0301: the loader does not NFC-normalize, and combining
    // marks are not identifier characters, so this is a lexer error (ADP-004).
    let Err(diagnostics) = check("let e\u{301} = 1\n") else {
        panic!("decomposed identifier must not check clean");
    };
    assert!(code_of(&diagnostics).contains(&"AIPO_LEX_UNEXPECTED_CHARACTER".to_string()));
}

#[test]
fn test_zero_width_and_bidi_controls_rejected_in_identifiers() {
    for (label, ch) in [
        ("zwsp", '\u{200b}'),
        ("zwj", '\u{200d}'),
        ("bidi-embed", '\u{202a}'),
        ("bidi-override", '\u{202e}'),
    ] {
        let source = format!("let a{ch}b = 1\n");
        let Err(diagnostics) = check(&source) else {
            panic!("{label} in identifier must not check clean");
        };
        assert!(
            code_of(&diagnostics).contains(&"AIPO_LEX_UNEXPECTED_CHARACTER".to_string()),
            "{label} rejected as unexpected character"
        );
    }
}

#[test]
fn test_mixed_scripts_currently_allowed() {
    // No confusable/mixed-script restriction exists today (ADP-004): pin that a
    // Cyrillic identifier is accepted exactly like a Latin one.
    let _guard = vm_lock();
    let source = "let \u{441}ount = 40\nlet count = 2\nio.println(\u{441}ount + count)\n";
    let stdout = pipeline::run_text("unicode.aipo", source).expect("runs today");
    assert_eq!(stdout, "42\n");
}

#[test]
fn test_emoji_and_zwj_preserved_in_strings() {
    let _guard = vm_lock();
    let source = "io.println(\"\u{1f389}\")\nio.println(\"\u{1f469}\u{200d}\u{1f4bb}\")\n";
    let stdout = pipeline::run_text("unicode.aipo", source).expect("runs");
    assert_eq!(stdout, "\u{1f389}\n\u{1f469}\u{200d}\u{1f4bb}\n");
}

#[test]
fn test_zero_width_in_strings_preserved_bytewise() {
    let _guard = vm_lock();
    let source = "io.println(len(\"a\u{200b}b\"))\n";
    let stdout = pipeline::run_text("unicode.aipo", source).expect("runs");
    assert_eq!(stdout, "3\n", "ZWSP is a code point in strings");
}

#[test]
fn test_nbsp_is_not_whitespace() {
    // U+00A0 is rejected rather than skipped: NBSP is not source whitespace.
    let Err(diagnostics) = check("let x = 1\u{a0}\n") else {
        panic!("trailing NBSP must not check clean");
    };
    assert!(code_of(&diagnostics).contains(&"AIPO_LEX_UNEXPECTED_CHARACTER".to_string()));
}

#[test]
fn test_extreme_combining_terminates() {
    let _guard = vm_lock();
    // The lexer NFC-normalizes the literal (`\u{e9}` + 299 leftover marks = 300
    // code points), then evaluation terminates normally.
    let heavy = format!("io.println(len(\"e{}\"))\n", "\u{301}".repeat(300));
    let stdout = pipeline::run_text("unicode.aipo", &heavy).expect("runs");
    assert_eq!(stdout, "300\n");
}
