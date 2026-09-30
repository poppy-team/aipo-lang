# Evidência — P00-G14 / Imposição de Contratos em Runtime

**Goal:** `P00-G14` — `invariant()` nas fronteiras de mutação e contratos de assinatura em runtime
**Fase:** P00 (Foundation) · **Registrado em:** 2026-09-15
**Ambiente:** rustc 1.98.1 (48a229cea 2026-09-01), cargo 1.98.1 (797e8a9bc 2026-08-05), Linux

Anexos de prova dos gates exigidos. Os comandos são reproduzíveis a partir da raiz do repositório.

## Por que este goal existiu

O `P00-G13` fechou três das cinco lacunas encontradas pelo corpus do S11 e deixou duas, porque ambas exigiam um
mecanismo de runtime que ainda não existia: revalidar `invariant()` após uma mutação e verificar
contratos de assinatura na fronteira da chamada. Ambas são critérios do gate de saída do MVP em
`docs/waves/wave-1-mvp.md`, portanto o gate não podia ser considerado completo antes delas.

| Lacuna | Fonte da decisão no canon | Resultado |
|---|---|---|
| G2b — `invariant()` não reavaliado após uma mutação | Sintaxe Canônica: `candidate → provisional apply → verify → commit`; Interlúdio §3: validado em fronteiras mutáveis estáveis, e uma validação que falha produz uma `Failure` | **fechada** |
| G3 — contratos de assinatura não verificados em runtime | Language Reference §4: `name: Type`, `name!: Type`, `-> T`, `T?`; uma violação encontrada apenas em runtime é um **fault de contrato** | **fechada** |

Os dois canais de erro são diferentes, e o canon diz isso explicitamente: uma falha de invariante em uma
fronteira de mutação é uma `Failure` recuperável ("a operação produz uma `Failure`"), enquanto uma violação de
contrato descoberta em runtime é um fault de programação que `attempt` não consegue capturar. Nenhuma semântica
precisou ser inventada para nenhum dos dois.

## Implementação

| Estágio | Mudança |
|---|---|
| `aipo-ir` | Novos `CoreInst::AssertContract { type_name, nullable, position }` e `CoreInst::CheckMutations`; `FnCtx` carrega o contrato de retorno da função e o span da última atribuição de campo do frame |
| `aipo-bytecode` | Novos opcodes `AssertContract` (nome u16 + nullable u8 + posição u16) e `CheckMutations`, ligados ao emitter, ao verifier (verificações de limites do índice de nomes, verificação da flag nullable) e ao disassembler |
| `aipo-vm` | `VmFault::ContractViolation`; um journal de mutações por frame (`MutationEntry`) com os valores de entrada; `SetField` aplica provisoriamente e registra no journal; `CheckMutations` verifica cada instância participante, faz rollback de todas elas e levanta uma `Failure` recuperável; `Vm::run` resolve cada entry point `Type.invariant` a partir da tabela de funções do módulo; `CallFrame.journal_start` delimita o journal por frame |
| `aipo-stdlib` / `aipo-vm` | `TypeTag::from_name`, para que um nome de contrato escrito seja mapeado de volta à categoria core que o runtime já testa |

### Onde os contratos são verificados

- **Parâmetros**: no prologue do próprio callee, depois do prologue de defaults, de modo que um valor default
  também seja verificado e `init`, métodos, funções simples e closures recebam a verificação em um único lugar.
- **Retornos**: imediatamente antes de cada `return expr`, sobre o valor que já está na stack. A
  instrução inspeciona o topo da stack sem consumi-lo, e é por isso que um único opcode serve aos dois casos.
- **Invariante na mutação**: ao final da operação mutável que a envolve (todo return de função/método,
  incluindo o epílogo implícito) e a cada statement de mutação do script de entrada do módulo, que
  não tem operação envolvente à qual adiar. O canon proíbe validar após cada atribuição
  interna, então a atribuição em si apenas registra no journal.

## Gate: `fmt` / `clippy`

```
$ cargo fmt --all -- --check
FMT=0

$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s)
CLIPPY_EXIT=0
```

