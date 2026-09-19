# Aipo — Fechamento Arquitetural 10/10

<aside>
✅

**Objetivo:** fechar as pendências de design que impediam considerar a arquitetura da Aipo “10/10” em coerência, implementabilidade e verificabilidade. “10/10” aqui significa **design fechado com critérios executáveis**, não afirmar que a implementação já existe ou já atingiu performance/qualidade final.

</aside>

## Estado geral

Aipo permanece uma linguagem dinâmica e strongly typed por valores, com contracts opcionais em boundaries; Rust-first, code-agent-first, host-neutral, Poppy como primeiro host de referência e backend JavaScript first-class.

A governança continua: biblioteca antes de sintaxe, baixa densidade cognitiva, uma regra principal por conceito, exceptions explícitas e recursos avançados somente quando provados por programas reais.

## 1. Simplicidade — fechado

### Contrato

- Core pequeno e lento para crescer.
- Prelude mínimo.
- Sem classes/inheritance, user generics, arbitrary unions, operator overloading ou macro system geral na V1.
- Host features entram por APIs/profiles, não keywords de domínio.
- ADP obrigatório para qualquer aumento de sintaxe/semântica.

### Budget cognitivo

Toda feature nova deve declarar: conceito novo, símbolos novos, exceções, implicit behavior, interaction matrix e alternativas library-only. Falhar no five-cost test bloqueia canonização.

## 2. Legibilidade — fechado

- Formatter oficial e determinístico é parte da linguagem/toolchain.
- Named arguments e named struct construction são padrão para chamadas/configurações largas.
- Nomes completos preferidos; stdlib evita abreviações obscuras.
- `=>` só para lambda de uma expressão; lógica maior usa `fn ... end`.
- `|>` só para composição esquerda→direita; não cria contexto mágico.
- Diagnostics mostram causa, source span, conceito envolvido e ação sugerida.

## 3. Semântica — fechado

### Valores e identidade

Value semantics: `none`, Bool, Int, Float, Byte, String e host values declarados como value. Managed identity: struct instances, List, Dict, Bytes, Function/Closure, Task/Stream e objetos managed equivalentes.

`==` compara igualdade semântica do tipo; `same(a,b)` pergunta identidade quando o tipo possuir identidade. `copy(value)` é shallow copy canônica; deep copy só via APIs/schema explícitas.

### Números

Int público permanece ±(2^53−1), garantindo equivalência exata com JavaScript Number para inteiros suportados. Float é binary64 finito; NaN/Infinity não são valores normais produzidos por parsing/serialization portátil sem API explícita.

### Failure versus fault

`Failure` é recuperável e participa de `attempt`/`or_else`. Programmer error, stale forbidden handle, sandbox violation, budget exhaustion, stack overflow e cancellation são faults/control conditions e não são capturados por `attempt` comum.

## 4. Uso geral — fechado

Aipo não é game-only. O critério de release exige quatro proof programs:

1. jogo Poppy real;
2. CLI/tool real usando fs/http/json/packages;
3. aplicação Web/JS real com source maps e async;
4. package portátil rodando VM e JS sem código condicional de host no núcleo.

A stdlib canônica cobre text/Unicode, collections, JSON/binary/encoding, URL/path, time/task, fs, HTTP/net, crypto, testing/log.

## 5. Declarativo — níveis 1/2/3 fechados

### Nível 1 — named construction

```
let player = Player{
    id = 1,
    name = "Ana",
    health = 100
}
```

### Nível 2 — trailing builder explícito

```
ui.window(title = "Inventory") do window
    window.column(spacing = 8) do column
        column.text("Items")
    end
end
```

### Nível 3 — subject-dot escopado

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

Regra normativa: `.member` só resolve para o **subject do trailing block lexical atual**. Nested block substitui o subject. Unqualified `member()` nunca procura receiver oculto. Para acessar outer subject, use o nome explícito capturado (`window...`). Subject-dot baixa cedo no HIR para acesso explícito e não cria runtime mechanism. Fora de um subject block é compile error.

## 6. Async — fechado

`Task[T]`, `async fn`, `await`, `await do` e structured concurrency são parte do roadmap V1 após baseline mínimo da VM.

`await do` usa a semântica fechada na stdlib: awaits sequenciais conhecidos, sem await em subexpressões, sem paralelismo implícito, `return` para função envolvente e Failure/cancellation separados.

Scheduler contract define ordering observável, cancellation points, task ownership, timeout e host integration. VM e JS possuem differential tests de ordering/cancellation.

## 7. Game scripting — fechado

Poppy profile é biblioteca/host schema, não dialect.

- Lifecycle via interfaces normais (`Behavior`).
- Eventos via closures/trailing blocks.
- Host values matemáticos por value semantics.
- Entity/Asset/etc. por opaque generational handles.
- Game APIs são discoverable via generated Host Schema.
- Lua permanece fallback/reference durante maturação, sem impedir Aipo de virar default.

## 8. ECS — fechado

Nunca expor `bevy_ecs::World`, Rust references ou lifetimes.

