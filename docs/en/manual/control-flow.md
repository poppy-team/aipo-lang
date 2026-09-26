# Control Flow & Failures

Aipo offers clean control structures combined with a transactional failure model.

---

## Conditionals

### `if ... then ... else` Expressions

The `if` statement can be used as a block statement or an inline value expression:

```aipo
// Block form
if score >= 90 then
  status = "Excellent"
else if score >= 70 then
  status = "Passed"
else
  status = "Needs Improvement"
end

// Inline expression form
let message = if is_active then "Online" else "Offline"
```

---

## Loops

### `while`

Executes the body while the condition is true:

```aipo
let var i = 0
while i < 10 do
  print(i)
  i += 1
end
```

### `loop`

Canonical infinite loop terminated via `break`:

```aipo
let var retries = 0
loop do
  retries += 1
  if check_connection() then
    break
  end
  if retries >= 5 then
    fail "Timeout after 5 retries"
  end
end
```

### `repeat`

Post-condition loop:

```aipo
let var count = 0
repeat do
  count += 1
until count >= 5
```

### `each`

Canonical iteration over collections (`List`, `Dict`, `Set`, `Sequence`):

```aipo
let fruits = ["Apple", "Banana", "Orange"]
each item in fruits do
  print(item)
end

// Iterating over dictionaries (keys in insertion order)
let settings = { "host": "127.0.0.1", "port": 5432 }
each key in settings do
  print(key + " => " + settings[key])
end
```

---

## Failure Model & Transactions (`attempt ... recover`)

Errors are raised explicitly via `fail` and caught with **transactional mutation journaling and automatic rollback**:

```aipo
struct Vault {
  var balance: Float,
  invariant() {
    self.balance >= 0.0
  }
}

let v = Vault { balance: 100.0 }

attempt
  v.balance -= 200.0 // Violates Vault invariant!
recover err
  print("Caught failure: " + err)
end

// Because the block failed, the mutation was rolled back!
print("Recovered balance: " + v.balance) // 100.0
```
