# Aipo V1 — Language Reference

<aside>
📘

**Referência normativa consolidada da Aipo V1.** Este documento descreve a linguagem como ela deve ser lida e usada, sem histórico de exploração. Quando material histórico divergir desta referência, prevalecem esta página, a sintaxe canônica vigente e decisões posteriores explicitamente aprovadas.

</aside>

<aside>
🧭

**Escopo desta referência:** somente regras já fechadas. Detalhes ainda em aberto não são promovidos silenciosamente a norma. A gramática formal, o modelo semântico formal e a suíte de conformidade serão documentos P0 complementares.

</aside>

## 1. Modelo da linguagem

Aipo V1 é uma linguagem **dinâmica e fortemente tipada, com contratos opcionais restritos às assinaturas de funções e análise estática oportunista**.

- Tipos pertencem aos valores; bindings e campos permanecem dinamicamente tipados.
- Parâmetros e retornos podem declarar contratos opcionais; código sem anotações continua plenamente válido e idiomático.
- Não há classes nem herança.
- `struct` descreve dados/shape; `impl` organiza comportamento associado e os hooks estruturais especiais `init()` e `invariant()`.
- Funções são valores de primeira classe e suportam closures lexicais.
- `impl` organiza comportamento associado sem criar um segundo conceito de método.
- `none`, falha recuperável e ausência de valor de retorno são conceitos distintos.
- O runtime é gerenciado; ownership, borrowing, allocator e coleta de memória não fazem parte do modelo mental normal do usuário.

Documentos relacionados: [Aipo V1 — Sintaxe Canônica Consolidada](Aipo V1 — Sintaxe Canônica Consolidada 3d59bb7d023f8184b297c100cde50c68.md) e [Governança de Design e Evolução da Aipo](Governança de Design e Evolução da Aipo 3d59bb7d023f8162a363c89994b8caca.md).

## 2. Forma lexical e estilo canônico

- Arquivos-fonte usam a extensão `.aipo` e texto Unicode válido.
- A linguagem é case-sensitive.
- Identificadores aceitam Unicode, são normalizados em NFC, podem conter `_` e podem conter números após o primeiro caractere.
- `_` isolado é reservado para usos especiais da linguagem, como descarte explícito.
- Convenção canônica: `snake_case` para bindings, funções, parâmetros e campos; `PascalCase` para structs, interfaces e tipos.
- Comentários comuns começam com `#` e seguem até o fim da linha.
- Não há comentário de bloco na V1.
- Blocos explícitos terminam com `end`.
- Indentação não é semântica; o formatter canônico usa quatro espaços por nível.
- Nova linha normalmente encerra uma instrução.
- `;` não faz parte da V1.
- Uma expressão pode continuar dentro de `()`, `[]` e `{}` ou quando a linha termina sintaticamente incompleta.
- Não existe `\\` de continuação explícita de linha.

## 3. Valores fundamentais e identidade

### Valores com semântica de valor

- `none`
- `Bool`
- `Int`
- `Float`
- `Byte`
- `String`

Esses valores não expõem identidade física como parte da semântica pública.

### Valores gerenciados com identidade

- instâncias de `struct`;
- `List`;
- `Dict`;
- `Bytes`;
- `Function` e `Closure`.

Atribuir ou passar um valor gerenciado compartilha a mesma identidade; não há deep copy automático.

```
var a = [1, 2]
let b = a

a.add(3)
io.print(b) # observa a mesma List
```

`copy(value)` cria uma cópia superficial explícita. Valores gerenciados contidos continuam compartilhados.

## 4. Tipos, inferência e contratos de assinatura

Aipo mantém um **núcleo dinâmico**. Bindings `let`/`var` e campos de `struct` não aceitam contratos declarativos de tipo na V1. O compilador pode observar/refinar tipos pelo fluxo, mas inferência não cria um contrato persistente.

Contratos opcionais existem apenas em assinaturas:

```
fn distance(a: Point, b: Point) -> Float
    return calculate_distance(a, b)
end
```

- `name: Type` restringe opcionalmente um parâmetro.
- `-> Type` restringe opcionalmente o valor de sucesso retornado.
- Um parâmetro sem anotação permanece dinamicamente irrestrito.
- Ausência de `-> Type` significa ausência de contrato declarado de retorno, não `Void`.
- Contratos são verificados na entrada/saída da função quando o compilador não puder provar a compatibilidade antecipadamente.
- Uma incompatibilidade comprovável é diagnóstico antes da execução; uma violação descoberta somente em runtime é **contract fault** de programação e não é capturável por `attempt`.
- Em parâmetro mutável tipado, a forma canônica é `name!: Type`: `!` concede permissão de mutação pelo caminho e `: Type` restringe o valor.
- `init(...)`, funções anônimas, interfaces e parâmetros de trailing blocks usam o mesmo mecanismo de contratos opcionais.
- Tipos inferidos servem ao analisador/tooling; contratos escritos são promessas estáveis da API.
- `Any`, contratos em bindings/campos, unions arbitrárias, intersections e generics de usuário não fazem parte da V1.
- `List` e `Dict` podem ser usados como categorias simples de assinatura; contratos profundos `List[T]`/`Dict[K, V]` ficam adiados.
- Não há coerção implícita ampla entre categorias diferentes de valores.

### Optional

