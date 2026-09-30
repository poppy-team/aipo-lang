# Evidência — P01-G02 / Deep Quality Gauntlet

**Goal:** `P01-G02` — validação reproduzível de performance, segurança, conformidade,
acessibilidade e programas gerados, sem alterar a semântica canônica
da linguagem
**Fase:** P01 · **Registrado em:** 2026-09-20
**Ambiente:** Linux x86_64 (4 CPU, 5 GiB RAM), rustc/cargo 1.98.1, Node
v24.18.0, git a8709881 (trabalho commitado depois)

## Gates (todos executados, todos verdes)

```
$ cargo fmt --all -- --check                                   # 0
$ cargo check --workspace --all-targets                        # 0 (7.6s)
$ cargo clippy --workspace --all-targets --all-features -- -D warnings  # 0
$ cargo test --workspace --all-targets                         # 253 passed, 0 failed, 46 suites
$ cargo doc --workspace --no-deps                              # 0
$ prumo validate . && prumo doctor .                           # both clean
$ cargo +1.85 check --workspace --all-targets (MSRV)           # 0
$ cargo audit                                                  # 0 advisories / 31 deps
$ cargo deny check                                             # advisories+bans+licenses+sources ok
$ cargo +nightly miri test -p aipo-vm                          # 44 passed
```

## Delta do inventário de testes (165 → 253)

| Área | Adicionado |
|---|---|
| Testkit (`aipo-testkit`, novo) | Rng, descoberta de corpus, runners de pipeline/js, watchdog portável, timing, transformações metamórficas, gerador AipoSmith + testes de validade |
| Benchmarks (`aipo-bench`, novo) | 9 estágios de frontend × 4 tamanhos, 4 comandos, 16 workloads de VM, 2 workloads de JS, 3 análises de escalonamento; `docs/performance/baseline.md` |
| Propriedades | lexer (3), source (6), totalidade de tokens do syntax (2), modelo de valores (7), matriz de contratos (2), unicode da stdlib (10), equivalência do fmt (2) |
| Diferencial gerado | `generated.rs` (80 programas VM==JS), `metamorphic.rs` (4) |
| Segurança | `hostile.rs` (11), `resource.rs` (9), `unicode_security.rs` (8) |
| UI/acessibilidade | `diagnostic_ui.rs` (4 + 10 goldens), `suggestions.rs` (3) |
| Conformidade de backend | `sourcemap.rs` (4), `determinism.rs` (2), `properties.rs` (+coerência de VLQ), extensões do selftest do shim |
| Emitter/VM | testes da guarda de overflow (3), regressão de aridade (1), DictMap (5), testes de profundidade (3+2) |
| Exemplos | 19 novos exemplos + harness `examples.rs` (23 casos × VM+JS) |
| Fuzz | 3 targets libFuzzer (build verde); fuzz de gramática + execução portável em `fuzz_smoke` |

## Defeitos encontrados e corrigidos (todos com artefatos de regressão)

1. **Abort do host em aninhamento profundo (SIGABRT, exit 134)** — o parser
   recursive-descent estoura a stack do host acima de ~500–800 níveis de parênteses / ~300–600 níveis
   de bloco. Corrigido: guardas de profundidade de expressão/bloco + garantia de progresso +
   `AIPO_PARSE_NESTING_TOO_DEEP` (ADP-005). Comprovado por `resource.rs` em threads de teste
   de 2 MiB.
2. **Hang latente em loops de corpo** — qualquer falha de parse que não consome entrada faz os loops de
   recovery `while` girarem para sempre (encontrado ao corrigir o 1). Corrigido pela mesma garantia
   de progresso, comprovado por construção que não altera entradas que terminam + snapshots
   inalterados.
3. **Panic do IR builder em programas válidos** — construir uma struct com `init`
   dentro de uma função parametrizada causava panic (`hidden receiver local`,
   `builder.rs:415`). Encontrado pelo libFuzzer em ~2k execuções. Corrigido via
   `hidden_name`; regressão: `programs/23`.
