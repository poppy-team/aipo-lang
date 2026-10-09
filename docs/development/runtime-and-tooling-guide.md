# Runtime, embedding e ferramentas: uso e reimplementação

**Rodada:** P07-G02 · **Data:** 2026-10-09 · **Base:** `f0a0d70d176be031cd4b600fde1df6179429fb9a`.
**Status desta publicação:** documentação de recuperação; implementação local ainda não publicada.

> O ambiente de desenvolvimento desconectou com `409 environment_offline` antes de enviar o código. Este commit preserva o guia e o registro de trabalho; **não contém a implementação descrita abaixo**. Os comandos/APIs novos só estarão disponíveis depois da recuperação e publicação do checkout. O objetivo permanece DRAFT. Testes não foram executados, por instrução explícita do usuário. Não usar esta PR como prova de implementação concluída ou autorização de merge.

Este guia registra o que foi implementado no checkout local, os motivos, como usar/reimplementar e os limites conhecidos. O [registro de recuperação](../evidence/P07-G02/README.md) distingue checagens anteriores, o último snapshot compilado e o trabalho ainda necessário. O [guia anterior](runtime-hardening-guide.md) documenta a P07-G01 já presente na base.

## 1. As cinco frentes e seu estado

| Frente | Trabalho realizado no checkout | Limite/pendência |
|---|---|---|
| 1. Correção e segurança | RegVerifier para operandos, CFG, inicialização e escopos; revisão estrutural de coleções; prova semântica conservadora para `is` Wasm | Revisão adicional de aliases/shadowing do tipo à direita no Wasm; testes e fuzz não executados |
| 2. Cobertura e embedding | ExecutionPlan explícito Reg/canônico; linker; budgets Reg/C; begin/pump/abort C; limites e cache Wasm | Código não enviado; Reg nativo continua subconjunto; callbacks externos não são limitados pelo budget guest |
| 3. Sessão, shell e perfis | REPL persistente, multilinha/histórico/completion; recarga com migração/rollback; scheduler preservado entre unidades; aliases de features | Novo scheduler compilado com Clippy/Rust 1.99, ainda sem repetição final do check 1.96; nenhum porte MCU |
| 4. Performance | Janelas vivas, corpos de função Rc e constantes preparadas por módulo; driver de benchmark pareado | Dados locais não recuperados; medição final do último snapshot ficou pendente |
| 5. Ferramentas/distribuição | new/watch/profile/debug/plan, LSP básico, vendor offline, empacotador/instalador e workflows | Código e scripts ainda não enviados; workflows não executados; nenhuma release publicada |

O plano Reg fornece a superfície compartilhada por seleção explícita da VM canônica. Isso não significa paridade nativa de closures, async, interfaces, hooks, invariantes e journal no RegVM.

## 2. Build e perfis previstos na implementação

O checkout elevou o MSRV de Rust 1.85 para **1.96**, porque o lock existente inclui Wasmtime 49.0.2 e Cranelift 0.136.2, que declaram 1.96. Não se alterou a sintaxe da linguagem nem se criaram crates por perfil. A atualização ativou simplificações exigidas pelo Clippy; os lints foram preservados.

```sh
rustup toolchain install 1.96.0 --profile minimal

# Completo, incluindo Wasmtime
cargo +1.96.0 build --locked -p aipo-cli --release

# Shell com formatter, regex e Unicode extra, sem JIT
cargo +1.96.0 build --locked -p aipo-cli --release \
    --no-default-features --features profile-shell

# Emissores JS/Wasm, sem Wasmtime
cargo +1.96.0 build --locked -p aipo-cli --release \
    --no-default-features --features profile-web

# CLI mínima nativa, ainda com std
cargo +1.96.0 build --locked -p aipo-cli --profile nano \
    --no-default-features --features profile-embedded
```

Esses aliases ainda não estão neste commit de recuperação. `--no-default-features` é necessário: acrescentar uma feature não remove defaults. `nano` é perfil Cargo de size optimization/LTO/strip/panic abort; não é `no_std`. A CLI mínima ainda tem serviços de SO da stdlib. Normalização fundamental segue o canon; `unicode` habilita operações adicionais.

Não há promessa de Value com 16 bytes, RAM mínima ou startup em milissegundos. O exemplo existente `layout_probe` e o alvo real devem produzir essas medidas.

