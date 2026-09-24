# Aipo — Stdlib V1 Canônica e Contratos de Portabilidade

<aside>
📚

**Status: canônico para V1, salvo pontos marcados como pós-V1.** Esta página fecha a arquitetura funcional da standard library da Aipo segundo a filosofia de baixa carga cognitiva, uma forma principal por conceito, portabilidade VM↔JS e host capabilities explícitas.

</aside>

## Princípios normativos

- O core da linguagem continua pequeno; capacidade cresce por módulos.
- O Prelude é mínimo e não vira depósito de conveniências.
- APIs portáteis devem ter a mesma semântica observável na VM Rust e no backend JavaScript.
- APIs dependentes do ambiente usam capabilities e target checking.
- Uma API host-only não ganha fallback no backend: o target deve anunciar a capability que pode
  cumprir; o slice `fs` é uma exceção explícita à paridade VM↔JS.
- O caminho simples deve ser o caminho seguro.
- Async I/O é async-first; evitar pares artificiais `foo`/`foo_async` como API principal.
- Um conceito recebe um nome canônico; não duplicar `map/select/transform`, `reduce/fold/inject`, etc.
- Tipos específicos de host nunca vazam para a stdlib portátil.

## Modelo de camadas

```
Language Core
    ↓
Small Prelude
    ↓
Portable Stdlib
    ↓
Capability-aware Stdlib
    ↓
Host Profiles
    ↓
Official Packages
```

### Prelude V1

Disponível sem import somente quando faz parte do modelo mental fundamental: `none`, `true`, `false`, `Int`, `Float`, `Byte`, `String`, `List`, `Dict`, `Bytes`, `len`, `copy`, `same`, `some` e `fail`.

`print`, `map`, `filter`, `reduce`, `random`, `sleep`, `open`, `json`, `assert`, `min` e `max` não entram globalmente.

## Collections — modelo canônico

`List`, `Dict` e `Bytes` são fundamentais. `Set` pertence à stdlib.

O uso normal é **eager** e simples:

```
let alive = players.filter(player => player.alive())
let names = alive.map(player => player.name)
let target = players.find(player => player.id == target_id)
```

Quando o usuário quer evitar collections intermediárias, usa `.lazy()` explicitamente:

```
let names = players
    .lazy()
    .filter(player => player.alive())
    .map(player => player.name)
    .take(20)
    .collect()
```

A API pública chama o pipeline lazy de **Sequence**. `Iterator` permanece detalhe interno do runtime/tooling.

```
List / Dict / Set
      ↓ .lazy()
   Sequence
      ↓
 map/filter/take/...
      ↓
   collect()
```

### Vocabulário único

As operações eager e lazy preservam os mesmos nomes: `map`, `filter`, `flat_map`, `find`, `any`, `all`, `count`, `reduce`, `take`, `skip`, `group_by`, `distinct`, `zip`, `chain`, `chunk`, `window`, `enumerate` e equivalentes aprovadas.

`reduce` sempre recebe valor inicial:

```
let total = prices.reduce(0.0, (total, price) => total + price)
```

Não existe variante implícita que precise decidir o que fazer com coleção vazia.

### Ordenação e determinismo

- `Dict` preserva ordem de inserção.
- `Set` preserva ordem de inserção.
- Atualizar chave existente de `Dict` mantém sua posição.
- Remover e reinserir coloca a chave no final.
- `sort`/`sort_by` são estáveis.
- Iteração observável deve ser determinística quando a fonte e os inputs forem determinísticos.

### Mutação durante iteração

Modificar estruturalmente a mesma collection durante uma iteração ativa é runtime fault claro. Modificar objetos contidos continua permitido quando o contrato de mutabilidade permitir.

### `first`, `last` e `find`

`first()`/`last()` exigem coleção não vazia; coleção vazia gera runtime fault. APIs seguras explícitas como `first_or(default)`/`last_or(default)` evitam ambiguidade entre ausência e um valor real `none`.

