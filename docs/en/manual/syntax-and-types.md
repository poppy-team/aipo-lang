# Syntax & Data Types

Aipo is dynamically and strongly typed. Variables hold values with concrete types, and operations never perform silent implicit coercions.

---

## Primitive Types

### Integers (`Int`)
64-bit signed integers (`i64`). Supports decimal, hexadecimal (`0x`), binary (`0b`), and octal (`0o`) notation, plus underscores for readability:

```aipo
let x = 42
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
Boolean truth values (`true` and `false`):

```aipo
let is_active = true
let is_ready = false
```

### Text (`String`)
Canonical UTF-8 strings with automatic Unicode NFC normalization at construction boundaries:

```aipo
let name = "Aipo"
let greeting = "Hello, " + name + "!"
```

### Null Value (`None`)
Represents the explicit absence of a value (`none`):

```aipo
let optional_value = none
```

---

## Collections

### Lists (`List`)
Ordered dynamic arrays indexed from 0:

```aipo
let items = [1, 2, 3, 4]
print(items[0]) // 1
items.push(5)
```

Higher-order collection methods: `map`, `flat_map`, `reduce`, `any`, `all`, `filter`.

### Dictionaries (`Dict`)
Key-value mappings that preserve insertion order:

```aipo
let config = {
  "host": "localhost",
  "port": 8080
}
print(config["port"]) // 8080
```

### Sets (`Set`)
Collections of unique elements that retain insertion order:

```aipo
let s = Set()
s.add("alpha")
s.add("beta")
s.add("alpha") // Ignored, duplicate
print(s.contains("beta")) // true
```

### Lazy Sequences (`Sequence`)
On-demand evaluated generator sequences without allocating intermediate collections.

---

## Binary Data (`Bytes`)

Contiguous byte buffers engineered for high-performance network protocols and binary I/O:

```aipo
let b = Bytes(16) // Allocates 16 zeroed bytes
b.write_u32_le(0, 42)
let val = b.read_u32_le(0)
```

---

## Operators & Precedence

- **Arithmetic**: `+`, `-`, `*`, `/`, `%`
- **Comparison**: `==`, `!=`, `<`, `<=`, `>`, `>=`
- **Logical**: `and`, `or`, `not`
- **Safe Navigation**: `?.` (evaluates left-hand side; if `none`, avoids accessing fields or evaluating method arguments)
- **Lazy Fallback**: `or_else`
- **Pipe Operator**: `|>` for expressive call chaining:

```aipo
let result = value
  |> normalize
  |> validate
  |> persist
```