## 3. ExecutionPlan e verificação Reg

Uso depois da publicação do código:

```sh
aipo run src/main.aipo
aipo run src/main.aipo --engine=reg
aipo plan src/main.aipo
```

`plan` compila sem executar e informa JSON com `engine` e `reason`. Reg nativo só é selecionado quando todo o IR é suportado e verificado. Construção/acesso a membros seleciona serviços canônicos compartilhados; emitter sem uma capacidade gera motivo explícito para seleção canônica. Nenhum IR é descartado para simular suporte.

A API `analyze_to_reg_module` permanece estrita. `analyze_to_execution_plan` devolve enum Register ou Canonical. Host callbacks dependentes do contexto Vm, como o headless test host, selecionam a VM mesmo no perfil Reg.

### Algoritmo do verifier

O novo `aipo-bytecode/src/reg_verifier.rs` implementa:

1. Inspeção de todos os operandos, inclusive unreachable: registradores, janelas, pools, constantes String exigidas, referências de função, targets e projeções de iteração. Pools numéricos devem respeitar Int seguro e Float finito.
2. Análise do CFG com 256 fatos booleanos de inicialização, stack de handlers e profundidade de guards. Parâmetros começam inicializados; joins usam interseção. Toda leitura deve estar inicializada em todos os caminhos.
3. Aresta excepcional conservadora na instalação do handler, com registrador de erro inicializado e escopo anterior preservado. Underflow/inconsistência de scopes e fallthrough alcançável sem Return são erros.
4. Verificação nas fronteiras emitter, run_module, run_function, invoke_function e raw run, antes de efeitos guest. A entrada raw trata registradores do host como inicializados, inclusive quando contêm none.
5. Reutilização da prova somente durante o módulo correspondente, protegida por RAII inclusive durante unwind.

Para reimplementar um opcode, atualize formato, emitter, verifier, CFG, runtime e conformance. A palavra Reg é u32: opcode 7 bits, A 8, B 9, C 8; Bx/sBx usa 17 bits, sBx com bias 65536. Registradores continuam limitados a 0–255. B pode representar índice/imediato conforme opcode; não o trate sempre como registrador.

## 4. Coleções com revisão estrutural

Comparar apenas comprimento perdia mutação remove+insert em uma fronteira de callback. `Collection<T>` acrescenta revisão estrutural para List, Set e Bytes; DictMap possui revisão equivalente.

| Operação | Revisão |
|---|---|
| Inserção, remoção, clear e reorganização estrutural | Incrementa |
| Substituição de elemento existente ou valor de chave existente | Preserva |
| Igualdade com mesmos elementos/ordem, mesmo após mutações | Ignora revisão |

Guards guardam identidade e revisão. Retornos, handlers e scheduler conservam as profundidades corretas. Métodos de ordem superior verificam inclusive a volta de callback nativo. Substituição de elemento continua permitida pelo canon.

**Migração Rust planejada no código local:** payloads List/Set/Bytes tornam-se `Rc<RefCell<Collection<_>>>`, não `Rc<RefCell<Vec<_>>>`. Use `.into()` na construção manual ou os construtores:

```rust
use aipo_vm::Value;
let items = Value::list(vec![Value::Int(1), Value::Int(2)]);
let bytes = Value::bytes(vec![1, 2, 3]);
let text = Value::string("Aipo");
```

`Value::set` pressupõe elementos deduplicados. O acesso mutável é por slice, sem DerefMut de Vec que permita resize fora da revisão. Hosts devem usar métodos estruturais de Collection. A tabela de funções Reg passa a `Vec<Rc<RegCompiledFunction>>`.

## 5. Sessão persistente e linker

A Session local guarda Vm, imagem BytecodeModule ligada, catálogo HIR, mutabilidade, ownership por arquivo e geração. Cada unidade compila statements novos; declarações anteriores compõem o catálogo. O histórico não é reexecutado para reconstruir estado.

```aipo
var count = 0
fn next() {
    count += 1
    return count
}
next()
next()
```

A última correção preserva Tasks, resultados, filas, joins, grupos, ids e tick entre unidades. Uma tarefa vinculada em uma unidade pode ser aguardada em outra. O snapshot captura também scheduler, resultados e células de upvalue retidas por tasks. `run_persistent_at` exige imagem append-only com offsets antigos válidos; `run_at` conserva a entrada de execução nova independente.

