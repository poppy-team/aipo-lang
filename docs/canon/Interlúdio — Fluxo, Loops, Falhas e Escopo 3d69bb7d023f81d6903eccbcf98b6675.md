# Interlúdio — Fluxo, Loops, Falhas e Escopo

<aside>
✅

**Decisão fechada.** Este interlúdio consolida o Lote 3 da Aipo V1: `if`/`match`, loops/iteração, modelo `Failure` vs. runtime fault e escopo lexical/closures.

</aside>

## 1. `if` e `match`

- `if` é construção de controle, não expressão na V1.
- Condições exigem `Bool`; não há truthiness.
- `if`, `elif` e `else` criam escopos lexicais próprios.
- `match` avalia sua expressão uma única vez.
- Cada `when` compara por `==`; valores separados por vírgula significam alternativa lógica.
- O primeiro ramo compatível vence; não há fallthrough.
- `else` é opcional; sem match e sem `else`, o fluxo continua após o bloco.
- Cada `when`/`else` cria escopo próprio.
- Pattern matching estrutural, destructuring, guards e type patterns ficam fora da V1. Teste de tipo continua sendo responsabilidade de `is`.

```
match status
when "loading"
    show_loading()
when "ready", "done"
    show_content()
else
    show_error()
end
```

## 2. `loop`, `while`, `repeat` e `each`

A V1 mantém quatro formas distintas e pedagógicas:

- `loop`: repetição indefinida;
- `while`: repetição condicional;
- `repeat n`: repetição contada;
- `each`: iteração sobre uma fonte iterável conhecida pela linguagem/runtime.

`repeat` avalia sua contagem uma única vez e exige `Int >= 0`; `repeat 0` executa zero vezes e não cria índice implícito.

`each` admite:

```
each value in list
    ...
end

each index, value in list
    ...
end

each key in dict
    ...
end

each key, value in dict
    ...
end

each char in text
    ...
end

each index, char in text
    ...
end

each i in 0..10
    ...
end
```

Em `String`, cada elemento iterado continua sendo `String` de um Unicode code point; não existe `Char` fundamental na V1.

`break` e `continue` afetam somente o loop mais próximo; labels e `break value` ficam fora da V1.

Mutação estrutural da coleção atualmente iterada é runtime fault: adicionar/remover elementos de `List` ou adicionar/remover chaves de `Dict` durante sua própria iteração é inválido. Substituir um elemento existente de `List` ou o valor de uma chave já existente de `Dict` pode ocorrer quando o caminho for mutável.

## 3. `Failure` e runtime fault

Aipo separa erros esperados de violações de programação:

- **Failure**: problema esperado/recuperável de operação, como arquivo ausente, timeout, conexão recusada ou dado externo inválido.
- **Runtime fault**: violação de regra da linguagem/programa, como contract fault, índice fora da faixa, chave ausente por `dict[key]`, divisão por zero, overflow, `repeat` negativo ou mutação estrutural durante iteração.

Somente `Failure` é capturável por `attempt`/`failed`. Runtime faults não são capturados.

Falhas recuperáveis não tratadas propagam automaticamente, sem marcador na assinatura.

Fallback local usa `expression else fallback`:

```
let config = load_config() else default_config()
```

O fallback executa apenas quando a expressão da esquerda produz `Failure`; runtime faults continuam como faults. Se o fallback também falhar, sua `Failure` propaga normalmente.

Tratamento agrupado:

```
attempt
    let config = load_config()
    start(config)
failed err
    io.print(err.message)
end
```

A primeira `Failure` interrompe o corpo e entra em `failed`; `err` é binding somente leitura e `failed _` descarta o objeto.

`fail("message")` cria uma `Failure`; `fail(err)` repropaga a mesma falha preservando seu contexto.

`attempt` não é transacional e não desfaz efeitos anteriores. Transações pertencem a APIs/bibliotecas, por exemplo via trailing blocks.

## 4. Escopo lexical e closures

Criam escopo lexical: funções, ramos de `if`, ramos de `match`, `loop`, `while`, `repeat`, `each`, `attempt`, `failed` e trailing blocks `do`.

- Nomes vivem do ponto de declaração até o fim do escopo.
- Redeclaração no mesmo escopo é erro.
- Shadowing em escopo interno é permitido.
- Um novo binding não existe durante seu próprio initializer; uma referência de mesmo nome resolve para binding externo, se houver.

Closures capturam bindings lexicais, não snapshots arbitrários de valores.

- `var` capturado permanece compartilhado e mutável.
- `let` capturado permanece somente leitura.
- O binding capturado permanece vivo enquanto alguma closure o referencia.
- A V1 não possui `nonlocal`, listas de captura, `move capture` ou modalidades explícitas de captura.

Cada iteração de `each` cria bindings frescos, de modo que closures criadas em iterações diferentes capturam bindings diferentes.

Funções locais existem a partir de sua declaração no fluxo local, mas seu nome é visível dentro do próprio corpo para recursão. Regras de pré-declaração no nível de módulo serão consolidadas com o sistema de módulos.

Trailing blocks seguem exatamente as mesmas regras de captura e escopo de closures comuns.

## Princípio do lote

**Controle de fluxo deve ser previsível, falhas recuperáveis devem permanecer distintas de programming faults e closures devem obedecer escopo lexical sem modos especiais de captura.**