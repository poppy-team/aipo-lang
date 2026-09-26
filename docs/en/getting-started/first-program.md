# Your First Program in 5 Minutes

In this quick walkthrough, you will write, verify, compile, and execute your first program in Aipo.

---

## 1. Hello, World!

Create a file named `hello.aipo`:

```aipo
# hello.aipo
io.println("Hello from Aipo!")
```

Run it directly with the CLI:

```bash
aipo run hello.aipo
```

**Expected output:**
```
Hello from Aipo!
```

---

## 2. Structs, Invariants, and Methods

Let's model a bank account with an invariant guaranteeing a non-negative balance. Create `account.aipo`:

```aipo
# account.aipo
struct Account
    holder
    balance = 0.0
    fixed account_number
end

impl Account
    init(holder, balance = 0.0, account_number = 0)
        self.holder = holder
        self.balance = balance
        self.account_number = account_number
    end

    # Invariant: executed upon construction and every field mutation
    invariant()
        self.balance >= 0.0
    end

    fn deposit(self!, amount: Float)
        if amount <= 0.0
            return fail("Deposit amount must be positive")
        end
        self.balance = self.balance + amount
    end

    fn withdraw(self!, amount: Float)
        if amount <= 0.0
            return fail("Withdrawal amount must be positive")
        end
        
        # Tries to execute the withdrawal. If self.balance >= 0.0 fails,
        # the attempt block rolls back the mutation automatically!
        attempt
            self.balance = self.balance - amount
        failed err
            return fail("Withdrawal rejected: insufficient balance")
        end
    end
end

# Instantiating the account
let acc = Account{holder = "Alice Johnson", balance = 150.0, account_number = 1042}

io.println(f"Account created for: {acc.holder}")
io.println(f"Initial balance: {acc.balance}")

acc.deposit(50.0)
io.println(f"Balance after deposit: {acc.balance}")

acc.withdraw(75.0)
io.println(f"Balance after withdrawal: {acc.balance}")
```

Run the program:

```bash
aipo run account.aipo
```

**Expected output:**
```
Account created for: Alice Johnson
Initial balance: 150.0
Balance after deposit: 200.0
Balance after withdrawal: 125.0
```

---

## 3. Inspecting Bytecode

Aipo provides full transparency into compiler output. You can disassemble the bytecode for any file:

```bash
aipo disasm account.aipo
```

The output shows bytecode mnemonics annotated with corresponding source line and column numbers.

---

## 4. Compiling to JavaScript

You can transpile the exact same program to JavaScript for Node.js or browser execution:

```bash
aipo build account.aipo -o dist/account.js
node dist/account.js
```

The emitted JavaScript provides bit-for-bit behavioral parity with the native Rust VM.