`find(predicate)` retorna valor ou `none`; `find_index` existe quando a presença de `none` na coleção precisa ser distinguida de “não encontrado”.

## `Sequence`, `Task` e `Stream`

Os conceitos permanecem separados:

```
List      = materializada
Sequence  = lazy síncrona
Task[T]   = um resultado futuro
Stream[T] = vários resultados futuros
```

`Stream[T]` entra quando a infraestrutura async estiver estável; não sobrecarregar `Sequence` com async.

## Pipe e lambdas

`|>` é açúcar target-neutral para composição esquerda→direita. Regra base: `value |> transform` baixa para `transform(value)`; uma chamada com argumentos adicionais injeta o valor como primeiro argumento. Não há placeholder mágico na V1.

```
let names = players
    |> filter(player => player.alive())
    |> map(player => player.name)
```

Lambda curta usa `=>` e possui exatamente uma expressão:

```
player => player.health > 0
(a, b) => a + b
() => create_player()
```

Lógica com statements usa `fn ... end`; trailing callback/declarative block usa `do ... end`.

## Text e Unicode

`String` é imutável, UTF-8 e normalizada em NFC nas fronteiras normais de construção/decodificação. Igualdade de strings compara conteúdo canônico.

### Unidade humana padrão

Operações de tamanho, recorte e posição de texto usam **extended grapheme clusters**, não bytes e não code points, quando a API é human-facing.

```
len(text)
text.slice(0, 5)
text.graphemes()
text.words()
```

Byte-level é sempre explícito:

```
let bytes = text.encode_utf8()
let byte_count = bytes.len()
```

Não haverá indexação arbitrária `text[index]` na V1; isso evita fingir acesso O(1) e evita quebrar Unicode. APIs de busca retornam posições em grafemas quando human-facing; APIs binárias retornam byte offsets explicitamente nomeados.

### Operações principais

`contains`, `starts_with`, `ends_with`, `find`, `replace`, `split`, `join`, `trim`, `lower`, `upper`, `casefold`, `slice`, `graphemes`, `words`, `lines` e normalização explícita avançada quando necessária.

Case conversion padrão é locale-neutral e determinística. Regras linguísticas/locale-aware entram em módulo/package de internationalization, não em comportamento implícito de `lower()`.

## Regex

`regex` é stdlib portátil, sem literal `/.../`.

```
import regex
let pattern = regex.compile("^[a-z0-9_]+$") or_else fail("invalid pattern")
```

A sintaxe V1 é um **subconjunto Unicode de tempo previsível/linear**, deliberadamente sem backreferences e sem look-around dependente de backtracking. Isso prioriza sandbox, previsibilidade e paridade VM↔JS. Recursos avançados incompatíveis podem existir futuramente em package separado, nunca mudando silenciosamente a semântica de `regex`.

## JSON

`json.parse(text)` retorna valor Aipo ou `Failure`; `json.stringify(value, pretty = false)` retorna String ou `Failure` quando o grafo não é serializável.

Mapeamento base:

```
null    → none
boolean → Bool
integer dentro do range Aipo → Int
outros números finitos representáveis → Float
string  → String NFC
array   → List
object  → Dict ordered
```

Na V1, chaves duplicadas de objeto são erro de parse em modo padrão. `NaN`/`Infinity` não são JSON. `Dict` ordered torna saída estável quando a ordem da fonte é estável.

## Binary e Encoding

`Bytes` é a coleção binária gerenciada. `binary` fornece readers/writers, inteiros signed/unsigned suportados pelo formato, Float, varints quando justificável e operações de slicing.

Endianness é sempre explícita quando importa; nada de “native endian” em formato portátil.

`encoding` contém UTF-8, Base64, Base64URL, Hex e percent-encoding. Decodificação inválida produz `Failure`, nunca substituição silenciosa salvo API explicitamente “lossy”.

## URL