## Gate: `test`

```
$ cargo test --workspace
passed: 141  failed: 0
```

Nova certificação:

| Fixture | Comprova |
|---|---|
| `docs/conformance/programs/14_signature_contracts.aipo` | parâmetro `Int`, default verificado, `T?` aceitando `none`, o contrato `Function`, um contrato de `struct`, um receiver mutável `p!: Point` e um retorno `-> Int` |
| `docs/conformance/diagnostics/12_runtime_contract_violation.aipo` | uma violação de contrato de parâmetro é um fault (`AIPO_RT_TYPE_MISMATCH`) |
| `docs/conformance/diagnostics/13_runtime_return_contract.aipo` | uma violação de contrato de retorno é um fault |
| `docs/conformance/programs/15_invariant_on_mutation.aipo` | commit na fronteira de um método, rollback com o valor de entrada preservado em uma falha de método, o mesmo para uma atribuição direta no script de entrada, e rollback de duas instâncias participantes em uma única operação |
| `docs/conformance/diagnostics/14_runtime_invariant_mutation_uncaught.aipo` | uma falha de invariante de mutação não tratada encerra o programa como uma `Failure` recuperável (`AIPO_RT_FAILURE_UNCAUGHT`) |
| `crates/aipo-vm/tests/data_and_errors.rs::test_guarded_mutation_rolls_back_at_boundary` | o journal, a verificação na fronteira e o rollback na camada da VM, com um validator do host |
| `crates/aipo-vm/tests/data_and_errors.rs::test_signature_contract_violation_is_a_fault` | `AssertContract` levanta `VmFault::ContractViolation`, e não uma `Failure` |
| `crates/aipo-vm/tests/data_and_errors.rs::test_nullable_contract_accepts_none` | `T?` aceita `none` |
| `crates/aipo-bytecode/src/lib.rs::test_compile_verify_and_disassemble_contracts_and_mutation_boundaries` | `AssertContract`, `CheckMutations` e `AssertInvariant` sobrevivem ao round trip emitter → verifier → disassembler |

## Gate: `doc`

```
$ cargo doc --workspace --no-deps
(no warnings)
```

## Gate: `documentation_impact`

- `docs/adp/ADP-002-construction-hooks-and-runtime-contracts.md` — G2b e G3 marcadas como resolvidas, com
  as decisões registradas, incluindo a nota de desvio sobre *onde* a verificação de mutação é executada.
- `docs/conformance/README.md` — matriz de snapshots e inventário de fixtures estendidos.
- `docs/stdlib/mvp-subset.md` — contratos de assinatura e fronteiras de mutação de invariantes documentados como
  comportamento implementado.
- `CHANGELOG.md`, `PROJECT_STATE.md`, `docs/PRUMO.md`.

## Limitações conhecidas (registradas, não escondidas)

- **Contratos de interface não são verificados em runtime.** Um contrato que nomeia uma interface é aceito,
  porque o MVP não tem verificação estrutural de conformidade em runtime; inventar uma falha para um contrato
  que o runtime não consegue avaliar seria pior do que a verificação ausente.
- **As verificações de contrato executam no prologue do callee, não no ponto de chamada.** O comportamento observável é
  o mesmo (um fault na fronteira, com o nome do parâmetro na mensagem), mas uma incompatibilidade conhecida
  estaticamente ainda é reportada em runtime, e não antes da execução; `aipo-sema` ainda não usa as
  anotações.
- **Os corpos de `init` ainda usam o canal de fault de construção.** Um invariante violado durante
  a construção é `VmFault::InvariantViolation` (nada pode ser publicado), enquanto um invariante
  violado por uma mutação posterior é uma `Failure` recuperável (existe um estado de entrada a restaurar).
  Ambos vêm do canon, de parágrafos diferentes.
- **Uma `Failure` que escapa de uma operação antes de sua fronteira mantém o valor aplicado.** O canon só
  exige rollback quando a *validação* falha, portanto as entradas são descartadas sem restauração quando uma
  falha não relacionada desenrola o frame.
