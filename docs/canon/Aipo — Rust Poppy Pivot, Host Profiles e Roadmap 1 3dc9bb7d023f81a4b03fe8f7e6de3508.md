# Aipo — Rust/Poppy Pivot, Host Profiles e Roadmap 10/10

<aside>
🦀

**Decisão aprovada em 2026-09-15:** a implementação oficial da Aipo passa a ser **Rust-first e code-agent-first**. Poppy Game Engine torna-se o primeiro host de referência e principal laboratório de uso real, **sem transformar Aipo em uma linguagem exclusiva da Poppy**. O backend JavaScript continua parte fundamental do projeto.

</aside>

## Princípio arquitetural central

Aipo mantém **um único núcleo sintático e semântico**. Game, Web e Script não são dialetos: são perfis de host/biblioteca sobre a mesma linguagem.

```
                  AIPO CORE
   syntax + semantics + stdlib portable
                       │
             Host ABI / Registry
      ┌────────────────┼────────────────┐
      ▼                ▼                ▼
 Script Host       Poppy Profile      JS/Web Host
CLI/embedding      game engine       JS transpilation
      │                │                │
      ▼                ▼                ▼
  Rust VM          Poppy Engine      JavaScript runtime
```

**Regra:** recursos específicos de domínio devem entrar primeiro como APIs de host/biblioteca. Só entram no core quando o problema for realmente geral, recorrente e não puder ser resolvido com a linguagem existente.

## Objetivos simultâneos

1. Ser excelente como linguagem pequena de scripting embutível.
2. Ser a linguagem de gameplay de primeira classe da Poppy.
3. Continuar agradável para automação, CLI, ferramentas e aplicações leves.
4. Preservar o plano de transpilar para JavaScript com semântica equivalente.
5. Manter baixa carga cognitiva mesmo conforme o ecossistema cresce.
6. Ser especialmente fácil de implementar, auditar e evoluir com code agents.

## Implementação oficial Rust-first

A arquitetura recomendada passa a ser:

```
.aipo
  ↓
aipo-source
  ↓
aipo-lexer
  ↓
aipo-syntax       lossless syntax tree
  ↓
aipo-ast          typed facade
  ↓
aipo-hir
  ↓
aipo-sema
  ↓
aipo-ir           target-neutral Core IR
  ├──→ aipo-bytecode → aipo-vm → native/embedded hosts
  └──→ aipo-js       → JavaScript + source map + runtime shim
```

Topologia inicial de crates:

```
crates/
├── aipo-source
├── aipo-lexer
├── aipo-syntax
├── aipo-ast
├── aipo-hir
├── aipo-sema
├── aipo-ir
├── aipo-bytecode
├── aipo-vm
├── aipo-runtime
├── aipo-host
├── aipo-stdlib
├── aipo-diagnostics
├── aipo-formatter
├── aipo-lsp
├── aipo-cli
├── aipo-js
└── aipo-poppy
```

### Tooling preferido

- **Logos**: candidato principal para lexer se reduzir boilerplate sem comprometer diagnósticos; lexer handwritten continua alternativa válida após spike.
- **Rowan**: lossless syntax tree para preservar trivia, apoiar formatter, IDE e recuperação incremental.
- **Ungrammar ou schema equivalente próprio**: geração de facade AST e consistência estrutural.
- **Parser handwritten recursive descent + Pratt**: baseline preferido do compilador canônico.
- **Ariadne + Diagnostic model próprio**: apresentação de diagnósticos; códigos/semântica pertencem à Aipo.
- **gc-arena**: candidato principal para a primeira VM Rust managed, sujeito a spike de integração, profiling e requisitos de hot reload.
- **slotmap/generational handles ou estrutura Poppy-owned equivalente**: handles externos seguros; objetos da engine nunca são ponteiros Rust expostos ao script.
- **notify**: file watching/hot reload de source/assets quando adequado.
- **insta**: snapshots de tokens, syntax tree, AST/HIR, IR, bytecode e diagnostics.
- **proptest**: property testing de parser, valores, collections e invariantes semânticos.
- **cargo-fuzz**: fuzzing de lexer/parser/decoder/runtime boundaries.
- **cargo-mutants**: mutation testing em partes críticas quando a suíte amadurecer.
- **cargo-nextest**: execução rápida da suíte.
- **Criterion**: benchmarks de frontend/VM/GC/bindings.
- **Tree-sitter**: integração externa de editores/highlighting; não é o parser canônico.
- **LSP Rust dedicado**: diagnostics, completion, hover, rename, references, signature help, semantic tokens e code actions.
- **cargo xtask**: comandos estáveis para code agents e manutenção do repositório.

