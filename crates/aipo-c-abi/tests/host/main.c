/**
 * \file main.c
 * \brief Native C integration harness verifying Aipo C ABI 0.2.0 and P0 contracts.
 *
 * This executable is compiled with a real C compiler (gcc or clang) against
 * include/aipo.h and linked with libaipo_c_abi. When run under AddressSanitizer,
 * it proves that memory layout, value snapshots, reentrancy guards, and handle
 * lifecycles operate without memory corruption, leaks, or undefined behavior.
 */

#include "aipo.h"
#include <assert.h>
#include <math.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAX_SAFE_INT ((1LL << 53) - 1)

static void test_version(void) {
    uint32_t major = 99, minor = 99, patch = 99;
    aipo_version(&major, &minor, &patch);
    assert(major == 0);
    assert(minor == 2);
    assert(patch == 0);
    printf("  [PASS] version query reports 0.2.0\n");
}

/* ------------------------------------------------------------- C1 */

static void test_c1_bytes_survive_and_release(void) {
    aipo_runtime_t *rt = aipo_runtime_create();
    assert(rt != NULL);

    const char *source =
        "fn make_bytes()\n"
        "    let data = Bytes(8)\n"
        "    data.write_u8(0, 65)\n"
        "    data.write_u8(1, 66)\n"
        "    data.write_u8(2, 67)\n"
        "    return data\n"
        "end\n"
        "fn churn()\n"
        "    let x = \"allocating something else\"\n"
        "    return x\n"
        "end\n";

    aipo_status_t status = aipo_runtime_load_module(rt, "c1_mod", source);
    assert(status == AIPO_OK);

    aipo_value_t result;
    memset(&result, 0, sizeof(result));
    status = aipo_runtime_call(rt, "c1_mod", "make_bytes", NULL, 0, &result);
    assert(status == AIPO_OK);
    assert(result.tag == AIPO_VAL_BYTES);
    assert(result.bytes_len == 8);
    assert(result.bytes_ptr != NULL);

    /* Churn runtime to verify snapshot independence from VM execution */
    aipo_value_t churn_res;
    memset(&churn_res, 0, sizeof(churn_res));
    status = aipo_runtime_call(rt, "c1_mod", "churn", NULL, 0, &churn_res);
    assert(status == AIPO_OK);
    aipo_value_release(rt, churn_res);

    /* Check bytes remain valid */
    const uint8_t expected[8] = {65, 66, 67, 0, 0, 0, 0, 0};
    assert(memcmp(result.bytes_ptr, expected, 8) == 0);

    /* Release snapshot */
    aipo_value_release(rt, result);

    aipo_runtime_destroy(rt);
    printf("  [PASS] C1: Bytes snapshot survives owner death and releases cleanly\n");
}

/* ------------------------------------------------------------- C2 */

static int g_save_called = 0;
static int g_delete_called = 0;

static aipo_status_t cb_save(
    aipo_runtime_t *rt,
    const aipo_value_t *args,
    size_t argc,
    aipo_value_t *out_result
) {
    (void)rt; (void)args; (void)argc;
    g_save_called++;
    out_result->tag = AIPO_VAL_INT;
    out_result->int_val = 100;
    return AIPO_OK;
}

static aipo_status_t cb_delete(
    aipo_runtime_t *rt,
    const aipo_value_t *args,
    size_t argc,
    aipo_value_t *out_result
) {
    (void)rt; (void)args; (void)argc;
    g_delete_called++;
    out_result->tag = AIPO_VAL_INT;
    out_result->int_val = 200;
    return AIPO_OK;
}

static void test_c2_callback_identity_wins_over_arity(void) {
    aipo_runtime_t *rt = aipo_runtime_create();
    assert(rt != NULL);

    aipo_status_t s1 = aipo_runtime_register_host_fn(rt, "save", 1, NULL, cb_save);
    assert(s1 == AIPO_OK);
    aipo_status_t s2 = aipo_runtime_register_host_fn(rt, "delete", 1, NULL, cb_delete);
    assert(s2 == AIPO_OK);

    const char *source =
        "fn call_save(x)\n"
        "    return save(x)\n"
        "end\n"
        "fn call_delete(x)\n"
        "    return delete(x)\n"
        "end\n";

    aipo_status_t s3 = aipo_runtime_load_module(rt, "c2_mod", source);
    assert(s3 == AIPO_OK);

    g_save_called = 0;
    g_delete_called = 0;

    aipo_value_t arg;
    memset(&arg, 0, sizeof(arg));
    arg.tag = AIPO_VAL_INT;
    arg.int_val = 42;

    aipo_value_t res;
    memset(&res, 0, sizeof(res));

    aipo_status_t status = aipo_runtime_call(rt, "c2_mod", "call_save", &arg, 1, &res);
    assert(status == AIPO_OK);
    assert(res.int_val == 100);
    assert(g_save_called == 1);
    assert(g_delete_called == 0);

    status = aipo_runtime_call(rt, "c2_mod", "call_delete", &arg, 1, &res);
    assert(status == AIPO_OK);
    assert(res.int_val == 200);
    assert(g_save_called == 1);
    assert(g_delete_called == 1);

    aipo_runtime_destroy(rt);
    printf("  [PASS] C2: Host callback identity matches callee, never arity fallback\n");
}

