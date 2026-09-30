# Evidência — P00-G16 / Revisão de Saída da Wave 1

**Goal:** `P00-G16` — auditar o subset do MVP certificado contra os critérios de saída e entregá-lo com uma pontuação do gauntlet
**Fase:** P00 (Foundation) · **Registrado em:** 2026-09-19
**Ambiente:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Anexos de prova dos gates exigidos. Os comandos são reproduzíveis a partir da raiz do repositório.
A especificação do corpus está em `docs/conformance/README.md`; este registro é a pontuação e o hand-off, não uma segunda especificação.

## Por que este goal existiu

O `P00-G15` fechou os três últimos residuais (`AIPO_SEM_CONTRACT_VIOLATION_STATIC`, conformidade estrutural de interfaces, ADP-001 Q3/Q4/Q5) e ambos os documentos ADP passaram a `resolvido`. O risco restante não era uma lacuna da linguagem, e sim uma lacuna de hand-off: nenhum registro único classificava cada critério de saída da Wave 1 em `docs/waves/wave-1-mvp.md` como certificado vs. explicitamente não entregue, com o comando que prova cada afirmação.

O `P00-G16` também carregou duas conclusões semânticas encontradas durante a janela de auditoria (visibilidade em escopo de módulo e `fn` local canônica), certificadas por `programs/19_local_functions` e `programs/20_module_scope`. Este registro cobre tanto a baseline de código quanto a auditoria.

## Baseline certificada

| Tipo | Quantidade | Local |
|---|---|---|
| Programas executáveis + snapshots de stdout | 20 + 20 | `docs/conformance/programs/` (`01`–`20`) |
| Fixtures de diagnóstico + códigos esperados | 19 + 19 | `docs/conformance/diagnostics/` (`01`–`19`) |
| Pares golden do formatter | 8 + 8 | `docs/conformance/formatting/` |
| Casos de módulos | 3 | `docs/conformance/modules/` (`basic`, `cycle`, `missing`) |

Inventário verificado em disco (`20/20`, `19/19`, `8/8`, `3` entry points). Nenhum `.stdout` órfão sem `.aipo`, nenhum `.code` sem `.aipo`.

Nova certificação desde o `P00-G15`:

| Artefato | Comprova |
|---|---|
| `docs/conformance/programs/19_local_functions.aipo` | declarações `fn` locais: captura compartilhada de `var` (`create_counter`), autorrecursão via `FillSelfCapture`, captura léxica de parâmetro envolvente |
| `docs/conformance/programs/20_module_scope.aipo` | bindings de escopo de módulo visíveis dentro de corpos `fn`: `var total` mutado por `add`, `let scale` lido por `scaled` |

Execução direta:

```
$ cargo run -q -p aipo-cli -- run docs/conformance/programs/19_local_functions.aipo
3
120
small
large

$ cargo run -q -p aipo-cli -- run docs/conformance/programs/20_module_scope.aipo
total: 5
40
```

Ambos coincidem com seus `.stdout` commitados.

## Rubric do gauntlet — 100%

| Gate | Peso | Comando | Resultado |
|---|---|---|---|
| MVP executável | 30% | `cargo test -p aipo-cli --test conformance` | **aprovado** — 13/13, todas as fixtures de programas e módulos coincidem com o stdout commitado |
| Disciplina de diagnósticos | 20% | mesma suíte, testes de falha | **aprovado** — toda fixture falha com seu código commitado; falhas pelo motivo errado fariam a suíte falhar |
| Determinismo | 20% | `cargo test -p aipo-formatter` + testes do corpus de `fmt --check` | **aprovado** — 6 + 4 + 1 = 11/11; formatter idempotente, fontes canônicas não reportam drift |
| Robustez | 15% | `cargo test -p aipo-cli --test fuzz_smoke` | **aprovado** — 3/3; bytes aleatórios, programas mutados e truncados nunca causam panic |
| Gates do repositório | 15% | `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo doc --workspace --no-deps` | **aprovado** — os quatro verdes |

Saída dos comandos:

```
$ cargo test -p aipo-cli --test conformance
test result: ok. 13 passed; 0 failed

$ cargo test -p aipo-formatter
test result: ok. 6 passed; 0 failed
test result: ok. 4 passed; 0 failed
test result: ok. 1 passed; 0 failed

$ cargo test -p aipo-cli --test fuzz_smoke
test result: ok. 3 passed; 0 failed

$ cargo test --workspace
PASSED=146 FAILED=0
```

## Gate: `fmt` / `clippy` / `test` / `doc` / `prumo`