## Desenvolvimento code-agent-first

A especificação passa a ser tratada como **contrato executável**.

```
ADP / design decision
        ↓
formal-ish grammar + semantics
        ↓
positive + negative fixtures
        ↓
syntax snapshot
        ↓
AST/HIR snapshot
        ↓
semantic diagnostics
        ↓
Core IR snapshot
        ↓
bytecode/runtime behavior
        ↓
VM ↔ JS differential tests
        ↓
formatter/LSP/conformance
```

Cada feature relevante deve possuir acceptance criteria verticais cobrindo, quando aplicável: lexer, parser, recovery, AST, HIR, semantic analysis, IR, VM, JS emitter, diagnostics, formatter, LSP, tests, fuzzing e documentação.

## Poppy Profile — arquitetura aprovada

Poppy é o primeiro host profile oficial, mas a linguagem permanece host-neutral.

```
poppy-scripting
├── ScriptBackend
├── AipoBackend          target estratégico/default futuro
└── LuaBackend           referência, fallback e differential host durante a transição
```

Lua não precisa ser removida imediatamente. Durante a maturação da Aipo, os mesmos jogos/fixtures podem comparar comportamento Rust/Lua/Aipo quando isso trouxer confiança.

### Native host values

Aipo ganha suporte semântico interno a **host values com semântica de valor**, sem nova sintaxe obrigatória.

Exemplos Poppy:

- `Vec2`
- `Vec3`
- `Vec4`
- `Quat`
- `Color`
- `Rect`
- `Transform2D`
- `Transform3D`
- matrizes/pequenos valores matemáticos quando justificável.

Esses valores copiam por valor e não viram identidades gerenciadas apenas porque foram definidos pelo host.

### Opaque/generational host handles

Objetos externos como `Entity`, `Asset`, `Texture`, `Sound`, `Animation` e `Scene` atravessam a fronteira por handles opacos/geracionais. Nunca expor ponteiros Rust ou internals do ECS.

Regras:

- stale handle nunca produz use-after-free;
- operações que aceitam ausência podem produzir `none`;
- operações que exigem objeto válido podem produzir `Failure` ou diagnostic/fault conforme contrato;
- identidade externa pertence ao host.

### Bindings gerados pelo Semantic Registry

O `PoppyTypeRegistry + SemanticRegistry` alimenta automaticamente:

- bindings runtime Aipo;
- tipos/contratos conhecidos pelo analyzer;
- autocomplete;
- hover docs;
- signatures;
- editor/tooltips;
- documentação;
- metadata de capabilities;
- versão/depreciação/migração.

Evitar listas manuais paralelas de bindings.

## ECS — mutação escopada aprovada

A API Poppy deve preferir acesso escopado em closures em vez de expor empréstimos/lifetimes ao usuário.

Exemplo conceitual:

```
game.world.edit(entity, Transform) do transform!
    transform.position.x += 1.0
end
```

ou queries compostas equivalentes.

Semântica:

1. Poppy resolve handle/component.
2. O host cria um acesso temporário válido somente durante a closure.
3. A closure recebe receiver/binding com permissão de mutação explícita via `!`.
4. O valor escopado não pode escapar, ser persistido em global/heap de longa vida ou sobreviver ao callback.
5. Ao sair do bloco, Poppy encerra o acesso, valida e aplica/agenda a mutação no safe point adequado.

O usuário não precisa conhecer `&mut`, lifetimes, `RefCell` ou regras internas de borrow do ECS.

## Lifecycle de gameplay como biblioteca

Não adicionar keywords `behavior`, `component`, `system`, `event`, `signal` ou hooks específicos de jogos ao core.

Poppy oferece interfaces e APIs normais:

```
interface Behavior
    fn start(self!, game: Game)
    fn update(self!, game: Game)
    fn fixed_update(self!, game: Game)
end
```

O usuário combina `struct`, `impl`, `interface` e `satisfy`, preservando o modelo mental existente.

## Eventos como biblioteca

Eventos usam closures/trailing blocks e tipos do host:

```
events.on(DamageEvent) do event
    health -= event.amount
end
```

Nenhuma segunda semântica de callback/signal entra no core.

## Hot reload — contrato arquitetural

Primeira geração prioriza **reload de código + restauração/migração de estado serializável** em vez de tentar preservar arbitrariamente todo o heap da VM.

```
old module
    ↓ snapshot state
Poppy semantic state / behavior state
    ↓ reload code
new module
    ↓ schema-aware migration
rehydrated state
```

Regras:

- campos preservados por stable IDs/names conforme schema;
- campos novos usam default quando possível;
- campos removidos são descartados com diagnóstico quando relevante;
- incompatibilidades produzem diagnostics acionáveis;
- state migration deve ser determinística e testável;
- full heap migration fica fora da primeira geração.

## Determinism profile

Para replay, testes, rollback futuro e debugging, Poppy define comportamento determinístico onde necessário sem contaminar o core com sintaxe de jogos.

Especificar no host profile:

- fixed tick;
- ordering de systems/callbacks/eventos;
- RNG seedable via host API;
- semantics de `Dict`/iterações quando observáveis;
- clock/time explícito;
- ordering de tarefas relevantes;
- state digest/replay fixtures.

## Sandbox, capabilities e budgets

Scripts não recebem filesystem, processo, sockets, GPU ou internals do engine livremente.

A VM/host deve suportar pelo menos:

- instruction/fuel budget;
- stack depth limit;
- memory/allocation budget ou accounting;
- cancellation/interruption;
- capabilities por módulo/package;
- auditabilidade de host calls privilegiadas.

Budget exhaustion e host security violations são faults/control conditions do host, não `Failure` recuperável comum do gameplay.

## `enum` simples — direção aprovada para protótipo

Adicionar um enum propositalmente pequeno, inicialmente sem ADTs/payload variants/pattern destructuring.

```
enum EnemyState
    Idle
    Patrol
    Chase
    Attack
    Dead
end
```

Objetivo: estados, modos e opções fechadas frequentes em games e uso geral, com custo cognitivo baixo.

Fora da primeira versão do enum:

- payload variants;
- algebraic data types gerais;
- custom discriminants salvo necessidade concreta;
- pattern matching estrutural sofisticado;
- métodos mágicos específicos de enum.

## `List[T]` e `Dict[K, V]` — protótipo aprovado

Reabrir contratos profundos apenas para collections built-in:

```
fn enemies_in_area(area: Rect) -> List[Entity]
fn load_manifest() -> Dict[String, Asset]
```

Regras:

- continuam opcionais;
- melhoram API boundaries, tooling, agents e bindings gerados;
- não abrem generics definidos pelo usuário;
- collections sem parâmetros continuam válidas em código dinâmico quando apropriado.

## Async/await — direção aprovada

`async`/`await` passa a ser objetivo de primeira classe após o baseline mínimo da VM, não apenas possibilidade remota.

Modelo desejado:

- `async fn` produz uma task/future Aipo gerenciada pelo scheduler do host/runtime;
- `await expression` suspende a task atual sem bloquear thread;
- cancelamento e lifecycle pertencem ao task runtime/host;
- VM nativa e backend JS precisam preservar a mesma semântica observável;
- JS emitter pode aproveitar `async`/`await` nativo quando semanticamente equivalente, com shim onde necessário.

### Await block — ADP experimental aprovada para exploração

Objetivo: reduzir ruído quando uma sequência possui muitos awaits sem esconder concorrência ou ordering.

Forma candidata:

```
await do
    fade.in()
    time.seconds(1.0)
    player.walk_to(target)
    dialog.show("Hello")
end
```

Semântica candidata mais simples: **cada expressão de nível superior do bloco que produz uma Task é aguardada sequencialmente**. Statements normais continuam normais. O bloco não cria paralelismo, não faz await implícito em chamadas arbitrariamente aninhadas e não muda o comportamento de funções chamadas dentro dele.

Exemplo com valores:

```
await do
    let config = load_config()
    apply(config)
    let profile = load_profile()
    start(profile)
end
```

A ADP deve fechar antes de canonização:

- como distinguir Task de valor dinâmico em runtime/análise;
- comportamento de `return`, `fail`, `attempt`, loops e nested blocks;
- se RHS de `let`/`var` auto-await quando produzir Task;
- nested `await do`;
- cancelamento;
- lowering para VM;
- lowering JS;
- diagnósticos para Task esquecida;
- interação com `do ... end` comum.

**Regra de design:** o block form deve ser açúcar para awaits explícitos sequenciais, nunca uma segunda semântica de async.

## Consolidação de ergonomia aprovada — 2026-09-15

As seguintes direções passam a ser decisões de design aprovadas para prototipação e fechamento por ADPs/conformance, preservando a governança da linguagem:

### Nível 1 — construção nomeada

Reutilizar a semântica já existente de `name = value` para construção legível de structs/valores.

```
let player = Player{
    id = 1,
    name = "Ana",
    health = 100
}
```

Quando houver `init`, os nomes correspondem aos parâmetros de construção; na construção automática, correspondem aos campos. A forma posicional pode permanecer para casos pequenos quando não prejudicar legibilidade.