/* ------------------------------------------------------------- C3 & C4 */

static void test_c3_c4_validation_and_handle_create_checked(void) {
    aipo_runtime_t *rt = aipo_runtime_create();
    assert(rt != NULL);

    /* Out of safe range integer */
    aipo_value_t bad_int;
    memset(&bad_int, 0, sizeof(bad_int));
    bad_int.tag = AIPO_VAL_INT;
    bad_int.int_val = MAX_SAFE_INT + 1;

    aipo_handle_t handle;
    memset(&handle, 0, sizeof(handle));
    aipo_status_t status = aipo_handle_create_checked(rt, bad_int, &handle);
    assert(status != AIPO_OK);
    assert(handle.index == 0 && handle.generation == 0);

    /* NaN float */
    aipo_value_t bad_float;
    memset(&bad_float, 0, sizeof(bad_float));
    bad_float.tag = AIPO_VAL_FLOAT;
    bad_float.float_val = NAN;

    status = aipo_handle_create_checked(rt, bad_float, &handle);
    assert(status != AIPO_OK);
    assert(handle.index == 0 && handle.generation == 0);

    /* Valid integer */
    aipo_value_t good_val;
    memset(&good_val, 0, sizeof(good_val));
    good_val.tag = AIPO_VAL_INT;
    good_val.int_val = 12345;

    status = aipo_handle_create_checked(rt, good_val, &handle);
    assert(status == AIPO_OK);
    assert(handle.generation > 0);

    /* Resolve handle */
    aipo_value_t resolved;
    memset(&resolved, 0, sizeof(resolved));
    status = aipo_handle_resolve(rt, handle, &resolved);
    assert(status == AIPO_OK);
    assert(resolved.tag == AIPO_VAL_INT);
    assert(resolved.int_val == 12345);

    /* Release handle */
    status = aipo_handle_release(rt, handle);
    assert(status == AIPO_OK);

    /* Stale handle query */
    status = aipo_handle_resolve(rt, handle, &resolved);
    assert(status == AIPO_ERR_STALE_HANDLE);

    aipo_runtime_destroy(rt);
    printf("  [PASS] C3 & C4: Checked handle minting rejects invalid values and prevents stale access\n");
}

/* ------------------------------------------------------------- C5 */

static int g_c5_reentrant_call_status = -1;
static int g_c5_reentrant_grant_status = -1;

static aipo_status_t reentrant_probe_cb(
    aipo_runtime_t *rt,
    const aipo_value_t *args,
    size_t argc,
    aipo_value_t *out_result
) {
    (void)args; (void)argc;

    aipo_value_t nested;
    memset(&nested, 0, sizeof(nested));

    /* Re-entering call */
    aipo_status_t s_call = aipo_runtime_call(rt, "c5_mod", "inner", NULL, 0, &nested);
    g_c5_reentrant_call_status = (int)s_call;

    /* Re-entering grant */
    aipo_status_t s_grant = aipo_runtime_grant_capability(rt, "clock");
    g_c5_reentrant_grant_status = (int)s_grant;

    /* Re-entering destroy must be safely ignored/refused */
    aipo_runtime_destroy(rt);

    out_result->tag = AIPO_VAL_INT;
    out_result->int_val = 1;
    return AIPO_OK;
}

