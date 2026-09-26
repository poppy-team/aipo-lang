# Sintaxe & Tipos de Dados

O Aipo é dinamicamente e fortemente tipado. Toda variável armazena um valor com tipo concreto, e operações entre tipos não realizam coerções implícitas perigosas.

---

## Variáveis e Imutabilidade

O Aipo distingue expressamente ligações imutáveis de variáveis mutáveis:

- **`let` (Imutável)**: Define uma constante local. Uma vez atribuído, o identificador não pode receber um novo valor.
- **`var` (Mutável)**: Define uma variável que pode sofrer reatribuição (`=`, `+=`, `-=`, etc.).

```aipo
# Ligação imutável: tentativa de reatribuição gera erro estático
let linguagem = "Aipo"

# Variável mutável: pode ser alterada ao longo da execução
var contador = 0
contador += 1
contador = contador * 2
io.println(contador) # 2
```

---

## Tipos Primitivos

### Inteiros (`Int`)
Inteiros de 64 bits assinados (`i64`). Suporta notação decimal, hexadecimal (`0x`), binária (`0b`) e octal (`0o`), além de separadores visuais com sublinhado (`_`).

```aipo
let decimal = 42
let hex = 0x2A
let bin = 0b101010
let milhao = 1_000_000
```

### Decimais (`Float`)
Números de ponto flutuante de precisão dupla (64 bits, IEEE 754).

```aipo
let pi = 3.14159
let taxa = 0.05
```

### Booleanos (`Bool`)
Valores lógicos puros (`true` e `false`).

```aipo
let ativo = true
let pronto = false
```

### Texto (`String`)
Strings UTF-8 canônicas com normalização Unicode NFC automática nas fronteiras de construção. Suporta interpolação formatada com o prefixo `f"..."` e concatenação com `+`:

```aipo
let versao = "0.1.0"
let mensagem = f"Bem-vindo ao Aipo v{versao}!"
io.println(mensagem)
```

### Nulo (`None`)
Representa a ausência explícita de valor (`none`).

```aipo
let opcional = none
```

---

## Coleções

### Listas (`List`)
Vetores dinâmicos ordenados indexados a partir de 0. Mutações em listas utilizam o método `.add()` (não `push`):

```aipo
var itens = [1, 2, 3]
io.println(itens[0]) # 1

# Adicionar elemento ao final
itens.add(4)
io.println(itens.len()) # 4

# Checar pertinência
io.println(itens.contains(3)) # true
```

Operações de alta ordem disponíveis na stdlib e VM: `map`, `flat_map`, `reduce`, `any`, `all`, `filter`, `sort`, `sort_by`.

### Dicionários (`Dict`)
Mapas chave-valor indexáveis que preservam a ordem original de inserção. A verificação canônica de existência é realizada por `.has(chave)`:

```aipo
let config = {
    "host": "localhost",
    "port": 8080,
}

if config.has("port")
    io.println(f"Conectando em porta: {config[\"port\"]}")
end
```

::: tip 💡 Por que não existe `dict.get()`?
O Aipo adota a decisão arquitetural canônica (ADP-001) de eliminar um método ambíguo `get()`, que não conseguiria distinguir "chave ausente" de "chave cujo valor é `none`". Em vez disso, utilize `dict.has(chave)` para verificação explícita e `dict[chave]` para acesso direto.
:::

### Conjuntos (`Set`)
Coleções de valores únicos que mantêm a ordem de inserção do primeiro registro:

```aipo
var s = Set()
s.add("alpha")
s.add("beta")
s.add("alpha") # Ignorado: duplicatas são descartadas

io.println(s.has("beta")) # true (utiliza .has(), não .contains())
io.println(s.len())        # 2
```

### Sequências Lazy (`Sequence`)
Geradores iteráveis avaliados sob demanda sem alocar coleções intermediárias completas na memória. Criadas chamando `.lazy()` sobre coleções:

```aipo
let seq = [1, 2, 3, 4, 5].lazy()
    .filter(x => x % 2 != 0)
    .map(x => x * 10)

let resultado = seq.to_list()
io.println(resultado) # [10, 30, 50]
```

---

## Manipulação de Bytes (`Bytes`)

Vetores de bytes contíguos de alta performance, projetados para parsing de protocolos de rede e formatos binários:

```aipo
let b = Bytes(16) # Aloca 16 bytes inicializados em zero
b.write_u32_le(0, 42)
let val = b.read_u32_le(0)
io.println(val) # 42
```

---

## Operadores & Precedência

- **Aritméticos**: `+`, `-`, `*`, `/`, `//` (divisão inteira truncada), `%` (módulo)
- **Comparação**: `==`, `!=`, `<`, `<=`, `>`, `>=`
- **Lógicos**: `and`, `or`, `not`
- **Navegação Segura**: `?.` (avalia o operando esquerdo; se for `none`, não executa o acesso nem os argumentos da chamada)
- **Operador de Fallback Lazy**: `or_else` (avalia o lado direito apenas se o lado esquerdo falhar ou for nulo)
- **Operador Pipe**: `|>` para encadeamento de chamadas:

```aipo
let resultado = valor
    |> normalizar
    |> validar
    |> salvar
```