### Nível 2 — builders explícitos

`do ... end` + parâmetros explícitos continuam sendo a base declarativa segura e geral para UI, HTML, scenes, config, routing, tests, resources e outras DSL-like libraries.

```
html do page
    page.body do body
        body.h1("Hello")
    end
end
```

Não introduzir receiver implícito global nem lookup mágico de nomes como base da DSL.

### Nível 3 — subject-dot declarativo

Prototipar um açúcar contextual em trailing blocks com subject explícito. Dentro de `do subject`, uma expressão iniciada por `.` baixa diretamente para `subject.`.

```
ui.window(title = "Inventory") do window
    .column(spacing = 8) do column
        .text("Items")
        .button("Close") do
            close_inventory()
        end
    end
end
```

Regras pretendidas:

- `.foo(...)` nunca faz busca global/implícita; equivale apenas a `subject.foo(...)`;
- nested blocks trocam o subject de maneira lexical e local;
- não altera `self`;
- não cria métodos, receiver dinâmico ou nova semântica de dispatch;
- deve ser validado em pelo menos UI, scenes/config e HTML/Web antes de canonização.

## Matriz de evolução 10/10 — tópicos aprovados para discussão sistemática

O roadmap qualitativo passa a avaliar separadamente: **simplicidade, legibilidade, semântica, uso geral, declarativo, async, game scripting, ECS, hot reload, JavaScript, sandbox/security, tooling, packages, code agents e performance**.

Cada tópico deve ter critérios mensuráveis, casos reais, interaction tests e impacto VM↔JS/Poppy. Aumentar a nota de uma área não pode reduzir silenciosamente simplicidade/previsibilidade de outra.

## `await do` — direção ergonômica aprovada

A forma em bloco é aprovada como objetivo de design, mantendo semântica sequencial e previsível. A ADP deve formalizar `Task[T]`, structured concurrency, cancellation, diagnostics para tasks esquecidas e lowering VM↔JS. A forma base `await expression` permanece semanticamente fundamental; `await do` é açúcar, não uma segunda máquina de async.

## Próximo foco: Standard Library

A próxima discussão de design será a stdlib. Ela deve ser organizada em camadas para preservar generalidade e portabilidade:

1. **Prelude/Core pequeno** — operações universais e muito frequentes, sem inflar o namespace.
2. **Portable Standard Library** — módulos com semântica idêntica na VM, Poppy e JavaScript sempre que possível.
3. **Host Standard Library** — APIs padronizadas cuja implementação depende de capabilities do host, com availability/target checking explícito.
4. **Host Profiles** — Poppy, Web e outros hosts adicionam APIs próprias sem criar dialetos.
5. **Official Packages** — capacidades úteis, porém grandes/especializadas, distribuídas fora da stdlib para evitar crescimento permanente do core.

A stdlib deve obedecer aos mesmos princípios da linguagem: uma operação/forma canônica, biblioteca antes de sintaxe, APIs previsíveis, nomes completos, baixo custo cognitivo e documentação/diagnósticos first-class.

## Backend JavaScript continua first-class

Poppy não substitui Web/JS.

A arquitetura deve preservar um **Core IR target-neutral** e uma suíte diferencial VM ↔ JS.

### Contrato do backend JS

- mesma semântica pública de `Int`, `Float`, `Byte`, strings e `none`;
- mesma semântica de `Failure` versus runtime fault onde tecnicamente possível;
- identity/copy/same coerentes;
- closures e module semantics equivalentes;
- `async`/`await` equivalentes;
- source maps;
- `.d.ts`/metadata gerada para interoperabilidade TypeScript quando útil;
- runtime shim pequeno e versionado para diferenças que JS não representa diretamente;
- differential conformance tests para cada feature suportada.

### Portabilidade por capability, não por dialeto

Um arquivo Aipo é portável conforme as APIs que importa.

```
import math       # portable
import text       # portable
import poppy      # requires Poppy host
import web        # requires JS/Web host
```

O compilador/tooling deve poder informar claramente quando um módulo depende de uma capability/host específico.

## Ergonomia declarativa — direção de design

Aipo deve ficar **mais declarativa sem criar uma mini-linguagem paralela**.

Prioridade:

1. builders via funções + `do ... end`;
2. nested builders tipados/contratados;
3. named arguments e defaults onde já fizerem sentido;
4. collections/destructuring/pipelines existentes;
5. somente então avaliar receiver/contexto implícito restrito.

Exemplo explícito atual preferido:

```
ui.window("Inventory") do window
    window.column() do column
        column.text("Items")
        column.button("Close") do
            close_inventory()
        end
    end
end
```