`T?` significa exatamente **`T` ou `none`** em contratos de assinatura e pode ser usado como expressão de tipo em testes `is`. Não cria wrapper `Optional`/`Option`/`Maybe`; `T??` é inválido e unions arbitrárias continuam fora da V1. Em `value is T?`, o narrowing resultante permanece `T | none`, não apenas `T`.

```
fn find_player(id: Int) -> Player?
    ...
end

if value is Player?
    ...
end
```

`none` é um valor normal, não uma falha e não é falsey. Condições exigem `Bool`.

### Teste de tipo e interface

```
value is Player
value is Drawable
```

`is` testa o tipo concreto ou conformidade estrutural com uma interface e sempre produz `Bool`; incompatibilidade produz `false`, nunca falha. O analisador pode fazer narrowing flow-sensitive enquanto o fluxo justificar a informação, sem criar contrato persistente.

A forma múltipla `a, b, c is T` é açúcar geral para `a is T and b is T and c is T`; as expressões são avaliadas da esquerda para a direita e uma única vez cada. A negação canônica é `not value is T`, equivalente a `not (value is T)`; `is not` não é alias.

### Presença e navegação segura

`some(value)` é equivalente conceitualmente a `value != none` e também pode ser usado por dot-call:

```
if player.some()
    player.draw()
end
```

A navegação segura usa exclusivamente `?.` para ausência por `none`:

```
let name = player?.name
let result = service?.load()
```

Se o receiver for `none`, o acesso/chamada não ocorre e a expressão produz `none`.

## 5. Números

- `Int` representa exatamente inteiros no intervalo **-9_007_199_254_740_991 .. 9_007_199_254_740_991** (`±(2^53 - 1)`). A VM Rust pode armazená-lo internamente em `i64`, mas valores fora desse intervalo não pertencem à semântica pública de `Int`. Essa regra preserva equivalência exata com o backend JavaScript sem exigir `BigInt` no código normal.
- `Float` usa IEEE 754 binary64, mas somente valores finitos são válidos na semântica pública.
- `Byte` é um inteiro compacto de valor com intervalo **0..255**, destinado principalmente a dados binários, imagens, cores, áudio, rede e buffers. `Byte(value)` é conversão explícita e verificada; valores fora do intervalo não fazem wraparound nem saturação silenciosa.
- A única promoção numérica implícita é `Int -> Float` em operações numéricas mistas. `Byte` pode ser promovido para `Int` quando participa de aritmética numérica comum; reduzir para `Byte` continua explícito.
- `Float -> Int` é explícito por `Int(value)` e trunca em direção a zero; `Float(value)` pode converter explicitamente `Int`/`Byte` quando desejado.
- `Int(value)`, `Float(value)` e `Byte(value)` também podem receber `String`; texto inválido ou valor fora do intervalo produz `Failure`, permitindo recuperação com `or_else`.
- Não há coerção automática entre números e `String`, `Bool` ou outras categorias; `Bool` não pertence à família numérica.
- `/` é divisão real e retorna `Float`.
- `div` é divisão inteira e trunca em direção a zero.
- `%` é o resto associado ao mesmo quociente de `div`.
- Overflow/range overflow inteiro e resultados `Float` não finitos geram falha/fault numérico conforme a operação, nunca wraparound ou saturação silenciosa.

```
5 / 2      # 2.5
5 div 2    # 2
-5 div 2   # -2
-5 % 2     # -1
```

`Bool` não pertence à família numérica; `true` e `1` são categorias distintas.

### Conversões explícitas

```
let port = Int("8080") or_else 8080
let speed = Float("4.5") or_else 1.0
let alpha = Byte("255") or_else Byte(255)
let label = String(port)
```

`String(value)` continua sendo a conversão textual explícita; interpolação `f"...{value}..."` permanece a forma preferida para composição de texto humano.

## 6. Strings e bytes

`String` é imutável, Unicode, normalizada em NFC e usa UTF-8 internamente.

- Operações textuais normais trabalham com code points, não bytes crus.
- `text[i]` indexa por code point e retorna `String`; não há `Char` fundamental na V1.
- `len(text)` conta code points.
- `byte_len(text)` expõe o comprimento UTF-8 em bytes.
- Índices negativos são permitidos.

### Literais

```
"texto"
f"Olá, {name}"
r"C:\\Users\\name"
fr"raw + {value}"

"""texto
multilinha"""
```

- `f` ativa interpolação.
- `r` desativa escapes de backslash.
- `fr` é a ordem canônica combinada; `rf` não é alias.
- Em f-strings, `{{` e `}}` produzem chaves literais.
- Strings comuns tratam `{...}` como texto literal.

### `Bytes`

`Bytes` é um bloco binário mutável e gerenciado, separado semanticamente de `String`.

