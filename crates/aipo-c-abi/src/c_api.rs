//! C ABI exports for Aipo (ADP-009).

use crate::runtime::AipoRuntime;
use crate::types::{aipo_handle_t, aipo_host_fn_t, aipo_runtime_t, aipo_status_t, aipo_value_t};
use aipo_host::HostValue;
use std::ffi::c_char;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Major version component (0 for v0.1.0).
pub const AIPO_VERSION_MAJOR: u32 = 0;
/// Minor version component (1 for v0.1.0).
pub const AIPO_VERSION_MINOR: u32 = 1;
/// Patch version component (0 for v0.1.0).
pub const AIPO_VERSION_PATCH: u32 = 0;

/// Returns the runtime version.
///
/// # Safety
///
/// If non-null, `major`, `minor`, and `patch` must point to valid, aligned writable `u32` locations.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipo_version(major: *mut u32, minor: *mut u32, patch: *mut u32) {
    if !major.is_null() {
        unsafe {
            *major = AIPO_VERSION_MAJOR;
        }
    }
    if !minor.is_null() {
        unsafe {
            *minor = AIPO_VERSION_MINOR;
        }
    }
    if !patch.is_null() {
        unsafe {
            *patch = AIPO_VERSION_PATCH;
        }
    }
}

/// Creates a new opaque Aipo runtime instance.
///
/// Returns null if memory allocation fails.
#[unsafe(no_mangle)]
pub extern "C" fn aipo_runtime_create() -> *mut aipo_runtime_t {
    let result = catch_unwind(AssertUnwindSafe(|| {
        Box::into_raw(Box::new(AipoRuntime::new()))
    }));
    result.unwrap_or(std::ptr::null_mut())
}

/// Destroys an Aipo runtime instance and releases all associated memory.
///
/// # Safety
///
/// If non-null, `rt` must be a valid pointer returned by [`aipo_runtime_create`] that has
/// not already been destroyed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipo_runtime_destroy(rt: *mut aipo_runtime_t) {
    if rt.is_null() {
        return;
    }
    let _ = catch_unwind(AssertUnwindSafe(|| unsafe {
        drop(Box::from_raw(rt));
    }));
}

/// Grants a host capability to the runtime.
///
/// # Safety
///
/// `rt` must point to a live [`aipo_runtime_t`] and `capability` must point to a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipo_runtime_grant_capability(
    rt: *mut aipo_runtime_t,
    capability: *const c_char,
) -> aipo_status_t {
    if rt.is_null() || capability.is_null() {
        return aipo_status_t::AIPO_ERR_NULL_POINTER;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let runtime = unsafe { &mut *rt };
        let c_str = unsafe { std::ffi::CStr::from_ptr(capability) };
        let cap_name = match c_str.to_str() {
            Ok(s) => s,
            Err(e) => {
                runtime.last_error = Some(format!("invalid UTF-8 in capability name: {e}"));
                return aipo_status_t::AIPO_ERR_USAGE;
            }
        };
        match runtime.grant(cap_name) {
            Ok(()) => aipo_status_t::AIPO_OK,
            Err(e) => {
                runtime.last_error = Some(e);
                aipo_status_t::AIPO_ERR_USAGE
            }
        }
    }));
    result.unwrap_or(aipo_status_t::AIPO_ERR_FAULT)
}

/// Revokes a host capability from the runtime.
///
/// # Safety
///
/// `rt` must point to a live [`aipo_runtime_t`] and `capability` must point to a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipo_runtime_revoke_capability(
    rt: *mut aipo_runtime_t,
    capability: *const c_char,
) -> aipo_status_t {
    if rt.is_null() || capability.is_null() {
        return aipo_status_t::AIPO_ERR_NULL_POINTER;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let runtime = unsafe { &mut *rt };
        let c_str = unsafe { std::ffi::CStr::from_ptr(capability) };
        let cap_name = match c_str.to_str() {
            Ok(s) => s,
            Err(e) => {
                runtime.last_error = Some(format!("invalid UTF-8 in capability name: {e}"));
                return aipo_status_t::AIPO_ERR_USAGE;
            }
        };
        match runtime.revoke(cap_name) {
            Ok(()) => aipo_status_t::AIPO_OK,
            Err(e) => {
                runtime.last_error = Some(e);
                aipo_status_t::AIPO_ERR_USAGE
            }
        }
    }));
    result.unwrap_or(aipo_status_t::AIPO_ERR_FAULT)
}

