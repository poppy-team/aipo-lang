//! Automated tests for the Aipo Game Host bridge and dynamic script execution.

#![forbid(unsafe_code)]

use aipo_game_host::host_bridge;
use aipo_vm::Value;
use std::path::Path;

#[test]
fn test_host_surface_registration() {
    let mut surface = aipo_cli::prelude_surface();
    host_bridge::register_surface_symbols(&mut surface);

    // Verify key host functions are registered
    assert!(surface.contains("host_draw_rect"));
    assert!(surface.contains("host_draw_circle"));
    assert!(surface.contains("host_draw_text"));
    assert!(surface.contains("host_key_down"));
    assert!(surface.contains("host_mouse_x"));
    assert!(surface.contains("host_load_texture"));
    assert!(surface.contains("host_draw_sprite"));
    assert!(surface.contains("host_set_camera"));
    assert!(surface.contains("host_load_sound"));
    assert!(surface.contains("host_play_sound"));
    assert!(surface.contains("host_play_preset"));
    assert!(surface.contains("host_synth_sound"));
    assert!(surface.contains("host_play_music"));
    assert!(surface.contains("host_stop_music"));

    // Verify canonical __aipo_game_* aliases
    assert!(surface.contains("__aipo_game_draw_rect"));
    assert!(surface.contains("__aipo_game_key_down"));
    assert!(surface.contains("__aipo_game_load_texture"));
    assert!(surface.contains("__aipo_game_play_preset"));
    assert!(surface.contains("__aipo_game_synth_sound"));

    // Verify game module symbol
    assert!(surface.contains("game"));
}

#[test]
fn test_host_vm_natives_registration() {
    let (mut vm, _) = aipo_cli::standard_environment();
    host_bridge::register_vm_natives(&mut vm);

    // Verify globals exist
    assert!(vm.globals.contains_key("host_draw_rect"));
    assert!(vm.globals.contains_key("__aipo_game_draw_rect"));
    assert!(vm.globals.contains_key("host_key_down"));
    assert!(vm.globals.contains_key("host_play_preset"));
    assert!(vm.globals.contains_key("__aipo_game_play_preset"));
    assert!(vm.globals.contains_key("game"));

    // Verify `game` is a dictionary with methods
    let game_val = vm.globals.get("game").expect("game global exists");
    match game_val {
        Value::Dict(map) => {
            let borrowed = map.borrow();
            assert!(
                borrowed
                    .get(&Value::String(std::rc::Rc::new("draw_rect".to_string())))
                    .is_some()
            );
            assert!(
                borrowed
                    .get(&Value::String(std::rc::Rc::new("key_down".to_string())))
                    .is_some()
            );
            assert!(
                borrowed
                    .get(&Value::String(std::rc::Rc::new("load_texture".to_string())))
                    .is_some()
            );
            assert!(
                borrowed
                    .get(&Value::String(std::rc::Rc::new("play_preset".to_string())))
                    .is_some()
            );
            assert!(
                borrowed
                    .get(&Value::String(std::rc::Rc::new("synth_sound".to_string())))
                    .is_some()
            );
        }
        other => panic!("expected game to be Dict, got {}", other.type_name()),
    }
}

