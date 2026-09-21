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
            Value::Native {
                name: "task.spawn".to_string(),
                arity: 2,
                func: dummy_task_fn,
            },
        ),
        (
            Value::String(Rc::new("sleep".to_string())),
            Value::Native {
                name: "task.sleep".to_string(),
                arity: 1,
                func: dummy_task_fn,
            },
        ),
        (
            Value::String(Rc::new("all".to_string())),
            Value::Native {
                name: "task.all".to_string(),
                arity: 1,
                func: dummy_task_fn,
            },
        ),
        (
            Value::String(Rc::new("race".to_string())),
            Value::Native {
                name: "task.race".to_string(),
                arity: 1,
                func: dummy_task_fn,
            },
        ),
        (
            Value::String(Rc::new("timeout".to_string())),
            Value::Native {
                name: "task.timeout".to_string(),
                arity: 2,
                func: dummy_task_fn,
            },
        ),
        (
            Value::String(Rc::new("cancel".to_string())),
            Value::Native {
                name: "task.cancel".to_string(),
                arity: 1,
                func: dummy_task_fn,
            },
        ),
        (
            Value::String(Rc::new("group".to_string())),
            Value::Native {
                name: "task.group".to_string(),
                arity: 0,
                func: dummy_task_fn,
            },
        ),
    ];

    Value::Dict(Rc::new(RefCell::new(DictMap::from_entries(entries))))
}
