# Concurrency & Async

Aipo's concurrency model is **cooperative, deterministic, and virtual-time driven**, eliminating low-level data races and flaky test executions.

---

## Asynchronous Functions (`async fn`)

Functions executing timers, cooperative I/O, or asynchronous orchestration are declared with `async fn`. Invoking an async function eagerly schedules it on the cooperative scheduler and returns its `Task` handle:

```aipo
async fn fetch_resource(id)
    # task.sleep is a virtual-time cooperative suspension primitive
    task.sleep(50)
    return f"Data for {id}"
end

# Invocation spawns the task and returns a Task handle
let task_handle = fetch_resource("users")

# Explicitly awaits completion and unwraps the result
let data = await task_handle
io.println(data) # "Data for users"
```

---

## Sequential Wait Blocks (`await do ... end`)

Rather than allowing uncontrolled `await` expressions scattered inside complex subexpressions, Aipo mandates explicit `await do ... end` blocks for clear, predictable sequential sequencing:

```aipo
async fn fetch_step_1()
    return 10
end

async fn fetch_step_2()
    return 20
end

async fn run_pipeline()
    await do
        let a = await fetch_step_1()
        let b = await fetch_step_2()
        return a + b
    end
end

let total = await run_pipeline()
io.println(f"Total accumulated: {total}") # 30
```

This engineering discipline prevents unmonitored dangling promises and unobserved tasks (`AIPO_SEM_FORGOTTEN_TASK`).

---

## Async Combinators (`task.*`)

The standard library provides high-level primitives:

- **`task.spawn(callable, args_list)`**: Spawns a new concurrent task in the scheduler with the provided arguments.
- **`task.sleep(ms)`**: Suspends the current task for the specified virtual-time clock ticks.
- **`task.all(tasks_list)`**: Awaits completion of all tasks in the list, returning a list of results.
- **`task.race(tasks_list)`**: Resolves when the first task finishes, cooperatively cancelling remaining candidates.
- **`task.timeout(task, ms)`**: Cancels the target task if it exceeds the specified virtual-time duration.
- **`task.cancel(task)`**: Cooperatively cancels an active task handle.
- **`task.group()`**: Creates a structured task group for coordinated lifecycles and cascading cancellation.

```aipo
let t1 = fetch_step_1()
let t2 = fetch_step_2()

# Awaits both tasks in deterministic concurrency
let results = task.all([t1, t2])
io.println(results) # [10, 20]
```

---

## Cycle Detection (`AIPO_RT_AWAIT_CYCLE`)

The Aipo runtime tracks dependencies between awaiting tasks. Mutual dependencies trigger the deterministic fault `AIPO_RT_AWAIT_CYCLE` with a full trail of involved identifiers instead of stalling the process indefinitely.

