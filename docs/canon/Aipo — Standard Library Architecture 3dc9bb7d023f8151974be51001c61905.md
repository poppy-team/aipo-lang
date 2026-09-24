# Aipo — Standard Library Architecture

<aside>
📚

**Status:** design em amadurecimento. Esta página registra a arquitetura da stdlib da Aipo e separa decisões aprovadas de questões ainda abertas. A stdlib deve servir igualmente a scripting geral, Poppy e backend JavaScript sem criar dialetos.

</aside>

## Objetivo

Construir uma standard library pequena na superfície, ampla em capacidade e previsível na organização. A maior parte do poder deve vir de módulos/bibliotecas, não de novas keywords ou built-ins.

## Regra de camadas

```
Aipo Language Core
        ↓
Small Prelude
        ↓
Portable Stdlib
        ↓
Capability-aware Host Stdlib
        ↓
Host Profiles
   ├── Poppy
   └── Web
        ↓
Official Packages
```

### 1. Language Core

Somente semântica que precisa existir para a linguagem funcionar: valores fundamentais, funções, closures, structs/interfaces, collections fundamentais, Failure, módulos e primitives de runtime. Evitar transformar conveniências em built-ins.

### 2. Small Prelude

Conjunto mínimo de operações extremamente frequentes e universais disponível sem import. Deve permanecer deliberadamente pequeno e estável.

Candidatos a revisar: `len`, `copy`, `same`, `some`, conversões fundamentais (`Int`, `Float`, `Byte`, `String`) e primitives diretamente ligadas à semântica de `Failure`. A inclusão final de cada nome precisa justificar poluição global.

### 3. Portable Standard Library

Módulos cuja semântica deve ser reproduzível entre VM nativa, Poppy e backend JavaScript.

Primeira taxonomia candidata:

```
collections
text
math
random
json
binary
encoding
time.core
functional
iter
reflect-lite
```

`reflect-lite` é apenas nome de trabalho: qualquer introspecção deve ser deliberadamente pequena, estável e incapaz de virar metaprogramação geral acidental.

### 4. Capability-aware Host Stdlib

APIs gerais, porém dependentes do ambiente, são expostas por módulos padronizados e checadas por target/capability em vez de escondidas no core.

Candidatos:

```
fs
path
process
env
net
http
time
crypto
console
```

O compilador/tooling deve conseguir explicar quando um target/host não oferece uma capability requerida.

O slice `env.read` segue essa fronteira: `env.get` e `env.has` são callbacks nativos que recebem o
`Vm` corrente, consultam o `EnvironmentSource` instalado no `HostContext` desse VM e exigem a
capability efetiva `env.read`. A instalação de um provider não concede a capability; a ausência
do provider é negada com `AIPO_RT_CAPABILITY_DENIED`, enquanto a ausência de uma variável é um
`none`/`false` normal. O metadata de native functions registra `env.read` para auditoria, mas o
registry de metadata continua sendo catálogo, não autorização.

### 5. Host Profiles

Host profiles adicionam APIs próprias sem mudar a linguagem.

```
poppy.*
web.*
```

Poppy fornece ECS, input, assets, audio, physics, UI/runtime e demais APIs da engine. Web fornece DOM/Browser APIs quando aplicável.

### 6. Official Packages

Recursos úteis mas grandes, especializados ou de evolução independente devem ficar fora da stdlib, mesmo quando mantidos oficialmente.

Candidatos futuros: database clients, templating avançado, image codecs adicionais, archive/compression families, web frameworks, serialization formats menos universais, graphics helpers e domain SDKs.

## Princípios aprovados

- A stdlib não deve duplicar operações sob vários nomes.
- Preferir funções/módulos normais a sintaxe nova.
- Nomes devem ser claros; evitar abreviações obscuras.
- APIs portáteis devem ter differential tests VM↔JS.
- APIs dependentes de host precisam declarar capabilities.
- Falhas recuperáveis usam `Failure`; programmer/runtime faults continuam distintos.
- Async APIs devem integrar com `Task[T]`, cancellation e structured concurrency em vez de callbacks paralelos incompatíveis.
- Tipos host-specific não podem vazar para módulos portáteis.
- Poppy é consumidor e laboratório da stdlib, não autoridade para redefinir semântica geral.

## Questões de design para a conversa

1. Quão pequeno deve ser o Prelude?
2. `iter` deve ser um conceito/library central ou collections devem expor toda iteração diretamente?
3. `Set` entra como collection fundamental ou módulo de stdlib?
4. `Date/Time` deve ser portátil enquanto clock/filesystem permanecem capability-based?
5. Até onde `math` incorpora vetores/matrizes gerais sem competir com host values da Poppy?
6. Regex entra na stdlib portátil ou como package oficial?
7. HTTP pertence à stdlib de host ou package oficial?
8. Reflection mínima vale o custo semântico?
9. Quais APIs devem ser async-first desde a primeira versão?
10. Quais guarantees de determinismo precisam existir na stdlib portátil?

