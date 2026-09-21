# Aipo V1 — Sintaxe Canônica Consolidada

<aside>
✅

**Fonte normativa da superfície V1.** Quando exemplos históricos da Especificação Viva ou do Registro de Decisões divergirem desta página, esta página e as decisões mais recentes prevalecem.

</aside>

## Princípios da revisão de setembro de 2026

- Uma forma canônica por operação; aliases sintáticos desnecessários são evitados.
- Reutilizar expressões normais da linguagem em vez de criar mini-DSLs quando possível.
- `struct` descreve dados/shape; `impl` organiza comportamento associado e os hooks estruturais especiais `init()` e `invariant()` sem introduzir classes, herança ou métodos virtuais.
- Ausência (`none`), falha recuperável e ausência de valor de retorno são conceitos distintos.

## Struct, `init`, `fixed` e `invariant()`

A forma canônica abandona regras de campo com `where`/`@`. A `struct` descreve somente seus campos/shape. A lógica estrutural associada vive em `impl Type`: `init()` define a construção e `invariant()` define as propriedades que precisam permanecer verdadeiras.

```
struct Player
    fixed id
    name
    health
end

impl Player
    init(id: Int, name: String, health: Int = 100)
        self.id = id
        self.name = name
        self.health = health
    end

    invariant()
        self.name != ""
        self.health >= 0
        self.health <= 100
    end
end
```

- `init` é opcional e existe no máximo uma vez por `struct` na V1.
- `invariant()` é opcional e existe no máximo uma vez por `struct` na V1.
- Ambos pertencem a `impl Type`, mas são hooks estruturais especiais, não funções comuns chamáveis por dot-call nem valores de função.
- `Type{...}` usa o `init` quando ele existe; `init` não produz valor diretamente — a construção produz a nova instância. `()` permanece reservado à superfície de chamada.
- Dentro de `init`, `self` representa a instância ainda em construção e pode receber seus campos iniciais.
- Todos os campos obrigatórios precisam estar inicializados quando `init` termina. Defaults de parâmetros de `init` definem ergonomicamente valores opcionais da construção.
- Quando uma `struct` não declara `init`, permanece disponível a construção automática baseada nos campos segundo a forma canônica da linguagem.
- `fixed field` é propriedade estrutural do campo: ele pode receber seu valor durante a fase de construção — inclusive por `init` — e não pode ser substituído depois que a instância é publicada. Campos não carregam contratos declarativos de tipo na V1.
- `fixed` não congela profundamente o objeto armazenado. Se um campo `fixed` contiver uma identidade gerenciada, a identidade do campo não pode ser substituída, mas mutações internas continuam sujeitas às regras normais do caminho.
- `invariant()` não recebe parâmetros e usa `self` como receiver somente leitura.
- O corpo de `invariant()` contém uma ou mais expressões `Bool`; todas precisam ser verdadeiras após a conclusão de `init`/construção e nos pontos de validação de mutações posteriores.
- Linhas do hook equivalem conceitualmente a condições combinadas por `and`.
- Como `invariant()` usa expressões normais, ele pode incluir testes `is` para segurança opt-in sem criar uma mini-DSL separada.
- Na V1, invariants não observam estado mutável profundo de objetos gerenciados alcançados por alias (`List`, `Dict` ou outra struct gerenciada). Isso evita rastreamento oculto de aliasing. Invariants devem depender de campos com semântica de valor e de relações entre esses valores.
- Atualização de campo sujeita a invariant segue `candidate -> aplicação provisória -> verificação -> commit`; em falha recuperável, o valor anterior é preservado.

**Princípio:** `struct` declara o que existe; `init()` define como nasce; `invariant()` define o que precisa continuar verdadeiro; funções normais definem comportamento.

## Trailing blocks: `do ... end`

Chamadas podem receber uma closure como último argumento usando um bloco explícito `do ... end`.

