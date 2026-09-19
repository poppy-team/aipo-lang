# Aipo — Backlog de Sintaxe e Recursos Pós-V1

<aside>
🗂️

**Status: não normativo.** Esta página registra recursos deliberadamente fora da Aipo V1 e ideias que podem ser revisitadas. Estar aqui **não significa aprovação nem compromisso de implementação**.

</aside>

## Candidatas já discutidas e com interesse concreto

- Expressão condicional de valor — **fechada para V1** como `if condition then value else value`; a V1 não terá ternário `?:`.
- Índice opcional em `repeat`, preferencialmente de forma explícita em vez de um nome mágico implícito.
- Política de concatenação — **fechada para V1**: manter `String + String`; privilegiar interpolação para texto e `join`/builder para composição maior.
- Elipse do sujeito em cadeias de condições, por exemplo `value is Int and >= 0 and <= 100`, somente se puder ser uma regra geral coerente e não uma mini-DSL exclusiva de `invariant()`.
- Omissão de `()` antes de trailing block — **fechada para V1**: `html do ... end` é açúcar contextual para `html() do ... end`.
- Pipeline `|>` — **fechado para V1** como açúcar de chamada, sem semântica própria de execução.
- Binding destructuring superficial em `let`/`var` — **fechado para V1**; destructuring assignment/nested/rest e pattern matching continuam fora.
- Builders/DSL blocks — **arquitetura V1 fechada** em `do ... end` com builder explícito; receiver/contexto implícito continua backlog.
- Block expressions somente se resolverem casos reais sem transformar toda a linguagem em expression-oriented.
- `async`/`await`, fibers/coroutines e event loop após o baseline da VM.
- `defer` ou outra forma explícita/determinística para lifecycle de recursos externos, se a biblioteca de recursos demonstrar necessidade.
- Contratos profundos built-in `List[T]` e `Dict[K, V]`, caso o custo/benefício em APIs justifique reabrir a decisão.

## Deliberadamente fora da V1; reconsiderar apenas com caso de uso forte

- Classes e herança.
- `new`.
- Métodos virtuais/dispatch OO clássico.
- Getters/setters ou propriedades mágicas como mecanismo especial.
- Overload definido pelo usuário.
- Operator overloading definido pelo usuário.
- Generics definidos pelo usuário.
- Unions arbitrárias e intersections.
- `Any` como contrato universal explícito.
- `Void`/`Unit` público apenas para funções sem resultado.
- Aliases sintáticos redundantes como `for` para `each`, `case` para `when` ou `switch` paralelo a `match`.
- `try/catch`, `throw` e `raise` como segundo sistema de erro paralelo a Failure.
- Import no estilo `from ... import ...`.
- Comentários de bloco.
- `;` como separador de statements.
- `++`/`--`.
- Assignment expressions e chained assignment.
- Destructuring assignment.
- Pattern matching estrutural/guards em `match`.
- Listas explícitas de captura de closures, `nonlocal` ou move-capture.
- Ownership/borrowing/referências manuais como modelo normal do usuário.
- Range descendente implícito e stride sintático.
- Tipos de assinatura detalhados para `Function`.
- Bound-method value automático e callable objects.

## Regra de governança

Uma candidata só sai deste backlog quando houver: problema real demonstrável, desenho sintático e semântico, lowering claro, impacto em parser/analisador/Core IR/VM/JS, casos-limite e exemplos de código suficientes. Recursos que apenas economizam caracteres sem reduzir carga cognitiva devem permanecer fora.

## Promovidos à V1 em 2026-09-10

As ideias abaixo deixaram de ser backlog e passaram a ser parte da superfície V1; os registros antigos acima permanecem apenas como histórico desta página:

- `repeat count as index` com índice opt-in explícito — **promovido à V1; remover da seção de candidatas em próxima limpeza editorial**.
- `callee do ... end` como açúcar contextual para chamada sem argumentos + trailing block — **promovido à V1**.
- Pipeline `|>` sem placeholder.
- Destructuring superficial em `let`/`var`.
- Continuação de comparações por `and` com sujeito avaliado uma única vez.
- Manutenção de `String + String`, com interpolação para texto e `join`/builder para composição maior.
- Builders explícitos como primeira arquitetura oficial das DSL-like libraries.

