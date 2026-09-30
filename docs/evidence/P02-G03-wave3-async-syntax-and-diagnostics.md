# Evidência — P02-G03 / Wave 3 Infra: Sintaxe `async fn`, `await do` e Diagnósticos Estáticos

**Goal:** `P02-G03` — Implementar e certificar sintaxe `async fn`, desugaring de `await do`, diagnósticos estáticos (`AIPO_SEM_AWAIT_IN_SUBEXPRESSION`, `AIPO_SEM_FORGOTTEN_TASK`, `AIPO_SEM_NESTED_AWAIT_DO`), com testes e conformance.
**Fase:** P02 (Wave 3 — Async & Tipos Expandidos) · **Registrado em:** 2026-09-21
**Ambiente:** Linux x86_64, rustc/cargo 1.98.1, Node v24.18.0

## Gates (todos executados, todos verdes)

```
$ cargo fmt --all -- --check                                           # exit 0
$ cargo clippy --workspace --all-targets -- -D warnings                 # 0 warnings
$ cargo test --workspace --all-targets                                  # 301 passed, 0 failed
$ cargo doc --workspace --no-deps                                       # 0 warnings
$ node crates/aipo-js/runtime/selftest.mjs                              # all assertions passed
$ cargo test -p aipo-cli --test conformance                             # 13 passed (27 programs, 29 diagnostics)
$ cargo test -p aipo-js --test differential                             # 8 passed (VM == JS == stdout)
$ cargo test -p aipo-js --test properties                               # 3 passed
```

`prumo validate` / `prumo doctor` não podem ser executados neste ambiente: a instalação local do prumo 0.6.0
está sem seus recursos de framework (`framework-check` reporta
`open src/prumo/resources/catalog/catalog.json: no such file or directory` e
`missing schema: prumo.schema.json`). Trata-se de um defeito de instalação, independente do
repositório — nenhum arquivo do projeto sob controle de versão referencia esses caminhos.

## Resumo da Implementação

| Área | Decisões & Detalhes de Implementação |
|---|---|
| **`async fn` como protocolo de chamada** | Chamar uma `async fn` cria a task de forma eager e devolve seu handle `Task`; o `await` é explícito em todo lugar (canon: sem paralelismo implícito). O parsing é compartilhado por `Item::Fn` de nível superior, `Stmt::Fn` local, closures anônimas `async fn(...)` e métodos de `impl`. |
| **`async fn` de nível superior é um item** | `is_item_start` não tratava `TokenKind::Async`, então uma `async fn` de nível superior sofria lowering como statement local e o `AIPO_SEM_FORGOTTEN_TASK` nunca a enxergava. Corrigido em `crates/aipo-syntax/src/parser.rs`. |
| **`async fn` anônima** | `parse_async_function_decl` também é usada para a forma de valor; antes, a forma anônima nunca avançava além de `fn`. Resultado: uma `async fn` anônima é uma closure com o mesmo protocolo de chamada. |
| **Métodos `async fn` em `impl`** | `register_struct_method` carrega uma flag `is_async` pelos quatro pontos de chamada de registro (`aipo-cli`, `aipo-testkit`, diferencial de `aipo-js`, data/errors de `aipo-vm`); a VM e o runtime JS criam o corpo do método como task com o receiver como argumento 0. |
| **`await do … end`** | Sofre desugaring para awaits sequenciais sobre o corpo do bloco; a forma de bloco aninhada reporta `AIPO_SEM_NESTED_AWAIT_DO`, e `await` em subexpressões arbitrárias reporta `AIPO_SEM_AWAIT_IN_SUBEXPRESSION`. |
| **`AIPO_SEM_FORGOTTEN_TASK`** | Uma expressão sabidamente `Task` usada como statement solto, sem `await`, binding ou combinador, é reportada; vincular e retornar uma task permanecem silenciosos. |
| **Contrato estático de await (ADP-006 §G)** | Uma `Task` só é produzida por uma chamada de `async fn` ou por um combinador, então um operando literal de `await` é comprovável antes da execução e reporta `AIPO_SEM_CONTRACT_VIOLATION_STATIC` em vez de chegar ao fault de runtime. |
| **Recuperação de contrato paramétrico** | O caminho de skip para `Task[T]`/`List[Int]` consumia o `)` de fechamento do chamador, causando em cascata um `AIPO_PARSE_UNEXPECTED_TOKEN` espúrio após o `AIPO_SEM_PARAMETRIC_CONTRACT` real; a recuperação agora para quando o colchete da própria seção fecha. |
| **Ciclo de await e cancelamento** | Conduzir/aguardar uma task cujo resultado já havia sido consumido reexecutava o corpo para sempre; o resultado agora é absorvido na fronteira e um ciclo gera fault com `AIPO_RT_AWAIT_CYCLE`. Aguardar uma task cancelada gera fault com `AIPO_RT_CANCELLED`. Ambos implementados em `aipo-vm/src/vm/{failure,mod}.rs` e espelhados em `crates/aipo-js/runtime/aipo-runtime.js`. |
| **Endurecimento de literais numéricos** | Um único módulo compartilhado `aipo-lexer::number` é dono das regras de literais (bases, separadores, formas de float, limites de `i64`). O lexer e o IR builder o chamam, eliminando a divergência anterior entre a validação léxica e a materialização de constantes. Dígitos ausentes após um prefixo de base, separadores malformados e inteiros fora da faixa reportam `AIPO_LEX_INVALID_NUMBER`; um literal acima da faixa de safe-integer gera fault com `AIPO_RT_OVERFLOW`. |

