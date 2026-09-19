# Governança de Design e Evolução da Aipo

<aside>
⚖️

**Status: normativa para a evolução da linguagem.** Esta página define como novas features, mudanças sintáticas e decisões semânticas devem ser avaliadas. Ela não substitui a especificação da linguagem; governa como a especificação pode evoluir.

</aside>

## Objetivo

Preservar a Aipo como uma linguagem pequena, dinâmica, legível, previsível e progressivamente aprendível. A governança existe para impedir que melhorias locais acumulem complexidade global, aliases, exceções ou modelos mentais desnecessários.

## Constituição de design

### 1. Simplicidade cognitiva acima de concisão

Simplicidade é medida por quantidade de conceitos, exceções, regras implícitas e interpretações necessárias — não por número de caracteres. Uma forma ligeiramente maior é preferível quando for mais previsível e exigir menos regras para ser compreendida.

### 2. Uma operação, uma forma canônica

A linguagem deve evitar aliases sintáticos e construções sobrepostas. Quando uma operação possui uma forma oficial, formas alternativas equivalentes não permanecem apenas por familiaridade.

### 3. Açúcar sintático não cria novo modelo mental

Açúcar pode reduzir cerimônia, mas não deve introduzir uma segunda semântica escondida. Dot-call, por exemplo, continua sendo uma forma conveniente de expressar comportamento associado; ele não deve exigir classes, despacho virtual ou outro modelo paralelo.

### 4. Complexidade avançada é opt-in

Código simples deve continuar plenamente válido e idiomático. Um iniciante pode escrever funções, bindings e estruturas sem conhecer interfaces, contratos, `satisfy`, `fixed`, `invariant` ou detalhes do runtime; esses recursos aparecem conforme a necessidade.

### 5. O caminho simples continua correto

A linguagem não deve ensinar primeiro uma forma simples para depois declarar que “código sério” precisa abandoná-la. Inferência, código sem contratos de tipo e construções diretas continuam formas legítimas quando adequadas ao problema.

### 6. Biblioteca antes de sintaxe

Antes de criar uma keyword, operador ou construção especial, verificar se uma função ou abstração de biblioteca resolve o problema com clareza suficiente. Sintaxe nova é reservada a problemas que não podem ser representados corretamente ou ergonomicamente como biblioteca.

### 7. Expressões normais antes de mini-DSLs

Recursos declarativos devem reutilizar a linguagem existente sempre que possível. `invariant`, por exemplo, usa expressões `Bool` normais em vez de uma DSL própria de validação.

### 8. Features ortogonais

Cada recurso deve responder a uma pergunta diferente. Se duas features passam a representar essencialmente a mesma responsabilidade, uma delas deve ser removida, fundida ou redesenhada.

### 9. Poucos modificadores

Evitar cadeias de adjetivos e flags em declarações. Modificadores só entram quando representam uma propriedade fundamental, recorrente e independente. Aipo não deve evoluir para declarações compostas por combinações de muitos modificadores.

### 10. Magia limitada e consistente

Regras implícitas são aceitáveis quando eliminam uma carga cognitiva maior e permanecem previsíveis. A adição de novas formas de comportamento implícito exige justificativa mais forte do que uma feature explicitamente visível.

## Orçamento de complexidade

Toda feature nova precisa justificar o custo que adiciona. Uma proposta é favorecida quando pelo menos uma destas condições é verdadeira:

- elimina outro conceito, construção ou boilerplate recorrente;
- resolve um problema real e frequente;
- aumenta segurança de maneira significativa;
- torna comportamento existente mais previsível;
- habilita um domínio importante que antes era inviável;
- simplifica substancialmente implementação, ensino ou uso sem deslocar a complexidade para outra parte.

Não é justificativa suficiente, isoladamente:

- “outras linguagens possuem”;
- “fica mais elegante em um exemplo curto”;
- “reduz caracteres”;
- “parece moderno”;
- “permite mais um estilo equivalente”.