### Linker

`append_unit` verifica a unidade, trabalha em candidata clonada, deduplica nomes/constantes, reloca referências de funções/closures/contratos e desloca entradas/spans. Saltos relativos não mudam. Limites do formato e imagem ligada são verificados antes do commit. `function_by_name` busca a definição mais recente.

Índices publicados não podem ser compactados enquanto Function/Closure/Task ainda referencia código antigo. Imagem, tasks e resultados podem crescer em sessões longas; coleta de unidades/tasks e ciclos Rc permanece futura. Deduplicação linear pode ser substituída por índices de interning após medição.

### REPL

| Comando | Uso |
|---|---|
| `:help` | Ajuda |
| `:history` | Unidades anteriores |
| `:complete PREFIX` | Globals com prefixo |
| `:load PATH` | Avalia unidade |
| `:reload PATH` | Substitui declarações do arquivo e preserva estado compatível |
| `:reset` | Limpa sessão |
| `:quit` | Encerra |

Balanceamento usa lexer, ignorando comentários/strings. Entrada redirecionada não emite prompts e informa falhas pelo exit code. REPL com `--engine=reg` identifica o uso da sessão canônica.

Histórico JSONL: até 1.000 unidades; leitura limitada a 8 MiB, arquivo maior ignorado. Caminho XDG_STATE_HOME ou HOME/.local/state seguido de aipo/history.jsonl. AIPO_HISTORY_FILE troca caminho; vazio desliga. Escrita usa temporário, sync e rename, com 0600 em Unix. Editor de linha com setas/Tab ainda é evolução futura.

## 6. Reload, rollback e migração

`aipo watch src/main.aipo` acompanha a árvore do manifest mais próximo ou da entrada sem manifest. Hashes de conteúdo, polling 250 ms; extensões aipo/toml/lock/ahs; ignora symlinks/.git/target/node_modules; limites de 64 níveis e 10.000 arquivos. Imports de caminho fora da árvore exigem watcher adicional/recarga explícita.

`Session::reload_with`:

1. Captura definições, scheduler e heap alcançável.
2. Remove catálogo público do arquivo antigo; compila, liga e inicializa candidata.
3. Preserva globals não chamáveis existentes que continuam declarados.
4. Executa migração fornecida pelo host e valida campos, ordem e flags fixed das instâncias retidas.
5. Confirma geração/caches ou restaura globals, código, catálogo, ownership, scheduler e heap.

```rust
use aipo_cli::Session;
use std::path::Path;
let mut session = Session::new();
session.set_instruction_budget(Some(100_000));
let status = session.reload_with(
    "var count = 0\n", Path::new("counter.aipo"),
    &mut std::io::stdout(), &mut std::io::stderr(),
    |_globals| Ok(()),
);
```

HeapSnapshot percorre coleções, Dict, Bytes, structs, receivers, Failure.payload, upvalue cells e dados/caches de Sequence. Restaura as próprias alocações Rc, preservando aliases mantidos pelo host. Clonar somente o mapa de globals não basta. Captura adicional de células/results do scheduler cobre tarefas retidas.

Tipo removido com instância viva exige migração. Inicializador pode mutar estado anterior; em falha isso é revertido, em sucesso faz parte do estado confirmado. Snapshot não é GC: ciclos Rc e retenção de tarefas/código continuam problemas de lifecycle.

**Limite externo:** arquivos, rede, processos, callbacks e recursos opacos do host não são revertidos. O host precisa coordenar suas próprias transações. Os streams out/err da Session carregam diagnósticos; o sink io da stdlib tem seu contrato próprio e não deve ser anunciado como isolamento automático por Session.

## 7. Embedding Rust/C

Runtime pertence a uma thread: Value usa Rc/RefCell e não é Send. Reentrada durante callback C é recusada. Pointers C continuam exigindo lifetime, tamanho e alinhamento válidos; checar null não valida um pointer arbitrário.

### Budgets

RegVm configura orçamento cumulativo por `set_instruction_budget(Some(n))`, reiniciando consumo; None desliga contagem. VM canônica preserva consumo em entradas budgetadas e oferece reset explícito. Session reinicia por avaliação. Opcode budget não limita memória total, duração de nativo nem wall-clock.