#[test]
fn test_interactive_game_script_compilation_and_execution() {
    let path_buf =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/26_interactive_game.aipo");
    let path = path_buf.as_path();
    assert!(
        path.exists(),
        "examples/26_interactive_game.aipo must exist at {}",
        path.display()
    );

    let mut host_surface = aipo_cli::prelude_surface();
    host_bridge::register_surface_symbols(&mut host_surface);

    // 1. Compile script with host surface
    let (_source, module) = match aipo_cli::compile_file(path, Some(&host_surface)) {
        Ok(res) => res,
        Err((_src, diags)) => {
            panic!("Compilation failed with diagnostics: {:?}", diags);
        }
    };

    assert!(
        !module.code.is_empty(),
        "Bytecode module should not be empty"
    );

    // 2. Initialize VM and register host natives
    let (mut vm, _) = aipo_cli::standard_environment();
    aipo_cli::register_module_symbols(&mut vm, &module);
    host_bridge::register_vm_natives(&mut vm);

    // 3. Execute top-level statements
    let run_res = vm.run(&module);
    assert!(
        run_res.is_ok(),
        "Top-level execution should succeed: {:?}",
        run_res.err()
    );

    // 4. Verify setup function is callable
    let setup_fn = vm
        .globals
        .get("setup")
        .cloned()
        .expect("setup function should exist in globals");
    let setup_res = vm.invoke(&module, setup_fn, &[]);
    assert!(setup_res.is_ok(), "setup() should succeed");

    // 5. Verify update function is callable with delta time
    let update_fn = vm
        .globals
        .get("update")
        .cloned()
        .expect("update function should exist in globals");
    let update_res = vm.invoke(&module, update_fn, &[Value::Float(0.016)]);
    assert!(
        update_res.is_ok(),
        "update(dt) should succeed: {:?}",
        update_res.err()
    );

    // 6. Verify draw function is callable without panicking
    let draw_fn = vm
        .globals
        .get("draw")
        .cloned()
        .expect("draw function should exist in globals");
    let draw_res = vm.invoke(&module, draw_fn, &[]);
    assert!(
        draw_res.is_ok(),
        "draw() should succeed headlessly without panic: {:?}",
        draw_res.err()
    );
}

#[test]
fn test_camera_and_sprite_host_calls() {
    let (mut vm, _) = aipo_cli::standard_environment();
    host_bridge::register_vm_natives(&mut vm);

    // Call host_set_camera
    let set_cam = vm
        .globals
        .get("host_set_camera")
        .expect("host_set_camera exists");
    let res = vm.invoke(
        &aipo_bytecode::BytecodeModule::new(),
        set_cam.clone(),
        &[Value::Float(100.0), Value::Float(150.0), Value::Float(1.5)],
    );
    assert!(res.is_ok());

    // Call host_reset_camera
    let reset_cam = vm
        .globals
        .get("host_reset_camera")
        .expect("host_reset_camera exists");
    let res = vm.invoke(
        &aipo_bytecode::BytecodeModule::new(),
        reset_cam.clone(),
        &[],
    );
    assert!(res.is_ok());

    // Call host_load_texture (nonexistent path returns 0 fallback)
    let load_tex = vm
        .globals
        .get("host_load_texture")
        .expect("host_load_texture exists");
    let res = vm.invoke(
        &aipo_bytecode::BytecodeModule::new(),
        load_tex.clone(),
        &[Value::String(std::rc::Rc::new(
            "nonexistent.png".to_string(),
        ))],
    );
    assert_eq!(res.unwrap(), Value::Int(0));

    // Call host_draw_sprite with fallback texture (id 0)
    let draw_sprite = vm
        .globals
        .get("host_draw_sprite")
        .expect("host_draw_sprite exists");
    let res = vm.invoke(
        &aipo_bytecode::BytecodeModule::new(),
        draw_sprite.clone(),
        &[
            Value::Int(0),
            Value::Float(10.0),
            Value::Float(20.0),
            Value::Float(32.0),
            Value::Float(32.0),
            Value::Float(0.0),
            Value::Bool(false),
        ],
    );
    assert!(res.is_ok());

    // Call host_draw_sprite_subrect (spritesheet slice)
    let draw_sub = vm
        .globals
        .get("host_draw_sprite_subrect")
        .expect("host_draw_sprite_subrect exists");
    let res = vm.invoke(
        &aipo_bytecode::BytecodeModule::new(),
        draw_sub.clone(),
        &[
            Value::Int(0),
            Value::Float(0.0),
            Value::Float(0.0),
            Value::Float(16.0),
            Value::Float(16.0),
            Value::Float(10.0),
            Value::Float(20.0),
            Value::Float(32.0),
            Value::Float(32.0),
            Value::Bool(true),
        ],
    );
    assert!(res.is_ok());
}

