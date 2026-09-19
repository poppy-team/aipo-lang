# 02 — Compiladores, Bytecode, VM e Garbage Collection

<aside>
⚙️

**Baseline atual:** frontend handwritten → Semantic AST/Core IR pequeno → stack bytecode → VM Odin → tracing GC. Esta página reúne os materiais que devem guiar a implementação e as otimizações posteriores.

</aside>

## Guia prático principal

### Crafting Interpreters — Robert Nystrom

- [Livro completo](https://craftinginterpreters.com/)
- [Table of Contents](https://craftinginterpreters.com/contents.html)
- [A Bytecode Virtual Machine](https://craftinginterpreters.com/a-bytecode-virtual-machine.html)
- [A Virtual Machine](https://craftinginterpreters.com/a-virtual-machine.html)
- [Garbage Collection](https://craftinginterpreters.com/garbage-collection.html)
- [Strings](https://craftinginterpreters.com/strings.html)

**Prioridade:** P0 durante a primeira implementação. O percurso de bytecode, VM, Value, strings, hash tables, calls, closures e mark-sweep é muito próximo do que precisamos, adaptado a Odin e à semântica Aipo.

**Nota importante:** implementar o GC cedo; ele é uma preocupação transversal e fica muito mais difícil de adicionar após referências heap-managed se espalharem pela VM.

## Frontend e parsing

### Vaughan Pratt — Top Down Operator Precedence

- [Paper original — DOI](https://doi.org/10.1145/512927.512931)

**Uso:** base conceitual do Pratt parser para expressões e precedência.

### Odin como implementação de referência

- [Odin regex Pratt parser](https://pkg.odin-lang.org/core/text/regex/parser/)
- [Odin tokenizer](https://pkg.odin-lang.org/core/odin/tokenizer/)
- [Odin parser para tooling](https://pkg.odin-lang.org/core/odin/parser/)

**Uso:** exemplos reais em Odin para scanner, spans, diagnostics, Pratt e organização do parser.

## Arquitetura de compiladores

### Engineering a Compiler — Cooper & Torczon, 3ª edição

- [Página oficial Elsevier](https://shop.elsevier.com/books/engineering-a-compiler/cooper/978-0-12-815412-0)

**Uso:** referência acadêmica para semantic analysis, IR, runtime support, code shape, dataflow e otimização. Não usar o livro como justificativa para inflar a Aipo com SSA/LLVM antes de necessidade real.

## Bytecode e arquitetura da VM

### The Implementation of Lua 5.0

- [Paper](https://www.jucs.org/jucs_11_7/the_implementation_of_lua.html)

**Uso:** register VM, activation records, tables, closures e coroutines. Comparar futuramente com nossa stack VM.

### Virtual Machine Showdown: Stack versus Registers

- [ACM/DOI — versão ampliada](https://doi.org/10.1145/1328195.1328197)

**Uso:** quantificar trade-offs de stack vs register VM. Aipo começa stack-based por simplicidade; mudar só após profiling.

### Wren Performance

- [Wren Performance](https://wren.io/performance.html)

**Uso:** bytecode interpreter, dispatch, single-pass compiler como contraste e custo de JIT.

### Luau Performance

- [How we make Luau fast](https://luau.org/performance/)

**Uso:** inline caching, specialized builtins, allocator, GC pacing, table/string performance, closure caching e otimizações de bytecode. Consultar após a VM correta e benchmarkada.

### Odin regex bytecode/VM

- [Bytecode compiler](https://pkg.odin-lang.org/core/text/regex/compiler/)
- [Threaded VM](https://pkg.odin-lang.org/core/text/regex/virtual_machine/)

**Uso:** representação de opcodes, operands, jumps e interpreter loop em Odin. É referência de implementação, não arquitetura direta da Aipo.

## Garbage Collection

### The Garbage Collection Handbook — Jones, Hosking & Moss, 2ª ed.

- [Site oficial / edições](https://gchandbook.org/editions.html)

**Uso:** referência acadêmica principal de tracing GC, mark-sweep, tri-color marking, write barriers, incremental/concurrent/generational GC, weak refs e finalização.

### Lua GC

- [Lua 5.5 — Garbage Collection](https://www.lua.org/manual/5.5/manual.html#2.5)

**Uso:** comportamento incremental/generational em uma scripting language pequena.

### Luau GC e allocator

- [Luau Performance](https://luau.org/performance/)

**Uso:** pacing, redução de pausas e sweeping paginado, especialmente para game workloads.

### Crafting Interpreters — Mark/Sweep

- [Garbage Collection](https://craftinginterpreters.com/garbage-collection.html)

**Uso:** implementação do primeiro collector correto e stress-GC.

## Referências históricas de memória

- [Lobster — Memory Management](https://aardappel.github.io/lobster/memory_management.html)
- [Lean — Reference Counting](https://lean-lang.org/doc/reference/latest/Run-Time-Code/Reference-Counting/)
- [Perceus](https://doi.org/10.1145/3453483.3454032)

**Status:** pesquisa comparativa. Não substituir o tracing GC atual sem benchmark/problema real.

## Ordem recomendada de consulta durante implementação

1. Crafting Interpreters: scanner/parser/VM/GC conforme a fase.
2. Odin core: implementação equivalente na linguagem host.
3. Lua/Wren: design de VM/embedding.
4. GC Handbook: quando o primeiro mark-sweep funcionar e precisarmos evoluir incremental/pacing.
5. Luau: somente quando houver dados reais de performance.
6. VM Showdown: apenas se dispatch/bytecode aparecerem como gargalo mensurável.