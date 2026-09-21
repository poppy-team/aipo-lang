//! Exhaustive integration and unit tests for aipo-stdlib.
//!
//! Covers Prelude V1, math module, string module, io module,
//! and end-to-end VM execution calling stdlib routines.

use aipo_bytecode::{BytecodeModule, Constant, OpCode};
use aipo_runtime::NativeRegistry;
use aipo_stdlib::{convert, io, math, prelude, register_stdlib, string};
use aipo_vm::{FailureValue, Value, Vm};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[test]
fn test_prelude_len() {
    // String
    let str_val = Value::String(Rc::new("café🚀".to_string()));
    let len_res = prelude::native_len(&[str_val]).unwrap();
    assert_eq!(len_res, Value::Int(5));

    // List
    let list_val = Value::List(Rc::new(RefCell::new(vec![
        Value::Int(1),
        Value::Int(2),
        Value::Int(3),
    ])));
    let len_res = prelude::native_len(&[list_val]).unwrap();
    assert_eq!(len_res, Value::Int(3));

    // Dict
    let dict_val = Value::Dict(Rc::new(RefCell::new(
        [
            (Value::Int(1), Value::String(Rc::new("a".to_string()))),
            (Value::Int(2), Value::String(Rc::new("b".to_string()))),
        ]
        .into_iter()
        .collect(),
    )));
    let len_res = prelude::native_len(&[dict_val]).unwrap();
    assert_eq!(len_res, Value::Int(2));

    // Failure propagation
    let fail_val = Value::Failure(Rc::new(FailureValue {
        message: "bad".to_string(),
    }));
    let len_res = prelude::native_len(std::slice::from_ref(&fail_val)).unwrap();
    assert_eq!(len_res, fail_val);

    // Invalid type
    let err = prelude::native_len(&[Value::Int(42)]).unwrap_err();
    assert!(matches!(err, aipo_vm::VmFault::TypeMismatch { .. }));
}

#[test]
fn test_prelude_copy() {
    // List shallow copy: modification to clone does not mutate original
    let orig_list = Value::List(Rc::new(RefCell::new(vec![Value::Int(1), Value::Int(2)])));
    let copy_val = prelude::native_copy(std::slice::from_ref(&orig_list)).unwrap();

    assert_eq!(orig_list, copy_val);
    if let Value::List(cloned_rc) = &copy_val {
        cloned_rc.borrow_mut().push(Value::Int(3));
    }
    if let Value::List(orig_rc) = &orig_list {
        assert_eq!(orig_rc.borrow().len(), 2);
    }
    if let Value::List(cloned_rc) = &copy_val {
        assert_eq!(cloned_rc.borrow().len(), 3);
    }

    // Scalar copy is identity
    let int_val = Value::Int(42);
    assert_eq!(prelude::native_copy(&[int_val]).unwrap(), Value::Int(42));
}

#[test]
fn test_prelude_same() {
    let list_a = Value::List(Rc::new(RefCell::new(vec![Value::Int(1)])));
    let list_b = Value::List(Rc::new(RefCell::new(vec![Value::Int(1)])));
    let list_c = list_a.clone();

    // Two distinct lists with same contents have distinct identities
    assert_eq!(
        prelude::native_same(&[list_a.clone(), list_b]).unwrap(),
        Value::Bool(false)
    );

    // Same reference
    assert_eq!(
        prelude::native_same(&[list_a, list_c]).unwrap(),
        Value::Bool(true)
    );

    // Scalar equality
    assert_eq!(
        prelude::native_same(&[Value::Int(5), Value::Int(5)]).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        prelude::native_same(&[Value::Int(5), Value::Int(6)]).unwrap(),
        Value::Bool(false)
    );
}

#[test]
fn test_prelude_some() {
    assert_eq!(
        prelude::native_some(&[Value::None]).unwrap(),
        Value::Bool(false)
    );
    assert_eq!(
        prelude::native_some(&[Value::Int(0)]).unwrap(),
        Value::Bool(true)
    );
    assert_eq!(
        prelude::native_some(&[Value::Bool(false)]).unwrap(),
        Value::Bool(true)
    );
}

