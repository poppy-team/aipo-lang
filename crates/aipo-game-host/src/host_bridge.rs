//! Host FFI bridge connecting Aipo VM bytecode execution with Macroquad/Miniquad GPU rendering.
//!
//! Provides hardware-accelerated 2D draw calls, real-time keyboard/mouse input,
//! texture and spritesheet pipeline, and camera transformations with headless fallback.

#![forbid(unsafe_code)]

use aipo_sema::PreludeSurface;
use aipo_vm::{Value, Vm, VmFault};
use macroquad::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Mutex;

/// Texture cache for storing GPU textures loaded by scripts.
struct TextureCache {
    textures: Vec<Texture2D>,
    fallback_texture: Option<Texture2D>,
}

impl TextureCache {
    const fn new() -> Self {
        Self {
            textures: Vec::new(),
            fallback_texture: None,
        }
    }

    fn get_fallback(&mut self) -> Option<Texture2D> {
        if let Some(ref tex) = self.fallback_texture {
            return Some(tex.clone());
        }
        let tex = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Texture2D::from_rgba8(1, 1, &[255, 255, 255, 255])
        }))
        .ok()?;
        self.fallback_texture = Some(tex.clone());
        Some(tex)
    }

    fn load(&mut self, path: &str) -> i64 {
        match std::fs::read(path) {
            Ok(bytes) => {
                let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    Texture2D::from_file_with_format(&bytes, None)
                }));
                match res {
                    Ok(tex) => {
                        self.textures.push(tex);
                        self.textures.len() as i64
                    }
                    Err(_) => 0,
                }
            }
            Err(err) => {
                eprintln!("[aipo-game-host] warning: could not load texture '{path}': {err}");
                0
            }
        }
    }

    fn get(&mut self, id: i64) -> Option<Texture2D> {
        if id > 0 && (id as usize) <= self.textures.len() {
            Some(self.textures[(id - 1) as usize].clone())
        } else {
            self.get_fallback()
        }
    }
}

static TEXTURE_CACHE: Mutex<TextureCache> = Mutex::new(TextureCache::new());

// --- Headless-safe Macroquad Wrappers ---

fn safe_is_key_down(key: KeyCode) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| is_key_down(key))).unwrap_or(false)
}

fn safe_is_key_pressed(key: KeyCode) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| is_key_pressed(key))).unwrap_or(false)
}

fn safe_mouse_position() -> (f32, f32) {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(mouse_position)).unwrap_or((0.0, 0.0))
}

fn safe_is_mouse_button_down(mb: MouseButton) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| is_mouse_button_down(mb)))
        .unwrap_or(false)
}

fn safe_screen_width() -> f32 {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(screen_width)).unwrap_or(840.0)
}

fn safe_screen_height() -> f32 {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(screen_height)).unwrap_or(680.0)
}

fn safe_frame_time() -> f32 {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(get_frame_time)).unwrap_or(1.0 / 60.0)
}

fn safe_draw_rect(x: f32, y: f32, w: f32, h: f32, color: Color) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        draw_rectangle(x, y, w, h, color)
    }));
}

fn safe_draw_rect_lines(x: f32, y: f32, w: f32, h: f32, th: f32, color: Color) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        draw_rectangle_lines(x, y, w, h, th, color)
    }));
}

fn safe_draw_circle(cx: f32, cy: f32, radius: f32, color: Color) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        draw_circle(cx, cy, radius, color)
    }));
}

fn safe_draw_text(text: &str, x: f32, y: f32, size: f32, color: Color) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        draw_text(text, x, y, size, color)
    }));
}

fn safe_clear_background(color: Color) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| clear_background(color)));
}

fn safe_draw_texture(tex: Option<&Texture2D>, x: f32, y: f32, params: DrawTextureParams) {
    if let Some(t) = tex {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            draw_texture_ex(t, x, y, WHITE, params)
        }));
    }
}

fn safe_set_camera(tx: f32, ty: f32, zoom: f32, sw: f32, sh: f32) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        set_camera(&Camera2D {
            offset: vec2(0.0, 0.0),
            target: vec2(tx, ty),
            rotation: 0.0,
            zoom: vec2((2.0 / sw) * zoom, (2.0 / sh) * zoom),
            render_target: None,
            viewport: None,
        });
    }));
}