**Fechado em 2026-09-10:** escolha condicional de valor usa `if condition then value else value`; fallback recuperável usa `expression or_else fallback`; `or` permanece exclusivamente Boolean e o ternário `?:` fica fora da V1. Receiver implícito de DSL, block expressions gerais, nested/rest destructuring e elipse de comparação por `or` permanecem backlog.

## Decisão fechada — condicional x fallback de Failure

```
# escolha condicional de valor
if condition then then_value else else_value

# fallback recuperável de Failure
expression or_else fallback
```

- `or` permanece exclusivamente Boolean.
- `or_else` é lazy e só avalia o fallback quando a expressão à esquerda termina em `Failure`.
- Runtime faults não ativam `or_else`.
- O ternário `?:`, `otherwise`, `.or(...)` mágico e o antigo `expression else fallback` ficam fora da superfície V1.

## Decisão fechada — números compactos, bytes e strings

**Status: promovida à V1 em 2026-09-10.** Aipo separa deliberadamente a semântica numérica cotidiana da representação compacta de armazenamento.

- `Int` representa exatamente `±(2^53 - 1)` para manter equivalência exata VM↔JavaScript; a VM Odin pode usar `i64` internamente.
- `Float` permanece IEEE 754 binary64 finito.
- `Byte` é valor inteiro `0..255` com conversão explícita/verificada.
- `Bytes` é bloco binário mutável e gerenciado, separado de `String`.
- formatos `i8/u8/i16/u16/i32/u32/i64/u64/f32/f64` existem nas APIs de buffer/packing, não como uma família de tipos cotidianos da linguagem.
- leituras compactas retornam `Int`/`Float`/`Byte`; `i64/u64` fora do intervalo público de `Int` produzem `Failure`.
- `Int(String)`, `Float(String)` e `Byte(String)` são conversões explícitas recuperáveis; não há coerção automática.
- `String.encode()` usa UTF-8 e `Bytes.decode()` valida UTF-8.
- a ordem de bytes de packing deve ser definida pela API e nunca depender silenciosamente da plataforma.
- `Int64`, `UInt64`, `BigInt`, tipos numéricos sized cotidianos e `Float32` como tipo normal permanecem fora da V1 e só serão reabertos mediante caso de uso real.

Referências arquiteturais: Luau `buffer`, JavaScript `TypedArray`/`DataView`, Lua packing e tipos sized do Odin.

## Revisão de backlog após pivô Rust/Poppy — 2026-09-15

### Promovidos para protótipo ativo

- **`enum` simples**: conjunto fechado de nomes, sem payload variants/ADTs/pattern matching estrutural na primeira versão.
- **`List[T]` / `Dict[K, V]` como contracts built-in**: úteis em APIs Poppy/Web/Script, sem abrir user-defined generics.
- **`async`/`await`**: passa de possibilidade remota para objetivo de primeira classe após o baseline mínimo da VM.
- **Await block sequencial**: explorar `await do ... end` como açúcar para muitos awaits no mesmo fluxo. A forma só pode ser canonizada se tiver lowering simples, diagnostics fortes e paridade VM↔JS.

### Declarative ergonomics — exploração controlada

Trailing blocks continuam o mecanismo-base para APIs declarativas. Antes de adicionar sintaxe nova, explorar builders tipados/contratados e APIs com receiver explícito. Receiver/contexto implícito dentro de builders pode ser prototipado futuramente somente se:

- reduzir de fato carga cognitiva em UI/scenes/config/HTML;
- não introduzir lookup invisível ou ambiguity;
- tiver scope lexical claramente delimitado;
- baixar para closures/chamadas normais;
- funcionar igualmente na VM e no backend JS;
- produzir diagnostics claros quando um nome pertence ao contexto externo versus builder.

### Regra de preservação geral

Nenhuma pressão da Poppy deve promover sintaxe específica de game engine ao core sem utilidade geral demonstrável. Features puramente de gameplay permanecem no Poppy Profile/host API.

Referência consolidada: [Aipo — Rust/Poppy Pivot, Host Profiles e Roadmap 10/10](Aipo — Rust Poppy Pivot, Host Profiles e Roadmap 1 3dc9bb7d023f81a4b03fe8f7e6de3508.md).