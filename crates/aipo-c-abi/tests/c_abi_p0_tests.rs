//! Reproduction tests for the seven P0 boundary defects in the Aipo C ABI (ADP-009).
//!
//! Each test is named after the defect it pins down. Before the corresponding fix
//! lands, the test fails; after it, the test passes. The defect identifiers match
//! the technical dossier dated 2026-09-28, revalidated against this working tree.
//!
//! These run from Rust, which sees the same `#[repr(C)]` surface a C host sees.
//!
//! Scope: this file exercises the boundary through Rust test harnesses.
//! Native C-side compilation and execution are verified by `tests/host/main.c`
//! and `tests/c_host_tests.rs`, which run the native test binary under
//! AddressSanitizer and UndefinedBehaviorSanitizer (clang -fsanitize=address,undefined).

use aipo_c_abi::*;
use std::ffi::{CStr, CString, c_char};

/// Largest integer the Aipo value model accepts: 2^53 - 1.
const MAX_SAFE_INT: i64 = (1_i64 << 53) - 1;

fn load(rt: *mut aipo_runtime_t, name: &str, source: &str) -> aipo_status_t {
    let mod_name = CString::new(name).unwrap();
    let src = CString::new(source).unwrap();
    unsafe { aipo_runtime_load_module(rt, mod_name.as_ptr(), src.as_ptr()) }
}

fn call(
    rt: *mut aipo_runtime_t,
    module: &str,
    func: &str,
    args: &[aipo_value_t],
) -> (aipo_status_t, aipo_value_t) {
    let mod_name = CString::new(module).unwrap();
    let fn_name = CString::new(func).unwrap();
    let mut result = aipo_value_t::none();
    let status = unsafe {
        aipo_runtime_call(
            rt,
            mod_name.as_ptr(),
            fn_name.as_ptr(),
            args.as_ptr(),
            args.len(),
            &mut result,
        )
    };
    (status, result)
}