```
transaction() do
    save(user)
    save(order)
end

file.use("config.txt") do file
    process(file.read())
end

items.each_pair() do key, value
    io.print(key, value)
end
```

A forma é açúcar sintático para uma função anônima passada como último argumento. Parâmetros após `do` são parâmetros normais da closure; captura lexical, mutabilidade e retorno seguem exatamente as regras de closures comuns. `return` dentro do trailing block retorna da própria closure, não da função externa. Não há `yield` especial nem non-local return.

O parser pode manter um `trailing_block` apenas até uma fase de lowering que o converte em `FunctionLiteral`; VM, bytecode e backends não precisam de um segundo modelo semântico de bloco.

Essa feature existe para composição geral de bibliotecas: builders, transações, escopos de recursos, testes, configuração, UI, pipelines e DSLs. Domínios específicos devem ser implementados por bibliotecas/frameworks sempre que possível, em vez de ganhar sintaxe própria no core.

Detalhes e possibilidades: [Interlúdio — Hooks Estruturais e Blocos como Argumentos](Interlúdio — Hooks Estruturais e Blocos como Argum 3d59bb7d023f81329bfcdfb74573e481.md).

## Núcleo dinâmico e contratos de assinatura

Bindings e campos permanecem dinamicamente tipados. Contratos opcionais aparecem somente em parâmetros e retornos de funções/assinaturas.

```
fn load_user(id: Int) -> User?
    ...
end

fn process(data, retries: Int)
    ...
end
```

- `name: Type` é contrato opcional de parâmetro.
- `-> Type` é contrato opcional de retorno.
- Omissão de contrato mantém a fronteira dinâmica.
- `let`/`var` e campos de `struct` não aceitam anotação de tipo.
- `is` faz teste/narrowing no fluxo; `invariant()` protege garantias duradouras de estado.
- `T?` significa `T` ou `none` em assinaturas e também pode aparecer em testes `is`.
- `List[T]`/`Dict[K, V]`, `Any`, unions arbitrárias, intersections e generics de usuário ficam fora da V1 inicial.

**Princípio:** o programador declara tipos onde valores atravessam fronteiras; o compilador observa tipos dentro dessas fronteiras.

Detalhes: [Interlúdio — Núcleo Dinâmico e Contratos de Assinatura](Interlúdio — Núcleo Dinâmico e Contratos de Assina 3d59bb7d023f812b8260ef925cf08cbc.md).

## Iteração: `each`

`each` substitui `for`; `for` não permanece como alias.

```
each player in players
    player.draw()
end

each index, player in players
    io.print(index, player)
end

each key, value in scores
    io.print(key, value)
end

each i in 0..10
    io.print(i)
end
```

Bindings introduzidos por `each` pertencem somente ao corpo do loop e cada iteração cria bindings próprios capturáveis por closures.

## Comportamento associado: `impl`

`impl Type` é a única forma canônica da V1 para agrupar funções receiver-associated.

```
impl Player
    fn alive(self) -> Bool
        return self.health > 0
    end

    fn damage(self!, amount: Int)
        self.health -= amount
    end
end
```

- `self` é o receiver explícito; `self!` concede acesso mutável através do receiver.
- `impl` é organização e açúcar estrutural, não classe, herança, dispatch virtual ou categoria separada de método.
- `player.damage(10)` continua sendo dot-call sobre uma operação associada.
- A forma histórica `fn Player.damage(player!, ...)` é superseded na superfície V1.
- A V1 não permite funções sem receiver dentro de `impl`; construtores/factories continuam funções normais de módulo.

## Interfaces e `satisfy`

```
interface Drawable
    fn draw(self)
end

interface Damageable
    fn damage(self!, amount: Int)
end

satisfy Player: Drawable, Damageable
```

Interfaces continuam estruturais e `satisfy` continua uma promessa/verificação explícita, não herança ou mixin.

## Seleção: `match` / `when`

