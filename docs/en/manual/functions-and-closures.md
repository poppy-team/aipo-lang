# Functions, Closures & Lambdas

Functions in Aipo are first-class citizens: they can be passed as arguments, assigned to variables, returned from functions, and capture variables from lexical enclosing scopes.

---

## Function Declarations

Declared using the `fn` keyword:

```aipo
fn add(a, b) {
  return a + b
}

let sum = add(10, 20) // 30
```

Functions without an explicit `return` evaluate to `none`.

---

## Default Values & Named Arguments

Parameters can specify default values:

```aipo
fn connect(host, port = 8080, timeout = 5000) {
  print("Connecting to " + host + ":" + port + " with timeout " + timeout + "ms")
}

connect("localhost") // uses port 8080, timeout 5000
connect("db.internal", 5432) // uses port 5432, timeout 5000
```

Calls can also specify named arguments:

```aipo
connect("api.service", timeout = 1000)
```

---

## Anonymous Functions & Closures

Anonymous functions capture variables from enclosing scopes via shared upvalues:

```aipo
fn make_counter(start = 0) {
  let var count = start
  return fn() {
    count += 1
    return count
  }
}

let counter = make_counter(10)
print(counter()) // 11
print(counter()) // 12
```

---

## Concise Lambdas (`=>`)

For short single-expression functions:

```aipo
let numbers = [1, 2, 3, 4, 5]

// Map elements
let doubled = numbers.map(x => x * 2)

// Filter elements
let evens = numbers.filter(x => x % 2 == 0)
```

---

## Local Functions & Self-Recursion

Functions declared inside other functions support full self-recursion via `FillSelfCapture`:

```aipo
fn factorial(n) {
  fn loop_rec(current, acc) {
    if current <= 1 then
      return acc
    end
    return loop_rec(current - 1, acc * current)
  }

  return loop_rec(n, 1)
}

print(factorial(5)) // 120
```
