# Matriz de Conformance da Linguagem

A suíte de conformidade (*conformance suite*) do Aipo é a garantia de que as decisões semânticas da especificação são rigorosamente obedecidas em todas as execuções.

---

## Estrutura da Suíte de Conformance

A suíte reside em `docs/conformance/` e é dividida em três pilares:

### 1. Programas Canônicos (`docs/conformance/programs/`)
Mais de 28 programas completos cobrindo:
- `01_hello.aipo`: Inicialização, strings e saída padrão.
- `02_recursion.aipo`: Profundidade de chamadas e isolamento de frames.
- `03_control_flow.aipo`: Condicionais `if/else`, laços `while`, `loop` e `repeat`.
- `04_collections.aipo`: Operações em listas e dicionários.
- `05_structs_and_impl.aipo`: Declaração de estruturas e métodos associados.
- `06_closures.aipo`: Captura léxica e upvalues compartilhados.
- `08_failures.aipo`: Disparo de falhas com `fail`.
- `12_bytes.aipo`: Manipulação e packing binário em `Bytes`.
- `13_init_and_invariant.aipo`: Hooks `init()` e validação de `invariant()`.
- `15_invariant_on_mutation.aipo`: Interceptação de invariantes em mutações de campos.
- `16_interface_contracts.aipo`: Validação de contratos em tempo de execução.
- `21_attempt_recovery_and_journal.aipo`: Rollback transacional atômico de estruturas.
- `24_async_functions_and_await.aipo`: Funções assíncronas e bloco `await do`.
- `26_async_combinators.aipo`: Combinadores de tarefas (`task.all`, `task.race`, etc.).
- `28_time_clock_capability.aipo`: Proteção da capability `clock` na Host ABI.

### 2. Suíte de Diagnósticos (`docs/conformance/diagnostics/`)
Testes negativos que garantem que códigos malformados ou ilegais são rejeitados com diagnósticos precisos:
- `AIPO_LEX_INVALID_NUMBER`: Literais numéricos com ponto sem dígitos seguintes.
- `AIPO_SEM_FIXED_REASSIGN`: Tentativa de mutação em campos imutáveis `fixed`.
- `AIPO_SEM_AWAIT_IN_SUBEXPRESSION`: Uso de `await` fora de blocos sequenciais.
- `AIPO_SEM_FORGOTTEN_TASK`: Criação de tarefa assíncrona não aguardada.
- `AIPO_RT_AWAIT_CYCLE`: Detecção de deadlocks e ciclos entre tarefas concorrentes.

### 3. Conformance de Formatação (`docs/conformance/formatting/`)
Casos de teste garantindo a estabilidade e idempotência do formatador canônico `aipo fmt`.