## Critério de 10/10 da stdlib

Uma stdlib excelente para Aipo deve tornar comuns os programas de uso geral sem fazer a linguagem parecer maior; permitir tooling/autocomplete forte; preservar paridade VM↔JS; integrar naturalmente com hosts; ser amigável para code agents; e permitir que capacidades especializadas evoluam fora do núcleo sem fragmentar o ecossistema.

## Decisões aprovadas — pipe, lambdas, networking e crypto

### Pipe operator como ergonomia canônica

O operador `|>` permanece parte canônica da ergonomia Aipo e deve ser implementado como açúcar simples, previsível e target-neutral. Seu papel principal é transformar pipelines longos em leitura esquerda→direita sem criar um modelo funcional paralelo.

```
let names = players
    |> filter(player => player.alive())
    |> map(player => player.name)
    |> sort
```

Regra de lowering a fechar formalmente: `value |> function` baixa para `function(value)`; chamadas com argumentos adicionais devem ter uma única regra canônica e sem placeholders mágicos inicialmente. O pipe deve interoperar com funções normais, lambdas, dot-call e APIs de iterator sem introduzir overloads especiais.

### Lambdas de expressão — direção aprovada

Aipo manterá `fn(...) ... end` para funções anônimas completas e `do ... end` para trailing blocks, mas ganhará uma forma curta para **lambdas de uma única expressão**.

Forma candidata preferida:

```
player => player.health > 0

(a, b) => a + b
```

`=>` é preferido a `->` porque `->` já pertence aos contratos de retorno. A lambda curta possui retorno implícito apenas porque seu corpo é obrigatoriamente uma única expressão; blocos, múltiplas instruções, `return` explícito e lógica maior continuam usando `fn ... end` ou trailing block.

Objetivo: melhorar collections, iterators, pipelines, callbacks pequenos e APIs declarativas sem criar uma segunda linguagem dentro da Aipo.

Exemplos:

```
let alive = players.filter(player => player.health > 0)

let names = players
    .iter()
    .filter(player => player.alive())
    .map(player => player.name)
    .collect()
```

Questões para ADP: zero-argument lambda, contracts em parâmetros de lambda curta, precedência de `=>`, destructuring em parâmetros, async lambda e diagnostics de ambiguity. A primeira versão deve ser propositalmente pequena.

### HTTP e networking entram na stdlib oficial

`http` e `net` deixam de ser candidatos a packages externos e passam a fazer parte da standard library capability-aware.

- `http`: API de alto nível para request/response, headers, URLs, client e eventualmente server onde o host oferecer suporte.
- `net`: primitives de rede de nível inferior como TCP/UDP/sockets/endpoints, sempre capability-gated e preferencialmente async-first.
- Web/JS pode mapear `http` para APIs do ambiente quando semanticamente possível; operações não suportadas pelo browser devem produzir diagnóstico de target/capability, não uma API falsa.
- Poppy pode limitar rede por capabilities de projeto/mod/package.
- APIs async usam `Task[T]`/`await`; evitar duplicatas `*_async` como superfície principal.

### Cryptography entra na stdlib oficial

Aipo terá módulo `crypto`, também capability-aware quando depender de entropy/keystore do host. **Aipo não implementará primitivas criptográficas próprias.** A VM Rust deve usar bibliotecas criptográficas auditadas/maduras; Web/JS deve usar APIs criptográficas seguras do ambiente quando equivalentes.

Taxonomia inicial candidata:

```
crypto.hash
crypto.mac
crypto.kdf
crypto.random
crypto.aead
crypto.signature
crypto.password
```

Princípios:

- oferecer APIs de alto nível com defaults seguros;
- não expor algoritmos inseguros/obsoletos como caminho fácil;
- separar random determinístico (`random`) de CSPRNG (`crypto.random`);
- senha não usa hash simples: `crypto.password` fornece password hashing/KDF apropriado;
- encryption autenticada deve ser preferida a combinação manual cipher+MAC;
- chaves/secrets devem ter representação controlada e evitar conversão textual acidental;
- operações que dependam de keystore/system entropy declaram capabilities;
- comportamento VM↔JS deve ter testes de vetores oficiais quando a mesma primitive existir nos dois hosts.

### Matriz atualizada de stdlib

```
Portable / mostly portable
├── collections
├── iter
├── text
├── math
├── random
├── json
├── binary
├── encoding
├── path
├── time
├── task
├── testing
└── log

Capability-aware official stdlib
├── fs
├── process
├── env
├── console
├── http
├── net
└── crypto

Host profiles
├── poppy.*
└── web.*
```

### Regra de ergonomia funcional

Aipo passa a tratar estas quatro superfícies como complementares, não concorrentes:

```
fn(...) ... end          função/closure completa
value => expression      lambda curta
call(...) do ... end     bloco/callback declarativo
value |> transform       composição esquerda→direita
```

Cada uma resolve um problema distinto. Novas formas funcionais só devem ser adicionadas se um caso importante não couber claramente nessas quatro.