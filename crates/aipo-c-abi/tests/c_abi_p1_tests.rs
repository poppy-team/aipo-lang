//! Reproduction and validation tests for Phase 2:
//! - M1: Multi-module function isolation and Arc sharing.
//! - M2: Atomic 2-stage module loading and definition rollback.
//! - S1: Execution budgets (instruction limits) and counter queries.

use aipo_c_abi::*;
use std::ffi::{CStr, CString};

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

// ---------------------------------------------------------------- M1

#[test]
fn test_m1_multi_module_function_isolation() {
    let rt = aipo_runtime_create();
    assert!(!rt.is_null());

    // Module A defines compute() returning 111
    let status = load(
        rt,
        "mod_a",
        r#"
fn compute()
    return 111
end
"#,
    );
    assert_eq!(status, aipo_status_t::AIPO_OK, "{}", last_error(rt));

    // Module B defines compute() returning 222
    let status = load(
        rt,
        "mod_b",
        r#"
fn compute()
    return 222
end
"#,
    );
    assert_eq!(status, aipo_status_t::AIPO_OK, "{}", last_error(rt));

    // Module A must resolve its own compute() without being hijacked by Module B
    let (s_a1, res_a1) = call(rt, "mod_a", "compute", &[]);
    assert_eq!(s_a1, aipo_status_t::AIPO_OK);
    assert_eq!(res_a1.int_val, 111);

    // Module B resolves its own compute()
    let (s_b, res_b) = call(rt, "mod_b", "compute", &[]);
    assert_eq!(s_b, aipo_status_t::AIPO_OK);
    assert_eq!(res_b.int_val, 222);

    // Module A still resolves to 111
    let (s_a2, res_a2) = call(rt, "mod_a", "compute", &[]);
    assert_eq!(s_a2, aipo_status_t::AIPO_OK);
    assert_eq!(res_a2.int_val, 111);

    // Module C defines a global variable 'compute' with non-function value
    let status = load(
        rt,
        "mod_c",
        r#"
let compute = 999
"#,
    );
    assert_eq!(status, aipo_status_t::AIPO_OK, "{}", last_error(rt));

    // Calling mod_a.compute() must not resolve to global variable 'compute'
    let (s_a3, res_a3) = call(rt, "mod_a", "compute", &[]);
    assert_eq!(s_a3, aipo_status_t::AIPO_OK);
    assert_eq!(res_a3.int_val, 111);

    unsafe { aipo_runtime_destroy(rt) };
}

// ---------------------------------------------------------------- M2

#[test]
fn test_m2_atomic_two_stage_module_loading_rollback() {
    let rt = aipo_runtime_create();
    assert!(!rt.is_null());

    // Pre-existing good module
    let status = load(
        rt,
        "good_mod",
        r#"
struct GoodPoint
    x
    y
end

fn get_answer()
    return 42
end
"#,
    );
    assert_eq!(status, aipo_status_t::AIPO_OK, "{}", last_error(rt));

    // Attempt to load a module that compiles fine but faults during top-level execution
    let status = load(
        rt,
        "faulty_mod",
        r#"
struct BadStruct
    leak
end

# Division by zero at top level
let crash = 1 / 0
"#,
    );
    assert_eq!(
        status,
        aipo_status_t::AIPO_ERR_FAULT,
        "top-level fault must return AIPO_ERR_FAULT"
    );

    // Verify faulty_mod is not callable and not committed
    let (s_bad, _) = call(rt, "faulty_mod", "anything", &[]);
    assert_eq!(s_bad, aipo_status_t::AIPO_ERR_USAGE);
    assert!(last_error(rt).contains("module 'faulty_mod' is not loaded"));

    // Verify the good module is completely intact and operational
    let (s_good, res_good) = call(rt, "good_mod", "get_answer", &[]);
    assert_eq!(s_good, aipo_status_t::AIPO_OK);
    assert_eq!(res_good.int_val, 42);

    unsafe { aipo_runtime_destroy(rt) };
}

// ---------------------------------------------------------------- S1

#[test]
fn test_s1_instruction_budget_and_counters() {
    let rt = aipo_runtime_create();
    assert!(!rt.is_null());

    // Configure budget of 100 instructions
    let s_budget = unsafe { aipo_runtime_set_instruction_budget(rt, 100) };
    assert_eq!(s_budget, aipo_status_t::AIPO_OK);

    let status = load(
        rt,
        "loop_mod",
        r#"
fn spin_loop()
    var counter = 0
    while true
        counter = counter + 1
    end
    return counter
end

fn quick_fn()
    return 77
end
"#,
    );
    assert_eq!(status, aipo_status_t::AIPO_OK, "{}", last_error(rt));

    // Calling infinite loop must fault when budget trips
    let (s_spin, _) = call(rt, "loop_mod", "spin_loop", &[]);
    assert_eq!(
        s_spin,
        aipo_status_t::AIPO_ERR_FAULT,
        "infinite loop must halt with fault"
    );
    assert!(
        last_error(rt).contains("execution budget exceeded: instruction limit of 100 reached"),
        "error message must describe budget limit: {}",
        last_error(rt)
    );

    let count = unsafe { aipo_runtime_instruction_count(rt) };
    assert!(count > 100, "instruction count must reflect executed steps");

    // Reset instruction counter
    assert_eq!(
        unsafe { aipo_runtime_reset_instruction_count(rt) },
        aipo_status_t::AIPO_OK
    );
    assert_eq!(unsafe { aipo_runtime_instruction_count(rt) }, 0);

    // Disable budget (0 = unlimited)
    assert_eq!(
        unsafe { aipo_runtime_set_instruction_budget(rt, 0) },
        aipo_status_t::AIPO_OK
    );

    let (s_quick, res_quick) = call(rt, "loop_mod", "quick_fn", &[]);
    assert_eq!(s_quick, aipo_status_t::AIPO_OK);
    assert_eq!(res_quick.int_val, 77);

    unsafe { aipo_runtime_destroy(rt) };
}