- indexação trabalha por byte e produz `Byte`;
- `String.encode()` produz bytes UTF-8;
- `bytes.decode()` valida UTF-8 e produz `String` ou `Failure`;
- operações de leitura/escrita compacta usam formatos de armazenamento, não novos tipos numéricos cotidianos: `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `f32` e `f64`;
- leituras `i8..u32` produzem `Int`; `f32`/`f64` produzem `Float`; `u8` pode ser obtido como `Byte` quando a API específica pedir byte;
- `read_i64`/`read_u64` só produzem `Int` quando o valor decodificado cabe no intervalo público de `Int`; caso contrário produzem `Failure`. `write_i64`/`write_u64` podem armazenar qualquer `Int` Aipo, mas a V1 não introduz `Int64`, `UInt64` nem `BigInt` como tipos cotidianos;
- leituras/escritas fazem bounds checking; acesso fora da área válida é fault de programação, não wraparound;
- endianness é propriedade explícita da API de packing quando relevante; a V1 não depende da ordem nativa da plataforma.

```
let data = Bytes(32)
data.write_i32(0, score)
data.write_f32(4, position.x)
data.write_u8(8, alpha)

let score = data.read_i32(0)
let x = data.read_f32(4)
let alpha = data.read_u8(8)

let encoded = "Olá".encode()
let decoded = encoded.decode() or_else "invalid UTF-8"
```

**Princípio:** `Int`/`Float`/`Byte` definem a semântica de programação; `i8/u8/.../f32/f64` definem formatos compactos de armazenamento e interoperabilidade.

## 7. Bindings e mutabilidade

### `let`

Cria um binding somente leitura pelo caminho de acesso.

```
let player = Player{1, "Ana"}
```

O binding não pode ser reatribuído nem usado como caminho mutável. Isso **não** congela globalmente uma identidade compartilhada.

### `var`

Permite reatribuição e mutação pelo caminho.

```
var player = Player{1, "Ana"}
player.health = 80
```

### Parâmetros mutáveis

Parâmetros são somente leitura por padrão. O sufixo `!` concede acesso mutável pelo caminho daquele parâmetro.

```
fn reset(player!)
    player.health = 100
end
```

`!` é permissão de mutação pelo caminho; não é ownership, referência explícita ou deep mutability.

## 8. Funções

A forma canônica de função nomeada é:

```
fn name(parameters) -> OptionalReturnContract
    ...
end
```

O contrato de retorno é opcional.

### Parâmetros default

```
fn greet(name: String, greeting: String = "Olá")
    io.print(greeting, name)
end
```

Parâmetros obrigatórios precedem parâmetros com default.

### Funções com e sem valor

Aipo distingue funções **com valor** e funções **sem resultado** sem expor `Void`/`Unit`.

`return` encerra a função ou closure sem produzir valor; `return expression` produz valor. `return` e `return none` são semanticamente distintos.

Uma função que não executa `return expression` em nenhum caminho é semanticamente uma **função sem valor de resultado**.

```
fn log(message)
    io.print(message)
end
```

Ela pode ser chamada como statement, mas não pode ser usada onde um valor é exigido.

`none` continua sendo um valor real. Para produzir ausência, a função deve retornar `none` explicitamente.

Se uma função possui qualquer `return expression` alcançável, ela é value-producing. Todos os caminhos normais alcançáveis devem produzir valor ou encerrar o fluxo (`fail`, retorno anterior ou fluxo comprovadamente não terminante); misturar `return` sem valor com `return expression` nessa função é erro.

Sem contrato `-> T`, uma função value-producing pode retornar categorias diferentes em ramos diferentes; essas unions pertencem apenas à análise interna/tooling. `fail(...)` encerra o caminho e não precisa satisfazer o contrato de retorno. Funções/closures carregam internamente um result kind `VALUE` ou `NO_RESULT`, sem expor esse mecanismo como tipo público.

### Funções anônimas

A mesma keyword `fn` cria funções anônimas:

```
let double = fn(value)
    return value * 2
end
```

Não existe uma segunda sintaxe `lambda`, `=>` ou equivalente na V1.

### Funções locais e closures

Funções podem ser declaradas dentro de funções. Funções locais passam a existir quando sua declaração é alcançada no fluxo local.

Closures capturam automaticamente os bindings lexicais realmente usados:

- a captura é do binding, não de uma fotografia arbitrária;
- `var` capturado permanece compartilhado e mutável;
- `let` capturado permanece somente leitura;
- não existem listas de captura, `nonlocal`, `move capture` ou modalidades explícitas na V1.

### Trailing blocks

Uma chamada pode passar uma closure como último argumento por meio de `do ... end`:

```
transaction() do
    save(user)
    save(order)
end

file.use("config.txt") do file
    process(file.read())
end
```

Parâmetros após `do` são parâmetros normais da closure. A forma é açúcar sintático para uma `fn(...) ... end` passada como último argumento; captura lexical, mutabilidade e `return` seguem exatamente as regras de funções anônimas comuns. `return` dentro do trailing block retorna da própria closure. Não há `yield` especial nem non-local return.

O objetivo é dar às bibliotecas um mecanismo geral para builders, transações, escopos de recursos, testes, configuração, UI, pipelines e DSLs sem introduzir sintaxe específica de domínio no core.

Detalhes e possibilidades: [Interlúdio — Hooks Estruturais e Blocos como Argumentos](Interlúdio — Hooks Estruturais e Blocos como Argum 3d59bb7d023f81329bfcdfb74573e481.md).

## 9. `struct`

`struct` declara dados/shape; não contém métodos nem hooks aninhados e seus campos não carregam contratos declarativos de tipo.

```
struct Player
    fixed id
    name
    health = 100