Uma futura forma de receiver implícito pode ser explorada apenas se reduzir carga cognitiva em UI/scenes/config sem gerar ambiguity ou lookup oculto. Qualquer proposta deve ter scope delimitado, lowering simples e diagnostics fortes.

## Roadmap para 10/10

Aipo só deve ser considerada 10/10 para o objetivo quando estes eixos estiverem maduros:

### Linguagem/semântica

- gramática formal suficientemente precisa;
- modelo semântico formal-ish para valores, identity, mutability, Failure/fault, modules e async;
- `enum` simples validado por programas reais;
- collection contracts validados;
- async/task semantics fechada;
- diagnostics acionáveis e consistentes.

### Runtime

- VM correta e previsível;
- GC com pause/budget adequado a workloads interativos;
- execution budgets;
- hot reload robusto;
- debugger/profiler hooks;
- stable host ABI interno versionado.

### Poppy

- generated bindings do Semantic Registry;
- scoped ECS mutation;
- host values/handles;
- lifecycle/events/timers/tasks;
- save/hot-reload migration;
- deterministic replay;
- capability sandbox;
- conformance games.

### Geral/Web

- stdlib portátil suficiente;
- filesystem/process/network somente via profiles/capabilities adequadas;
- JS emitter completo;
- source maps;
- TypeScript interop;
- browser/web APIs como biblioteca host;
- VM ↔ JS differential conformance.

### Tooling

- formatter estável;
- LSP excelente;
- Tree-sitter/editor integrations;
- docs geradas/searchable;
- package/project tooling;
- debugger/profiler;
- REPL e playground úteis.

### Engenharia por agents

- spec executável;
- golden/snapshot tests;
- fuzzing;
- mutation/property tests;
- architecture/dependency rules verificáveis;
- ADR/ADP rastreáveis;
- small-context crate boundaries;
- `xtask` workflows idempotentes;
- automatic quality gates.

## Não adicionar apenas por conveniência

Continuam fora salvo prova forte de necessidade:

- classes/herança;
- user-defined operator overloading;
- user-defined generics;
- macros/metaprogramação geral;
- annotations/decorators como sistema aberto de metadata;
- ownership/borrowing/pointers na superfície normal;
- segundo sistema de exceptions;
- DSL grammar exclusiva de Poppy;
- implicit globals do engine;
- acesso direto a ECS/renderer internals;
- JIT/Cranelift antes de benchmark justificar.

<aside>
🌿

**Critério final:** Poppy deve provar a Aipo, não aprisioná-la. Cada feature criada por pressão de game scripting deve continuar sendo uma boa feature de uma linguagem geral ou permanecer no Poppy Profile como biblioteca/host behavior.

</aside>

[Aipo — Standard Library Architecture](Aipo — Standard Library Architecture 3dc9bb7d023f8151974be51001c61905.md)

## Ergonomia funcional e stdlib — decisões aprovadas

- `|>` é confirmado como operador canônico de pipeline/composição esquerda→direita, com lowering simples e sem placeholders mágicos na primeira versão.
- Aipo ganhará **lambdas curtas de expressão** usando `=>` como direção preferida, preservando `->` exclusivamente para contratos de retorno. `fn ... end` continua sendo a forma completa; `do ... end`, a forma declarativa/trailing block.
- A combinação canônica de ergonomia funcional passa a ser: `fn(...) ... end`, `value => expression`, `call(...) do ... end` e `value |> transform`.
- `http` e `net` passam a integrar a stdlib oficial capability-aware e async-first.
- `crypto` passa a integrar a stdlib oficial capability-aware, com APIs de alto nível e defaults seguros; primitivas criptográficas devem vir de implementações auditadas/maduras do host, nunca de crypto própria escrita na Aipo.
- `random` determinístico/seedable e `crypto.random` criptograficamente seguro são conceitos separados.
- A arquitetura detalhada da biblioteca padrão está em [Aipo — Standard Library Architecture](Aipo — Standard Library Architecture 3dc9bb7d023f8151974be51001c61905.md).

[Aipo — Stdlib V1 Canônica e Contratos de Portabilidade](Aipo — Stdlib V1 Canônica e Contratos de Portabili 3dc9bb7d023f8115ae2bce7f48301dcf.md)

[Aipo — Fechamento Arquitetural 10/10](Aipo — Fechamento Arquitetural 10 10 3dc9bb7d023f818b9504c7ecc535cc56.md)

[Aipo — Sistema de Documentação de Implementação e Diretivas para Code Agents](Aipo — Sistema de Documentação de Implementação e  3dc9bb7d023f816fbd68e44b28f84554.md)