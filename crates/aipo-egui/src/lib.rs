//! `aipo-egui` — Agnostic immediate-mode GUI host adapter and egui bindings for Aipo.
//!
//! Provides engine-agnostic bindings for `egui`:
//! - Pure immediate-mode GUI lifecycle (`Context`, `begin_frame`, `end_frame`).
//! - Standard widgets (labels, headings, buttons, checkboxes, sliders, text edits, separators).
//! - Native containers and panels (`begin_window`, `end_window`).
//! - Generational handles and `deny-by-default` capabilities (`egui`).
//! - Geometric shape extraction for external renderers (GPU, Wasm Canvas, TUI, or Headless tests).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod adapter;
pub mod context;
pub mod schema;
pub mod shapes;

pub use adapter::{
    EGUI_LOCK, EguiService, create_module, egui_begin_frame, egui_begin_window, egui_button,
    egui_checkbox, egui_create_context, egui_destroy_context, egui_end_frame, egui_end_window,
    egui_heading, egui_label, egui_progress_bar, egui_separator, egui_slider, egui_text_edit,
    egui_wants_keyboard_input, egui_wants_pointer_input, install_egui, register_egui, revoke_egui,
    with_egui,
};
pub use context::{EguiSession, FrameInputOptions};
pub use schema::egui_schema;
pub use shapes::shape_to_value;
