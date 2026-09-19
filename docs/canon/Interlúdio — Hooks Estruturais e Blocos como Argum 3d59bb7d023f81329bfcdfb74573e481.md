# Interlúdio — Hooks Estruturais e Blocos como Argumentos

<aside>
✅

**Decisões fechadas:** `struct` permanece focada em dados/shape; `init()` e `invariant()` são hooks estruturais especiais dentro de `impl Type`; `self!` continua sendo a forma canônica de receiver mutável; chamadas podem receber um trailing block `do ... end`, semanticamente equivalente a uma closure passada como último argumento.

</aside>

## Princípio geral

A Aipo deve adicionar construções especiais apenas quando elas representam uma fase estrutural automática da vida de um valor ou quando são açúcar sintático para uma semântica que a linguagem já possui.

Esse princípio evita acumular métodos mágicos, mini-DSLs e novos modelos de execução.

## `struct` e `impl`: separação consolidada

A divisão canônica passa a ser:

```
struct Player
    fixed id
    name
    health = 100
end

impl Player
    init(id, name, health = 100)
        self.id = id
        self.name = name
        self.health = health
    end

    invariant()
        self.id is Int
        self.name is String
        self.health is Int
        self.health >= 0
        self.health <= 100
    end

    fn alive(self)
        return self.health > 0
    end

    fn damage(self!, amount)
        self.health -= amount
    end
end
```

A leitura estrutural é deliberada:

- `struct`: quais dados existem;
- `init()`: como o estado inicial nasce;
- `invariant()`: o que precisa continuar verdadeiro;
- `fn`: o que o valor sabe fazer por comportamento associado.

## `init()` e `invariant()` como hooks especiais

`init()` e `invariant()` pertencem a `impl Type`, mas não são funções comuns.

### `init()`

- existe no máximo uma vez por `struct`;
- é usado pela construção `Type{...}` quando declarado; `()` permanece reservado a chamadas;
- pode inicializar `self` durante a fase de construção;
- não é chamável por dot-call;
- não é um valor de função e não pode ser capturado/referenciado como closure.

### `invariant()`

- é opcional e existe no máximo uma vez por `struct`;
- não recebe parâmetros;
- usa `self` implicitamente como receiver somente leitura;
- não é chamável pelo programador;
- não produz valor utilizável;
- contém condições que devem permanecer verdadeiras;
- é verificado ao final da construção e nos pontos protegidos de mutação posteriores;
- uma mutação protegida que viola o invariant não deve publicar o novo estado.

A forma vive em `impl` para manter toda lógica associada usando `self.member`, eliminando a exceção anterior em que o bloco dentro da própria `struct` referenciava campos nus.

### Segurança progressiva e tipagem dinâmica

Mesmo numa eventual variante mais dinâmica da Aipo, `invariant()` pode funcionar como segurança opt-in:

```
impl Range
    invariant()
        self.min is Int
        self.max is Int
        self.min <= self.max
    end
end
```

Assim, a `struct` descreve shape e o autor escolhe quando reforçar propriedades de tipo ou domínio. O `invariant()` não deve virar uma mini-DSL de contratos: usa expressões normais da linguagem, incluindo `is`.

### Teste `is` agrupado — decisão consolidada

A forma múltipla é açúcar geral da linguagem:

```
self.id, self.health is Int
```

equivale a:

```
self.id is Int and self.health is Int
```

As expressões são avaliadas da esquerda para a direita e uma única vez cada. A regra vale fora de `invariant()` também; não é sintaxe exclusiva do hook.

## Limite de hooks especiais

Para a V1, a direção é manter apenas:

- `init()` — nascimento/construção;
- `invariant()` — validade persistente.

Operações como `clone`, `copy`, `equals`, `hash`, `string`, `iter`, `serialize` ou `call` devem permanecer funções normais, interfaces ou bibliotecas, e não hooks mágicos.

Um eventual hook de destruição (`deinit()` ou equivalente) fica fora da V1. Como a Aipo usa runtime gerenciado, finalização ligada ao GC seria não determinística e inadequada para recursos como arquivos, sockets e locks. Recursos determinísticos devem ser estudados separadamente por APIs explícitas ou futuros blocos de escopo.

## `self!` permanece canônico

Aipo mantém:

```
fn inspect(self)
    ...
end

fn update(self!, value)
    ...
end
```

`self!` significa permissão de mutação através do caminho do receiver. Não significa ownership, referência explícita ou deep mutability.

A alternativa verbal `mut self` continua desnecessária para a Aipo; `var self` é evitado porque `var` já descreve a mutabilidade de bindings, não o modo de acesso de um receiver.

