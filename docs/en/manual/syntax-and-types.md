# Syntax & Data Types

Aipo is dynamically and strongly typed. Variables hold values with concrete types, and operations never perform dangerous silent implicit coercions.

---

## Variables and Immutability

Aipo expressly distinguishes immutable bindings from mutable variables:

- **`let` (Immutable)**: Defines a local constant. Once bound, the identifier cannot be reassigned.
- **`var` (Mutable)**: Defines a variable that can be reassigned (`=`, `+=`, `-=`, etc.).

```aipo
# Immutable binding: attempts to reassign trigger a static check error
let language = "Aipo"

# Mutable variable: value can change throughout execution
var counter = 0
counter += 1
counter = counter * 2
io.println(counter) # 2
```

---

## Primitive Types

### Integers (`Int`)
64-bit signed integers (`i64`). Supports decimal, hexadecimal (`0x`), binary (`0b`), and octal (`0o`) notation, plus underscores for visual grouping:

```aipo
let decimal = 42
let hex = 0x2A
let bin = 0b101010
let million = 1_000_000
```

### Floats (`Float`)
IEEE 754 64-bit double-precision floating-point numbers:

```aipo
let pi = 3.14159
let rate = 0.05
```

### Booleans (`Bool`)
Pure logical truth values (`true` and `false`):

```aipo
let is_active = true
let is_ready = false
```

### Text (`String`)
Canonical UTF-8 strings with automatic Unicode NFC normalization at construction boundaries. Supports formatted interpolation with the `f"..."` prefix and concatenation via `+`:

```aipo
let version = "0.1.0"
let message = f"Welcome to Aipo v{version}!"
io.println(message)
```

### Null Value (`None`)
Represents the explicit absence of a value (`none`):

```aipo
let optional_value = none
```

---

## Collections

### Lists (`List`)
Ordered dynamic arrays indexed from 0. Adding elements to a list uses `.add()` (not `push`):

```aipo
var items = [1, 2, 3]
io.println(items[0]) # 1

# Append an element to the end
items.add(4)
io.println(items.len()) # 4

# Check containment
io.println(items.contains(3)) # true
```

Higher-order collection methods available in stdlib and VM: `map`, `flat_map`, `reduce`, `any`, `all`, `filter`, `sort`, `sort_by`.

### Dictionaries (`Dict`)
Indexable key-value mappings that preserve original insertion order. Checking existence is done via `.has(key)`:

```aipo
let config = {
    "host": "localhost",
    "port": 8080,
}

if config.has("port")
    io.println(f"Connecting to port: {config[\"port\"]}")
end
```

::: tip 💡 Why is there no `dict.get()` method?
Aipo follows the canonical architectural decision (ADP-001) to eliminate an ambiguous `get()` method that could not distinguish a missing key from a key whose stored value is `none`. Instead, use `dict.has(key)` for explicit presence checks and `dict[key]` for direct access.
:::

### Sets (`Set`)
Collections of unique elements that preserve the insertion order of first arrival:

```aipo
var s = Set()
s.add("alpha")
s.add("beta")
s.add("alpha") # Ignored: duplicates are discarded

io.println(s.has("beta")) # true (uses .has(), not .contains())
io.println(s.len())        # 2
```

### Lazy Sequences (`Sequence`)
On-demand evaluated generator sequences that process data without allocating intermediate collections in memory. Created by calling `.lazy()` on any collection:

```aipo
let seq = [1, 2, 3, 4, 5].lazy()
    .filter(x => x % 2 != 0)
    .map(x => x * 10)

let result = seq.to_list()
io.println(result) # [10, 30, 50]
```

---

## Binary Data (`Bytes`)

Contiguous byte buffers engineered for high-performance network protocols and binary I/O:

```aipo
let b = Bytes(16) # Allocates 16 zeroed bytes
b.write_u32_le(0, 42)
let val = b.read_u32_le(0)
io.println(val) # 42
```

---

## Operators & Precedence

- **Arithmetic**: `+`, `-`, `*`, `/`, `//` (truncated integer division), `%` (modulo)
- **Comparison**: `==`, `!=`, `<`, `<=`, `>`, `>=`
- **Logical**: `and`, `or`, `not`
- **Safe Navigation**: `?.` (evaluates left-hand side; if `none`, avoids accessing fields or evaluating method arguments)
- **Lazy Fallback**: `or_else` (evaluates right-hand side only if left-hand side fails or is null)
- **Pipe Operator**: `|>` for call chaining:

```aipo
let result = value
    |> normalize
    |> validate
    |> persist
```

