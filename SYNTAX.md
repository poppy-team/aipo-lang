# Aipo — Referência Canônica de Sintaxe

**Status:** normative
**Escopo:** superfície sintática aceita pelo compilador nesta revisão
**Verificado contra:** `crates/aipo-lexer/src/token.rs`, `crates/aipo-syntax/src/parser.rs`, `crates/aipo-ast/src/ast.rs`
**Corpus de prova:** `examples/*.aipo`, `docs/conformance/programs/*.aipo`, `docs/conformance/formatting/*.expected.aipo`
**Complementa:** `docs/manual/*` (narrativo), `docs/language/authority-map.md` (hierarquia de autoridade)

Este documento existe porque o manual em `docs/manual/` é narrativo e não lista
a superfície completa. As tabelas abaixo foram extraídas do código, não da
documentação — divergências entre manual e compilador foram registradas em
`docs/journal/` no mesmo commit.

---

## 0. TL;DR

- **46 keywords** e **36 operadores**.
- Comentário é **`#`** até o fim da linha. **`//` NÃO é comentário** — é divisão inteira.
- Todo bloco aceita **duas** terminações: `end` **ou** `{ }`. Escolha uma e mantenha.
- **Não existe:** `enum`, `try`/`catch`/`throw`, `defer`, generics, `pub`/`private`,
  `for`, `class`, `const`, bitwise, `;`, `@attr`, cast com `as`.

---

## 1. Terminação de bloco

Toda construção que abre bloco aceita as duas formas. Misturar no mesmo arquivo
é permitido pelo compilador, mas é anti-padrão de legibilidade.

```aipo
if ativo {            # forma de chaves
    io.println("ok")
}

if ativo then         # forma `end`
    io.println("ok")
end
```

Aplica-se a: `fn`, `async fn`, `struct`, `impl`, `init`, `invariant`,
`interface`, `if`/`elif`/`else`, `match` (e cada braço `when`), `loop`,
`while`, `repeat`, `each`, `attempt`/`failed`, `await do`, bloco trailer.

---

## 2. Comentários e trivia

| Item | Regra |
|---|---|
| Comentário de linha | `#` até o fim da linha — **único** comentário da linguagem |
| Comentário de bloco | não existe |
| Newline | **significativo** — termina statement; newlines consecutivos colapsam em um |
| Espaços | espaço, tab e CR são ignorados |
| BOM | removido; CRLF/CR normalizados para LF |
| Identificadores | começam com `_` ou letra Unicode; continuam com `_` ou alfanumérico Unicode |
| NFC | aplicado ao **conteúdo de literais de string**, não a identificadores |

> **Armadilha:** `//` é `SlashSlash` (divisão inteira). Escrever um comentário com
> `//` produz `AIPO_RT_DIV_ZERO` ou `AIPO_SEM_UNKNOWN_NAME` — nunca um comentário.

---

## 3. Literais

### 3.1 Strings

| Forma | Nome |
|---|---|
| `"..."` | string normal |
| `f"..."` | formatada (interpolada) |
| `r"..."` | raw (barras literais) |
| `fr"..."` / `rf"..."` | raw + formatada (ambas as ordens) |
| `"""..."""` | multilinha |
| `r"""..."""` / `fr"""..."""` | raw multilinha |

- Escapes (fora de raw): `\n` `\r` `\t` `\\` `\"` `\0` `\u{HEX}`
- String de linha única **não** pode conter newline cru
- F-string: `f"{expr}"`; `{{` e `}}` produzem brace literal; `}` isolado é erro
- F-strings são dessucradas no parser para `"seg" + String(expr) + "seg"`

```aipo
let pasta = "logs"
let caminho = fr"C:\sistema\{pasta}\app.txt"   # C:\sistema\logs\app.txt
let json = f"{{\"k\": \"{chave}\"}}"            # {"k": "..."}
```

### 3.2 Números

