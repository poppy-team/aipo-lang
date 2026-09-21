#![no_main]
use libfuzzer_sys::fuzz_target;
use aipo_lexer::Lexer;
use aipo_source::{Source, SourceId};

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);
    let source = Source::new(SourceId::next(), "fuzz.aipo", &text);
    let lexer = Lexer::new(&source);
    let (tokens, _) = lexer.tokenize();
    for token in &tokens {
        let _ = source.validate_span(token.span);
    }
});
