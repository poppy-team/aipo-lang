//! Standard library for the Aipo programming language.
//!
//! Provides the canonical Prelude V1 and standard library modules (`math`, `string`, `io`),
//! registering their native functions and module dictionaries into the Aipo VM and NativeRegistry.
//!
//! See `docs/stdlib/mvp-subset.md` for the closed V1 surface implemented here and
//! `docs/adp/ADP-001-byte-and-core-types-as-values.md` for the recorded representation gaps.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod bytes;
pub mod collections;
pub mod convert;
pub mod duration;
pub mod io;
pub mod math;
pub mod prelude;
pub mod string;
pub mod task;

use aipo_runtime::{NativeFunctionMeta, NativeRegistry};
use aipo_vm::{TypeTag, Value, Vm, VmFault};

/// Registers the canonical Prelude V1 and all standard library modules into the VM and NativeRegistry.
pub fn register_stdlib(vm: &mut Vm, registry: &mut NativeRegistry) {
    register_prelude(vm, registry);
    register_math(registry);
    register_string(registry);
    register_io(registry);
    register_task(registry);
    register_modules(vm);
    register_methods(vm);
}

/// Binds a module-style function as a receiver-first method.
///
/// Canon spells the module forms with the receiver in the first position
/// (`string.contains(text, sub)`), so a method is just the same native with the
/// receiver prepended: `text.contains(sub)`. Generating the adapter keeps the two
/// spellings on exactly one implementation.
macro_rules! receiver_first {
    ($wrapper:ident, $target:path) => {
        fn $wrapper(receiver: &Value, args: &[Value]) -> Result<Value, VmFault> {
            let mut all: Vec<Value> = Vec::with_capacity(args.len() + 1);
            all.push(receiver.clone());
            all.extend_from_slice(args);
            $target(&all)
        }
    };
}

receiver_first!(method_string_len, string::string_len);
receiver_first!(method_string_byte_len, string::string_byte_len);
receiver_first!(method_string_contains, string::string_contains);
receiver_first!(method_string_starts_with, string::string_starts_with);
receiver_first!(method_string_ends_with, string::string_ends_with);
receiver_first!(method_string_find, string::string_find);
receiver_first!(method_string_lower, string::string_lower);
receiver_first!(method_string_upper, string::string_upper);
receiver_first!(method_string_capitalize, string::string_capitalize);
receiver_first!(method_string_reverse, string::string_reverse);
receiver_first!(method_string_trim, string::string_trim);
receiver_first!(method_string_split, string::string_split);
receiver_first!(method_string_join, string::string_join);
receiver_first!(method_string_replace, string::string_replace);
receiver_first!(method_string_slice, string::string_slice);
receiver_first!(method_string_format, string::string_format);

/// Registers dot-call sugar for the core types so `text.upper()` and `list.len()` resolve.
///
/// Mutation during iteration is enforced by the VM, not here, so every mutating method
/// registered below is rejected with `MutationDuringIteration` while its receiver is
/// inside an active `each` or higher-order iteration.
fn register_methods(vm: &mut Vm) {
    vm.register_method_native("String", "len", 0, method_string_len);
    vm.register_method_native("String", "byte_len", 0, method_string_byte_len);
    vm.register_method_native("String", "contains", 1, method_string_contains);
    vm.register_method_native("String", "starts_with", 1, method_string_starts_with);
    vm.register_method_native("String", "ends_with", 1, method_string_ends_with);
    vm.register_method_native("String", "find", 1, method_string_find);
    vm.register_method_native("String", "lower", 0, method_string_lower);
    vm.register_method_native("String", "upper", 0, method_string_upper);
    vm.register_method_native("String", "capitalize", 0, method_string_capitalize);
    vm.register_method_native("String", "reverse", 0, method_string_reverse);
    vm.register_method_native("String", "trim", 0, method_string_trim);
    vm.register_method_native("String", "split", 1, method_string_split);
    vm.register_method_native("String", "join", 1, method_string_join);
    vm.register_method_native("String", "replace", 2, method_string_replace);
    vm.register_method_native("String", "slice", 2, method_string_slice);
    vm.register_method_native("String", "format", 1, method_string_format);

    collections::register_methods(vm);
    bytes::register_methods(vm);
    duration::register_methods(vm);
}

fn native(
    name: &str,
    arity: usize,
    func: fn(&[Value]) -> Result<Value, aipo_vm::VmFault>,
) -> Value {
    Value::Native {
        name: name.to_string(),
        arity,
        func,
    }
}

