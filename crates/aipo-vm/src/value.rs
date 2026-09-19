//! Value model for the Aipo VM runtime.

use crate::convert::TypeTag;
use crate::fault::VmFault;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;
use unicode_normalization::UnicodeNormalization;

/// Maximum safe integer in Aipo: 2^53 - 1.
pub const MAX_SAFE_INT: i64 = 9_007_199_254_740_991;
/// Minimum safe integer in Aipo: -(2^53 - 1).
pub const MIN_SAFE_INT: i64 = -9_007_199_254_740_991;

/// Validates that an integer is within the normative ±(2^53 - 1) boundary.
///
/// # Errors
/// Returns `VmFault::Overflow` if outside safe integer bounds.
pub fn check_safe_int(n: i64) -> Result<i64, VmFault> {
    if (MIN_SAFE_INT..=MAX_SAFE_INT).contains(&n) {
        Ok(n)
    } else {
        Err(VmFault::Overflow {
            details: format!("{n} exceeds integer range ±(2^53 - 1)"),
        })
    }
}

/// Validates that a float is finite (not NaN or Infinity).
///
/// # Errors
/// Returns `VmFault::NonFiniteFloat` if NaN or Infinite.
pub fn check_finite_float(f: f64) -> Result<f64, VmFault> {
    if f.is_finite() {
        Ok(f)
    } else {
        Err(VmFault::NonFiniteFloat)
    }
}

/// Recoverable failure payload (Model B).
#[derive(Debug, Clone, PartialEq)]
pub struct FailureValue {
    /// Human-readable explanation of the failure.
    pub message: String,
}

/// Instance of a user-defined struct.
#[derive(Debug, Clone, PartialEq)]
pub struct StructInstance {
    /// Name of the struct type.
    pub type_name: String,
    /// Ordered field name and value pairs.
    pub fields: Vec<(String, Value)>,
    /// Set of fields marked as `fixed` (immutable after initialization).
    pub fixed_fields: HashSet<String>,
    /// `true` while the instance is still being built by an `init` hook.
    ///
    /// Canon allows `fixed` fields to receive their value during construction — including from
    /// `init` — and forbids replacing them only after the instance is published. The flag is set
    /// when a deferred construction starts and cleared by [`StructInstance::seal`].
    pub under_construction: bool,
}

impl StructInstance {
    /// Retrieves a reference to a field's value.
    #[must_use]
    pub fn get_field(&self, name: &str) -> Option<&Value> {
        self.fields.iter().find(|(k, _)| k == name).map(|(_, v)| v)
    }

    /// Mutates an existing field value, validating `fixed` constraints.
    ///
    /// # Errors
    /// Returns `VmFault::FixedFieldMutation` if the field is fixed,
    /// or `VmFault::NoSuchField` if the field does not exist.
    pub fn set_field(&mut self, name: &str, new_value: Value) -> Result<(), VmFault> {
        if !self.under_construction && self.fixed_fields.contains(name) {
            return Err(VmFault::FixedFieldMutation {
                type_name: self.type_name.clone(),
                field: name.to_string(),
            });
        }
        if let Some((_, v)) = self.fields.iter_mut().find(|(k, _)| k == name) {
            *v = new_value;
            Ok(())
        } else {
            Err(VmFault::NoSuchField {
                type_name: self.type_name.clone(),
                field: name.to_string(),
            })
        }
    }

    /// Restores a field to a journaled value during an invariant rollback.
    ///
    /// Canon returns the direct protected fields of the participating instances to their
    /// entry values when a boundary validation fails, so a rollback bypasses the `fixed`
    /// guard: it writes back the value the field already held.
    pub fn restore_field(&mut self, name: &str, value: Value) {
        if let Some((_, slot)) = self.fields.iter_mut().find(|(k, _)| k == name) {
            *slot = value;
        }
    }

    /// Publishes the instance once construction finishes.
    ///
    /// Canon's `init` runs with `self` still in construction, so `fixed` fields are mutable
    /// until this point. Sealing restores the declared `fixed` set and clears the flag, which
    /// makes every later assignment to those fields a [`VmFault::FixedFieldMutation`].
    pub fn seal(&mut self, fixed_fields: HashSet<String>) {
        self.fixed_fields = fixed_fields;
        self.under_construction = false;
    }
}