static void test_c5_reentrancy_guards(void) {
    aipo_runtime_t *rt = aipo_runtime_create();
    assert(rt != NULL);

    aipo_status_t s_reg = aipo_runtime_register_host_fn(rt, "probe", 0, NULL, reentrant_probe_cb);
    assert(s_reg == AIPO_OK);

    const char *source =
        "fn inner()\n"
        "    return 42\n"
        "end\n"
        "fn outer()\n"
        "    return probe()\n"
        "end\n";

    aipo_status_t s_load = aipo_runtime_load_module(rt, "c5_mod", source);
    assert(s_load == AIPO_OK);

    aipo_value_t res;
    memset(&res, 0, sizeof(res));

    aipo_status_t s_call = aipo_runtime_call(rt, "c5_mod", "outer", NULL, 0, &res);
    assert(s_call == AIPO_OK);

    assert(g_c5_reentrant_call_status == AIPO_ERR_USAGE);
    assert(g_c5_reentrant_grant_status == AIPO_ERR_USAGE);

    /* Runtime remains healthy after safe no-op destroy */
    s_call = aipo_runtime_call(rt, "c5_mod", "inner", NULL, 0, &res);
    assert(s_call == AIPO_OK);
    assert(res.int_val == 42);

    aipo_runtime_destroy(rt);
    printf("  [PASS] C5: Runtime rejects reentrant calls, capability mutations, and destroy during execution\n");
}

/* ------------------------------------------------------------- C6 */

static void test_c6_null_args_positive_argc(void) {
    aipo_runtime_t *rt = aipo_runtime_create();
    assert(rt != NULL);

    const char *source =
        "fn noop()\n"
        "    return 1\n"
        "end\n";
    aipo_status_t s_load = aipo_runtime_load_module(rt, "c6_mod", source);
    assert(s_load == AIPO_OK);

    aipo_value_t res;
    memset(&res, 0, sizeof(res));

    /* Valid 0-arg call with NULL */
    aipo_status_t s1 = aipo_runtime_call(rt, "c6_mod", "noop", NULL, 0, &res);
    assert(s1 == AIPO_OK);

    /* Invalid call: argc > 0 but args == NULL */
    aipo_status_t s2 = aipo_runtime_call(rt, "c6_mod", "noop", NULL, 2, &res);
    assert(s2 == AIPO_ERR_NULL_POINTER);

    aipo_runtime_destroy(rt);
    printf("  [PASS] C6: argc > 0 with args == NULL is rejected as AIPO_ERR_NULL_POINTER\n");
}

/* ------------------------------------------------------------- C7 */

static void test_c7_nul_string_length(void) {
    aipo_runtime_t *rt = aipo_runtime_create();
    assert(rt != NULL);

    const char *source =
        "fn make_nul_string()\n"
        "    let b = \"abc\".encode()\n"
        "    b.write_u8(1, 0)\n"
        "    return b.decode()\n"
        "end\n";
    aipo_status_t s_load = aipo_runtime_load_module(rt, "c7_mod", source);
    assert(s_load == AIPO_OK);

    aipo_value_t res;
    memset(&res, 0, sizeof(res));

    aipo_status_t s_call = aipo_runtime_call(rt, "c7_mod", "make_nul_string", NULL, 0, &res);
    assert(s_call == AIPO_OK);
    assert(res.tag == AIPO_VAL_STRING);
    assert(res.str_len == 3);
    assert(memcmp(res.str_ptr, "a\0c", 3) == 0);

    aipo_value_release(rt, res);
    aipo_runtime_destroy(rt);
    printf("  [PASS] C7: String with embedded NUL reports exact length and content\n");
}

/* ------------------------------------------------------------- M1 */

static void test_m1_module_scope_isolation(void) {
    aipo_runtime_t *rt = aipo_runtime_create();
    assert(rt != NULL);

    const char *src_a =
        "fn compute()\n"
        "    return 111\n"
        "end\n";

    const char *src_b =
        "fn compute()\n"
        "    return 222\n"
        "end\n";

    aipo_status_t s1 = aipo_runtime_load_module(rt, "mod_a", src_a);
    assert(s1 == AIPO_OK);

    aipo_status_t s2 = aipo_runtime_load_module(rt, "mod_b", src_b);
    assert(s2 == AIPO_OK);

    aipo_value_t res_a;
    memset(&res_a, 0, sizeof(res_a));
    aipo_status_t s_call_a = aipo_runtime_call(rt, "mod_a", "compute", NULL, 0, &res_a);
    assert(s_call_a == AIPO_OK);
    assert(res_a.tag == AIPO_VAL_INT);
    assert(res_a.int_val == 111);

    aipo_value_t res_b;
    memset(&res_b, 0, sizeof(res_b));
    aipo_status_t s_call_b = aipo_runtime_call(rt, "mod_b", "compute", NULL, 0, &res_b);
    assert(s_call_b == AIPO_OK);
    assert(res_b.tag == AIPO_VAL_INT);
    assert(res_b.int_val == 222);

    /* Verify mod_a still returns 111 */
    memset(&res_a, 0, sizeof(res_a));
    s_call_a = aipo_runtime_call(rt, "mod_a", "compute", NULL, 0, &res_a);
    assert(s_call_a == AIPO_OK);
    assert(res_a.int_val == 111);

    aipo_runtime_destroy(rt);
    printf("  [PASS] M1: Module-scoped function resolution preserves isolation without shadowing\n");
}