fn safe_reset_camera() {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(set_default_camera));
}

// --- Helper Conversion Functions ---

fn to_f32(val: &Value) -> Result<f32, VmFault> {
    match val {
        Value::Float(f) => Ok(*f as f32),
        Value::Int(n) => Ok(*n as f32),
        other => Err(VmFault::TypeMismatch {
            expected: "Float or Int".to_string(),
            actual: other.type_name().to_string(),
        }),
    }
}

fn to_i64(val: &Value) -> Result<i64, VmFault> {
    match val {
        Value::Int(n) => Ok(*n),
        Value::Float(f) => Ok(*f as i64),
        other => Err(VmFault::TypeMismatch {
            expected: "Int".to_string(),
            actual: other.type_name().to_string(),
        }),
    }
}

fn to_bool(val: &Value) -> Result<bool, VmFault> {
    match val {
        Value::Bool(b) => Ok(*b),
        other => Err(VmFault::TypeMismatch {
            expected: "Bool".to_string(),
            actual: other.type_name().to_string(),
        }),
    }
}

fn to_string(val: &Value) -> Result<String, VmFault> {
    match val {
        Value::String(s) => Ok((**s).clone()),
        other => Ok(other.to_string()),
    }
}

/// Maps integer keycodes to Macroquad's KeyCode.
fn map_keycode(code: i64) -> Option<KeyCode> {
    match code {
        32 => Some(KeyCode::Space),
        256 => Some(KeyCode::Escape),
        257 => Some(KeyCode::Enter),
        258 => Some(KeyCode::Tab),
        259 => Some(KeyCode::Backspace),
        260 => Some(KeyCode::Insert),
        261 => Some(KeyCode::Delete),
        262 => Some(KeyCode::Right),
        263 => Some(KeyCode::Left),
        264 => Some(KeyCode::Down),
        265 => Some(KeyCode::Up),
        // ASCII A-Z (65..=90)
        65 => Some(KeyCode::A),
        66 => Some(KeyCode::B),
        67 => Some(KeyCode::C),
        68 => Some(KeyCode::D),
        69 => Some(KeyCode::E),
        70 => Some(KeyCode::F),
        71 => Some(KeyCode::G),
        72 => Some(KeyCode::H),
        73 => Some(KeyCode::I),
        74 => Some(KeyCode::J),
        75 => Some(KeyCode::K),
        76 => Some(KeyCode::L),
        77 => Some(KeyCode::M),
        78 => Some(KeyCode::N),
        79 => Some(KeyCode::O),
        80 => Some(KeyCode::P),
        81 => Some(KeyCode::Q),
        82 => Some(KeyCode::R),
        83 => Some(KeyCode::S),
        84 => Some(KeyCode::T),
        85 => Some(KeyCode::U),
        86 => Some(KeyCode::V),
        87 => Some(KeyCode::W),
        88 => Some(KeyCode::X),
        89 => Some(KeyCode::Y),
        90 => Some(KeyCode::Z),
        // Numbers 0-9 (48..=57)
        48 => Some(KeyCode::Key0),
        49 => Some(KeyCode::Key1),
        50 => Some(KeyCode::Key2),
        51 => Some(KeyCode::Key3),
        52 => Some(KeyCode::Key4),
        53 => Some(KeyCode::Key5),
        54 => Some(KeyCode::Key6),
        55 => Some(KeyCode::Key7),
        56 => Some(KeyCode::Key8),
        57 => Some(KeyCode::Key9),
        _ => None,
    }
}

// --- Host Native Functions ---

/// host_init(width: Int, height: Int, title: String, target_fps: Int) -> Bool
pub fn host_init(_args: &[Value]) -> Result<Value, VmFault> {
    Ok(Value::Bool(true))
}

/// host_should_close() -> Bool
pub fn host_should_close(_args: &[Value]) -> Result<Value, VmFault> {
    Ok(Value::Bool(safe_is_key_pressed(KeyCode::Escape)))
}

/// host_close() -> None
pub fn host_close(_args: &[Value]) -> Result<Value, VmFault> {
    Ok(Value::None)
}

