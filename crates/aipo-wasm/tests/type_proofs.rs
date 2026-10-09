//! Compile-time type-proof regressions; not executed in P07-G02.
use aipo_source::{Source, SourceId};

fn compile(text: &str) -> Result<Vec<u8>, aipo_wasm::WasmCompileError> {
    let source = Source::new(SourceId::next(), "types.aipo", text);
    let (program, diagnostics) = aipo_syntax::parse(&source);
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    aipo_wasm::compile_hir(&aipo_hir::lower(program))
}

#[test]
fn untyped_parameters_and_mixed_joins_do_not_get_a_fabricated_int_tag() {
    assert!(compile("fn uncertain(value) { return value is Int }\n").is_err());
    assert!(
        compile(
            "fn mixed(flag: Bool) {\nlet value = if flag then 1 else true\nreturn value is Int\n}\n"
        )
        .is_err()
    );
}

#[test]
fn semantic_bool_and_nullable_none_can_be_proven() {
    assert!(compile("fn yes() { return true is Bool }\nfn no() { return true is Int }\nfn nullable() { return none is Int? }\n").is_ok());
}
