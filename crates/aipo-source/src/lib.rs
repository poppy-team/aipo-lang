//! `aipo-source` provides source management, UTF-8 normalization, BOM stripping,
//! and accurate line/column mapping over byte-based spans.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod source;
pub mod source_map;
pub mod span;

pub use error::SourceError;
pub use source::{Source, SourceId};
pub use source_map::SourceMap;
pub use span::{SourceLocation, SourceSpan};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bom_stripping() {
        let text_with_bom = "\u{FEFF}let x = 42\n";
        let source = Source::new(SourceId(1), "test.aipo", text_with_bom);
        assert_eq!(source.text(), "let x = 42\n");
    }

    #[test]
    fn test_newline_normalization() {
        let crlf = "line 1\r\nline 2\rline 3\n";
        let source = Source::new(SourceId(1), "test.aipo", crlf);
        assert_eq!(source.text(), "line 1\nline 2\nline 3\n");
        assert_eq!(source.line_count(), 4);
    }

    #[test]
    fn test_location_and_span() {
        let text = "alpha\nbeta\ngamma";
        let source = Source::new(SourceId(1), "test.aipo", text);

        let loc0 = source.location(0).unwrap();
        assert_eq!(loc0.line, 1);
        assert_eq!(loc0.column, 1);

        let loc_beta = source.location(6).unwrap();
        assert_eq!(loc_beta.line, 2);
        assert_eq!(loc_beta.column, 1);

        let span = SourceSpan::new(6, 10);
        assert_eq!(source.slice(span), Some("beta"));

        let line2_content = source.line_content(2).unwrap();
        assert_eq!(line2_content, "beta");
    }

    #[test]
    fn test_span_merge() {
        let s1 = SourceSpan::new(2, 5);
        let s2 = SourceSpan::new(8, 12);
        let merged = s1.merge(s2);
        assert_eq!(merged.start, 2);
        assert_eq!(merged.end, 12);
    }

    #[test]
    fn test_source_map() {
        let mut map = SourceMap::new();
        let id = map.add_source("main.aipo", "fn main() do end\n");
        let src = map.get(id).unwrap();
        assert_eq!(src.name(), "main.aipo");
        assert_eq!(map.get_by_name("main.aipo").unwrap().id(), id);
    }
}