```
$ cargo fmt --all -- --check
FMT_EXIT=0

$ cargo clippy --workspace --all-targets -- -D warnings
CLIPPY_EXIT=0 (warnings: 0)

$ cargo test --workspace
PASSED=146 FAILED=0

$ cargo doc --workspace --no-deps
DOC_EXIT=0 (warnings: 0)

$ prumo validate .
Prumo validation passed.

$ prumo doctor .
Prumo Doctor: all checks passed cleanly.
```

Verificações adicionais de superfície:

```
$ cargo run -q -p aipo-cli -- check docs/conformance/programs/20_module_scope.aipo
CHECK_EXIT=0

$ cargo run -q -p aipo-cli -- check docs/conformance/diagnostics/15_sem_contract_violation.aipo
error: [AIPO_SEM_CONTRACT_VIOLATION_STATIC] argument for parameter 'x' of 'double' cannot satisfy contract 'Int'
CHECK_FAIL_EXIT=1

$ cargo run -q -p aipo-cli -- --version
aipo 0.1.0
```

## Gate de saída do MVP — todos os critérios classificados

`docs/waves/wave-1-mvp.md` lista cinco condições de saída. Status de cada uma:

| Condição de saída | Status | Evidência |
|---|---|---|
| `aipo run` executa programas pequenos reais | **certificada** | snapshots de `programs/01`–`20`; `cargo test -p aipo-cli --test conformance` 13/13 |
| `aipo check` reporta diagnósticos; `aipo fmt` idempotente | **certificada** | códigos de `diagnostics/01`–`19`; goldens de `formatting/01`–`08` + `cargo test -p aipo-formatter` 11/11 |
| fixtures de pass/fail verdes; snapshots commitados; testes de integração verdes | **certificada** | conformance 13/13; `cargo test --workspace` 146/146 |
| Nenhum panic do Rust escapa como erro de usuário (fuzz smoke) | **certificada** | `cargo test -p aipo-cli --test fuzz_smoke` 3/3 |
| CI: fmt/clippy/test/doc verdes; `prumo validate` + `prumo doctor` verdes; delta de docs resolvido | **certificada** | gates acima; saídas do `prumo` acima; tabela de delta abaixo |

Subset de linguagem do MVP (`docs/waves/wave-1-mvp.md`, recorte formal) — todos os itens certificados pelo corpus ou pelas suítes unitárias:

| Área do subset | Status | Prova representativa |
|---|---|---|
| Literais (`none`/`true`/`false`, Int ±(2^53−1), Float finito, `Byte`, `"…"`, `f/r/fr`, `"""`) | **certificado** | `programs/01_hello`, `07_strings_and_math`, `17_unicode_nfc` |
| Bindings (`let`/`var`, caminhos `var`/`!`/`self!`/`fixed`, superfície de destructuring) | **certificado** | `programs/05_structs_and_impl`, `10_integrated`, `19_local_functions`, `20_module_scope` |
| Operadores (`.`, `?.`, chamada/índice, `* / div %`, `+ -`, `..`, comparações, `is`, `not`/`and`/`or`, `or_else`, `\|>`, atribuição composta) | **certificado** | `programs/03_control_flow`, `04_collections`, `07_strings_and_math`, `09_slicing` |
| Fluxo de controle (`if/elif/else`, `if/then` inline, `match/when`, `loop`/`while`/`repeat`/`each`, `break`/`continue`) | **certificado** | `programs/03_control_flow`, `10_integrated` |
| Funções (`fn`, defaults, argumentos nomeados, `fn` local, `fn` anônima, closures + captura por iteração, `return`) | **certificado** | `programs/02_recursion`, `06_closures`, `11_defaults_and_named_args`, `19_local_functions` |
| Chamadas (posicional-antes-de-nomeado, `do…end` trailing, pipeline, dot-call para funções associadas de `impl`) | **certificado** | `programs/06_closures`, `10_integrated`, `11_defaults_and_named_args` |
| Dados (`struct`+defaults+`fixed`, `Type{…}`, `impl`+`init`+`invariant`, operações de `List`/`Dict`, índices negativos, `a..b`) | **certificado** | `programs/04_collections`, `05_structs_and_impl`, `09_slicing`, `12_bytes`, `13_init_and_invariant`, `15_invariant_on_mutation` |
| Erros (`fail`, `or_else`, `attempt/failed`, `err.message`; faults numéricos/de índice/de chave/de iteração/de contrato) | **certificado** | `programs/08_failures`, `14_signature_contracts`, `15_invariant_on_mutation`, `diagnostics/06`–`10`, `14` |
| Contratos (param/return `Type`, `!`, `-> T`, `T?`; verificações de fronteira em runtime; narrowing com `is`; interfaces + `satisfy` estrutural) | **certificado** | `programs/14_signature_contracts`, `16_interface_contracts`, `diagnostics/12/13/15/16/17/18` |
| Módulos (um arquivo = um módulo; `import m`, `import m: names`, `import m as alias`, `export`, acíclico, init-once) | **certificado** | `modules/basic`, `modules/cycle`, `modules/missing` |
| Prelude + stdlib (`len/copy/same/some/fail`, conversões, `io`, `string`, `List`/`Dict`, `math`) | **certificado** | `programs/01_hello`, `04_collections`, `07_strings_and_math`, `17_unicode_nfc`, `18_tolerant_slices_and_clamp` |
| CLI (`run`/`check`/`fmt`, `--message-format=jsonl`, exit codes `0/1/2`) | **certificado** | suíte de conformidade (testes de `--help`/`--version`, jsonl, códigos de uso) |