Adicionar módulo portátil `url` baseado semanticamente no WHATWG URL model onde aplicável.

```
let endpoint = url.parse("https://example.com/api") or_else ...
```

URL parsing/normalization é separado de HTTP e de filesystem paths.

## Path e FS

`path` é puro e portátil para composição/normalização lógica. `fs` é capability-aware e async-first.
No slice host-only atual, a superfície é deliberadamente menor que a visão futura:

```
let config_path = path.join("config", "app.json")
let text = await fs.read_text(config_path)
let visible_roots = fs.roots()
```

`fs.read_text(path)` é uma task assíncrona: a chamada devolve `Task` imediatamente e o provider
só é consultado quando o scheduler executa a task. `fs.roots()` é síncrono, exige a capability
separada `filesystem.roots` e devolve `List[String]` ordenado. A policy de roots e de paths pertence
ao provider; a trait `FilesystemSource` copia strings e devolve erros tipados. Não há
`write_text`, `open`, `delete`, handle bruto, operação recursiva ou fallback de filesystem no
JavaScript/Web. A CLI não instala provider nem concede `filesystem.read`/`filesystem.roots`.

Arquivo ausente e erro de leitura do provider são `Failure` recuperável. Provider ausente ou
capability ausente é o fault estável `AIPO_RT_CAPABILITY_DENIED`. Cancelamento continua sendo
fault, nunca `Failure`. A metadata nativa declara asyncness e capabilities; o schema AHS
(`aipo_stdlib::fs::schema`) é a descrição para tooling, enquanto `NativeRegistry` permanece apenas
catálogo.

## Time

`time` contém tipos puros portáteis: `Duration`, `Date`, `TimeOfDay`, `DateTime` com offset e parsing/formatting ISO 8601. Clock é capability:

- `time.now()` → wall clock.
- `time.monotonic()` → monotonic clock.
- `task.sleep(duration)` → scheduler async.

IANA timezone database é grande, atualizável e fica em package oficial `timezone` inicialmente; o core `time` suporta UTC/fixed offsets sem depender de database global mutável.

## Task e structured concurrency

`Task[T]` é contrato built-in. `async fn foo() -> T` retorna `Task[T]` ao ser chamada e `await` produz `T`.

Módulo `task` contém `sleep`, `all`, `race`, `timeout`, `cancel`, `group`, `spawn` e primitives de cancellation.

Structured concurrency é padrão: tasks filhas pertencem a um scope/group; sair do scope cancela ou aguarda conforme contrato. `detach` não é caminho normal e pode exigir capability/policy do host.

Task criada e descartada sem `await`, grupo ou spawn explícito recebe diagnóstico.

## `await do` — semântica fechada

`await do ... end` é açúcar para awaits sequenciais, não closure e não paralelismo.

Dentro do bloco, uma expression statement cujo resultado inteiro é conhecido como `Task[T]` é aguardada; initializer de `let`/`var` conhecido como Task é aguardado e vincula `T`. Essa regra continua em blocos de controle lexicamente contidos (`if`, `each`, `match`) mas **não** entra em funções/lambdas/trailing callbacks definidos dentro dele e nunca faz await dentro de argumentos/subexpressões arbitrárias.

Valores dinâmicos/unknown exigem `await` explícito. `return` retorna da função async envolvente. `fail` segue a semântica normal de Failure. `attempt` captura apenas Failure recuperável; cancellation, budget e security faults não viram Failure. `await do` aninhado é redundante e recebe warning/diagnóstico.

## HTTP

`http` é stdlib oficial capability-aware e async-first. Superfície principal: `Client`, `Request`, `Response`, methods convenientes (`get`, `post`, ...), headers, bodies text/bytes/JSON, redirects, timeout, streaming e limits.

Defaults são seguros: TLS verification ativa, redirects limitados, body size/configurable limits, timeouts razoáveis definidos pelo host profile. Desabilitar TLS verification não pertence à API simples e, se existir para tooling local, deve ser explicitamente unsafe/dev-only no host policy.