## Teste dos cinco custos

Toda proposta relevante deve avaliar explicitamente:

1. **Custo sintático:** quantas novas formas o usuário e o parser precisam reconhecer?
2. **Custo semântico:** quantas novas regras de comportamento aparecem?
3. **Custo pedagógico:** o que precisa ser ensinado antes do uso seguro e correto?
4. **Custo de tooling:** impacto em formatter, parser, highlighting, autocomplete, análise estática, documentação e diagnósticos.
5. **Custo de interação:** como a feature combina com tipos opcionais, interfaces, identidade, erros, mutabilidade, módulos, FFI e demais partes da linguagem?

O custo de interação tem peso especial: features simples isoladamente podem gerar uma matriz complexa quando combinadas.

## Classes de mudança

### Nível 0 — Clarificação

Formaliza comportamento já pretendido sem alterar sintaxe ou semântica observável. Pode ser incorporado com baixo risco.

### Nível 1 — Refinamento

Ajusta regras existentes sem criar um novo modelo mental. Exige revisão de consistência, exemplos e diagnósticos.

### Nível 2 — Extensão

Adiciona capacidade nova. Exige proposta de design, análise dos cinco custos, exemplos reais e revisão de interação sistêmica.

### Nível 3 — Mudança estrutural

Altera o modelo mental fundamental da linguagem. Exemplos possíveis: classes, ownership explícito, overload complexo, macros/metaprogramação ou sistema genérico de alto custo semântico. Durante o fechamento da V1, mudanças de Nível 3 só devem ser aceitas diante de uma limitação fundamental demonstrada.

## Checklist obrigatório de proposta

Uma proposta de feature deve responder:

- **Problema:** qual problema real existe hoje?
- **Frequência:** ele é comum ou excepcional?
- **Alternativa atual:** como o código é escrito sem a feature?
- **Biblioteca:** uma função ou biblioteca resolveria adequadamente?
- **Sintaxe:** qual é a menor nova superfície necessária?
- **Semântica:** quais regras novas aparecem?
- **Interações:** com quais partes da linguagem ela interfere?
- **Ensino:** qual conceito novo o usuário precisa aprender?
- **Diagnósticos:** quais novos erros aparecem e como serão explicados?
- **Remoção:** a feature substitui alguma coisa existente?
- **Reversibilidade:** seria possível retirá-la durante a fase experimental?

## Política de aliases

A sintaxe canônica não deve manter aliases desnecessários. Não devem coexistir duas keywords ou duas construções equivalentes apenas para acomodar preferências de estilo ou familiaridade com outras linguagens.

## Política de keywords

Não existe meta artificial de minimizar o número bruto de keywords. Cada keyword, porém, precisa representar um conceito fundamental, distinto e suficientemente frequente para justificar presença no núcleo.

## Política de operadores e símbolos

Operadores novos exigem justificativa maior que funções de biblioteca. Operadores customizáveis, famílias de marcadores simbólicos e pontuação com múltiplos significados devem ser evitados quando aumentarem a carga de leitura ou introduzirem resolução implícita.

## Política de mutabilidade e modificadores

A linguagem deve preservar uma distinção pequena e clara entre bindings, caminhos de acesso e propriedades estruturais. Novos modificadores como variações de `const`, `readonly`, `mutable`, `late`, `required` ou similares não entram sem demonstrar uma responsabilidade semântica não coberta pelas regras existentes.

## Política de `struct` e `impl`

- `struct` permanece predominantemente declarativa: dados e invariantes.
- `init` permanece a construção especial de inicialização quando necessária.
- `impl` organiza funções receiver-associated e não deve crescer por padrão para abrigar tipos aninhados, propriedades mágicas, overloads, constantes associadas ou um modelo de classes disfarçado.

## Política de tipos e contratos