```
match status
when "loading"
    io.print("loading")
when "ready", "done"
    io.print("ready")
else
    io.print("unknown")
end
```

`when` é a única palavra de ramo de `match`; `case` não faz parte da V1.

## Falhas: propagação, fallback e `attempt`

Falhas recuperáveis não tratadas propagam automaticamente. Uma única expressão pode usar fallback local:

```
let config = load_config() or_else default_config()
```

Para agrupar operações usa-se `attempt` / `failed`:

```
attempt
    let config = load_config()
    start(config)
failed err
    io.print(err.message)
end
```

- O nome após `failed` é um binding local somente leitura da falha capturada; `failed _` descarta o objeto de erro.
- `attempt` não é transacional: efeitos anteriores à falha não são revertidos.
- Runtime faults de programação não são capturados.
- `or_else` é o fallback local canônico para `Failure`: avalia o operando direito apenas quando a expressão à esquerda termina em `Failure`; runtime faults não ativam o fallback.
- `or` permanece exclusivamente Boolean e não é sobrecarregado para tratamento de falha.
- A V1 não possui operador ternário `?:`; escolha condicional de valor reutiliza `if condition then value else value` quando a expressão condicional for usada em contexto de valor.
- As formas históricas `try expression`, `try ... else ... end`, `try ... end`, `value or handler` e fallback de Failure escrito apenas com `else` não pertencem à superfície V1.
- `fail("message")` cria falha recuperável; `fail(err)` repropaga a mesma falha.

## Funções com valor e funções sem valor

Uma função que não executa `return value` em nenhum caminho é semanticamente **sem valor de resultado**.

```
fn log(message)
    io.print(message)
end
```

Pode ser usada como statement:

```
log("started")
```

Mas não em posição que exige valor:

```
let result = log("started") # erro
```

- A V1 não expõe `Void`/`Unit` como tipo público apenas para representar isso.
- `none` permanece um valor real e intencional de ausência; uma função que precisa produzir ausência deve `return none` explicitamente.
- Se uma função produz valor em algum caminho, todos os caminhos normais alcançáveis precisam produzir valor ou encerrar o fluxo (`fail`, retorno anterior etc.). Cair implicitamente ao fim é erro.

## Modelo de valores

**Semântica de valor:** `none`, `Bool`, `Int`, `Float`, `Byte`, `String`. Identidade física não é observável semanticamente.

**Identidade gerenciada:** instâncias de `struct`, `List`, `Dict`, `Bytes`, `Function/Closure`. Atribuição/passagem compartilha a mesma identidade; `copy(value)` permanece cópia superficial explícita.

`==` compara valor/conteúdo quando definido; `same(a, b)` pergunta identidade gerenciada; `is` testa tipo concreto ou conformidade estrutural.

## Visibilidade de campos

- A V1 não possui `public`, `private` ou `protected` por membro.
- Se a `struct` está acessível, seus campos declarados são acessíveis.
- `_name` não cria privacidade.
- `fixed` controla substituição do campo, não visibilidade.
- Mutabilidade continua seguindo o caminho de acesso (`let`, `var`, parâmetro comum, `self!`, namespace importado somente leitura).

## Escopo lexical

- Nomes vivem do ponto de declaração até o fim do bloco correspondente.
- Redeclarar o mesmo nome no mesmo escopo é erro.
- Shadowing em bloco interno é permitido.
- Parâmetros pertencem ao escopo da função.
- Bindings de `each`, incluindo índice/chave/valor, pertencem somente ao loop.
- Declarações estruturais de módulo (`fn`, `struct`, `interface`, `impl`, `satisfy`, `import`, `export`) seguem as regras estruturais do módulo; bindings executáveis `let`/`var` seguem ordem textual.

## Resumo canônico

