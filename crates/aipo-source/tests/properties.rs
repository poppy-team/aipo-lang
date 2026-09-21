//! Source properties: spans, locations and normalization.
//!
//! Invariants under test: every valid span slices successfully; invalid spans
//! fail closed (`None`/`Err`, never a panic); CRLF and BOM normalization never
//! moves a logical location; invalid UTF-8 from the filesystem is a controlled
//! `SourceError`, not a crash.

#![forbid(unsafe_code)]

use aipo_source::{Source, SourceId, SourceSpan};

fn source(text: &str) -> Source {
    Source::new(SourceId::next(), "prop.aipo", text)
}

#[test]
fn test_valid_spans_slice_and_locate() {
    let text = "let x = 1\nio.println(x)\n";
    let src = source(text);
    let span = SourceSpan::new(4, 5);
    assert_eq!(src.slice(span), Some("x"));
    assert!(src.validate_span(span).is_ok());
    let location = src.location(4).expect("offset 4 locates");
    assert_eq!((location.line, location.column), (1, 5));
    assert_eq!(src.line_content(2), Some("io.println(x)"));
}

#[test]
fn test_invalid_spans_fail_closed() {
    let src = source("hi\n");
    for span in [
        SourceSpan::new(0, 99),
        SourceSpan::new(5, 3),
        SourceSpan::new(99, 100),
    ] {
        assert_eq!(src.slice(span), None, "slice fails closed: {span:?}");
        assert!(src.validate_span(span).is_err(), "validate fails closed");
    }
    assert_eq!(src.location(99), None);
    assert_eq!(src.line_content(99), None);
}

#[test]
fn test_crlf_preserves_logical_locations() {
    let lf = "ab\ncd\nef\n";
    let crlf = "ab\r\ncd\r\nef\r\n";
    let left = source(lf);
    let right = source(crlf);
    assert_eq!(left.text(), right.text());
    for (line, expected) in [(1, "ab"), (2, "cd"), (3, "ef")] {
        assert_eq!(left.line_content(line), Some(expected));
        assert_eq!(right.line_content(line), Some(expected));
    }
    // The `d` sits at the same logical position in both normalizations.
    let left_loc = left.location(lf.find('d').unwrap()).unwrap();
    let right_loc = right.location(right.text().find('d').unwrap()).unwrap();
    assert_eq!(
        (left_loc.line, left_loc.column),
        (right_loc.line, right_loc.column)
    );
    assert_eq!((left_loc.line, left_loc.column), (2, 2));
}

#[test]
fn test_bom_is_stripped_without_shifting() {
    let src = source("﻿let x = 1\n");
    assert_eq!(src.text(), "let x = 1\n");
    let location = src.location(0).unwrap();
    assert_eq!((location.line, location.column), (1, 1));
}

#[test]
fn test_invalid_utf8_file_is_a_controlled_error() {
    let dir = std::env::temp_dir();
    let path = dir.join("aipo-invalid-utf8.aipo");
    std::fs::write(&path, [0x66, 0x6F, 0x80, 0x6F]).expect("scratch is writable");
    let mut map = aipo_source::SourceMap::new();
    let result = map.load_file(&path);
    assert!(
        result.is_err(),
        "invalid UTF-8 is a SourceError, not a panic"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_line_spans_round_trip() {
    let text = "a\nbb\nccc\n";
    let src = source(text);
    for line in 1..=3 {
        let span = src.line_span(line).expect("line span exists");
        let content = src.line_content(line).expect("line content exists");
        // `line_span` covers the terminator; `line_content` strips it.
        assert_eq!(src.slice(span), Some(format!("{content}\n").as_str()));
    }
    assert_eq!(src.slice(src.line_span(4).unwrap()), Some(""));
    assert_eq!(src.line_span(5), None);
}
