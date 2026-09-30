# Evidência — P00-G12 / Slice S11 (Hardening de Conformidade & Gate do MVP)

**Goal:** `P00-G12` — Hardening de conformidade: fixtures, snapshots, gauntlet e gate do MVP
**Fase:** P00 (Foundation) · **Slice:** S11 · **Registrado em:** 2026-09-15
**Ambiente:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Anexos de prova dos gates exigidos. Os comandos são reproduzíveis a partir da raiz do repositório.
O layout do corpus e a matriz de snapshots estão em `docs/conformance/README.md`; este registro é a
pontuação, não a especificação.

## Entregável

O slice transforma "o MVP funciona" em uma afirmação que um comando pode falsificar. Tudo o que o subset do MVP
afirma é ou executado com uma saída commitada, ou rejeitado com um código de diagnóstico commitado:

| Tipo | Quantidade | Local |
|---|---|---|
| Programas executáveis + snapshots de stdout | 11 + 11 | `docs/conformance/programs/` |
| Fixtures de diagnóstico + códigos esperados | 10 + 10 | `docs/conformance/diagnostics/` |
| Pares golden do formatter | 8 + 8 | `docs/conformance/formatting/` |
| Casos de módulos | 3 | `docs/conformance/modules/` |

Os snapshots são observações e só são regenerados sob demanda
(`AIPO_UPDATE_SNAPSHOTS=1`); os códigos de diagnóstico, as expectativas do formatter e as fixtures de módulos são
escritos à mão porque são especificações.

## Rubric do gauntlet

| Gate | Peso | Comando | Resultado |
|---|---|---|---|
| MVP executável | 30% | `cargo test -p aipo-cli --test conformance` | **aprovado** — 13/13, todas as fixtures de programas e módulos coincidem com o stdout commitado |
| Disciplina de diagnósticos | 20% | mesma suíte, testes de falha | **aprovado** — toda fixture falha com seu código commitado, incluindo a guarda de que uma fixture de parse reporta um código `AIPO_PARSE_*` |
| Determinismo | 20% | `cargo test -p aipo-formatter` | **aprovado** — 11/11; formatter idempotente, fontes canônicas não reportam drift, `fmt --check` nunca reescreve |
| Robustez | 15% | `cargo test -p aipo-cli --test fuzz_smoke` | **aprovado** — 3/3; bytes aleatórios, programas mutados e programas truncados nunca causam panic no pipeline |
| Gates do repositório | 15% | `cargo fmt --check`, `clippy -D warnings`, `test --workspace`, `doc` | **aprovado** — os quatro verdes |

Saída dos comandos:

```
$ cargo test -p aipo-cli --test conformance
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo test -p aipo-cli --test fuzz_smoke
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

$ cargo test -p aipo-formatter
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Gate: `fmt` / `check` / `clippy` / `test` / `doc`

```
$ cargo fmt --all -- --check
FMT_EXIT=0

$ cargo check --workspace --all-targets
    Finished `dev` profile [unoptimized + debuginfo] target(s)

$ cargo clippy --workspace --all-targets -- -D warnings
(no output, exit 0)

$ cargo test --workspace
passed: 136  failed: 0

$ cargo doc --workspace --no-deps
   Generated /home/raillen/Documentos/Projetos/aipo-lang/target/doc/aipo_ast/index.html and 14 other files
```

## Gate de saída do MVP

`docs/waves/wave-1-mvp.md` lista cinco condições de saída. Status de cada uma:

| Condição de saída | Status | Evidência |
|---|---|---|
| `aipo run` executa programas pequenos reais | **atendida** | snapshots de `programs/*`, casos de `modules/*` |
| `aipo check` reporta diagnósticos; `aipo fmt` idempotente | **atendida** | fixtures de `diagnostics/*`; testes golden e de idempotência do formatter |
| fixtures de pass/fail verdes; snapshots commitados; testes de integração verdes | **atendida** | suíte de conformidade 13/13 |
| Nenhum panic do Rust escapa como erro de usuário (fuzz smoke) | **atendida** | `fuzz_smoke` 3/3 |
| CI: fmt/clippy/test/doc verdes; `prumo validate` e `prumo doctor` verdes; delta de docs resolvido | **atendida** | gates acima; saídas do `prumo` abaixo |

```
$ prumo validate
Prumo validation passed.

$ prumo doctor
Prumo Doctor: all checks passed cleanly.
```

## Lacunas verificadas (o gate é parcial, e diz por quê)

O corpus encontrou quatro pontos em que o canon e a implementação ainda diferem. Exercitá-los é
o objetivo do corpus — o gate do S11 não os esconde. Cada um está registrado em
`docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md` com um programa mínimo de
demonstração, e cada um é carregado como critério de aceitação do `P00-G13`:

| Lacuna | Hoje | O canon exige |
|---|---|---|
| G1 `init` não invocado por `Type{...}` | a construção apenas preenche os campos nomeados | a construção executa `init` quando declarado |
| G2 `invariant()` não avaliado | instância publicada sem verificação | os invariantes valem na publicação |
| G3 contratos de assinatura não verificados em runtime | o valor cruza a fronteira sem verificação | verificação em runtime nas fronteiras |
| G4 `Bytes` não tem forma de construção | `Bytes([1,2,3])` levanta `AIPO_RT_NOT_CALLABLE` | construção e indexação estão no escopo |

Estas lacunas **não** são regressões e **não** são falhas dos critérios de aceitação deste goal: o
entregável do S11 é o corpus e o gate honesto, e ele entrega ambos. Elas são o motivo de o
gate ser registrado como **parcial** e não como **fechado**.

## Delta de documentação

| Artefato | Mudança |
|---|---|
| `docs/conformance/README.md` | novo — layout do corpus, invariantes das fixtures, matriz de snapshots, regeneração, rubric do gauntlet |
| `docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md` | novo — as quatro lacunas verificadas acima, com programas de demonstração |
| `docs/evidence/P00-G11-cli-and-formatter.md` | registra os entregáveis do S10 que este slice pontua |
| `docs/evidence/P00-G10-backend-completion.md` | registra as lacunas de backend que este slice exercita |
| `.ai/goals/P00-G13.goal.json` | goal sucessor para as quatro lacunas |
| `CHANGELOG.md`, `PROJECT_STATE.md` | slice S11 registrado, próxima ação definida como S12 |