| Base | Prefixo | Exemplo |
|---|---|---|
| Decimal | — | `42`, `3.14`, `1.5e-3`, `12e5` |
| Hexadecimal | `0x` / `0X` | `0xFF` → 255 |
| Binário | `0b` / `0B` | `0b101010` → 42 |
| Octal | `0o` / `0O` | `0o777` → 511 |

- `_` é separador **apenas entre dois dígitos da mesma base**: `1_000_000` ✓, `0xFF_FF` ✓
- Inválidos: `_1`, `1_`, `1__0`, `1e_5`, `0x_FF` → `AIPO_LEX_INVALID_NUMBER`
- `Int` é inteiro assinado de 64 bits. Overflow de literal é adiado para o load → `AIPO_RT_OVERFLOW`
- `Float` é IEEE 754 de 64 bits. **Sem `NaN` e sem `Infinity`** — divisão por zero e
  raiz de negativo geram falha estruturada

### 3.3 Literais nomeados

`true` · `false` · `none` · `_` (descarte em padrão de binding)

---

## 4. Declarações de topo

Sete formas de `Item`:

```aipo
fn nome(params) -> T { } | end
async fn nome(params) -> T { } | end
struct Nome { campos }                     # ou: struct Nome ... end
impl Tipo { init / invariant / fn / async fn }
interface Nome { fn assinatura(params) -> T }
satisfy Tipo: Interface1, Interface2
import a.b.c [as alias] [: n1, n2]
export n1, n2, n3
```

| Observação | Regra |
|---|---|
| `interface` | contém **apenas assinaturas** (`fn` ou `async fn`), nunca corpo |
| `satisfy` | `:` obrigatório, lista separada por vírgula |
| `export` | apenas **nomes simples** — `export mod.Nome` é erro de parse |
| `import` | caminho pontilhado é válido (`auth.user`), alias com `as`, seleção com `:` |
| Módulo | **um arquivo `.aipo` é um módulo.** Não há declaração de módulo |

---

## 5. Structs e contratos

```aipo
struct Servidor {
    id                       # fixo (imutável) por padrão
    var status = "online"    # var = mutável
    var carga = 0.0          # com valor padrão
}
```

- Campos de struct **não aceitam anotação de tipo**. `x: Float` é erro de parse.
- Bare `x` = fixo sem default; `var x` = mutável sem default; `x = v` / `var x = v` = com default.
- Reatribuir campo fixo após a construção → `AIPO_SEM_IMMUTABLE_FIELD_REASSIGN`.

### 5.1 Construção

```aipo
let s = Servidor{ id: 1, status: "online" }   # ':' e '=' ambos aceitos
let s = Servidor{ 1, "online" }                # posicional também
```

> **Armadilha:** construção `Tipo{...}` exige **primeira letra maiúscula**.
> `mod.tipo{...}` nunca é reconhecido como construção.

### 5.2 `impl` — hooks e métodos

```aipo
impl Servidor {
    init(id) { self.id = id }              # hook de construção
    invariant { self.inicio <= self.fim }  # invariante estrutural
    fn atual(self) -> Int { return self.x }
    fn bump(var self) { self.x += 1 }      # var self = receptor mutável
}
```

- `init` recebe `self` mutável automaticamente se você não declarar.
- `invariant` é verificado a cada mutação de campo; dentro de `attempt`, viola
  dispara rollback do journal.
- `self` é só-leitura por padrão. `self!` e `var self` declaram mutabilidade.

### 5.3 Interfaces

Subtipagem **estrutural e automática**: um tipo satisfaz uma `interface` só com os
métodos de assinatura compatível. Nenhuma declaração é necessária. `satisfy` existe
como declaração explícita e redundante.

---

## 6. Statements