end
```

### Campos

- Campos são dinamicamente tipados.
- Garantias permanentes de categoria/estado pertencem a `invariant()`.
- Campos podem ter default.
- Campos obrigatórios devem preceder campos com default quando a construção automática é usada.
- Se a struct está acessível, seus campos declarados também são acessíveis; não há `public`, `private` ou `protected` por membro.
- `_name` é somente convenção de nome, não privacidade.

### Construção automática

Sem `init`, uma struct é construída com `Type{...}`.

```
let a = Player{1, "Ana"}
let b = Player{2, "Bia", health = 80}
```

- Argumentos posicionais seguem a ordem dos campos.
- Argumentos nomeados usam `name = value`.
- Posicionais vêm antes dos nomeados.
- Nomeados podem aparecer em qualquer ordem.
- Campos obrigatórios sem default precisam receber valor.
- `new` não faz parte da V1.

## 10. `init`, `fixed` e `invariant()`

`init()` e `invariant()` são hooks estruturais especiais declarados dentro de `impl Type`. Eles não são funções comuns chamáveis por dot-call nem valores de função.

### `init`

Quando a construção precisa de lógica explícita, declara-se no máximo um `init` dentro de `impl Type`.

```
impl Player
    init(id: Int, name: String, health: Int = 100)
        self.id = id
        self.name = name
        self.health = health
    end
end
```

- `Type{...}` usa `init` quando ele existe; a presença de `init(...)` não muda a superfície de construção.
- `init` não é chamável por dot-call.
- `self` representa a instância ainda em construção.
- Todos os campos obrigatórios precisam estar inicializados quando `init` termina.
- Defaults de parâmetros de `init` fazem parte da assinatura de construção.

### `fixed`

`fixed` impede que o valor armazenado naquele campo seja substituído depois da construção.

```
struct Entity
    fixed id
end
```

`fixed` não congela profundamente objetos gerenciados armazenados no campo.

### `invariant()`

`invariant()` é opcional, existe no máximo uma vez por `struct`, não recebe parâmetros e usa `self` como receiver somente leitura.

```
impl Player
    invariant()
        self.name != ""
        self.health >= 0
        self.health <= 100
    end
end
```

- É verificado ao final da construção e nos pontos definidos de mutação do estado protegido.
- As linhas equivalem conceitualmente a condições combinadas por `and`.
- O hook usa expressões normais da linguagem; testes `is` podem ser usados quando o autor quiser reforçar propriedades de tipo em runtime sem criar uma mini-DSL separada.
- Na V1, invariants não observam estado mutável profundo alcançado por alias de `List`, `Dict` ou outra identidade gerenciada.
- Uma atualização protegida segue conceitualmente: candidato → aplicação provisória → verificação → commit; se a validação falha, o estado anterior é preservado.
- Para a V1, `init()` e `invariant()` são os únicos hooks estruturais especiais. Operações como clone, hash, serialização, iteração ou representação textual permanecem funções/interfaces normais.

## 11. Comportamento associado e dot-call

`impl Type` é a forma canônica de organizar funções receiver-associated.

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

- `self` é receiver somente leitura.
- `self!` permite mutação pelo receiver.
- `impl` não cria classe, herança, método virtual ou dispatch OO clássico.
- Funções sem receiver não ficam dentro de `impl` na V1.
- Factories e construtores alternativos são funções normais de módulo.

Dot-call preserva a ergonomia:

```
player.damage(10)
```

sem transformar comportamento associado em um segundo modelo de função.

## 12. Interfaces e `satisfy`

Interfaces são contratos estruturais de comportamento e contêm apenas assinaturas.

```
interface Drawable
    fn draw(self)
end

interface Damageable
    fn damage(self!, amount: Int)
