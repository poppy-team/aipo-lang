# Capítulo 1 — Fundamentos do Frontend

<aside>
🎓

**Objetivo:** registrar, em linguagem didática, como estamos construindo a Aipo desde o código-fonte até o compilador. Esta página serve tanto para revisão futura quanto como material de ensino para quem nunca implementou uma linguagem.

</aside>

## Como usar este caderno

- Os conceitos são apresentados em ordem incremental e com exemplos pequenos.
- Decisões de sintaxe e semântica pertencem à especificação da Aipo; decisões de implementação são estudadas e escolhidas em conjunto.
- Ferramentas e tecnologias não são escolhidas antes de entendermos o problema que elas resolvem.
- Sempre separar três camadas: **linguagem**, **compilador** e **runtime**.

## Visão geral do caminho

```
arquivo .aipo
    ↓
Source / carregamento do arquivo
    ↓
texto UTF-8
    ↓
Lexer
    ↓
Tokens
    ↓
Parser
    ↓
AST
    ↓
análise semântica
    ↓
Core IR
    ├──→ bytecode → Aipo VM
    └──→ JavaScript → Web/JS host
```

O **Core IR** pertence ao compilador e deve permanecer acima dos detalhes de opcode. Isso permite que bytecode e JavaScript reutilizem o mesmo frontend sem obrigar a Aipo a ter backend C/native.

## 1. Source: o que existe antes do lexer

Um arquivo `.aipo` começa como bytes no disco. Antes do lexer, uma camada de **Source** carrega esses bytes, valida a codificação e disponibiliza o texto ao frontend.

Exemplo:

```
hello.aipo
    ↓
bytes no disco
    ↓
validação UTF-8
    ↓
"let age = 20"
    ↓
Lexer
```

O lexer não precisa ser responsável por abrir arquivos. Isso mantém as responsabilidades separadas e facilita testes, REPLs, código gerado e fontes vindas de memória.

### Decisão de codificação da Aipo

- Código-fonte usa **UTF-8** como única codificação oficial.
- UTF-8 sem BOM é o formato normal recomendado.
- Um **BOM UTF-8** no começo do arquivo é aceito e ignorado.
- Quebras de linha **LF** e **CRLF** são aceitas.
- Para o frontend, LF e CRLF representam semanticamente a mesma quebra de linha.
- Bytes que não formem UTF-8 válido produzem diagnóstico antes da lexagem normal.

### BOM, LF e CRLF em termos simples

**BOM — Byte Order Mark:** pequena sequência opcional de bytes no começo do arquivo. Em UTF-8 ela não é necessária, mas alguns editores a adicionam; por isso a Aipo aceita e ignora.

**LF — Line Feed:** quebra de linha representada por `\n`, comum em Linux e macOS moderno.

**CRLF — Carriage Return + Line Feed:** quebra de linha representada por `\r\n`, tradicionalmente comum no Windows.

Visualmente, ambos produzem o mesmo código:

```
let x = 10
let y = 20
```

## 2. O que é um lexer

O **lexer** percorre o texto da esquerda para a direita e transforma caracteres em unidades chamadas **tokens**.

Código:

```
let age = 20
```

Saída conceitual:

```
LET
IDENTIFIER("age")
EQUAL
INTEGER("20")
NEWLINE
EOF
```

### Ideia fundamental

```
texto
    ↓
Lexer
    ↓
Tokens
```

O lexer responde:

> **Que peças existem aqui?**
> 

Ele ainda não decide se o programa faz sentido semanticamente.

## 3. O que é um token

Um token representa uma peça reconhecida do código-fonte.

Exemplos de categorias:

- Keywords: `let`, `var`, `fn`, `if`, `else`, `struct`, `return`.
- Identificadores: `player`, `health`, `calculate_damage`.
- Literais: `10`, `3.14`, `"hello"`, `true`, `none`.
- Operadores e pontuação: `=`, `==`, `+`, `-`, `(`, `)`, `[`, `]`, `.`, `,`, `:`.
- Controle lexical: `NEWLINE`, `EOF`.