| Forma | Observação |
|---|---|
| `let <padrão> = expr` | imutável |
| `var <padrão> = expr` | mutável |
| `alvo = expr` | atribuição simples |
| `alvo op= expr` | `+=` `-=` `*=` `/=` `div=` `//=` `%=` |
| `if c { } elif c { } else { }` | `then` opcional; sem parênteses na condição |
| `if c then a else b` | **inline, com valor** — expressão e statement |
| `match alvo { when p1, p2 { } else { } }` | `when`, nunca `case` |
| `loop { }` | laço infinito canônico |
| `while c { }` | sem palavra `do` |
| `repeat N [as i] { }` | `as` vincula o índice |
| `each a in xs { }` | multi-binding: `each i, v in dict { }` |
| `break` / `continue` | sem label |
| `return [expr]` | sem expr retorna `none` |
| `fail expr` | ou `return fail(...)` |
| `attempt { } failed [erro] { }` | `failed` obrigatório, binding opcional |
| `fn nome(...) { }` | função local, auto-recursão resolvida |
| `await expr` / `await do { }` | ver §10 |

Padrões de binding: `nome` · `_` · `[a, b]` · `{x, y}`

---

## 7. Falhas

```aipo
fail "mensagem"                          # ou return fail("mensagem")
attempt { risky() } failed erro { io.println(erro.message) }
let porta = ler_porta() or_else 8080
```

Modelo transacional: `attempt` registra um journal e reverte atomicamente todas as
mutações em caso de `fail` ou violação de invariante.

**Não existe:** `try`, `catch`, `finally`, `throw`, `raise`, `defer`.

---

## 8. Funções

```aipo
fn soma(a, b) { return a + b }
fn conecta(host, porta = 8080, timeout = 5000) { }
conecta("api", timeout = 1000)                 # argumento nomeado

fn(x) { ... }                                  # closure com captura léxica
x => x * 2     (a, b) => a + b     () => e     # lambda curto
f(a) do ... end     f do ... end                # bloco trailer
```

Formas de parâmetro aceitas: `nome` · `var nome` · `nome!` · `self` · `self!` ·
`var self` · `_` — cada uma opcionalmente com `: T`, `: T?` e `= default`.

Retorno: `-> T` e `-> T?`. Sem `->`, o contrato não é declarado.

> **Armadilha:** anotação de tipo aceita **um único nome simples**.
> `fn f(x: mod.Tipo)` é erro de parse — use `fn f(x: Tipo)` e importe o módulo.

---

## 9. Async

```aipo
async fn buscar(recurso) { task.sleep(50)  return recurso }
let t = buscar("usuarios")
let v = await t

await do {              # bloco sequencial
    let a = await etapa_1()
    let b = await etapa_2()
    return a + b
}
```

Combinadores são stdlib, não sintaxe: `task.spawn`, `task.sleep`, `task.all`,
`task.race`, `task.timeout`, `task.cancel`, `task.group`.

Tempo virtual e determinístico. Espera mútua transitiva → `AIPO_RT_AWAIT_CYCLE`.

---

## 10. Módulos e pacotes

```aipo
import math_util                 # qualificado
import math_util as mu           # com apelido
import math_util: somar, dobro   # seletivo
export somar, dobro
```

- **Privado por padrão.** Acesso a símbolo não exportado → `AIPO_SEM_UNKNOWN_NAME`.
- `import.types` (sem espaço) é **erro de parse**. Use `import types`.
- Pacote = manifesto `aipo.toml` com coordenadas `namespace.name`.
- Dependência remota exige **commit SHA de 40 dígitos**; branch e tag são rejeitados.
- `aipo run` e `aipo build` nunca acessam a rede.

---

## 11. Operadores e precedência

Do mais fraco ao mais forte:

| # | Operadores |
|---|---|
| 1 | `\|>` pipeline |
| 2 | `or_else` |
| 3 | `or` |
| 4 | `and` |
| 5 | `not` (prefixo) |
| 6 | `==` `!=` `<` `<=` `>` `>=` `is` / `is T?` |
| 7 | `..` (range e slice) |
| 8 | `+` `-` |
| 9 | `*` `/` `div` `//` `%` |
| 10 | prefixo `-x` `+x` · pós-fixos `()` `[]` `.` `?.` `do` |

### 11.1 Notas por operador