## Trailing blocks — decisão fechada

Chamadas podem receber uma closure como último argumento usando `do ... end`.

### Sem parâmetros

```
transaction() do
    save(user)
    save(order)
end
```

### Um parâmetro

```
file.use("config.txt") do file
    process(file.read())
end
```

### Múltiplos parâmetros

```
items.each_pair() do key, value
    io.print(key, value)
end
```

### Dessugaring conceitual

```
run_task("build") do task
    task.start()
end
```

é açúcar para uma forma equivalente a:

```
run_task(
    "build",
    fn(task)
        task.start()
    end
)
```

O trailing block não cria um segundo tipo de função. É uma closure lexical normal passada como último argumento.

## Regras semânticas dos blocos

- `do` abre explicitamente o bloco e `end` o fecha;
- indentação continua não semântica;
- parâmetros após `do` são parâmetros normais da closure;
- captura lexical segue exatamente as regras das closures comuns;
- `return` dentro do trailing block retorna da closure, não da função externa;
- não há `yield` especial nem non-local return;
- a chamada normal e a forma com trailing block devem compartilhar o mesmo modelo de resolução de argumentos;
- o formatter deve manter o bloco visualmente associado à chamada.

## Por que `do` é preferido

A forma abaixo foi rejeitada como canônica:

```
builder.section()
    ...
end
```

Sem um marcador de abertura, o parser precisaria inferir que uma chamada comum iniciou um bloco, aumentando ambiguidades e dependência de contexto. `do` mantém o início do bloco explícito e combina com a tradição `... end` já existente na Aipo.

Sintaxes com delimitadores extras para parâmetros, como `do |item|`, também são evitadas; a forma textual `do item` preserva a preferência da linguagem por poucos símbolos.

## Implementação no frontend

O parser pode inicialmente representar:

```
foo(a, b) do value
    body
end
```

como uma chamada com `trailing_block`. Uma etapa de lowering a transforma em uma chamada normal cujo último argumento é uma `FunctionLiteral`.

```
Call
├── callee: foo
├── args: a, b
└── trailing block
```

passa conceitualmente para:

```
Call
├── callee: foo
└── args
    ├── a
    ├── b
    └── FunctionLiteral(value)
```

Depois desse lowering, VM, bytecode, interpretador e backends não precisam manter uma categoria semântica especial para `do`.

## Possibilidades abertas pela feature

Trailing blocks são mecanismo geral de composição de bibliotecas, não uma feature de web. Possíveis usos incluem:

- builders e DSLs declarativas;
- transações;
- escopos de recursos;
- testes;
- configuração;
- UI desktop;
- árvores de cena;
- pipelines;
- tarefas/concorrência futura;
- frameworks web;
- APIs que precisem executar comportamento dentro de um contexto controlado.

O critério de design é que bibliotecas forneçam a política enquanto a linguagem fornece apenas closures e a ergonomia do bloco.

## DSLs por biblioteca/framework

Uma biblioteca pode modelar uma DSL com `struct`, `impl`, funções e trailing blocks, sem macros e sem ensinar o compilador sobre o domínio.

Exemplo genérico:

```
ui.window("Settings") do window
    window.label("Volume")
    window.slider(volume)
end
```

O framework pode criar um builder, entregá-lo como parâmetro da closure e coletar os resultados. Essa abordagem preserva resolução explícita de nomes e evita receivers/contextos mágicos.

### Direção oficial para Web e DSL-like APIs

HTML, CSS, DOM/componentes e o backend JavaScript passam a ser **casos de uso oficiais da Aipo**, mas continuam fora da gramática do core. `struct` + `impl`, funções, closures e trailing blocks devem modelar builders de nós/regras e árvores declarativas.

A mesma estratégia vale para UI, scenes e config. O objetivo é tornar a Aipo expansiva por bibliotecas sem criar macros complexas, receivers implícitos mágicos ou uma gramática especial para cada domínio.

## Critério de evolução

Antes de introduzir qualquer novo hook ou nova sintaxe de DSL, perguntar:

1. a necessidade não pode ser expressa por função, closure, `struct`, `impl` ou interface?
2. a nova forma reutiliza semântica existente ou cria um novo modelo mental?
3. a feature beneficia mais de um domínio?
4. o lowering pode apagar o açúcar cedo e manter o núcleo simples?
5. há código real suficiente demonstrando que o ganho de legibilidade supera o custo de linguagem?

**Princípio consolidado:** a Aipo deve ser pequena no núcleo e expansiva por composição. Hooks especiais representam ciclo de vida estrutural; trailing blocks dão às bibliotecas poder para criar APIs declarativas sem transformar cada domínio em sintaxe nativa.