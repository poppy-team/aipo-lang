# Interlúdio — Chamadas, Construção, Impl e Módulos

<aside>
✅

**Lote 4 fechado.** Aipo separa visualmente chamada de função e construção de `struct`: chamadas usam `()`, enquanto construção usa exclusivamente `Type{...}`.

</aside>

## 1. Chamadas, argumentos e defaults

Argumentos são avaliados da esquerda para a direita, uma única vez. Argumentos posicionais vêm antes dos nomeados; um parâmetro não pode ser fornecido duas vezes; parâmetros obrigatórios ausentes, nomes inexistentes e argumentos extras são erro.

```
create_player("Ana", health = 80, active = true)
```

Parâmetros obrigatórios precedem parâmetros com default. Expressões default são avaliadas a cada chamada, permitindo defaults mutáveis seguros:

```
fn create(items = [])
    ...
end
```

Um default pode referenciar parâmetros anteriores, mas não posteriores.

Dot-call existe apenas para funções associadas por `impl Type`; uma função global comum não ganha dot-call automaticamente. Trailing blocks permanecem closures passadas como último argumento e não aceitam argumentos posteriores ao bloco.

## 2. Construção de `struct`: `{}`

A construção de uma `struct` usa sempre chaves:

```
let point = Point{10, 20}
let named = Point{x = 10, y = 20}
```

`()` fica reservado à chamada de funções, hooks na declaração e demais operações call-like. A presença de `init(...)` não muda a superfície de construção: o usuário continua escrevendo `Type{...}`.

Sem `init`, a construção automática segue a ordem dos campos e aceita argumentos nomeados. Quando existe `init`, sua assinatura substitui completamente a assinatura automática de campos; `Type{...}` é validado contra os parâmetros de `init`.

Durante construção, `self` pode inicializar campos ainda não preenchidos. Campos `fixed` podem receber exatamente uma atribuição durante a construção e nunca podem ser substituídos depois da publicação da instância.

## 3. `invariant()` e fronteiras mutáveis

`invariant()` é verificado após construção automática, após `init()` concluído com sucesso e ao término bem-sucedido de uma fronteira que recebeu capacidade mutável sobre a instância.

A validação ocorre em estado estável, não após cada assignment interno. Isso permite estados temporariamente inválidos durante a implementação de uma operação mutável.

Quando a validação falha, campos diretos protegidos da instância retornam ao estado de entrada e a operação produz uma `Failure`. Não existe rollback implícito de I/O, banco, outros objetos ou estado mutável profundo alcançado por alias.

Se uma única operação recebe múltiplas instâncias mutáveis com invariant, todas são validadas antes do commit lógico; se qualquer uma falhar, os campos diretos das instâncias participantes retornam ao estado de entrada.

## 4. `impl`, interfaces e `satisfy`

Múltiplos blocos `impl Type` são permitidos para organização, mas cada operação pode ser definida apenas uma vez por tipo. Overload por tipo ou aridade não existe na V1.

Em toda a `struct`, independentemente do número de blocos `impl`, pode existir no máximo um `init()` e um `invariant()`.

A resolução de `value.operation()` usa o tipo concreto e sua tabela estável de operações associadas; não há prototype lookup, monkey patch ou alteração dinâmica de comportamento associado.

Interfaces permanecem estruturais. `satisfy Type: Interface` declara intenção e exige verificação antecipada, mas não é requisito para que `value is Interface` seja verdadeiro. Compatibilidade de assinatura permanece estrita na V1: posição, mutabilidade e contratos precisam preservar exatamente a chamada prometida pela interface.

Interfaces não possuem estado, campos, defaults, implementação, `init`, `invariant` ou herança entre interfaces na V1.

## 5. Módulos, imports e exports

Na V1, um arquivo `.aipo` corresponde a um módulo. Tudo é privado por padrão e `export` expõe nomes deliberadamente.

```
export Player, create_player
```

A forma básica de import preserva namespace:

```
import player
player.create_player("Ana")
```

Import seletivo reutiliza a mesma keyword:

```
import player: Player, create_player
```

Alias de módulo é permitido:

```
import graphics as gfx
```

Aliases individuais em imports seletivos ficam fora da V1.

Namespaces importados são somente leitura e módulos não são objetos comuns mutáveis.

Declarações estruturais de módulo (`fn`, `struct`, `interface`, `impl`, `satisfy`) são conhecidas durante a resolução do módulo inteiro, permitindo referências cruzadas e recursão mútua. Bindings executáveis `let`/`var` seguem ordem textual.

Código top-level executável inicializa o módulo uma única vez na primeira carga. O grafo de imports deve ser acíclico na V1.

Um módulo de entrada pode fornecer `fn main()` como entry point; bibliotecas não precisam declarar `main`.

## Princípio de superfície

**`()` chama comportamento; `{}` constrói dados estruturados.** Essa separação torna parser, leitura e tooling mais previsíveis e elimina a ambiguidade visual entre chamada de função e construção de `struct`.

## Status

Este documento registra o fechamento do **Lote 4**. A decisão posterior do Lote 5 é independente e não altera as regras de chamadas/construção/módulos aqui registradas.