## Net

`net` é stdlib oficial abaixo de HTTP: `tcp`, `udp`, `dns`, endpoints/addresses e `tls` quando o host oferece. Async-first. Browser target simplesmente não anuncia raw socket capabilities; target checker explica a incompatibilidade antes da execução.

## Crypto

`crypto` é stdlib oficial, mas Aipo **não cria primitives criptográficas próprias**.

Superfície de alto nível:

```
crypto.hash
crypto.mac
crypto.password
crypto.kdf
crypto.random
crypto.seal / crypto.open
crypto.signature
crypto.key
```

`crypto.password` usa password hashing moderno (baseline Argon2id em runtime que suporte). `crypto.random` usa CSPRNG e é separado de `random` seedable/determinístico.

A API cotidiana de encryption é envelope-versioned e authenticated encryption; nonce seguro é gerado internamente quando possível. Compatibilidade algorítmica explícita pode ser oferecida em submódulos avançados, mas a API simples não exige que o usuário monte cipher+nonce+MAC manualmente.

Secrets usam tipos opacos (`SecretBytes`/key handles) que não imprimem, não serializam e não entram em logs acidentalmente. Comparações sensíveis usam constant-time primitives do backend.

## Random determinístico

`random.create(seed = ...)` é PRNG determinístico e portável para testes, simulations e Poppy replay. Poppy pode expor `game.random` com seed do host. Nunca usar esse módulo para tokens, keys ou salts.

## Testing

`testing` integra com o runner oficial:

```
test("damage decreases health") do
    var player = Player{ health = 100 }
    player.damage(20)
    expect.equal(player.health, 80)
end
```

Baseline: `expect.equal`, `not_equal`, `true`, `false`, `none`, `some`, `failure`, `contains`, `approx`, helpers async e fixtures via funções normais. Property testing permanece tooling/package oficial até justificar promoção.

## Log

`log` é structured logging, com níveis `trace`, `debug`, `info`, `warning`, `error` e campos nomeados.

```
log.info("Player connected", player_id = player.id, region = region)
```

Secrets são redacted pelo tipo. Hosts mapeiam sinks sem mudar o programa.

## Capability-aware official stdlib V1

```
Portable
├── collections
├── text
├── regex
├── math
├── random
├── json
├── binary
├── encoding
├── url
├── path
├── time (pure types)
├── task
├── testing
└── log

Capability-aware
├── fs
├── env
├── process
├── console
├── http
├── net
└── crypto

Host profiles
├── poppy.*
└── web.*
```

## Implementação Rust — referências preferidas, não pins eternos

- Unicode segmentation/word boundaries: `unicode-segmentation`.
- NFC/normalization: `unicode-normalization`.
- Regex engine/reference: Rust `regex`/`regex-automata`, preservando subset Aipo próprio.
- Date/time implementation reference: `time` crate.
- URL: `url` crate/WHATWG model.
- HTTP client baseline: `reqwest` sobre runtime adapter; lower layers podem usar `hyper` quando necessário.
- TLS: `rustls` como baseline Rust-first.
- Crypto primitives: RustCrypto ecosystem; Argon2id via RustCrypto `argon2`; AEAD/backend escolhido por perfil versionado.

A API Aipo não deve espelhar APIs de crates Rust: crates são providers internos substituíveis.

## Fora da stdlib V1

Database clients, ORM, templating avançado, image codecs, archives/compression families extensas, framework web, cloud SDKs, IANA timezone database, advanced regex/backtracking, UI e graphics helpers ficam em official/community packages.

## Critério de fechamento

Uma nova API de stdlib só entra se tiver: semântica nomeada; contracts de Failure/fault; capability/target quando aplicável; VM↔JS test plan; cancellation quando async; limites de recurso; documentação de determinismo; exemplos e diagnostics esperados.