/* ------------------------------------------------------------- M2 */

static void test_m2_atomic_module_load_rollback(void) {
    aipo_runtime_t *rt = aipo_runtime_create();
    assert(rt != NULL);

    const char *src_good =
        "struct GoodPoint\n"
        "    x\n"
        "    y\n"
        "end\n"
        "fn get_answer()\n"
        "    return 42\n"
        "end\n";

    aipo_status_t s_good = aipo_runtime_load_module(rt, "good_mod", src_good);
    assert(s_good == AIPO_OK);

    /* Top-level fault (divide by zero) */
    const char *src_bad =
        "struct BadStruct\n"
        "    leak\n"
        "end\n"
        "let crash = 1 / 0\n";

    aipo_status_t s_bad = aipo_runtime_load_module(rt, "bad_mod", src_bad);
    assert(s_bad == AIPO_ERR_FAULT);

    /* Pre-existing good module still works */
    aipo_value_t res;
    memset(&res, 0, sizeof(res));
    aipo_status_t s_call = aipo_runtime_call(rt, "good_mod", "get_answer", NULL, 0, &res);
    assert(s_call == AIPO_OK);
    assert(res.int_val == 42);

    /* Faulty module was discarded */
    aipo_status_t s_bad_call = aipo_runtime_call(rt, "bad_mod", "get_answer", NULL, 0, &res);
    assert(s_bad_call == AIPO_ERR_USAGE);

    aipo_runtime_destroy(rt);
    printf("  [PASS] M2: Atomic module load rolls back VM definitions on top-level initialization fault\n");
}

/* ------------------------------------------------------------- S1 */

static void test_s1_instruction_budget_halts_infinite_loop(void) {
    aipo_runtime_t *rt = aipo_runtime_create();
    assert(rt != NULL);

    const char *src_loop =
        "fn spin_loop()\n"
        "    var counter = 0\n"
        "    while true\n"
        "        counter = counter + 1\n"
        "    end\n"
        "    return counter\n"
        "end\n"
        "fn quick_fn()\n"
        "    return 77\n"
        "end\n";

    aipo_status_t s_load = aipo_runtime_load_module(rt, "loop_mod", src_loop);
    assert(s_load == AIPO_OK);

    /* Configure budget of 100 instructions */
    aipo_status_t s_budget = aipo_runtime_set_instruction_budget(rt, 100);
    assert(s_budget == AIPO_OK);

    aipo_value_t res;
    memset(&res, 0, sizeof(res));

    aipo_status_t s_call = aipo_runtime_call(rt, "loop_mod", "spin_loop", NULL, 0, &res);
    assert(s_call == AIPO_ERR_FAULT);

    uint64_t count = aipo_runtime_instruction_count(rt);
    assert(count > 100);

    /* Reset counter */
    aipo_status_t s_reset = aipo_runtime_reset_instruction_count(rt);
    assert(s_reset == AIPO_OK);
    assert(aipo_runtime_instruction_count(rt) == 0);

    /* Remove budget limit (0 = unlimited) */
    s_budget = aipo_runtime_set_instruction_budget(rt, 0);
    assert(s_budget == AIPO_OK);

    s_call = aipo_runtime_call(rt, "loop_mod", "quick_fn", NULL, 0, &res);
    assert(s_call == AIPO_OK);
    assert(res.int_val == 77);

    aipo_runtime_destroy(rt);
    printf("  [PASS] S1: Instruction budget halts runaway execution and counter resets\n");
}

int main(void) {
    printf("=== Aipo C ABI Native Verification Harness ===\n");
    test_version();
    test_c1_bytes_survive_and_release();
    test_c2_callback_identity_wins_over_arity();
    test_c3_c4_validation_and_handle_create_checked();
    test_c5_reentrancy_guards();
    test_c6_null_args_positive_argc();
    test_c7_nul_string_length();
    test_m1_module_scope_isolation();
    test_m2_atomic_module_load_rollback();
    test_s1_instruction_budget_halts_infinite_loop();
    printf("=== All C ABI native harness assertions PASSED ===\n");
    return 0;
}
