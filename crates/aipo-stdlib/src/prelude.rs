//! Canonical Prelude V1 functions for Aipo.
//!
//! Provides fundamental globally available functions without requiring imports:
//! - `len`: determines collection or string length
//! - `copy`: shallow copy of collections and structs
//! - `same`: reference identity for references, equality for primitives
//! - `some`: presence check (is not `none`)
//! - `fail`: produces a recoverable Failure value

use aipo_vm::{StructInstance, Value, VmFault, check_safe_int};
use std::cell::RefCell;
use std::rc::Rc;

/// Computes the length of a String, List, or Dict.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if operand is not a measurable type,
/// or `VmFault::Overflow` if length exceeds safe integer range.
pub fn native_len(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: "1 argument".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    match &args[0] {
        Value::String(s) => {
            #[allow(clippy::cast_possible_wrap)]
            let count = s.chars().count() as i64;
            check_safe_int(count).map(Value::Int)
        }
        Value::List(l) => {
            #[allow(clippy::cast_possible_wrap)]
            let count = l.borrow().len() as i64;
            check_safe_int(count).map(Value::Int)
        }
        Value::Dict(d) => {
            #[allow(clippy::cast_possible_wrap)]
            let count = d.borrow().len() as i64;
            check_safe_int(count).map(Value::Int)
        }
        Value::Bytes(b) => {
            #[allow(clippy::cast_possible_wrap)]
            let count = b.borrow().len() as i64;
            check_safe_int(count).map(Value::Int)
        }
        Value::Set(s) => {
            #[allow(clippy::cast_possible_wrap)]
            let count = s.borrow().len() as i64;
            check_safe_int(count).map(Value::Int)
        }
        Value::Failure(f) => Ok(Value::Failure(Rc::clone(f))),
        other => Err(VmFault::TypeMismatch {
            expected: "String, List, Dict, Bytes, or Set".to_string(),
            actual: other.type_name().to_string(),
        }),
    }
}

/// Shallow copies a collection or struct.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if argument count is not 1.
pub fn native_copy(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: "1 argument".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    match &args[0] {
        Value::List(l) => {
            let cloned = l.borrow().clone();
            Ok(Value::List(Rc::new(RefCell::new(cloned))))
        }
        Value::Dict(d) => {
            let cloned = d.borrow().clone();
            Ok(Value::Dict(Rc::new(RefCell::new(cloned))))
        }
        Value::Bytes(b) => {
            let cloned = b.borrow().clone();
            Ok(Value::Bytes(Rc::new(RefCell::new(cloned))))
        }
        Value::Set(s) => {
            let cloned = s.borrow().clone();
            Ok(Value::Set(Rc::new(RefCell::new(cloned))))
        }
        Value::Struct(s) => {
            let inner = s.borrow();
            let new_inst = StructInstance {
                type_name: inner.type_name.clone(),
                fields: inner.fields.clone(),
                fixed_fields: inner.fixed_fields.clone(),
                under_construction: inner.under_construction,
            };
            Ok(Value::Struct(Rc::new(RefCell::new(new_inst))))
        }
        other => Ok(other.clone()),
    }
}

/// Checks identity comparison between two values.
///
/// Reference types (`List`, `Dict`, `Struct`, `Bytes`, `Set`) compare identity (`Rc::ptr_eq`).
/// Primitives compare by equality.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if argument count is not 2.
pub fn native_same(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 2 {
        return Err(VmFault::TypeMismatch {
            expected: "2 arguments".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    let a = &args[0];
    let b = &args[1];

    let same = match (a, b) {
        (Value::List(l1), Value::List(l2)) => Rc::ptr_eq(l1, l2),
        (Value::Dict(d1), Value::Dict(d2)) => Rc::ptr_eq(d1, d2),
        (Value::Bytes(b1), Value::Bytes(b2)) => Rc::ptr_eq(b1, b2),
        (Value::Set(s1), Value::Set(s2)) => Rc::ptr_eq(s1, s2),
        (Value::Struct(s1), Value::Struct(s2)) => Rc::ptr_eq(s1, s2),
        (Value::String(s1), Value::String(s2)) => Rc::ptr_eq(s1, s2) || s1 == s2,
        (v1, v2) => v1 == v2,
    };

    Ok(Value::Bool(same))
}

/// Checks whether a value is not `none`.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if argument count is not 1.
pub fn native_some(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: "1 argument".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    Ok(Value::Bool(!matches!(&args[0], Value::None)))
}

/// Creates a recoverable Failure value (Failure Model B).
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if argument count is not 1.
pub fn native_fail(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: "1 argument".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    let (message, payload) = match &args[0] {
        Value::String(s) => (s.as_str().to_string(), Value::None),
        Value::Failure(f) => (f.message.clone(), f.payload.clone()),
        other => (other.to_string(), other.clone()),
    };

    Ok(Value::failure_with_payload(message, payload))
}