/// Ordered dictionary preserving insertion order.
///
/// Entries live in a `Vec` so iteration order is stable and structural equality
/// stays order-sensitive. A secondary index accelerates the overwhelmingly common
/// case — lookup and upsert by `String` key — while every other key category
/// keeps the exact linear-scan semantics. The index only ever points at the
/// *first* matching entry, mirroring what a scan would find.
#[derive(Clone, Default)]
pub struct DictMap {
    entries: Vec<(Value, Value)>,
    str_index: HashMap<String, usize>,
}

impl fmt::Debug for DictMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.entries)
    }
}

impl PartialEq for DictMap {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }
}

impl FromIterator<(Value, Value)> for DictMap {
    fn from_iter<I: IntoIterator<Item = (Value, Value)>>(iter: I) -> Self {
        Self::from_entries(iter.into_iter().collect())
    }
}

impl DictMap {
    /// Builds a map from entries in order, indexing the first occurrence of
    /// each `String` key.
    #[must_use]
    pub fn from_entries(entries: Vec<(Value, Value)>) -> Self {
        let mut map = Self {
            entries,
            str_index: HashMap::new(),
        };
        map.rebuild_index();
        map
    }

    /// Number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the map holds no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Entries in insertion order.
    #[must_use]
    pub fn entries(&self) -> &[(Value, Value)] {
        &self.entries
    }

    /// Looks up a key: `String` keys use the index, every other category scans,
    /// exactly as a full scan would.
    #[must_use]
    pub fn get(&self, key: &Value) -> Option<&Value> {
        if let Value::String(s) = key {
            if let Some(&pos) = self.str_index.get(s.as_str()) {
                return self.entries.get(pos).map(|(_, v)| v);
            }
            return None;
        }
        self.entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Updates the first matching entry or pushes a new one, preserving order.
    pub fn upsert(&mut self, key: Value, value: Value) {
        if let Value::String(s) = &key {
            if let Some(&pos) = self.str_index.get(s.as_str()) {
                if let Some((_, v)) = self.entries.get_mut(pos) {
                    *v = value;
                    return;
                }
            }
        } else if let Some((_, v)) = self.entries.iter_mut().find(|(k, _)| k == &key) {
            *v = value;
            return;
        }
        let pos = self.entries.len();
        if let Value::String(s) = &key {
            self.str_index.entry(s.to_string()).or_insert(pos);
        }
        self.entries.push((key, value));
    }

    /// Removes the first matching entry, reporting whether one existed.
    pub fn remove(&mut self, key: &Value) -> bool {
        let pos = self.position(key);
        match pos {
            Some(at) => {
                self.entries.remove(at);
                self.rebuild_index();
                true
            }
            None => false,
        }
    }

    /// Removes all entries.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.str_index.clear();
    }

    /// Values in insertion order.
    #[must_use]
    pub fn values(&self) -> Vec<Value> {
        self.entries.iter().map(|(_, v)| v.clone()).collect()
    }

    /// Keys in insertion order.
    #[must_use]
    pub fn keys(&self) -> Vec<Value> {
        self.entries.iter().map(|(k, _)| k.clone()).collect()
    }

    fn position(&self, key: &Value) -> Option<usize> {
        if let Value::String(s) = key {
            self.str_index.get(s.as_str()).copied()
        } else {
            self.entries.iter().position(|(k, _)| k == key)
        }
    }

    fn rebuild_index(&mut self) {
        self.str_index.clear();
        for (pos, (key, _)) in self.entries.iter().enumerate() {
            if let Value::String(s) = key {
                self.str_index.entry(s.to_string()).or_insert(pos);
            }
        }
    }
}