| Operador | Semântica |
|---|---|
| `/` | **sempre** retorna `Float` |
| `//` e `div` | mesma coisa: divisão inteira truncada |
| `//=` e `div=` | mesma coisa: atribuição com divisão inteira |
| `\|>` | passa o valor como **primeiro** argumento da chamada à direita |
| `or_else` | avalia a direita se a esquerda disparar `fail` |
| `..` | range como valor (`0..3`) e em slice |
| `?.` | encadeável; qualquer elo `none` propaga `none` |
| `is T?` | teste de tipo anulável |

Slice aceita limite omitido: `xs[..n]` · `xs[n..]` · `xs[..]`

### 11.2 Rejeições explícitas

| Construção | Código |
|---|---|
| Comparação encadeada `a < b < c` | `AIPO_PARSE_UNEXPECTED_TOKEN` |
| `is not T` | use `not v is T` |
| Elisão de comparação **após** `or` | só funciona após `and` |

Elisão após `and` é açúcar: `v >= 0 and <= 100` → `(v >= 0) and (v <= 100)`.
Açúcar multi-sujeito em `is`: `a, b, c is Int` → os três `is Int`.

---

## 12. Lista completa de keywords (46)

| Grupo | Keywords |
|---|---|
| Binding | `let` `var` `fixed` |
| Função | `fn` `async` `await` |
| Tipo | `struct` `impl` `interface` `satisfy` |
| Hook | `init` `invariant` |
| Controle | `if` `elif` `else` `then` `end` `match` `when` `loop` `while` `repeat` `each` `in` `break` `continue` `return` |
| Falha | `fail` `or_else` `attempt` `failed` |
| Módulo | `import` `export` |
| Operador-word | `is` `not` `and` `or` `div` `as` |
| Literal | `true` `false` `none` |
| Receptor | `self` `self!` |
| Bloco | `do` |
| Descarte | `_` |

Contextuais: `self!` (só após `self`) e `div=` (só após `div`).

> **Armadilha:** `as` **não** é cast. Aparece só em alias de `import` e em
> `repeat N as i`. `div` e `self!` são palavras-chave — não podem ser identificadores.

---

## 13. Palavras-chave que NÃO existem

`enum` · `defer` · `try` · `catch` · `finally` · `throw` · `raise` · `for` ·
`class` · `const` · `static` · `pub` · `private` · `public` · `unsafe` ·
`yield` · `lambda` · `module` · `package` · `void` · `case`

Operadores inexistentes: `&&` · `||` · `++` · `--` · `**` · `&` · `;` ·
`<<` · `>>` · `~` · `^` · `@attr` · `?:` · cast `as` · `*ref` · `->campo`

Os caracteres `@`, `&`, `;`, `&&`, `||`, `++` não são tokenizados: caem em
`unexpected character`. Um `|` isolado é erro com dica *"did you mean '|>'?"*.

**`enum` está no backlog pós-V1** (`docs/canon/Aipo — Backlog de Sintaxe …`),
não na linguagem atual.

---

## 14. Limites de aninhamento

| Limite | Valor | Diagnóstico |
|---|---|---|
| Profundidade de expressão | 128 | `AIPO_PARSE_NESTING_TOO_DEEP` |
| Profundidade de bloco | 64 | `AIPO_PARSE_NESTING_TOO_DEEP` |

---

## 15. Onde conferir

| Fonte | Caminho |
|---|---|
| Tokens e keywords | `crates/aipo-lexer/src/token.rs`, `crates/aipo-lexer/src/lexer.rs` |
| Tabela de grafia | `crates/aipo-formatter/src/tokens.rs` |
| Gramática e precedência | `crates/aipo-syntax/src/parser.rs` |
| AST | `crates/aipo-ast/src/ast.rs` |
| Exemplos executáveis | `examples/*.aipo` |
| Snapshots de saída | `docs/conformance/programs/*.stdout` |
| Código que **deve** falhar | `docs/conformance/diagnostics/*.aipo` + `.code` |
| Fonte canônica por convenção | `docs/conformance/formatting/*.expected.aipo` |

Para alterar a gramática: o corpus de conformidade é o árbitro. `*.stdout` é a
saída exata de `aipo run`; `*.code` lista um código de diagnóstico por linha e
falhar pelo motivo errado também falha.
