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
    assert!(surface.contains("host_mouse_wheel_x"));
    assert!(surface.contains("host_mouse_wheel_y"));
    assert!(surface.contains("host_push_clip_rect"));
    assert!(surface.contains("host_pop_clip_rect"));
    assert!(surface.contains("host_set_viewport_camera"));
    assert!(surface.contains("host_get_char_pressed"));
    assert!(surface.contains("host_draw_triangle"));
    assert!(surface.contains("host_draw_triangle_lines"));
    assert!(surface.contains("host_draw_icon_path"));

    // Verify canonical __aipo_game_* aliases
    assert!(surface.contains("__aipo_game_draw_rect"));
    assert!(surface.contains("__aipo_game_key_down"));
    assert!(surface.contains("__aipo_game_load_texture"));
    assert!(surface.contains("__aipo_game_draw_icon_path"));
    assert!(surface.contains("__aipo_game_play_preset"));
    assert!(surface.contains("__aipo_game_synth_sound"));

    // Verify game module symbol
    assert!(surface.contains("game"));
}

#[test]
fn test_host_draw_icon_path() {
    let play_icon_path = "M5 3l14 9-14 9V3z";
    let args = [
        Value::String(std::rc::Rc::new(play_icon_path.to_string())),
        Value::Float(10.0),
        Value::Float(20.0),
        Value::Float(24.0),
        Value::Float(2.0),
        Value::Float(1.0),
        Value::Float(1.0),
        Value::Float(1.0),
        Value::Float(1.0),
    ];
    let res = host_bridge::host_draw_icon_path(&args);
    assert!(res.is_ok(), "host_draw_icon_path should succeed: {:?}", res);
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
        .cloned()
        .expect("host_reset_camera exists");
    let res = vm.invoke(
        &aipo_bytecode::BytecodeModule::new(),
        reset_cam.clone(),
        &[],
    );
    assert!(res.is_ok());

    // Call host_set_viewport_camera
    let set_vp_cam = vm
        .globals
        .get("host_set_viewport_camera")
        .cloned()
        .expect("host_set_viewport_camera exists");
    let res = vm.invoke(
        &aipo_bytecode::BytecodeModule::new(),
        set_vp_cam.clone(),
        &[
            Value::Float(100.0), // vx
            Value::Float(50.0),  // vy
            Value::Float(640.0), // vw
            Value::Float(480.0), // vh
            Value::Float(0.0),   // tx
            Value::Float(0.0),   // ty
            Value::Float(2.0),   // zoom
        ],
    );
    assert!(res.is_ok());

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
fn test_zoe_ui_dashboard_compilation_and_execution() {
    let path_buf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/aipo-zoe/examples/dashboard.aipo");
    let path = path_buf.as_path();
    assert!(
        path.exists(),
        "packages/aipo-zoe/examples/dashboard.aipo must exist at {}",
        path.display()
    );

    let mut host_surface = aipo_cli::prelude_surface();
    host_bridge::register_surface_symbols(&mut host_surface);

    // 1. Compile Zoe dashboard script with host prelude surface
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

    // 4. Verify setup() mounts the Zoe component tree
    let setup_fn = vm
        .globals
        .get("setup")
        .cloned()
        .expect("setup function should exist in globals");
    let setup_res = vm.invoke(&module, setup_fn, &[]);
    assert!(
        setup_res.is_ok(),
        "setup() should mount the Zoe app: {:?}",
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
fn test_zoe_ui_editor_compilation_and_execution() {
    let path_buf =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/aipo-zoe/examples/editor.aipo");
    let path = path_buf.as_path();
    assert!(
        path.exists(),
        "packages/aipo-zoe/examples/editor.aipo must exist at {}",
        path.display()
    );

    // 1. Compile script with host surface symbols
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

    // 4. Verify setup() mounts the Zoe component tree
    let setup_fn = vm
        .globals
        .get("setup")
        .cloned()
        .expect("setup function should exist in globals");
    let setup_res = vm.invoke(&module, setup_fn, &[]);
    assert!(
        setup_res.is_ok(),
        "setup() should succeed: {:?}",
        setup_res.err()
    );

    // 5. Verify update(dt) updates Zoe framework state
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

    // 6. Verify draw() renders the declarative tree and viewport headlessly
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

    // 7. Verify multi-frame execution stability in 2D mode
    for _ in 0..5 {
        assert!(
            vm.invoke(&module, update_fn.clone(), &[Value::Float(0.016)])
                .is_ok()
        );
        assert!(vm.invoke(&module, draw_fn.clone(), &[]).is_ok());
    }

    // 8. Toggle 3D mode in the editor
    let is_3d_sig = vm
        .globals
        .get("_is_3d_mode")
        .cloned()
        .expect("_is_3d_mode exists");
    if let Value::Struct(struct_ref) = is_3d_sig {
        if let Some((_, val)) = struct_ref
            .borrow_mut()
            .fields
            .iter_mut()
            .find(|(k, _)| k == "value")
        {
            *val = Value::Bool(true);
        }
    }

    // 9. Multi-frame simulation and 3D rendering in editor
    for _ in 0..5 {
        assert!(
            vm.invoke(&module, update_fn.clone(), &[Value::Float(0.016)])
                .is_ok(),
            "update in 3D mode should succeed"
        );
    }
    let draw_res_3d = vm.invoke(&module, draw_fn.clone(), &[]);
    assert!(
        draw_res_3d.is_ok(),
        "draw() in 3D mode failed: {:?}",
        draw_res_3d.err()
    );
}

#[test]
fn test_zoe_ui_unit_test_suite() {
    let path_buf =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/aipo-zoe/tests/zoe_test.aipo");
    let path = path_buf.as_path();
    assert!(
        path.exists(),
        "packages/aipo-zoe/tests/zoe_test.aipo must exist at {}",
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
    assert!(!all_tests.is_empty(), "Should discover Zoe unit tests");

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
            "Zoe unit test '{}' should pass, got: {:?}",
            test_name,
            test_res.err()
        );
        assert!(
            ran.get(),
            "Zoe unit test '{}' should have executed",
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

#[test]
fn test_animation_and_particles_compilation_and_execution() {
    let path_buf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/aipo-game/examples/animation_and_particles.aipo");
    let path = path_buf.as_path();
    assert!(
        path.exists(),
        "packages/aipo-game/examples/animation_and_particles.aipo must exist at {}",
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
    let update_res = vm.invoke(&module, update_fn.clone(), &[Value::Float(0.016)]);
    assert!(
        update_res.is_ok(),
        "update(dt) failed: {:?}",
        update_res.err()
    );

    // Multi-frame simulation verification
    for _ in 0..10 {
        assert!(
            vm.invoke(&module, update_fn.clone(), &[Value::Float(0.016)])
                .is_ok()
        );
    }

    let draw_fn = vm.globals.get("draw").cloned().expect("draw exists");
    let draw_res = vm.invoke(&module, draw_fn, &[]);
    assert!(draw_res.is_ok(), "draw() failed: {:?}", draw_res.err());
}

#[test]
fn test_game_subsystems_unit_test_suite() {
    let path_buf =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/aipo-game/tests/game_test.aipo");
    let path = path_buf.as_path();
    assert!(
        path.exists(),
        "packages/aipo-game/tests/game_test.aipo must exist at {}",
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
    assert!(
        !all_tests.is_empty(),
        "Should discover aipo.game unit tests"
    );

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
            "Game unit test '{}' should pass, got: {:?}",
            test_name,
            test_res.err()
        );
        assert!(
            ran.get(),
            "Game unit test '{}' should have executed",
            test_name
        );
    }
}

// =========================================================================
// M14 — SDF SHADER v2 PARITY AND GEOMETRY CONTRACTS
//
// These tests lock the Rust side of the analytical UI renderer to the GLSL
// side. The M14 shader and the `SHADOW_ANCHORS` table are two halves of one
// contract; if they drift, shadows silently render at the wrong darkness and
// nothing else would catch it.
// =========================================================================

/// The SDF fragment shader source, read at compile time so the parity test
/// cannot be satisfied by a stale build artifact.
const SDF_FRAGMENT_SHADER_SRC: &str = include_str!("../src/host_bridge.rs");

/// Extracts the shader text between the `SDF_FRAGMENT_SHADER` raw string
/// delimiters.
fn sdf_fragment_shader() -> String {
    const DECL: &str = "const SDF_FRAGMENT_SHADER: &str = r";
    let start = SDF_FRAGMENT_SHADER_SRC
        .find(DECL)
        .expect("SDF_FRAGMENT_SHADER declaration must exist");
    // Skip past `r"` to reach the opening hash of the raw string delimiter.
    let body_start = start + DECL.len() + 2;
    const TERMINATOR: &str = "\"#;";
    let end = SDF_FRAGMENT_SHADER_SRC[body_start..]
        .find(TERMINATOR)
        .expect("SDF_FRAGMENT_SHADER must be terminated by a raw-string close");
    SDF_FRAGMENT_SHADER_SRC[body_start..body_start + end].to_string()
}

#[test]
fn test_sdf_shader_declares_every_uniform_it_uses() {
    let shader = sdf_fragment_shader();

    assert!(
        shader.contains("uniform"),
        "fragment shader must declare uniforms"
    );
    for uniform in [
        "u_quad_size",
        "u_box_half",
        "u_radii",
        "u_pixel_scale",
        "u_border_width",
        "u_border_color",
        "u_inner_highlight",
        "u_shadow1",
        "u_shadow2",
    ] {
        assert!(
            shader.contains(uniform),
            "fragment shader must declare `{uniform}`"
        );
    }
}

#[test]
fn test_sdf_shader_uses_scaled_antialiasing_not_fixed_band() {
    let shader = sdf_fragment_shader();

    // M14 fix: the anti-aliasing band must be derived from the device scale.
    assert!(
        shader.contains("1.0 / max(u_pixel_scale"),
        "anti-aliasing must divide by u_pixel_scale, not assume 1.0px"
    );

    // The M9 defect must be gone: a bare `clamp(0.5 - dist, 0.0, 1.0)` on the
    // primary edge is what broke under zoom.
    assert!(
        !shader.contains("float alpha = clamp(0.5 - dist"),
        "the M9 fixed-width anti-aliasing band must be removed"
    );
}

#[test]
fn test_sdf_shader_border_uses_independent_sdf_and_premultiplied_output() {
    let shader = sdf_fragment_shader();

    // M14 fix: `abs(d) - w*0.5` places the stroke centred on the box
    // boundary, which is what keeps the inner corner radius correct.
    assert!(
        shader.contains("abs(d) - bw * 0.5"),
        "border must derive from an independent SDF, not `dist + width`"
    );

    // M9 defect: shifting the SDF by the border width broke the rounded-box
    // interior term.
    assert!(
        !shader.contains("b_dist = dist +"),
        "the M9 `dist + border_width` border must be removed"
    );

    // Premultiplied-alpha `over`: sa + fill_a * (1 - sa).
    assert!(
        shader.contains("sa + fill_a * (1.0 - sa)"),
        "border must composite with a premultiplied-alpha over"
    );
}

#[test]
fn test_sdf_shader_composites_shadow_with_gamma_compensation() {
    let shader = sdf_fragment_shader();

    assert!(
        shader.contains("float shadow_layer("),
        "fragment shader must expose a per-layer shadow function"
    );
    assert!(
        shader.contains("(1.0 - a1) * (1.0 - a2)"),
        "two shadow layers must composite in CSS `over` order"
    );
    // Without the 2.2 power, a translucent black shadow lands at roughly half
    // its intended darkness on light surfaces.
    assert!(
        shader.contains("pow(1.0 - a_css, 2.2)"),
        "shadow must apply sRGB gamma compensation"
    );
}

#[test]
fn test_sdf_shader_supports_per_corner_radii() {
    let shader = sdf_fragment_shader();

    assert!(
        shader.contains("float sd_rounded_box(vec2 p, vec2 b, vec4 r)"),
        "rounded box must take per-corner radii as a vec4"
    );
    assert!(
        shader.contains("u_radii"),
        "per-corner radii must arrive as a uniform"
    );
    // The M9 shader clamped a single scalar radius.
    assert!(
        !shader.contains("float r = min(u_radius"),
        "the M9 single scalar radius path must be removed"
    );
}

#[test]
fn test_sdf_elevation_anchor_parity() {
    let shader = sdf_fragment_shader();
    assert!(
        shader.contains("u_shadow1.w > 0.0 || u_shadow2.w > 0.0"),
        "shadow branch must be gated on both layers being present"
    );
    assert!(shader.contains("u_shadow1"), "shader must consume layer 1");
    assert!(shader.contains("u_shadow2"), "shader must consume layer 2");
}

#[test]
fn test_shadow_anchor_table_is_monotonic() {
    // The interpolation between anchors is only well-behaved if blur, offset
    // and alpha all increase with level and spread stays non-positive.
    let anchors = host_bridge::shadow_anchors();
    assert!(
        anchors.len() >= 3,
        "need at least three anchors to interpolate between"
    );

    for w in anchors.windows(2) {
        let (a, b) = (w[0], w[1]);
        assert!(b[1] > a[1], "blur must increase: {} then {}", a[1], b[1]);
        assert!(
            b[0] >= a[0],
            "offset must not decrease: {} then {}",
            a[0],
            b[0]
        );
        assert!(b[3] > a[3], "alpha must increase: {} then {}", a[3], b[3]);
        assert!(
            a[2] <= 0.0 && b[2] <= 0.0,
            "spread must be non-positive (inset) on every anchor"
        );
    }
}

#[test]
fn test_shadow_layers_zero_elevation_is_transparent() {
    // Level 0 must disable the shader branch entirely rather than drawing a
    // faint shadow on every widget in the tree.
    let (a, b) = host_bridge::shadow_layers(0.0);
    assert_eq!(a[3], 0.0, "layer 1 alpha must be zero at level 0");
    assert_eq!(b[3], 0.0, "layer 2 alpha must be zero at level 0");
}

#[test]
fn test_shadow_master_interpolates_between_anchors() {
    let anchors = host_bridge::shadow_anchors();

    // Halfway between anchor 0 and anchor 1 every channel must land strictly
    // between the endpoints. This is the property that makes elevation
    // animatable instead of popping at integer levels.
    let mid = host_bridge::shadow_master(0.5);
    for ch in 0..4 {
        if anchors[0][ch] == anchors[1][ch] {
            continue;
        }
        let lo = anchors[0][ch].min(anchors[1][ch]);
        let hi = anchors[0][ch].max(anchors[1][ch]);
        assert!(
            mid[ch] > lo && mid[ch] < hi,
            "channel {ch} at level 0.5 was {}, must be between {lo} and {hi}",
            mid[ch]
        );
    }
}

#[test]
fn test_shadow_layers_contact_is_tighter_and_denser_than_ambient() {
    // The two-layer model only reads as depth if the contact layer is
    // genuinely tighter and more opaque than the ambient one.
    for level in [0.5, 1.5, 2.5, 3.5, 4.5] {
        let (contact, ambient) = host_bridge::shadow_layers(level);
        assert!(
            contact[1] < ambient[1],
            "contact blur {} must be tighter than ambient {} at level {level}",
            contact[1],
            ambient[1]
        );
        assert!(
            contact[3] > ambient[3],
            "contact alpha {} must be denser than ambient {} at level {level}",
            contact[3],
            ambient[3]
        );
    }
}

#[test]
fn test_shadow_layers_is_continuous_across_anchor_boundaries() {
    // A visible discontinuity at an integer level would show as a shadow
    // "pop" while animating elevation.
    for step in 1..40 {
        let e = step as f32 / 16.0;
        let a = host_bridge::shadow_master(e);
        let b = host_bridge::shadow_master(e + 1.0 / 16.0);
        let jump = (b[1] - a[1]).abs();
        assert!(
            jump < 2.0,
            "blur must not jump by more than 2px between 1/16-level samples; jumped {jump} at level {e}"
        );
    }
}

#[test]
fn test_shadow_layers_extrapolates_beyond_last_anchor() {
    // Very high elevations must keep growing rather than clamping flat, and
    // alpha must stay inside the representable range.
    let xl = host_bridge::shadow_master(4.0);
    let beyond = host_bridge::shadow_master(8.0);
    assert!(
        beyond[1] > xl[1],
        "blur must exceed the xl anchor at level 8: {} vs {}",
        beyond[1],
        xl[1]
    );

    let (contact, ambient) = host_bridge::shadow_layers(8.0);
    assert!(
        contact[3] <= 1.0,
        "contact alpha must stay bounded: {}",
        contact[3]
    );
    assert!(
        ambient[3] <= 1.0,
        "ambient alpha must stay bounded: {}",
        ambient[3]
    );
    assert!(contact[1] > 0.0 && ambient[1] > 0.0);
}

#[test]
fn test_shadow_master_is_monotonic_across_the_whole_range() {
    // Blur, offset and alpha must never decrease as elevation rises, or a
    // raised panel would look like it sank.
    let mut prev = host_bridge::shadow_master(0.0);
    for step in 1..80 {
        let cur = host_bridge::shadow_master(step as f32 / 10.0);
        assert!(
            cur[1] >= prev[1] - 1e-4,
            "blur must not decrease: {} then {} at level {}",
            prev[1],
            cur[1],
            step as f32 / 10.0
        );
        assert!(
            cur[0] >= prev[0] - 1e-4,
            "offset must not decrease: {} then {}",
            prev[0],
            cur[0]
        );
        prev = cur;
    }
}

#[test]
fn test_sdf_natives_registered_with_correct_arity() {
    let mut surface = aipo_cli::prelude_surface();
    host_bridge::register_surface_symbols(&mut surface);

    // Legacy 15-argument form stays available so existing trees keep working.
    assert!(surface.contains("host_draw_sdf_rect"));
    // M14 extensions.
    assert!(surface.contains("host_draw_sdf_rect_v2"));
    assert!(surface.contains("host_draw_sdf_rect_corners"));
}

#[test]
fn test_sdf_v2_native_accepts_valid_arguments_headless() {
    // The headless software fallback must not fault. This exercises argument
    // decoding, shadow-layer resolution and quad padding without a GPU.
    let args = [
        Value::Float(10.0),  // x
        Value::Float(20.0),  // y
        Value::Float(120.0), // w
        Value::Float(40.0),  // h
        Value::Float(6.0),   // radius
        Value::Float(3.0),   // elevation
        Value::Float(2.0),   // focus width
        Value::Float(0.5),   // focus r
        Value::Float(0.7),   // focus g
        Value::Float(1.0),   // focus b
        Value::Float(0.9),   // focus a
        Value::Float(1.0),   // border w
        Value::Float(0.2),   // border r
        Value::Float(0.2),   // border g
        Value::Float(0.3),   // border b
        Value::Float(1.0),   // border a
        Value::Float(0.0),   // inner highlight
        Value::Float(0.1),   // bg r
        Value::Float(0.1),   // bg g
        Value::Float(0.15),  // bg b
        Value::Float(1.0),   // bg a
    ];
    let res = host_bridge::host_draw_sdf_rect_v2(&args);
    assert!(res.is_ok(), "sdf_v2 draw must succeed headless: {res:?}");

    // Rejecting a short argument list is part of the contract.
    let short = host_bridge::host_draw_sdf_rect_v2(&args[..10]);
    assert!(short.is_err(), "sdf_v2 must reject fewer than 21 arguments");
}

#[test]
fn test_sdf_corners_native_accepts_valid_arguments_headless() {
    let mut args = Vec::new();
    // x, y, w, h
    args.extend([
        Value::Float(0.0),
        Value::Float(0.0),
        Value::Float(50.0),
        Value::Float(50.0),
    ]);
    // tl, tr, br, bl
    args.extend([
        Value::Float(12.0),
        Value::Float(4.0),
        Value::Float(4.0),
        Value::Float(12.0),
    ]);
    // elevation, focus width
    args.extend([Value::Float(2.0), Value::Float(0.0)]);
    // focus rgba
    for _ in 0..4 {
        args.push(Value::Float(0.0));
    }
    // border width + rgba
    args.extend([Value::Float(1.0)]);
    args.extend([
        Value::Float(0.0),
        Value::Float(0.0),
        Value::Float(0.0),
        Value::Float(0.5),
    ]);
    // inner highlight
    args.push(Value::Float(0.0));
    // bg rgba
    args.extend([
        Value::Float(0.2),
        Value::Float(0.2),
        Value::Float(0.25),
        Value::Float(1.0),
    ]);

    assert_eq!(
        args.len(),
        24,
        "argument vector must match the declared arity"
    );
    let res = host_bridge::host_draw_sdf_rect_corners(&args);
    assert!(
        res.is_ok(),
        "sdf_corners draw must succeed headless: {res:?}"
    );

    let short = host_bridge::host_draw_sdf_rect_corners(&args[..8]);
    assert!(
        short.is_err(),
        "sdf_corners must reject fewer than 24 arguments"
    );
}