Aipo permanece dinâmica e fortemente tipada, com contratos opcionais. Novas features de tipos devem preservar o fato de que código sem anotações é legítimo e não uma versão inferior do código anotado. Complexidade de type system só entra quando resolver problemas concretos que contratos leves e interfaces estruturais não resolvam adequadamente.

## Política de tratamento de falhas

A linguagem deve preservar um modelo pequeno. Novas construções equivalentes a exceções tradicionais, múltiplas famílias de captura, propagação ou retry não entram sem demonstrar incapacidade do modelo atual de representar casos reais importantes.

## Política de compatibilidade por fase

### Aipo 0.x

A linguagem é experimental. Corrigir uma decisão ruim tem prioridade sobre preservar compatibilidade prematura. Mudanças incompatíveis devem ser registradas e justificadas.

### Aipo 1.0

Congela o núcleo sintático e semântico: bindings, funções, structs, interfaces, controle de fluxo, erros, identidade, mutabilidade, contratos e módulos essenciais.

### Aipo 1.x

Prioriza compatibilidade, stdlib, tooling, desempenho, diagnósticos e pequenas extensões compatíveis. Mudanças estruturais devem, por padrão, ser adiadas para uma versão maior ou para experimentação separada.

## Processo ADP — Aipo Design Proposal

Mudanças de Nível 2 ou 3 devem possuir uma ADP curta e rastreável com:

```
ADP-NNN — Título

Problema
Motivação
Sintaxe proposta
Semântica
Alternativas consideradas
Interações sistêmicas
Complexidade adicionada
Exemplos reais
Argumentos contra
Estado experimental
Decisão final
```

Uma ADP não precisa ser burocrática; sua função é tornar explícito por que a mudança merece existir.

## Regra da implementação antes da canonização

Para features maiores:

1. discutir o problema;
2. escrever especificação experimental;
3. implementar protótipo;
4. usar em programas reais;
5. avaliar diagnósticos e interação com outras features;
6. somente então promover à sintaxe/semântica canônica.

## Regra dos três programas

Uma feature relevante deve ser avaliada em pelo menos três contextos distintos. Uma construção que só funciona bem no snippet criado para demonstrá-la ainda não está madura.

## Regra do código grande

Decisões sintáticas relevantes também devem ser avaliadas por scanning e manutenção em arquivos maiores, múltiplos módulos e APIs públicas. Legibilidade em cinco linhas não é evidência suficiente de legibilidade sistêmica.

## Regra dos diagnósticos

Uma feature só está completa quando seus principais erros podem ser explicados de forma curta e acionável. Diagnósticos que dependem de longa resolução implícita contam como custo da feature.

## Previsibilidade local

O significado de um trecho deve depender principalmente do próprio trecho e de relações explícitas. Imports ocultos, overload global, coerções implícitas amplas, extensions invisíveis, macros e metaprogramação devem enfrentar um padrão de aprovação elevado.

## Teste final de necessidade

Antes da aprovação, perguntar: **se esta feature não existir, a Aipo continua completa e agradável para seus casos de uso?** Se a ausência causa apenas inconveniência marginal, a feature tende a permanecer fora. Se um problema comum fica artificialmente difícil ou inseguro, a proposta ganha força.

## Resumo constitucional

1. Uma operação, uma forma canônica.
2. Menos conceitos vence menos caracteres.
3. Biblioteca antes de sintaxe.
4. Expressões normais antes de mini-DSLs.
5. Complexidade avançada deve ser opt-in.
6. O código simples deve continuar idiomático.
7. Features devem ser ortogonais.
8. Sem aliases sintáticos desnecessários.
9. Grandes features precisam provar valor em código real.
10. Toda feature nova deve justificar o custo cognitivo que adiciona.

<aside>
🌿

**Regra de espírito:** a Aipo deve ficar mais capaz sem parecer progressivamente maior para quem só precisa do núcleo simples.

</aside>