fn last_error(rt: *mut aipo_runtime_t) -> String {
    let mut buf = [0_i8; 512];
    let written = unsafe { aipo_last_error(rt, buf.as_mut_ptr(), buf.len()) };
    if written == 0 {
        return String::new();
    }
    unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

// ---------------------------------------------------------------- C1

/// C1: a `Bytes` value returned across the boundary pointed into a buffer owned by
/// the returned `Value`, which dies when the call returns. The pointer is dangling
/// and any reallocation of the `RefCell<Vec<u8>>` invalidates it further.
#[test]
fn c1_bytes_returned_from_call_survive_owner_death() {
    let rt = aipo_runtime_create();
    assert!(!rt.is_null());

    let status = load(
        rt,
        "bytes_mod",
        r#"
fn make_bytes() {
    let data = Bytes(8)
    data.write_u8(0, 65)
    data.write_u8(1, 66)
    data.write_u8(2, 67)
    return data
}
"#,
    );
    assert_eq!(status, aipo_status_t::AIPO_OK, "load: {}", last_error(rt));

    let (status, result) = call(rt, "bytes_mod", "make_bytes", &[]);
    assert_eq!(status, aipo_status_t::AIPO_OK, "call: {}", last_error(rt));
    assert_eq!(result.tag, aipo_val_tag_t::AIPO_VAL_BYTES);
    assert_eq!(result.bytes_len, 8);

    // The owning `Value` and its `Rc<RefCell<Vec<u8>>>` are gone by now. Reading the
    // returned pointer is exactly what a C host would do, and must still see the data.
    let bytes = unsafe { std::slice::from_raw_parts(result.bytes_ptr, result.bytes_len) };
    assert_eq!(
        bytes,
        &[65, 66, 67, 0, 0, 0, 0, 0],
        "returned Bytes must survive the death of its owner"
    );

    // Churn the runtime so any pooled allocation is reused before the second read.
    let _ = call(rt, "bytes_mod", "make_bytes", &[]);
    let bytes_again = unsafe { std::slice::from_raw_parts(result.bytes_ptr, result.bytes_len) };
    assert_eq!(
        bytes_again,
        &[65, 66, 67, 0, 0, 0, 0, 0],
        "returned Bytes must stay valid for the documented lifetime"
    );

    unsafe { aipo_value_release(rt, result) };
    unsafe { aipo_runtime_destroy(rt) };
}

// ---------------------------------------------------------------- C2

static C2_INVOCED: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

extern "C" fn c2_delete_callback(
    _rt: *mut aipo_runtime_t,
    args: *const aipo_value_t,
    argc: usize,
    out_result: *mut aipo_value_t,
) -> aipo_status_t {
    C2_INVOCED.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    if argc != 1 || args.is_null() || out_result.is_null() {
        return aipo_status_t::AIPO_ERR_USAGE;
    }
    unsafe { *out_result = aipo_value_t::bool_val(true) };
    aipo_status_t::AIPO_OK
}

extern "C" fn c2_unused_callback(
    _rt: *mut aipo_runtime_t,
    _args: *const aipo_value_t,
    _argc: usize,
    out_result: *mut aipo_value_t,
) -> aipo_status_t {
    C2_INVOCED.fetch_add(100, std::sync::atomic::Ordering::SeqCst);
    unsafe { *out_result = aipo_value_t::bool_val(true) };
    aipo_status_t::AIPO_OK
}

/// C2: two host functions share arity 1 but differ in required capability. The
/// fallback that picked an arbitrary same-arity entry could invoke the wrong
/// callback and check the wrong capability.
#[test]
fn c2_callback_identity_wins_over_arity() {
    let rt = aipo_runtime_create();
    assert!(!rt.is_null());

    // Both arity 1. `remove_entry` needs a capability we grant; `wipe_everything`
    // needs one we do not. An arity-based dispatch would confuse them.
    let allowed = CString::new("system.storage").unwrap();
    let denied = CString::new("system.vault").unwrap();
    let name_a = CString::new("remove_entry").unwrap();
    let name_b = CString::new("wipe_everything").unwrap();

    unsafe {
        assert_eq!(
            aipo_runtime_grant_capability(rt, allowed.as_ptr()),
            aipo_status_t::AIPO_OK
        );
        // Register the denied one first so an iteration-order fallback would
        // reach it before the allowed one for equal arities.
        assert_eq!(
            aipo_runtime_register_host_fn(
                rt,
                name_b.as_ptr(),
                1,
                denied.as_ptr(),
                c2_unused_callback
            ),
            aipo_status_t::AIPO_OK
        );
        assert_eq!(
            aipo_runtime_register_host_fn(
                rt,
                name_a.as_ptr(),
                1,
                allowed.as_ptr(),
                c2_delete_callback
            ),
            aipo_status_t::AIPO_OK
        );
    }

    let status = load(
        rt,
        "c2_mod",
        r#"
fn run(data) {
    return remove_entry(data)
}

fn run_wipe(data) {
    return wipe_everything(data)
}
"#,
    );
    assert_eq!(status, aipo_status_t::AIPO_OK, "load: {}", last_error(rt));

    C2_INVOCED.store(0, std::sync::atomic::Ordering::SeqCst);
    let (status, result) = call(rt, "c2_mod", "run", &[aipo_value_t::int_val(1)]);
    assert_eq!(
        status,
        aipo_status_t::AIPO_OK,
        "the allowed callback must run: {}",
        last_error(rt)
    );
    assert_eq!(result.tag, aipo_val_tag_t::AIPO_VAL_BOOL);
    assert!(result.bool_val);
    assert_eq!(
        C2_INVOCED.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "exactly the named callback must run, not a same-arity neighbour"
    );

    // The denied capability must still be enforced for the other function. A
    // capability denial inside guest code surfaces as a runtime fault (guest code
    // can catch it with `attempt`); the point here is that *this* function's
    // capability was the one checked, and that the wrong callback never ran.
    let (status, _) = call(rt, "c2_mod", "run_wipe", &[aipo_value_t::int_val(1)]);
    assert_ne!(
        status,
        aipo_status_t::AIPO_OK,
        "capability check must key on the called function, not an arity neighbour"
    );
    assert_eq!(
        C2_INVOCED.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "the denied callback must not have executed"
    );

    unsafe { aipo_runtime_destroy(rt) };
}

// ---------------------------------------------------------------- C3

#[test]
fn c3_rejects_out_of_contract_values() {
    let rt = aipo_runtime_create();
    assert!(!rt.is_null());

    let status = load(rt, "c3_mod", "fn echo(v) {\n    return v\n}\n");
    assert_eq!(status, aipo_status_t::AIPO_OK, "load: {}", last_error(rt));

    // Integer past the canonical safe range.
    let (status, _) = call(
        rt,
        "c3_mod",
        "echo",
        &[aipo_value_t::int_val(MAX_SAFE_INT + 1)],
    );
    assert_ne!(
        status,
        aipo_status_t::AIPO_OK,
        "MAX_SAFE_INT + 1 must be rejected"
    );

    // Non-finite floats.
    let nan = aipo_value_t::float_val(f64::NAN);
    let (status, _) = call(rt, "c3_mod", "echo", &[nan]);
    assert_ne!(status, aipo_status_t::AIPO_OK, "NaN must be rejected");

    let inf = aipo_value_t::float_val(f64::INFINITY);
    let (status, _) = call(rt, "c3_mod", "echo", &[inf]);
    assert_ne!(status, aipo_status_t::AIPO_OK, "infinity must be rejected");

    // A string that is valid UTF-8 but not NFC-normalized: "e" + U+0301 is
    // canonically "é" (U+00E9). The VM contract normalizes on the way in.
    let nfd = CString::new("e\u{301}").unwrap();
    let mut v = aipo_value_t::none();
    v.tag = aipo_val_tag_t::AIPO_VAL_STRING;
    v.str_ptr = nfd.as_ptr();
    v.str_len = nfd.to_bytes().len();
    let (status, result) = call(rt, "c3_mod", "echo", &[v]);
    assert_eq!(status, aipo_status_t::AIPO_OK, "call: {}", last_error(rt));
    let echoed = unsafe { CStr::from_ptr(result.str_ptr) }.to_bytes();
    assert_eq!(
        echoed,
        "é".as_bytes(),
        "strings must cross the boundary in NFC"
    );

    // An embedded NUL: the length-delimited read must keep it, and it must not be
    // silently truncated to a shorter string. `CString` cannot build this payload
    // at all — it rejects interior NULs — which is precisely the case the explicit
    // `str_len` exists to serve, so the buffer is built by hand.
    let with_nul: &[u8] = b"ab\0cd";
    let mut v = aipo_value_t::none();
    v.tag = aipo_val_tag_t::AIPO_VAL_STRING;
    v.str_ptr = with_nul.as_ptr().cast::<c_char>();
    v.str_len = with_nul.len();
    let (status, result) = call(rt, "c3_mod", "echo", &[v]);
    assert_eq!(status, aipo_status_t::AIPO_OK, "call: {}", last_error(rt));
    assert_eq!(
        result.str_len,
        with_nul.len(),
        "the boundary must echo exactly the `str_len` bytes it was handed"
    );
    let echoed = unsafe { std::slice::from_raw_parts(result.str_ptr.cast::<u8>(), result.str_len) };
    assert_eq!(
        echoed, with_nul,
        "an embedded NUL must survive the round trip"
    );

    unsafe { aipo_runtime_destroy(rt) };
}

// ---------------------------------------------------------------- C4

#[test]
fn c4_handle_create_rejects_invalid_value() {
    let rt = aipo_runtime_create();
    assert!(!rt.is_null());

    // An integer outside the canonical range must fail loudly. Previously the
    // conversion error was swallowed and a valid handle to `none` was returned.
    let out_of_range = aipo_value_t::int_val(MAX_SAFE_INT + 1);
    let mut handle = aipo_handle_t::default();
    let status = unsafe { aipo_handle_create_checked(rt, out_of_range, &mut handle) };
    assert_ne!(
        status,
        aipo_status_t::AIPO_OK,
        "an out-of-range value must not produce a handle"
    );
    assert_eq!(
        handle,
        aipo_handle_t::default(),
        "no handle may be created for a rejected value"
    );

    // A valid value still succeeds.
    let good = aipo_value_t::int_val(7);
    let status = unsafe { aipo_handle_create_checked(rt, good, &mut handle) };
    assert_eq!(
        status,
        aipo_status_t::AIPO_OK,
        "valid value: {}",
        last_error(rt)
    );

    let mut resolved = aipo_value_t::none();
    let status = unsafe { aipo_handle_resolve(rt, handle, &mut resolved) };
    assert_eq!(status, aipo_status_t::AIPO_OK);
    assert_eq!(resolved.tag, aipo_val_tag_t::AIPO_VAL_INT);
    assert_eq!(resolved.int_val, 7);
    assert_eq!(
        unsafe { aipo_handle_release(rt, handle) },
        aipo_status_t::AIPO_OK
    );

    unsafe { aipo_runtime_destroy(rt) };
}

// ---------------------------------------------------------------- C5

extern "C" fn dummy_cb(
    _rt: *mut aipo_runtime_t,
    _args: *const aipo_value_t,
    _argc: usize,
    _out: *mut aipo_value_t,
) -> aipo_status_t {
    aipo_status_t::AIPO_OK
}

extern "C" fn c5_reentrant_callback(
    rt: *mut aipo_runtime_t,
    _args: *const aipo_value_t,
    _argc: usize,
    out_result: *mut aipo_value_t,
) -> aipo_status_t {
    // Attempt to re-enter the runtime from inside a host callback, while the VM
    // is already mutably borrowed. This must be refused, not aliased.
    let mod_name = CString::new("c5_mod").unwrap();
    let fn_name = CString::new("inner").unwrap();
    let mut nested = aipo_value_t::none();
    let call_status = unsafe {
        aipo_runtime_call(
            rt,
            mod_name.as_ptr(),
            fn_name.as_ptr(),
            std::ptr::null(),
            0,
            &mut nested,
        )
    };
    C5_REENTRANT_CALL_STATUS.store(call_status as i64, std::sync::atomic::Ordering::SeqCst);

    let cap_name = CString::new("clock").unwrap();
    let grant_status = unsafe { aipo_runtime_grant_capability(rt, cap_name.as_ptr()) };
    C5_REENTRANT_GRANT_STATUS.store(grant_status as i64, std::sync::atomic::Ordering::SeqCst);

    let revoke_status = unsafe { aipo_runtime_revoke_capability(rt, cap_name.as_ptr()) };
    C5_REENTRANT_REVOKE_STATUS.store(revoke_status as i64, std::sync::atomic::Ordering::SeqCst);

    let reg_name = CString::new("reentrant_reg").unwrap();
    let reg_status = unsafe {
        aipo_runtime_register_host_fn(rt, reg_name.as_ptr(), 0, std::ptr::null(), dummy_cb)
    };
    C5_REENTRANT_REG_STATUS.store(reg_status as i64, std::sync::atomic::Ordering::SeqCst);

    let load_src = CString::new("fn extra() { return 42 }").unwrap();
    let load_status = unsafe { aipo_runtime_load_module(rt, mod_name.as_ptr(), load_src.as_ptr()) };
    C5_REENTRANT_LOAD_STATUS.store(load_status as i64, std::sync::atomic::Ordering::SeqCst);

    // Calling destroy inside a callback must be safely ignored/refused without freeing the VM.
    unsafe { aipo_runtime_destroy(rt) };

    unsafe { *out_result = aipo_value_t::int_val(1) };
    aipo_status_t::AIPO_OK
}

static C5_REENTRANT_CALL_STATUS: std::sync::atomic::AtomicI64 =
    std::sync::atomic::AtomicI64::new(-1);
static C5_REENTRANT_GRANT_STATUS: std::sync::atomic::AtomicI64 =
    std::sync::atomic::AtomicI64::new(-1);
static C5_REENTRANT_REVOKE_STATUS: std::sync::atomic::AtomicI64 =
    std::sync::atomic::AtomicI64::new(-1);
static C5_REENTRANT_REG_STATUS: std::sync::atomic::AtomicI64 =
    std::sync::atomic::AtomicI64::new(-1);
static C5_REENTRANT_LOAD_STATUS: std::sync::atomic::AtomicI64 =
    std::sync::atomic::AtomicI64::new(-1);

/// C5: the dispatcher handed the C callback a full `*mut aipo_runtime_t` while the
/// VM was already mutably borrowed from that same runtime. Re-entering it must be
/// rejected before any `&mut` is formed, and `destroy` during execution refused.
#[test]
fn c5_runtime_rejects_reentrant_calls() {
    let rt = aipo_runtime_create();
    assert!(!rt.is_null());

    let fn_name = CString::new("outer_host_fn").unwrap();
    let status = unsafe {
        aipo_runtime_register_host_fn(
            rt,
            fn_name.as_ptr(),
            0,
            std::ptr::null(),
            c5_reentrant_callback,
        )
    };
    assert_eq!(status, aipo_status_t::AIPO_OK);

    let status = load(
        rt,
        "c5_mod",
        r#"
fn inner() {
    return 1
}

fn outer() {
    return outer_host_fn()
}
"#,
    );
    assert_eq!(status, aipo_status_t::AIPO_OK, "load: {}", last_error(rt));

    C5_REENTRANT_CALL_STATUS.store(-1, std::sync::atomic::Ordering::SeqCst);
    C5_REENTRANT_GRANT_STATUS.store(-1, std::sync::atomic::Ordering::SeqCst);
    C5_REENTRANT_REVOKE_STATUS.store(-1, std::sync::atomic::Ordering::SeqCst);
    C5_REENTRANT_REG_STATUS.store(-1, std::sync::atomic::Ordering::SeqCst);
    C5_REENTRANT_LOAD_STATUS.store(-1, std::sync::atomic::Ordering::SeqCst);

    let (status, _) = call(rt, "c5_mod", "outer", &[]);
    assert_eq!(status, aipo_status_t::AIPO_OK, "outer: {}", last_error(rt));

    assert_eq!(
        C5_REENTRANT_CALL_STATUS.load(std::sync::atomic::Ordering::SeqCst),
        aipo_status_t::AIPO_ERR_USAGE as i64,
        "re-entering call from host callback must return AIPO_ERR_USAGE"
    );
    assert_eq!(
        C5_REENTRANT_GRANT_STATUS.load(std::sync::atomic::Ordering::SeqCst),
        aipo_status_t::AIPO_ERR_USAGE as i64,
        "re-entering grant_capability from host callback must return AIPO_ERR_USAGE"
    );
    assert_eq!(
        C5_REENTRANT_REVOKE_STATUS.load(std::sync::atomic::Ordering::SeqCst),
        aipo_status_t::AIPO_ERR_USAGE as i64,
        "re-entering revoke_capability from host callback must return AIPO_ERR_USAGE"
    );
    assert_eq!(
        C5_REENTRANT_REG_STATUS.load(std::sync::atomic::Ordering::SeqCst),
        aipo_status_t::AIPO_ERR_USAGE as i64,
        "re-entering register_host_fn from host callback must return AIPO_ERR_USAGE"
    );
    assert_eq!(
        C5_REENTRANT_LOAD_STATUS.load(std::sync::atomic::Ordering::SeqCst),
        aipo_status_t::AIPO_ERR_USAGE as i64,
        "re-entering load_module from host callback must return AIPO_ERR_USAGE"
    );

    // Verify runtime is still alive and healthy after rejected destroy during callback
    let (status, res) = call(rt, "c5_mod", "inner", &[]);
    assert_eq!(status, aipo_status_t::AIPO_OK);
    assert_eq!(res.int_val, 1);

    unsafe { aipo_runtime_destroy(rt) };
}

// ---------------------------------------------------------------- C6

#[test]
fn c6_null_args_with_positive_argc_is_rejected() {
    let rt = aipo_runtime_create();
    assert!(!rt.is_null());

    let status = load(rt, "c6_mod", "fn noop() {\n    return 1\n}\n");
    assert_eq!(status, aipo_status_t::AIPO_OK, "load: {}", last_error(rt));

    let mod_name = CString::new("c6_mod").unwrap();
    let fn_name = CString::new("noop").unwrap();
    let mut result = aipo_value_t::none();

    // argc = 0 with args = NULL is a legitimate zero-argument call.
    let status = unsafe {
        aipo_runtime_call(
            rt,
            mod_name.as_ptr(),
            fn_name.as_ptr(),
            std::ptr::null(),
            0,
            &mut result,
        )
    };
    assert_eq!(
        status,
        aipo_status_t::AIPO_OK,
        "zero-arg call: {}",
        last_error(rt)
    );

    // argc > 0 with args = NULL contradicts the header contract and must be an
    // explicit error instead of silently reading zero arguments.
    let status = unsafe {
        aipo_runtime_call(
            rt,
            mod_name.as_ptr(),
            fn_name.as_ptr(),
            std::ptr::null(),
            3,
            &mut result,
        )
    };
    assert_eq!(
        status,
        aipo_status_t::AIPO_ERR_NULL_POINTER,
        "argc > 0 with NULL args must be rejected, not treated as zero args"
    );

    unsafe { aipo_runtime_destroy(rt) };
}

// ---------------------------------------------------------------- C7

#[test]
fn c7_string_with_nul_reports_consistent_length() {
    let rt = aipo_runtime_create();
    assert!(!rt.is_null());

    // Build a String that genuinely contains a NUL inside the VM: encode "ab" to
    // bytes, punch a 0 into the middle, and decode back to a String.
    let status = load(
        rt,
        "c7_mod",
        r#"
fn make_nul_string() {
    let b = "abc".encode()
    b.write_u8(1, 0)
    return b.decode()
}
"#,
    );
    assert_eq!(status, aipo_status_t::AIPO_OK, "load: {}", last_error(rt));

    let (status, result) = call(rt, "c7_mod", "make_nul_string", &[]);
    assert_eq!(status, aipo_status_t::AIPO_OK, "call: {}", last_error(rt));
    assert_eq!(result.tag, aipo_val_tag_t::AIPO_VAL_STRING);

    // The VM value really does hold "a\0b" (3 bytes). The boundary must report a
    // length that matches the bytes it actually points at. Previously the string
    // was swapped for a fixed placeholder while the original length was kept, so
    // a consumer reading `str_len` bytes would run past the allocation.
    assert_eq!(
        result.str_len, 3,
        "str_len must describe the buffer the pointer actually references"
    );
    let bytes = unsafe { std::slice::from_raw_parts(result.str_ptr.cast::<u8>(), result.str_len) };
    assert_eq!(
        bytes, b"a\0c",
        "a NUL inside a String must survive the boundary without substitution"
    );

    unsafe { aipo_value_release(rt, result) };
    unsafe { aipo_runtime_destroy(rt) };
}