/// Dynamic value manipulated by the Aipo VM stack machine.
///
/// Sharing (`List`, `Dict`, struct instances, closure upvalue cells) is
/// `Rc<RefCell<…>>`: the interpreter is single-threaded by decision (see the
/// `aipo-vm` crate contract on threading), so `Value` is intentionally `!Send`.
/// A concurrent host must keep one VM per thread or replace this primitive first.
#[derive(Clone)]
pub enum Value {
    /// Absence of a value (`none`).
    None,
    /// Boolean value (`true` or `false`).
    Bool(bool),
    /// 64-bit integer within ±(2^53 - 1).
    Int(i64),
    /// Finite 64-bit floating-point number.
    Float(f64),
    /// Immutable UTF-8 string.
    String(Rc<String>),
    /// Ordered, mutable list of values.
    List(Rc<RefCell<Vec<Value>>>),
    /// Ordered dictionary preserving insertion order.
    Dict(Rc<RefCell<DictMap>>),
    /// Instantiated user struct with field immutability tracking.
    Struct(Rc<RefCell<StructInstance>>),
    /// VM-managed bytecode function pointer.
    Function {
        /// Target entry point instruction offset.
        entry_ip: usize,
        /// Expected number of parameters.
        arity: usize,
    },
    /// Closure capturing upvalues.
    Closure {
        /// Entry instruction pointer.
        entry_ip: usize,
        /// Expected number of parameters.
        arity: usize,
        /// Captured upvalues.
        upvalues: Vec<Rc<RefCell<Value>>>,
    },
    /// Native host function callable by the VM.
    Native {
        /// Native function name.
        name: String,
        /// Expected argument count.
        arity: usize,
        /// Native function callback pointer.
        func: fn(&[Value]) -> Result<Value, VmFault>,
    },
    /// Compact integer in the canonical `Byte` range `0..=255`.
    Byte(u8),
    /// Immutable byte buffer (`Bytes`).
    Bytes(Rc<Vec<u8>>),
    /// Fundamental type value usable as a runtime `is` target and, for the
    /// convertible core types, as a callable conversion.
    Type(TypeTag),
    /// Half-open integer range produced by `a..b`.
    Range {
        /// Inclusive start.
        start: i64,
        /// Exclusive end.
        end: i64,
    },
    /// Method already bound to its receiver, produced by field access on a
    /// collection, string or struct instance.
    BoundMethod {
        /// Method name.
        name: String,
        /// Arguments expected after the receiver.
        arity: usize,
        /// Receiver value the method operates on.
        receiver: Box<Value>,
        /// How the method is executed.
        kind: MethodKind,
    },
    /// Recoverable failure (Model B).
    Failure(Rc<FailureValue>),
    /// Internal sentinel standing for a defaulted parameter the caller omitted.
    ///
    /// Canon evaluates parameter defaults inside the callee, so the value travels from the
    /// call site to the callee prologue as a marker rather than as a value. No Aipo program
    /// can observe it: the prologue replaces every marker it can reach before the body runs.
    Unset,
}

/// Execution strategy of a bound method.
#[derive(Clone)]
pub enum MethodKind {
    /// Receiver-first native implementation (`fn(&Value, &[Value])`).
    Native(fn(&Value, &[Value]) -> Result<Value, VmFault>),
    /// User function defined in the module, called with the receiver as argument 0.
    Function {
        /// Bytecode entry offset of the method body.
        entry_ip: usize,
        /// Parameter count including the receiver.
        total_arity: usize,
    },
    /// Higher-order method the VM executes itself so it can call back into Aipo code.
    HigherOrder,
}