Exemplo:

```
let player_name = "Ana"
```

Pode virar:

```
LET
IDENTIFIER("player_name")
EQUAL
STRING("Ana")
NEWLINE
EOF
```

## 4. Espaços, comentários e newlines

Espaços normais são insignificantes entre tokens e podem ser descartados pelo lexer.

Assim:

```
let age = 20
```

e:

```
let     age    =     20
```

produzem essencialmente os mesmos tokens.

Comentários começam com `#` e vão até o final da linha:

```
let age = 20 # idade atual
```

O conteúdo do comentário é ignorado pelo lexer, mas a quebra de linha continua existindo.

Como a Aipo não usa `;` como terminador normal de instruções, `NEWLINE` é útil como token explícito no frontend inicial.

## 5. Como um lexer manual funciona

Um lexer manual mantém um cursor sobre a fonte.

```
let age = 20
^
cursor
```

O ciclo básico é:

```
olhar caractere atual
    ↓
classificar o que começa ali
    ↓
consumir os caracteres necessários
    ↓
produzir um token
    ↓
continuar
```

### Operações conceituais úteis

**current** — retorna o caractere atual.

**advance** — avança o cursor.

**peek** — olha o próximo caractere sem mover o cursor.

**match** — verifica se o próximo caractere corresponde ao esperado e, se corresponder, o consome.

Isso facilita distinguir operadores como:

```
=
==
!
!=
<
<=
>
>=
```

### Identificadores e keywords

Uma estratégia simples é ler primeiro uma palavra como identificador e só depois verificar se ela é reservada.

```
"let"    → LET
"if"     → IF
"struct" → STRUCT
"player" → IDENTIFIER
```

### Números

Ao encontrar um dígito, o lexer continua consumindo os caracteres que pertencem ao literal numérico.

```
20 → INTEGER
3.14 → FLOAT
```

A validação detalhada das regras numéricas pode ser dividida entre lexer e etapas posteriores conforme o desenho do frontend.

## 6. EOF

**EOF** significa **End Of File** — fim do arquivo.

É um token sentinela que informa ao parser que não existem mais tokens.

Exemplo:

```
LET
IDENTIFIER("age")
EQUAL
INTEGER("20")
NEWLINE
EOF
```

## 7. Responsabilidade de cada etapa

Esta distinção deve permanecer clara:

```
Lexer
"Que peças existem?"

Parser
"Como essas peças estão organizadas?"

Análise semântica
"Essa organização é permitida pela linguagem?"
```

Por exemplo:

```
10 + true
```

O lexer pode reconhecer normalmente:

```
INTEGER(10)
PLUS
TRUE
```

Dizer que somar `Int` com `Bool` é inválido não é responsabilidade do lexer.

## 8. SourceSpan: onde algo apareceu no código

Para produzir bons erros, o compilador precisa saber onde cada token e cada construção apareceram na fonte.

Chamamos esse intervalo de **SourceSpan** ou simplesmente **Span**.

Exemplo:

```
let age = 20
    ^^^
```

O token `age` ocupa um intervalo no arquivo.

### Decisão da Aipo

`SourceSpan` usa **offsets de bytes UTF-8** com intervalo semiaberto `[start, end)`.

Conceitualmente:

```
Span
├── start_byte
└── end_byte
```

`[start, end)` significa:

- `start` está incluído;
- `end` não está incluído.

Se `age` ocupa as posições 4, 5 e 6:

```
span = 4..7
```

Esse padrão evita vários erros de `+1` e facilita calcular o tamanho:

```
length = end - start
```

## 9. Por que offsets em bytes

UTF-8 usa quantidade variável de bytes por caractere.

Exemplo:

```
c a f é
```

`c`, `a` e `f` usam um byte cada; `é` usa mais de um byte em UTF-8.

Para o compilador, trabalhar com offsets de bytes é útil porque:

- aponta diretamente para posições no buffer original;
- facilita extrair fatias do texto;
- combina com a representação real UTF-8;
- mantém `Span` pequeno e simples.