#[test]
fn test_camera_and_sprites_script_compilation_and_execution() {
    let path_buf =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/27_camera_and_sprites.aipo");
    let path = path_buf.as_path();
    assert!(
        path.exists(),
        "examples/27_camera_and_sprites.aipo must exist at {}",
        path.display()
    );

    let mut host_surface = aipo_cli::prelude_surface();
    host_bridge::register_surface_symbols(&mut host_surface);

    let (_source, module) = match aipo_cli::compile_file(path, Some(&host_surface)) {
        Ok(res) => res,
        Err((_src, diags)) => {
            panic!("Compilation failed with diagnostics: {:?}", diags);
        }
    };

    assert!(
        !module.code.is_empty(),
        "Bytecode module should not be empty"
    );

    let (mut vm, _) = aipo_cli::standard_environment();
    aipo_cli::register_module_symbols(&mut vm, &module);
    host_bridge::register_vm_natives(&mut vm);

    let run_res = vm.run(&module);
    assert!(
        run_res.is_ok(),
        "Top-level execution should succeed: {:?}",
        run_res.err()
    );

    let setup_fn = vm.globals.get("setup").cloned().expect("setup exists");
    assert!(vm.invoke(&module, setup_fn, &[]).is_ok());

    let update_fn = vm.globals.get("update").cloned().expect("update exists");
    assert!(
        vm.invoke(&module, update_fn, &[Value::Float(0.016)])
            .is_ok()
    );

    let draw_fn = vm.globals.get("draw").cloned().expect("draw exists");
    assert!(vm.invoke(&module, draw_fn, &[]).is_ok());
}

#[test]
fn test_audio_host_calls() {
    let (mut vm, _) = aipo_cli::standard_environment();
    host_bridge::register_vm_natives(&mut vm);

    let empty_mod = aipo_bytecode::BytecodeModule::new();

    // 1. Call host_load_sound with nonexistent file (returns 0 safely)
    let load_snd = vm
        .globals
        .get("host_load_sound")
        .expect("host_load_sound exists");
    let res = vm.invoke(
        &empty_mod,
        load_snd.clone(),
        &[Value::String(std::rc::Rc::new(
            "nonexistent_audio.wav".to_string(),
        ))],
    );
    assert_eq!(res.unwrap(), Value::Int(0));

    // 2. Call host_play_preset ("coin", volume 0.8, pitch 1.0)
    let play_preset = vm
        .globals
        .get("host_play_preset")
        .expect("host_play_preset exists");
    let res = vm.invoke(
        &empty_mod,
        play_preset.clone(),
        &[
            Value::String(std::rc::Rc::new("coin".to_string())),
            Value::Float(0.8),
            Value::Float(1.0),
        ],
    );
    assert!(res.is_ok(), "host_play_preset should succeed headlessly");

    // 3. Call host_synth_sound to dynamically synthesize a sound
    let synth_snd = vm
        .globals
        .get("host_synth_sound")
        .expect("host_synth_sound exists");
    let res = vm.invoke(
        &empty_mod,
        synth_snd.clone(),
        &[
            Value::String(std::rc::Rc::new("square".to_string())),
            Value::Float(440.0),
            Value::Float(0.2),
            Value::Float(0.1),
            Value::Float(0.8),
        ],
    );
    assert!(res.is_ok(), "host_synth_sound should succeed");
    let snd_id = match res.unwrap() {
        Value::Int(id) => {
            assert!(id > 0);
            id
        }
        other => panic!("expected Int sound id, got {:?}", other),
    };

    // 4. Call host_play_sound with synthesized sound handle
    let play_snd = vm
        .globals
        .get("host_play_sound")
        .expect("host_play_sound exists");
    let res = vm.invoke(
        &empty_mod,
        play_snd.clone(),
        &[Value::Int(snd_id), Value::Float(0.9), Value::Float(1.0)],
    );
    assert!(res.is_ok(), "host_play_sound should succeed");

    // 5. Call host_stop_sound
    let stop_snd = vm
        .globals
        .get("host_stop_sound")
        .expect("host_stop_sound exists");
    let res = vm.invoke(&empty_mod, stop_snd.clone(), &[Value::Int(snd_id)]);
    assert!(res.is_ok(), "host_stop_sound should succeed");

    // 6. Call host_play_music and host_stop_music
    let play_music = vm
        .globals
        .get("host_play_music")
        .expect("host_play_music exists");
    let res = vm.invoke(
        &empty_mod,
        play_music.clone(),
        &[Value::Int(snd_id), Value::Float(0.5), Value::Bool(true)],
    );
    assert!(res.is_ok(), "host_play_music should succeed");

    let stop_music = vm
        .globals
        .get("host_stop_music")
        .expect("host_stop_music exists");
    let res = vm.invoke(&empty_mod, stop_music.clone(), &[]);
    assert!(res.is_ok(), "host_stop_music should succeed");
}