### C cooperativo

O runtime liga módulos e soma offsets corretos para chamar funções de unidades anteriores. Carga com erro guest recuperável restaura definições/heap/scheduler; instâncias com layout incompatível são rejeitadas.

```c
bool completed = false;
aipo_value_t result;
aipo_status_t status = aipo_runtime_begin(rt, "demo", "main", NULL, 0);
while (status == AIPO_OK && !completed) {
    status = aipo_runtime_pump(rt, 1000, &completed, &result);
    /* Retorne ao event loop entre pumps. */
}
if (status == AIPO_OK && completed) {
    aipo_value_release(rt, result);
}
```

Begin prepara função como computação principal cooperativa, exige argumentos compatíveis e ausência de captures na entrada selecionada. Pump limita quanta; callback ainda pode bloquear. O resultado só vale quando completed é true. Load/call/begin recusam runtime ocupado. Abort cancela a computação e conserva mutações/I/O concluídos, sem rollback. Falha recuperável e cancelamento continuam distintos.

Reg raw ganha budget/count/reset e copy_last_error. O copy devolve tamanho necessário incluindo NUL; buffer nulo com capacidade zero consulta tamanho; buffer curto recebe cópia truncada terminada. Int fora da faixa segura ou retorno diferente de Int quando out_int é pedido gera erro, sem conversão silenciosa para zero.

String/Bytes C são snapshots do runtime, liberados por aipo_value_release, nunca free. Não reter depois de destruir runtime. Unwind panic é contido; panic=abort não pode ser recuperado.

## 8. Wasm

O código local deixou de aceitar `is` por tipo físico de armazenamento: i32 pode ser Bool/pointer; i64 não prova Int. None corresponde ao nullable; tipos fundamentais/nominais conhecidos usam prova semântica. Parâmetros/joins/calls sem prova são rejeitados com orientação para usar VM.

**Revisão ainda necessária antes de publicar:** RHS aliases/shadowing podem exigir rejeição explícita ou fatos de tipo. A análise conservadora do lado esquerdo não resolve sozinha um tipo de destino dinâmico. Completar esses casos e sua conformance antes de certificar a mudança.

WasmExecutionOptions acrescenta fuel, memory_bytes por memória linear e output_bytes no buffer. Excesso de saída é rejeitado antes de crescer buffer; grow excessivo gera trap. Limites não representam toda a memória do processo JIT nem trabalho externo.

WasmRunner reutiliza Engine e cache LRU limitado por número de módulos; Store independente por execução conserva isolamento de memória/globals/fuel/saída. Reduzir capacidade remove módulos menos recentes; zero desliga cache. Helper de conveniência continua disponível. Não houve mudança para WAMR/wasmi nem certificação de MCU.

## 9. Ferramentas

| Comando após publicação do código | Resultado |
|---|---|
| `aipo new demo` | Manifest, src/main.aipo e README sem sobrescrever diretório |
| `aipo plan src/main.aipo` | Plano/razão em JSON, sem executar |
| `aipo profile src/main.aipo --budget 100000 --json` | Tempo, instructions, calls, native calls e field cache |
| `aipo debug src/main.aipo` | step/continue/break BYTE_OFFSET/globals/quit |
| `aipo watch src/main.aipo` | Candidata com última geração válida preservada |
| `aipo lsp` | LSP sobre stdin/stdout |

Debugger usa byte offsets e quantum do scheduler; pode alternar tasks. Falha não capturada não resulta em sucesso. DAP, stepping por linha, avaliação no frame e inspeção lexical completa permanecem futuros.

LSP: initialize/shutdown/exit, full sync com versões, diagnósticos UTF-16, completion de keywords/prelude/declarações e formatação opcional. Analisa documento aberto e usa resolução de pacote. Diag de outra fonte é contextualizado por caminho/linha, sem inventar offsets do documento atual. Headers limitados a 8 KiB e mensagens a 8 MiB; JSON inválido/request inválido são distintos. Métodos ausentes têm erro explícito.

Não há ainda hover, rename, definitions, semantic tokens, sync incremental ou cancelamento real de trabalho em andamento. Stdout deve conter somente frames do protocolo.

## 10. Vendor e distribuição