/// Compiles and initializes an Aipo module from UTF-8 source code.
///
/// # Safety
///
/// `rt` must point to a live [`aipo_runtime_t`]. `name` and `source` must point to valid null-terminated UTF-8 C strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipo_runtime_load_module(
    rt: *mut aipo_runtime_t,
    name: *const c_char,
    source: *const c_char,
) -> aipo_status_t {
    if rt.is_null() || name.is_null() || source.is_null() {
        return aipo_status_t::AIPO_ERR_NULL_POINTER;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let runtime = unsafe { &mut *rt };
        let name_str = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(e) => {
                runtime.last_error = Some(format!("invalid UTF-8 in module name: {e}"));
                return aipo_status_t::AIPO_ERR_USAGE;
            }
        };
        let source_str = match unsafe { std::ffi::CStr::from_ptr(source) }.to_str() {
            Ok(s) => s,
            Err(e) => {
                runtime.last_error = Some(format!("invalid UTF-8 in source text: {e}"));
                return aipo_status_t::AIPO_ERR_USAGE;
            }
        };
        match runtime.load_module(name_str, source_str) {
            Ok(()) => aipo_status_t::AIPO_OK,
            Err((status, msg)) => {
                runtime.last_error = Some(msg);
                status
            }
        }
    }));
    result.unwrap_or(aipo_status_t::AIPO_ERR_FAULT)
}

/// Calls an Aipo function in a loaded module.
///
/// # Safety
///
/// `rt` must point to a live [`aipo_runtime_t`]. `module_name` and `func_name` must point to valid null-terminated C strings.
/// If `argc > 0`, `args` must point to an array of at least `argc` initialized [`aipo_value_t`] elements.
/// If non-null, `out_result` must point to a valid writable [`aipo_value_t`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipo_runtime_call(
    rt: *mut aipo_runtime_t,
    module_name: *const c_char,
    func_name: *const c_char,
    args: *const aipo_value_t,
    argc: usize,
    out_result: *mut aipo_value_t,
) -> aipo_status_t {
    if rt.is_null() || module_name.is_null() || func_name.is_null() {
        return aipo_status_t::AIPO_ERR_NULL_POINTER;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let runtime = unsafe { &mut *rt };
        let mod_str = match unsafe { std::ffi::CStr::from_ptr(module_name) }.to_str() {
            Ok(s) => s,
            Err(e) => {
                runtime.last_error = Some(format!("invalid UTF-8 in module name: {e}"));
                return aipo_status_t::AIPO_ERR_USAGE;
            }
        };
        let fn_str = match unsafe { std::ffi::CStr::from_ptr(func_name) }.to_str() {
            Ok(s) => s,
            Err(e) => {
                runtime.last_error = Some(format!("invalid UTF-8 in function name: {e}"));
                return aipo_status_t::AIPO_ERR_USAGE;
            }
        };

        let arg_slice = if argc == 0 || args.is_null() {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(args, argc) }
        };

        match runtime.call(mod_str, fn_str, arg_slice) {
            Ok(res) => {
                if !out_result.is_null() {
                    unsafe {
                        *out_result = res;
                    }
                }
                aipo_status_t::AIPO_OK
            }
            Err((status, msg)) => {
                runtime.last_error = Some(msg);
                status
            }
        }
    }));
    result.unwrap_or(aipo_status_t::AIPO_ERR_FAULT)
}