### Read/query

Queries fornecem view/snapshot lógico válido durante scope conhecido. Values pequenos podem ser copiados; identity/handles permanecem host-owned.

### Mutation

```
game.world.edit(entity, Transform) do transform!
    transform.position.x += 1.0
end
```

Scoped mutable bindings não escapam do callback. Structural changes (spawn/despawn/add/remove component) entram em **command buffer** e são aplicadas em safe points definidos pelo Poppy scheduler. Isso evita invalidar query/borrow durante traversal e mantém replay/determinism.

## 9. Hot reload — fechado para geração 1

Geração 1 = code reload + schema-aware serialized state migration, não heap surgery.

- module/symbol schema possui stable logical identity enquanto nome/path não mudar;
- campos preservados por schema path + type compatibility;
- campo novo usa default quando disponível;
- campo removido é descartado com diagnostic quando há estado persistido;
- rename/type migration incompatível exige migration function do host/profile, não heurística mágica;
- Poppy mantém state snapshot, recarrega module e reidrata no safe point;
- tasks antigas são canceladas antes de substituir module salvo contrato explícito do host;
- handles externos são revalidados por generation.

Full arbitrary heap preservation fica explicitamente pós-V1.

## 10. JavaScript backend — fechado

- Output ESM.
- Source maps obrigatórios.
- Runtime shim pequeno, versionado e testado.
- `.d.ts`/host metadata quando útil para interop.
- Int range, String NFC, ordered Dict/Set, identity/copy/same e async ordering seguem Aipo, não defaults acidentais de JS.
- Recoverable Failure baixa para representação runtime distinguível; `attempt` captura somente Aipo Failure, não qualquer JS exception.
- Runtime faults permanecem faults.
- VM↔JS differential suite cobre valores, strings Unicode, collections/order, modules, closures, Failure/fault, async, JSON, regex subset e portable stdlib.

**Importante:** JavaScript output não é por si só uma security sandbox. Untrusted embedded scripts devem usar Aipo VM/host sandbox ou um isolamento externo adequado.

## 11. Sandbox — fechado

Capability model é deny-by-default para hosts sandboxed.

Hierarquia inicial:

```
filesystem.read
filesystem.write
filesystem.roots
network.http
network.tcp
network.udp
process.spawn
env.read
clock.wall
clock.monotonic
crypto.random
crypto.keystore
poppy.*
```

Budgets: instructions/fuel, stack depth, heap/allocation accounting, host-call cost, task count, open handle count e optional wall-time deadline. Host calls privileged são auditáveis. No dynamic native library loading/eval escape em sandbox profile.

Package capabilities declaradas são upper bound; host/project policy pode conceder menos, nunca mais silenciosamente.

## 12. Tooling — fechado como produto mínimo 10/10

CLI canônica:

```
aipo run
aipo check
aipo fmt
aipo test
aipo build
aipo doc
aipo repl
aipo package
aipo bench
aipo explain <diagnostic-code>
aipo doctor
aipo conformance
```

LSP first-class: diagnostics, completion, hover, signature help, go-to-definition, references, rename, semantic tokens, document/workspace symbols, code actions, inlay hints opcionais e host-schema-aware completion.

Debugger/DAP: breakpoints, step, stack, locals, closures, tasks e host handles representáveis. Profiler: bytecode hotspots, allocation/GC, host-call cost e task latency.

Formatter é idempotent e possui golden corpus. Tree-sitter/editor grammars são derivados/validados contra syntax fixtures, não parser canônico.

Tooling aceita `--message-format=jsonl` com stable diagnostic codes para IDEs/code agents.

## 13. Packages — fechado

Manifest oficial: `aipo.toml`; lockfile: `aipo.lock`.

Resolver SemVer determinístico; lockfile registra exact versions, source, checksum e target/capability metadata relevante.

### Segurança

- Nenhum arbitrary install/postinstall lifecycle script na V1.
- Registry artifacts têm checksums obrigatórios.
- Dependency graph e capabilities são auditáveis por `aipo package audit`.
- Offline cache e vendoring suportados.
- Path dependencies para desenvolvimento; Git dependency suportada somente com revision pin no lockfile.
- Package declara targets compatíveis e capabilities solicitadas.

Registry signing/transparency log é evolução desejada; checksums + HTTPS + immutable version artifacts são baseline.

## 14. Code agents — interface arquitetural fechada

A implementação detalhada será documentada na próxima etapa, mas o contrato do produto fica fechado:

- specs canônicas versionadas;
- ADPs com status e supersession;
- fixtures positivas/negativas por feature;
- snapshots syntax/HIR/IR/bytecode/diagnostics;
- machine-readable diagnostics JSONL;
- machine-readable grammar/schema;
- **Aipo Host Schema (AHS)** emitido por Poppy e outros hosts;
- conformance matrix VM↔JS↔host;
- comandos estáveis via CLI/xtask sem exigir interpretação humana de logs;
- acceptance criteria verticais para cada feature.

