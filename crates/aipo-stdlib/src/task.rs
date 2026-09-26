//! Canonical `task` standard library module.
//!
//! Exposes the scheduler-intercepted primitives (`spawn`, `sleep`, `all`,
//! `race`, `timeout`, `cancel`, `group`) into the `task` module dictionary.

use aipo_vm::{DictMap, Value, VmFault};
use std::cell::RefCell;
use std::rc::Rc;

fn dummy_task_fn(_args: &[Value]) -> Result<Value, VmFault> {
    // Intercepted by Vm::begin_call before reaching this callback
    Ok(Value::None)
}

/// Constructs the canonical `task` module dictionary.
#[must_use]
pub fn create_module() -> Value {
    let entries = vec![
        (
            Value::String(Rc::new("spawn".to_string())),
            Value::native("task.spawn", 2, dummy_task_fn),
        ),
        (
            Value::String(Rc::new("sleep".to_string())),
            Value::native("task.sleep", 1, dummy_task_fn),
        ),
        (
            Value::String(Rc::new("all".to_string())),
            Value::native("task.all", 1, dummy_task_fn),
        ),
        (
            Value::String(Rc::new("race".to_string())),
            Value::native("task.race", 1, dummy_task_fn),
        ),
        (
            Value::String(Rc::new("timeout".to_string())),
            Value::native("task.timeout", 2, dummy_task_fn),
        ),
        (
            Value::String(Rc::new("cancel".to_string())),
            Value::native("task.cancel", 1, dummy_task_fn),
        ),
        (
            Value::String(Rc::new("group".to_string())),
            Value::native("task.group", 0, dummy_task_fn),
        ),
    ];

    Value::Dict(Rc::new(RefCell::new(DictMap::from_entries(entries))))
}