/// Registers a host native function in the runtime.
///
/// # Safety
///
/// `rt` must point to a live [`aipo_runtime_t`]. `name` must point to a valid null-terminated C string.
/// `required_capability` (if non-null) must point to a valid null-terminated C string.
/// `callback` must be a valid function pointer with the expected C signature.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipo_runtime_register_host_fn(
    rt: *mut aipo_runtime_t,
    name: *const c_char,
    arity: usize,
    required_capability: *const c_char,
    callback: aipo_host_fn_t,
) -> aipo_status_t {
    if rt.is_null() || name.is_null() {
        return aipo_status_t::AIPO_ERR_NULL_POINTER;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let runtime = unsafe { &mut *rt };
        let fn_name = match unsafe { std::ffi::CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(e) => {
                runtime.last_error = Some(format!("invalid UTF-8 in host fn name: {e}"));
                return aipo_status_t::AIPO_ERR_USAGE;
            }
        };
        let req_cap = if required_capability.is_null() {
            None
        } else {
            match unsafe { std::ffi::CStr::from_ptr(required_capability) }.to_str() {
                Ok(s) => Some(s),
                Err(e) => {
                    runtime.last_error = Some(format!("invalid UTF-8 in required capability: {e}"));
                    return aipo_status_t::AIPO_ERR_USAGE;
                }
            }
        };
        runtime.register_host_fn(fn_name, arity, req_cap, callback);
        aipo_status_t::AIPO_OK
    }));
    result.unwrap_or(aipo_status_t::AIPO_ERR_FAULT)
}

/// Creates a new generational handle for an integer object identifier.
///
/// # Safety
///
/// `rt` must point to a live [`aipo_runtime_t`]. If `val` contains pointers (`str_ptr` or `bytes_ptr`), they must be valid for the duration of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipo_handle_create(
    rt: *mut aipo_runtime_t,
    val: aipo_value_t,
) -> aipo_handle_t {
    if rt.is_null() {
        return aipo_handle_t::default();
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let runtime = unsafe { &mut *rt };
        let host_val = val.to_host_value().unwrap_or(HostValue::None);
        runtime.handle_create(host_val)
    }));
    result.unwrap_or_default()
}

/// Resolves a generational handle to its host value.
///
/// # Safety
///
/// `rt` must point to a live [`aipo_runtime_t`] and `out_value` must point to a valid writable [`aipo_value_t`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipo_handle_resolve(
    rt: *mut aipo_runtime_t,
    handle: aipo_handle_t,
    out_value: *mut aipo_value_t,
) -> aipo_status_t {
    if rt.is_null() || out_value.is_null() {
        return aipo_status_t::AIPO_ERR_NULL_POINTER;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let runtime = unsafe { &mut *rt };
        match runtime.handle_resolve(handle) {
            Ok(val) => {
                let vm_val = match aipo_vm::host_value_to_value(&val) {
                    Ok(v) => v,
                    Err(_) => return aipo_status_t::AIPO_ERR_FAULT,
                };
                let c_val = aipo_value_t::from_vm_value(&vm_val, |s| runtime.pool_string(s));
                unsafe {
                    *out_value = c_val;
                }
                aipo_status_t::AIPO_OK
            }
            Err(_) => aipo_status_t::AIPO_ERR_STALE_HANDLE,
        }
    }));
    result.unwrap_or(aipo_status_t::AIPO_ERR_FAULT)
}

/// Releases a generational handle, invalidating it forever.
///
/// # Safety
///
/// `rt` must point to a live [`aipo_runtime_t`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipo_handle_release(
    rt: *mut aipo_runtime_t,
    handle: aipo_handle_t,
) -> aipo_status_t {
    if rt.is_null() {
        return aipo_status_t::AIPO_ERR_NULL_POINTER;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let runtime = unsafe { &mut *rt };
        match runtime.handle_release(handle) {
            Ok(()) => aipo_status_t::AIPO_OK,
            Err(_) => aipo_status_t::AIPO_ERR_STALE_HANDLE,
        }
    }));
    result.unwrap_or(aipo_status_t::AIPO_ERR_FAULT)
}

/// Copies the last error message into the provided buffer.
///
/// Returns the number of bytes written, including null terminator.
///
/// # Safety
///
/// `rt` must point to a live [`aipo_runtime_t`] and `buffer` must point to a valid writable buffer of at least `buffer_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipo_last_error(
    rt: *const aipo_runtime_t,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    if rt.is_null() || buffer.is_null() || buffer_len == 0 {
        return 0;
    }
    let runtime = unsafe { &*rt };
    let msg = match &runtime.last_error {
        Some(s) => s.as_bytes(),
        None => b"no error",
    };
    let copy_len = (buffer_len - 1).min(msg.len());
    unsafe {
        std::ptr::copy_nonoverlapping(msg.as_ptr().cast::<c_char>(), buffer, copy_len);
        *buffer.add(copy_len) = 0;
    }
    copy_len + 1
}
