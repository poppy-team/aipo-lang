# Concurrency & Async

Aipo's concurrency model is **cooperative, deterministic, and virtual-time driven**, eliminating low-level data races and flaky test executions.

---

## Asynchronous Functions (`async fn`)

Functions executing I/O, timers, or cross-process calls must be declared with `async`:

```aipo
async fn fetch_resource(id) {
  let timer = task.sleep(50)
  await do
    timer
  end
  return "Data for " + id
}
```

---

## Sequential Wait Blocks (`await do ... end`)

Rather than allowing arbitrary `await` calls in subexpressions, Aipo mandates explicit `await do ... end` blocks:

```aipo
async fn run_pipeline() {
  let task_a = task.spawn(fn() { step_one() })
  let task_b = task.spawn(fn() { step_two() })

  // Explicit sequential wait
  await do
    task_a
    task_b
  end

  print("All steps completed!")
}
```

This prevents forgotten tasks (`AIPO_SEM_FORGOTTEN_TASK`) and race conditions.

---

## Async Combinators (`task.*`)

The standard library includes primitives:

- **`task.spawn(fn)`**: Spawns a cooperative fiber in the scheduler.
- **`task.sleep(ms)`**: Suspends the current task using deterministic virtual time.
- **`task.all(list)`**: Awaits completion of all tasks in the list.
- **`task.race(list)`**: Resolves when the first task finishes and cancels the rest.
- **`task.timeout(task, ms)`**: Cancels the task if it exceeds the duration.
- **`task.cancel(task)`**: Cooperatively cancels an active task.
- **`task.group()`**: Creates a structured task group for coordinated lifecycles.

---

## Cycle Detection (`AIPO_RT_AWAIT_CYCLE`)

The runtime tracks dependencies between awaiting tasks. Mutual dependencies trigger `AIPO_RT_AWAIT_CYCLE` deterministically instead of stalling indefinitely.
