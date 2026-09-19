# 05 — Referências Históricas de Decisão e Alternativas

<aside>
🗃️

**Arquivo de decisões.** Estas fontes já foram úteis para comparar caminhos que a Aipo não segue hoje ou para justificar pequenas escolhas sintáticas. Permanecem registradas para evitar repetir pesquisa e para reabrir decisões com contexto quando houver evidência nova.

</aside>

## Linguagem host e garbage collectors externos

### Go GC

- [Go GC Guide](https://go.dev/doc/gc-guide)

**Por que foi estudado:** comparar um tracing GC moderno/concurrente com D e com a decisão de escrever o runtime da Aipo em uma linguagem host diferente.

**Conclusão atual:** excelente collector pronto, mas Aipo escolheu Odin porque seu heap/GC é próprio e o projeto também serve como percurso de systems programming.

### D GC

- [D core.memory](https://dlang.org/phobos/core_memory.html)

**Por que foi estudado:** D foi candidata forte como host por produtividade, GC para estruturas do compilador, `@nogc` e interoperabilidade nativa.

**Conclusão atual:** D permanece referência de produtividade, não host oficial.

### Nim ARC/ORC

- [Nim Memory Management](https://nim-lang.org/docs/mm.html)
- [Introduction to ARC/ORC](https://nim-lang.org/blog/2020/10/15/introduction-to-arc-orc-in-nim.html)

**Por que foi estudado:** comparar tracing GC com automatic reference counting/cycle handling e memory managers de baixa latência.

**Conclusão atual:** Aipo permanece tracing GC; ARC/ORC são contraste de pesquisa.

## Alternativas de gerenciamento de memória

### Lobster

- [Memory Management](https://aardappel.github.io/lobster/memory_management.html)

**Uso histórico:** ownership analysis automático para eliminar grande parte do custo de RC sem expor borrowing ao usuário.

### Lean 4

- [Reference Counting](https://lean-lang.org/doc/reference/latest/Run-Time-Code/Reference-Counting/)

**Uso histórico:** RC otimizado, borrowing inferido e distinção de referências compartilhadas.

### Koka / Perceus

- [Perceus — Microsoft Research](https://www.microsoft.com/en-us/research/publication/perceus-garbage-free-reference-counting-with-reuse/)
- [Perceus — DOI](https://doi.org/10.1145/3453483.3454032)

**Uso histórico:** precise reference counting, reuse analysis e comparação acadêmica com tracing GC.

**Decisão preservada:** essas técnicas não fazem parte do roadmap obrigatório da Aipo; só reabrir diante de um problema mensurável que o tracing GC não resolva satisfatoriamente.

## Referências de decisões sintáticas

### Odin — `or_else` e expressões condicionais

- [Odin Overview](https://odin-lang.org/docs/overview/)

**Uso:** referência para separar fallback de optional/failure de `or` Boolean e para comparar formas de expressão condicional.

**Decisão Aipo:** `or_else` é fallback recuperável; `or` continua Boolean; Aipo não adotou `?:`.

### Rust `Result::or_else`

- [Rust std:](https://doc.rust-lang.org/std/result/):result:[:Result](https://doc.rust-lang.org/std/result/)

**Uso:** contraste de uma API lazy de recuperação quando erro é um valor explícito. Aipo usa `Failure` propagável, portanto `or_else` é operador/special form da linguagem e não método mágico.

### Elixir pipeline

- [Elixir Kernel — pipe operator](https://hexdocs.pm/elixir/Kernel.html#%7C%3E/2)

**Uso:** referência para `|>` inserindo o valor esquerdo como primeiro argumento da chamada seguinte.

**Decisão Aipo:** pipeline é açúcar removido cedo no lowering; a VM não recebe opcode especial de pipe.

### Ruby `Integer#times`

- [Ruby Integer](https://docs.ruby-lang.org/en/master/Integer.html#method-i-times)

**Uso:** referência de repetição contada com índice opcional disponibilizado ao bloco.

**Decisão Aipo:** `repeat count` não cria nome mágico; `repeat count as i` expõe o contador explicitamente.

### Swift ternary

- [Swift — Basic Operators](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/basicoperators/)

**Uso:** comparar concisão do ternário com uma forma verbal.

**Decisão Aipo:** ternário `?:` rejeitado na V1; reutiliza-se `if condition then value else value`.

### Kotlin builders

- [Type-safe Builders](https://kotlinlang.org/docs/type-safe-builders.html)

**Uso:** trailing lambdas, builders hierárquicos e os problemas reais de receivers implícitos aninhados.

**Decisão Aipo:** V1 usa trailing blocks com builder explícito; receiver implícito permanece possibilidade futura.

## Portabilidade e múltiplos targets

### Haxe

- [Compiler Targets](https://haxe.org/documentation/introduction/compiler-targets.html)

**Uso histórico:** comprovar a viabilidade de frontend/semântica comum com backends distintos.

**Decisão Aipo:** Bytecode VM é backend principal; JavaScript é backend alternativo. Sem C/native no roadmap atual.

## Regra para reabrir uma decisão

Uma decisão histórica só deve voltar ao design ativo quando houver pelo menos um dos seguintes:

- benchmark reproduzível indicando gargalo real;
- caso de uso que a superfície atual não consegue expressar com clareza;
- limitação de portabilidade/embedding comprovada;
- evidência de que uma alternativa reduz complexidade total, e não apenas troca complexidade de lugar.

Ao reabrir, registrar a decisão anterior, a nova evidência e o custo de migração.