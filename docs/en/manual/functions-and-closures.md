# Functions, Closures & Lambdas

Functions in Aipo are first-class citizens: they can be passed as arguments, assigned to variables, returned from functions, and capture variables from lexical enclosing scopes.

---

## Function Declarations

Functions are introduced with the `fn` keyword and closed with `end` (no curly braces):

```aipo
fn add(a, b)
    return a + b
end

let total = add(10, 20)
io.println(total) # 30
```

Functions without an explicit `return` or returning with no value evaluate to `none`.

---

## Default Values & Named Arguments

Parameters can specify default values:

```aipo
fn connect(host, port = 8080, timeout = 5000)
    io.println(f"Connecting to {host}:{port} with timeout {timeout}ms")
end

connect("localhost")           # uses port 8080, timeout 5000
connect("db.internal", 5432)   # uses port 5432, timeout 5000
```

Calls can also specify named arguments for clarity and flexibility:

```aipo
connect("api.service", timeout = 1000)
```

---

## Anonymous Functions & Closures

Anonymous functions (`fn(params) ... end`) capture variables from enclosing scopes via shared, safe upvalues:

```aipo
fn make_counter(start = 0)
    var count = start
    return fn()
        count += 1
        return count
    end
end

let counter = make_counter(10)
io.println(counter()) # 11
io.println(counter()) # 12
```

Aipo's VM implements safe shared upvalues, ensuring that mutations in captured variables reflect correctly across multiple closures.

---

## Concise Lambdas (`=>`)

For short single-expression functions (especially useful in collection pipelines):

```aipo
let numbers = [1, 2, 3, 4, 5]

# Doubling elements with single-parameter lambda
let doubled = numbers.map(x => x * 2)
io.println(doubled) # [2, 4, 6, 8, 10]

# Filtering even numbers
let evens = numbers.filter(x => x % 2 == 0)
io.println(evens) # [2, 4]
```

Multi-parameter lambdas wrap parameters in parentheses: `(a, b) => a + b`.

---

## Local Functions & Self-Recursion

Functions declared inside other functions support full self-recursion via `FillSelfCapture`:

```aipo
fn factorial(n)
    fn loop_rec(current, acc)
        if current <= 1
            return acc
        end
        return loop_rec(current - 1, acc * current)
    end

    return loop_rec(n, 1)
end

io.println(factorial(5)) # 120
```