#[test]
fn test_prelude_fail() {
    let res =
        prelude::native_fail(&[Value::String(Rc::new("database offline".to_string()))]).unwrap();
    match res {
        Value::Failure(f) => assert_eq!(f.message, "database offline"),
        other => panic!("expected Failure, got {other:?}"),
    }
}

#[test]
fn test_math_abs() {
    assert_eq!(math::math_abs(&[Value::Int(-42)]).unwrap(), Value::Int(42));
    assert_eq!(math::math_abs(&[Value::Int(42)]).unwrap(), Value::Int(42));
    assert_eq!(
        math::math_abs(&[Value::Float(-3.5)]).unwrap(),
        Value::Float(3.5)
    );
}

#[test]
fn test_math_min_max() {
    assert_eq!(
        math::math_min(&[Value::Int(10), Value::Int(20)]).unwrap(),
        Value::Int(10)
    );
    assert_eq!(
        math::math_max(&[Value::Int(10), Value::Int(20)]).unwrap(),
        Value::Int(20)
    );
    assert_eq!(
        math::math_min(&[Value::Int(10), Value::Float(5.5)]).unwrap(),
        Value::Float(5.5)
    );
    assert_eq!(
        math::math_max(&[Value::Int(10), Value::Float(5.5)]).unwrap(),
        Value::Float(10.0)
    );
}

#[test]
fn test_math_floor_ceil_round() {
    assert_eq!(
        math::math_floor(&[Value::Float(3.7)]).unwrap(),
        Value::Int(3)
    );
    assert_eq!(
        math::math_ceil(&[Value::Float(3.2)]).unwrap(),
        Value::Int(4)
    );
    assert_eq!(
        math::math_round(&[Value::Float(3.4)]).unwrap(),
        Value::Int(3)
    );
    assert_eq!(
        math::math_round(&[Value::Float(3.5)]).unwrap(),
        Value::Int(4)
    );
}

#[test]
fn test_math_sqrt() {
    assert_eq!(
        math::math_sqrt(&[Value::Int(16)]).unwrap(),
        Value::Float(4.0)
    );
    assert_eq!(
        math::math_sqrt(&[Value::Float(25.0)]).unwrap(),
        Value::Float(5.0)
    );

    // Negative sqrt produces recoverable Failure
    let neg_res = math::math_sqrt(&[Value::Int(-1)]).unwrap();
    assert!(neg_res.is_failure());
}

#[test]
fn test_math_pow() {
    assert_eq!(
        math::math_pow(&[Value::Int(2), Value::Int(8)]).unwrap(),
        Value::Int(256)
    );
    assert_eq!(
        math::math_pow(&[Value::Float(4.0), Value::Float(0.5)]).unwrap(),
        Value::Float(2.0)
    );
}

