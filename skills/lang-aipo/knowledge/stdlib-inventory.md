# Aipo V1 — Inventário da Biblioteca Padrão (Stdlib)

Este documento descreve os módulos embutidos da biblioteca padrão do runtime Aipo V1.

---

## 1. Módulo `io` (Entrada e Saída)

| Função | Assinatura | Descrição |
|---|---|---|
| `io.print` | `io.print(msg: String)` | Imprime texto na saída padrão sem quebra de linha. |
| `io.println` | `io.println(msg: String)` | Imprime texto na saída padrão seguido de `\n`. |
| `io.read_line` | `io.read_line() -> String` | Lê uma linha da entrada padrão até `\n`. |
| `io.read_file` | `io.read_file(path: String) -> String` | Lê todo o conteúdo de um arquivo em UTF-8. Dispara `fail` se inexistente. |
| `io.write_file` | `io.write_file(path: String, content: String) -> Bool` | Escreve arquivo de forma atômica. Dispara `fail` em erro de disco. |

---

## 2. Módulo `math` (Matemática e Aritmética)

| Função | Assinatura | Descrição |
|---|---|---|
| `math.sqrt` | `math.sqrt(x: Float) -> Float` | Raiz quadrada de ponto flutuante. |
| `math.abs` | `math.abs(x: Num) -> Num` | Retorna o valor absoluto de Int ou Float. |
| `math.floor` | `math.floor(x: Float) -> Int` | Arredonda para baixo. |
| `math.ceil` | `math.ceil(x: Float) -> Int` | Arredonda para cima. |
| `math.min` | `math.min(a: Num, b: Num) -> Num` | Menor entre dois números. |
| `math.max` | `math.max(a: Num, b: Num) -> Num` | Maior entre dois números. |
| `math.pow` | `math.pow(base: Num, exp: Num) -> Float` | Exponenciação. |

> **Nota:** Não existem `NaN` nem `Infinity` em Float. Operações inválidas disparam falha estruturada do runtime.

---

## 3. Módulo `task` (Concorrência e Tempo Virtual)

| Função | Assinatura | Descrição |
|---|---|---|
| `task.sleep` | `task.sleep(ms: Int)` | Pausa a execução cooperativa da corrotina por `ms` milissegundos. |
| `task.spawn` | `task.spawn(fn) -> Task` | Cria uma nova tarefa cooperativa em segundo plano. |
| `task.all` | `task.all(tasks: List) -> List` | Aguarda todas as tarefas completarem em paralelo. |
| `task.race` | `task.race(tasks: List) -> Any` | Retorna o resultado da primeira tarefa concluída. |
| `task.timeout` | `task.timeout(ms: Int, task) -> Any` | Executa tarefa limitando o tempo máximo de execução. |
| `task.now` | `task.now() -> Int` | Retorna timestamp virtual em milissegundos. |

---

## 4. Módulo `sys` (Ambiente e Sistema Operacional)

| Função | Assinatura | Descrição |
|---|---|---|
| `sys.args` | `sys.args() -> List` | Retorna os argumentos da linha de comando fornecidos ao processo. |
| `sys.exit` | `sys.exit(code: Int)` | Encerra o processo imediatamente com código numérico de saída. |
| `sys.env` | `sys.env(name: String) -> String?` | Lê variável de ambiente do sistema operacional (`none` se ausente). |

---

## 5. Primitivos Globais e Conversões de Tipo

Funções globais sempre acessíveis sem necessidade de `import`:

| Primitivo | Exemplo | Descrição |
|---|---|---|
| `len(x)` | `len([1, 2, 3])`, `len("texto")` | Retorna comprimento de lista, dicionário ou string. |
| `Int(x)` | `Int(3.14)` $\rightarrow$ `3`, `Int("42")` $\rightarrow$ `42` | Conversão e truncamento para inteiro de 64 bits. |
| `Float(x)` | `Float(42)` $\rightarrow$ `42.0` | Conversão para número de ponto flutuante de 64 bits. |
| `String(x)` | `String(123)` $\rightarrow$ `"123"` | Converte qualquer valor primitivo em representação de string. |
| `Bool(x)` | `Bool(1)` $\rightarrow$ `true` | Conversão explícita para booleano. |
