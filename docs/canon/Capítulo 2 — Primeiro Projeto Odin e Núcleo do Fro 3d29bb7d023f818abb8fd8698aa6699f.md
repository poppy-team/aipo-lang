# Capítulo 2 — Primeiro Projeto Odin e Núcleo do Frontend

<aside>
🛠️

**Próxima fase prática:** criar o primeiro projeto Odin da Aipo e implementar incrementalmente as primeiras estruturas reais do frontend.

</aside>

## Objetivos da fase

- Criar e executar o primeiro programa do compilador em Odin.
- Entender apenas o mínimo de `package`, `import`, procedures e testes necessário para começar.
- Implementar `SourceSpan` em Odin.
- Implementar `TokenKind` e `Token` em Odin.
- Construir o primeiro lexer mínimo capaz de reconhecer um programa como `let age = 20`.
- Testar cada avanço antes de adicionar o próximo conceito.
- Manter alocação da infraestrutura do compilador separada do futuro heap gerenciado da Aipo.

## Método

A fase segue o ciclo oficial: recuperação → problema concreto → conceito mínimo → exemplo → PRIMM → modificação → pequena criação → teste → explicação → registro.

## Critério de conclusão

A fase termina quando um pequeno arquivo Aipo puder ser transformado em tokens corretos com spans por uma implementação em Odin compreendida e testada pelo estudante.

## Registro de aprendizagem

Esta página será atualizada conforme a implementação avançar, registrando conceitos de Odin aprendidos, erros relevantes, decisões do frontend e exercícios realizados.

## Aula prática 1 — O primeiro programa Odin

### Problema

Antes de implementar `SourceSpan`, tokens ou lexer, precisamos estabelecer o ciclo mínimo de desenvolvimento: editar Odin → verificar/compilar → executar → testar.

### Conceitos novos desta aula

Somente três ideias entram agora:

- `package`: identifica o package do arquivo.
- `import`: traz um package necessário.
- `proc`: declara uma procedure; `main :: proc()` é o ponto de entrada do executável.

### Primeiro programa

```
package main

import "core:fmt"

main :: proc() {
    fmt.println("Aipo compiler")
}
```

### Ciclo prático

1. Validar a toolchain com `odin version`.
2. Criar uma pasta mínima para o projeto.
3. Verificar o package com `odin check .`.
4. Executar com `odin run .`.
5. Antes de executar, prever a saída esperada.
6. Relacionar cada linha com sua responsabilidade.

### Primeiro teste

Odin possui test runner próprio. Testes são procedures marcadas com `@(test)` e recebem `^testing.T`.

```
package main

import "core:testing"

@(test)
compiler_smoke_test :: proc(t: ^testing.T) {
    testing.expect(t, true, "compiler smoke test")
}
```

Executar com:

```bash
odin test .
```

O test runner deve fazer parte do ciclo normal do projeto; seu rastreamento de memória ajuda a detectar leaks e frees incorretos à medida que começarmos a usar allocators explícitos.

## Aula prática 2 — `SourceSpan`

A primeira estrutura real do frontend representa um intervalo semiaberto de bytes `[start, end)` no source UTF-8.

```bash
Source_Span :: struct {
    start_byte: int,
    end_byte:   int,
}
```

Nesta etapa, o objetivo é aprender `struct` somente através de um problema real: apontar com precisão onde tokens e diagnósticos aparecem no arquivo.

## Aula prática 3 — `TokenKind` e `Token`

Em seguida entram `enum` e composição de structs:

```
Token_Kind :: enum {
    Invalid,
    Identifier,
    Integer,
    Let,
    Equal,
    Newline,
    Eof,
}

Token :: struct {
    kind: Token_Kind,
    span: Source_Span,
}
```

A lista crescerá incrementalmente conforme o lexer aprender novas regras. Não declarar antecipadamente todos os tokens apenas para completar uma tabela teórica.

## Aula prática 4 — primeiro lexer

O primeiro milestone continua deliberadamente pequeno:

```
let age = 20
```

Fluxo esperado:

```
LET
IDENTIFIER("age")
EQUAL
INTEGER("20")
NEWLINE
EOF
```

A implementação deve começar com cursor simples, funções pequenas e testes por comportamento. Unicode completo, strings avançadas e recuperação de erro entram depois.

## Memória nesta fase

Não implementar o GC da Aipo durante o lexer inicial.

- Source, tokens, AST e temporários pertencem à memória do **compilador Odin**.
- Inicialmente preferir estruturas simples; quando lifetimes agrupados começarem a justificar, introduzir `context.temp_allocator`/arenas de forma pedagógica.
- O **Aipo Heap** só nasce na fase da VM/runtime e terá seu próprio tracing GC.

Essa separação evita confundir “Odin gerencia memória manualmente” com “o usuário Aipo gerencia memória”.

## Ciclo de qualidade

```
mudança pequena
    ↓
odin check .
    ↓
odin test .
    ↓
golden test do frontend
    ↓
próxima mudança
```

Quando passarmos a código low-level da VM/GC, adicionaremos sanitizers e stress tests ao mesmo ciclo.

## Nota normativa antes do primeiro lexer

Usar [Aipo V1 — Sintaxe Canônica Consolidada](Aipo V1 — Sintaxe Canônica Consolidada 3d59bb7d023f8184b297c100cde50c68.md) como fonte da superfície atual. O lexer deve reconhecer apenas formas canônicas vigentes e não ressuscitar aliases históricos.

O primeiro milestone `let age = 20` permanece válido e não depende de futuras features como async/await, DSL builders ou backend JavaScript.

## Próxima microetapa

Depois do smoke test e de `SourceSpan`, implementar `TokenKind`, `Token` e um lexer mínimo para `let age = 20`, cada um acompanhado por testes pequenos e explicação do conceito de Odin aprendido.