#[test]
fn test_string_operations() {
    let s = Value::String(Rc::new("  Hello, Aipo World!  ".to_string()));

    // Trim
    let trimmed = string::string_trim(&[s]).unwrap();
    assert_eq!(
        trimmed,
        Value::String(Rc::new("Hello, Aipo World!".to_string()))
    );

    // Lower and Upper
    let lower = string::string_lower(std::slice::from_ref(&trimmed)).unwrap();
    assert_eq!(
        lower,
        Value::String(Rc::new("hello, aipo world!".to_string()))
    );

    let upper = string::string_upper(std::slice::from_ref(&trimmed)).unwrap();
    assert_eq!(
        upper,
        Value::String(Rc::new("HELLO, AIPO WORLD!".to_string()))
    );

    // Contains, StartsWith, EndsWith
    let needle = Value::String(Rc::new("Aipo".to_string()));
    assert_eq!(
        string::string_contains(&[trimmed.clone(), needle.clone()]).unwrap(),
        Value::Bool(true)
    );

    let prefix = Value::String(Rc::new("Hello".to_string()));
    assert_eq!(
        string::string_starts_with(&[trimmed.clone(), prefix]).unwrap(),
        Value::Bool(true)
    );

    let suffix = Value::String(Rc::new("World!".to_string()));
    assert_eq!(
        string::string_ends_with(&[trimmed.clone(), suffix]).unwrap(),
        Value::Bool(true)
    );

    // Replace
    let replaced = string::string_replace(&[
        trimmed,
        Value::String(Rc::new("World".to_string())),
        Value::String(Rc::new("Universe".to_string())),
    ])
    .unwrap();
    assert_eq!(
        replaced,
        Value::String(Rc::new("Hello, Aipo Universe!".to_string()))
    );

    // Split and Join
    let csv = Value::String(Rc::new("a,b,c".to_string()));
    let comma = Value::String(Rc::new(",".to_string()));
    let split_res = string::string_split(&[csv, comma.clone()]).unwrap();
    if let Value::List(parts) = &split_res {
        assert_eq!(parts.borrow().len(), 3);
    } else {
        panic!("expected List");
    }
    let joined =
        string::string_join(&[Value::String(Rc::new(" - ".to_string())), split_res]).unwrap();
    assert_eq!(joined, Value::String(Rc::new("a - b - c".to_string())));

    // Slice
    let orig = Value::String(Rc::new("0123456789".to_string()));
    let sliced = string::string_slice(&[orig.clone(), Value::Int(2), Value::Int(5)]).unwrap();
    assert_eq!(sliced, Value::String(Rc::new("234".to_string())));

    // Negative indexing in slice
    let sliced_tail = string::string_slice(&[orig, Value::Int(-3), Value::Int(10)]).unwrap();
    assert_eq!(sliced_tail, Value::String(Rc::new("789".to_string())));
}

/// Canon makes NFC an invariant of `String` and puts normalization at the construction
/// boundaries, so every operation that can join a base character with a combining mark
/// normalizes its output (ADP-001 Q5, closed by `P00-G15`).
#[test]
fn test_string_operations_preserve_nfc() {
    let mark = Value::String(Rc::new("\u{0301}".to_string()));
    let composed = Value::String(Rc::new("é".to_string()));

    // `join` brings the mark next to a base character.
    let joined = string::string_join(&[
        Value::String(Rc::new(String::new())),
        Value::List(Rc::new(RefCell::new(vec![
            Value::String(Rc::new("e".to_string())),
            mark.clone(),
        ]))),
    ])
    .unwrap();
    assert_eq!(joined, composed, "join must store the composed character");

    // `replace` and `format` alike.
    let replaced = string::string_replace(&[
        Value::String(Rc::new("e.".to_string())),
        Value::String(Rc::new(".".to_string())),
        mark.clone(),
    ])
    .unwrap();
    assert_eq!(replaced, composed);

    let formatted = string::string_format(&[
        Value::String(Rc::new("e{v}".to_string())),
        dict_with(vec![("v", mark)]),
    ])
    .unwrap();
    assert_eq!(formatted, composed);

    // A substring keeps the invariant for free: removing characters never makes two
    // previously non-adjacent characters adjacent.
    let sliced = string::string_slice(&[composed.clone(), Value::Int(0), Value::Int(1)]).unwrap();
    assert_eq!(sliced, composed);

    // A mark with no precomposed form is left alone rather than dropped: `e` + two combining
    // acutes normalizes to the composed pair plus one surviving mark.
    let doubled = string::string_format(&[
        Value::String(Rc::new("e{v}".to_string())),
        dict_with(vec![(
            "v",
            Value::String(Rc::new("\u{0301}\u{0301}".to_string())),
        )]),
    ])
    .unwrap();
    assert_eq!(
        string::string_len(&[doubled]).unwrap(),
        Value::Int(2),
        "normalization composes what it can and keeps the rest"
    );
}

#[test]
fn test_string_byte_len_and_len() {
    // "café🚀": 5 code points, 1 + 1 + 1 + 2 + 4 = 9 UTF-8 bytes.
    let text = Value::String(Rc::new("café🚀".to_string()));
    assert_eq!(
        string::string_len(std::slice::from_ref(&text)).unwrap(),
        Value::Int(5)
    );
    assert_eq!(
        string::string_byte_len(std::slice::from_ref(&text)).unwrap(),
        Value::Int(9)
    );
    assert_eq!(
        string::string_byte_len(&[Value::String(Rc::new(String::new()))]).unwrap(),
        Value::Int(0)
    );
}