Linha e coluna são principalmente uma necessidade de **apresentação de diagnósticos**.

## 10. Source e conversão para linha/coluna

O objeto `Source` pode guardar algo conceitualmente assim:

```
Source
├── path
├── text_utf8
└── line_starts
```

`line_starts` registra em qual byte cada linha começa.

Exemplo:

```
linha 1 → byte 0
linha 2 → byte 11
linha 3 → byte 22
```

Com um `Span(25, 30)`, a camada de diagnóstico pode descobrir em qual linha ele está e calcular uma coluna adequada para exibição.

Assim separamos:

```
representação interna
Span(25, 30)

        ↓ diagnóstico

representação para o usuário
example.aipo:3:4
```

## 11. Estruturas conceituais atuais

A linguagem hospedeira da implementação já foi escolhida: **Odin**. O desenho conceitual do frontend permanece independente dos detalhes de Odin e da estratégia de memória do runtime:

```
Source
├── path
├── text_utf8
└── line_starts
```

```
Span
├── start_byte
└── end_byte
```

```
Token
├── kind: TokenKind
└── span: SourceSpan
```

O lexema original permanece no `Source` e é recuperado pelo `span` quando necessário; o token não duplica o texto por padrão.

Exemplo conceitual:

```
Token {
    kind: IDENTIFIER
    span: 4..7
}
```

## 12. Diagnósticos como objetivo de projeto

Guardar spans desde o lexer permite erros como:

```
example.aipo:1:11

let age = "20
          ^^^
error: string não foi fechada
```

A qualidade dos diagnósticos é parte importante da Aipo porque a linguagem tem como meta baixa carga cognitiva e bom aprendizado.

## 13. Decisões já fechadas nesta etapa

- Source Aipo: UTF-8 obrigatório.
- BOM UTF-8 inicial: aceito e ignorado.
- LF e CRLF: aceitos como a mesma quebra de linha.
- Lexer separado do carregamento de arquivos.
- Lexer transforma texto em tokens e não executa análise semântica.
- Espaços comuns são descartados entre tokens.
- Comentários `#` são ignorados até o fim da linha.
- `NEWLINE` faz parte do modelo inicial de tokens.
- `EOF` marca explicitamente o fim do fluxo de tokens.
- `SourceSpan`: `[start, end)` em offsets de bytes UTF-8.
- Linha e coluna são derivadas pelo `Source` para diagnósticos.

## 14. Próximos capítulos

- `TokenKind`: como representar categorias de tokens.
- Regras lexicais completas da Aipo.
- Unicode em identificadores.
- Literais numéricos e Strings.
- Estratégias de recuperação de erro no lexer.
- Parser: como tokens viram estrutura.
- AST: como representar código dentro do compilador.
- Análise semântica.
- Representações intermediárias.
- Bytecode, VM, runtime e backends.

## Glossário rápido

**Source:** representação de um arquivo/código-fonte dentro do frontend.

**UTF-8:** codificação Unicode usada pelos arquivos `.aipo`.

**Lexer:** componente que transforma caracteres em tokens.

**Token:** unidade lexical reconhecida no código.

**TokenKind:** categoria de um token, como `IDENTIFIER`, `INTEGER` ou `PLUS`.

**Span / SourceSpan:** intervalo da fonte ocupado por um token ou construção.

**Offset:** posição numérica dentro do buffer de origem.

**EOF:** marcador de fim do arquivo.

**Parser:** componente que usa tokens para descobrir a estrutura sintática do programa.

**Análise semântica:** etapa que verifica se a estrutura formada pelo parser faz sentido segundo as regras da linguagem.

## 15. TokenKind: que tipo de peça é esta?

`TokenKind` responde somente à pergunta: **que categoria de token o lexer reconheceu?**

Exemplo:

```
let age = 20
```

Em vez de o parser receber apenas os textos `"let"`, `"age"`, `"="` e `"20"`, ele recebe categorias já classificadas:

```
LET
IDENTIFIER
EQUAL
INTEGER
```