end
```

A conformidade é estrutural. `satisfy` é uma promessa/verificação explícita e opcional:

```
satisfy Player: Drawable, Damageable
```

- `satisfy` não injeta comportamento.
- Não cria herança ou mixin.
- A presença ou ausência de `satisfy` não muda o resultado runtime de `value is Interface`.
- Interfaces podem ser usadas como contratos nas posições tipadas da V1: **parâmetros e retornos de funções/assinaturas**. Bindings e campos permanecem sem contratos declarativos; contratos built-in profundos de coleção ficam fora da V1 inicial.
- Usar uma interface como contrato não envolve/wrappa o valor e não muda sua identidade.

### Compatibilidade de assinatura

A implementação deve aceitar toda chamada prometida pela interface.

- Nome da operação deve coincidir.
- Posição, mutabilidade e contratos dos parâmetros participam da compatibilidade; nomes dos parâmetros não.
- `self`/`self!` devem coincidir na V1.
- Interfaces não declaram valores default.
- Contratos explícitos de parâmetros e retorno devem coincidir exatamente na V1.
- Uma implementação pode acrescentar apenas parâmetros extras opcionais que não invalidem chamadas prometidas.

## 13. Igualdade, identidade e cópia

### Igualdade

`==` e `!=` comparam valor/conteúdo quando a categoria é comparável.

- Categorias claramente incompatíveis em `==` resultam em `false`, não fault.
- Igualdade numérica aplica a promoção `Int -> Float`; por isso `1 == 1.0` é `true`, sem apagar a diferença entre `Int` e `Float` para `is`.
- `List` usa igualdade estrutural recursiva.
- `Dict` usa igualdade estrutural independente da ordem das entradas.
- Structs são estruturalmente iguais quando têm o mesmo tipo e todos os campos declarados são iguais.
- Funções/closures não suportam igualdade estrutural na V1.

### Identidade

`same(a, b)` pergunta se dois valores gerenciados representam a mesma identidade. Só é válido para categorias com identidade gerenciada; usá-lo com `none`, `Bool`, `Int`, `Float` ou `String` é erro de uso.

Não existem hooks mágicos `equals()`/`hash()` na V1.

### Cópia

`copy(value)` é sempre cópia superficial explícita.

## 14. Coleções

### `List`

`List` é ordenada, dinâmica, mutável e possui identidade gerenciada.

```
let empty = []
var values = [10, 20, 30]
```

- Sem contrato, pode conter valores de categorias diferentes, inclusive `none`.
- Indexação começa em zero.
- Índices negativos contam a partir do fim.
- Índice fora da faixa gera `IndexError`.
- Atribuição por índice não cresce automaticamente a lista.
- Slicing usa `list[start..end]`, com final exclusivo e limites opcionais (`list[..end]`, `list[start..]`, `list[..]`).

A API detalhada de operações de coleção pertence à futura Standard Library Reference, não a esta referência do núcleo da linguagem.

### `Dict`

`Dict` é uma coleção associativa mutável com identidade gerenciada.

- Chaves V1 são `String`, `Int` ou `Bool`.
- `none`, `Float`, coleções, structs e funções/closures não são chaves V1.
- `none` pode ser valor e funções/closures podem ser armazenadas como valores.
- `dict[key]` exige chave existente; ausência de chave não retorna `none`, pois `none` pode ser um valor armazenado.
- `dict.has(key)` é a forma canônica de testar presença antes do acesso.
- A V1 inicial não define `get/find` ambíguo nem contratos profundos `Dict[K, V]`; `Dict` aparece apenas como categoria simples em assinaturas.

## 15. Ranges e controle de fluxo

### Ranges

`start..end` cria um range crescente **half-open**: inclui `start` e exclui `end`.

```
0..10 # 0 até 9
```

- Isso permite `0..len(items)` sem `- 1`.
- Range descendente implícito e stride/step sintático ficam fora da V1; direção/passo explícitos podem ser oferecidos por biblioteca futuramente.
- Slicing reutiliza a mesma semântica half-open: `items[1..4]`, `items[..3]`, `items[2..]` e `items[..]`.
- `List` e `String` aceitam índices negativos; slicing de `String` opera por Unicode code points.

### Controle de fluxo

Condições exigem `Bool`.

### `if` / `elif` / `else`

```
if health <= 0
    die()
elif health < 20
    warn()
else
    continue_playing()
end
```

Zero ou mais `elif`; `else` é opcional.

### Condicional inline

A forma curta é controle de fluxo de **uma única instrução por ramo**, não expressão de valor:

```
if ready then run()
if ready then run() else wait()
```

- `then` separa condição e instrução do ramo verdadeiro.
- `else` é opcional e, quando presente, recebe uma única instrução.
- A forma inline não abre bloco e não usa `end`.
- `elif` inline e conditional expression de valor ficam fora da V1.

### `match` / `when`

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

`match` é a única seleção múltipla da V1; `case` e `switch` não são aliases.

### `while`

```
while running
    tick()
end
```

### `loop`

```
loop
    tick()
end
```

Representa repetição indefinida explícita.

### `repeat`

```
repeat 3
    io.print("hello")
end
```

A contagem é avaliada uma vez e precisa resultar em `Int` não negativo. `repeat` não cria índice implicitamente.

### `each`

`each` é a forma canônica de iteração; `for` não permanece como alias.

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

Bindings introduzidos pelo loop pertencem ao corpo. Cada iteração possui bindings próprios capturáveis por closures.

### `break` e `continue`

`break` encerra o loop mais próximo; `continue` avança para a próxima iteração. Labels de loop ficam fora da V1.

## 16. Falhas recuperáveis e faults

Aipo separa **falhas recuperáveis** de **runtime faults de programação**.

Falhas recuperáveis não tratadas propagam automaticamente ao chamador. A assinatura de uma função não precisa declarar que ela pode falhar.

### Fallback de uma expressão

```
let config = load_config() or_else default_config()
```

`expression else fallback` trata a falha recuperável da expressão à esquerda. Runtime faults não são capturados.

### `attempt` / `failed`

```
attempt
    let config = load_config()
    start(config)
failed err
    io.print(err.message)
end
```

- `err` é binding local somente leitura.
- `failed _` descarta explicitamente o objeto da falha.
- `attempt` não é transacional: efeitos anteriores à falha não são revertidos automaticamente.

### `fail`

```
fail("message")
fail(err)
```

A primeira forma cria uma falha recuperável; a segunda repropaga a mesma falha.

Não existem aliases V1 como `throw`, `raise`, `try/catch` ou as formas históricas `try`/`or`.

## 17. Módulos

Cada arquivo `.aipo` define um módulo. O caminho físico em `src/` forma seu caminho lógico.

```
# game/player.aipo -> game.player
import game.player