#[test]
fn test_freya_ui_dashboard_compilation_and_execution() {
    let path_buf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/aipo-freya/examples/dashboard.aipo");
    let path = path_buf.as_path();
    assert!(
        path.exists(),
        "packages/aipo-freya/examples/dashboard.aipo must exist at {}",
        path.display()
    );

    let mut host_surface = aipo_cli::prelude_surface();
    host_bridge::register_surface_symbols(&mut host_surface);

    // 1. Compile Freya dashboard script with host prelude surface
    let (_source, module) = match aipo_cli::compile_file(path, Some(&host_surface)) {
        Ok(res) => res,
        Err((_src, diags)) => {
            panic!("Compilation failed with diagnostics: {:?}", diags);
        }
    };

    assert!(
        !module.code.is_empty(),
        "Bytecode module should not be empty"
    );

    // 2. Initialize VM and register host natives
    let (mut vm, _) = aipo_cli::standard_environment();
    aipo_cli::register_module_symbols(&mut vm, &module);
    host_bridge::register_vm_natives(&mut vm);

    // 3. Execute top-level script statements
    let run_res = vm.run(&module);
    assert!(
        run_res.is_ok(),
        "Top-level execution should succeed: {:?}",
        run_res.err()
    );

    // 4. Verify setup() mounts the Freya component tree
    let setup_fn = vm
        .globals
        .get("setup")
        .cloned()
        .expect("setup function should exist in globals");
    let setup_res = vm.invoke(&module, setup_fn, &[]);
    assert!(
        setup_res.is_ok(),
        "setup() should mount the Freya app: {:?}",
        setup_res.err()
    );

    // 5. Verify update(dt) processes layout and events
    let update_fn = vm
        .globals
        .get("update")
        .cloned()
        .expect("update function should exist in globals");
    let update_res = vm.invoke(&module, update_fn.clone(), &[Value::Float(0.016)]);
    assert!(
        update_res.is_ok(),
        "update(dt) should succeed: {:?}",
        update_res.err()
    );

    // 6. Verify draw() renders the declarative tree headlessly
    let draw_fn = vm
        .globals
        .get("draw")
        .cloned()
        .expect("draw function should exist in globals");
    let draw_res = vm.invoke(&module, draw_fn.clone(), &[]);
    assert!(
        draw_res.is_ok(),
        "draw() should succeed headlessly without panic: {:?}",
        draw_res.err()
    );

    // 7. Verify multi-frame execution stability
    for _ in 0..5 {
        assert!(
            vm.invoke(&module, update_fn.clone(), &[Value::Float(0.016)])
                .is_ok()
        );
        assert!(vm.invoke(&module, draw_fn.clone(), &[]).is_ok());
    }
}

