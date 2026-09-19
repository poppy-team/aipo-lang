# Interlúdio — Fixed, Visibilidade, Mutabilidade e Escopo

<aside>
🧭

Objetivo: separar quatro conceitos que parecem próximos, mas têm papéis diferentes na Aipo: **visibilidade**, **escopo**, **mutabilidade do caminho de acesso** e **estabilidade estrutural de um campo com `fixed`**.

</aside>

## Modelo mental

- `export` responde **quem pode enxergar um símbolo através da fronteira de módulo/package**.
- Escopo lexical responde **onde um nome existe no código**.
- `let`/`var`, parâmetros comuns e `self!` respondem **se aquele caminho de acesso permite mutação**.
- `fixed` responde **se o slot de um campo pode ser substituído depois da construção**.
- `fixed` não controla visibilidade e não significa deep freeze.

## `fixed`

Forma canônica:

```
struct Player
    fixed id: Int
    name: String
    health: Int = 100
end
```

`fixed id: Int` recebe seu valor durante a fase de construção e não pode receber outro valor depois que a instância foi publicada. Quando a `struct` possui `init`, esse é o lugar natural para inicializar campos `fixed`:

```
struct Player
    fixed id: Int
    name: String
end

impl Player
    init(id: Int, name: String)
        self.id = id
        self.name = name
    end
end

var player = Player(1, "Ana")
player.id = 2       # erro: campo fixed
player.name = "Bia" # permitido se o caminho de acesso for mutável
```

Isso torna `fixed` simples de ler: **pode ser escrito enquanto a instância nasce; depois, o slot fica estável**.

### `fixed` não é `let`

`let` pertence ao binding:

```
let player = Player(id: 1, name: "Ana")
```

Esse binding não pode ser reatribuído e não oferece caminho mutável para o objeto.

`fixed`, em contraste, pertence à definição da `struct` e vale para aquele campo em qualquer instância, mesmo quando a instância é acessada por um binding `var` ou por `self!`.

### `fixed` não é deep freeze

```
struct Inventory
    fixed items: List = []
end
```

A identidade armazenada em `items` não pode ser substituída:

```
inventory.items = other_list # erro
```

Mas `fixed` por si só não congela o objeto `List` armazenado. Se o caminho de acesso permitir mutação, operações internas continuam seguindo as regras normais de mutabilidade:

```
inventory.items.add(item)
```

Na V1, invariants não observam estado mutável profundo de `List`, `Dict` ou outra identidade gerenciada alcançada por alias; isso evita rastreamento oculto de aliasing.

## Visibilidade

Aipo V1 não possui `public`, `private` ou `protected` por membro. A fronteira pública é declarada por `export`.

```
struct Player
    fixed id: Int
    name: String
end

fn create_player(name: String) -> Player
    return Player(id: next_id(), name: name)
end

export Player, create_player
```

Se `Player` está acessível através da API, seus campos declarados também são acessíveis. Aipo trata `struct` como dado transparente, não como classe com encapsulamento por campo.

`fixed` não torna um campo privado; apenas impede a substituição daquele campo.

Namespaces importados são somente leitura como caminho de acesso. A mutabilidade interna do módulo deve ser exposta deliberadamente por operações apropriadas, em vez de permitir que outro módulo reatribua bindings do namespace importado.

## Escopo lexical

O escopo da Aipo é lexical. Um nome passa a existir no ponto da declaração e permanece até o fim do bloco correspondente.

```
fn example()
    let outer = 10

    if ready
        let inner = 20
        io.print(outer)
        io.print(inner)
    end

    io.print(outer)
    io.print(inner) # erro: inner saiu do escopo
end
```

Redeclarar o mesmo nome no mesmo escopo é erro:

```
let score = 10
let score = 20 # erro
```

Shadowing em bloco interno é permitido:

```
let name = "outer"

if ready
    let name = "inner"
    io.print(name)
end

io.print(name)
```

Tooling pode avisar sobre shadowing confuso, mas a construção é válida.

## Escopos especiais

### Parâmetros e `self`

Parâmetros pertencem ao escopo da função. Em `impl`, `self` é o receiver explícito e também pertence ao escopo da função:

```
impl Player
    fn rename(self!, name: String)
        self.name = name
    end
end
```

`self!` concede acesso mutável através do receiver; não ignora `fixed` nem invariants.

### `each`

Bindings introduzidos por `each` pertencem somente ao corpo do loop. Cada iteração cria bindings próprios, o que também define corretamente a captura por closures.

```
each player in players
    io.print(player.name)
end
```

### `failed`

O nome após `failed` é um binding local somente leitura da falha capturada:

```
attempt
    save()
failed err
    io.print(err.message)
end
```

`failed _` descarta explicitamente o objeto de erro.

### `invariant`

Dentro do bloco `invariant`, os campos da `struct` podem ser referenciados diretamente:

```
struct Player
    health: Int = 100

    invariant
        health >= 0
        health <= 100
    end
end
```

As linhas são condições Boolean que precisam permanecer verdadeiras nos pontos de validação definidos pela semântica da struct.

## Quatro eixos, sem sobreposição

| Recurso | Pergunta que responde |
| --- | --- |
| `export` | Quem pode enxergar isso fora da fronteira pública? |
| escopo lexical | Onde esse nome existe? |
| `let` / `var` / parâmetro / `self!` | Este caminho permite mutação? |
| `fixed` | Este campo pode ser substituído após a construção? |

Princípio pedagógico: **visibilidade, escopo, mutabilidade e identidade são conceitos separados; a Aipo evita usar uma única keyword para significar mais de uma dessas coisas.**