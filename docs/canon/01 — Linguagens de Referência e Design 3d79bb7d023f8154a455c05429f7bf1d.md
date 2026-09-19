# 01 — Linguagens de Referência e Design

<aside>
🧭

**Objetivo:** saber qual linguagem consultar para cada decisão da Aipo. Prioridade não significa copiar sintaxe; significa reutilizar experiência comprovada.

</aside>

## Referências primárias

### Lua

**Papel na Aipo:** principal referência de identidade como linguagem pequena, eficiente, embutível, dinâmica e gerenciada; referência de VM, GC, coroutines, tabelas e host API.

- [Lua — documentação oficial](https://www.lua.org/docs.html)
- [Lua 5.5 Reference Manual](https://www.lua.org/manual/5.5/)
- [Lua 5.5 — Garbage Collection](https://www.lua.org/manual/5.5/manual.html#2.5)
- [Lua 5.5 — Coroutines](https://www.lua.org/manual/5.5/manual.html#2.6)
- [The Implementation of Lua 5.0](https://www.jucs.org/jucs_11_7/the_implementation_of_lua.html) — paper sobre register VM, closures, tables e coroutines.

**Consultar quando:** VM, GC incremental/generational, embedding, tabelas, closures, coroutines, portabilidade e filosofia de linguagem pequena.

### Luau

**Papel na Aipo:** principal referência de scripting para games, bytecode eficiente, otimizações pragmáticas, GC pacing, redução de pausas, buffers compactos e separação entre runtime e host.

- [Luau — site/documentação](https://luau.org/)
- [Luau — Performance](https://luau.org/performance/)
- [Luau — Standard Library](https://luau.org/library/)
- [Luau — Compatibility](https://luau.org/compatibility/)
- [Luau — Embedding/API](https://luau.org/api/)

**Consultar quando:** otimizar VM, GC para game loop, buffers, arrays/tables, inline caching, host APIs e sandboxing.

### Odin

**Papel na Aipo:** linguagem oficial de implementação do compilador, VM e runtime.

- [Odin — documentação oficial](https://odin-lang.org/docs/)
- [Odin — Overview](https://odin-lang.org/docs/overview/)
- [Odin — Running Tests](https://odin-lang.org/docs/testing/)
- [Odin Package Documentation](https://pkg.odin-lang.org/)

**Consultar quando:** qualquer decisão de implementação host, allocators, arenas, FFI, testes, threads e integração com bibliotecas.

### Wren

**Papel na Aipo:** referência de VM pequena, stack bytecode, fibers e API de embedding cuidadosamente desenhada.

- [Wren](https://wren.io/)
- [Embedding Wren](https://wren.io/embedding/)
- [Configuring the VM](https://wren.io/embedding/configuring-the-vm.html)
- [Wren Performance](https://wren.io/performance.html)
- [Wren Concurrency/Fibers](https://wren.io/concurrency.html)

**Consultar quando:** desenho da API host↔VM, rooting de objetos, fibers, configuração de heap e simplicidade de runtime.

## Referências secundárias

### Janet

**Papel:** fibers/event loop, embedding e relação entre linguagem segura e C API não segura.

- [Janet](https://janet-lang.org/)
- [Janet C API](https://janet-lang.org/capi/index.html)
- [Janet Event Loop](https://janet-lang.org/docs/event_loop.html)

**Consultar quando:** `async/await`, scheduler cooperativo, event loop e limites de FFI.

### Haxe

**Papel:** referência para uma única linguagem com múltiplos backends, especialmente JavaScript.

- [Haxe Documentation](https://haxe.org/documentation/)
- [Haxe Compiler Targets](https://haxe.org/documentation/introduction/compiler-targets.html)
- [Haxe Language Introduction](https://haxe.org/documentation/introduction/language-introduction.html)

**Consultar quando:** Core IR/backend separation, diferenças semânticas entre targets, geração de JavaScript e portabilidade.

### Kotlin

**Papel:** referência de builders/DSLs baseados em funções, trailing lambdas e controle de receivers.

- [Kotlin Type-safe Builders](https://kotlinlang.org/docs/type-safe-builders.html)
- [Kotlin DslMarker](https://kotlinlang.org/api/core/kotlin-stdlib/kotlin/-dsl-marker/)

**Consultar quando:** HTML/CSS/UI/scene/config DSLs, principalmente se receiver implícito voltar à discussão.

### Python

**Papel:** ergonomia de `async/await` e structured concurrency como referência de API, não de VM.

- [Python asyncio](https://docs.python.org/3/library/asyncio.html)
- [Python Coroutines and Tasks](https://docs.python.org/3/library/asyncio-task.html)

**Consultar quando:** desenho público de `async`, `await`, task groups, cancelamento e timeouts.

### AngelScript

**Papel:** scripting embutível orientado a aplicações/games e customização de memória/host APIs.

- [AngelScript API Reference](https://www.angelcode.com/angelscript/sdk/docs/manual/doc_api.html)
- [AngelScript Memory Functions](https://www.angelcode.com/angelscript/sdk/docs/manual/group__api__memory__functions.html)

**Consultar quando:** host bindings, engine integration e política de memória do runtime.

## Pesquisa e contraste — não são roadmap atual

### Lobster

- [Lobster Memory Management](https://aardappel.github.io/lobster/memory_management.html)
- [Lobster Language Reference](https://aardappel.github.io/lobster/language_reference.html)

**Uso:** estudar value semantics, automatic ownership analysis e trade-offs RC versus tracing GC. Aipo permanece tracing-GC.

### Lean 4

- [Lean — Reference Counting](https://lean-lang.org/doc/reference/latest/Run-Time-Code/Reference-Counting/)

**Uso:** estudar RC otimizado, borrowing inferido e reuse; material de pesquisa, não baseline.

### Koka / Perceus

- [Perceus — Microsoft Research](https://www.microsoft.com/en-us/research/publication/perceus-garbage-free-reference-counting-with-reuse/)
- [Perceus — DOI/PLDI 2021](https://doi.org/10.1145/3453483.3454032)

**Uso:** pesquisa de precise RC/reuse e comparação de memory managers; não implementar na Aipo atual.

## Regra de leitura

Quando houver conflito de inspiração: **Aipo > Lua/Luau > demais referências**. A semântica já aprovada da Aipo prevalece sobre conveniência de copiar comportamento externo.