//! Host FFI bridge connecting Aipo VM bytecode execution with Macroquad/Miniquad GPU rendering.
//!
//! Provides hardware-accelerated 2D draw calls, real-time keyboard/mouse input,
//! texture and spritesheet pipeline, and camera transformations with headless fallback.

#![forbid(unsafe_code)]

use crate::audio_system::{AUDIO, SynthConfig};
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

#[derive(Debug, Clone, Copy)]
pub struct ClipRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

static CLIP_STACK: Mutex<Vec<ClipRect>> = Mutex::new(Vec::new());

fn current_clip() -> Option<ClipRect> {
    CLIP_STACK
        .lock()
        .ok()
        .and_then(|stack| stack.last().copied())
}

fn safe_mouse_wheel() -> (f32, f32) {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(mouse_wheel)).unwrap_or((0.0, 0.0))
}

fn safe_is_key_pressed(key: KeyCode) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| is_key_pressed(key))).unwrap_or(false)
}

fn safe_get_char_pressed() -> Option<char> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(get_char_pressed)).unwrap_or(None)
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
    let (draw_x, draw_y, draw_w, draw_h) = if let Some(clip) = current_clip() {
        let x1 = x.max(clip.x);
        let y1 = y.max(clip.y);
        let x2 = (x + w).min(clip.x + clip.w);
        let y2 = (y + h).min(clip.y + clip.h);
        if x2 <= x1 || y2 <= y1 {
            return;
        }
        (x1, y1, x2 - x1, y2 - y1)
    } else {
        (x, y, w, h)
    };
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        draw_rectangle(draw_x, draw_y, draw_w, draw_h, color)
    }));
}

