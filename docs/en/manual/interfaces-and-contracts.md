# Interfaces & Contracts

Aipo bridges dynamic flexibility with **signature contracts**, **structural invariants**, and **automatic structural interface subtyping**.

---

## Structs (`struct`)

Structures define aggregate types delimited by curly braces `{ ... }`.

By safe and predictable design default, **all fields in a struct are immutable**. When a field must be mutable during the instance lifecycle, declare it explicitly with the `var` keyword:

```aipo
struct Server {
    id
    created_at
    var status = "offline"
    var cpu_load = 0.0
}

# Structural instantiation using symmetric key-value ':'
let s = Server{
    id: "srv-1",
    created_at: 1600000000,
    status: "online",
    cpu_load: 0.42,
}

io.println(s.id)     # "srv-1"
io.println(s.status) # "online"
```

Reassigning an immutable field after construction triggers the static semantic diagnostic `AIPO_SEM_IMMUTABLE_FIELD_REASSIGN`.

---

## Construction Hook (`init`)

The `init` hook is declared inside an `impl StructName { ... }` block to validate, normalize, and initialize instance fields before publication:

```aipo
struct User {
    email
    name
}

impl User {
    init(email, name) {
        if not email.contains("@") {
            return fail("Invalid email address format")
        }
        self.email = email
        self.name = name
    }
}

let u = User{ email: "user@example.com", name: "Dev" }
io.println(u.email) # "user@example.com"
```

---

## Structural Invariants (`invariant`)

Invariants declare logical predicates inside the `impl` block that **must remain true throughout the lifetime of the object**:

```aipo
struct Interval {
    var start = 0
    var end_val = 0
}

impl Interval {
    init(start, end_val) {
        self.start = start
        self.end_val = end_val
    }

    invariant {
        self.start <= self.end_val
    }
}

let inter = Interval{ start: 5, end_val: 10 }
io.println(inter.start)   # 5
io.println(inter.end_val) # 10
```

Whenever a field is mutated, the invariant predicate is automatically re-evaluated. If it fails, the operation is rejected. Inside an `attempt { ... }` block, provisional mutations are automatically rolled back by the transaction journal.

---

## Methods and Universal Mutability (`var self`)

Methods associated with a type are defined inside `impl StructName { ... }` blocks.

By default, the `self` receiver is **read-only**. When a method needs to mutate internal instance state, it explicitly declares `var self`, harmonizing method mutability with the universal variable rules of the language:

```aipo
struct Counter {
    var count = 0
}

impl Counter {
    # Read-only method: self is immutable
    fn current(self) -> Int {
        return self.count
    }

    # Mutator method: var self explicitly signals state modification
    fn increment(var self) {
        self.count += 1
    }
}
```

---

## Interfaces and Automatic Structural Subtyping (`interface`)

Interfaces define method contracts. In Aipo, interface conformance requires no ceremony or orphan statements: **subtyping is structural and automatic** (inspired by modern languages like Go and Luau).

If a struct implements all methods required by an `interface` with compatible signatures and contracts (including receiver mutability `var self` vs `self`), it **automatically satisfies the interface**:

```aipo
interface Drawable {
    fn draw(self) -> String
}

struct Button {
    label
}

impl Button {
    fn draw(self) -> String {
        return f"[Button: {self.label}]"
    }
}

# Button automatically satisfies Drawable!
# No 'implements' keyword or orphan 'satisfy' statements required.

# Function accepting any type that fulfills the Drawable contract
fn render_element(item: Drawable) -> String {
    return item.draw()
}

let btn = Button{ label: "Submit" }
io.println(render_element(btn)) # "[Button: Submit]"
```

Conformance is verified statically by the semantic analyzer (`aipo-sema`), checking method existence, parameter arity, parameter and return types, and receiver mutability compatibility (`self` vs `var self`).
