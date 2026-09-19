# aipo-syntax

`aipo-syntax` implements recursive descent statement/item parsing, Pratt precedence-climbing expression parsing, and error recovery for Aipo V1.

## Architecture & Guarantees
- **Pratt Parsing**: Correct associativity and precedence for arithmetic, pipeline (`|>`), fallback (`or_else`), logical, comparison, and unary operators.
- **Syntactic Sugar**: Seamlessly parses trailing blocks (`callee() do ... end`), shallow destructuring (`let [a, b] = ...`, `let {x, y} = ...`), and pipeline expressions.
- **Robust Recovery**: Never panics on syntax errors; synchronizes at statement/item boundaries and continues emitting high-quality diagnostics.
- **Safety**: `#![forbid(unsafe_code)]`.