let p = player.Player{1, "Ana"}
```

### `import`

A forma básica preserva namespace:

```
import game.player
player.create_player("Ana")
```

Import seletivo usa `:`:

```
import game.player: Player, create_player
```

Alias de módulo usa `as`:

```
import editor.player as editor_player
```

- `import module` traz o módulo como namespace; o nome local padrão é o último componente do caminho.
- `import module: name, name` traz explicitamente os nomes selecionados ao escopo local.
- `import module as alias` cria alias para o namespace do módulo.
- Aliases individuais dentro de import seletivo e `from ... import ...` ficam fora da V1.
- Namespaces importados são somente leitura pelo importador.
- Cada módulo é inicializado no máximo uma vez por execução.
- Dependências circulares entre módulos são proibidas na V1.

### `export`

Declarações de módulo são privadas por padrão. A API pública é explícita:

```
export Player, create_player, Drawable
```

`export` também realiza reexport:

```
import game.player, game.enemy
export player.Player, enemy.Enemy
```

Aliases públicos usam `as`:

```
export player.Player as GamePlayer
```

Não existe keyword separada `reexport`.

### Estrutura e execução do módulo

Aipo usa conceitualmente duas fases:

1. **estrutura:** `import`, `export`, `satisfy`, `fn`, `struct`, `interface` e `impl` são coletados/resolvidos sem depender da ordem textual;
2. **execução:** bindings `let`/`var`, expressões e statements top-level executam em ordem textual.

Código executável top-level é permitido; `fn main()` não é obrigatório pela linguagem.

## 18. Escopo lexical

- Um binding executável existe do ponto de declaração até o fim do bloco correspondente.
- Redeclarar o mesmo nome no mesmo escopo é erro.
- Shadowing em bloco interno é permitido.
- Parâmetros pertencem ao escopo da função.
- Bindings de `each` pertencem somente ao loop.
- Funções locais obedecem à ordem de execução local.
- Declarações estruturais de módulo obedecem ao modelo estrutural do módulo, não à ordem textual dos statements.

## 19. Visibilidade e namespaces

- Não existem `public`, `private` ou `protected` por membro.
- `export` controla a fronteira pública de módulo/package.
- Se uma struct é acessível, seus campos declarados também são acessíveis.
- Campos e funções associadas de um tipo compartilham o namespace público do tipo; colisões de nome são erro, não possuem regra de precedência.
- `fixed` controla substituição de campo, não visibilidade.

## 20. Recursos deliberadamente fora da V1

A ausência abaixo é parte do desenho da V1, não um esquecimento:

- classes e herança;
- `new`;
- overload de funções definido pelo usuário;
- generics definidos pelo usuário;
- aliases sintáticos redundantes como `for` para `each` ou `case` para `when`;
- métodos virtuais/dispatch OO clássico;
- getters/setters e propriedades mágicas como conceito separado;
- `Void`/`Unit` público apenas para representar função sem resultado;
- import seletivo no estilo `from ... import ...`;
- `try/catch`, `throw` e `raise` como modelo paralelo de falhas;
- comentários de bloco;
- `;` como separador de statements;
- listas explícitas de captura de closure;
- ownership/borrowing explícito e referências manuais no modelo normal da linguagem;
- `async`/`await`, tasks/channels e paralelismo fazem parte do **roadmap posterior à V1**, não da superfície V1 atual;
- sintaxe específica de HTML/CSS/UI/scene/config não entra no core: esses domínios devem reutilizar funções, closures e trailing blocks `do ... end` por biblioteca.

## 21. Exemplo integrado

```
export Player, Drawable, create_player

interface Drawable
    fn draw(self)
end

struct Player
    fixed id
    name
    health
end

satisfy Player: Drawable

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

    fn alive(self) -> Bool
        return self.health > 0
    end

    fn damage(self!, amount: Int)
        self.health -= amount
    end

    fn draw(self)
        io.print(f"{self.name}: {self.health}")
    end
end

fn create_player(id: Int, name: String) -> Player
    return Player{id, name}
end

var players = [
    create_player(1, "Ana"),
    create_player(2, "Bia")
]

each player in players
    player.draw()
end

attempt
    save(players)
failed err
    io.print(err.message)