impl Value {
    /// Returns the language-level type name for diagnostics.
    #[must_use]
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Bool(_) => "Bool",
            Self::Int(_) => "Int",
            Self::Float(_) => "Float",
            Self::String(_) => "String",
            Self::List(_) => "List",
            Self::Dict(_) => "Dict",
            Self::Struct(_) => "struct",
            Self::Function { .. }
            | Self::Closure { .. }
            | Self::Native { .. }
            | Self::BoundMethod { .. } => "Function",
            Self::Byte(_) => "Byte",
            Self::Bytes(_) => "Bytes",
            Self::Type(_) => "Type",
            Self::Range { .. } => "Range",
            Self::Failure(_) => "Failure",
            // Internal sentinel: never observable from Aipo, so the name only ever
            // appears in an invariant-violation message.
            Self::Unset => "<omitted argument>",
        }
    }

    /// Numeric projection used for sort-key comparison; `None` for non-numeric values.
    #[must_use]
    pub(crate) fn numeric_key(&self) -> Option<f64> {
        match self {
            Self::Int(n) =>
            {
                #[allow(clippy::cast_precision_loss)]
                Some(*n as f64)
            }
            Self::Byte(b) => Some(f64::from(*b)),
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Widens `Byte` to `Int` so numeric operators can treat it as an integer.
    #[must_use]
    fn numeric(&self) -> Self {
        match self {
            Self::Byte(b) => Self::Int(i64::from(*b)),
            other => other.clone(),
        }
    }

    /// Returns `true` when either operand is a `Byte`, so operators can widen first.
    fn needs_byte_widening(&self, other: &Self) -> bool {
        matches!(self, Self::Byte(_)) || matches!(other, Self::Byte(_))
    }

    /// Checks if value is a recoverable failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Failure(_))
    }

    /// Extracts boolean value, returning `TypeMismatch` if not a Bool.
    ///
    /// # Errors
    /// Returns `VmFault::TypeMismatch` if self is not `Value::Bool`.
    pub fn as_bool(&self) -> Result<bool, VmFault> {
        match self {
            Self::Bool(b) => Ok(*b),
            other => Err(VmFault::TypeMismatch {
                expected: "Bool".to_string(),
                actual: other.type_name().to_string(),
            }),
        }
    }

    /// Binary addition `+`.
    ///
    /// # Errors
    /// Returns `VmFault` on overflow, non-finite float, or incompatible operand types.
    pub fn add(&self, other: &Self) -> Result<Self, VmFault> {
        if self.is_failure() {
            return Ok(self.clone());
        }
        if other.is_failure() {
            return Ok(other.clone());
        }
        if self.needs_byte_widening(other) {
            return self.numeric().add(&other.numeric());
        }

        match (self, other) {
            (Self::Int(a), Self::Int(b)) => {
                let res = a.checked_add(*b).ok_or_else(|| VmFault::Overflow {
                    details: format!("{a} + {b} exceeded i64 representation"),
                })?;
                Ok(Self::Int(check_safe_int(res)?))
            }
            (Self::Float(a), Self::Float(b)) => {
                let res = check_finite_float(a + b)?;
                Ok(Self::Float(res))
            }
            (Self::Int(a), Self::Float(b)) => {
                #[allow(clippy::cast_precision_loss)]
                let res = check_finite_float(*a as f64 + b)?;
                Ok(Self::Float(res))
            }
            (Self::Float(a), Self::Int(b)) => {
                #[allow(clippy::cast_precision_loss)]
                let res = check_finite_float(a + *b as f64)?;
                Ok(Self::Float(res))
            }
            (Self::String(a), Self::String(b)) => {
                // Canon makes NFC an invariant of `String`, and concatenation is the operation
                // that can join a base character with a combining mark (interpolation lowers to
                // `+`), so the result is re-normalized before the program can observe it.
                let combined: String = a.chars().chain(b.chars()).nfc().collect();
                Ok(Self::String(Rc::new(combined)))
            }
            (Self::List(a), Self::List(b)) => {
                let mut items = a.borrow().clone();
                items.extend(b.borrow().iter().cloned());
                Ok(Self::List(Rc::new(RefCell::new(items))))
            }
            _ => Err(VmFault::TypeMismatch {
                expected: "Int, Float, String, or List".to_string(),
                actual: format!("{} and {}", self.type_name(), other.type_name()),
            }),
        }
    }

    /// Binary subtraction `-`.
    ///
    /// # Errors
    /// Returns `VmFault` on overflow, non-finite float, or incompatible operand types.
    pub fn sub(&self, other: &Self) -> Result<Self, VmFault> {
        if self.is_failure() {
            return Ok(self.clone());
        }
        if other.is_failure() {
            return Ok(other.clone());
        }
        if self.needs_byte_widening(other) {
            return self.numeric().sub(&other.numeric());
        }

        match (self, other) {
            (Self::Int(a), Self::Int(b)) => {
                let res = a.checked_sub(*b).ok_or_else(|| VmFault::Overflow {
                    details: format!("{a} - {b} exceeded i64 representation"),
                })?;
                Ok(Self::Int(check_safe_int(res)?))
            }
            (Self::Float(a), Self::Float(b)) => {
                let res = check_finite_float(a - b)?;
                Ok(Self::Float(res))
            }
            (Self::Int(a), Self::Float(b)) => {
                #[allow(clippy::cast_precision_loss)]
                let res = check_finite_float(*a as f64 - b)?;
                Ok(Self::Float(res))
            }
            (Self::Float(a), Self::Int(b)) => {
                #[allow(clippy::cast_precision_loss)]
                let res = check_finite_float(a - *b as f64)?;
                Ok(Self::Float(res))
            }
            _ => Err(VmFault::TypeMismatch {
                expected: "Int or Float".to_string(),
                actual: format!("{} and {}", self.type_name(), other.type_name()),
            }),
        }
    }

    /// Binary multiplication `*`.
    ///
    /// # Errors
    /// Returns `VmFault` on overflow, non-finite float, or incompatible operand types.
    pub fn mul(&self, other: &Self) -> Result<Self, VmFault> {
        if self.is_failure() {
            return Ok(self.clone());
        }
        if other.is_failure() {
            return Ok(other.clone());
        }
        if self.needs_byte_widening(other) {
            return self.numeric().mul(&other.numeric());
        }

        match (self, other) {
            (Self::Int(a), Self::Int(b)) => {
                let res = a.checked_mul(*b).ok_or_else(|| VmFault::Overflow {
                    details: format!("{a} * {b} exceeded i64 representation"),
                })?;
                Ok(Self::Int(check_safe_int(res)?))
            }
            (Self::Float(a), Self::Float(b)) => {
                let res = check_finite_float(a * b)?;
                Ok(Self::Float(res))
            }
            (Self::Int(a), Self::Float(b)) => {
                #[allow(clippy::cast_precision_loss)]
                let res = check_finite_float(*a as f64 * b)?;
                Ok(Self::Float(res))
            }
            (Self::Float(a), Self::Int(b)) => {
                #[allow(clippy::cast_precision_loss)]
                let res = check_finite_float(a * *b as f64)?;
                Ok(Self::Float(res))
            }
            _ => Err(VmFault::TypeMismatch {
                expected: "Int or Float".to_string(),
                actual: format!("{} and {}", self.type_name(), other.type_name()),
            }),
        }
    }

    /// Binary float division `/`. Always produces a Float.
    ///
    /// # Errors
    /// Returns `VmFault::DivisionByZero` or `VmFault::NonFiniteFloat`.
    pub fn div(&self, other: &Self) -> Result<Self, VmFault> {
        if self.is_failure() {
            return Ok(self.clone());
        }
        if other.is_failure() {
            return Ok(other.clone());
        }
        if self.needs_byte_widening(other) {
            return self.numeric().div(&other.numeric());
        }

        #[allow(clippy::cast_precision_loss)]
        let (a, b) = match (self, other) {
            (Self::Int(a), Self::Int(b)) => (*a as f64, *b as f64),
            (Self::Float(a), Self::Float(b)) => (*a, *b),
            (Self::Int(a), Self::Float(b)) => (*a as f64, *b),
            (Self::Float(a), Self::Int(b)) => (*a, *b as f64),
            _ => {
                return Err(VmFault::TypeMismatch {
                    expected: "Int or Float".to_string(),
                    actual: format!("{} and {}", self.type_name(), other.type_name()),
                });
            }
        };

        if b == 0.0 {
            return Err(VmFault::DivisionByZero);
        }

        let res = check_finite_float(a / b)?;
        Ok(Self::Float(res))
    }

    /// Binary integer truncated division `div`.
    ///
    /// # Errors
    /// Returns `VmFault::DivisionByZero` if divisor is 0, or `VmFault::Overflow`.
    pub fn int_div(&self, other: &Self) -> Result<Self, VmFault> {
        if self.is_failure() {
            return Ok(self.clone());
        }
        if other.is_failure() {
            return Ok(other.clone());
        }
        if self.needs_byte_widening(other) {
            return self.numeric().int_div(&other.numeric());
        }

        match (self, other) {
            (Self::Int(a), Self::Int(b)) => {
                if *b == 0 {
                    return Err(VmFault::DivisionByZero);
                }
                let res = a.checked_div(*b).ok_or_else(|| VmFault::Overflow {
                    details: format!("{a} div {b} overflowed"),
                })?;
                Ok(Self::Int(check_safe_int(res)?))
            }
            _ => Err(VmFault::TypeMismatch {
                expected: "Int".to_string(),
                actual: format!("{} and {}", self.type_name(), other.type_name()),
            }),
        }
    }

    /// Binary modulo `%`.
    ///
    /// # Errors
    /// Returns `VmFault::DivisionByZero` if divisor is 0.
    pub fn modulo(&self, other: &Self) -> Result<Self, VmFault> {
        if self.is_failure() {
            return Ok(self.clone());
        }
        if other.is_failure() {
            return Ok(other.clone());
        }
        if self.needs_byte_widening(other) {
            return self.numeric().modulo(&other.numeric());
        }

        match (self, other) {
            (Self::Int(a), Self::Int(b)) => {
                if *b == 0 {
                    return Err(VmFault::DivisionByZero);
                }
                let res = a.checked_rem(*b).ok_or_else(|| VmFault::Overflow {
                    details: format!("{a} % {b} overflowed"),
                })?;
                Ok(Self::Int(check_safe_int(res)?))
            }
            _ => Err(VmFault::TypeMismatch {
                expected: "Int".to_string(),
                actual: format!("{} and {}", self.type_name(), other.type_name()),
            }),
        }
    }

    /// Unary numeric negation `-`.
    ///
    /// # Errors
    /// Returns `VmFault::Overflow` if integer negates beyond safe bounds.
    pub fn negate(&self) -> Result<Self, VmFault> {
        if self.is_failure() {
            return Ok(self.clone());
        }
        if matches!(self, Self::Byte(_)) {
            return self.numeric().negate();
        }

        match self {
            Self::Int(a) => {
                let res = a.checked_neg().ok_or_else(|| VmFault::Overflow {
                    details: format!("negation of {a} overflowed"),
                })?;
                Ok(Self::Int(check_safe_int(res)?))
            }
            Self::Float(a) => Ok(Self::Float(check_finite_float(-a)?)),
            _ => Err(VmFault::TypeMismatch {
                expected: "Int or Float".to_string(),
                actual: self.type_name().to_string(),
            }),
        }
    }

    /// Logical NOT `not`.
    ///
    /// # Errors
    /// Returns `VmFault::TypeMismatch` if operand is not Bool.
    pub fn not(&self) -> Result<Self, VmFault> {
        if self.is_failure() {
            return Ok(self.clone());
        }
        let b = self.as_bool()?;
        Ok(Self::Bool(!b))
    }

    /// Equality comparison `==`.
    ///
    /// # Errors
    /// Propagates failure if either operand is Failure.
    pub fn equal(&self, other: &Self) -> Result<Self, VmFault> {
        if self.is_failure() {
            return Ok(self.clone());
        }
        if other.is_failure() {
            return Ok(other.clone());
        }
        Ok(Self::Bool(self == other))
    }

    /// Inequality comparison `!=`.
    ///
    /// # Errors
    /// Propagates failure if either operand is Failure.
    pub fn not_equal(&self, other: &Self) -> Result<Self, VmFault> {
        if self.is_failure() {
            return Ok(self.clone());
        }
        if other.is_failure() {
            return Ok(other.clone());
        }
        Ok(Self::Bool(self != other))
    }

    /// Relational comparison `<`.
    ///
    /// # Errors
    /// Returns `VmFault::TypeMismatch` if operands cannot be ordered.
    pub fn less(&self, other: &Self) -> Result<Self, VmFault> {
        if self.is_failure() {
            return Ok(self.clone());
        }
        if other.is_failure() {
            return Ok(other.clone());
        }
        if self.needs_byte_widening(other) {
            return self.numeric().less(&other.numeric());
        }

        match (self, other) {
            (Self::Int(a), Self::Int(b)) => Ok(Self::Bool(a < b)),
            (Self::Float(a), Self::Float(b)) => Ok(Self::Bool(a < b)),
            #[allow(clippy::cast_precision_loss)]
            (Self::Int(a), Self::Float(b)) => Ok(Self::Bool((*a as f64) < *b)),
            #[allow(clippy::cast_precision_loss)]
            (Self::Float(a), Self::Int(b)) => Ok(Self::Bool(*a < (*b as f64))),
            (Self::String(a), Self::String(b)) => Ok(Self::Bool(a < b)),
            _ => Err(VmFault::TypeMismatch {
                expected: "comparable Int, Float, or String".to_string(),
                actual: format!("{} and {}", self.type_name(), other.type_name()),
            }),
        }
    }

    /// Relational comparison `<=`.
    ///
    /// # Errors
    /// Returns `VmFault::TypeMismatch` if operands cannot be ordered.
    pub fn less_equal(&self, other: &Self) -> Result<Self, VmFault> {
        if self.is_failure() {
            return Ok(self.clone());
        }
        if other.is_failure() {
            return Ok(other.clone());
        }
        if self.needs_byte_widening(other) {
            return self.numeric().less_equal(&other.numeric());
        }

        match (self, other) {
            (Self::Int(a), Self::Int(b)) => Ok(Self::Bool(a <= b)),
            (Self::Float(a), Self::Float(b)) => Ok(Self::Bool(a <= b)),
            #[allow(clippy::cast_precision_loss)]
            (Self::Int(a), Self::Float(b)) => Ok(Self::Bool((*a as f64) <= *b)),
            #[allow(clippy::cast_precision_loss)]
            (Self::Float(a), Self::Int(b)) => Ok(Self::Bool(*a <= (*b as f64))),
            (Self::String(a), Self::String(b)) => Ok(Self::Bool(a <= b)),
            _ => Err(VmFault::TypeMismatch {
                expected: "comparable Int, Float, or String".to_string(),
                actual: format!("{} and {}", self.type_name(), other.type_name()),
            }),
        }
    }

    /// Relational comparison `>`.
    ///
    /// # Errors
    /// Returns `VmFault::TypeMismatch` if operands cannot be ordered.
    pub fn greater(&self, other: &Self) -> Result<Self, VmFault> {
        other.less(self)
    }

    /// Relational comparison `>=`.
    ///
    /// # Errors
    /// Returns `VmFault::TypeMismatch` if operands cannot be ordered.
    pub fn greater_equal(&self, other: &Self) -> Result<Self, VmFault> {
        other.less_equal(self)
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a == b,
            #[allow(clippy::cast_precision_loss)]
            (Self::Int(a), Self::Float(b)) => (*a as f64) == *b,
            #[allow(clippy::cast_precision_loss)]
            (Self::Float(a), Self::Int(b)) => *a == (*b as f64),
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Byte(a), Self::Byte(b)) => a == b,
            (Self::Byte(a), Self::Int(b)) => i64::from(*a) == *b,
            (Self::Int(a), Self::Byte(b)) => *a == i64::from(*b),
            #[allow(clippy::cast_precision_loss)]
            (Self::Byte(a), Self::Float(b)) => f64::from(*a) == *b,
            #[allow(clippy::cast_precision_loss)]
            (Self::Float(a), Self::Byte(b)) => *a == f64::from(*b),
            (Self::Bytes(a), Self::Bytes(b)) => a == b,
            (Self::Type(a), Self::Type(b)) => a == b,
            (Self::Range { start: s1, end: e1 }, Self::Range { start: s2, end: e2 }) => {
                s1 == s2 && e1 == e2
            }
            (
                Self::BoundMethod {
                    name: n1,
                    arity: a1,
                    receiver: r1,
                    ..
                },
                Self::BoundMethod {
                    name: n2,
                    arity: a2,
                    receiver: r2,
                    ..
                },
            ) => n1 == n2 && a1 == a2 && r1 == r2,
            (Self::List(a), Self::List(b)) => *a.borrow() == *b.borrow(),
            (Self::Dict(a), Self::Dict(b)) => *a.borrow() == *b.borrow(),
            (Self::Struct(a), Self::Struct(b)) => {
                let a = a.borrow();
                let b = b.borrow();
                a.type_name == b.type_name && a.fields == b.fields
            }
            (
                Self::Native {
                    name: n1,
                    arity: a1,
                    ..
                },
                Self::Native {
                    name: n2,
                    arity: a2,
                    ..
                },
            ) => n1 == n2 && a1 == a2,
            (Self::Failure(a), Self::Failure(b)) => a.message == b.message,
            _ => false,
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Int(n) => write!(f, "{n}"),
            Self::Float(n) => write!(f, "{n:?}"),
            Self::String(s) => write!(f, "{s:?}"),
            Self::List(l) => write!(f, "{:?}", l.borrow()),
            Self::Dict(d) => write!(f, "{:?}", d.borrow()),
            Self::Struct(s) => {
                let s = s.borrow();
                write!(f, "{}{:?}", s.type_name, s.fields)
            }
            Self::Function { entry_ip, arity } => {
                write!(f, "<fn@{entry_ip} arity={arity}>")
            }
            Self::Closure {
                entry_ip, arity, ..
            } => {
                write!(f, "<closure@{entry_ip} arity={arity}>")
            }
            Self::Native { name, arity, .. } => {
                write!(f, "<native fn {name} arity={arity}>")
            }
            Self::Byte(b) => write!(f, "{b}"),
            Self::Bytes(b) => write!(f, "Bytes({} bytes)", b.len()),
            Self::Type(tag) => write!(f, "<type {}>", tag.name()),
            Self::Range { start, end } => write!(f, "{start}..{end}"),
            Self::BoundMethod { name, arity, .. } => {
                write!(f, "<method {name} arity={arity}>")
            }
            Self::Failure(err) => write!(f, "failure({:?})", err.message),
            Self::Unset => write!(f, "<unset>"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Bool(b) => write!(f, "{b}"),
            Self::Int(n) => write!(f, "{n}"),
            Self::Float(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{n:.1}")
                } else {
                    write!(f, "{n}")
                }
            }
            Self::String(s) => write!(f, "{s}"),
            Self::List(l) => {
                let l = l.borrow();
                write!(f, "[")?;
                for (i, v) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Self::Dict(d) => {
                let d = d.borrow();
                write!(f, "#{{")?;
                for (i, (k, v)) in d.entries().iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "}}")
            }
            Self::Struct(s) => {
                let s = s.borrow();
                write!(f, "{}{{", s.type_name)?;
                for (i, (k, v)) in s.fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k}: {v}")?;
                }
                write!(f, "}}")
            }
            Self::Function { entry_ip, .. } => write!(f, "<fn@{entry_ip}>"),
            Self::Closure { entry_ip, .. } => write!(f, "<closure@{entry_ip}>"),
            Self::Native { name, .. } => write!(f, "<fn {name}>"),
            Self::Byte(b) => write!(f, "{b}"),
            Self::Bytes(_) => write!(f, "<bytes>"),
            Self::Type(tag) => write!(f, "{}", tag.name()),
            Self::Range { start, end } => write!(f, "{start}..{end}"),
            Self::BoundMethod { name, .. } => write!(f, "<fn {name}>"),
            Self::Failure(err) => write!(f, "fail(\"{}\")", err.message),
            Self::Unset => write!(f, "<unset>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn str_key(name: &str) -> Value {
        Value::String(Rc::new(name.to_string()))
    }

    #[test]
    fn test_dict_map_string_lookup_uses_index() {
        let map = DictMap::from_entries(vec![
            (str_key("b"), Value::Int(2)),
            (str_key("a"), Value::Int(1)),
        ]);
        assert_eq!(map.get(&str_key("a")), Some(&Value::Int(1)));
        assert_eq!(map.get(&str_key("missing")), None);
        assert_eq!(map.keys(), vec![str_key("b"), str_key("a")]);
    }

    #[test]
    fn test_dict_map_non_string_keys_scan() {
        let map = DictMap::from_entries(vec![
            (Value::Int(1), str_key("one")),
            (
                Value::List(Rc::new(RefCell::new(vec![Value::Int(2)]))),
                str_key("two"),
            ),
        ]);
        assert_eq!(map.get(&Value::Int(1)), Some(&str_key("one")));
        assert_eq!(map.get(&Value::Int(9)), None);
    }

    #[test]
    fn test_dict_map_upsert_preserves_first_match_and_order() {
        let mut map = DictMap::from_entries(vec![(str_key("a"), Value::Int(1))]);
        map.upsert(str_key("b"), Value::Int(2));
        map.upsert(str_key("a"), Value::Int(10));
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&str_key("a")), Some(&Value::Int(10)));
        assert_eq!(map.keys(), vec![str_key("a"), str_key("b")]);
        map.upsert(Value::Int(7), Value::Int(70));
        assert_eq!(map.get(&Value::Int(7)), Some(&Value::Int(70)));
    }

    #[test]
    fn test_dict_map_remove_rebuilds_index() {
        let mut map = DictMap::from_entries(vec![
            (str_key("a"), Value::Int(1)),
            (str_key("b"), Value::Int(2)),
            (str_key("c"), Value::Int(3)),
        ]);
        assert!(map.remove(&str_key("b")));
        assert!(!map.remove(&str_key("b")));
        assert_eq!(map.get(&str_key("c")), Some(&Value::Int(3)));
        assert_eq!(map.keys(), vec![str_key("a"), str_key("c")]);
        map.clear();
        assert!(map.is_empty());
        assert_eq!(map.get(&str_key("a")), None);
    }

    #[test]
    fn test_dict_map_equality_ignores_index() {
        let first = DictMap::from_entries(vec![(str_key("a"), Value::Int(1))]);
        let mut second = DictMap::from_entries(vec![(str_key("a"), Value::Int(1))]);
        second.upsert(str_key("a"), Value::Int(1));
        assert_eq!(first, second);
    }
}
