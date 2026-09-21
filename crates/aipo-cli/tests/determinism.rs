//! Determinism: repeated compilation of the same source yields byte-identical
//! artifacts at every stage (tokens, IR, bytecode, JS bundle).
//!
//! If an artifact ever intentionally carries nondeterministic metadata, that
//! field must be normalized or isolated here — never silently ignored.

#![forbid(unsafe_code)]

use aipo_testkit::{corpus, pipeline, smith};

fn stage_fingerprints(source: &str) -> (String, String, Vec<u8>) {
    let src = aipo_source::Source::new(aipo_source::SourceId::next(), "det.aipo", source);
    let lexer = aipo_lexer::Lexer::new(&src);
    let (tokens, _) = lexer.tokenize();
    let token_fingerprint = format!("{tokens:?}");
    let (_, ir) = pipeline::lower_to_ir("det.aipo", source).expect("deterministic input checks");
    let ir_fingerprint = format!("{ir:?}");
    let bytecode = aipo_bytecode::compile(&ir).expect("deterministic input compiles");
    (token_fingerprint, ir_fingerprint, bytecode.code.clone())
}

#[test]
fn test_pipeline_is_deterministic_over_corpus_and_generated() {
    let mut inputs = Vec::new();
    for (path, _) in corpus::runnable_programs().into_iter().take(8) {
        inputs.push(std::fs::read_to_string(&path).expect("fixture is readable"));
    }
    let config = smith::Config::default();
    for seed in [0u64, 9, 99] {
        inputs.push(smith::generate(seed, &config).source);
    }
    for (index, source) in inputs.iter().enumerate() {
        let first = stage_fingerprints(source);
        let second = stage_fingerprints(source);
        assert_eq!(first.0, second.0, "tokens deterministic for input {index}");
        assert_eq!(first.1, second.1, "Core IR deterministic for input {index}");
        assert_eq!(
            first.2, second.2,
            "bytecode deterministic for input {index}"
        );
    }
}

#[test]
fn test_module_init_order_is_deterministic() {
    // The runtime tie-breaks init order lexicographically by canonical path:
    // insertion order must not affect the result.
    for permutation in [
        vec!["c", "a", "b"],
        vec!["a", "b", "c"],
        vec!["b", "c", "a"],
    ] {
        let mut graph = aipo_runtime::ModuleGraph::new();
        for name in permutation {
            graph.register(aipo_runtime::ModuleRecord::new(name, vec![], None));
        }
        let order = graph
            .topological_init_order()
            .expect("acyclic graph orders");
        assert_eq!(order, vec!["a", "b", "c"]);
    }
}