#[test]
fn test_freya_ui_unit_test_suite() {
    let path_buf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/aipo-freya/tests/freya_test.aipo");
    let path = path_buf.as_path();
    assert!(
        path.exists(),
        "packages/aipo-freya/tests/freya_test.aipo must exist at {}",
        path.display()
    );

    let mut host_surface = aipo_cli::prelude_surface();
    host_bridge::register_surface_symbols(&mut host_surface);

    let (_source, module) = match aipo_cli::compile_file(path, Some(&host_surface)) {
        Ok(res) => res,
        Err((_src, diags)) => {
            panic!("Compilation failed with diagnostics: {:?}", diags);
        }
    };

    // 1. Discover all tests
    let (mut vm, _) = aipo_cli::standard_environment();
    aipo_cli::register_module_symbols(&mut vm, &module);
    host_bridge::register_vm_natives(&mut vm);

    let discovered = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    vm.set_test_mode(aipo_vm::TestMode::Discover(discovered.clone()));
    let run_res = vm.run(&module);
    assert!(
        run_res.is_ok(),
        "Discovery run should succeed: {:?}",
        run_res.err()
    );

    let all_tests = discovered.borrow().clone();
    assert!(!all_tests.is_empty(), "Should discover Freya unit tests");

    // 2. Execute each discovered test in isolated VM with host natives
    for test_name in &all_tests {
        let (mut test_vm, _) = aipo_cli::standard_environment();
        aipo_cli::register_module_symbols(&mut test_vm, &module);
        host_bridge::register_vm_natives(&mut test_vm);

        let ran = std::rc::Rc::new(std::cell::Cell::new(false));
        test_vm.set_test_mode(aipo_vm::TestMode::Execute {
            target: test_name.clone(),
            ran: ran.clone(),
        });

        let test_res = test_vm.run(&module);
        assert!(
            test_res.is_ok(),
            "Freya unit test '{}' should pass, got: {:?}",
            test_name,
            test_res.err()
        );
        assert!(
            ran.get(),
            "Freya unit test '{}' should have executed",
            test_name
        );
    }
}

#[test]
fn test_tilemap_and_game_hud_compilation_and_execution() {
    let path_buf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/aipo-game/examples/tilemap_and_hud.aipo");
    let path = path_buf.as_path();
    assert!(
        path.exists(),
        "packages/aipo-game/examples/tilemap_and_hud.aipo must exist at {}",
        path.display()
    );

    let mut host_surface = aipo_cli::prelude_surface();
    host_bridge::register_surface_symbols(&mut host_surface);

    let (_source, module) = match aipo_cli::compile_file(path, Some(&host_surface)) {
        Ok(res) => res,
        Err((_src, diags)) => {
            panic!("Compilation failed with diagnostics: {:?}", diags);
        }
    };

    assert!(
        !module.code.is_empty(),
        "Bytecode module should not be empty"
    );

    let (mut vm, _) = aipo_cli::standard_environment();
    aipo_cli::register_module_symbols(&mut vm, &module);
    host_bridge::register_vm_natives(&mut vm);

    let run_res = vm.run(&module);
    assert!(
        run_res.is_ok(),
        "Top-level execution should succeed: {:?}",
        run_res.err()
    );

    let setup_fn = vm.globals.get("setup").cloned().expect("setup exists");
    let setup_res = vm.invoke(&module, setup_fn, &[]);
    assert!(setup_res.is_ok(), "setup() failed: {:?}", setup_res.err());

    let update_fn = vm.globals.get("update").cloned().expect("update exists");
    let update_res = vm.invoke(&module, update_fn, &[Value::Float(0.016)]);
    assert!(
        update_res.is_ok(),
        "update(dt) failed: {:?}",
        update_res.err()
    );

    let draw_fn = vm.globals.get("draw").cloned().expect("draw exists");
    assert!(vm.invoke(&module, draw_fn, &[]).is_ok());
}
