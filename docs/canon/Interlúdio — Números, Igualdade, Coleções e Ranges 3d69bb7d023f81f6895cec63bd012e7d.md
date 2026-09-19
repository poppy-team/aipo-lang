# Interlúdio — Números, Igualdade, Coleções e Ranges

<aside>
✅

**Decisão fechada — Lote 2.** Este documento registra as regras normativas aprovadas para números/conversões, igualdade/identidade, `List`/`Dict` e ranges/indexação/slicing da Aipo V1.

</aside>

## 1. Números e conversões

Aipo V1 expõe duas categorias numéricas fundamentais: `Int` e `Float`.

- `Int` é inteiro assinado de 64 bits.
- `Float` usa IEEE 754 binary64 e a semântica pública aceita somente valores finitos.
- A única promoção numérica implícita é `Int -> Float` em operações numéricas mistas.
- `Float -> Int` exige `Int(value)` e trunca em direção a zero.
- `Float(value)` converte explicitamente `Int` para `Float` quando desejado.
- Não há coerção automática entre números e `String`, `Bool` ou outras categorias.
- `Bool` não pertence à família numérica.

```
10 + 2.5     # 12.5
Float(10)    # 10.0
Int(3.8)     # 3
```

Divisão mantém formas distintas:

```
5 / 2       # 2.5
5 div 2     # 2
5 % 2       # 1
```

`/` produz divisão real; `div` faz divisão inteira truncando em direção a zero; `%` usa o mesmo quociente de `div`. Overflow de `Int` e resultados `Float` não finitos não fazem wrap/saturação silenciosa.

## 2. Igualdade e identidade

Aipo separa igualdade semântica de identidade física.

```
a == b
 a != b
same(a, b)
```

- `==` e `!=` comparam valor/conteúdo quando a categoria possui igualdade definida.
- Categorias claramente incompatíveis resultam em `false`, não fault.
- Igualdade numérica considera a promoção `Int -> Float`; portanto `1 == 1.0` é `true`, embora `1 is Int` e `1.0 is Float` permaneçam verdadeiros separadamente.
- `List` usa igualdade estrutural recursiva.
- `Dict` usa igualdade estrutural independente da ordem das entradas.
- Duas instâncias de uma mesma `struct` são iguais quando seus campos declarados são estruturalmente iguais.
- Funções/closures não ganham igualdade estrutural na V1.

`same(a, b)` pergunta exclusivamente se dois valores **gerenciados com identidade** representam a mesma identidade. É válido para structs, `List`, `Dict`, functions/closures e outras identidades gerenciadas; aplicar `same()` a valores sem identidade, como `Int`, `Float`, `Bool`, `String` e `none`, é erro de uso.

Não haverá hook mágico `equals()`/`hash()` na V1.

## 3. `List` e `Dict`

`List` e `Dict` são coleções dinâmicas, heterogêneas, mutáveis e com identidade gerenciada. Contratos profundos `List[T]` e `Dict[K, V]` permanecem fora da V1 inicial.

### `List`

```
let empty = []
var values = [10, "hello", none]

values[0]
values[-1]
values[1] = 20
```

- É ordenada e indexada por `Int`.
- Índices negativos contam a partir do fim.
- Índice fora da faixa gera erro/fault de indexação; nunca produz `none` silenciosamente.
- Atribuição por índice não cresce a lista automaticamente.
- Operações comuns da biblioteca incluem `add`, `insert`, `remove`, `remove_at`, `clear` e `len`.

### `Dict`

```
let user = {
    "name": "Ana",
    "age": 20
}

user["name"]
user["age"] = 21
```

- Acesso por `dict[key]` exige que a chave exista; ausência não é representada por `none`, pois `none` pode ser um valor legítimo armazenado.
- `dict.has(key)` é a operação canônica para testar presença antes do acesso.
- A API inicial evita adicionar um `find/get` ambíguo que não consiga distinguir chave ausente de valor armazenado igual a `none`.
- Chaves V1 permanecem limitadas a valores fundamentais estáveis/hashing simples definidos pela especificação normativa; detalhes finais da lista de tipos de chave pertencem à referência de coleções.
- `len(dict)`, remoção e limpeza pertencem à API fundamental da coleção.

## 4. Ranges, indexação e slicing

A forma canônica de range é half-open:

```
0..10
```

representa `0` até `9`.

Isso mantém a relação natural com `len()`:

```
each i in 0..len(items)
    io.print(items[i])
end
```

- `..` é crescente e half-open na V1.
- Range descendente implícito não faz parte da V1; uma API explícita para passo/direção pode existir na biblioteca posteriormente.
- Indexação de `List` e `String` aceita índices negativos.
- `String[index]` opera por Unicode code point e retorna `String`, não byte cru.

Slicing usa a mesma semântica half-open:

```
items[1..4]   # índices 1, 2, 3
items[..3]
items[2..]
items[..]
```

- Limites podem ser omitidos.
- Slicing de `String` trabalha por code points.
- Stride/step sintático como `items[::2]` fica fora da V1.

## Princípio conjunto

**Operações comuns devem ser convenientes; conversões, perdas de informação e ambiguidades devem permanecer explícitas.**

Essas quatro decisões devem ser tratadas em conjunto no parser, runtime, stdlib e suíte de conformidade para preservar a mesma semântica entre VM e futuros backends.