fn safe_draw_rect_lines(x: f32, y: f32, w: f32, h: f32, th: f32, color: Color) {
    if let Some(clip) = current_clip() {
        let x1 = x.max(clip.x);
        let y1 = y.max(clip.y);
        let x2 = (x + w).min(clip.x + clip.w);
        let y2 = (y + h).min(clip.y + clip.h);
        if x2 <= x1 || y2 <= y1 {
            return;
        }
    }
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
    if let Some(clip) = current_clip() {
        if y < clip.y || (y - size) > (clip.y + clip.h) || x > (clip.x + clip.w) {
            return;
        }
    }
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

fn safe_macroquad_call<F: FnOnce() + std::panic::UnwindSafe>(f: F) {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let _ = std::panic::catch_unwind(f);
    std::panic::set_hook(prev_hook);
}

fn safe_set_camera(tx: f32, ty: f32, zoom: f32, sw: f32, sh: f32) {
    safe_macroquad_call(|| {
        set_camera(&Camera2D {
            offset: vec2(0.0, 0.0),
            target: vec2(tx, ty),
            rotation: 0.0,
            zoom: vec2((2.0 / sw) * zoom, (2.0 / sh) * zoom),
            render_target: None,
            viewport: None,
        });
    });
}

fn safe_set_camera_viewport(
    tx: f32,
    ty: f32,
    zoom: f32,
    gl_x: i32,
    gl_y: i32,
    gl_w: i32,
    gl_h: i32,
) {
    let vw = (gl_w as f32).max(1.0);
    let vh = (gl_h as f32).max(1.0);
    safe_macroquad_call(|| {
        set_camera(&Camera2D {
            offset: vec2(0.0, 0.0),
            target: vec2(tx, ty),
            rotation: 0.0,
            zoom: vec2((2.0 / vw) * zoom, (2.0 / vh) * zoom),
            render_target: None,
            viewport: Some((gl_x, gl_y, gl_w, gl_h)),
        });
    });
}

fn safe_reset_camera() {
    safe_macroquad_call(set_default_camera);
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

/// host_get_char_pressed() -> String
pub fn host_get_char_pressed(_args: &[Value]) -> Result<Value, VmFault> {
    if let Some(ch) = safe_get_char_pressed() {
        Ok(Value::String(std::rc::Rc::new(ch.to_string())))
    } else {
        Ok(Value::String(std::rc::Rc::new(String::new())))
    }
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

/// host_mouse_wheel_x() -> Float
pub fn host_mouse_wheel_x(_args: &[Value]) -> Result<Value, VmFault> {
    let (wx, _) = safe_mouse_wheel();
    Ok(Value::Float(wx as f64))
}

/// host_mouse_wheel_y() -> Float
pub fn host_mouse_wheel_y(_args: &[Value]) -> Result<Value, VmFault> {
    let (_, wy) = safe_mouse_wheel();
    Ok(Value::Float(wy as f64))
}

/// host_push_clip_rect(x, y, w, h)
pub fn host_push_clip_rect(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 4 {
        return Err(VmFault::TypeMismatch {
            expected: "4 arguments (x, y, w, h)".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }
    let x = to_f32(&args[0])?;
    let y = to_f32(&args[1])?;
    let w = to_f32(&args[2])?;
    let h = to_f32(&args[3])?;

    if let Ok(mut stack) = CLIP_STACK.lock() {
        let new_clip = if let Some(parent) = stack.last() {
            let x1 = x.max(parent.x);
            let y1 = y.max(parent.y);
            let x2 = (x + w).min(parent.x + parent.w);
            let y2 = (y + h).min(parent.y + parent.h);
            ClipRect {
                x: x1,
                y: y1,
                w: (x2 - x1).max(0.0),
                h: (y2 - y1).max(0.0),
            }
        } else {
            ClipRect { x, y, w, h }
        };
        stack.push(new_clip);
    }
    Ok(Value::None)
}

/// host_pop_clip_rect()
pub fn host_pop_clip_rect(_args: &[Value]) -> Result<Value, VmFault> {
    if let Ok(mut stack) = CLIP_STACK.lock() {
        stack.pop();
    }
    Ok(Value::None)
}

/// host_begin_frame(r: Float, g: Float, b: Float)
pub fn host_begin_frame(args: &[Value]) -> Result<Value, VmFault> {
    if let Ok(mut stack) = CLIP_STACK.lock() {
        stack.clear();
    }
    host_clear_background(args)
}

/// host_end_frame()
pub fn host_end_frame(_args: &[Value]) -> Result<Value, VmFault> {
    if let Ok(mut stack) = CLIP_STACK.lock() {
        stack.clear();
    }
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

/// host_draw_line(x1, y1, x2, y2, thickness, r, g, b, a)
pub fn host_draw_line(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 9 {
        return Err(VmFault::TypeMismatch {
            expected: "9 arguments (x1, y1, x2, y2, thickness, r, g, b, a)".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }
    let x1 = to_f32(&args[0])?;
    let y1 = to_f32(&args[1])?;
    let x2 = to_f32(&args[2])?;
    let y2 = to_f32(&args[3])?;
    let th = to_f32(&args[4])?;
    let r = to_f32(&args[5])?;
    let g = to_f32(&args[6])?;
    let b = to_f32(&args[7])?;
    let a = to_f32(&args[8])?;
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        draw_line(x1, y1, x2, y2, th, Color::new(r, g, b, a));
    }));
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

/// host_set_viewport_camera(vx, vy, vw, vh, target_x, target_y, zoom)
pub fn host_set_viewport_camera(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 7 {
        return Err(VmFault::TypeMismatch {
            expected: "7 arguments (vx, vy, vw, vh, target_x, target_y, zoom)".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }
    let vx = to_f32(&args[0])?;
    let vy = to_f32(&args[1])?;
    let vw = to_f32(&args[2])?.max(1.0);
    let vh = to_f32(&args[3])?.max(1.0);
    let tx = to_f32(&args[4])?;
    let ty = to_f32(&args[5])?;
    let z = to_f32(&args[6])?.max(0.001);

    let sh = safe_screen_height();
    let gl_x = vx as i32;
    let gl_y = (sh - (vy + vh)).max(0.0) as i32;
    let gl_w = vw as i32;
    let gl_h = vh as i32;

    safe_set_camera_viewport(tx, ty, z, gl_x, gl_y, gl_w, gl_h);
    Ok(Value::None)
}

/// host_load_sound(path: String) -> Int
pub fn host_load_sound(args: &[Value]) -> Result<Value, VmFault> {
    if args.is_empty() {
        return Ok(Value::Int(0));
    }
    let path = to_string(&args[0])?;
    let mut audio = AUDIO.lock().map_err(|_| VmFault::CapabilityDenied {
        capability: "host.audio".to_string(),
        operation: "audio_lock".to_string(),
    })?;
    let id = audio.load_file(&path);
    Ok(Value::Int(id))
}

/// host_play_sound(sound_id: Int, volume: Float, pitch: Float) -> None
pub fn host_play_sound(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 3 {
        return Err(VmFault::TypeMismatch {
            expected: "3 arguments (sound_id, volume, pitch)".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }
    let id = to_i64(&args[0])?;
    let volume = to_f32(&args[1])?;
    let _pitch = to_f32(&args[2])?;
    let audio = AUDIO.lock().map_err(|_| VmFault::CapabilityDenied {
        capability: "host.audio".to_string(),
        operation: "audio_lock".to_string(),
    })?;
    audio.play(id, volume);
    Ok(Value::None)
}

/// host_play_preset(name: String, volume: Float, pitch: Float) -> None
pub fn host_play_preset(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 3 {
        return Err(VmFault::TypeMismatch {
            expected: "3 arguments (name, volume, pitch)".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }
    let name = to_string(&args[0])?;
    let volume = to_f32(&args[1])?;
    let pitch = to_f32(&args[2])?;
    let mut audio = AUDIO.lock().map_err(|_| VmFault::CapabilityDenied {
        capability: "host.audio".to_string(),
        operation: "audio_lock".to_string(),
    })?;
    audio.play_preset(&name, volume, pitch);
    Ok(Value::None)
}

/// host_synth_sound(wave_type: String, start_freq: Float, freq_slide: Float, duration: Float, volume: Float) -> Int
pub fn host_synth_sound(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 5 {
        return Err(VmFault::TypeMismatch {
            expected: "5 arguments (wave_type, start_freq, freq_slide, duration, volume)"
                .to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }
    let wave_type = to_string(&args[0])?;
    let start_freq = to_f32(&args[1])?;
    let freq_slide = to_f32(&args[2])?;
    let duration = to_f32(&args[3])?;
    let volume = to_f32(&args[4])?;

    let config = SynthConfig {
        wave_type,
        start_freq,
        freq_slide,
        duration,
        duty_cycle: 0.5,
        volume,
    };

    let mut audio = AUDIO.lock().map_err(|_| VmFault::CapabilityDenied {
        capability: "host.audio".to_string(),
        operation: "audio_lock".to_string(),
    })?;
    let id = audio.synth_sound(&config);
    Ok(Value::Int(id))
}

/// host_stop_sound(sound_id: Int) -> None
pub fn host_stop_sound(args: &[Value]) -> Result<Value, VmFault> {
    if args.is_empty() {
        return Ok(Value::None);
    }
    let id = to_i64(&args[0])?;
    let audio = AUDIO.lock().map_err(|_| VmFault::CapabilityDenied {
        capability: "host.audio".to_string(),
        operation: "audio_lock".to_string(),
    })?;
    audio.stop(id);
    Ok(Value::None)
}

/// host_play_music(sound_id: Int, volume: Float, is_loop: Bool) -> None
pub fn host_play_music(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 3 {
        return Err(VmFault::TypeMismatch {
            expected: "3 arguments (sound_id, volume, is_loop)".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }
    let id = to_i64(&args[0])?;
    let volume = to_f32(&args[1])?;
    let is_loop = to_bool(&args[2])?;
    let mut audio = AUDIO.lock().map_err(|_| VmFault::CapabilityDenied {
        capability: "host.audio".to_string(),
        operation: "audio_lock".to_string(),
    })?;
    audio.play_music(id, volume, is_loop);
    Ok(Value::None)
}

/// host_stop_music() -> None
pub fn host_stop_music(_args: &[Value]) -> Result<Value, VmFault> {
    let mut audio = AUDIO.lock().map_err(|_| VmFault::CapabilityDenied {
        capability: "host.audio".to_string(),
        operation: "audio_lock".to_string(),
    })?;
    audio.stop_music();
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
    ("host_get_char_pressed", 0, host_get_char_pressed),
    ("host_mouse_x", 0, host_mouse_x),
    ("host_mouse_y", 0, host_mouse_y),
    ("host_mouse_btn", 1, host_mouse_btn),
    ("host_mouse_wheel_x", 0, host_mouse_wheel_x),
    ("host_mouse_wheel_y", 0, host_mouse_wheel_y),
    ("host_push_clip_rect", 4, host_push_clip_rect),
    ("host_pop_clip_rect", 0, host_pop_clip_rect),
    ("host_screen_width", 0, host_screen_width),
    ("host_screen_height", 0, host_screen_height),
    ("host_frame_time", 0, host_frame_time),
    ("host_clear_background", 3, host_clear_background),
    ("host_begin_frame", 3, host_begin_frame),
    ("host_end_frame", 0, host_end_frame),
    ("host_draw_rect", 8, host_draw_rect),
    ("host_draw_rect_lines", 9, host_draw_rect_lines),
    ("host_draw_circle", 7, host_draw_circle),
    ("host_draw_line", 9, host_draw_line),
    ("host_draw_text", 7, host_draw_text),
    ("host_load_texture", 1, host_load_texture),
    ("host_draw_sprite", 7, host_draw_sprite),
    ("host_draw_sprite_subrect", 10, host_draw_sprite_subrect),
    ("host_set_camera", 3, host_set_camera),
    ("host_reset_camera", 0, host_reset_camera),
    ("host_set_viewport_camera", 7, host_set_viewport_camera),
    ("host_load_sound", 1, host_load_sound),
    ("host_play_sound", 3, host_play_sound),
    ("host_play_preset", 3, host_play_preset),
    ("host_synth_sound", 5, host_synth_sound),
    ("host_stop_sound", 1, host_stop_sound),
    ("host_play_music", 3, host_play_music),
    ("host_stop_music", 0, host_stop_music),
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
    surface.add_variable("egui");
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

    // Install and register agnostic egui host profile
    aipo_egui::install_egui(aipo_egui::EguiService::new());
    aipo_egui::register_egui(vm);
}

fn render_epaint_shape(shape: &egui::epaint::Shape) {
    match shape {
        egui::epaint::Shape::Vec(children) => {
            for child in children {
                render_epaint_shape(child);
            }
        }
        egui::epaint::Shape::Rect(rect_shape) => {
            let r = rect_shape.rect;
            let c = rect_shape.fill;
            if c.a() > 0 {
                let col = Color::new(
                    f32::from(c.r()) / 255.0,
                    f32::from(c.g()) / 255.0,
                    f32::from(c.b()) / 255.0,
                    f32::from(c.a()) / 255.0,
                );
                safe_draw_rect(r.min.x, r.min.y, r.width(), r.height(), col);
            }
            if rect_shape.stroke.width > 0.0 && rect_shape.stroke.color.a() > 0 {
                let sc = rect_shape.stroke.color;
                let stroke_col = Color::new(
                    f32::from(sc.r()) / 255.0,
                    f32::from(sc.g()) / 255.0,
                    f32::from(sc.b()) / 255.0,
                    f32::from(sc.a()) / 255.0,
                );
                safe_draw_rect_lines(
                    r.min.x,
                    r.min.y,
                    r.width(),
                    r.height(),
                    rect_shape.stroke.width,
                    stroke_col,
                );
            }
        }
        egui::epaint::Shape::Text(text_shape) => {
            let pos = text_shape.pos;
            let text = text_shape.galley.text();
            let mut c = text_shape.fallback_color;
            if c.a() == 0 {
                c = egui::Color32::from_rgb(220, 225, 235);
            }
            let font_size = (text_shape.galley.size().y * 0.95).clamp(14.0, 36.0);
            let text_col = Color::new(
                f32::from(c.r()) / 255.0,
                f32::from(c.g()) / 255.0,
                f32::from(c.b()) / 255.0,
                f32::from(c.a()) / 255.0,
            );
            safe_draw_text(text, pos.x, pos.y + font_size * 0.85, font_size, text_col);
        }
        egui::epaint::Shape::LineSegment { points, stroke } => {
            let c = stroke.color;
            let col = Color::new(
                f32::from(c.r()) / 255.0,
                f32::from(c.g()) / 255.0,
                f32::from(c.b()) / 255.0,
                f32::from(c.a()) / 255.0,
            );
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                draw_line(
                    points[0].x,
                    points[0].y,
                    points[1].x,
                    points[1].y,
                    stroke.width,
                    col,
                );
            }));
        }
        egui::epaint::Shape::Path(path) => {
            if path.points.len() >= 2 {
                let c = match &path.stroke.color {
                    egui::epaint::ColorMode::Solid(c) => *c,
                    egui::epaint::ColorMode::UV(_) => egui::Color32::WHITE,
                };
                let col = Color::new(
                    f32::from(c.r()) / 255.0,
                    f32::from(c.g()) / 255.0,
                    f32::from(c.b()) / 255.0,
                    f32::from(c.a()) / 255.0,
                );
                for i in 0..(path.points.len() - 1) {
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        draw_line(
                            path.points[i].x,
                            path.points[i].y,
                            path.points[i + 1].x,
                            path.points[i + 1].y,
                            path.stroke.width,
                            col,
                        );
                    }));
                }
                if path.closed && path.points.len() > 2 {
                    let last = path.points.len() - 1;
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        draw_line(
                            path.points[last].x,
                            path.points[last].y,
                            path.points[0].x,
                            path.points[0].y,
                            path.stroke.width,
                            col,
                        );
                    }));
                }
            }
        }
        egui::epaint::Shape::Circle(circle) => {
            let c = circle.fill;
            let col = Color::new(
                f32::from(c.r()) / 255.0,
                f32::from(c.g()) / 255.0,
                f32::from(c.b()) / 255.0,
                f32::from(c.a()) / 255.0,
            );
            safe_draw_circle(circle.center.x, circle.center.y, circle.radius, col);
        }
        _ => {}
    }
}

/// Renders the shapes generated by the active egui session on top of the screen.
pub fn render_egui_overlay() {
    let _ = aipo_egui::with_egui("egui", "render_egui_overlay", |service| {
        let Some(handle) = service.last_active_handle.or(service.active_handle) else {
            return Ok(());
        };
        let Some(session) = service.sessions.get(handle) else {
            return Ok(());
        };
        let Some(ref output) = session.last_output else {
            return Ok(());
        };

        for clipped in &output.shapes {
            render_epaint_shape(&clipped.shape);
        }
        Ok(())
    });
}
