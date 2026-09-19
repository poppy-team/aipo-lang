# Interlúdio — Expressões, Mutação, Strings e Callables

<aside>
✅

**Decisão normativa:** este interlúdio registra o fechamento do Lote 5 da Aipo V1. As quatro áreas abaixo estão fechadas para a superfície inicial da linguagem.

</aside>

## 1. Operadores e precedência

A precedência da V1 é fixa e propositalmente pequena. Da maior para a menor:

1. postfix: acesso `.`, navegação segura `?.`, chamada `()`, indexação/slicing `[]`;
2. unários numéricos `+` e `-`;
3. multiplicativos `*`, `/`, `div`, `%`;
4. aditivos `+`, `-`;
5. range `..`;
6. comparações `<`, `<=`, `>`, `>=`, `==`, `!=` e teste `is`;
7. `not`;
8. `and`;
9. `or`;
10. fallback recuperável `else`.

Assignment não é expressão e não participa da tabela.

- `and`/`or` fazem short-circuit, exigem `Bool` e sempre retornam `Bool`.
- `not` exige `Bool`; `not value is T` é lido como `not (value is T)`.
- `a else b` avalia `b` apenas quando `a` termina em `Failure`; faults não ativam fallback.
- Comparações encadeadas como `a < b < c` ficam fora da V1.
- Ordenação relacional é válida para números e `String`; Strings usam ordem lexicográfica por Unicode code points.
- A V1 não possui operator overloading definido pelo usuário.

## 2. Assignment e mutação

`=` é statement e não produz valor. Não há chained assignment.

- Targets mutáveis: binding `var`, campo através de caminho mutável e posição mutável de `List`/`Dict`.
- `let`, parâmetro comum e `self` não concedem caminho mutável; `name!` e `self!` concedem.
- `fixed` só pode receber valor uma vez durante construção e não pode ser substituído depois.
- `list[index] = value` substitui apenas posição existente; não cresce a lista.
- `dict[key] = value` insere ou substitui a entrada.
- Compound assignments V1: `+=`, `-=`, `*=`, `/=`, `div=`, `%=`. O target/path é avaliado exatamente uma vez.
- `++`, `--`, assignment expressions e destructuring assignment ficam fora.

## 3. Strings, interpolação e conversão textual

`String` continua imutável, Unicode/NFC, UTF-8 internamente e orientada a code points.

- `String + String` concatena e cria nova String.
- Não existe coerção automática de números, Bool, `none`, structs ou coleções para String.
- `String(value)` é a conversão textual explícita canônica para `String`, `Int`, `Float`, `Bool` e `none`.
- User-defined structs e coleções não recebem hooks mágicos de stringificação na V1; representação customizada é comportamento normal de biblioteca/função.
- `f"...{expr}..."` aceita expressão completa, avalia interpolações uma vez da esquerda para a direita e reutiliza a conversão textual dos valores fundamentais.
- `{{` e `}}` produzem chaves literais em f-strings.
- `r` desativa escapes de backslash; `fr` é a forma canônica para raw + interpolação.
- Prefixos também são válidos em multiline strings.
- Multiline preserva conteúdo/newlines; não há dedent semântico automático nem concatenação implícita de literais adjacentes.

## 4. `Function`, closures e chamada indireta

`Function` é a categoria built-in de valores chamáveis. Closure é uma Function com ambiente lexical capturado, não uma segunda abstração de chamada.

- Funções nomeadas, locais, anônimas e closures são first-class.
- `callback(...)` segue exatamente a mesma semântica de chamada direta: avaliação esquerda→direita, argumentos posicionais/nomeados, defaults e contratos.
- `Function` como contrato significa apenas "valor chamável". Tipos de assinatura detalhados ficam fora da V1.
- Quando o analyzer conhece a função concreta, pode verificar aridade e contratos antecipadamente; incompatibilidade descoberta apenas em runtime é programming fault, não `Failure`.
- Contratos escritos na função/closure permanecem ativos mesmo quando ela é passada como `Function`.
- Funções/closures não suportam igualdade estrutural por `==`; `same` observa identidade gerenciada.
- Referências repetidas à mesma função nomeada usam a identidade daquela declaração; cada avaliação de uma função anônima/closure cria identidade nova.
- `object.operation` sem chamada não cria automaticamente bound-method value. Para callback receiver-associated usa-se closure explícita.
- Callable objects, hook `call`, overload de função e signature generics ficam fora da V1.

## Síntese

A decisão favorece uma regra geral: **expressões devem ser previsíveis, mutação deve ser visível e callbacks devem reutilizar o mesmo modelo de Function já existente, sem protocolos mágicos paralelos.**