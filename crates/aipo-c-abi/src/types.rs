//! C ABI types, status codes, and value representations (ADP-009).

use aipo_host::{Handle, HostValue};
use aipo_vm::Value;
use std::ffi::c_char;

/// Return / status code for C ABI operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum aipo_status_t {
    /// Operation succeeded.
    AIPO_OK = 0,
    /// Usage error: invalid argument or null pointer.
    AIPO_ERR_USAGE = 1,
    /// Compilation or semantic diagnostic error.
    AIPO_ERR_DIAGNOSTIC = 2,
    /// Uncaught failure (Model B) returned by Aipo code.
    AIPO_ERR_UNCAUGHT_FAILURE = 3,
    /// Runtime fault (e.g. division by zero, type mismatch).
    AIPO_ERR_FAULT = 4,
    /// Operation refused because host capability was not granted.
    AIPO_ERR_CAPABILITY_DENIED = 5,
    /// Attempted to use a released or stale generational handle.
    AIPO_ERR_STALE_HANDLE = 6,
    /// Unexpected null pointer passed across the boundary.
    AIPO_ERR_NULL_POINTER = 7,
}

/// Tag discriminator for [`aipo_value_t`].
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum aipo_val_tag_t {
    /// None value.
    AIPO_VAL_NONE = 0,
    /// Boolean value.
    AIPO_VAL_BOOL = 1,
    /// 64-bit integer within ±(2^53 - 1).
    AIPO_VAL_INT = 2,
    /// Finite 64-bit float.
    AIPO_VAL_FLOAT = 3,
    /// UTF-8 string snapshot.
    AIPO_VAL_STRING = 4,
    /// Byte sequence.
    AIPO_VAL_BYTES = 5,
    /// Generational host handle.
    AIPO_VAL_HANDLE = 6,
    /// Failure value with message.
    AIPO_VAL_FAILURE = 7,
}

/// Generational handle representation across the C ABI.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(non_camel_case_types)]
pub struct aipo_handle_t {
    /// Slot index in the handle table.
    pub index: u64,
    /// Generation count of the slot.
    pub generation: u32,
}

impl From<Handle> for aipo_handle_t {
    fn from(handle: Handle) -> Self {
        Self {
            index: handle.index() as u64,
            generation: handle.generation(),
        }
    }
}

impl From<aipo_handle_t> for Handle {
    fn from(h: aipo_handle_t) -> Self {
        Handle::from_raw(h.index as usize, h.generation)
    }
}

/// Opaque runtime handle for C ABI.
#[allow(non_camel_case_types)]
pub type aipo_runtime_t = crate::runtime::AipoRuntime;

/// Host native callback signature for C ABI.
#[allow(non_camel_case_types)]
pub type aipo_host_fn_t = extern "C" fn(
    rt: *mut aipo_runtime_t,
    args: *const aipo_value_t,
    argc: usize,
    out_result: *mut aipo_value_t,
) -> aipo_status_t;

/// Plain, C-compatible value representation.
///
/// Simple values are copied by value. Strings and bytes point to UTF-8 / byte buffers
/// owned either by the host or kept in the runtime's memory pool during the call.
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(non_camel_case_types)]
pub struct aipo_value_t {
    /// Type tag discriminator.
    pub tag: aipo_val_tag_t,
    /// Boolean payload.
    pub bool_val: bool,
    /// Integer payload.
    pub int_val: i64,
    /// Float payload.
    pub float_val: f64,
    /// Pointer to string data (null-terminated or bounded by `str_len`).
    pub str_ptr: *const c_char,
    /// Byte length of string data.
    pub str_len: usize,
    /// Pointer to raw bytes.
    pub bytes_ptr: *const u8,
    /// Byte length of raw bytes.
    pub bytes_len: usize,
    /// Generational handle payload.
    pub handle_val: aipo_handle_t,
}