#[test]
fn test_string_find() {
    let text = Value::String(Rc::new("café🚀café".to_string()));

    // Index is measured in code points, not bytes.
    let rocket = Value::String(Rc::new("🚀".to_string()));
    assert_eq!(
        string::string_find(&[text.clone(), rocket]).unwrap(),
        Value::Int(4)
    );

    // Empty substring is found at index 0 (canon edge case).
    assert_eq!(
        string::string_find(&[text.clone(), Value::String(Rc::new(String::new()))]).unwrap(),
        Value::Int(0)
    );

    // Missing substring yields none.
    assert_eq!(
        string::string_find(&[text, Value::String(Rc::new("zzz".to_string()))]).unwrap(),
        Value::None
    );
}

#[test]
fn test_string_capitalize() {
    assert_eq!(
        string::string_capitalize(&[Value::String(Rc::new("hELLO world".to_string()))]).unwrap(),
        Value::String(Rc::new("Hello world".to_string()))
    );

    // Leading non-cased characters are preserved.
    assert_eq!(
        string::string_capitalize(&[Value::String(Rc::new("123abc".to_string()))]).unwrap(),
        Value::String(Rc::new("123Abc".to_string()))
    );

    // Text without any cased character is unchanged (canon edge case).
    assert_eq!(
        string::string_capitalize(&[Value::String(Rc::new("123".to_string()))]).unwrap(),
        Value::String(Rc::new("123".to_string()))
    );

    assert_eq!(
        string::string_capitalize(&[Value::String(Rc::new(String::new()))]).unwrap(),
        Value::String(Rc::new(String::new()))
    );
}

#[test]
fn test_string_reverse_graphemes() {
    // "e\u{301}" is a single extended grapheme cluster (e + combining acute).
    // Reversing keeps the cluster intact and the result is re-normalized to NFC,
    // so the trailing cluster comes back as the precomposed "é".
    let composed = Value::String(Rc::new("e\u{301}ab".to_string()));
    let reversed = string::string_reverse(&[composed]).unwrap();
    assert_eq!(reversed, Value::String(Rc::new("baé".to_string())));

    // Emoji with variation selector stays a single visual unit.
    let flag = Value::String(Rc::new("🇧🇷".to_string()));
    let reversed = string::string_reverse(std::slice::from_ref(&flag)).unwrap();
    assert_eq!(reversed, flag);

    assert_eq!(
        string::string_reverse(&[Value::String(Rc::new(String::new()))]).unwrap(),
        Value::String(Rc::new(String::new()))
    );
}

fn dict_with(entries: Vec<(&str, Value)>) -> Value {
    Value::Dict(Rc::new(RefCell::new(
        entries
            .into_iter()
            .map(|(k, v)| (Value::String(Rc::new(k.to_string())), v))
            .collect(),
    )))
}

#[test]
fn test_string_format() {
    let template = Value::String(Rc::new("idade: {age}, nome: {name}".to_string()));
    let values = dict_with(vec![
        ("age", Value::Int(25)),
        ("name", Value::String(Rc::new("Ada".to_string()))),
    ]);

    let rendered = string::string_format(&[template, values.clone()]).unwrap();
    assert_eq!(
        rendered,
        Value::String(Rc::new("idade: 25, nome: Ada".to_string()))
    );

    // Templates without placeholders are returned unchanged.
    let plain = Value::String(Rc::new("sem placeholders".to_string()));
    assert_eq!(
        string::string_format(&[plain, values.clone()]).unwrap(),
        Value::String(Rc::new("sem placeholders".to_string()))
    );

    // Extra keys are allowed and ignored.
    let only_age = Value::String(Rc::new("age={age}".to_string()));
    assert_eq!(
        string::string_format(&[only_age, values.clone()]).unwrap(),
        Value::String(Rc::new("age=25".to_string()))
    );

    // Escaped braces produce literal braces.
    let escaped = Value::String(Rc::new("{{age}} {age}".to_string()));
    assert_eq!(
        string::string_format(&[escaped, values.clone()]).unwrap(),
        Value::String(Rc::new("{age} 25".to_string()))
    );

    // Missing key is a recoverable Failure.
    let missing = Value::String(Rc::new("{absent}".to_string()));
    assert!(
        string::string_format(&[missing, values.clone()])
            .unwrap()
            .is_failure()
    );

    // Malformed template is a recoverable Failure.
    let unclosed = Value::String(Rc::new("{age".to_string()));
    assert!(
        string::string_format(&[unclosed, values.clone()])
            .unwrap()
            .is_failure()
    );

    // Positional placeholders are outside V1 and produce a recoverable Failure.
    let positional = Value::String(Rc::new("{0}".to_string()));
    assert!(
        string::string_format(&[positional, values.clone()])
            .unwrap()
            .is_failure()
    );

    // values must be a Dict.
    let err = string::string_format(&[Value::String(Rc::new("{age}".to_string())), Value::Int(1)])
        .unwrap_err();
    assert!(matches!(err, aipo_vm::VmFault::TypeMismatch { .. }));
}

