# Sintaxe & Tipos de Dados

O Aipo é dinamicamente e fortemente tipado. Toda variável armazena um valor com tipo concreto, e operações entre tipos não realizam coerções implícitas.

---

## Tipos Primitivos

### Inteiros (`Int`)
Inteiros de 64 bits assinados (`i64`). Suporta notação decimal, hexadecimal (`0x`), binária (`0b`) e octal (`0o`), além de separadores visuais com sublinhado (`_`).

```aipo
let x = 42
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
Valores verdadeiros ou falsos (`true` e `false`).

```aipo
let ativo = true
let pronto = false
```

### Texto (`String`)
Strings UTF-8 canônicas com normalização Unicode NFC automática nas fronteiras de construção:

```aipo
let nome = "Aipo"
let saudacao = "Olá, " + nome + "!"
```

### Nulo (`None`)
Representa a ausência explícita de valor (`none`).

```aipo
let opcional = none
```

---

## Coleções

### Listas (`List`)
Vetores dinâmicos ordenados indexados a partir de 0:

```aipo
let itens = [1, 2, 3, 4]
print(itens[0]) // 1
itens.push(5)
```

Operações de alta ordem disponíveis na stdlib: `map`, `flat_map`, `reduce`, `any`, `all`, `filter`.

### Dicionários (`Dict`)
Mapas de chave e valor que preservam a ordem de inserção:

```aipo
let mapa = {
  "host": "localhost",
  "port": 8080
}
print(mapa["port"]) // 8080
```

### Conjuntos (`Set`)
Coleções de valores únicos que preservam a ordem original de inserção:

```aipo
let s = Set()
s.add("alpha")
s.add("beta")
s.add("alpha") // Ignorado, já existente
print(s.contains("beta")) // true
```

### Sequências Lazy (`Sequence`)
Geradores iteráveis avaliados sob demanda sem alocar coleções intermediárias completas na memória.

---

## Manipulação de Bytes (`Bytes`)

Vetores de bytes contíguos de alta performance, projetados para parsing de protocolos de rede e formatos binários:

```aipo
let b = Bytes(16) // Aloca 16 bytes inicializados em zero
b.write_u32_le(0, 42)
let val = b.read_u32_le(0)
```

---

## Operadores & Precedência

- **Aritméticos**: `+`, `-`, `*`, `/`, `%`
- **Comparação**: `==`, `!=`, `<`, `<=`, `>`, `>=`
- **Lógicos**: `and`, `or`, `not`
- **Navegação Segura**: `?.` (avalia o operando esquerdo; se for `none`, não executa o acesso nem os argumentos da chamada)
- **Operador de Fallback Lazy**: `or_else`
- **Operador Pipe**: `|>` para encadeamento elegante de chamadas:

```aipo
let resultado = valor
  |> normalizar
  |> validar
  |> salvar
```