## Não entrega explícita (não são lacunas, não são regressões)

O que o subset do MVP declara fora do escopo continua fora do escopo. Nenhum slice posterior herda uma lacuna oculta:

| Não entrega | Fonte do escopo | Status |
|---|---|---|
| APIs de packing de `Bytes` (`read_i32`/`write_f32`/…, `String.encode`/`Bytes.decode`) | exclusões de `docs/waves/wave-1-mvp.md`; Adiado em `docs/stdlib/mvp-subset.md` | **não entregue por design** — apenas `Bytes(count)` + índice/`len`/slice são certificados (`programs/12_bytes`) |
| Backend `aipo-js` | contratos das crates (Wave 2); exclusões do MVP | **não entregue** — o pipeline é apenas bytecode → VM; crate não iniciada |
| `Set` / `Sequence` lazy | exclusões do MVP; Adiado da stdlib | **não entregue** — nenhum kind, nenhum literal, nenhuma fixture os reivindica |
| LSP / REPL | exclusões do MVP | **não entregue** — a CLI é apenas `run`/`check`/`fmt` |
| async/await, host ABI/Poppy, pacotes/registry, regex/json/fs/http, `graphemes`, hot reload | exclusões do MVP | **não entregue** — registrado aqui para que o planejamento da Wave 2 comece de uma lista explícita |

Ambos os documentos ADP permanecem `resolvidos` e não são reabertos por este goal.

## Próximo slice recomendado

A Wave 1 está fechada. A baseline certificada acima é o ponto de partida. Ordem recomendada para o planejamento da Wave 2 (nenhum código da Wave 2 foi iniciado por este goal):

1. Spike do backend `aipo-js` contra o corpus congelado (`programs/01`–`20` como oráculo de paridade) — o menor slice que prova o segundo backend sem tocar a superfície da linguagem.
2. APIs de packing de `Bytes` somente após a decisão de paridade com JS, porque encode/decode cruza a fronteira NFC documentada no ADP-001 Q5.
3. `Set`/`Sequence` e LSP/REPL após a questão do backend, nessa ordem — cada um é independente dos demais quando o corpus é o árbitro.

## Gate: `documentation_impact`

| Artefato | Mudança |
|---|---|
| `docs/conformance/README.md` | pontuações do gauntlet com os comandos exatos; inventário verificado (`20/20`, `19/19`, `8/8`, 3 casos de módulos); lista de não entregas explícita |
| `docs/stdlib/mvp-subset.md` | a superfície Adiada lista packing de `Bytes`, `aipo-js`, `Set`/`Sequence`, LSP/REPL como não entrega explícita |
| `docs/evidence/P00-G16-wave-1-exit-review.md` | novo — este registro de hand-off |
| `docs/PRUMO.md` | o índice de evidências ganha `P00-G16` |
| `CHANGELOG.md`, `PROJECT_STATE.md` | saída da Wave 1 registrada; `P00-G16` DONE |

## Limitações conhecidas (registradas, não escondidas)

- **A auditoria certifica o subset, não todas as combinações.** O corpus mais 146 testes unitários/de integração é o árbitro; interações de features não testadas fora do corpus ainda estão sujeitas à política de não invenção e viram ADPs quando encontradas.
- **Os contratos estáticos continuam apenas comprováveis.** `AIPO_SEM_CONTRACT_VIOLATION_STATIC` dispara para literais contra contratos de tipo core e `none` contra não nullable; todo o resto continua sendo um fault de contrato em runtime na fronteira da chamada.
- **A conformidade de interfaces é nome + aridade visível ao chamador.** Um nome coincidente com um tipo de parâmetro incompatível é descoberto quando a operação executa, não na verificação do contrato.
- **Os gates do `prumo` são fornecidos pelo ambiente.** `prumo validate` / `prumo doctor` estão verdes nesta máquina; o CI precisa executar os mesmos quatro gates do cargo mais estes dois.