#[test]
fn test_string_canon_argument_rules() {
    // split with an empty separator is invalid in V1.
    let err = string::string_split(&[
        Value::String(Rc::new("abc".to_string())),
        Value::String(Rc::new(String::new())),
    ])
    .unwrap_err();
    assert!(matches!(err, aipo_vm::VmFault::TypeMismatch { .. }));

    // split preserves empty fields, including at the edges.
    let parts = string::string_split(&[
        Value::String(Rc::new(",a,b,".to_string())),
        Value::String(Rc::new(",".to_string())),
    ])
    .unwrap();
    assert_eq!(
        parts,
        Value::List(Rc::new(RefCell::new(vec![
            Value::String(Rc::new(String::new())),
            Value::String(Rc::new("a".to_string())),
            Value::String(Rc::new("b".to_string())),
            Value::String(Rc::new(String::new())),
        ])))
    );

    let empty_parts = string::string_split(&[
        Value::String(Rc::new(String::new())),
        Value::String(Rc::new(",".to_string())),
    ])
    .unwrap();
    assert_eq!(
        empty_parts,
        Value::List(Rc::new(RefCell::new(vec![Value::String(Rc::new(
            String::new()
        ))])))
    );

    // replace with an empty pattern is invalid in V1.
    let err = string::string_replace(&[
        Value::String(Rc::new("abc".to_string())),
        Value::String(Rc::new(String::new())),
        Value::String(Rc::new("x".to_string())),
    ])
    .unwrap_err();
    assert!(matches!(err, aipo_vm::VmFault::TypeMismatch { .. }));
}

#[test]
fn test_string_join_canon_rules() {
    let comma = Value::String(Rc::new(",".to_string()));

    // join(separator, []) == ""
    let empty = string::string_join(&[
        comma.clone(),
        Value::List(Rc::new(RefCell::new(Vec::new()))),
    ])
    .unwrap();
    assert_eq!(empty, Value::String(Rc::new(String::new())));

    // A single element returns that element.
    let single = string::string_join(&[
        comma.clone(),
        Value::List(Rc::new(RefCell::new(vec![Value::String(Rc::new(
            "only".to_string(),
        ))]))),
    ])
    .unwrap();
    assert_eq!(single, Value::String(Rc::new("only".to_string())));

    // Elements must already be String: no implicit coercion.
    let err = string::string_join(&[
        comma,
        Value::List(Rc::new(RefCell::new(vec![Value::Int(1)]))),
    ])
    .unwrap_err();
    assert!(matches!(err, aipo_vm::VmFault::TypeMismatch { .. }));
}

#[test]
fn test_math_truncate() {
    assert_eq!(
        math::math_truncate(&[Value::Float(3.9)]).unwrap(),
        Value::Int(3)
    );
    assert_eq!(
        math::math_truncate(&[Value::Float(-3.9)]).unwrap(),
        Value::Int(-3)
    );
    assert_eq!(
        math::math_truncate(&[Value::Int(7)]).unwrap(),
        Value::Int(7)
    );
}

