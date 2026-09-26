# Interfaces & Contracts

Aipo bridges dynamic flexibility with **signature contracts**, **structural invariants**, and **formal interfaces**.

---

## Structs (`struct`)

Structures define aggregate types. Fields are immutable unless declared with `var`, or strictly immutable with `fixed`:

```aipo
struct Server {
  fixed id: String,        // Immutable after construction
  fixed created_at: Int,   // Immutable after construction
  var status: String,      // Mutable
  var cpu_load: Float      // Mutable
}
```

Reassigning a `fixed` field after construction generates the compile-time fault `AIPO_SEM_FIXED_REASSIGN`.

---

## Construction Hook (`init`)

Validates and normalizes instances immediately upon instantiation:

```aipo
struct User {
  email: String,
  name: String,

  init() {
    if not self.email.contains("@") then
      fail "Invalid email address format"
    end
  }
}
```

---

## Structural Invariants (`invariant`)

Predicates that must hold true throughout the lifetime of the object:

```aipo
struct Interval {
  var start: Int,
  var end: Int,

  invariant() {
    self.start <= self.end
  }
}
```

Whenever a field is mutated, the invariant is checked. Violations revert mutations atomically inside `attempt` blocks.

---

## Interfaces & Implementations (`interface` / `satisfy`)

Interfaces define method contracts that structs implement:

```aipo
interface Drawable {
  fn render(ctx) -> None
  fn get_bounds() -> List
}

struct Button {
  label: String
}

impl Drawable for Button {
  fn render(ctx) {
    ctx.draw_text(self.label)
  }

  fn get_bounds() {
    return [0, 0, 100, 30]
  }
}
```

The semantic analyzer verifies arity, parameter names, `self` receiver alignment, and async signatures before execution.