```
struct Player
    fixed id
    name
    health = 100
end

interface Drawable
    fn draw(self)
end

satisfy Player: Drawable

impl Player
    invariant()
        self.name != ""
        self.health >= 0
        self.health <= 100
    end

    fn alive(self) -> Bool
        return self.health > 0
    end

    fn damage(self!, amount: Int)
        self.health -= amount
    end

    fn draw(self)
        io.print(self.name)
    end
end

each player in players
    player.draw()
end

match player.health
when 100
    io.print("perfect")
when 0
    io.print("dead")
else
    io.print("alive")
end

attempt
    save(player)
failed err
    io.print(err.message)
end
```

## Fechamento do Lote 1 — contratos, `is`, `T?` e retornos

- Contratos opcionais continuam restritos a parâmetros/retornos. Parâmetro mutável tipado usa `name!: Type`; receivers em `impl Type` continuam `self`/`self!` sem repetir tipo.
- Incompatibilidade comprovável é erro antecipado; violação descoberta em runtime é contract fault de programação, não falha recuperável capturável por `attempt`.
- `is` sempre produz `Bool`, faz narrowing flow-sensitive e aceita a forma múltipla `a, b, c is T`, açúcar para testes combinados por `and`. A negação canônica é `not value is T`; `is not` não é alias.
- `T?` significa exatamente `T` ou `none`, não cria wrapper e não admite `T??`; continua válido apenas em assinaturas e testes `is`.
- `return` encerra sem valor; `return expression` produz valor; `return none` produz o valor real `none`.
- Uma função com qualquer `return expression` alcançável é value-producing e todos os caminhos normais precisam produzir valor ou encerrar o fluxo. Sem `-> T`, retornos heterogêneos continuam válidos e são apenas inferidos internamente.
- Funções/closures no-result não podem ser usadas em posição de valor; `fail(...)` encerra o caminho e trailing blocks retornam apenas de sua própria closure.

**Status:** as quatro áreas estão fechadas para a V1.

## Fechamento do Lote 2 — números, igualdade, coleções e ranges

- `Int` é assinado de 64 bits e `Float` usa binary64 finito. A única promoção implícita é `Int -> Float`; `Float -> Int` exige `Int(value)` e trunca em direção a zero.
- `/` produz divisão real; `div` faz divisão inteira; `%` usa o mesmo quociente. Não há coerções automáticas com `String`/`Bool` nem wraparound silencioso.
- `==`/`!=` comparam valor/conteúdo; categorias claramente incompatíveis produzem `false`. `1 == 1.0` é `true` pela regra numérica. `same(a, b)` pergunta identidade e só é válido para valores com identidade gerenciada.
- `List` e `Dict` são coleções dinâmicas, heterogêneas, mutáveis e com identidade. `List[T]`/`Dict[K, V]` não fazem parte da V1 inicial.
- `dict[key]` exige chave existente e `dict.has(key)` testa presença; chave ausente nunca é silenciosamente convertida em `none`.
- `start..end` é range crescente half-open. `List`/`String` aceitam índices negativos; slicing também é half-open e permite limites omitidos. Range descendente implícito e stride ficam fora da V1.

**Princípio:** operações comuns são convenientes; conversões, perdas de informação e ambiguidades permanecem explícitas.

Detalhes: [Interlúdio — Números, Igualdade, Coleções e Ranges](Interlúdio — Números, Igualdade, Coleções e Ranges 3d69bb7d023f81f6895cec63bd012e7d.md).

## Fechamento do Lote 3 — fluxo, loops, falhas e escopo

