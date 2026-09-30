# Evidência — P01-G01 / Paridade JS para o Subset Completo do MVP

**Goal:** `P01-G01` — entregar o emitter `aipo-js` (ESM + shim versionado + source maps
ECMA-426) e `aipo build`, com uma suíte diferencial VM↔JS verde sobre o corpus
congelado da Wave 1
**Fase:** P01 (paridade JavaScript, Wave 2) · **Registrado em:** 2026-09-19
**Ambiente:** rustc 1.98.1, cargo 1.98.1, Linux, Node v24.18.0 (a suíte exige Node ≥ 20)

Anexos de prova dos gates exigidos. Os comandos são reproduzíveis a partir da raiz do repositório.
O plano da Wave 2 está em `docs/waves/wave-2-js-parity.md`; este registro é a pontuação, não uma
segunda especificação. Nenhuma mudança na superfície da linguagem neste goal.

## Por que este goal existiu

A Wave 1 fechou com um pipeline bytecode→VM certificado e uma lista explícita de não entregas encabeçada
pelo backend `aipo-js`. O canon (Fechamento Arquitetural §10) exige que o backend JS
preserve a semântica do Aipo — faixa de `Int`, NFC de `String`, `Dict` ordenado, separação `Failure`/fault —
e nunca acidentes do JS, tratando a divergência entre backends como bug. Este goal entrega
o menor slice que prova isso: o subset completo do MVP (`programs/01`–`20`) executando
de forma idêntica em ambos os backends.

## Implementação

| Área | Decisão |
|---|---|
| Entrada do emitter | Core IR neutro em relação ao alvo (`aipo-ir`); `aipo-js` depende apenas de `aipo-ir` + `aipo-diagnostics` (+ `serde_json` para codificação do bundle) — sem acoplamento com bytecode, conforme os contratos das crates |
| Geração de código | O bundle carrega o módulo como dado (`app.js` embute o JSON do Core IR e chama `runModule`); a semântica vive uma única vez, no shim versionado (`RUNTIME_VERSION = 1.0.0`, registrado em `app.js`, `aipo-js::RUNTIME_VERSION`, `RUNTIME_VERSION` do shim) |
| Núcleo do interpretador | Máquina de stack sobre ops do Core IR em `crates/aipo-js/runtime/aipo-runtime.js`: frames com `vars`/`cells`, alvos de salto absolutos, stack de handlers, journal de mutações por frame, stack de guardas de iteração, limite de stack de 1024 — espelhando os algoritmos de `aipo-vm` para que a paridade valha por construção |
| Resolução de escopo (defeito encontrado e corrigido durante o slice) | Corpos de closures endereçam capturas com `Load`/`Store` simples; o builder registra os nomes em `upvalues`, mas o emitter resolve por layout. A primeira versão lia cells apenas para `GetUpvalue` e quebrava a captura compartilhada de `var` (`programs/06`: contador preso em 0). Corrigido: `Load`/`Store` resolvem params/locals → cells de upvalue → globals, exatamente como o fallback `Slot` do emitter de bytecode |
| Globals de função | A execução JS pré-registra toda função sem ponto no nome como global, espelhando o prologue do bytecode (`MakeFunction`+`SetGlobal` por função); os bindings de escopo de módulo permanecem globals, de modo que funções aninhadas os resolvem por nome |
| Verificações de construção | `AssertInvariant` remove o `Bool` da stack e não empilha nada (a primeira versão empilhava `none`, que `SealStruct` então confundia com a instância — `programs/13` gerava fault). Coincide com a semântica de pop da VM |
| Paridade do texto de Failure | Toda mensagem recuperável coincide byte a byte com o código-fonte Rust (`invalid integer text: "…"`, `math.clamp bounds are inverted: …`, `missing format value for placeholder {…}`, …), porque `err.message` é saída observável do programa |
| Rigor de `Bytes` | `Bytes(x)` aceita apenas `Int` (sem widening de `Byte`), como `convert_bytes`; a mensagem de fora de faixa coincide (`outside the constructible range 0..=67108864`) |
| Source maps | Mapa ECMA-426 válido por bundle (`version: 3`, `sources: [<entry>.aipo]`, `sourcesContent`, mappings VLQ por linha); `app.js` termina com `sourceMappingURL=app.js.map` |
| Exit codes | `runModule` retorna `1` em fault/failure não capturada e `app.js` chama `process.exit(code)` (a primeira versão sempre saía com 0) |
| `aipo build` | Novo comando estável `aipo build <path> [--out <dir>] [--message-format=…]` que compartilha o frontend de `run`/`check` (parse → resolução de módulos → sema → IR). Falhas em tempo de check saem com 1, com códigos idênticos, e não emitem nada; faults de runtime compilam normalmente e aparecem sob `node` com códigos idênticos |