#[test]
fn test_math_clamp() {
    assert_eq!(
        math::math_clamp(&[Value::Int(15), Value::Int(0), Value::Int(10)]).unwrap(),
        Value::Int(10)
    );
    assert_eq!(
        math::math_clamp(&[Value::Int(-5), Value::Int(0), Value::Int(10)]).unwrap(),
        Value::Int(0)
    );
    assert_eq!(
        math::math_clamp(&[Value::Int(5), Value::Int(0), Value::Int(10)]).unwrap(),
        Value::Int(5)
    );

    // Mixed operands follow Int -> Float promotion.
    assert_eq!(
        math::math_clamp(&[Value::Float(7.5), Value::Int(0), Value::Float(5.5)]).unwrap(),
        Value::Float(5.5)
    );

    // Inverted bounds are a value outside a valid range, so they produce a recoverable
    // Failure (ADP-001 Q3, closed by `P00-G15`).
    let inverted = math::math_clamp(&[Value::Int(5), Value::Int(10), Value::Int(0)]).unwrap();
    assert!(inverted.is_failure());
}

#[test]
fn test_math_round_half_away_from_zero() {
    assert_eq!(
        math::math_round(&[Value::Float(2.5)]).unwrap(),
        Value::Int(3)
    );
    assert_eq!(
        math::math_round(&[Value::Float(-2.5)]).unwrap(),
        Value::Int(-3)
    );
}

#[test]
fn test_convert_int() {
    assert_eq!(
        convert::convert_int(&[Value::Int(42)]).unwrap(),
        Value::Int(42)
    );
    // Float truncates toward zero.
    assert_eq!(
        convert::convert_int(&[Value::Float(3.8)]).unwrap(),
        Value::Int(3)
    );
    assert_eq!(
        convert::convert_int(&[Value::Float(-3.8)]).unwrap(),
        Value::Int(-3)
    );
    // String parses the whole text.
    assert_eq!(
        convert::convert_int(&[Value::String(Rc::new("8080".to_string()))]).unwrap(),
        Value::Int(8080)
    );
    // Invalid or out-of-range text produces a recoverable Failure.
    assert!(
        convert::convert_int(&[Value::String(Rc::new("abc".to_string()))])
            .unwrap()
            .is_failure()
    );
    assert!(
        convert::convert_int(&[Value::String(Rc::new("4.5".to_string()))])
            .unwrap()
            .is_failure()
    );
    assert!(
        convert::convert_int(&[Value::String(Rc::new("999999999999999999999".to_string()))])
            .unwrap()
            .is_failure()
    );
    // Unsupported category is a fault, not a Failure.
    let err = convert::convert_int(&[Value::Bool(true)]).unwrap_err();
    assert!(matches!(err, aipo_vm::VmFault::TypeMismatch { .. }));
}

#[test]
fn test_convert_float() {
    assert_eq!(
        convert::convert_float(&[Value::Float(1.5)]).unwrap(),
        Value::Float(1.5)
    );
    assert_eq!(
        convert::convert_float(&[Value::Int(10)]).unwrap(),
        Value::Float(10.0)
    );
    // Canon example: Float("4.5") or_else 1.0
    assert_eq!(
        convert::convert_float(&[Value::String(Rc::new("4.5".to_string()))]).unwrap(),
        Value::Float(4.5)
    );
    assert!(
        convert::convert_float(&[Value::String(Rc::new("x".to_string()))])
            .unwrap()
            .is_failure()
    );
}

#[test]
fn test_convert_byte() {
    // Canon example: Byte("255") or_else Byte(255)
    assert_eq!(
        convert::convert_byte(&[Value::String(Rc::new("255".to_string()))]).unwrap(),
        Value::Int(255)
    );
    assert_eq!(
        convert::convert_byte(&[Value::Int(255)]).unwrap(),
        Value::Int(255)
    );
    assert_eq!(
        convert::convert_byte(&[Value::Float(0.0)]).unwrap(),
        Value::Int(0)
    );

    // Out of range never wraps around or saturates.
    assert!(
        convert::convert_byte(&[Value::Int(256)])
            .unwrap()
            .is_failure()
    );
    assert!(
        convert::convert_byte(&[Value::Int(-1)])
            .unwrap()
            .is_failure()
    );
    assert!(
        convert::convert_byte(&[Value::String(Rc::new("256".to_string()))])
            .unwrap()
            .is_failure()
    );
    assert!(
        convert::convert_byte(&[Value::Float(1.5)])
            .unwrap()
            .is_failure()
    );
}

