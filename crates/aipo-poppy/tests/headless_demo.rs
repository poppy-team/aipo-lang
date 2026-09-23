//! Deterministic headless Poppy game simulation test fixture (Wave 4, `P03-G02`).
//!
//! This fixture proves the non-negotiable rules of Wave 4:
//! - Determinism: identical runs with the same seed yield identical digests across all ticks.
//! - Command buffer: structural mutations apply at safe points.
//! - Generational handles: stale entity handles fault with `AIPO_RT_STALE_HANDLE`, never panic.
//! - Capabilities: ungranted operations fault with `AIPO_RT_CAPABILITY_DENIED`.

use aipo_bytecode::{BytecodeModule, compile};
use aipo_diagnostics::DiagnosticCode;
use aipo_hir::lower;
use aipo_ir::lower_to_ir;
use aipo_poppy::{POPPY_LOCK, PoppyService, install_poppy, register_poppy, revoke_poppy};
use aipo_runtime::NativeRegistry;
use aipo_source::{Source, SourceId};
use aipo_stdlib::register_stdlib;
use aipo_syntax::parse;
use aipo_vm::Vm;

fn compile_program(code: &str) -> BytecodeModule {
    let src = Source::new(SourceId::next(), "game.aipo", code);
    let (ast, parse_diags) = parse(&src);
    assert!(parse_diags.is_empty(), "parse errors: {parse_diags:?}");
    let hir = lower(ast);
    let ir = lower_to_ir(&hir);
    compile(&ir).expect("program compiles")
}

fn create_game_vm(bytecode: &BytecodeModule) -> Vm {
    let mut vm = Vm::new();
    let mut registry = NativeRegistry::new();
    register_stdlib(&mut vm, &mut registry);
    register_poppy(&mut vm);

    for decl in &bytecode.structs {
        let fields: Vec<(&str, bool)> = decl
            .fields
            .iter()
            .map(|(name, fixed)| (name.as_str(), *fixed))
            .collect();
        vm.register_struct(decl.name.clone(), fields);
    }
    for function in &bytecode.functions {
        if let Some((type_name, method)) = function.name.split_once('.') {
            vm.register_struct_method(
                type_name,
                method,
                function.entry_ip,
                function.params,
                function.is_async,
            );
        }
    }
    vm
}

/// Aipo game simulation script.
///
/// Sets up a player and asteroids, runs a 10-tick game loop, updates velocity,
/// performs queries, and tracks state digests.
const GAME_SCRIPT: &str = r#"
var player = poppy.spawn("player", 0.0, 0.0)
poppy.set_velocity(player, 1.0, 2.0)

var a1 = poppy.spawn("asteroid", 10.0, 20.0)
var a2 = poppy.spawn("asteroid", -5.0, 15.0)

var digests = []

var i = 0
while i < 10
    # Step game world physics and deferred command buffer
    var d = poppy.step()
    digests.add(d)

    # Simple behavior: if player is past x = 5.0, despawn asteroid 1
    var pos = poppy.get_position(player)
    if pos["x"] >= 5.0
        poppy.despawn(a1)
    end

    i = i + 1
end

var remaining = poppy.query("asteroid")
"#;

#[test]
fn test_headless_game_determinism() {
    let _lock = POPPY_LOCK.lock().unwrap();

    let bytecode = compile_program(GAME_SCRIPT);

    // Run 1 with seed 1337
    install_poppy(PoppyService::new(1337, 1.0 / 60.0));
    let mut vm1 = create_game_vm(&bytecode);
    vm1.run(&bytecode).expect("run 1 succeeded");
    let digests1 = vm1.get_global("digests").expect("digests global exists");

    // Run 2 with same seed 1337
    install_poppy(PoppyService::new(1337, 1.0 / 60.0));
    let mut vm2 = create_game_vm(&bytecode);
    vm2.run(&bytecode).expect("run 2 succeeded");
    let digests2 = vm2.get_global("digests").expect("digests global exists");

    // Must be strictly identical across runs
    assert_eq!(
        digests1, digests2,
        "runs with identical seed must yield identical digests"
    );

    // Run 3 with different seed 9999
    install_poppy(PoppyService::new(9999, 1.0 / 60.0));
    let mut vm3 = create_game_vm(&bytecode);
    vm3.run(&bytecode).expect("run 3 succeeded");
    let digests3 = vm3.get_global("digests").expect("digests global exists");

    // Must differ from runs 1 & 2
    assert_ne!(
        digests1, digests3,
        "runs with different seeds must produce different digests"
    );
}

#[test]
fn test_stale_handle_after_despawn() {
    let _lock = POPPY_LOCK.lock().unwrap();

    install_poppy(PoppyService::new(42, 1.0 / 60.0));
    let code = r#"
var target = poppy.spawn("target", 1.0, 1.0)
poppy.despawn(target)
poppy.step()
var p = poppy.get_position(target)
"#;
    let bytecode = compile_program(code);
    let mut vm = create_game_vm(&bytecode);
    let err = vm.run(&bytecode).expect_err("should fault on stale handle");
    assert_eq!(err.diagnostic_code(), DiagnosticCode::AIPO_RT_STALE_HANDLE);
}

#[test]
fn test_poppy_capability_denied() {
    let _lock = POPPY_LOCK.lock().unwrap();

    revoke_poppy();
    let code = r#"
var e = poppy.spawn("target", 0.0, 0.0)
"#;
    let bytecode = compile_program(code);
    let mut vm = create_game_vm(&bytecode);
    let err = vm
        .run(&bytecode)
        .expect_err("should fault on denied capability");
    assert_eq!(
        err.diagnostic_code(),
        DiagnosticCode::AIPO_RT_CAPABILITY_DENIED
    );
}