## Inventário de Testes & Verificação

Testes unitários e de propriedades:

- `crates/aipo-lexer/src/lib.rs` e `crates/aipo-lexer/tests/properties.rs`: tabelas de literais numéricos (decimal/hex/binário/octal, separadores, limites) e uma propriedade de que qualquer literal aceito pelo lexer faz round-trip pelo classificador compartilhado.
- `crates/aipo-ir/src/lib.rs`: o IR builder constrói o mesmo valor que o lexer classificou para toda forma de literal aceita.
- `crates/aipo-sema/src/lib.rs`: os três diagnósticos estáticos e o contrato estático de await — cada caso positivo e os casos negativos silenciosos (task vinculada, task retornada ao chamador, chamada de `async fn` aguardada).
- `crates/aipo-vm/tests/wave3_async_semantics.rs`: spawn/await de `async fn`, ordenação de `await do`, fronteiras de forgotten-task/reporte, faults de await-cycle e de await cancelado.
- `crates/aipo-js/tests/properties.rs`: propriedades de literais numéricos e de emissão async sobre o corpus.

Corpus de conformidade (29 fixtures de diagnóstico no total):

- `docs/conformance/programs/24_async_functions_and_await.aipo` — `async fn` de nível superior, local, anônima e método de `impl`; task vinculada, passada e com join; `await` como statement, inicializador e valor de retorno.
- `docs/conformance/programs/25_await_do.aipo` — açúcar sequencial `await do` sobre várias tasks, incluindo uma `fn` local que aguarda dentro do bloco.
- `docs/conformance/programs/26_async_combinators.aipo` — `task.all`/`race`/`timeout`/`cancel`/`sleep`/`group` compostos com ordenação determinística.
- `docs/conformance/programs/27_numeric_literal_bases.aipo` — toda base de literal, separador e limite de faixa de `i64` aceitos.
- `docs/conformance/diagnostics/20_lex_invalid_number.aipo` → `AIPO_LEX_INVALID_NUMBER`
- `docs/conformance/diagnostics/21_runtime_overflow_literal.aipo` → `AIPO_RT_OVERFLOW`
- `docs/conformance/diagnostics/22_runtime_overflow_unrepresentable.aipo` → `AIPO_RT_OVERFLOW`
- `docs/conformance/diagnostics/23_sem_await_in_subexpression.aipo` → `AIPO_SEM_AWAIT_IN_SUBEXPRESSION`
- `docs/conformance/diagnostics/24_sem_forgotten_task.aipo` → `AIPO_SEM_FORGOTTEN_TASK`
- `docs/conformance/diagnostics/25_sem_nested_await_do.aipo` → `AIPO_SEM_NESTED_AWAIT_DO`
- `docs/conformance/diagnostics/26_runtime_uncaught_failure_through_await.aipo` → `AIPO_RT_FAILURE_UNCAUGHT`
- `docs/conformance/diagnostics/27_runtime_await_cycle.aipo` → `AIPO_RT_AWAIT_CYCLE`
- `docs/conformance/diagnostics/28_runtime_await_cancelled.aipo` → `AIPO_RT_CANCELLED`
- `docs/conformance/diagnostics/29_sem_await_literal.aipo` → `AIPO_SEM_CONTRACT_VIOLATION_STATIC`

Ambas as suítes enumeram o corpus a partir do disco, então toda nova fixture é exercitada por
`cargo test -p aipo-cli --test conformance` e, para paridade diferencial, por
`test_vm_and_js_agree_with_committed_stdout` em `crates/aipo-js/tests/differential.rs`.

## Impacto na Documentação

- `docs/diagnostics/catalog.md`: quatro novos códigos semânticos e três novos códigos de fault de runtime registrados.
- `docs/conformance/README.md`: linhas de programas 24–27, linhas de diagnósticos 20–28, a tabela de entrega
  das Waves 2/3 e o inventário pós-`P02-G03`.
- `CHANGELOG.md`: a adição do `P02-G03` e as quatro correções de correção.
- `PROJECT_STATE.md`: slice concluído e goal ativo.
- `.ai/goals/P02-G03.goal.json`: critérios de aceitação objetivos, não-objetivos e histórico de estados.

## Residuais

- `AIPO_RT_AWAIT_IN_CALLBACK` é exercitado pela evidência da stdlib da Wave 3 (`P02-G02`); este goal
  não adiciona uma fixture para ele.
- Os critérios de saída da Wave 3 em `docs/waves/wave-3-async.md` estão atendidos para sintaxe, diagnósticos,
  faults de cancelamento e paridade VM↔JS; a Wave 4 (host ABI/Poppy) continua fora do escopo.