#[test]
fn test_convert_bytes() {
    // Canon example: `let data = Bytes(32)` is a mutable zero-filled block.
    let bytes = convert::convert_bytes(&[Value::Int(32)]).unwrap();
    let Value::Bytes(buffer) = &bytes else {
        panic!("Bytes(32) must produce a Bytes value, got {bytes:?}");
    };
    assert_eq!(buffer.borrow().len(), 32);
    assert!(buffer.borrow().iter().all(|byte| *byte == 0));

    // Zero is a valid (empty) block, and negative or oversized counts are recoverable.
    let empty = convert::convert_bytes(&[Value::Int(0)]).unwrap();
    assert_eq!(empty, Value::Bytes(Rc::new(RefCell::new(Vec::new()))));
    assert!(
        convert::convert_bytes(&[Value::Int(-1)])
            .unwrap()
            .is_failure()
    );
    assert!(
        convert::convert_bytes(&[Value::Int(i64::MAX)])
            .unwrap()
            .is_failure()
    );

    // Only `Int` is accepted; other categories are a fault.
    let err = convert::convert_bytes(&[Value::String(Rc::new("32".to_string()))]).unwrap_err();
    assert!(matches!(err, aipo_vm::VmFault::TypeMismatch { .. }));
}

#[test]
fn test_convert_string() {
    assert_eq!(
        convert::convert_string(&[Value::String(Rc::new("abc".to_string()))]).unwrap(),
        Value::String(Rc::new("abc".to_string()))
    );
    // Canon example: String(port)
    assert_eq!(
        convert::convert_string(&[Value::Int(8080)]).unwrap(),
        Value::String(Rc::new("8080".to_string()))
    );
    assert_eq!(
        convert::convert_string(&[Value::Float(1.5)]).unwrap(),
        Value::String(Rc::new("1.5".to_string()))
    );
    assert_eq!(
        convert::convert_string(&[Value::Float(2.0)]).unwrap(),
        Value::String(Rc::new("2.0".to_string()))
    );
    assert_eq!(
        convert::convert_string(&[Value::Bool(false)]).unwrap(),
        Value::String(Rc::new("false".to_string()))
    );
    assert_eq!(
        convert::convert_string(&[Value::None]).unwrap(),
        Value::String(Rc::new("none".to_string()))
    );

    // No magic stringification for collections.
    let list = Value::List(Rc::new(RefCell::new(vec![Value::Int(1)])));
    let err = convert::convert_string(&[list]).unwrap_err();
    assert!(matches!(err, aipo_vm::VmFault::TypeMismatch { .. }));
}

#[test]
fn test_vm_execution_with_prelude_conversion() {
    let mut vm = Vm::new();
    let mut registry = NativeRegistry::new();
    register_stdlib(&mut vm, &mut registry);

    // Bytecode: GetGlobal "Int"; Constant "8080"; Call 1; Return
    let mut module = BytecodeModule::new();
    module.names.push("Int".to_string());
    let c0 = module.constants.len() as u16;
    module.constants.push(Constant::String("8080".to_string()));

    module.code.push(OpCode::GetGlobal as u8);
    module.code.extend_from_slice(&0u16.to_be_bytes());
    module.code.push(OpCode::Constant as u8);
    module.code.extend_from_slice(&c0.to_be_bytes());
    module.code.push(OpCode::Call as u8);
    module.code.push(1);
    module.code.push(OpCode::Return as u8);

    let res = vm.run(&module).unwrap();
    assert_eq!(res, Value::Int(8080));
}