4. **Divergência de journal VM↔JS na recuperação de `attempt`** — encontrada por análise manual
   durante a revisão do gauntlet, comprovada com um probe (VM `2` vs JS `1`), corrigida no
   shim; regressão: `programs/21`.
5. **Divergência de zero com sinal** — exibição de `-0.0` e zero negativo de `Int`
   (`-33 * 0 === -0` em JS, impossível em i64 do Rust). Encontrada pelo diferencial
   gerado (seed 1001). Corrigida no shim (`floatText`, `normInt`);
   regressão: `programs/22` + asserções do selftest.
6. **Parsing quadrático de f-string** — o padding `" ".repeat(offset)` por placeholder
   medido em 68× por 10× de entrada (23.4 ms para 400 f-strings). Corrigido por
   slices sem padding + deslocamento de spans (`shift.rs`): 1.77 ms, snapshots idênticos.
7. **Exemplos deteriorados** — `03` (`return` inválido em `invariant`), `04`
   (pipelines com `|` inicial, `fold` inexistente) falhavam na main atual. Corrigidos;
   o harness de exemplos impede a recorrência.
8. **`timeout` exclusivo de Unix no harness de fuzz** — substituído pelo portável
   `testkit::proc::run_bounded` (poll + `Child::kill`).

## ADPs abertos (ambiguidades documentadas, semântica inalterada)

- **ADP-003 (rascunho)** — orçamentos de execução: existe apenas o limite de profundidade da stack;
  fuel/memória/interrupção indecididos. A linha "Owns fuel" do contrato de crate foi corrigida.
- **ADP-004 (rascunho)** — política de identificadores/segurança Unicode: identificadores decompostos
  rejeitados, confusáveis irrestritos, NBSP rejeitado; tudo fixado por
  `unicode_security.rs`, nada alterado.
- **ADP-005 (aceito)** — limites de recursão do parser (128/64/128) com análise de
  margem, novo código de diagnóstico, controle de cascata.

## Cobertura (LLVM, `docs/testing/coverage.sh`)

- Totais: 52.4% de linhas / 38.5% de funções / 46.2% de regiões (inclui código de teste,
  o runner de bench em 0% por design, e dependências de terceiros).
- Destaques do src do workspace: lexer 96/100, hir 94/100, formatter 94/97, js
  94/80, cli 92/86, sema 89/93, syntax 82/73, vm 84/82, stdlib 84/95.
- Os arquivos com menor cobertura motivaram testes direcionados: Display de erros de runtime/source, spans
  exóticos de placeholder. Os itens de navegação restantes (ramos de erro de math/prelude,
  goldens do disassembler, ramos de fault de valor) estão listados em
  `docs/testing/gauntlet-gap-matrix.md` — sem meta de 100% (comportamento primeiro).

## Relatório de fuzz

| Target | Orçamento | Execs | Crashes | Resultado |
|---|---|---|---|---|
| `lexer_tokens` (libFuzzer) | 60 s + 22 seeds | 115,186 | 0 | limpo |
| `frontend_check` (libFuzzer) | ~90 s + 22 seeds + replay de crash | 65,801 | 1 (panic de IR → corrigido, programa 23) | 1 corrigido |
| `formatter` (libFuzzer) | 120 s + 22 seeds | 215,209 | 0 | limpo |
| Smoke determinístico (`fuzz_smoke`, 5 testes) | seeds fixas | — | 0 | limpo |

## Relatório de mutação (verificação pontual manual, `cargo-mutants` adiado)

| Mutante | Resultado |
|---|---|
| Aridade `==` → `>=` (call.rs) | **Sobreviveu a tudo** → escrito `test_call_with_extra_arguments_faults_on_arity` → morto → revertido |
| Faixa de safe-int `..=` → `..` (value.rs) | Morto por `value_properties` → revertido |
| Limite de profundidade de bloco 64 → 1M (parser.rs) | Morto (o teste de recurso aborta = a guarda é load-bearing) → revertido |

