# A Saga de Otimização & Rigor Metodológico

Uma das maiores lições de engenharia do projeto Aipo é a **disciplina de medição empírica**: código que parece uma otimização no papel muitas vezes não gera ganho real quando submetido a rigorosos testes pareados.

Documentar os experimentos que foram **revertidos** é tão importante quanto celebrar as melhorias mantidas.

---

## Ganhos Comprovados & Mantidos

Durante a evolução da VM e do runtime, as seguintes otimizações foram comprovadas por medição com controle estatístico:

1. **Cache Monomórfico de Field Slot**:
   - Reduziu o tempo do benchmark `fields` de 1.001,77ms para 718,20ms (**redução de 28,31%**).
2. **Cache de Base Frame**:
   - Evitou chamadas desnecessárias a `frames.last()` nos acessos a variáveis locais, gerando reduções estáveis de **-3,71% em `arithmetic`**, **-6,65% em `fields`** e **-4,17% em `recursion`**.
3. **Eliminação de Clones em `SetField`**:
   - Eliminou 600.000 clonagens de `Value` de forma determinística para estruturas não protegidas (*unguarded*), mantendo a semântica de rollback intacta.

---

## Os 7 Experimentos Revertidos

Sete hipóteses de otimização foram minuciosamente implementadas, testadas com o script pareado `scripts/perf/paired.sh` e **revertidas** após os dados confirmarem ausência de ganho demonstrável:

| Experimento | Hipótese Técnica | Resultado Observado | Decisão |
| :--- | :--- | :--- | :--- |
| **1. Cache de Metadados de Métodos** | Evitar buscas repetidas na tabela de métodos | Sem variação estatisticamente significante | **Revertido** |
| **2. Pool de Nomes (`Rc<str>` vs Enum)** | Reduzir alocações na resolução de identificadores | Custo de chaveamento superou a economia | **Revertido** |
| **3. Guardedness na Entrada de Cache** | Evitar checagens dinâmicas redundantes | Neutro no wall-clock | **Revertido** |
| **4. Flags `fixed` por Slot** | Bitmask compacta para campos imutáveis | Ruído dentro da margem de erro (±2%) | **Revertido** |
| **5. Cache de Tipos Guarded** | Reutilização de esquemas de proteção | Ganho nulo frente à contenção de cache L2 | **Revertido** |
| **6. Layout Denso de `StructInstance`** | Vetor contíguo para campos primitivos | Deslocamento de offset oscilou entre +7% e neutro | **Revertido** |
| **7. Despacho de Método Sem Alocar** | Evitar ponteiros intermediários | Compilador já eliminava alocações no caminho feliz | **Revertido** |

---

## O Diagnóstico do Laço de Despacho

Uma calibração com número fixo de iterações revelou um fato crucial:
- O custo por opcode na VM é **constante (~100ns)** e independe da mistura de operações.
- O gargalo estrutural não está nas instruções individuais, mas no laço de despacho (*dispatch overhead*) e no tamanho largo do tipo `VmFault` (72 bytes).
- A tentativa de otimizar a decodificação de opcodes sem `Result` produziu ganho **nulo (-0,16%)**, comprovando que o LLVM já afundava a montagem da falha para fora do caminho feliz (*cold branch*).

---

## Regras Canônicas de Medição (A precondição para o futuro)

Para evitar retrabalho em máquinas sujeitas a ruído e contenção de CPU:

1. **Diferença menor que ±5% não é evidência de ganho**.
2. **Um único lote pareado não decide nada**: toda proposta exige pelo menos dois lotes com inversão de ordem candidate/baseline.
3. **`arithmetic` é o controle obrigatório**, por não depender de structs nem de despacho dinâmico de métodos.
4. **Sob contenção de carga, tempo de CPU (`RUSAGE_CHILDREN`) é a única métrica confiável**. Wall-clock em ambiente concorrido produz falsos positivos.

A trilha de micro-otimização do interpretador foi formalmente **encerrada**. Qualquer reabertura futura exigirá hardware dedicado e foco em mudanças arquiteturais macro (`Call0..Call4`, `GetLocal8` ou jump tables com threaded dispatch).