- `if` usa `Bool` estrito, não é expressão e cada ramo possui escopo próprio. A forma inline canônica é `if condition then statement` com `else statement` opcional; ela aceita uma única instrução por ramo, não usa `end` e não produz valor.
- `match` avalia o alvo uma vez, usa `==`, escolhe o primeiro `when` compatível, não tem fallthrough e não recebe pattern matching complexo na V1.
- Loops canônicos: `loop`, `while`, `repeat n`, `each`; `break`/`continue` afetam o loop mais próximo, sem labels ou valor de `break`.
- `repeat` exige `Int >= 0`, avaliado uma única vez. `each` itera `List`, `Dict`, `String` e ranges; cada iteração cria bindings frescos.
- Alteração estrutural da coleção durante sua própria iteração é runtime fault; substituir elemento/valor já existente pode ser permitido por caminho mutável.
- `Failure` é recuperável; runtime fault é programming error. Somente `Failure` entra em `expression else fallback` ou `attempt`/`failed`.
- `fail("message")` cria Failure; `fail(err)` repropaga. `attempt` não reverte efeitos anteriores.
- Escopo é lexical, shadowing interno é permitido e redeclaração no mesmo escopo é erro. Closures capturam bindings, preservando mutabilidade de `var` e readonly de `let`.
- Funções locais existem a partir da declaração e podem ser recursivas.

Detalhes: [Interlúdio — Fluxo, Loops, Falhas e Escopo](Interlúdio — Fluxo, Loops, Falhas e Escopo 3d69bb7d023f81d6903eccbcf98b6675.md).

**Status:** Lote 3 fechado.

## Fechamento do Lote 4 — chamadas, construção, `impl` e módulos

- Argumentos são avaliados da esquerda para a direita, uma única vez. Posicionais vêm antes dos nomeados; defaults são avaliados a cada chamada.
- Dot-call existe apenas para operações declaradas em `impl Type`; funções globais comuns não recebem dot-call implicitamente.
- `struct` é construída com `Type{...}`. A presença de `init(...)` altera a assinatura aceita pelas chaves, não a superfície de construção. `()` permanece reservado a chamadas.

```
let point = Point{10, 20}
let named = Point{x = 10, y = 20}
```

- Múltiplos blocos `impl Type` são permitidos, mas cada operação é única por tipo; em toda a `struct` existe no máximo um `init()` e um `invariant()`.
- `invariant()` é validado em fronteiras mutáveis estáveis e após construção, permitindo estado temporário interno durante uma operação mutável.
- Um arquivo `.aipo` corresponde a um módulo. Tudo é privado por padrão; `export` expõe nomes. Imports canônicos: `import module`, `import module: names` e `import module as alias`.
- Declarações estruturais são resolvidas no módulo inteiro; `let`/`var` top-level seguem ordem textual. Imports são acíclicos na V1.

Detalhes: [Interlúdio — Chamadas, Construção, Impl e Módulos](Interlúdio — Chamadas, Construção, Impl e Módulos 3d69bb7d023f81a68cbde7a1f2d71297.md).

**Status:** Lote 4 fechado.

## Fechamento do Lote 5 — expressões, mutação, strings e callables

- Precedência canônica, da maior para a menor: postfix (`.`, `?.`, chamada, indexação), unários numéricos, `* / div %`, `+ -`, `..`, comparações/`is`, `not`, `and`, `or`, fallback `else`. Assignment é statement, não expressão.
- `and`/`or` fazem short-circuit e operam somente sobre `Bool`; `not value is T` significa `not (value is T)`. Comparações encadeadas não são açúcar da V1.
- `=` não produz valor. Compound assignments canônicos: `+=`, `-=`, `*=`, `/=`, `div=`, `%=`; o target é avaliado uma única vez. `++`, `--`, assignment expression e destructuring assignment ficam fora.
- Mutação exige caminho mutável (`var`, parâmetro `!`, `self!`). `List[index] = value` substitui posição existente; `Dict[key] = value` insere ou substitui.
- `String + String` é concatenação; nenhuma outra categoria é convertida implicitamente. `String(value)` converte valores fundamentais (`String`, `Int`, `Float`, `Bool`, `none`). Não há stringificação mágica de structs/coleções.
- `f"...{expr}..."` avalia interpolações uma vez, da esquerda para a direita; `r` é raw e `fr` a forma canônica combinada. Multiline não faz dedent semântico automático.
- `Function` é a categoria built-in de qualquer valor chamável; closure é `Function` + ambiente capturado. Chamada indireta usa as mesmas regras de `()` que chamada direta.
- `Function` como contrato verifica apenas callability; tipos de assinatura de função ficam fora da V1. Aridade/contratos são verificados antecipadamente quando conhecidos e em runtime como programming fault quando dinâmicos.
- Funções/closures não suportam `==`; identidade pode ser observada por `same`. Não há bound-method value automático em `object.operation`, callable object, hook `call` ou overload na V1.

