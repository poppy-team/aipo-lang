#![no_main]
use libfuzzer_sys::fuzz_target;
use aipo_sema::PreludeSurface;
use aipo_source::{Source, SourceId};

fn surface() -> PreludeSurface {
    let mut surface = PreludeSurface::fundamental();
    for name in [
        "io", "math", "string", "len", "copy", "same", "some", "fail", "Int", "Float", "Byte",
        "String", "Bool", "List", "Dict", "Bytes",
    ] {
        surface.add_variable(name);
    }
    surface
}

fuzz_target!(|data: &[u8]| {
    let text = String::from_utf8_lossy(data);
    let source = Source::new(SourceId::next(), "fuzz.aipo", &text);
    let (program, _) = aipo_syntax::parse(&source);
    let hir = aipo_hir::lower(program);
    let _ = aipo_sema::check_with_prelude(&source, &hir, &surface());
    let _ = aipo_ir::lower_to_ir(&hir);
});
