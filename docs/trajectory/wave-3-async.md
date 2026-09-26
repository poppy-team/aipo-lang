# Wave 3 — Tipos Ricos, Async & Concorrência

A **Wave 3** introduziu novos tipos de dados estruturais na linguagem e consolidou a **infraestrutura de concorrência assíncrona cooperativa com scheduler determinístico**.

---

## Marcos Conquistados

### 1. Tipos e Valores Ricos
- **`Set`**: Coleção de elementos únicos com preservação estrita da ordem original de inserção.
- **`Sequence`**: Geradores de avaliação preguiçosa (*lazy evaluation*), permitindo pipelines de processamento contínuo sem alocação intermediária.
- **`Bytes` Packing**: Métodos de leitura e escrita tipados em memória contígua (`read_u16_le`, `write_u32_be`, etc.), ideais para protocolos binários e I/O de alta velocidade.
- **`Duration`**: Tipo nativo para intervalos de tempo precisos.

### 2. Sintaxe & Semântica Assíncrona
- Funções declaradas com `async fn` passam a retornar handles de tarefas cooperativas.
- Bloco sequencial `await do ... end`, evitando a dispersão de `await` soltos em subexpressões e prevenindo condições de corrida sutis.
- Diagnósticos estáticos específicos:
  - `AIPO_SEM_AWAIT_IN_SUBEXPRESSION`: Bloqueia o uso de `await` fora de blocos dedicados.
  - `AIPO_SEM_FORGOTTEN_TASK`: Alerta quando uma tarefa criada não é aguardada nem associada a um grupo.
  - `AIPO_SEM_NESTED_AWAIT_DO`: Proíbe aninhamento confuso de blocos de espera.

### 3. Scheduler Cooperativo com Tempo Virtual
- O runtime do Aipo implementa um agendador de tarefas determinístico:
  - As tarefas cooperativas cedem o controle em pontos de suspensão explícitos (`task.sleep`, `await`).
  - O tempo não depende do relógio do sistema operacional (wall-clock), mas sim de um **relógio virtual determinístico**, permitindo que testes com dezenas de timeouts rodem em milissegundos sem flakiness.

### 4. Combinadores Assíncronos & Detecção de Ciclos
- Biblioteca `task` completa: `task.spawn`, `task.sleep`, `task.all`, `task.race`, `task.timeout`, `task.cancel`, `task.group`.
- Detecção em tempo de execução de ciclos de dependência transitiva entre tarefas (`AIPO_RT_AWAIT_CYCLE`), impedindo deadlocks silenciosos.