#[test]
fn test_registry_covers_new_surface() {
    let mut vm = Vm::new();
    let mut registry = NativeRegistry::new();
    register_stdlib(&mut vm, &mut registry);

    for name in [
        "Int", "Float", "Byte", "String", "len", "copy", "some", "fail",
    ] {
        assert!(registry.get(None, name).is_some(), "missing prelude {name}");
    }

    for name in ["truncate", "clamp"] {
        assert!(
            registry.get(Some("math"), name).is_some(),
            "missing math.{name}"
        );
    }

    for name in ["byte_len", "find", "capitalize", "reverse", "format"] {
        assert!(
            registry.get(Some("string"), name).is_some(),
            "missing string.{name}"
        );
    }
}

#[derive(Default, Clone)]
struct BufferSink(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for BufferSink {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_io_sink_capture() {
    let buf = Arc::new(Mutex::new(Vec::new()));
    let sink = BufferSink(Arc::clone(&buf));
    io::set_output_sink(Some(Box::new(sink)));

    io::io_print(&[Value::String(Rc::new("Hello".to_string()))]).unwrap();
    io::io_println(&[Value::String(Rc::new(" World".to_string()))]).unwrap();

    let output = String::from_utf8(buf.lock().unwrap().clone()).unwrap();
    assert_eq!(output, "Hello World\n");

    // Clean up sink
    io::set_output_sink(None);
}

#[test]
fn test_vm_execution_with_stdlib_prelude() {
    let mut vm = Vm::new();
    let mut registry = NativeRegistry::new();
    register_stdlib(&mut vm, &mut registry);

    // Bytecode program:
    // 1. GetGlobal "len"
    // 2. Constant "apple"
    // 3. Constant "banana"
    // 4. BuildList 2
    // 5. Call 1
    // 6. Return
    let mut module = BytecodeModule::new();
    module.names.push("len".to_string());
    #[allow(clippy::cast_possible_truncation)]
    let c0 = module.constants.len() as u16;
    module.constants.push(Constant::String("apple".to_string()));
    #[allow(clippy::cast_possible_truncation)]
    let c1 = module.constants.len() as u16;
    module
        .constants
        .push(Constant::String("banana".to_string()));

    // GetGlobal "len" (name index 0)
    module.code.push(OpCode::GetGlobal as u8);
    module.code.extend_from_slice(&0u16.to_be_bytes());

    // Push constant "apple"
    module.code.push(OpCode::Constant as u8);
    module.code.extend_from_slice(&c0.to_be_bytes());

    // Push constant "banana"
    module.code.push(OpCode::Constant as u8);
    module.code.extend_from_slice(&c1.to_be_bytes());

    // BuildList 2
    module.code.push(OpCode::BuildList as u8);
    module.code.extend_from_slice(&2u16.to_be_bytes());

    // Call 1
    module.code.push(OpCode::Call as u8);
    module.code.push(1);

    // Return
    module.code.push(OpCode::Return as u8);

    let res = vm.run(&module).unwrap();
    assert_eq!(res, Value::Int(2));
}

#[test]
fn test_vm_execution_with_stdlib_math_module() {
    let mut vm = Vm::new();
    let mut registry = NativeRegistry::new();
    register_stdlib(&mut vm, &mut registry);

    // Bytecode program:
    // 1. GetGlobal "math"
    // 2. GetField "abs"
    // 3. Constant -99
    // 4. Call 1
    // 5. Return
    let mut module = BytecodeModule::new();
    module.names.push("math".to_string());
    module.names.push("abs".to_string());
    #[allow(clippy::cast_possible_truncation)]
    let c0 = module.constants.len() as u16;
    module.constants.push(Constant::Int(-99));

    // GetGlobal "math" (name index 0)
    module.code.push(OpCode::GetGlobal as u8);
    module.code.extend_from_slice(&0u16.to_be_bytes());

    // GetField "abs" (name index 1)
    module.code.push(OpCode::GetField as u8);
    module.code.extend_from_slice(&1u16.to_be_bytes());

    // Constant -99
    module.code.push(OpCode::Constant as u8);
    module.code.extend_from_slice(&c0.to_be_bytes());

    // Call 1
    module.code.push(OpCode::Call as u8);
    module.code.push(1);

    // Return
    module.code.push(OpCode::Return as u8);

    let res = vm.run(&module).unwrap();
    assert_eq!(res, Value::Int(99));
}