impl Default for aipo_value_t {
    fn default() -> Self {
        Self {
            tag: aipo_val_tag_t::AIPO_VAL_NONE,
            bool_val: false,
            int_val: 0,
            float_val: 0.0,
            str_ptr: std::ptr::null(),
            str_len: 0,
            bytes_ptr: std::ptr::null(),
            bytes_len: 0,
            handle_val: aipo_handle_t::default(),
        }
    }
}

impl aipo_value_t {
    /// Creates a none value.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            tag: aipo_val_tag_t::AIPO_VAL_NONE,
            bool_val: false,
            int_val: 0,
            float_val: 0.0,
            str_ptr: std::ptr::null(),
            str_len: 0,
            bytes_ptr: std::ptr::null(),
            bytes_len: 0,
            handle_val: aipo_handle_t {
                index: 0,
                generation: 0,
            },
        }
    }

    /// Creates a boolean value.
    #[must_use]
    pub const fn bool_val(val: bool) -> Self {
        let mut v = Self::none();
        v.tag = aipo_val_tag_t::AIPO_VAL_BOOL;
        v.bool_val = val;
        v
    }

    /// Creates an integer value.
    #[must_use]
    pub const fn int_val(val: i64) -> Self {
        let mut v = Self::none();
        v.tag = aipo_val_tag_t::AIPO_VAL_INT;
        v.int_val = val;
        v
    }

    /// Creates a float value.
    #[must_use]
    pub const fn float_val(val: f64) -> Self {
        let mut v = Self::none();
        v.tag = aipo_val_tag_t::AIPO_VAL_FLOAT;
        v.float_val = val;
        v
    }

    /// Creates a handle value.
    #[must_use]
    pub const fn handle(h: aipo_handle_t) -> Self {
        let mut v = Self::none();
        v.tag = aipo_val_tag_t::AIPO_VAL_HANDLE;
        v.handle_val = h;
        v
    }

    /// Converts a VM [`Value`] to an [`aipo_value_t`].
    ///
    /// If the value contains a heap string or failure, `on_string` is called to pool
    /// the null-terminated `CString` and return its raw pointer.
    pub fn from_vm_value<F>(val: &Value, mut on_string: F) -> Self
    where
        F: FnMut(&str) -> *const c_char,
    {
        match val {
            Value::None => Self::none(),
            Value::Bool(b) => Self::bool_val(*b),
            Value::Int(i) => Self::int_val(*i),
            Value::Float(f) => Self::float_val(*f),
            Value::String(s) => {
                let mut v = Self::none();
                v.tag = aipo_val_tag_t::AIPO_VAL_STRING;
                v.str_ptr = on_string(s.as_str());
                v.str_len = s.len();
                v
            }
            Value::Bytes(b) => {
                let borrow = b.borrow();
                let mut v = Self::none();
                v.tag = aipo_val_tag_t::AIPO_VAL_BYTES;
                v.bytes_ptr = borrow.as_ptr();
                v.bytes_len = borrow.len();
                v
            }
            Value::HostHandle(h) => Self::handle((*h).into()),
            Value::Failure(f) => {
                let mut v = Self::none();
                v.tag = aipo_val_tag_t::AIPO_VAL_FAILURE;
                v.str_ptr = on_string(&f.message);
                v.str_len = f.message.len();
                v
            }
            other => {
                let s = other.to_string();
                let mut v = Self::none();
                v.tag = aipo_val_tag_t::AIPO_VAL_STRING;
                v.str_ptr = on_string(&s);
                v.str_len = s.len();
                v
            }
        }
    }

    /// Converts an [`aipo_value_t`] to an [`aipo_host::HostValue`].
    ///
    /// # Errors
    /// Returns `Err` if numeric bounds or UTF-8 invariants are violated.
    pub fn to_host_value(&self) -> Result<HostValue, String> {
        match self.tag {
            aipo_val_tag_t::AIPO_VAL_NONE => Ok(HostValue::None),
            aipo_val_tag_t::AIPO_VAL_BOOL => Ok(HostValue::Bool(self.bool_val)),
            aipo_val_tag_t::AIPO_VAL_INT => HostValue::int(self.int_val)
                .map_err(|e| format!("integer {} out of range: {e}", self.int_val)),
            aipo_val_tag_t::AIPO_VAL_FLOAT => HostValue::float(self.float_val)
                .map_err(|e| format!("float {} is non-finite: {e}", self.float_val)),
            aipo_val_tag_t::AIPO_VAL_STRING => {
                if self.str_ptr.is_null() {
                    return Ok(HostValue::String(String::new()));
                }
                let c_str = unsafe { std::ffi::CStr::from_ptr(self.str_ptr) };
                let s = c_str
                    .to_str()
                    .map_err(|e| format!("invalid UTF-8 in string: {e}"))?;
                Ok(HostValue::String(s.to_string()))
            }
            aipo_val_tag_t::AIPO_VAL_BYTES => {
                if self.bytes_ptr.is_null() || self.bytes_len == 0 {
                    return Ok(HostValue::Bytes(Vec::new()));
                }
                let slice = unsafe { std::slice::from_raw_parts(self.bytes_ptr, self.bytes_len) };
                Ok(HostValue::Bytes(slice.to_vec()))
            }
            aipo_val_tag_t::AIPO_VAL_HANDLE => {
                // Synthesize handle from index and generation
                // The table will check index and generation on resolution
                Err("cannot convert handle directly to HostValue without table lookup".to_string())
            }
            aipo_val_tag_t::AIPO_VAL_FAILURE => {
                Err("failure value cannot be passed as a normal host argument".to_string())
            }
        }
    }

    /// Converts an [`aipo_value_t`] to a VM [`Value`].
    pub fn to_vm_value(&self) -> Result<Value, String> {
        match self.tag {
            aipo_val_tag_t::AIPO_VAL_NONE => Ok(Value::None),
            aipo_val_tag_t::AIPO_VAL_BOOL => Ok(Value::Bool(self.bool_val)),
            aipo_val_tag_t::AIPO_VAL_INT => Ok(Value::Int(self.int_val)),
            aipo_val_tag_t::AIPO_VAL_FLOAT => Ok(Value::Float(self.float_val)),
            aipo_val_tag_t::AIPO_VAL_STRING => {
                if self.str_ptr.is_null() {
                    return Ok(Value::String(std::rc::Rc::new(String::new())));
                }
                let c_str = unsafe { std::ffi::CStr::from_ptr(self.str_ptr) };
                let s = c_str
                    .to_str()
                    .map_err(|e| format!("invalid UTF-8 in string: {e}"))?;
                Ok(Value::String(std::rc::Rc::new(s.to_string())))
            }
            aipo_val_tag_t::AIPO_VAL_BYTES => {
                if self.bytes_ptr.is_null() || self.bytes_len == 0 {
                    return Ok(Value::Bytes(std::rc::Rc::new(std::cell::RefCell::new(
                        Vec::new(),
                    ))));
                }
                let slice = unsafe { std::slice::from_raw_parts(self.bytes_ptr, self.bytes_len) };
                Ok(Value::Bytes(std::rc::Rc::new(std::cell::RefCell::new(
                    slice.to_vec(),
                ))))
            }
            aipo_val_tag_t::AIPO_VAL_HANDLE => Ok(Value::HostHandle(self.handle_val.into())),
            aipo_val_tag_t::AIPO_VAL_FAILURE => {
                if self.str_ptr.is_null() {
                    return Ok(Value::failure("failure"));
                }
                let c_str = unsafe { std::ffi::CStr::from_ptr(self.str_ptr) };
                let s = c_str.to_str().unwrap_or("failure");
                Ok(Value::failure(s.to_string()))
            }
        }
    }
}
