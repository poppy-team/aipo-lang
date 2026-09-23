//! Integration and unit tests for aipo-runtime: module dependency graph, cycle detection,
//! deterministic initialization ordering, and native function registry.

use aipo_diagnostics::DiagnosticCode;
use aipo_runtime::{
    ModuleGraph, ModuleRecord, ModuleState, NativeFunctionMeta, NativeRegistry, RuntimeError,
};

#[test]
fn test_module_graph_empty() {
    let graph = ModuleGraph::new();
    assert!(graph.is_empty());
    assert_eq!(graph.len(), 0);
    let order = graph.topological_init_order().unwrap();
    assert!(order.is_empty());
}

#[test]
fn test_module_graph_single_module() {
    let mut graph = ModuleGraph::new();
    graph.register(ModuleRecord::new("app.main", vec![], None));

    assert_eq!(graph.len(), 1);
    let order = graph.topological_init_order().unwrap();
    assert_eq!(order, vec!["app.main"]);
}

#[test]
fn test_module_graph_linear_dependencies() {
    let mut graph = ModuleGraph::new();
    graph.register(ModuleRecord::new(
        "app.main",
        vec!["app.service".to_string()],
        None,
    ));
    graph.register(ModuleRecord::new(
        "app.service",
        vec!["app.db".to_string()],
        None,
    ));
    graph.register(ModuleRecord::new("app.db", vec![], None));

    let order = graph.topological_init_order().unwrap();
    assert_eq!(order, vec!["app.db", "app.service", "app.main"]);
}

#[test]
fn test_module_graph_tie_breaking_order() {
    // Both b and a have no dependencies. Canonical path tie-break should place "a" before "b".
    let mut graph = ModuleGraph::new();
    graph.register(ModuleRecord::new("b", vec![], None));
    graph.register(ModuleRecord::new("a", vec![], None));
    graph.register(ModuleRecord::new(
        "c",
        vec!["a".to_string(), "b".to_string()],
        None,
    ));

    let order = graph.topological_init_order().unwrap();
    assert_eq!(order, vec!["a", "b", "c"]);
}

#[test]
fn test_module_graph_diamond_dependency() {
    // A -> B, C
    // B -> D
    // C -> D
    // D -> (none)
    let mut graph = ModuleGraph::new();
    graph.register(ModuleRecord::new("d", vec![], None));
    graph.register(ModuleRecord::new("b", vec!["d".to_string()], None));
    graph.register(ModuleRecord::new("c", vec!["d".to_string()], None));
    graph.register(ModuleRecord::new(
        "a",
        vec!["b".to_string(), "c".to_string()],
        None,
    ));

    let order = graph.topological_init_order().unwrap();
    // d must be first, then b before c (tie-break), then a
    assert_eq!(order, vec!["d", "b", "c", "a"]);
}

#[test]
fn test_module_graph_missing_dependency() {
    let mut graph = ModuleGraph::new();
    graph.register(ModuleRecord::new(
        "a",
        vec!["nonexistent".to_string()],
        None,
    ));

    let err = graph.topological_init_order().unwrap_err();
    match err {
        RuntimeError::ModuleNotFound { name } => assert_eq!(name, "nonexistent"),
        other => panic!("expected ModuleNotFound, got {other:?}"),
    }
}

#[test]
fn test_module_graph_direct_cycle() {
    let mut graph = ModuleGraph::new();
    graph.register(ModuleRecord::new("a", vec!["b".to_string()], None));
    graph.register(ModuleRecord::new("b", vec!["a".to_string()], None));

    let err = graph.topological_init_order().unwrap_err();
    match err {
        RuntimeError::CyclicDependency { cycle } => {
            assert!(cycle.contains(&"a".to_string()));
            assert!(cycle.contains(&"b".to_string()));
        }
        other => panic!("expected CyclicDependency, got {other:?}"),
    }
}

#[test]
fn test_module_graph_indirect_cycle() {
    let mut graph = ModuleGraph::new();
    graph.register(ModuleRecord::new("a", vec!["b".to_string()], None));
    graph.register(ModuleRecord::new("b", vec!["c".to_string()], None));
    graph.register(ModuleRecord::new("c", vec!["a".to_string()], None));

    let err = graph.topological_init_order().unwrap_err();
    match err {
        RuntimeError::CyclicDependency { cycle } => {
            assert!(cycle.len() >= 3);
        }
        other => panic!("expected CyclicDependency, got {other:?}"),
    }
}

#[test]
fn test_module_record_lifecycle() {
    let mut record = ModuleRecord::new("math", vec![], None);
    assert_eq!(record.state, ModuleState::Uninitialized);
    assert!(!record.is_initialized());

    record.state = ModuleState::Initializing;
    assert!(!record.is_initialized());

    record.state = ModuleState::Initialized;
    assert!(record.is_initialized());

    record.state = ModuleState::Failed("syntax error".to_string());
    assert!(!record.is_initialized());
}

#[test]
fn test_native_registry_operations() {
    let mut registry = NativeRegistry::new();
    registry.register(NativeFunctionMeta::new("len", 1, None, "collection length"));
    registry.register(NativeFunctionMeta::new("abs", 1, Some("math"), "abs value"));
    registry.register(NativeFunctionMeta::new("sin", 1, Some("math"), "sine"));
    registry.register(NativeFunctionMeta::new(
        "upper",
        1,
        Some("string"),
        "to uppercase",
    ));

    let prelude = registry.list_prelude();
    assert_eq!(prelude.len(), 1);
    assert_eq!(prelude[0].name, "len");

    let math_funcs = registry.list_module("math");
    assert_eq!(math_funcs.len(), 2);

    let modules = registry.list_modules();
    assert_eq!(modules, vec!["math", "string"]);

    let found = registry.get(Some("math"), "abs");
    assert!(found.is_some());
    assert_eq!(found.unwrap().arity, 1);

    let not_found = registry.get(None, "abs");
    assert!(not_found.is_none());
}