end
```

## 22. Hierarquia normativa

Esta referência consolida o uso da Aipo V1. Ela deve permanecer sincronizada com:

- [Aipo V1 — Sintaxe Canônica Consolidada](Aipo V1 — Sintaxe Canônica Consolidada 3d59bb7d023f8184b297c100cde50c68.md) — superfície sintática consolidada;
- [Governança de Design e Evolução da Aipo](Governança de Design e Evolução da Aipo 3d59bb7d023f8162a363c89994b8caca.md) — regras de evolução da linguagem;
- a Especificação Viva — histórico, justificativas e decisões detalhadas ainda não promovidas a referência limpa.

<aside>
✅

**Regra editorial:** uma nova decisão só entra nesta Language Reference depois de estar fechada. Explorações e alternativas permanecem na Especificação Viva ou em ADPs até a aprovação.

</aside>

## Fechamento do Lote 3 — fluxo, loops, falhas e escopo

- `if` é construção de controle, não expressão; condições exigem `Bool` e cada ramo cria escopo lexical próprio.
- `match` avalia o alvo uma única vez, compara `when` por `==`, usa primeiro match, não possui fallthrough e mantém pattern matching estrutural/guards fora da V1.
- A V1 distingue `loop`, `while`, `repeat n` e `each`; `break`/`continue` atuam no loop mais próximo e labels/`break value` ficam fora.
- `repeat` avalia sua contagem uma vez e exige `Int >= 0`; não cria índice implícito.
- `each` cobre `List`, `Dict`, `String` e ranges. Cada iteração cria bindings frescos. Mutação estrutural da coleção atualmente iterada é runtime fault; substituição de elemento/valor existente pode ocorrer quando o caminho é mutável.
- `Failure` representa problemas esperados e recuperáveis; runtime faults representam violações de programação. Apenas `Failure` é capturável por `attempt`/`failed`.
- Falhas recuperáveis propagam automaticamente; `expression else fallback` trata uma expressão; `attempt`/`failed` trata um bloco; `fail(message)` cria Failure e `fail(err)` repropaga a mesma falha.
- `attempt` não é transacional e não realiza rollback automático.
- Escopo é lexical; redeclaração no mesmo escopo é erro e shadowing interno é permitido. O binding novo não existe durante seu próprio initializer.
- Closures capturam bindings: `var` permanece compartilhado/mutável, `let` readonly; bindings capturados sobrevivem enquanto forem necessários. Não há `nonlocal`, listas de captura ou `move capture` na V1.
- Funções locais passam a existir a partir da declaração no fluxo local e podem referenciar o próprio nome para recursão.

Documento de decisão: [Interlúdio — Fluxo, Loops, Falhas e Escopo](Interlúdio — Fluxo, Loops, Falhas e Escopo 3d69bb7d023f81d6903eccbcf98b6675.md).

**Status:** as quatro áreas estão fechadas para a V1.

## Fechamento do Lote 4 — chamadas, construção, `impl` e módulos

- Chamadas avaliam argumentos da esquerda para a direita, uma única vez. Posicionais precedem nomeados; defaults são avaliados a cada chamada e podem referenciar apenas parâmetros anteriores.
- Dot-call existe somente para funções associadas por `impl Type`; funções globais comuns não ganham dot-call automaticamente. Trailing blocks continuam sendo o último argumento.
- Construção de `struct` usa exclusivamente `Type{...}`. `()` permanece a superfície de chamada. Quando existe `init(...)`, `Type{...}` é validado contra sua assinatura; sem `init`, usa-se a assinatura automática dos campos.
- `fixed` pode receber exatamente uma atribuição durante construção e não pode ser substituído depois. `invariant()` é validado após construção e em fronteiras mutáveis estáveis, não após cada assignment interno.
- Múltiplos blocos `impl Type` são permitidos, mas cada operação é única por tipo e há no máximo um `init()` e um `invariant()` por `struct` no total.
- Interfaces permanecem estruturais e estritas; `satisfy` declara intenção e pede verificação antecipada, sem alterar representação ou dispatch.
- Um arquivo `.aipo` corresponde a um módulo na V1. Tudo é privado por padrão; `export` expõe nomes. Formas canônicas: `import module`, `import module: name, name` e `import module as alias`.
- Declarações estruturais de módulo são resolvidas no módulo inteiro; bindings executáveis seguem ordem textual. Inicialização top-level ocorre uma vez e o grafo de imports deve ser acíclico na V1.

Detalhes e racional: [Interlúdio — Chamadas, Construção, Impl e Módulos](Interlúdio — Chamadas, Construção, Impl e Módulos 3d69bb7d023f81a68cbde7a1f2d71297.md).

**Status:** as quatro áreas estão fechadas para a V1.

## Fechamento do Lote 5 — expressões, mutação, strings e callables

### Operadores e precedência

A V1 adota uma tabela pequena e fixa de precedência, da maior para a menor: acesso/chamada/indexação; unários numéricos `+`/`-`; multiplicativos `*`, `/`, `div`, `%`; aditivos `+`, `-`; range `..`; comparações `<`, `<=`, `>`, `>=`, `==`, `!=` e teste `is`; `not`; `and`; `or`; fallback recuperável `else`. Assignment não é expressão e fica fora dessa tabela.

- `and` e `or` fazem short-circuit, exigem operandos `Bool` e sempre produzem `Bool`; não retornam um dos operandos como em Python/Lua.
- `not` exige `Bool`. Pela precedência, `not value is T` significa `not (value is T)`.
- O fallback `a else b` avalia `b` somente se `a` terminar em `Failure`; runtime faults não ativam fallback. Cadeias são avaliadas da esquerda para a direita.
- Comparações encadeadas como `a < b < c` não fazem parte da V1; escreve-se `a < b and b < c`.
- `<`, `<=`, `>`, `>=` são definidos para a família numérica e para `String` por ordem lexicográfica de Unicode code points. Categorias incompatíveis são erro/fault; `==`/`!=` continuam sendo a comparação geral que retorna `false` para categorias claramente incompatíveis.
- Não há operator overloading definido pelo usuário na V1.

### Assignment e mutação

`=` é statement de atribuição, não produz valor e não pode ser encadeado. Targets válidos são bindings `var`, campos alcançados por caminho mutável e posições mutáveis de `List`/`Dict`.

- `let`, parâmetros comuns e `self` não podem ser usados como raiz de um caminho mutável; parâmetros `name!` e receiver `self!` concedem essa permissão.
- `fixed` só recebe valor uma vez durante construção e não pode ser substituído depois.
- `list[index] = value` exige índice existente e não cresce a lista; `dict[key] = value` insere ou substitui a entrada.
- A V1 inclui `+=`, `-=`, `*=`, `/=`, `div=` e `%=`. Compound assignment avalia o target/path exatamente uma vez, lê o valor atual, aplica a mesma operação binária e grava o resultado.
- `++`, `--`, assignment expressions e destructuring assignment ficam fora da V1.

### Strings, interpolação e conversão textual

`String` permanece imutável, Unicode/NFC, UTF-8 internamente e orientada a code points na API normal.

- `+` concatena somente `String + String` e produz uma nova `String`; não converte números, Bool, `none`, structs ou coleções automaticamente.
- `String(value)` é a conversão textual explícita canônica para `String`, `Int`, `Float`, `Bool` e `none`. Valores de usuário/coleções não ganham stringificação mágica na V1; representação customizada deve ser construída por função normal/biblioteca.
- Interpolação `f"...{expr}..."` aceita expressões completas; cada expressão é avaliada uma vez, da esquerda para a direita, e usa a mesma conversão textual explícita definida para os valores fundamentais. `{{` e `}}` produzem chaves literais.
- `r` desativa escapes de backslash; `fr` combina raw + interpolação e é a ordem canônica. Prefixos também podem ser usados com strings multilinha.
- Strings multilinha preservam seu conteúdo/newlines; a V1 não introduz dedent semântico automático, concatenação implícita de literais adjacentes ou operadores alternativos de formatação.

### `Function`, closures e chamada indireta

`Function` é a categoria built-in de valores chamáveis. Uma closure é semanticamente uma `Function` com ambiente lexical capturado; não existe um segundo protocolo de chamada.

- Funções nomeadas, funções locais, funções anônimas e closures podem ser armazenadas, passadas e retornadas como valores.
- `callback(...)` usa exatamente as mesmas regras de chamada, avaliação, argumentos nomeados/defaults e contratos que uma chamada direta.
- O contrato simples `Function` verifica apenas que o valor é chamável; a V1 não possui tipos de assinatura como `Function[Int -> String]`. Quando a identidade concreta da função é conhecida, o analyzer pode verificar aridade/contratos antecipadamente; caso contrário, incompatibilidade de chamada descoberta em runtime é fault de programação, não `Failure`.
- Contratos escritos na própria função/closure continuam ativos quando ela é passada como `Function`.
- Funções/closures não têm igualdade estrutural por `==`; `same(a, b)` pode observar sua identidade gerenciada. Referências repetidas à mesma função nomeada compartilham a identidade daquela declaração; cada avaliação de uma função anônima/closure cria nova identidade.
- A V1 não cria automaticamente bound-method values por `object.operation` sem chamada. Para passar comportamento receiver-associated como callback, usa-se uma closure explícita.
- Não há callable objects, hook `call`, overload de função nem signature generics na V1.

**Status:** as quatro áreas estão fechadas para a V1.

Detalhes e justificativas: [Interlúdio — Expressões, Mutação, Strings e Callables](Interlúdio — Expressões, Mutação, Strings e Callab 3d69bb7d023f810a8628cfe5ac111878.md).

## Adendo normativo — ergonomia V1 aprovada em 2026-09-10

As seguintes formas integram a V1 e devem ser refletidas em gramática, analisador, lowering e suíte de conformidade:

- `repeat count as index` expõe opcionalmente o contador zero-based; sem `as`, `repeat` continua não introduzindo binding.
- `callee do ... end` é permitido apenas como açúcar para `callee() do ... end` quando a chamada não possui argumentos.
- `|>` é pipeline de chamada e insere o valor esquerdo como primeiro argumento da operação à direita; não possui placeholder na V1 e deve baixar cedo para chamadas normais.
- `let [a, b] = value`, `var [a, b] = value` e `let {name, age} = value` introduzem destructuring superficial somente em bindings. Nested/rest/destructuring assignment/pattern matching permanecem fora.
- Cadeias de comparação por `and` podem continuar com o sujeito elidido, como `health is Int and >= 0 and <= 100`; o sujeito é avaliado uma vez e `or` não participa dessa elipse na V1.
- `String + String` permanece concatenação; interpolação `f"..."` é preferida para composição textual com valores; coerção implícita por `+` continua proibida.
- DSL-like APIs usam inicialmente builders explícitos + trailing blocks; receiver implícito de DSL e block expressions gerais permanecem fora da V1.

A **grafia da expressão condicional de valor** e a **grafia final do fallback de `Failure`** continuam abertas e serão fechadas em conjunto, pois compartilham operadores/precedência e impacto de leitura.

## Decisão posterior — condicional de valor e fallback de Failure

- Aipo V1 **não** terá operador ternário `?:`.
- A escolha condicional de valor reutiliza a forma textual `if condition then value else value`; quando usada em contexto de valor, `else` é obrigatório e ambos os ramos precisam produzir valor compatível com o contexto.
- `or` permanece exclusivamente operador Boolean.
- `or_else` é o fallback local canônico para `Failure`: `expression or_else fallback`. O fallback é lazy e só é avaliado quando a expressão à esquerda termina em `Failure`; runtime faults não ativam `or_else`.
- A antiga forma de fallback `expression else fallback` fica superseded por `or_else` para eliminar sobrecarga semântica de `else` fora de condicionais.