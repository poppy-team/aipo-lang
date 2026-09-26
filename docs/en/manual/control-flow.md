# Control Flow & Failures

Aipo offers clean, expressive control structures combined with a transactional failure model featuring automatic atomic rollback.

---

## Conditionals

### Block Form `if ... elif ... else ... end`

In standard block form, branches do not use braces. Intermediate branches use `elif`, and the block is closed with `end`:

```aipo
var score = 85
var status = ""

if score >= 90
    status = "Excellent"
elif score >= 70
    status = "Passed"
else
    status = "Needs Improvement"
end

io.println(status) # "Passed"
```

### Inline Expression `if condition then a else b`

Aipo also supports single-line value conditional expressions using `then`:

```aipo
let is_active = true
let message = if is_active then "Online" else "Offline"
io.println(message) # "Online"
```

---

## Pattern Matching (`match ... when`)

The `match` construct compares an expression against one or more patterns per branch:

```aipo
let status = "approved"

match status
when "pending"
    io.println("Awaiting confirmation...")
when "approved", "completed"
    io.println("Operation finished successfully!")
else
    io.println("Unrecognized status")
end
```

---

## Loops

All loop constructs in Aipo terminate with the `end` keyword and **never use** the word `do`.

### `while`

Executes the body while the boolean condition evaluates to true:

```aipo
var i = 0
while i < 3
    io.println(f"Step: {i}")
    i += 1
end
```

### `loop`

Canonical continuous loop, designed for cycles that terminate via an explicit `break`:

```aipo
var retries = 0
loop
    retries += 1
    if retries >= 3
        break
    end
end
io.println(f"Total retries: {retries}")
```

### `repeat`

Repeats a block a fixed number of times with an optional iteration index (`repeat count as index`):

```aipo
# Executes 3 times (with indices 0, 1, and 2)
repeat 3 as idx
    io.println(f"Iteration number: {idx}")
end
```

### `each`

Canonical iteration over collections (`List`, `Dict`, `Set`, `Sequence`):

```aipo
# Simple list iteration
let fruits = ["Apple", "Banana", "Orange"]
each fruit in fruits
    io.println(fruit)
end

# Iteration with index and element
each idx, fruit in fruits
    io.println(f"{idx}: {fruit}")
end

# Iteration over dictionary (key and value in insertion order)
let config = {
    "host": "127.0.0.1",
    "port": 5432,
}
each key, val in config
    io.println(f"{key} => {val}")
end
```

---

## Failure Model & Transactions (`attempt ... failed`)

In Aipo, errors are neither uncontrolled stack-unwinding exceptions nor easily ignored status codes. Failures are raised explicitly with `fail` (or returned with `return fail(...)`) and handled in transactional blocks with **journaling and automatic atomic rollback**:

```aipo
struct Vault
    balance = 0.0
end

impl Vault
    invariant()
        self.balance >= 0.0
    end
end

let v = Vault{balance = 100.0}

attempt
    # This mutation temporarily drops balance to -100.0
    v.balance = v.balance - 200.0
failed err
    # Violating the invariant triggers automatic rollback restoring self.balance to 100.0!
    io.println(f"Caught failure: {err.message}")
end

# The balance remains untouched at its pre-attempt state!
io.println(f"Preserved balance: {v.balance}") # 100.0
```

If any operation inside an `attempt` block executes `fail` or violates a structural invariant, all mutations to journaled objects are atomically reverted to their original state.