#[test]
fn test_error_display_and_codes_cover_all_variants() {
    let cases = [
        (
            RuntimeError::ModuleNotFound {
                name: "m".to_string(),
            },
            DiagnosticCode::AIPO_SEM_UNKNOWN_MODULE,
            "m",
        ),
        (
            RuntimeError::CyclicDependency {
                cycle: vec!["a".to_string(), "b".to_string()],
            },
            DiagnosticCode::AIPO_SEM_IMPORT_CYCLE,
            "a",
        ),
        (
            RuntimeError::ExportNotFound {
                module: "m".to_string(),
                symbol: "s".to_string(),
            },
            DiagnosticCode::AIPO_SEM_EXPORT_UNKNOWN,
            "s",
        ),
        (
            RuntimeError::InitializationFailed {
                module: "m".to_string(),
                reason: "boom".to_string(),
            },
            DiagnosticCode::AIPO_RT_MODULE_INIT_FAILED,
            "boom",
        ),
    ];
    for (error, code, fragment) in cases {
        assert_eq!(error.diagnostic_code(), code);
        assert!(
            error.to_string().contains(fragment),
            "Display names the participant: {error}"
        );
        assert!(
            error.to_string().contains(code.as_str()),
            "Display carries the stable code [{code}]: {error}"
        );
    }
}

#[test]
fn test_module_graph_self_cycle_reports_the_node() {
    let mut graph = ModuleGraph::new();
    assert!(graph.register(ModuleRecord::new("a", vec!["a".to_string()], None)));

    let err = graph.topological_init_order().unwrap_err();
    match err {
        RuntimeError::CyclicDependency { cycle } => {
            assert_eq!(cycle, vec!["a".to_string(), "a".to_string()]);
        }
        other => panic!("expected CyclicDependency, got {other:?}"),
    }
}

#[test]
fn test_module_graph_rejects_duplicate_and_malformed_paths() {
    let mut graph = ModuleGraph::new();
    assert!(graph.register(ModuleRecord::new("app", vec![], None)));
    assert!(
        !graph.register(ModuleRecord::new("app", vec![], None)),
        "duplicate registration is rejected"
    );
    assert_eq!(graph.len(), 1);

    assert!(
        !graph.register(ModuleRecord::new("", vec![], None)),
        "empty path is rejected"
    );
    assert!(
        !graph.register(ModuleRecord::new(" app", vec![], None)),
        "untrimmed path is rejected"
    );
    assert!(
        !graph.register(ModuleRecord::new("a..b", vec![], None)),
        "empty segment is rejected"
    );
    assert!(
        !graph.register(ModuleRecord::new("x", vec!["".to_string()], None)),
        "empty dependency is rejected"
    );
    assert!(
        !graph.register(ModuleRecord::new(
            "y",
            vec!["d".to_string(), "d".to_string()],
            None
        )),
        "duplicate dependency is rejected"
    );
    assert_eq!(graph.len(), 1);
}

#[test]
fn test_module_graph_missing_dependency_is_deterministic() {
    let mut graph = ModuleGraph::new();
    graph.register(ModuleRecord::new(
        "m",
        vec!["zeta".to_string(), "alpha".to_string()],
        None,
    ));
    // The lexicographically smallest missing dependency wins regardless of the
    // declaration order inside the record.
    match graph.topological_init_order().unwrap_err() {
        RuntimeError::ModuleNotFound { name } => assert_eq!(name, "alpha"),
        other => panic!("expected ModuleNotFound, got {other:?}"),
    }
}

#[test]
fn test_native_registry_collisions_and_ordering_are_deterministic() {
    let mut registry = NativeRegistry::new();
    assert!(!registry.register(NativeFunctionMeta::new("b", 1, Some("math"), "bee")));
    assert!(!registry.register(NativeFunctionMeta::new("a", 1, Some("math"), "ay")));
    assert!(!registry.register(NativeFunctionMeta::new("z", 1, None, "zed")));
    // Replacing a module entry is reported and the newest metadata wins.
    assert!(registry.register(NativeFunctionMeta::new("a", 2, Some("math"), "ay v2")));

    let names: Vec<&str> = registry
        .list_module("math")
        .iter()
        .map(|meta| meta.name.as_str())
        .collect();
    assert_eq!(names, vec!["a", "b"]);
    assert_eq!(registry.get(Some("math"), "a").unwrap().arity, 2);

    // An empty module string normalizes to the Prelude slot.
    assert!(!registry.register(NativeFunctionMeta::new("p", 0, Some(""), "prelude")));
    assert_eq!(registry.list_prelude().len(), 2);
    assert_eq!(registry.list_modules(), vec!["math"]);
    assert!(registry.get(Some(""), "p").is_none());
    assert!(registry.get(None, "p").is_some());
    // Lookup is exact and case-sensitive.
    assert!(registry.get(Some("Math"), "a").is_none());
    assert!(registry.get(Some("math"), "A").is_none());
}
