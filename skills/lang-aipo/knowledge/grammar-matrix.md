# Aipo V1 — Matriz Gramatical e Mapeamento de AST

Este documento sintetiza a gramática formal de Aipo V1 conforme o ADR-001 e a estrutura da AST em `crates/aipo-ast/src/ast.rs`.

---

## 1. Top-Level Items (Declarações de Topo)

| Item | Sintaxe Canônica | Representação na AST | Observações |
|---|---|---|---|
| Função Livre | `fn nome(p1, p2) -> Ret { corpo }` | `Item::Fn` | `fn` só para funções livres. Auto-recursão permitida. |
| Função Async | `async fn nome(p1) -> Ret { corpo }` | `Item::AsyncFn` | Gera corrotina cooperativa determinística. |
| Struct | `struct Nome { campos }` | `Item::Struct` | Apenas campos. Tipos de campos opcionais. |
| Enum | `enum Nome { variantes, }` | `Item::Enum` | Tipo fechado de soma. Vírgula final obrigatória. |
| Método Associado | `Tipo:nome(params) -> Ret { corpo }` | `Item::Method` | `self` implícito; mutável exige `var self`. |
| Hook de Inicialização | `Tipo:init(params) { corpo }` | `Item::InitHook` | Executado na construção com `self` mutável. |
| Hook de Invariante | `Tipo:invariant { predicados }` | `Item::InvariantHook` | Avaliado após mutações; dispara rollback se falhar. |
| Associação em Lote | `Tipo::[fn1, fn2]` | `Item::BatchBind` | Promove funções com `self` como 1º param para métodos. |
| Interface | `interface Nome { assinaturas }` | `Item::Interface` | Apenas assinaturas estruturais sem corpo. |
| Diretiva | `#!diretiva` | `Item::Directive` | Anota item seguinte (`#!satisfies`, `#!test`, `#!deprecated`, `#!todo`). |
| Importação | `import caminho [as alias] [: itens]` | `Item::Import` | Caminho pontilhado ou seletivo. |
| Exportação | `export item1, item2` | `Item::Export` | Publica símbolos privados do módulo. |

---

## 2. Expressões e Statements

| Categoria | Sintaxe | Semântica |
|---|---|---|
| Binding Imutável | `let padrao = expr` | Vincula identificador de forma somente-leitura. |
| Binding Mutável | `var padrao = expr` | Permite reatribuição local posterior. |
| Atribuição | `alvo = expr` | Reatribui variável `var` ou campo `var`. |
| Atribuição Composta | `alvo +=`, `-=`, `*=`, `/=`, `//=`, `%=` | Inclui divisão inteira truncada (`//=`). |
| Cópia com Atualização | `alvo with { campo: valor }` | Devolve nova struct com campos modificados. |
| Condicional em Bloco | `if c { } elif c { } else { }` | Condição sem parênteses. Nunca usa `then`. |
| Expressão Condicional | `if c then a else b` | Expressão inline com valor. Sempre usa `then`. |
| Pattern Matching | `match alvo { when p if guard { } else { } }` | Destructuring em `when`, nunca usa `case`. |
| Laço Infinito | `loop { break }` | Laço canônico sem condição de terminação. |
| Laço Enquanto | `while c { }` | Laço condicional simples. |
| Laço Repetição | `repeat N [as i] { }` | Repete N vezes vinculando índice opcional. |
| Laço Iteração | `each item in xs { }` / `each k, v in dict { }` | Itera sobre coleções ou dicionários. |
| Retorno | `return [expr]` | Retorna valor ou `none`. |
| Disparo de Falha | `fail expr` | Dispara `Failure` transacional. |
| Bloco Transacional | `attempt { } failed [e] { }` | Executa com journal de rollback em caso de falha. |
| Aguardo Assíncrono | `await expr` / `await do { }` | Suspende execução até conclusão da tarefa. |

---

## 3. Precedência de Operadores

Da menor para a maior precedência:

1. `|>` (Pipeline)
2. `or_else` (Fallback seguro de falha)
3. `or` (Disjunção lógica)
4. `and` (Conjunção lógica)
5. `not` (Prefixo de negação lógica)
6. `==`, `!=`, `<`, `<=`, `>`, `>=`, `is`, `is T?` (Comparações e checagem de tipo)
7. `..` (Range de intervalo e fatiamento)
8. `+`, `-` (Adição e subtração)
9. `*`, `/`, `//`, `%` (Multiplicação, divisão Float, divisão inteira, módulo)
10. Unários (`-x`, `+x`), Pós-fixos (`()`, `[]`, `.`, `?.`, `do`, `?`, `with`)