/// Registers Prelude V1 metadata (including core-type conversions) and globally available values.
fn register_prelude(vm: &mut Vm, registry: &mut NativeRegistry) {
    registry.register(NativeFunctionMeta::new(
        "len",
        1,
        None,
        "Computes the length of a String, List, or Dict.",
    ));
    registry.register(NativeFunctionMeta::new(
        "copy",
        1,
        None,
        "Shallow copies a collection or struct.",
    ));
    registry.register(NativeFunctionMeta::new(
        "same",
        2,
        None,
        "Checks reference identity for references or equality for primitives.",
    ));
    registry.register(NativeFunctionMeta::new(
        "some",
        1,
        None,
        "Checks whether a value is not none.",
    ));
    registry.register(NativeFunctionMeta::new(
        "fail",
        1,
        None,
        "Creates a recoverable Failure value.",
    ));
    registry.register(NativeFunctionMeta::new(
        "Int",
        1,
        None,
        "Explicit conversion to Int (Float truncates toward zero, String is parsed).",
    ));
    registry.register(NativeFunctionMeta::new(
        "Float",
        1,
        None,
        "Explicit conversion to Float (Int widens, String is parsed).",
    ));
    registry.register(NativeFunctionMeta::new(
        "Byte",
        1,
        None,
        "Explicit, verified conversion to Byte in 0..=255.",
    ));
    registry.register(NativeFunctionMeta::new(
        "String",
        1,
        None,
        "Canonical textual conversion for String, Int, Float, Bool, and none.",
    ));

    vm.define_global("none", Value::None);
    vm.define_global("true", Value::Bool(true));
    vm.define_global("false", Value::Bool(false));

    vm.define_global("len", native("len", 1, prelude::native_len));
    vm.define_global("copy", native("copy", 1, prelude::native_copy));
    vm.define_global("same", native("same", 2, prelude::native_same));
    vm.define_global("some", native("some", 1, prelude::native_some));
    vm.define_global("fail", native("fail", 1, prelude::native_fail));

    // The conversion forms are first-class type values: `Int` is a value that can be
    // called, compared, passed around and used with `is`, all sharing one dispatch.
    vm.define_global("Int", Value::Type(TypeTag::Int));
    vm.define_global("Float", Value::Type(TypeTag::Float));
    vm.define_global("Byte", Value::Type(TypeTag::Byte));
    vm.define_global("String", Value::Type(TypeTag::String));
    vm.define_global("Bool", Value::Type(TypeTag::Bool));
    vm.define_global("List", Value::Type(TypeTag::List));
    vm.define_global("Dict", Value::Type(TypeTag::Dict));
    vm.define_global("Bytes", Value::Type(TypeTag::Bytes));
    vm.define_global("Set", Value::Type(TypeTag::Set));
    vm.define_global("Duration", Value::Type(TypeTag::Duration));
}

/// Signature of a core-type conversion native.
pub type ConversionNative = fn(&[Value]) -> Result<Value, VmFault>;

