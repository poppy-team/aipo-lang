//! Canonical `math` module for Aipo.
//!
//! Provides core mathematical operations and constants:
//! - `abs`: absolute value for integers and floats
//! - `min`: minimum of two numbers
//! - `max`: maximum of two numbers
//! - `floor`: floor to integer
//! - `ceil`: ceiling to integer
//! - `round`: round to nearest integer
//! - `sqrt`: square root (returns Failure for negative numbers)
//! - `pow`: exponentiation with bounds checks
//! - `truncate`: truncation toward zero, returning `Int`
//! - `clamp`: constrains a number to an inclusive interval
//! - `pi`: constant Archimedes ratio
//! - `e`: constant Euler's number

use aipo_vm::{
    DictMap, FailureValue, MAX_SAFE_INT, MIN_SAFE_INT, Value, VmFault, check_finite_float,
    check_safe_int,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Mathematical absolute value.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if operand is not Int or Float,
/// or `VmFault::Overflow` if integer overflows.
pub fn math_abs(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: "1 argument".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    match &args[0] {
        Value::Int(n) => {
            if *n == i64::MIN {
                return Err(VmFault::Overflow {
                    details: "math.abs integer overflow".to_string(),
                });
            }
            check_safe_int(n.abs()).map(Value::Int)
        }
        Value::Float(f) => check_finite_float(f.abs()).map(Value::Float),
        Value::Failure(f) => Ok(Value::Failure(Rc::clone(f))),
        other => Err(VmFault::TypeMismatch {
            expected: "Int or Float".to_string(),
            actual: other.type_name().to_string(),
        }),
    }
}

/// Returns the smaller of two numbers.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if operands are not Int or Float.
pub fn math_min(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 2 {
        return Err(VmFault::TypeMismatch {
            expected: "2 arguments".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int((*a).min(*b))),
        (Value::Float(a), Value::Float(b)) => check_finite_float(a.min(*b)).map(Value::Float),
        (Value::Int(a), Value::Float(b)) => {
            #[allow(clippy::cast_precision_loss)]
            let af = *a as f64;
            check_finite_float(af.min(*b)).map(Value::Float)
        }
        (Value::Float(a), Value::Int(b)) => {
            #[allow(clippy::cast_precision_loss)]
            let bf = *b as f64;
            check_finite_float(a.min(bf)).map(Value::Float)
        }
        (Value::Failure(f), _) | (_, Value::Failure(f)) => Ok(Value::Failure(Rc::clone(f))),
        (a, b) => Err(VmFault::TypeMismatch {
            expected: "two numbers (Int or Float)".to_string(),
            actual: format!("{} and {}", a.type_name(), b.type_name()),
        }),
    }
}

/// Returns the larger of two numbers.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if operands are not Int or Float.
pub fn math_max(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 2 {
        return Err(VmFault::TypeMismatch {
            expected: "2 arguments".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    match (&args[0], &args[1]) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int((*a).max(*b))),
        (Value::Float(a), Value::Float(b)) => check_finite_float(a.max(*b)).map(Value::Float),
        (Value::Int(a), Value::Float(b)) => {
            #[allow(clippy::cast_precision_loss)]
            let af = *a as f64;
            check_finite_float(af.max(*b)).map(Value::Float)
        }
        (Value::Float(a), Value::Int(b)) => {
            #[allow(clippy::cast_precision_loss)]
            let bf = *b as f64;
            check_finite_float(a.max(bf)).map(Value::Float)
        }
        (Value::Failure(f), _) | (_, Value::Failure(f)) => Ok(Value::Failure(Rc::clone(f))),
        (a, b) => Err(VmFault::TypeMismatch {
            expected: "two numbers (Int or Float)".to_string(),
            actual: format!("{} and {}", a.type_name(), b.type_name()),
        }),
    }
}

/// Floor function to nearest lower integer.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if operand is not Int or Float,
/// or `VmFault::Overflow` if result exceeds safe integer range.
pub fn math_floor(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: "1 argument".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => {
            let floored = f.floor();
            #[allow(clippy::cast_possible_truncation)]
            if floored < MIN_SAFE_INT as f64 || floored > MAX_SAFE_INT as f64 {
                return Err(VmFault::Overflow {
                    details: "math.floor exceeds safe integer bounds".to_string(),
                });
            }
            #[allow(clippy::cast_possible_truncation)]
            check_safe_int(floored as i64).map(Value::Int)
        }
        Value::Failure(f) => Ok(Value::Failure(Rc::clone(f))),
        other => Err(VmFault::TypeMismatch {
            expected: "Int or Float".to_string(),
            actual: other.type_name().to_string(),
        }),
    }
}

/// Ceiling function to nearest upper integer.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if operand is not Int or Float,
/// or `VmFault::Overflow` if result exceeds safe integer range.
pub fn math_ceil(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: "1 argument".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => {
            let ceiled = f.ceil();
            #[allow(clippy::cast_possible_truncation)]
            if ceiled < MIN_SAFE_INT as f64 || ceiled > MAX_SAFE_INT as f64 {
                return Err(VmFault::Overflow {
                    details: "math.ceil exceeds safe integer bounds".to_string(),
                });
            }
            #[allow(clippy::cast_possible_truncation)]
            check_safe_int(ceiled as i64).map(Value::Int)
        }
        Value::Failure(f) => Ok(Value::Failure(Rc::clone(f))),
        other => Err(VmFault::TypeMismatch {
            expected: "Int or Float".to_string(),
            actual: other.type_name().to_string(),
        }),
    }
}

/// Rounds to nearest integer.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if operand is not Int or Float,
/// or `VmFault::Overflow` if result exceeds safe integer range.
pub fn math_round(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: "1 argument".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => {
            let rounded = f.round();
            #[allow(clippy::cast_possible_truncation)]
            if rounded < MIN_SAFE_INT as f64 || rounded > MAX_SAFE_INT as f64 {
                return Err(VmFault::Overflow {
                    details: "math.round exceeds safe integer bounds".to_string(),
                });
            }
            #[allow(clippy::cast_possible_truncation)]
            check_safe_int(rounded as i64).map(Value::Int)
        }
        Value::Failure(f) => Ok(Value::Failure(Rc::clone(f))),
        other => Err(VmFault::TypeMismatch {
            expected: "Int or Float".to_string(),
            actual: other.type_name().to_string(),
        }),
    }
}

