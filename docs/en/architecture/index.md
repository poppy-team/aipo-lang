# System Macroarchitecture

The architecture of **Aipo** adheres strictly to **Clean Architecture** principles: high cohesion, low coupling, and explicit domain boundaries between each layer of the compiler and runtime.

---

## Layered Architecture Diagram

```mermaid
graph TD
    subgraph Frontend["Compiler Frontend"]
        Source["Source (UTF-8 Boundary)"] --> Lexer["aipo-lexer"]
        Lexer --> Parser["aipo-syntax & aipo-ast"]
        Parser --> HIR["aipo-hir (Lowering)"]
        HIR --> Sema["aipo-sema (Validation & Contracts)"]
    end

    subgraph Midend["Intermediate Representation & Bytecode"]
        Sema --> IR["aipo-ir (Core IR)"]
        IR --> Bytecode["aipo-bytecode (Compact Instructions)"]
    end

    subgraph Execution["Execution Targets"]
        Bytecode --> VM["aipo-vm (Native Rust Virtual Machine)"]
        Bytecode --> AIBC["Binary Archive .aibc"]
        IR --> JS["aipo-js (JavaScript ES2022 Backend)"]
    end

    subgraph HostBridge["Host ABI & Sandboxing"]
        VM --> Host["aipo-host (Capabilities & Handles)"]
        Host --> Poppy["aipo-poppy (Simulation Engine)"]
        Host --> CLI["aipo-cli (Unified Command Line)"]
    end
```

---

## Fundamental Architectural Invariants

1. **Frontend & Backend Agnosticism**: AST and HIR maintain zero awareness of bytecode layout, virtual machine mechanics, or JavaScript runtime details.
2. **Result-Based Error Model**: No Rust panics or internal unhandled exceptions ever leak as Aipo user errors. All faults are categorized into stable, human-friendly diagnostic codes.
3. **Safe Structural Invariants**: Struct mutations are journaled and validated against declared invariants, guaranteeing that memory and object state are never corrupted.
4. **Zero Hidden Allocations in the Fast Path**: VM instruction dispatch and value decoding avoid redundant clones, deep copying, and unnecessary `Box` allocations during evaluation.