/// Keeps the conversion natives reachable from Rust callers and tests.
#[must_use]
pub fn conversion_natives() -> [(&'static str, ConversionNative); 4] {
    [
        ("Int", convert::convert_int),
        ("Float", convert::convert_float),
        ("Byte", convert::convert_byte),
        ("String", convert::convert_string),
    ]
}

/// Registers `math` module metadata.
fn register_math(registry: &mut NativeRegistry) {
    registry.register(NativeFunctionMeta::new(
        "abs",
        1,
        Some("math"),
        "Computes absolute value.",
    ));
    registry.register(NativeFunctionMeta::new(
        "min",
        2,
        Some("math"),
        "Returns the minimum of two numbers.",
    ));
    registry.register(NativeFunctionMeta::new(
        "max",
        2,
        Some("math"),
        "Returns the maximum of two numbers.",
    ));
    registry.register(NativeFunctionMeta::new(
        "floor",
        1,
        Some("math"),
        "Floors number to integer.",
    ));
    registry.register(NativeFunctionMeta::new(
        "ceil",
        1,
        Some("math"),
        "Ceils number to integer.",
    ));
    registry.register(NativeFunctionMeta::new(
        "round",
        1,
        Some("math"),
        "Rounds number to nearest integer, half away from zero.",
    ));
    registry.register(NativeFunctionMeta::new(
        "truncate",
        1,
        Some("math"),
        "Truncates toward zero, returning Int when representable.",
    ));
    registry.register(NativeFunctionMeta::new(
        "sqrt",
        1,
        Some("math"),
        "Computes square root.",
    ));
    registry.register(NativeFunctionMeta::new(
        "pow",
        2,
        Some("math"),
        "Computes base raised to exponent.",
    ));
    registry.register(NativeFunctionMeta::new(
        "clamp",
        3,
        Some("math"),
        "Constrains a number to the inclusive [min, max] interval.",
    ));
}

/// Registers `string` module metadata.
fn register_string(registry: &mut NativeRegistry) {
    registry.register(NativeFunctionMeta::new(
        "len",
        1,
        Some("string"),
        "Computes string code point length.",
    ));
    registry.register(NativeFunctionMeta::new(
        "byte_len",
        1,
        Some("string"),
        "Computes the UTF-8 byte length of a string.",
    ));
    registry.register(NativeFunctionMeta::new(
        "contains",
        2,
        Some("string"),
        "Checks substring inclusion.",
    ));
    registry.register(NativeFunctionMeta::new(
        "starts_with",
        2,
        Some("string"),
        "Checks prefix match.",
    ));
    registry.register(NativeFunctionMeta::new(
        "ends_with",
        2,
        Some("string"),
        "Checks suffix match.",
    ));
    registry.register(NativeFunctionMeta::new(
        "find",
        2,
        Some("string"),
        "Returns the code point index of the first match, or none.",
    ));
    registry.register(NativeFunctionMeta::new(
        "lower",
        1,
        Some("string"),
        "Converts to lowercase.",
    ));
    registry.register(NativeFunctionMeta::new(
        "upper",
        1,
        Some("string"),
        "Converts to uppercase.",
    ));
    registry.register(NativeFunctionMeta::new(
        "capitalize",
        1,
        Some("string"),
        "Titlecases the first cased character and lowercases the rest.",
    ));
    registry.register(NativeFunctionMeta::new(
        "reverse",
        1,
        Some("string"),
        "Reverses extended grapheme clusters.",
    ));
    registry.register(NativeFunctionMeta::new(
        "trim",
        1,
        Some("string"),
        "Trims whitespace.",
    ));
    registry.register(NativeFunctionMeta::new(
        "split",
        2,
        Some("string"),
        "Splits string by a non-empty separator.",
    ));
    registry.register(NativeFunctionMeta::new(
        "join",
        2,
        Some("string"),
        "Joins a List of Strings with a separator.",
    ));
    registry.register(NativeFunctionMeta::new(
        "replace",
        3,
        Some("string"),
        "Replaces occurrences of a non-empty substring.",
    ));
    registry.register(NativeFunctionMeta::new(
        "slice",
        3,
        Some("string"),
        "Slices substring by character index range.",
    ));
    registry.register(NativeFunctionMeta::new(
        "format",
        2,
        Some("string"),
        "Substitutes named placeholders from a Dict into a runtime template.",
    ));
}

/// Registers `io` module metadata.
fn register_io(registry: &mut NativeRegistry) {
    registry.register(NativeFunctionMeta::new(
        "print",
        1,
        Some("io"),
        "Prints value without trailing newline.",
    ));
    registry.register(NativeFunctionMeta::new(
        "println",
        1,
        Some("io"),
        "Prints value with trailing newline.",
    ));
}

/// Registers `task` module metadata.
fn register_task(registry: &mut NativeRegistry) {
    registry.register(NativeFunctionMeta::new(
        "spawn",
        2,
        Some("task"),
        "Spawns a new asynchronous task.",
    ));
    registry.register(NativeFunctionMeta::new(
        "sleep",
        1,
        Some("task"),
        "Suspends current task until virtual clock reaches deadline.",
    ));
    registry.register(NativeFunctionMeta::new(
        "all",
        1,
        Some("task"),
        "Awaits completion of all tasks in a list.",
    ));
    registry.register(NativeFunctionMeta::new(
        "race",
        1,
        Some("task"),
        "Races tasks, returning the first result.",
    ));
    registry.register(NativeFunctionMeta::new(
        "timeout",
        2,
        Some("task"),
        "Awaits task with a virtual tick deadline.",
    ));
    registry.register(NativeFunctionMeta::new(
        "cancel",
        1,
        Some("task"),
        "Cancels an active task.",
    ));
    registry.register(NativeFunctionMeta::new(
        "group",
        0,
        Some("task"),
        "Creates a structured-concurrency task group.",
    ));
}

/// Defines the standard module dictionaries as VM globals.
fn register_modules(vm: &mut Vm) {
    vm.define_global("math", math::create_module());
    vm.define_global("string", string::create_module());
    vm.define_global("io", io::create_module());
    vm.define_global("task", task::create_module());
}