```sh
aipo package lock caminho/pacote
aipo package vendor caminho/pacote --out vendor-demo
aipo package vendor caminho/pacote --cache caminho/cache --out vendor-demo
aipo run vendor-demo/root/src/main.aipo
```

Vendor exige lock existente e grafo verificável, sem buscar rede. Copia root/dependências, recusa symlinks, reescreve sources para paths locais e produz lock local. Manifest de exportação conserva lock original e inventário SHA-256. Ignora arquivos ocultos/target/node_modules; assets nesses locais precisam de empacotamento explícito. Diretório novo só é publicado depois do stage completo. Registry, publicação e ranges SemVer continuam fora desse resolvedor exato/pinado.

Empacotador local: builds full/shell/web/embedded/nano em checkout limpo; arquivo tar.gz, SHA-256, licença e build.json com commit/target/compiler/perfil. Full inclui header/libs C. Ordem/timestamps são determinísticos para os mesmos payloads, com SOURCE_DATE_EPOCH, sem prometer binário idêntico entre máquinas.

Instalador: checksum antes da extração; paths seguros, uma raiz, somente arquivos regulares; rejeita duplicatas, traversal e symlinks. Prefixo explícito; força exigida para substituir arquivos. Prepara/backup/replace por arquivo e rollback em exceção. Não baixa nem executa scripts; não é transação contra crash do SO entre várias substituições.

Workflows definidos no checkout: matriz compile-only Linux/macOS/Windows × Rust mínimo/stable × shell/web/embedded, mais packaging manual por perfil/SO. Não foram executados nem enviados. Não houve release pública.

## 11. Performance e reimplementação

O trabalho Reg substitui cópia de 256 slots por move/restore da janela viva; corpos ficam Rc e constantes guest são preparadas por módulo. Chamadas internas reutilizam a prova correspondente, entradas externas verificam. Raw host conserva janela total. Depth limit permanece. Não foi implementado NaN boxing, arena gerenciadora completa ou threaded dispatch.

Benchmark local: base pinada versus candidata, 20 mil calls, caller de nove instruções e um registrador no callee, casos Int/String usados. Driver alterna builds, fixa CPU opcional, remove warmup, usa três lotes de cinco pares e registra hashes/compiler. Medição final do snapshot com scheduler preservado não foi concluída; CSV locais não foram recuperados. Não extrapolar números preliminares para linguagem/startup/RSS/MCU.

Reimplemente na ordem: verifier/coleções → linker/snapshot/scheduler → Session/reload → embedding → ferramentas → distribuição. Preserve diagnósticos, identidade, efeitos, ownership e scopes de unwind. Reutilize Value e serviços canônicos, sem fork semântico por perfil.

## 12. Próximos passos e critérios

| Prioridade | Trabalho | Critério |
|---|---|---|
| P0 | Recuperar checkout/patch e publicar código; fechar RHS Wasm; repetir checks finais | Conteúdo completo e comparável no GitHub, tree/commit verificados |
| P0 para certificar | Executar conformance/fuzz/Miri/ASan e matriz quando autorizado | Resultados reais, não inferidos de cargo check; respeitar instrução atual de não executar testes |
| P1 | Reg nativo captures/async/hooks via serviços compartilhados | Diferencial VM e erro fechado por capacidade |
| P1 | GC de ciclos e retenção de tasks/unidades antigas | Lifecycle/identidade definidos; memória de sessão longa medida |
| P1 | Recursos externos em reload e imports fora da árvore | Hooks transacionais do host e watcher completo |
| P2 | LSP/DAP/editor de linha completos | Scopes, locations, stepping/cancelamento reais |
| P2 | Heap/compilação/deadline de nativos | Limites do allocator/host separados de opcode budget |
| P2 | Registry/publicação/ranges SemVer | Política própria, não solver exato tratado como ranges |
| P3 | no_std/allocator/HAL/XIP | Porte e medidas em alvo concreto |
| P3 | Mais otimizações e engine Wasm alternativa | Baseline controlada, paridade e evidência reproduzível |

Casos de regressão foram escritos/compilados localmente para RegVerifier, aliases/rollback, Session/tasks e provas Wasm. Fixtures 35/36 foram criadas com stdout esperado do canon, sem regenerar ou executar. Não marcar goal DONE, não declarar gate de testes aprovado e não fazer merge desta recuperação como se fosse a implementação.
