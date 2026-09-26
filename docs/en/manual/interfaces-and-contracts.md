# Interfaces & Contracts

Aipo bridges dynamic flexibility with **signature contracts**, **structural invariants**, and **formal interfaces**.

---

## Structs (`struct`)

Structures define aggregate types closed with `end` (no curly braces). By default, fields are mutable unless declared with `fixed`:

```aipo
struct Server
    fixed id
    fixed created_at
    status = "offline"
    cpu_load = 0.0
end

let s = Server{
    id = "srv-1",
    created_at = 1600000000,
    status = "online",
    cpu_load = 0.42,
}

io.println(s.id)     # "srv-1"
io.println(s.status) # "online"
```

Reassigning a `fixed` field after construction generates the compile-time diagnostic `AIPO_SEM_FIXED_REASSIGN`.

---

## Construction Hook (`init`)

The `init` hook is declared inside an `impl StructName` block to validate and normalize instance fields before publication:

```aipo
struct User
    email
    name
end

impl User
    init(email, name)
        if not email.contains("@")
            return fail("Invalid email address format")
        end
        self.email = email
        self.name = name
    end
end

let u = User{email = "user@example.com", name = "Dev"}
io.println(u.email) # "user@example.com"
```

---

## Structural Invariants (`invariant`)

Invariants declare logical predicates inside the `impl` block that **must remain true throughout the lifetime of the object**:

```aipo
struct Interval
    start = 0
    end_val = 0
end

impl Interval
    init(start, end_val)
        self.start = start
        self.end_val = end_val
    end

    invariant()
        self.start <= self.end_val
    end
end

let inter = Interval{start = 5, end_val = 10}
io.println(inter.start)   # 5
io.println(inter.end_val) # 10
```

Whenever a field is mutated, the invariant is checked. Violations trigger automatic rollback of provisional mutations inside `attempt` blocks.

---

## Interfaces & Conformance (`interface` / `satisfy`)

Interfaces define method contracts. Conformance in Aipo is structural and declared explicitly with `satisfy` (not inheritance):

```aipo
interface Drawable
    fn draw(self) -> String
end

struct Button
    label
end

impl Button
    fn draw(self) -> String
        return f"[Button: {self.label}]"
    end
end

# Canonical structural conformance declaration
satisfy Button: Drawable

# Function requiring any value satisfying Drawable contract
fn render_element(item: Drawable) -> String
    return item.draw()
end

let btn = Button{label = "Submit"}
io.println(render_element(btn)) # "[Button: Submit]"
```

The semantic analyzer verifies arity, parameter names, receiver compatibility (`self` vs mutable `self!`), and async signatures before execution.