/// host_key_down(key_code: Int) -> Bool
pub fn host_key_down(args: &[Value]) -> Result<Value, VmFault> {
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    let code = to_i64(&args[0])?;
    let pressed = map_keycode(code).is_some_and(safe_is_key_down);
    Ok(Value::Bool(pressed))
}

/// host_key_pressed(key_code: Int) -> Bool
pub fn host_key_pressed(args: &[Value]) -> Result<Value, VmFault> {
    if args.is_empty() {
        return Ok(Value::Bool(false));
    }
    let code = to_i64(&args[0])?;
    let pressed = map_keycode(code).is_some_and(safe_is_key_pressed);
    Ok(Value::Bool(pressed))
}

/// host_mouse_x() -> Float
pub fn host_mouse_x(_args: &[Value]) -> Result<Value, VmFault> {
    let (mx, _) = safe_mouse_position();
    Ok(Value::Float(mx as f64))
}

/// host_mouse_y() -> Float
pub fn host_mouse_y(_args: &[Value]) -> Result<Value, VmFault> {
    let (_, my) = safe_mouse_position();
    Ok(Value::Float(my as f64))
}

/// host_mouse_btn(button: Int) -> Bool (0 = Left, 1 = Right, 2 = Middle)
pub fn host_mouse_btn(args: &[Value]) -> Result<Value, VmFault> {
    let btn_idx = if args.is_empty() {
        0
    } else {
        to_i64(&args[0])?
    };
    let mb = match btn_idx {
        1 => MouseButton::Right,
        2 => MouseButton::Middle,
        _ => MouseButton::Left,
    };
    Ok(Value::Bool(safe_is_mouse_button_down(mb)))
}

/// host_screen_width() -> Float
pub fn host_screen_width(_args: &[Value]) -> Result<Value, VmFault> {
    Ok(Value::Float(safe_screen_width() as f64))
}

/// host_screen_height() -> Float
pub fn host_screen_height(_args: &[Value]) -> Result<Value, VmFault> {
    Ok(Value::Float(safe_screen_height() as f64))
}

/// host_frame_time() -> Float
pub fn host_frame_time(_args: &[Value]) -> Result<Value, VmFault> {
    Ok(Value::Float(safe_frame_time() as f64))
}

/// host_clear_background(r: Float, g: Float, b: Float)
pub fn host_clear_background(args: &[Value]) -> Result<Value, VmFault> {
    let r = if !args.is_empty() {
        to_f32(&args[0])?
    } else {
        0.0
    };
    let g = if args.len() > 1 {
        to_f32(&args[1])?
    } else {
        0.0
    };
    let b = if args.len() > 2 {
        to_f32(&args[2])?
    } else {
        0.0
    };
    safe_clear_background(Color::new(r, g, b, 1.0));
    Ok(Value::None)
}

/// host_begin_frame(r: Float, g: Float, b: Float)
pub fn host_begin_frame(args: &[Value]) -> Result<Value, VmFault> {
    host_clear_background(args)
}

/// host_end_frame()
pub fn host_end_frame(_args: &[Value]) -> Result<Value, VmFault> {
    Ok(Value::None)
}