**Status:** Lote 5 fechado.

Detalhes e justificativas: [Interlúdio — Expressões, Mutação, Strings e Callables](Interlúdio — Expressões, Mutação, Strings e Callab 3d69bb7d023f810a8628cfe5ac111878.md).

## Materiais complementares da superfície

- [Aipo V1 — Exemplo Integrado de Sintaxe](Aipo V1 — Exemplo Integrado de Sintaxe 3d79bb7d023f8118a234daa9b29634fc.md) — exemplo amplo para revisão pedagógica e futura suíte de conformidade.
- [Aipo — Backlog de Sintaxe e Recursos Pós-V1](Aipo — Backlog de Sintaxe e Recursos Pós-V1 3d79bb7d023f8128ab0af9327dadf75d.md) — ideias explicitamente não normativas, preservadas para reavaliação futura.

## Fechamento de ergonomia V1 — 2026-09-10

<aside>
✅

As formas abaixo foram aprovadas para a superfície V1. Elas reutilizam semânticas já existentes e devem sofrer lowering cedo, mantendo VM e runtime pequenos.

</aside>

### `repeat` com índice opt-in

`repeat` continua sem criar nome mágico por padrão. Quando o índice for útil, ele é declarado explicitamente com `as`:

```
repeat 5
    tick()
end

repeat 5 as i
    io.print(i)
end
```

- A contagem é avaliada uma vez e exige `Int >= 0`.
- O índice começa em `0` e termina em `count - 1`.
- O binding do índice é somente leitura e pertence ao corpo.
- Cada iteração cria um binding próprio capturável por closures.

### Chamada sem `()` antes de trailing block

Uma chamada sem argumentos seguida imediatamente de `do` pode omitir `()`:

```
html do page
    ...
end
```

é açúcar exclusivamente contextual para:

```
html() do page
    ...
end
```

A omissão de parênteses **não** se generaliza a chamadas comuns; `foo bar` não se torna uma forma de chamada.

### Pipeline `|>`

`|>` encadeia chamadas colocando o valor à esquerda como primeiro argumento da chamada à direita:

```
source
    |> tokenize
    |> parse
    |> analyze

text |> replace("a", "b")
```

Dessugaring conceitual:

```
analyze(parse(tokenize(source)))
replace(text, "a", "b")
```

- Não há placeholder `_` de pipeline na V1.
- O nó de pipeline deve ser apagado em lowering antes de Core IR baixo/bytecode sempre que possível.
- A posição definitiva do pipeline relativamente ao operador de fallback de `Failure` será sincronizada quando a sintaxe final desse fallback for fechada.

### Destructuring superficial de bindings

A V1 aceita destructuring apenas na criação de bindings:

```
let [x, y] = position
var [width, height] = size
let [first, _, third] = values
let {name, age} = user
```

- A expressão à direita é avaliada uma única vez.
- `[...]` extrai posições; `{...}` extrai campos de `struct` ou chaves textuais homônimas de `Dict`.
- Ausência/incompatibilidade produz diagnóstico antecipado quando comprovável ou runtime fault quando dinâmica.
- Nested destructuring, rest patterns, destructuring assignment e pattern matching estrutural continuam fora da V1.

### Continuação de comparações com sujeito único

