# aipo-hir

`aipo-hir` defines the High-Level Intermediate Representation (HIR) and early semantics-preserving lowering passes for the Aipo language compiler.

## Architecture & Desugaring Passes
1. **Trailing Closures & Blocks**: Call expressions with trailing blocks are lowered into function literals passed as the final argument.
2. **Pipelines**: `left |> right(args)` is desugared into `right(left, args)` with left-hand expression passed as the first positional argument.
3. **Binding Destructuring**: `let [a, b] = val` and `let {x, y} = val` are expanded into synthetic intermediate bindings and individual indexed/property projections.
4. **Value Conditionals**: `if c then a else b` is preserved as a typed conditional expression.
5. **Safety**: `#![forbid(unsafe_code)]` with complete span preservation on all synthesized nodes.