Isso evita que o parser refaça o trabalho lexical.

### Famílias mentais de TokenKind

- **Keywords:** `LET`, `VAR`, `FN`, `IF`, `ELSE`, `RETURN`, `STRUCT` e outras palavras pertencentes à gramática atual. A revisão V1 também introduz superfícies como `EACH`, `IMPL`, `INVARIANT`, `FIXED`, `WHEN`, `ATTEMPT` e `FAILED`; a decisão sobre quais podem ser keywords contextuais será feita durante o lexer/parser.
- **Identificadores:** nomes definidos pelo programador usam `IDENTIFIER`.
- **Literais:** categorias como `INTEGER`, `FLOAT`, `STRING`, `TRUE`, `FALSE`, `NONE`.
- **Operadores:** por exemplo `PLUS`, `MINUS`, `STAR`, `SLASH`, `EQUAL`, `EQUAL_EQUAL`, `NOT_EQUAL`, `LESS`, `LESS_EQUAL`, `GREATER`, `GREATER_EQUAL`.
- **Pontuação:** por exemplo parênteses, colchetes, vírgula, ponto e dois-pontos.
- **Controle lexical:** `NEWLINE` e `EOF`.

### Token, Span e Source têm papéis diferentes

```
TokenKind
→ O QUE é?

SourceSpan
→ ONDE está?

Source
→ QUAL texto estava lá?
```

Se `age` possui `span = 4..7`, o texto `"age"` pode ser recuperado diretamente do `Source`. Portanto, a representação conceitual escolhida é pequena:

```
Token
├── kind: TokenKind
└── span: SourceSpan
```

O token não precisa carregar uma segunda cópia de seu lexema por padrão.

### Literais numéricos: reconhecer não é converter

Ao ler `20`, o lexer precisa reconhecer que o trecho **tem a forma lexical de um inteiro** e produzir `INTEGER`. Ele não precisa transformar imediatamente os caracteres `"20"` no valor inteiro `20`.

Pipeline didático:

```
source: "20"
    ↓ Lexer
INTEGER + span
    ↓ etapa sintática posterior
IntegerLiteral(20)
```

Essa separação mantém o lexer focado em classificação lexical e torna mais visível a divisão de responsabilidades do frontend.

### Por que Strings serão estudadas separadamente

Em uma String como:

```
"hello\nworld"
```

o texto-fonte contém os caracteres `\` e `n`, enquanto o valor lógico poderá conter uma quebra de linha. Por isso o tratamento de escapes merece uma decisão própria, em vez de ser generalizado a partir dos números.

### Decisões fechadas

- `Token = TokenKind + SourceSpan` como representação conceitual mínima.
- O lexema original permanece no `Source` e é recuperado pelo span quando necessário.
- O lexer classifica números, mas a conversão para o valor numérico final acontece posteriormente.
- Strings terão regras próprias estudadas em capítulo separado.

### Regra para memorizar

> **Lexer classifica; o Span localiza; o Source preserva o texto; etapas posteriores constroem significado e valores.**
> 

## 16. Identificadores, keywords e Unicode

O lexer não decide que `let` é keyword ao ler apenas `l` ou `le`. Ele lê primeiro o nome inteiro e só depois o classifica.

```
"let"    → LET
"letter" → IDENTIFIER
"if"     → IF
"if_value" → IDENTIFIER
```

### Regra mental dos identificadores

- início: letra Unicode adequada para identificadores ou `_`;
- continuação: letras Unicode, números adequados para identificadores ou `_`.

Na implementação, usaremos as propriedades Unicode `XID_Start` e `XID_Continue` em vez de inventar uma lista própria de alfabetos.

Exemplos válidos:

```
player
player2
player_health
_cache
café
posição
jogador_2
```

`2player` não começa como identificador. `player-name` é lexado como `IDENTIFIER(player)`, `MINUS`, `IDENTIFIER(name)`.

### `_` como descarte

`_` sozinho representa um valor deliberadamente ignorado. `_cache`, `_temp` e `player_` continuam nomes normais.

### Case-sensitive

Aipo diferencia maiúsculas e minúsculas:

```
player
Player
PLAYER
```

são três identificadores distintos. Por isso `let` é keyword, mas `Let` e `LET` são identificadores normais.

### Por que normalizar Unicode

Unicode pode representar alguns textos visualmente iguais usando sequências de bytes diferentes. Para evitar que dois `café` equivalentes virem nomes diferentes, a identidade lógica dos identificadores é normalizada para **NFC**.

O `Source` original não é alterado. O fluxo é:

```
Source original
    ├── Span → diagnóstico e texto exatamente escrito
    └── lexema → NFC → identidade lógica do nome