/// host_draw_rect(x, y, w, h, r, g, b, a)
pub fn host_draw_rect(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 8 {
        return Err(VmFault::TypeMismatch {
            expected: "8 arguments (x, y, w, h, r, g, b, a)".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }
    let x = to_f32(&args[0])?;
    let y = to_f32(&args[1])?;
    let w = to_f32(&args[2])?;
    let h = to_f32(&args[3])?;
    let r = to_f32(&args[4])?;
    let g = to_f32(&args[5])?;
    let b = to_f32(&args[6])?;
    let a = to_f32(&args[7])?;
    safe_draw_rect(x, y, w, h, Color::new(r, g, b, a));
    Ok(Value::None)
}

/// host_draw_rect_lines(x, y, w, h, thickness, r, g, b, a)
pub fn host_draw_rect_lines(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 9 {
        return Err(VmFault::TypeMismatch {
            expected: "9 arguments (x, y, w, h, thickness, r, g, b, a)".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }
    let x = to_f32(&args[0])?;
    let y = to_f32(&args[1])?;
    let w = to_f32(&args[2])?;
    let h = to_f32(&args[3])?;
    let th = to_f32(&args[4])?;
    let r = to_f32(&args[5])?;
    let g = to_f32(&args[6])?;
    let b = to_f32(&args[7])?;
    let a = to_f32(&args[8])?;
    safe_draw_rect_lines(x, y, w, h, th, Color::new(r, g, b, a));
    Ok(Value::None)
}

/// host_draw_circle(cx, cy, radius, r, g, b, a)
pub fn host_draw_circle(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 7 {
        return Err(VmFault::TypeMismatch {
            expected: "7 arguments (cx, cy, radius, r, g, b, a)".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }
    let cx = to_f32(&args[0])?;
    let cy = to_f32(&args[1])?;
    let radius = to_f32(&args[2])?;
    let r = to_f32(&args[3])?;
    let g = to_f32(&args[4])?;
    let b = to_f32(&args[5])?;
    let a = to_f32(&args[6])?;
    safe_draw_circle(cx, cy, radius, Color::new(r, g, b, a));
    Ok(Value::None)
}

/// host_draw_text(text, x, y, font_size, r, g, b)
pub fn host_draw_text(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 7 {
        return Err(VmFault::TypeMismatch {
            expected: "7 arguments (text, x, y, font_size, r, g, b)".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }
    let text = to_string(&args[0])?;
    let x = to_f32(&args[1])?;
    let y = to_f32(&args[2])?;
    let size = to_f32(&args[3])?;
    let r = to_f32(&args[4])?;
    let g = to_f32(&args[5])?;
    let b = to_f32(&args[6])?;
    safe_draw_text(&text, x, y, size, Color::new(r, g, b, 1.0));
    Ok(Value::None)
}

/// host_load_texture(path: String) -> Int
pub fn host_load_texture(args: &[Value]) -> Result<Value, VmFault> {
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    let path = to_string(&args[0])?;
    let mut cache = TEXTURE_CACHE
        .lock()
        .map_err(|_| VmFault::CapabilityDenied {
            capability: "host.texture".to_string(),
            operation: "texture_cache_lock".to_string(),
        })?;
    let id = cache.load(&path);
    Ok(Value::Int(id))
}

/// host_draw_sprite(tex_id, x, y, w, h, rot, flip_x)
pub fn host_draw_sprite(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 7 {
        return Err(VmFault::TypeMismatch {
            expected: "7 arguments (tex_id, x, y, w, h, rot, flip_x)".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }
    let id = to_i64(&args[0])?;
    let x = to_f32(&args[1])?;
    let y = to_f32(&args[2])?;
    let w = to_f32(&args[3])?;
    let h = to_f32(&args[4])?;
    let rot = to_f32(&args[5])?;
    let flip_x = to_bool(&args[6])?;

    let mut cache = TEXTURE_CACHE
        .lock()
        .map_err(|_| VmFault::CapabilityDenied {
            capability: "host.texture".to_string(),
            operation: "texture_cache_lock".to_string(),
        })?;
    let tex = cache.get(id);

    safe_draw_texture(
        tex.as_ref(),
        x,
        y,
        DrawTextureParams {
            dest_size: Some(vec2(w, h)),
            rotation: rot,
            flip_x,
            ..Default::default()
        },
    );
    Ok(Value::None)
}

/// host_draw_sprite_subrect(tex_id, sx, sy, sw, sh, dx, dy, dw, dh, flip_x)
pub fn host_draw_sprite_subrect(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 10 {
        return Err(VmFault::TypeMismatch {
            expected: "10 arguments (tex_id, sx, sy, sw, sh, dx, dy, dw, dh, flip_x)".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }
    let id = to_i64(&args[0])?;
    let sx = to_f32(&args[1])?;
    let sy = to_f32(&args[2])?;
    let sw = to_f32(&args[3])?;
    let sh = to_f32(&args[4])?;
    let dx = to_f32(&args[5])?;
    let dy = to_f32(&args[6])?;
    let dw = to_f32(&args[7])?;
    let dh = to_f32(&args[8])?;
    let flip_x = to_bool(&args[9])?;

    let mut cache = TEXTURE_CACHE
        .lock()
        .map_err(|_| VmFault::CapabilityDenied {
            capability: "host.texture".to_string(),
            operation: "texture_cache_lock".to_string(),
        })?;
    let tex = cache.get(id);

    safe_draw_texture(
        tex.as_ref(),
        dx,
        dy,
        DrawTextureParams {
            source: Some(Rect::new(sx, sy, sw, sh)),
            dest_size: Some(vec2(dw, dh)),
            flip_x,
            ..Default::default()
        },
    );
    Ok(Value::None)
}

/// host_set_camera(target_x: Float, target_y: Float, zoom: Float)
pub fn host_set_camera(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 3 {
        return Err(VmFault::TypeMismatch {
            expected: "3 arguments (target_x, target_y, zoom)".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }
    let tx = to_f32(&args[0])?;
    let ty = to_f32(&args[1])?;
    let z = to_f32(&args[2])?.max(0.01);

    let sw = safe_screen_width();
    let sh = safe_screen_height();

    safe_set_camera(tx, ty, z, sw, sh);
    Ok(Value::None)
}

/// host_reset_camera()
pub fn host_reset_camera(_args: &[Value]) -> Result<Value, VmFault> {
    safe_reset_camera();
    Ok(Value::None)
}

/// Type signature of a host native function callback.
type NativeFn = fn(&[Value]) -> Result<Value, VmFault>;

/// Entry representing a registered host native function (name, arity, callback).
type NativeEntry = (&'static str, usize, NativeFn);

/// Table of all host functions with name, arity, and native function pointer.
const NATIVES: &[NativeEntry] = &[
    ("host_init", 4, host_init),
    ("host_should_close", 0, host_should_close),
    ("host_close", 0, host_close),
    ("host_key_down", 1, host_key_down),
    ("host_key_pressed", 1, host_key_pressed),
    ("host_mouse_x", 0, host_mouse_x),
    ("host_mouse_y", 0, host_mouse_y),
    ("host_mouse_btn", 1, host_mouse_btn),
    ("host_screen_width", 0, host_screen_width),
    ("host_screen_height", 0, host_screen_height),
    ("host_frame_time", 0, host_frame_time),
    ("host_clear_background", 3, host_clear_background),
    ("host_begin_frame", 3, host_begin_frame),
    ("host_end_frame", 0, host_end_frame),
    ("host_draw_rect", 8, host_draw_rect),
    ("host_draw_rect_lines", 9, host_draw_rect_lines),
    ("host_draw_circle", 7, host_draw_circle),
    ("host_draw_text", 7, host_draw_text),
    ("host_load_texture", 1, host_load_texture),
    ("host_draw_sprite", 7, host_draw_sprite),
    ("host_draw_sprite_subrect", 10, host_draw_sprite_subrect),
    ("host_set_camera", 3, host_set_camera),
    ("host_reset_camera", 0, host_reset_camera),
];

/// Registers the game host native functions into the semantic analyzer's Prelude surface.
pub fn register_surface_symbols(surface: &mut PreludeSurface) {
    for (name, arity, _) in NATIVES {
        surface.add_function(name, *arity, *arity);
        // Also register __aipo_game_* canonical alias
        let canonical_alias = format!("__aipo_game_{}", name.trim_start_matches("host_"));
        surface.add_function(&canonical_alias, *arity, *arity);
    }
    surface.add_variable("game");
}

/// Registers the game host native functions into the Aipo VM instance.
pub fn register_vm_natives(vm: &mut Vm) {
    let mut game_dict_entries = Vec::new();

    for (name, arity, func) in NATIVES {
        let native_val = Value::native(*name, *arity, *func);
        // Define as top-level global
        vm.define_global(*name, native_val.clone());

        // Define canonical __aipo_game_* alias
        let canonical_alias = format!("__aipo_game_{}", name.trim_start_matches("host_"));
        vm.define_global(canonical_alias, native_val.clone());

        // Add to game module dict
        let clean_key = name.trim_start_matches("host_");
        game_dict_entries.push((Value::String(Rc::new(clean_key.to_string())), native_val));
    }

    let dict_map = aipo_vm::DictMap::from_entries(game_dict_entries);
    vm.define_global("game", Value::Dict(Rc::new(RefCell::new(dict_map))));
}