AHS contém modules, types, host values, handles, functions, params, return contracts, mutability, async, docs, capabilities, deprecation/version e trailing-block subject metadata.

## 15. Performance — fechado como contrato de engenharia

Não canonizar micro-otimizações antes de medir.

### Baseline arquitetural

- Stack-based bytecode VM.
- Compact instruction encoding + constant pool.
- Interned identifiers/symbols.
- No heap allocation para arithmetic scalar comum.
- Sequence lazy evita temporary Lists.
- Host values pequenos permanecem unboxed/copy-friendly quando a VM representation permitir sem quebrar simplicidade.
- Incremental managed GC via `gc-arena` como primeiro spike; provider pode mudar se profiling reprovar.
- No JIT/Cranelift/NaN-boxing/custom allocator antes de benchmark provar necessidade.

### Performance governance

`aipo bench` mantém corpus representativo: arithmetic, closures, list/dict, Sequence, strings Unicode, regex, GC-heavy, async scheduler, host calls, ECS bindings, hot reload e JS differential workload.

CI guarda baseline por classe e bloqueia regressões acima de budget definido após estabilização inicial. Poppy mede frame-time/GC debt em workloads de 60/120 Hz. O objetivo é **previsibilidade + regressão controlada**, não perseguir benchmark sintético às custas da linguagem.

## 16. Diagnostics — fechado transversalmente

Todo subsistema usa códigos estáveis (`AIPO-...`). Um diagnostic ideal contém: mensagem curta, primary span, related spans, why, suggested fix, target/capability context e machine-readable fields.

Errors de parser tentam recovery sem avalanche. Type/contract inference explica “known”, “unknown” e narrowing. Host stale handle/security fault não aparece como panic Rust.

## 17. Determinismo — fechado transversalmente

Portable semantics define ordered collections, stable sort, deterministic seeded RNG e explicit clocks. Poppy profile acrescenta fixed tick, event/system ordering, task safe points, command buffer ordering e state digest/replay fixtures.

Non-deterministic sources (`time.now`, crypto RNG, network) são explicitamente host capabilities e podem ser interceptados/substituídos em deterministic test profile.

## 18. Matriz de status 10/10

| Área | Design | Prova exigida |
| --- | --- | --- |
| Simplicidade | Fechado | five-cost + ADP |
| Legibilidade | Fechado | formatter corpus + diagnostics |
| Semântica | Fechado | conformance VM↔JS |
| Uso geral | Fechado | CLI + Web + portable package |
| Declarativo | Fechado | UI/config/HTML-style proof programs |
| Async | Fechado | ordering/cancellation suite |
| Game script | Fechado | Poppy real game |
| ECS | Fechado | scope/command-buffer tests |
| Hot reload | Fechado G1 | migration matrix |
| JS | Fechado | differential suite + source maps |
| Sandbox | Fechado | adversarial/fuzz/resource tests |
| Tooling | Fechado | CLI/LSP/DAP contracts |
| Packages | Fechado | resolver/lock/audit fixtures |
| Code agents | Interface fechada | machine-readable workflow |
| Performance | Contrato fechado | benchmark/regression corpus |

## O que continua deliberadamente aberto

Implementação concreta ainda deve validar: GC provider final; exact bytecode encoding; thresholds de performance; registry signing/transparency; timezone package provider; crypto provider por plataforma; debugger UI; advanced regex package; full heap hot reload; JIT. Esses itens **não bloqueiam a semântica V1** e devem ser decididos por spike/benchmark, não preferência estética.

## Próxima fase

Com o desenho de produto/linguagem fechado, a próxima etapa passa a ser **documentação de implementação e documentação para code agents**: decomposition por crates, dependency rules, waves/milestones, vertical slices, ADPs executáveis, schemas, fixtures, conformance, Gauntlet Loops, context packs e prompt/agent contracts.

## 19. Sistema de documentação de implementação — fechado

A implementação passa a obedecer ao caderno canônico [Aipo — Sistema de Documentação de Implementação e Diretivas para Code Agents](Aipo — Sistema de Documentação de Implementação e  3dc9bb7d023f816fbd68e44b28f84554.md) e às suas subpáginas de Rust Engineering, dependency rules, code agents/Prumo e Waves/Gauntlet.

Regras obrigatórias:

- Prumo CLI estrutura e governa a documentação de implementação no repositório;
- repository canonical state supera generated adapters/caches;
- Goals/Plans/Evidence/Gates formalizam trabalho e conclusão;
- micro-contextos seguem progressive context;
- features são implementadas por vertical slices com acceptance criteria executáveis;
- Rust segue Safe Rust first, Clean Code pragmático e modularidade por domínio;
- `unsafe` é excepcional, documentado, auditado e testado;
- dependency graph de crates é acíclico e target-neutral no núcleo;
- docs impact/delta/readiness/contradictions são tratados como engenharia, não housekeeping;
- Gauntlet Loop mede qualidade por evidência e não altera rubricas para inflar nota.

Uma implementação que viole esse sistema pode funcionar localmente, mas não é considerada conforme à arquitetura 10/10 da Aipo.