Uma cadeia unida por `and` pode omitir a repetição do mesmo sujeito em comparações consecutivas:

```
value >= 0 and <= 100
value is Int and >= 0 and <= 100
```

O sujeito é avaliado exatamente uma vez. Conceitualmente:

```
$temp = value
$temp is Int and $temp >= 0 and $temp <= 100
```

- A V1 restringe essa elipse a cadeias por `and` e aos operadores `==`, `!=`, `<`, `<=`, `>`, `>=` e `is`.
- A forma não se estende a `or` na V1, evitando grupos cujo sujeito fique ambíguo durante leitura.
- A regra é geral e não pertence especificamente a `invariant()`.

### Strings: interpolação, concatenação e construção incremental

A V1 mantém `String + String` como concatenação explícita e mantém interpolação como forma preferencial para texto humano com valores embutidos:

```
let full = first + " " + last
let greeting = f"Hello {full}"
```

Não há coerção implícita de outras categorias para `String` via `+`. Para coleções, bibliotecas devem preferir `join`; para construção incremental grande ou em loops, a Standard Library poderá expor um builder dedicado. Interpolação e concatenação são operações complementares, não aliases.

### DSL-like libraries — direção V1

HTML, CSS, UI, scenes, config e APIs declarativas devem usar **funções + trailing blocks + builders explícitos** como primeira arquitetura:

```
html do page
    page.head do head
        head.title("Aipo")
    end

    page.body do body
        body.h1("Hello")
    end
end
```

- O builder explícito preserva resolução lexical simples, autocomplete e diagnostics previsíveis.
- Receiver/contexto implícito de DSL e block expressions gerais continuam fora da V1 enquanto não houver caso real que justifique o custo.
- Funções comuns criadas pelo usuário podem compor builders; não há macro system necessário para esta camada.

### Forma condicional de valor — ainda aberta

A existência de uma expressão condicional curta continua aprovada como necessidade, mas **a grafia final** (`if ... then ... else ...` versus `condition ? value : value`) permanece em revisão junto com a grafia do fallback de `Failure`. Nenhuma das duas formas é canonizada por este fechamento.

## Fechamento do modelo numérico/binário — 2026-09-10

- `Int` deixa de ser normativamente signed 64-bit completo e passa a representar exatamente o intervalo **-9_007_199_254_740_991 .. 9_007_199_254_740_991** (`±(2^53 - 1)`), preservando equivalência exata entre VM e JavaScript.
- A VM pode continuar usando `i64` internamente; a restrição é semântica, não necessariamente física.
- `Float` permanece IEEE 754 binary64 finito.
- `Byte` entra na V1 como valor inteiro explícito `0..255`; conversões para `Byte` são verificadas e nunca fazem wraparound/saturação silenciosa.
- `Int("...")`, `Float("...")` e `Byte("...")` são conversões explícitas que podem produzir `Failure`; `or_else` é a recuperação idiomática.
- `String(value)` continua conversão textual explícita; interpolação continua preferida para composição textual humana.
- `Bytes` entra como bloco binário mutável e gerenciado, separado de `String`.
- Formatos `i8/u8/i16/u16/i32/u32/i64/u64/f32/f64` pertencem às APIs de `Bytes`/packing e **não** são tipos fundamentais cotidianos da VM.
- Leituras compactas convertem para `Int`/`Float`/`Byte`; valores `i64/u64` fora do intervalo de `Int` produzem `Failure` na V1.
- `String.encode()` usa UTF-8; `Bytes.decode()` valida UTF-8 e retorna `String` ou `Failure`.
- Packing multi-byte deve possuir ordem de bytes definida pela API e não depender implicitamente da endianness nativa da máquina.

**Princípio normativo:** a superfície numérica continua pequena; precisão de armazenamento, protocolos e games é expressa em `Bytes`, sem transformar Aipo em uma linguagem de tipos numéricos de systems programming.