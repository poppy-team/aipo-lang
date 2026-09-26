//! Canonical `testing` and `expect` module for Aipo.
//!
//! Implements canonical assertion primitives:
//! - `expect.equal(actual, expected)`
//! - `expect.not_equal(actual, expected)`
//! - `expect.true(value)`
//! - `expect.false(value)`
//! - `expect.none(value)`
//! - `expect.some(value)`
//! - `expect.failure(value)`
//! - `expect.contains(collection, element)`
//! - `expect.approx(actual, expected, tolerance = 0.0001)`
//!
//! Assertions return `none` on success or a recoverable `Failure` (Model B) on mismatch.

#![forbid(unsafe_code)]

use aipo_vm::{DictMap, FailureValue, Value, VmFault};
use std::cell::RefCell;
use std::rc::Rc;

fn require_arity(args: &[Value], expected: usize, op: &str) -> Result<(), VmFault> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(VmFault::TypeMismatch {
            expected: format!("{expected} argument(s) for {op}"),
            actual: format!("{} arguments", args.len()),
        })
    }
}

/// `expect.equal(actual, expected)`
pub fn expect_equal(args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 2, "expect.equal")?;
    let act = &args[0];
    let exp = &args[1];
    if act == exp {
        Ok(Value::None)
    } else {
        Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "expect.equal failed: expected {exp}, got {act}"
        )))))
    }
}

/// `expect.not_equal(actual, expected)`
pub fn expect_not_equal(args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 2, "expect.not_equal")?;
    let act = &args[0];
    let exp = &args[1];
    if act != exp {
        Ok(Value::None)
    } else {
        Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "expect.not_equal failed: expected values to differ, both are {act}"
        )))))
    }
}

/// `expect.true(value)`
pub fn expect_true(args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 1, "expect.true")?;
    if let Value::Bool(true) = &args[0] {
        Ok(Value::None)
    } else {
        Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "expect.true failed: expected true, got {}",
            args[0]
        )))))
    }
}

/// `expect.false(value)`
pub fn expect_false(args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 1, "expect.false")?;
    if let Value::Bool(false) = &args[0] {
        Ok(Value::None)
    } else {
        Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "expect.false failed: expected false, got {}",
            args[0]
        )))))
    }
}

/// `expect.none(value)`
pub fn expect_none(args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 1, "expect.none")?;
    if let Value::None = &args[0] {
        Ok(Value::None)
    } else {
        Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "expect.none failed: expected none, got {}",
            args[0]
        )))))
    }
}

/// `expect.some(value)`
pub fn expect_some(args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 1, "expect.some")?;
    if !matches!(&args[0], Value::None) {
        Ok(Value::None)
    } else {
        Ok(Value::Failure(Rc::new(FailureValue::new(
            "expect.some failed: expected a value, got none".to_string(),
        ))))
    }
}

/// `expect.failure(value)`
pub fn expect_failure(args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 1, "expect.failure")?;
    if args[0].is_failure() {
        Ok(Value::None)
    } else {
        Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "expect.failure failed: expected Failure, got {}",
            args[0]
        )))))
    }
}

/// `expect.contains(collection, element)`
pub fn expect_contains(args: &[Value]) -> Result<Value, VmFault> {
    require_arity(args, 2, "expect.contains")?;
    let coll = &args[0];
    let elem = &args[1];

    let found = match coll {
        Value::String(s) => match elem {
            Value::String(sub) => s.contains(sub.as_str()),
            other => {
                return Err(VmFault::TypeMismatch {
                    expected: "String substring".to_string(),
                    actual: other.type_name().to_string(),
                });
            }
        },
        Value::List(l) => l.borrow().iter().any(|item| item == elem),
        Value::Dict(d) => d.borrow().get(elem).is_some(),
        Value::Set(s) => s.borrow().contains(elem),
        Value::Bytes(b) => match elem {
            Value::Byte(byte) => b.borrow().contains(byte),
            Value::Int(n) if (0..=255).contains(n) => b.borrow().contains(&(*n as u8)),
            other => {
                return Err(VmFault::TypeMismatch {
                    expected: "Byte or Int in 0..=255".to_string(),
                    actual: other.type_name().to_string(),
                });
            }
        },
        other => {
            return Err(VmFault::TypeMismatch {
                expected: "String, List, Dict, Set, or Bytes for expect.contains".to_string(),
                actual: other.type_name().to_string(),
            });
        }
    };

    if found {
        Ok(Value::None)
    } else {
        Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "expect.contains failed: collection does not contain {elem}"
        )))))
    }
}

/// `expect.approx(actual, expected, tolerance = 0.0001)`
pub fn expect_approx(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() < 2 || args.len() > 3 {
        return Err(VmFault::TypeMismatch {
            expected: "2 or 3 arguments for expect.approx".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    let to_f64 = |v: &Value, name: &str| -> Result<f64, VmFault> {
        match v {
            Value::Float(f) => Ok(*f),
            Value::Int(n) => Ok(*n as f64),
            other => Err(VmFault::TypeMismatch {
                expected: format!("Float or Int for {name}"),
                actual: other.type_name().to_string(),
            }),
        }
    };

    let act = to_f64(&args[0], "actual")?;
    let exp = to_f64(&args[1], "expected")?;
    let tol = if args.len() == 3 {
        to_f64(&args[2], "tolerance")?
    } else {
        0.0001
    };

    let diff = (act - exp).abs();
    if diff <= tol {
        Ok(Value::None)
    } else {
        Ok(Value::Failure(Rc::new(FailureValue::new(format!(
            "expect.approx failed: difference {diff} exceeds tolerance {tol}"
        )))))
    }
}

/// Constructs the canonical `expect` module dictionary.
#[must_use]
pub fn create_expect_module() -> Value {
    let mut entries = Vec::new();
    macro_rules! reg {
        ($name:expr, $arity:expr, $func:expr) => {
            entries.push((
                Value::String(Rc::new($name.to_string())),
                Value::Native {
                    name: concat!("expect.", $name).to_string(),
                    arity: $arity,
                    func: $func,
                },
            ));
        };
    }

    reg!("equal", 2, expect_equal);
    reg!("not_equal", 2, expect_not_equal);
    reg!("true", 1, expect_true);
    reg!("false", 1, expect_false);
    reg!("none", 1, expect_none);
    reg!("some", 1, expect_some);
    reg!("failure", 1, expect_failure);
    reg!("contains", 2, expect_contains);
    reg!("approx", usize::MAX, expect_approx);

    Value::Dict(Rc::new(RefCell::new(DictMap::from_entries(entries))))
}

/// Constructs the canonical `testing` module dictionary.
#[must_use]
pub fn create_module() -> Value {
    let mut entries = Vec::new();
    entries.push((
        Value::String(Rc::new("expect".to_string())),
        create_expect_module(),
    ));

    macro_rules! reg {
        ($name:expr, $arity:expr, $func:expr) => {
            entries.push((
                Value::String(Rc::new($name.to_string())),
                Value::Native {
                    name: concat!("testing.", $name).to_string(),
                    arity: $arity,
                    func: $func,
                },
            ));
        };
    }

    reg!("equal", 2, expect_equal);
    reg!("not_equal", 2, expect_not_equal);
    reg!("true", 1, expect_true);
    reg!("false", 1, expect_false);
    reg!("none", 1, expect_none);
    reg!("some", 1, expect_some);
    reg!("failure", 1, expect_failure);
    reg!("contains", 2, expect_contains);
    reg!("approx", usize::MAX, expect_approx);

    Value::Dict(Rc::new(RefCell::new(DictMap::from_entries(entries))))
}