## Achados de segurança

- Proteção verificada: nenhum `unsafe` no workspace (`forbid`); `unsafe` transitivo
  inventariado (memchr/serde/byteorder/itoa/syn/unicode-normalization — todos
  mainstream, zero advisories); audit verde; deny verde (nova política em
  `deny.toml`); bytecode hostil rejeitado sem panic (11 testes); entrada profunda
  termina com um único diagnóstico; loop/recursão infinitos limitados externamente
  (o ADP-003 registra honestamente a ausência de orçamento dentro da VM).
- Lacuna verificada: fuel/memória/interrupção dentro da VM (ADP-003); política de
  identificadores (ADP-004).
- Risco futuro: fronteiras nativas/FFI (nenhuma existe) → sanitizers então.

## Relatório diferencial VM ↔ JS

- Escritos à mão: 23 programas + 19 diagnósticos + 3 casos de módulos + 23 exemplos.
- Gerados: 80 programas AipoSmith + 10 casos metamórficos, todos VM==JS.
- Divergências encontradas: 3 (recuperação de journal, zero com sinal ×2) — todas corrigidas com
  fixtures. Restantes: nenhuma.

## Relatório de acessibilidade dos diagnósticos

- 10 goldens de UI (humano + JSONL) fixam código/severidade/mensagem/span/notas/
  sugestões/exit/disciplina de stdout; zero bytes ANSI verificados em todos os lugares;
  cascata limitada (aninhamento = exatamente 1 erro); erros de módulo sem span cobertos.
- A mecânica de sugestões foi comprovada em diagnósticos construídos (não existe produtor —
  registrado, não inventado).
- Rubric (`diagnostic-accessibility-rubric.md`) + protocolo manual
  (`cognitive-accessibility-protocol.md`, 10 erros) publicados; sessões ainda
  não executadas (residual).

## Catálogo de exemplos

23 exemplos (01–05 pré-existentes, 06–24 novos), cada um com `.stdout` commitado,
cada um verificado na VM e no Node por `examples.rs`. A tabela detalhada por exemplo está no
relatório final (§K).

## Decisões de tooling

Veja `docs/testing/tooling-decisions.md` (adotados: bench runner, cobertura LLVM,
cargo-fuzz, audit, deny, Miri, testkit; rejeitados: Criterion, proptest, AFL++;
adiados: mutants, nextest, sanitizers, Iai). MSRV 1.85 verificado.

## Riscos residuais (explícitos)

- Os orçamentos de fuzz foram de minutos, não de dias; execuções mais profundas do libFuzzer ficam
  no tier nightly.
- As baselines de wall-clock têm ruído de runners compartilhados (MAD publicado junto).
- As sessões do protocolo cognitivo ainda não foram executadas.
- `publish = false` adicionado a todas as crates (motivado pelo deny); publicar no crates.io
  exige um passe de release (versões, versões, changelogs por crate).
- As contagens históricas em arquivos de evidência antigos são históricas por política; as contagens
  atuais estão neste registro.

## Impacto na documentação

Novos: `aipo-testkit`, `aipo-bench`, `fuzz/` (+README, targets, seeds),
`deny.toml`, `docs/performance/baseline.md`, `docs/testing/{gap-matrix,
tooling-decisions, ci-tiers, concurrency-audit, rubric, protocol,
coverage.sh}`, ADP-003/004/005, `examples/06–24` + snapshots, programas
21–23, goldens de UI, este registro. Atualizados: testing-strategy, crate-contracts
(Owns de vm), catálogo (`AIPO_PARSE_NESTING_TOO_DEEP`), README de conformidade
(23/23), URL do repositório em `Cargo.toml`, CHANGELOG (abaixo), PROJECT_STATE,
PRUMO.md.
Contradições encontradas: URL de repositório obsoleta (corrigida), afirmação de fuel da vm (corrigida),
menções a gc-arena (corrigidas antes), link do Prumo no README (adicionado antes).