## Gate: paridade diferencial

```
$ cargo test -p aipo-js --test differential
test result: ok. 2 passed; 0 failed   # 20/20 programs: VM == JS == committed .stdout

$ cargo test -p aipo-cli --test js_build
test result: ok. 3 passed; 0 failed   # bundle+node snapshots; 19/19 diagnostics
                                      # (check-time rejected at build, runtime faults identical under node);
                                      # modules/basic bundled, cycle/missing rejected
```

## Gate: `fmt` / `clippy` / `test` / `doc` / `prumo`

```
$ cargo fmt --all -- --check
FMT_EXIT=0

$ cargo clippy --workspace --all-targets -- -D warnings
CLIPPY_EXIT=0 (warnings: 0)

$ cargo test --workspace
PASSED=152 FAILED=0   # was 146; +1 aipo-js emitter, +2 differential, +3 js_build

$ cargo doc --workspace --no-deps
DOC_EXIT=0 (warnings: 0)

$ prumo validate .
Prumo validation passed.

$ prumo doctor .
Prumo Doctor: all checks passed cleanly.
```

## Gate: `documentation_impact`

| Artefato | Mudança |
|---|---|
| `docs/waves/wave-2-js-parity.md` | novo — objetivo da Wave 2, arquitetura, subset de paridade congelado, slices, gate de saída |
| `docs/reference/cli.md` | `build` é estável desde a Wave 2, com layout do bundle, regra de paridade e requisito de Node |
| `docs/crates/crate-contracts.md` | `aipo-js` marcado como entregue pelo `P01-G01` (deps, layout do bundle, suítes diferenciais) |
| `docs/evidence/P01-G01-js-parity-mvp-subset.md` | novo — este registro de hand-off |
| `docs/PRUMO.md` | o índice de evidências ganha `P01-G01` |
| `CHANGELOG.md`, `PROJECT_STATE.md`, `prumo.json` | Wave 2 em execução sob `P01-G01` |

## Limitações conhecidas (registradas, não escondidas)

- **O shim interpreta dados do Core IR; não é um gerador de código otimizante.** A codegen JS
  por função, a minificação do bundle e as baselines de performance (workload JS do `aipo bench`) são
  explicitamente trabalho futuro — paridade primeiro, performance depois.
- **Casos de borda do Display de `f64` em mensagens.** Floats integrais são renderizados como `x.0` em ambos os backends;
  magnitudes extremas (`1e21`) podem ser formatadas de forma diferente entre toolchains. Nenhuma fixture do corpus
  cobre essa faixa; uma divergência ali viraria um ADP, não uma correção silenciosa.
- **Os diagnósticos estáticos de contrato permanecem apenas no frontend.** `AIPO_SEM_CONTRACT_VIOLATION_STATIC`
  é reportado pelo `build` em tempo de build (verificado pelos diagnósticos 15/16 na suíte de build);
  nunca é um evento de runtime do JS.
- **A não entrega restante da Wave 2+ permanece inalterada:** APIs de packing de `Bytes`, `Set`/`Sequence`,
  LSP/REPL, async, host ABI/Poppy, pacotes — ainda fora do escopo, ainda explícitos.