/// Truncates toward zero, returning an `Int` when representable.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if operand is not Int or Float,
/// or `VmFault::Overflow` if the result exceeds the `Int` range.
pub fn math_truncate(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: "1 argument".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    match &args[0] {
        Value::Int(n) => Ok(Value::Int(*n)),
        Value::Float(f) => {
            let truncated = f.trunc();
            #[allow(clippy::cast_possible_truncation)]
            if truncated < MIN_SAFE_INT as f64 || truncated > MAX_SAFE_INT as f64 {
                return Err(VmFault::Overflow {
                    details: "math.truncate exceeds safe integer bounds".to_string(),
                });
            }
            #[allow(clippy::cast_possible_truncation)]
            check_safe_int(truncated as i64).map(Value::Int)
        }
        Value::Failure(f) => Ok(Value::Failure(Rc::clone(f))),
        other => Err(VmFault::TypeMismatch {
            expected: "Int or Float".to_string(),
            actual: other.type_name().to_string(),
        }),
    }
}

/// Constrains a number to the inclusive `[min, max]` interval.
///
/// Mixed operands follow the canonical `Int -> Float` promotion. Inverted bounds
/// (`min > max`) produce a recoverable `Failure`; see
/// `docs/adp/ADP-001-byte-and-core-types-as-values.md` for the open question.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if operands are not numbers, or
/// `VmFault::NonFiniteFloat` if a float operand is not finite.
pub fn math_clamp(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 3 {
        return Err(VmFault::TypeMismatch {
            expected: "3 arguments".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    if let Value::Failure(f) = &args[0] {
        return Ok(Value::Failure(Rc::clone(f)));
    }
    if let Value::Failure(f) = &args[1] {
        return Ok(Value::Failure(Rc::clone(f)));
    }
    if let Value::Failure(f) = &args[2] {
        return Ok(Value::Failure(Rc::clone(f)));
    }

    if let (Value::Int(value), Value::Int(low), Value::Int(high)) = (&args[0], &args[1], &args[2]) {
        if low > high {
            return Ok(Value::Failure(Rc::new(FailureValue {
                message: format!("math.clamp bounds are inverted: {low} > {high}"),
            })));
        }
        return Ok(Value::Int((*value).clamp(*low, *high)));
    }

    let to_float = |val: &Value| -> Result<f64, VmFault> {
        match val {
            Value::Int(n) =>
            {
                #[allow(clippy::cast_precision_loss)]
                Ok(*n as f64)
            }
            Value::Float(f) => check_finite_float(*f),
            other => Err(VmFault::TypeMismatch {
                expected: "numbers (Int or Float)".to_string(),
                actual: other.type_name().to_string(),
            }),
        }
    };

    let value = to_float(&args[0])?;
    let low = to_float(&args[1])?;
    let high = to_float(&args[2])?;

    if low > high {
        return Ok(Value::Failure(Rc::new(FailureValue {
            message: format!("math.clamp bounds are inverted: {low} > {high}"),
        })));
    }

    check_finite_float(value.clamp(low, high)).map(Value::Float)
}

/// Square root function.
///
/// Computes square root, returning recoverable `Value::Failure` on negative inputs.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if operand is not Int or Float.
pub fn math_sqrt(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 1 {
        return Err(VmFault::TypeMismatch {
            expected: "1 argument".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    let val = match &args[0] {
        Value::Int(n) =>
        {
            #[allow(clippy::cast_precision_loss)]
            (*n as f64)
        }
        Value::Float(f) => *f,
        Value::Failure(f) => return Ok(Value::Failure(Rc::clone(f))),
        other => {
            return Err(VmFault::TypeMismatch {
                expected: "Int or Float".to_string(),
                actual: other.type_name().to_string(),
            });
        }
    };

    if val < 0.0 {
        return Ok(Value::Failure(Rc::new(FailureValue {
            message: "cannot compute square root of negative number".to_string(),
        })));
    }

    check_finite_float(val.sqrt()).map(Value::Float)
}

/// Power / exponentiation function.
///
/// # Errors
/// Returns `VmFault::TypeMismatch` if operands are not numbers,
/// or `VmFault::Overflow` / `VmFault::NonFiniteFloat`.
pub fn math_pow(args: &[Value]) -> Result<Value, VmFault> {
    if args.len() != 2 {
        return Err(VmFault::TypeMismatch {
            expected: "2 arguments".to_string(),
            actual: format!("{} arguments", args.len()),
        });
    }

    match (&args[0], &args[1]) {
        (Value::Int(base), Value::Int(exp)) if *exp >= 0 && *exp <= 53 => {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let exp_u32 = *exp as u32;
            if let Some(res) = base.checked_pow(exp_u32) {
                if (MIN_SAFE_INT..=MAX_SAFE_INT).contains(&res) {
                    return Ok(Value::Int(res));
                }
            }
            #[allow(clippy::cast_precision_loss)]
            let f = (*base as f64).powf(*exp as f64);
            check_finite_float(f).map(Value::Float)
        }
        (Value::Int(base), Value::Int(exp)) => {
            #[allow(clippy::cast_precision_loss)]
            let f = (*base as f64).powf(*exp as f64);
            check_finite_float(f).map(Value::Float)
        }
        (Value::Float(base), Value::Float(exp)) => {
            check_finite_float(base.powf(*exp)).map(Value::Float)
        }
        (Value::Int(base), Value::Float(exp)) => {
            #[allow(clippy::cast_precision_loss)]
            let f = (*base as f64).powf(*exp);
            check_finite_float(f).map(Value::Float)
        }
        (Value::Float(base), Value::Int(exp)) => {
            #[allow(clippy::cast_precision_loss)]
            let f = base.powf(*exp as f64);
            check_finite_float(f).map(Value::Float)
        }
        (Value::Failure(f), _) | (_, Value::Failure(f)) => Ok(Value::Failure(Rc::clone(f))),
        (a, b) => Err(VmFault::TypeMismatch {
            expected: "two numbers (Int or Float)".to_string(),
            actual: format!("{} and {}", a.type_name(), b.type_name()),
        }),
    }
}

/// Constructs the canonical `math` module dictionary.
#[must_use]
pub fn create_module() -> Value {
    let entries = vec![
        (
            Value::String(Rc::new("abs".to_string())),
            Value::Native {
                name: "math.abs".to_string(),
                arity: 1,
                func: math_abs,
            },
        ),
        (
            Value::String(Rc::new("min".to_string())),
            Value::Native {
                name: "math.min".to_string(),
                arity: 2,
                func: math_min,
            },
        ),
        (
            Value::String(Rc::new("max".to_string())),
            Value::Native {
                name: "math.max".to_string(),
                arity: 2,
                func: math_max,
            },
        ),
        (
            Value::String(Rc::new("floor".to_string())),
            Value::Native {
                name: "math.floor".to_string(),
                arity: 1,
                func: math_floor,
            },
        ),
        (
            Value::String(Rc::new("ceil".to_string())),
            Value::Native {
                name: "math.ceil".to_string(),
                arity: 1,
                func: math_ceil,
            },
        ),
        (
            Value::String(Rc::new("round".to_string())),
            Value::Native {
                name: "math.round".to_string(),
                arity: 1,
                func: math_round,
            },
        ),
        (
            Value::String(Rc::new("sqrt".to_string())),
            Value::Native {
                name: "math.sqrt".to_string(),
                arity: 1,
                func: math_sqrt,
            },
        ),
        (
            Value::String(Rc::new("pow".to_string())),
            Value::Native {
                name: "math.pow".to_string(),
                arity: 2,
                func: math_pow,
            },
        ),
        (
            Value::String(Rc::new("truncate".to_string())),
            Value::Native {
                name: "math.truncate".to_string(),
                arity: 1,
                func: math_truncate,
            },
        ),
        (
            Value::String(Rc::new("clamp".to_string())),
            Value::Native {
                name: "math.clamp".to_string(),
                arity: 3,
                func: math_clamp,
            },
        ),
        (
            Value::String(Rc::new("pi".to_string())),
            Value::Float(std::f64::consts::PI),
        ),
        (
            Value::String(Rc::new("e".to_string())),
            Value::Float(std::f64::consts::E),
        ),
    ];

    Value::Dict(Rc::new(RefCell::new(DictMap::from_entries(entries))))
}