```

### Algoritmo conceitual

```
1. guardar início do span
2. consumir XID_Continue ou _
3. obter o lexema pelo span
4. normalizar para NFC
5. se for "_" → DISCARD
6. se coincidir com keyword → TokenKind da keyword
7. caso contrário → IDENTIFIER
```

### O que fixar

1. Leia o nome inteiro antes de classificá-lo.
2. Só depois verifique se ele é keyword.
3. Unicode equivalente precisa ter uma identidade lógica canônica.
4. Spans continuam apontando para os bytes originais do arquivo.

## Método oficial de estudo: Odin + compiladores + Aipo

A partir desta etapa, o caderno deixa de ser apenas uma sequência teórica de frontend e passa a acompanhar uma implementação incremental real em **Odin**.

### Ciclo de cada etapa

1. **Recuperar:** uma pergunta curta sobre algo já aprendido.
2. **Problema:** apresentar a necessidade concreta da Aipo.
3. **Conceito novo:** introduzir somente a ferramenta mínima necessária de Odin ou de compiladores.
4. **Exemplo resolvido:** observar um pequeno exemplo funcionando.
5. **PRIMM:** prever → executar → investigar → modificar → criar.
6. **Fading/scaffolding:** reduzir gradualmente a ajuda até a implementação autônoma.
7. **Testar:** criar ou executar testes imediatamente.
8. **Explicar:** resumir o que foi construído e por que existe.
9. **Revisitar:** fazer o conceito reaparecer posteriormente em outro contexto.

### Orçamento cognitivo

- No máximo um conceito estruturalmente difícil principal por etapa.
- Exemplos pequenos antes de abstrações maiores.
- Não introduzir uma feature de D apenas porque existe.
- Se surgir um assunto útil, mas prematuro, registrá-lo para depois em vez de interromper o fluxo principal.
- Preferir problema concreto → solução simples → necessidade percebida → nova ferramenta → refatoração.

### Progressão de D

**Nível 1 — frontend inicial:** `module`, `import`, tipos básicos, variáveis, funções, `enum`, `struct`, controle de fluxo, arrays/slices, `string`, `unittest`.

**Nível 2 — compiler:** associative arrays, ranges básicos, UFCS, contratos/imutabilidade quando úteis e organização modular maior.

**Nível 3 — VM:** ponteiros, unions, casts, alinhamento, operações de bits e representação de memória.

**Nível 4 — runtime:** GC de D como contraste, memória manual, `@safe`/`@system`, `@nogc`, allocators e C FFI.

**Nível 5 — somente se justificado:** templates, `static if`, CTFE, traits, metaprogramação e mixins.

### Aprendizagem em espiral

Um recurso não precisa ser dominado no primeiro contato. `struct`, `enum`, slices e outros conceitos voltam em etapas diferentes: lexer → parser → AST → bytecode → VM → runtime, cada vez com maior profundidade.

### Regra pedagógica central

> **Não teremos primeiro um curso de D e depois um curso de compiladores. Aprenderemos D construindo a Aipo e aprenderemos compiladores construindo a Aipo.**
> 

### Próxima mudança prática

Em vez de continuar antecipando toda a teoria do lexer, a próxima etapa deve começar com o primeiro projeto D executável e implementar gradualmente `Source`, `Span`, `Token` e o lexer mínimo.