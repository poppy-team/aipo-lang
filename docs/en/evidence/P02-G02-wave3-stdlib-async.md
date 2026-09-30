# Evidence — P02-G02 / Wave 3 Stdlib: Combinadores Assíncronos e Operações de Task

**Goal:** `P02-G02` — Implementar e certificar combinadores assíncronos (`task.spawn`, `task.sleep`, `task.all`, `task.race`, `task.timeout`, `task.cancel`, `task.group`) com tempo virtual determinístico e paridade VM↔JS.
**Phase:** P02 (Wave 3 — Async & Expanded Types) · **Recorded:** 2026-09-20
**Environment:** Linux x86_64, rustc/cargo 1.98.1, Node v24.18.0

## Gates (all executed, all green)

```
$ cargo fmt --all -- --check                                           # 0
$ cargo clippy --workspace --all-targets -- -D warnings                 # 0
$ cargo test --workspace                                               # 100% passed, 0 failed
$ cargo doc --workspace --no-deps                                      # 0 (0 warnings)
$ node crates/aipo-js/runtime/selftest.mjs                             # all assertions passed
$ cargo test -p aipo-js --test differential                            # 8 passed (VM == JS == stdout)
$ prumo validate .                                                     # clean
$ prumo doctor .                                                       # clean
```

## Summary of Implementation

| Area | Decisions & Implementation Details |
|---|---|
| **Deterministic Scheduler** | Single-threaded, deterministic cooperative task scheduler implemented in both Rust VM (`aipo-vm/src/vm/task.rs`) and JavaScript runtime (`aipo-runtime.js`). Tasks execute depth-first with FIFO run queue. Virtual time (`tick`) advances strictly when tasks sleep or wait on deadlines — pure computation never advances the clock. |
| **`task.spawn`** | `task.spawn(fn, args: List) -> Task`. Spawns a new task initialized with its own frame, stack, and upvalue environment, queuing it onto `run_queue`. |
| **`await`** | Resolves completed tasks inline (fast path). For pending tasks, suspends the waiter with operand stack and instruction pointer intact, handing over execution directly to the awaited task before the rest of the queue. Cycle detection halts with `AIPO_RT_AWAIT_CYCLE`. Invocation inside synchronous host callbacks faults with `AIPO_RT_AWAIT_IN_CALLBACK`. Awaiting cancelled tasks faults with `AIPO_RT_CANCELLED`. |
| **`task.sleep`** | `task.sleep(duration_or_ticks)`. Accepts `Int`/`Byte` ticks or `Duration` (1s = 1000 ticks). Negative values produce recoverable `Failure`. `sleep(0)` yields to other queued tasks as a deterministic reschedule point. On resume, re-executing the call checks virtual time and resolves with `None`. |
| **`task.all`** | `task.all(tasks: List) -> List`. Structured join waiting for all members. First failure in completion order fails the join; otherwise outcomes are collected in original member order. If any member is cancelled, faults with `AIPO_RT_CANCELLED`. Empty list returns `[]`. |
| **`task.race`** | `task.race(tasks: List) -> Value`. First member completion in deterministic scheduler order wins; cancelled winners fault. Empty list returns `Failure("race of no tasks")`. |
| **`task.timeout`** | `task.timeout(task, deadline_or_duration) -> Value`. Resolves with task outcome if completed before deadline; on deadline expiry, marks join as `Failure("timeout")` and cancels the late task. Empty target returns `Failure("timeout of no task")`. |
| **`task.cancel`** | `task.cancel(task) -> None`. Marks task as `Cancelled`. Driving or awaiting a cancelled task faults deterministically (`AIPO_RT_CANCELLED`). |
| **`task.group`** | `task.group() -> Group`. Creates a structured-concurrency scope with `group.spawn(fn, args: List) -> Task` and `group.wait() -> List`. Late spawns dynamically join open group waits. Collects member outcomes in completion order. |
| **VM ↔ JS Backend Parity** | `crates/aipo-js/runtime/aipo-runtime.js` contains a 100% equivalent cooperative scheduler with exact state machine (`Pending`, `Running`, `Sleeping`, `Blocked`, `Ready`, `Failed`, `Cancelled`), join mechanics, timeout polling, and `case 'Await':` opcode dispatch. Evaluated and certified via `selftest.mjs` and `crates/aipo-js/tests/differential.rs`. |

## Test Inventory & Verification

- `crates/aipo-vm/tests/wave3_pipeline.rs`: End-to-end integration tests covering `spawn_and_await`, `all_and_race`, `group`, `sleep_and_timeout`, `timeout_expires`, and `cancel`.
- `crates/aipo-js/runtime/selftest.mjs`: Unit assertions for `vGroup`, `Group` equality and formatting, plus async execution with `runModule`.
- `crates/aipo-js/tests/differential.rs`: Differential suite compiling Aipo async programs to bytecode (for VM) and Core IR bundle (for Node.js), asserting exact stdout and status code equality across:
  - `test_wave3_async_spawn_await_differential`
  - `test_wave3_async_all_and_race_differential`
  - `test_wave3_async_group_differential`
  - `test_wave3_async_sleep_and_timeout_differential`
  - `test_wave3_async_cancel_differential`
