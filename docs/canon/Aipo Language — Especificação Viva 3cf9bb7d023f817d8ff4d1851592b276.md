# Aipo Language — Especificação Viva

<aside>
📘

**Referência normativa consolidada:** [Aipo V1 — Language Reference](Aipo V1 — Language Reference 3d59bb7d023f811ab6a1f55cfcdc97c1.md). A superfície sintática canônica permanece detalhada em [Aipo V1 — Sintaxe Canônica Consolidada](Aipo V1 — Sintaxe Canônica Consolidada 3d59bb7d023f8184b297c100cde50c68.md). Esta Especificação Viva preserva histórico, justificativas e explorações; quando exemplos históricos divergirem da referência consolidada, eles são superseded.

</aside>

<aside>
⚖️

**Governança normativa de evolução:** [Governança de Design e Evolução da Aipo](Governança de Design e Evolução da Aipo 3d59bb7d023f8162a363c89994b8caca.md). Novas features e mudanças relevantes devem ser avaliadas por orçamento de complexidade, custos sistêmicos, ortogonalidade, necessidade real e prova em código antes de canonização.

</aside>

<aside>
✅

**Decisão atual — 2026-09-15:** Aipo continua pequena, geral, dinâmica/fortemente tipada e managed, mas sua implementação oficial passa a ser **Rust-first e code-agent-first**. Poppy Game Engine será o primeiro host de referência e principal laboratório de uso real, sem criar dialeto de engine e sem abandonar Script/Web. O backend JavaScript permanece first-class. A consolidação desta decisão está em [Aipo — Rust/Poppy Pivot, Host Profiles e Roadmap 10/10](Aipo — Rust Poppy Pivot, Host Profiles e Roadmap 1 3dc9bb7d023f81a4b03fe8f7e6de3508.md).

</aside>

## Filosofia

- Mantém fortemente a filosofia pessoal originalmente pensada para Aipim: linguagem de estudo, experimentação e uso pessoal, construída para aprender implementação de linguagens e produzir software real sem objetivo de competir comercialmente.
- Linguagem pequena, com inspiração de simplicidade em Lua/Ruby, sem copiar suas semânticas integralmente.
- Baixa carga cognitiva, leitura e escrita claras, poucos conceitos visíveis e poucas regras implícitas surpreendentes.
- Complexidade progressiva: começar com um núcleo simples e revelar recursos mais avançados somente quando úteis.
- Clareza e consistência têm prioridade sobre minimizar artificialmente o número de keywords.

## Tipagem — decidido

Aipo será **dinâmica e fortemente tipada, com contratos opcionais restritos às assinaturas de funções e análise estática oportunista**. Gradual typing completo não fará parte da V1.

- Tipos pertencem aos valores; bindings e campos de `struct` não carregam contratos declarativos.
- Código sem anotações continua plenamente válido e idiomático.
- Parâmetros podem declarar opcionalmente contratos na forma `name: Type`; retornos podem declarar opcionalmente `-> Type`.
- Contratos escritos são promessas reais da API: incompatibilidades evidentes podem ser antecipadas pelo compilador e, quando não puderem ser provadas estaticamente, são verificadas em runtime na fronteira da chamada/retorno.
- Um parâmetro sem anotação permanece dinamicamente irrestrito. A ausência de `-> T` significa ausência de contrato declarado de retorno, não `Void`.
- `let`/`var` e campos não aceitam anotações de tipo na V1; o compilador observa e refina tipos localmente por análise de fluxo sem transformar essa inferência em contrato persistente.
- `is` é o mecanismo explícito para teste e narrowing de valores no fluxo.
- `invariant()` é o mecanismo para garantias duradouras de estado, inclusive testes `is` quando uma struct precisar assegurar categorias de valores.
- `T?` permanece válido em contratos de assinatura significando exatamente `T` ou `none` e pode ser usado como expressão de tipo em testes `is`.
- Interfaces podem declarar os mesmos contratos opcionais em parâmetros e retornos.
- `List` e `Dict` podem aparecer como categorias simples em assinaturas; contratos profundos `List[T]` e `Dict[K, V]` ficam fora da V1 inicial e serão avaliados separadamente.
- `Any`, `Void`/`Unit`, generics de usuário, unions arbitrárias, intersections e overloads por tipo ficam fora do núcleo inicial.
- O tooling deve distinguir **tipo inferido** (conhecimento atual do analisador) de **contrato escrito** (promessa estável da API).
- Evitar coerções implícitas entre categorias diferentes de valores permanece como regra.

Documento de decisão: [Interlúdio — Núcleo Dinâmico e Contratos de Assinatura](Interlúdio — Núcleo Dinâmico e Contratos de Assina 3d59bb7d023f812b8260ef925cf08cbc.md).

### Exemplos

```
fn add(a, b)
    return a + b
end

fn distance(a: Point, b: Point) -> Float
    return calculate_distance(a, b)
end

fn create_player(name: String, health: Int = 100) -> Player
    return Player{name, health}
end
```

## Modelo fundamental — decidido

- Sem classes e sem herança.
- `struct` agrupa dados/shape; `impl` organiza comportamento associado e os hooks estruturais especiais `init()` e `invariant()`.
- Funções são first-class e suportam closures.
- `let` representa binding imutável; `var`, binding mutável.
- Dot-call é açúcar sintático para chamadas de função, não um conceito separado de métodos.
- `List` e `Dict` são as coleções fundamentais iniciais.
- Condições exigem `Bool`; não haverá truthiness amplo por padrão.
- Conversões entre categorias distintas permanecem explícitas; a única promoção numérica implícita da V1 é `Int -> Float` em operações numéricas mistas.
- `none` representa **ausência como um valor legítimo**. Isso é distinto de uma função/expressão que não produz valor de resultado.

## Números, igualdade, coleções e ranges — decidido

- `Int` é assinado de 64 bits; `Float` usa binary64 finito. A única promoção implícita é `Int -> Float`; `Float -> Int` usa `Int(value)` e trunca em direção a zero.
- `/` é divisão real; `div`, divisão inteira; `%`, resto associado ao mesmo quociente. Não há coerção automática com `String`/`Bool` nem wraparound numérico silencioso.
- `==`/`!=` comparam valor/conteúdo; categorias incompatíveis produzem `false`. `1 == 1.0` é `true` pela promoção numérica, sem apagar a distinção de tipo para `is`.
- `same(a, b)` é reservado a valores com identidade gerenciada e pergunta identidade, não igualdade estrutural.
- `List` e `Dict` são dinâmicos, heterogêneos e mutáveis; `List[T]`/`Dict[K, V]` permanecem fora da V1 inicial.
- Acesso de `Dict` por `dict[key]` exige chave existente; `dict.has(key)` testa presença. Chave ausente não é confundida com valor armazenado igual a `none`.
- `start..end` é range crescente half-open. Indexação aceita negativos em `List`/`String`; slicing é half-open e permite limites omitidos (`items[..3]`, `items[2..]`, `items[..]`). Stride e range descendente implícito ficam fora da V1.

Documento de decisão: [Interlúdio — Números, Igualdade, Coleções e Ranges](Interlúdio — Números, Igualdade, Coleções e Ranges 3d69bb7d023f81f6895cec63bd012e7d.md).

## Hooks estruturais e trailing blocks — decidido

- `init()` e `invariant()` vivem dentro de `impl Type` e são hooks especiais, não funções comuns.
- `init()` define a fase de construção; `invariant()` define propriedades que precisam permanecer verdadeiras durante a vida da instância.
- `invariant()` não recebe parâmetros, usa `self` como receiver somente leitura e pode reutilizar expressões normais como `is` para segurança opt-in.
- Para a V1, não serão adicionados outros hooks mágicos de clone/hash/string/serialize/iter etc.; esses comportamentos permanecem funções/interfaces normais. Finalização/destruição determinística será estudada separadamente se surgir necessidade concreta.
- `self!` permanece a forma canônica de conceder mutação através do receiver.
- Chamadas podem receber trailing blocks na forma `call(...) do ... end`, opcionalmente com parâmetros após `do`.
- O trailing block é apenas açúcar para uma closure passada como último argumento. Não cria `yield`, retorno não local, receiver implícito ou novo modelo de execução.
- A feature existe para permitir APIs declarativas por biblioteca: builders, testes, transações, recursos, configuração, UI, pipelines e DSLs. **HTML/CSS, UI, scenes e config são direções oficiais de bibliotecas DSL-like**, mas continuam não sendo sintaxe nativa do core: devem reutilizar funções, closures e `do ... end` sempre que possível.
- O frontend pode baixar (`lower`) `do ... end` cedo para `FunctionLiteral`, mantendo VM e backends simples.

Documento de decisão e possibilidades: [Interlúdio — Hooks Estruturais e Blocos como Argumentos](Interlúdio — Hooks Estruturais e Blocos como Argum 3d59bb7d023f81329bfcdfb74573e481.md).

## Memória — decisão arquitetural atual

Aipo é **managed por padrão e permanentemente baseada em tracing GC** no runtime principal. O usuário comum não precisa aprender ownership, borrowing, regions, ponteiros ou allocators para escrever Aipo.

- O heap/runtime managed da Aipo é semanticamente independente da memória normal usada pelo compilador/host escrito em Rust.
- A primeira implementação Rust deve priorizar correção, observabilidade e baixa latência. **gc-arena** é o candidato preferido para um spike inicial de tracing GC incremental/exato; collector próprio só deve ser escolhido se hot reload, profiling, integração ou requisitos semânticos demonstrarem vantagem concreta.
- Pacing/controle de orçamento de coleta apropriado a workloads interativos e games leves é requisito do runtime; pausas, allocation debt e comportamento por frame devem ser medidos nos conformance games.
- Um modo generational poderá ser estudado somente se benchmarks reais justificarem sua complexidade.
- RC + ownership inference/Perceus-like permanece como tema de pesquisa de implementação de linguagens, não como destino obrigatório nem requisito da Aipo.
- Recursos externos como arquivos, sockets, texturas e áudio devem ter ciclo de vida explícito por APIs de recurso/`defer`, sem transferir gerenciamento geral de memória ao usuário.

## Execução — arquitetura decidida

- Caminho principal: **source → compiler → bytecode → Aipo VM**.
- O frontend produz uma representação semântica/Core IR pequena e independente do formato final de bytecode.
- O backend principal emite bytecode portátil para a VM Aipo.
- **JavaScript** é o backend de Web planejado. Integração com TypeScript deve começar preferencialmente por declarações/tooling (`.d.ts`) e somente ganhar um emitter TS separado se houver benefício concreto.
- **Aipo não terá C/native como target da linguagem** no roadmap atual. Aplicações desktop podem ser empacotadas com a VM nativa + bytecode + assets sem compilar o programa Aipo para machine code.
- A VM, runtime, frontend, compilador e toolchain oficiais passam a ser implementados em **Rust**. A baseline Odin anterior permanece somente como histórico pedagógico/arquitetural; a mudança Rust-first aproxima Aipo da Poppy, reduz fronteiras FFI e melhora modularidade, segurança, tooling e desenvolvimento por code agents.
- Async/await entra após o baseline mínimo da VM como capacidade de primeira classe baseada em tasks/fibers + scheduler/event loop. A forma `await expression` é a base semântica; um **`await do ... end` sequencial** será prototipado como açúcar ergonômico para blocos com muitos awaits, com lowering explícito e paridade VM↔JavaScript antes de canonização.

## Perfis e usos pretendidos

A mesma linguagem terá três perfis de uso, sem dialetos sintáticos: **Script**, **Game** e **Web**.

- Script: automação, CLI, plugins e aplicações leves.
- Game: scripting embutível e games leves, com runtime/host APIs para janela, input, áudio, gráficos e assets.
- Web: backend JavaScript e bibliotecas de DOM/HTML/CSS quando aplicável.
- HTML, CSS, UI, scenes, config e APIs declarativas devem ser construídos preferencialmente como **DSL-like libraries** baseadas em funções + trailing blocks, não como novas gramáticas do core.

## Interfaces — modelo V1 decidido

Aipo terá **interfaces estruturais pequenas**, usadas como contratos de comportamento.

- Interfaces contêm apenas **assinaturas de funções receiver-associated**; não terão implementação/default body na V1.
- Comportamento reutilizável deve permanecer em funções normais, podendo usar um contrato de interface como parâmetro e aproveitar dot-call.
- A conformidade continua **estrutural**: um tipo que ofereça todas as operações exigidas com formas compatíveis pode satisfazer a interface mesmo sem declaração nominal obrigatória.
- Para tornar a intenção arquitetural explícita, a V1 terá a declaração prefixa opcional `satisfy`.

```
satisfy Sprite: Drawable, Resettable

struct Sprite
    texture
end
```

- Uma única declaração `satisfy` pode listar várias interfaces separadas por vírgula.
- `satisfy Type: Interface...` é uma promessa verificável: o tipo precisa fornecer **todas** as operações exigidas por cada interface declarada. Se faltar uma operação ou sua assinatura for incompatível, o compilador deve emitir diagnóstico na declaração de conformidade.
- `satisfy` declara intenção e pede verificação; não injeta comportamento, não cria herança e não transforma interface em mixin.
- Não haverá keyword `implements` na sintaxe V1.

**Princípio:** a estrutura permite polimorfismo implícito; `satisfy` torna uma relação arquitetural deliberada visível no código.

### Compatibilidade de assinaturas de interface — decidido

A compatibilidade segue a regra: **toda chamada válida prometida pela interface precisa continuar válida na implementação**.

- O nome da operação precisa coincidir.
- Os nomes dos parâmetros não fazem parte da compatibilidade; posição, mutabilidade e contratos fazem.
- O marcador `!` do receiver deve coincidir exatamente entre interface e implementação na V1.
- Interfaces não permitem valores default em suas assinaturas.
- Uma implementação não pode exigir parâmetros obrigatórios além dos prometidos pela interface.
- Uma implementação pode adicionar parâmetros extras opcionais, desde que todas as chamadas válidas pela interface continuem válidas.
- Contratos explícitos de parâmetros devem coincidir exatamente na V1. Se a interface declara `x: Float`, a implementação também deve declarar `Float`; não haverá covariance, contravariance ou inferência sofisticada para decidir conformidade.
- Se a interface não declara contrato para um parâmetro, a implementação também não pode restringi-lo com um contrato adicional.
- Contratos de retorno também devem coincidir exatamente. Se a interface declara `-> String`, a implementação precisa declarar `-> String`; ausência de anotação, `String?` ou outro contrato não satisfaz essa assinatura.
- A verificação de um `satisfy` explícito usa essas mesmas regras e deve produzir diagnóstico claro para operações ausentes ou incompatíveis.
- A conformidade estrutural implícita usa exatamente as mesmas regras de compatibilidade.

**Regra pedagógica:** mesma chamada, mesmos contratos, mesmo efeito mutável; a implementação só pode adicionar opções, nunca novas exigências.

### Uso de interfaces como contratos — decidido

- Uma interface pode aparecer nas posições de contrato permitidas pela V1: parâmetros e retornos de funções/assinaturas.
- Campos de `struct` e bindings `let`/`var` não recebem contratos declarativos, inclusive de interface.
- A passagem por uma fronteira contratada verifica em runtime se o valor satisfaz estruturalmente a interface, salvo quando o compilador puder provar isso antecipadamente com segurança.
- Usar uma interface como contrato **não converte o valor em um objeto-interface**, não cria wrapper e não cria nova identidade.
- O valor continua sendo o objeto original; para valores gerenciados, identidade compartilhada é preservada, inclusive para `same(...)`.
- `satisfy Type: Interface` pode fornecer ao compilador uma prova explícita e antecipada de conformidade, mas não altera a representação do valor nem é requisito para usar a interface como contrato.
- Interfaces continuam sendo descrições de capacidade, não categorias especiais de armazenamento.

**Princípio:** uma interface restringe o que uma fronteira exige de um valor; ela não transforma o valor em outra coisa.

### `is` com interfaces — decidido

- `value is ConcreteType` continua verificando o tipo concreto do valor.
- `value is Interface` verifica se o valor satisfaz estruturalmente a interface naquele ponto.
- Essa verificação usa as mesmas regras oficiais de compatibilidade de assinatura definidas para conformidade de interfaces.
- A presença ou ausência de uma declaração `satisfy` não muda o resultado semântico de `value is Interface`; `satisfy` apenas torna a intenção explícita e pede verificação antecipada.
- Dentro de um ramo em que `value is Interface` seja verdadeiro, análise estática oportunista e tooling podem tratar `value` como satisfazendo aquela interface e permitir suas operações contratadas.
- Não haverá operadores separados como `implements` ou `satisfies` para teste runtime; `is` cobre tanto tipos concretos quanto interfaces.

**Princípio:** `is` pergunta se o valor pertence ao tipo concreto indicado ou satisfaz o contrato estrutural indicado.

## Controle de fluxo — decidido em alto nível

- Condicionais usam `if`, zero ou mais `elif`, e `else` opcional, fechados por um único `end`.
- `if`/`elif` são destinados a condições booleanas arbitrárias; `match` é destinado à seleção de casos de um valor.
- `match` será a única construção de seleção múltipla; não haverá `switch` separado.
- `loop ... end` representa repetição indefinida explícita.
- `break` encerra o loop mais próximo; `continue` avança para a próxima iteração do loop mais próximo. Labels de loop ficam fora do núcleo inicial.
- `repeat n ... end` será a construção de repetição contada: executa o bloco `n` vezes. Não significa `repeat ... until`.
- `while` permanece para repetição condicional e `each` é a forma canônica de iteração sobre sequências, intervalos e coleções. `for` não permanece como alias.

### `repeat n` — decidido

- `n` deve produzir um `Int` não negativo.
- A expressão de contagem é avaliada uma única vez antes da primeira iteração.
- `repeat 0` é válido e executa zero vezes.
- `break` encerra antecipadamente; `continue` segue para a próxima repetição.
- `repeat` não cria índice implicitamente; quando o índice for necessário, usa-se `each index, value in collection` ou `each i in range`.
- Se a expressão de contagem produzir uma falha recuperável, essa falha propaga normalmente pelo Modelo B e o loop não começa.
- Se a expressão produzir um valor que não seja `Int`, ou um `Int` negativo, isso é **runtime fault de uso da construção**, não falha recuperável de dados.
- O diagnóstico deve informar que `repeat` exige um `Int` não negativo e mostrar o valor/tipo recebido quando possível.

## Erros — Modelo B decidido

Aipo usa **propagação automática de falhas recuperáveis**: uma falha não tratada localmente continua automaticamente para o chamador. Exceptions tradicionais e `try/catch` não são o modelo da linguagem.

- A propagação normal não exige marcador por chamada.
- `expression else fallback` trata localmente uma única expressão fallible.
- `attempt ... failed err ... end` agrupa várias operações sob um único tratamento de falha.
- O identificador após `failed` é um binding local somente leitura da falha capturada; `failed _` descarta explicitamente esse objeto.
- `attempt` não é transacional: efeitos anteriores à falha não são revertidos automaticamente.
- A capacidade de uma função falhar não precisa aparecer na assinatura. `-> T` descreve apenas o valor de sucesso.
- `none` representa ausência legítima e permanece semanticamente distinto de erro.
- Runtime faults de programação não são capturados por `else` nem por `attempt`.
- A representação interna exata (`Result`, tagged value, status de VM ou equivalente) continua sendo detalhe de implementação.
- As formas históricas `try expression`, `try ... else ... end`, `try ... end` e `value or handler` estão superseded e não pertencem à superfície V1.

**Princípio:** o programador escreve sintaxe quando quer **interromper a propagação padrão e tratar a falha**, não para reafirmar a propagação.

### `fail` e bindings de falha — decidido

- `fail("message")` cria uma falha recuperável e interrompe imediatamente o fluxo atual.
- `fail(err)` reenvia explicitamente a mesma falha capturada, preservando sua informação.
- `return value` representa término com sucesso; `fail(...)`, término com falha recuperável.
- Em `attempt ... failed err ... end`, `err` é um binding local somente leitura escolhido pelo programador; não existe nome mágico obrigatório no bloco agrupado.
- A propriedade pública obrigatória da falha na V1 será `.message: String`.
- O runtime pode carregar metadados adicionais internamente sem expô-los como parte obrigatória da V1.
- Para adicionar contexto, cria-se uma nova falha, por exemplo `fail(f"cannot read '{path}': {err.message}")`.
- `fail` dentro de `attempt` transfere o controle para o `failed` correspondente; fora de um tratamento local, a falha propaga automaticamente ao chamador.
- Não haverá sinônimos como `throw` ou `raise` no núcleo inicial.

## Questões abertas

- Sistema exato de valores dinâmicos e representação da VM.
- Coleções e structs dinâmicas.
- Formato de bytecode.

## Contratos parametrizados de coleção e generics — decisão atual

- A V1 não terá generics definidos pelo usuário: não haverá parâmetros de tipo em funções, structs ou interfaces, como `fn first[T](...)`, `struct Box[T]` ou `interface Iterable[T]`.
- `List` e `Dict` podem aparecer como categorias simples em contratos de parâmetro/retorno.
- **`List[T]` e `Dict[K, V]` ficam fora da V1 inicial.** A proposta anterior de tratá-los como contratos parametrizados built-in foi superseded pela consolidação normativa posterior.
- O polimorfismo normal do código sem contratos permanece dinâmico; funções reutilizáveis não precisam declarar parâmetros de tipo.
- Contratos profundos de coleção e generics reais poderão ser reconsiderados somente quando código real demonstrar uma necessidade que justifique sua carga de implementação e de modelo mental.

**Princípio:** manter o núcleo dinâmico pequeno; adicionar parametrização de tipos somente quando houver benefício concreto.

## Modelo de valores — decidido

- `none`, `Bool`, `Int`, `Float` e `String` são valores imutáveis.
- `String` possui semântica de valor, embora sua representação física possa viver na heap gerenciada.
- `List`, `Dict`, instâncias de `struct` e closures são valores gerenciados com identidade.
- Atribuir ou passar um valor gerenciado compartilha a mesma identidade; não há cópia automática profunda.
- `copy(value)` cria uma cópia superficial explícita. Valores gerenciados contidos continuam compartilhados; deep copy não faz parte do núcleo inicial.
- `let` fornece acesso somente de leitura através daquele binding: não permite reatribuição nem mutação através dele.
- `var` permite reatribuição e mutação através daquele binding.
- `let` não congela globalmente um objeto compartilhado: outro binding `var` que aponte para a mesma identidade ainda pode mutá-lo, e o binding `let` observará o novo estado.
- O GC e detalhes como stack, heap e referências permanecem invisíveis no código Aipo comum.

### Exemplo de identidade compartilhada

```
var player = Player("Ana", 100)
let view = player

player.health = 50
print(view.health) # 50
```

### Exemplo de cópia explícita

```
var a = [1, 2, 3]
var b = copy(a)
b.add(4)

# a permanece [1, 2, 3]
# b é [1, 2, 3, 4]
```

## Igualdade, identidade e comparação — decidido

- `==` e `!=` representam igualdade/desigualdade de valor ou conteúdo, não identidade de objeto.
- `same(a, b)` verifica se dois valores gerenciados possuem a mesma identidade.
- `value is Type` fica reservado para verificação/narrowing de tipo.
- `List` usa igualdade estrutural recursiva.
- `Dict` usa igualdade estrutural independente da ordem das entradas.
- Instâncias de `struct` são iguais quando possuem o mesmo tipo de struct e campos estruturalmente iguais.
- Funções/closures não suportam `==`/`!=` inicialmente; identidade pode ser verificada com `same()`.
- Valores de categorias claramente diferentes comparados com `==` retornam `false`; comparações de ordem incompatíveis geram erro.
- `<`, `<=`, `>`, `>=` ficam restritos a tipos com ordem natural definida.
- `String` terá ordenação Unicode determinística no núcleo; ordenação sensível a locale fica para biblioteca.
- O runtime deverá evitar recursão infinita em igualdade estrutural de grafos cíclicos.

## `none` e tipos opcionais — decidido em alto nível

- `none` é um valor real e imutável que representa ausência legítima; não representa falha.
- Código dinâmico pode produzir/retornar `none` sem declaração especial.
- Em contratos opcionais, `T?` significa exatamente `T` ou `none`; não introduz unions gerais.
- Operações inválidas sobre `none` geram erro de runtime claro; a análise estática oportunista pode antecipar casos evidentes.
- `if value != none` e `some(value)` estabelecem presença no fluxo; ferramentas/análises estáticas podem usar essa informação quando disponível.
- `none` não é falsey: condições continuam exigindo `Bool`.
- Coleções dinâmicas podem conter `none`; coleções tipadas devem declará-lo, por exemplo `List[Int?]`.
- Optionalidade (`T?`) e omissão/default de campos ou parâmetros são conceitos separados.
- Safe navigation fará parte da V1 por meio exclusivamente de `?.`, voltado somente à ausência representada por `none`.

### Safe navigation `?.` — decidido

- `value?.field` acessa o campo normalmente quando `value != none`; quando o receiver é `none`, o resultado da expressão é `none` e o acesso não ocorre.
- `value?.function(...)` segue a mesma regra: se o receiver for `none`, a chamada não é executada e o resultado é `none`.
- Encadeamentos como `user?.address?.city` são permitidos; a primeira etapa cujo receiver seja `none` encerra o restante da navegação e produz `none`.
- `?.` trata exclusivamente `none`. Ele não captura nem transforma falhas recuperáveis: se uma chamada produzir uma falha, ela continua propagando automaticamente pelo Modelo B.
- A V1 não adicionará uma família de operadores opcionais como `??`, `??=`, safe indexing ou safe call separado.
- `expression else fallback` continua reservado exclusivamente para falhas recuperáveis e nunca funciona como fallback de `none`.

**Princípio:** `none` usa navegação segura; falhas usam propagação/recuperação. Os dois canais permanecem separados.

### `some(value)` — decidido em alto nível

Aipo terá um builtin `some(value) -> Bool` equivalente conceitualmente a `value != none`; pelo dot-call, `value.some()` será a mesma operação. Ele não cria wrapper `Some(T)`. Se a linguagem mantiver análise estática de opcionais, o compilador poderá reconhecê-lo como type guard para narrowing; em uma Aipo puramente dinâmica, ele continua útil como predicado legível de presença.

## Modelo conceitual de ausência e erro

Aipo manterá ausência e falha separadas na semântica pública. Uma operação fallible pode ter sucesso com um valor que por sua vez seja opcional: conceitualmente `Success(T?)` versus `Error(E)`. Falhas recuperáveis não tratadas propagam automaticamente pelo Modelo B; `none` continua sendo um resultado normal e pode ser tratado com narrowing, `some()` ou safe navigation `?.`. A representação interna pode compartilhar infraestrutura/tagging sem obrigar a linguagem a colapsar os dois conceitos.

## Modelo numérico — decidido em alto nível

- Os tipos numéricos fundamentais da Aipo serão `Int` e `Float`.
- `Int` será inteiro assinado de 64 bits; `UInt` e famílias como `Int8`/`Int32` ficam fora do núcleo inicial.
- `Float` será representado com IEEE 754 binary64, mas a semântica pública da Aipo aceitará apenas valores finitos.
- Operações numéricas mistas permitem promoção restrita `Int -> Float`; não haverá coerções implícitas entre categorias diferentes.
- `Float -> Int` nunca será implícito. A conversão explícita `Int(x)` trunca em direção a zero; `Int(NaN)`, `Int(Infinity)` e valores fora da faixa de `Int` geram erro.
- `/` representa divisão real e retorna `Float`, inclusive quando os dois operandos são `Int`.
- `div` representa divisão inteira.
- `%` representa resto.
- `div` com inteiros trunca o quociente em direção a zero, inclusive com operandos negativos.
- `%` usa o resto correspondente a esse mesmo quociente, preservando `a == (a div b) * b + (a % b)`; quando o resto não é zero, ele tem o mesmo sinal do dividendo.
- Divisão inteira ou resto por zero geram falha numérica de runtime.
- O caso `MIN_INT div -1` gera overflow e, portanto, falha numérica.
- Overflow de `Int` gera erro; wraparound silencioso não faz parte da semântica normal.
- Underscore pode ser usado em literais numéricos para legibilidade, como `1_000_000`.
- O backend JavaScript deverá preservar a semântica i64 da Aipo; uma direção forte é representar `Int` com `BigInt` e aplicar checagem de faixa, enquanto `Float` usa `Number`.
- Resultados `NaN`, `+Infinity` ou `-Infinity` não são valores válidos de `Float` na Aipo; operações que os produziriam geram falha numérica de runtime.
- Isso inclui divisão real por zero e resultados inválidos/infinitos de operações e funções matemáticas.
- Essas falhas numéricas são faults de runtime, não erros recuperáveis que exigem `try`/`or` em cada expressão.
- A VM e backends devem preservar essa invariável nas operações numéricas e nas fronteiras como FFI/parsing.

## Potenciação e utilitários matemáticos — decidido em alto nível

- Potenciação será função `pow(base, exponent)`; `**` não entra na V1 e `^` não será usado para potência.
- Pelo dot-call, `base.pow(exponent)` é açúcar para `pow(base, exponent)`.
- `pow(Int, Int >= 0)` retorna `Int`; overflow gera falha numérica.
- `pow(Int, Int < 0)` retorna `Float`.
- Se qualquer operando de `pow` for `Float`, o resultado é `Float`.
- Resultados não finitos de `pow` seguem a regra geral de `Float` finito e geram falha numérica.
- `pow(0, negativo)` gera falha numérica; `pow(0, 0)` retorna `1`.
- `sqrt(x)` retorna sempre `Float`; resultados fora do domínio real ou não finitos geram falha numérica.
- `abs(Int)` retorna `Int` e `abs(Float)` retorna `Float`; `abs(MIN_INT)` gera overflow.
- `truncate`, `floor`, `ceil` e `round` serão operações de arredondamento/conversão para `Int`, desde que o resultado caiba na faixa de `Int`.
- `min`, `max` e `clamp` serão utilitários numéricos da biblioteca padrão; operações mistas seguem a promoção `Int -> Float`.
- `pow`, `sqrt`, `abs`, `round`, `floor`, `ceil`, `truncate`, `min`, `max` e `clamp` são funções/builtins de biblioteca, não novas keywords ou sintaxe especial.

**Ainda em aberto:** regra exata de empate de `round()`, por exemplo `round(2.5)`.

## Arredondamento — decidido

- `round(x)` retorna o `Int` mais próximo quando o resultado é representável.
- Em empate exato de metade, o arredondamento é **afastando de zero**: `round(2.5) == 3`, `round(-2.5) == -3`.
- `floor()`, `ceil()` e `truncate()` mantêm suas direções explícitas e também retornam `Int` quando representável.
- Uma política especializada como *round half to even* pode existir futuramente como função distinta de biblioteca, sem alterar a semântica de `round()`.
- Resultados fora da faixa de `Int` geram falha numérica, em coerência com o restante do modelo numérico.

## Strings e Unicode — decidido em alto nível

- `String` é imutável e possui semântica de valor.
- Strings Aipo são Unicode e usam UTF-8 como representação interna.
- Operações normais de texto trabalham com Unicode code points, nunca com bytes crus.
- A indexação `text[i]` opera por code point e retorna outra `String`; não haverá tipo fundamental separado `Char` na V1.
- `len(text)` retorna a quantidade de code points; `byte_len(text)` expõe o tamanho UTF-8 em bytes quando isso for necessário.
- Índices fora dos limites geram runtime fault de indexação, não `none` e não erro recuperável por `try`/`or`.
- Índices negativos serão permitidos em `String`, por exemplo `text[-1]` para o último code point; a mesma convenção deve ser considerada para `List`.
- Grapheme clusters não definem a indexação normal da V1; suporte a segmentação visual poderá vir por biblioteca, por exemplo `graphemes(text)`.
- A implementação pode cachear comprimento em code points por a String ser imutável, mas isso permanece detalhe interno.

## Prefixos de String — decidido

- String comum usa `"..."` e trata `{...}` literalmente.
- String interpolada usa prefixo `f`: `f"Olá, {name}"`.
- String raw usa prefixo `r`: `r"C:\Users\name"`; escapes com `\` não são processados como escapes normais.
- A combinação raw + interpolada usa exclusivamente a ordem canônica `fr`: `fr"..."`; `rf"..."` não será uma segunda forma equivalente.
- As mesmas combinações se aplicam a strings multilinha: `"""..."""`, `f"""..."""`, `r"""..."""` e `fr"""..."""`.
- Interpolação só ocorre em strings com `f`; uma String comum contendo `{name}` preserva essas chaves como texto literal.
- A interpolação continua sendo um contexto textual explícito: valores dentro de `{...}` podem ser convertidos para sua representação textual sem criar coerção implícita geral entre `String` e outros tipos.
- `r` e `f` são modificadores ortogonais: `r` controla escapes e `f` controla interpolação.
- As regras exatas para incluir delimitadores e casos extremos de raw strings ainda serão formalizadas separadamente.

## Escapes e raw strings — decidido

- Strings normais reconhecem apenas os escapes `\n`, `\t`, `\r`, `\\`, `\"` e `\u{...}`.
- `\u{...}` é a única sintaxe de escape Unicode da V1 e deve representar um Unicode scalar value válido; valores fora da faixa Unicode e surrogates são rejeitados.
- Escape desconhecido, como `\q`, é erro de compilação.
- `\0` não entra como escape próprio na V1; o code point zero pode ser escrito explicitamente como `\u{0}`.
- Em `r"..."` e `r"""..."""`, backslash é sempre literal e nunca inicia escapes.
- Raw strings podem terminar normalmente em `\`.
- Em raw string de uma linha, aspas literais são escritas duplicando o delimitador, por exemplo `r"Ele disse ""Olá"""`.
- Em raw string multilinha, `"` isolado é texto normal e apenas `"""` fecha o literal; uma ocorrência literal do delimitador pode ser representada pela duplicação do próprio delimitador.
- Em strings com prefixo `f`, `{{` produz `{` e `}}` produz `}`; strings sem `f` tratam chaves como texto comum.
- `fr` combina exatamente os dois comportamentos: `f` ativa interpolação e `r` desativa escapes de backslash.
- Dentro de `{ ... }` de uma f-string, o conteúdo é uma expressão Aipo normal e segue as regras normais da linguagem.
- Toda `String` Aipo deve conter Unicode válido; UTF-8 inválido não é um estado público válido de `String`.

## Normalização Unicode, igualdade e hashing — decidido

- Toda `String` Aipo contém Unicode válido e é normalizada automaticamente para **NFC** antes de ser exposta ao programa.
- NFKC não será aplicado automaticamente; compatibility normalization fica para função explícita de biblioteca.
- Igualdade de `String` usa o conteúdo Unicode canônico; formas canonicamente equivalentes como `"é"` e `"e\u{301}"` comparam como iguais.
- A normalização é invariável de `String`, não uma operação especial feita apenas em `==`.
- Hashing deve respeitar a mesma equivalência: se `a == b`, então `hash(a) == hash(b)`.
- Chaves `String` em `Dict` usam essa mesma representação canônica, evitando duplicidade por diferentes formas Unicode equivalentes.
- Busca e utilitários textuais recebem Strings já normalizadas; `contains`, `find`, `starts_with`, `ends_with` e equivalentes não precisam criar uma semântica paralela de equivalência canônica.
- NFC não implica comparação case-insensitive, remoção de acentos, transliteração ou locale-aware comparison. Essas transformações permanecem explícitas.
- O prefixo `r` afeta apenas parsing de escapes; raw strings continuam Unicode válido e NFC.
- `len(text)` continua contando code points após a normalização; NFC não transforma todo grapheme visual em um único code point.
- Entradas textuais externas como arquivos, terminal, FFI e backend JavaScript devem preservar essa invariável ao produzir uma `String` Aipo.
- `String` representa texto semântico, não bytes exatos. Uma futura abstração `Bytes` ficará responsável por preservar dados binários e representação byte-a-byte.
- Consequentemente, `read_text()` pode normalizar o texto e não garantir round-trip byte-idêntico; preservação exata deve usar `read_bytes()`/`Bytes`.

## Symbols/Atoms — adiado para reavaliação futura

- `Symbol` não fará parte do núcleo público da Aipo V1.
- A necessidade será reavaliada futuramente caso apareça uso recorrente de `String` como tag, estado ou nome simbólico.
- A implementação do compilador/VM poderá usar interning e identificadores internos como `SymbolId` sem expor esse conceito na linguagem.
- Regra de projeto: não expor ao usuário um conceito apenas porque ele é útil internamente.

## API de String — `reverse`, `capitalize` e `format` — decidido

- `reverse(text)` entra na V1 e reverte **extended grapheme clusters Unicode**, preservando unidades visuais compostas em vez de reverter bytes ou code points cegamente.
- O resultado de `reverse()` continua obedecendo às invariantes de `String`: Unicode válido, UTF-8 e normalização NFC.
- `capitalize(text)` entra na V1. Ele aplica titlecase ao primeiro caractere que possui caixa e lowercase aos demais caracteres com caixa; pontuação, espaços e símbolos iniciais são preservados.
- `capitalize()` usa o comportamento Unicode padrão e determinístico; regras específicas de locale ficam para APIs futuras explícitas.
- `format(template, values)` entra na V1 para **templates conhecidos em runtime**, complementando `f"..."`, que continua sendo a forma de interpolação de expressões conhecidas no código-fonte.
- `f"..."` pode conter expressões Aipo normais dentro de `{...}` porque o compilador analisa o código-fonte.
- `format()` **não avalia expressões Aipo em runtime** e não funciona como `eval`; seus placeholders aceitam somente nomes simples, como `{name}` e `{count}`.
- A forma principal é `format(template, values)`; pelo dot-call, `template.format(values)` é açúcar sintático para a mesma operação.
- `format()` usa placeholders nomeados; placeholders posicionais como `{0}` ou `{}` ficam fora da V1.
- `{{` e `}}` representam chaves literais em templates formatados, seguindo a mesma convenção das f-strings.
- Especificadores de formatação embutidos, como `{price:.2f}`, ficam adiados; formatação numérica especializada será feita por funções explícitas de biblioteca inicialmente.
- Template malformado ou placeholder necessário ausente produz **erro recuperável**, compatível com `or`/`try`, pois templates podem vir de arquivos, traduções ou configuração externa.
- Valores extras fornecidos em `values` e não referenciados pelo template são permitidos e ignorados.

## Casos-limite da API básica de `String` — decidido

- `contains("")`, `starts_with("")` e `ends_with("")` retornam `true`.
- `find("")` retorna `0`; quando a substring não existe, `find()` retorna `none`. Índices permanecem medidos em code points.
- `replace(text, "", value)` é inválido na V1 e gera erro de argumento; `replace()` substitui todas as ocorrências não sobrepostas quando o padrão é não vazio.
- `split(text, "")` é inválido; o separador é obrigatório e não pode ser vazio.
- `split()` preserva campos vazios, inclusive nas extremidades. Assim, `"a,,b".split(",") == ["a", "", "b"]`, `",a,b,".split(",") == ["", "a", "b", ""]` e `"".split(",") == [""]`.
- `join(separator, []) == ""`; uma lista com um item retorna esse item; elementos devem ser `String`, sem coerção implícita.
- `trim("")`, `lower("")`, `upper("")`, `capitalize("")` e `reverse("")` retornam `""`.
- `capitalize()` preserva texto sem caracteres com propriedade de caixa, por exemplo `"123"`.
- `format()` sem placeholders retorna o template inalterado; chave ausente ou template malformado geram erro recuperável; chaves extras são ignoradas.
- `format()` aceita valores não-String como valores de placeholders porque é um contexto textual explícito, assim como `f"..."`; isso não cria coerção geral para concatenação ou `join()`.
- A API mantém a distinção entre comportamento textual explícito e coerções implícitas: `"idade: " + 25` continua inválido, enquanto `f"idade: {25}"` e `"{age}".format({"age": 25})` são válidos.

## `List` — modelo fundamental decidido

- `List` é uma sequência ordenada, dinâmica, mutável e com identidade gerenciada.
- Atribuição e passagem compartilham identidade; não há cópia automática da lista.
- Sem contrato explícito, uma `List` pode conter valores de tipos diferentes, incluindo `none`.
- `List[T]` é um contrato runtime opcional sobre os elementos; inserções e atribuições incompatíveis violam o contrato.
- O literal vazio é `[]`.
- `let` impede mutação através daquele binding, mas não congela globalmente a `List`; aliases `var` podem mutar a mesma identidade.
- Indexação começa em zero e aceita índices negativos, que contam a partir do fim.
- Acesso ou atribuição por índice fora da faixa gera `IndexError`; atribuição por índice nunca aumenta automaticamente a lista.
- Slicing usa `list[start..end]`, com limite final exclusivo, limites opcionais e índices negativos.
- Slices fora da faixa são tolerantes/clamped, ao contrário de índices exatos.
- Um slice cria uma nova `List`, mas a cópia é superficial: valores gerenciados contidos continuam compartilhando identidade.
- `list[..]` é semanticamente equivalente a uma cópia superficial da lista.
- Slice step (`list[::2]`, `list[::-1]`) fica fora da V1.
- `==` usa igualdade estrutural e `same()` continua reservado para identidade.

### Exemplos

```
var a = [1, 2, 3]
var b = a
b.add(4)
# a e b observam [1, 2, 3, 4]

var values = [10, 20, 30, 40]
values[-1] = 99
let part = values[1:3]
# part é uma nova List [20, 30]
```

## List — mutação básica (parcialmente decidido)

- A API pública deve preferir nomes comuns e autoexplicativos, evitando jargão técnico quando houver alternativa clara.
- `add(value)` adiciona ao final e retorna `none`.
- `insert(index, value)` insere antes da posição indicada e retorna `none`; índices negativos seguem a convenção posicional já definida.
- `remove(value)` remove apenas a primeira ocorrência e retorna `Bool`: `true` se removeu algo, `false` se o valor não existia.
- `remove_at(index)` remove o elemento da posição indicada; índice inválido gera `IndexError`.
- `clear()` esvazia a List preservando sua identidade compartilhada e retorna `none`.
- Sinônimos como `push`, `append`, `erase` e `delete` ficam fora da API principal.
- O nome da operação equivalente a `pop()` — remover e também devolver o elemento — permanece em aberto; `take()` e `take_at()` foram rejeitados por não serem intuitivos o bastante.

## API de mutação de `List` — nomes canônicos acessíveis

- A API pública da Aipo terá **uma forma canônica por operação**, evitando aliases técnicos como `push`, `pop`, `append`, `erase` e `delete` na V1.
- A documentação/tooling poderá mencionar esses termos apenas como equivalentes de outras linguagens.
- `add(value)` adiciona ao final e retorna `none`.
- `insert(index, value)` insere antes da posição indicada e retorna `none`.
- `remove(value)` remove a primeira ocorrência e retorna `Bool` indicando se removeu algo.
- `remove_at(index)` remove pela posição e **retorna o elemento removido**; não haverá uma segunda função `take_at()`/`pop_at()` para a mesma operação.
- `remove_last()` remove o último elemento e **retorna o elemento removido**; será a forma acessível correspondente ao conceito geralmente chamado de `pop()`.
- `clear()` esvazia a mesma `List`, preservando sua identidade, e retorna `none`.
- Índices inválidos em `remove_at()` e `remove_last()` sobre lista vazia geram `IndexError`.

Princípio de nomenclatura: **preferir nomes cotidianos e autoexplicativos a jargão técnico quando não houver perda de precisão semântica.**

## `List` — API básica de consulta — decidido

- `len(list)` retorna a quantidade de elementos como `Int`.
- `contains(value)` retorna `Bool` e usa a igualdade normal `==` para testar pertencimento.
- `find(value)` retorna o índice da primeira ocorrência como `Int?`; retorna `none` quando não encontra.
- `first()` retorna o primeiro elemento e `last()` retorna o último; ambos são estritos e geram `IndexError` em lista vazia.
- `is_empty()` retorna `Bool` e é a forma canônica de perguntar se a lista está vazia.
- Não haverá sinônimos como `size`, `length`, `has`, `includes` ou `empty` na API canônica.
- Um `get(index) -> T?` simples não entra na V1 neste momento, pois `none` pode ser um elemento legítimo da `List` e isso tornaria ambíguo distinguir “posição inexistente” de “posição contendo `none`”.
- Acesso seguro com fallback ou outra abstração fica em aberto para discussão futura, idealmente de forma coerente também com `Dict`.

### Casos-limite

```
[].first()       # IndexError
[].last()        # IndexError
[].is_empty()    # true
[].find(10)      # none
[].contains(10)  # false
```

**Princípio de nomenclatura:** funções públicas devem preferir nomes cotidianos, fáceis de inferir e uma única forma canônica por operação; terminologia técnica equivalente pode aparecer na documentação e no tooling, sem virar alias público.

## List — transformações e utilidades decididas

- `count(value)` conta ocorrências usando `==` e retorna `0` quando não houver correspondências.
- `reverse()` retorna uma nova `List` com a ordem invertida; a lista original não é modificada. A nova lista é uma cópia superficial e preserva as identidades dos elementos gerenciados.
- `sort()` retorna uma nova `List` ordenada segundo a ordem natural já definida para os elementos; não modifica a lista original. Misturas de valores sem ordem comparável geram erro de comparação.
- `sort_by(function)` retorna uma nova `List` ordenada pela chave produzida pela função fornecida; a intenção é evitar exigir uma função comparadora no caso comum.
- `filter(function)` retorna uma nova `List` apenas com os elementos cuja função produz `Bool true`; por não haver truthiness geral, o predicado deve produzir `Bool`.
- `transform(function)` é o nome canônico da operação tradicionalmente chamada de `map`: aplica uma transformação a cada elemento e retorna uma nova `List`.
- `map`, `find_all`, `select` e `collect` não entram como aliases públicos na V1.
- Regra geral: operações explicitamente mutadoras como `add`, `remove` e `clear` alteram a identidade existente; operações de transformação como `reverse`, `sort`, `filter` e `transform` produzem nova `List`.
- Uma busca por predicado que retorna um elemento permanece adiada, pois precisa ser reconciliada com a possibilidade de `none` ser um elemento legítimo da `List`.

## Iteração de `List` com `each` — decidido

- A forma básica de iteração é `each value in list ... end`.
- Quando o índice também for necessário, a forma canônica será `each index, value in list ... end`.
- A ordem é sempre `index, value`.
- Não haverá `at` para esse caso e `enumerate()` não será necessário na V1.
- O índice começa em `0` e acompanha a indexação normal de `List`.
- Uma `List` vazia produz zero iterações.
- A forma com uma variável entrega apenas o valor; a forma com duas variáveis entrega índice e valor.

### Exemplos

```
each player in players
    print(player)
end

each index, player in players
    print(index, player)
end
```

**Princípio:** preferir uma regra pequena e familiar no próprio `for` a introduzir uma keyword auxiliar (`at`) ou uma função técnica (`enumerate`) apenas para recuperar o índice.

## Iteração de `List` e mutabilidade — decidido

- `each value in list ... end` percorre os elementos na ordem da `List`.
- `each index, value in list ... end` fornece índice e valor; a ordem canônica é sempre `index, value`.
- Não haverá `at` nem `enumerate()` na V1 para obter índice durante `each`.
- O valor iterado é somente leitura por padrão; `each var value in list ... end` permite mutação através do binding.
- Na forma com índice, `index` permanece sempre somente leitura; o valor pode ser `var`, por exemplo `each index, var player in players`.
- Reatribuir um binding `var` do `each` altera apenas o binding local; não substitui automaticamente o elemento armazenado na `List`.
- A mesma `List` que está sendo percorrida não pode ser modificada enquanto a iteração estiver ativa, inclusive por alias. Isso inclui `add`, `insert`, `remove`, `remove_at`, `remove_last`, `clear` e atribuição `list[index] = value`.
- Modificar o estado de um objeto contido através de `for var value` continua permitido, porque isso não modifica a própria `List`.
- Regra pedagógica: **`each` segue o mesmo modelo de `let`/`var`; não modifique a coleção que está percorrendo.**

## Tipos nominais leves (`type`/`newtype`) — adiado

- Aipo V1 **não** incluirá `type`/`newtype` nominal nem alias de tipo como recurso fundamental.
- O recurso não foi rejeitado: será reavaliado futuramente caso o uso real mostre necessidade recorrente de distinguir semanticamente valores com a mesma representação, como `UserId`/`ProductId`, `Meters`/`Seconds` ou `Email`/`String`.
- A decisão segue a filosofia de não adicionar abstrações apenas por utilidade teórica; novos conceitos entram quando resolvem um problema recorrente que justifique sua carga cognitiva e semântica.
- Até lá, `struct` permanece disponível quando for realmente necessária uma identidade semântica explícita.

## Dict — decisão parcial sobre chaves

- `none` não poderá ser usado como chave de `Dict` na V1.
- A justificativa é manter `none` focado em ausência de valor e evitar uma forma de chave pouco útil e cognitivamente estranha.
- A definição completa dos tipos permitidos como chave continua em discussão.

## Dict — funções como valores e chaves — decidido

- Funções e closures são valores de primeira classe e podem ser armazenadas normalmente como valores de `Dict`.
- O mesmo princípio vale para `List` e `struct`: coleções podem conter funções/closures como valores.
- Funções/closures não podem ser usadas como chaves de `Dict` na V1.
- `none` também não pode ser usado como chave de `Dict` na V1.
- Motivo: chaves devem permanecer limitadas a valores com igualdade/hash estáveis e semântica simples; funções possuem identidade, mas não igualdade estrutural normal.
- Consequências como igualdade estrutural de coleções que contenham funções e serialização de funções serão definidas separadamente quando esses tópicos forem formalizados.

## Blocos e fechamento — decidido em alto nível

- Blocos explícitos da Aipo serão fechados com `end`.
- `end` permanece como o delimitador oficial mesmo com a possível introdução futura de formas curtas de uma linha.
- A existência e a sintaxe exata de blocos/condicionais de uma linha permanecem em avaliação; candidatos atuais incluem `if ready: run()` e `if ready then run()`.

## Blocos e condicionais inline — decidido

- Blocos normais continuam sendo fechados explicitamente com `end`.
- Indentação permanece uma convenção visual/canônica, não a fonte principal de delimitação semântica do bloco.
- Aipo terá uma forma curta de condicional com `then`, sem `end`, porque ela não abre um bloco multilinha.
- Forma curta sem `else`: `if condition then statement`.
- Forma curta com `else`: `if condition then statement else statement`.
- Cada ramo inline aceita uma única instrução na V1; múltiplas instruções continuam exigindo o bloco normal com `end`.
- `else` inline é opcional.
- `elif` inline não entra inicialmente; condições com múltiplos ramos continuam usando a forma multilinha.
- A forma inline é controle de fluxo curto, não uma expressão condicional de valor na V1.

### Exemplos

```
if invalid then return
if ready then run() else wait()

if ready
    run()
    save()
else
    wait()
end
```

## Estrutura de blocos, indentação e fim de instrução — decidido

- Blocos são fechados explicitamente com `end`.
- A indentação não possui significado semântico; ela comunica a estrutura para humanos e será normalizada pelo formatter.
- O estilo canônico usa 4 espaços por nível. Tabs podem ser aceitos na entrada sem alterar a semântica.
- A quebra de linha normalmente encerra uma instrução.
- `;` não entra na V1 e múltiplas instruções livres na mesma linha não são permitidas.
- O `if` inline é uma construção sintática própria e não viola a regra de uma instrução por linha.
- Expressões podem continuar livremente dentro de `()`, `[]` e `{}`.
- Fora desses delimitadores, uma expressão pode continuar quando a linha está sintaticamente incompleta, como após operador binário no final da linha.
- Não haverá `\` como marcador de continuação.
- Se a linha já forma uma instrução completa, a próxima linha inicia outra instrução; continuação ambígua não é inferida.
- Linhas vazias são ignoradas.

### Princípio

`end` define blocos; nova linha normalmente encerra instruções; indentação é visual.

## Sintaxe de blocos, indentação e comentários — decidido

- Blocos explícitos são fechados com `end`.
- Indentação não possui significado semântico; o formatter oficial usa 4 espaços por nível.
- Nova linha normalmente encerra uma instrução; `;` fica fora da V1.
- Expressões podem continuar naturalmente dentro de `()`, `[]` e `{}` e também quando a linha termina em uma forma sintaticamente incompleta, como um operador binário.
- Não haverá `\` para continuação explícita de linha.
- A forma curta condicional usa `then`: `if condition then statement`.
- `else` inline também é permitido: `if condition then statement else statement`.
- A forma inline aceita uma única instrução por ramo e não usa `end`; `elif` inline e múltiplas instruções separadas por `;` ficam fora da V1.
- Comentários comuns usam apenas `#`, do marcador até o fim da linha.
- Comentários de bloco/multilinha ficam fora da V1; várias linhas de comentário usam `#` em cada linha.
- Comentários de documentação ficam para avaliação futura quando houver um sistema oficial de documentação.

## Sintaxe lexical, comentários e identificadores — decidido

- Blocos são fechados explicitamente com `end`; indentação não é semântica e o formatter oficial usa 4 espaços por nível.
- Nova linha normalmente encerra uma instrução; `;` não faz parte da V1.
- Expressões podem continuar naturalmente dentro de `()`, `[]` e `{}`, ou quando a linha termina sintaticamente incompleta, como após um operador.
- `if condition then statement` e `if condition then statement else statement` são formas inline próprias, com uma instrução por ramo e sem `end`.
- Comentários comuns usam apenas `#` até o fim da linha; comentários de bloco/multilinha ficam fora da V1.
- Aipo é case-sensitive.
- Convenção canônica: `snake_case` para valores, variáveis, funções, parâmetros e campos; `PascalCase` para structs, interfaces e tipos.
- Identificadores aceitam Unicode e são normalizados em NFC; números são permitidos após o primeiro caractere e `_` pode fazer parte de nomes.
- `_` isolado fica reservado como marcador especial futuro, não como variável comum.
- Somente keywords realmente pertencentes à gramática atual são reservadas. Nomes de operações/biblioteca como `pop`, `shift`, `unshift`, `map` etc. não são keywords e não serão reservados preventivamente.
- Para evolução futura, preferir keywords contextuais quando isso evitar quebrar código existente.

## Escopo e shadowing — decidido

- O escopo da Aipo é lexical: um nome existe do ponto em que é declarado até o fim do bloco correspondente.
- Redeclarar o mesmo nome no mesmo escopo é erro; para mudar um valor existente usa-se atribuição a um binding `var`.
- Shadowing em blocos internos é permitido: um bloco interno pode declarar um nome igual a um nome externo sem alterar o binding externo.
- O tooling pode emitir aviso quando o shadowing parecer acidental, mas isso não é erro da linguagem.
- Parâmetros pertencem ao escopo da função e não podem ser redeclarados no mesmo escopo da função.
- Bindings introduzidos por `each`, incluindo índice, chave e valor, pertencem somente ao loop e não escapam após seu `end`.
- Um binding interno pode escolher `let` ou `var` independentemente da mutabilidade do binding externo que ele sombreia.

Regra pedagógica: **você não pode criar duas vezes o mesmo nome no mesmo lugar; um bloco interno pode ter seus próprios nomes.**

## `Dict`: API de mutação — decidido

- `dict[key] = value` é a forma canônica de adicionar uma nova chave ou substituir o valor de uma chave existente.
- Não haverá `add`, `set`, `put` ou `insert` para `Dict` na V1.
- `dict.remove(key)` remove a entrada quando ela existe e retorna `Bool`: `true` se removeu, `false` se a chave não existia.
- `remove(key)` não retorna o valor removido, evitando ambiguidade quando o valor armazenado é `none`.
- `dict.clear()` remove todas as entradas, preserva a identidade do mesmo `Dict` e retorna `none`.
- Não haverá aliases como `pop`, `delete`, `erase`, `unset` ou `drop` na V1.

```
var user = {}
user["name"] = "Ana"
user["age"] = 25
user["age"] = 26

let removed = user.remove("age")
user.clear()
```

## `Dict` — consulta e acesso seguro

- `len(dict)` retorna a quantidade de entradas.
- `dict.has(key)` retorna `Bool` e verifica presença de chave; não serão adicionados aliases como `has_key`, `contains_key`, `exists` ou `includes` na V1.
- `dict[key]` é acesso estrito: retorna o valor quando a chave existe e gera `KeyError` quando não existe.
- `dict.get(key, default)` exige sempre um valor padrão e retorna o valor armazenado quando a chave existe; caso contrário, retorna `default`.
- `get(key)` sem fallback não existirá na V1, evitando confundir chave ausente com chave presente contendo `none`.
- `dict.keys()` retorna uma nova `List` de chaves na ordem de inserção.
- `dict.values()` retorna uma nova `List` de valores na ordem de inserção; a lista é nova, mas valores gerenciados internos continuam compartilhando identidade.
- `dict.is_empty()` retorna `Bool`.
- A combinação canônica é: `dict[key]` quando a chave deve existir; `dict.get(key, default)` quando ausência deve usar fallback; `has(key)` quando é necessário distinguir explicitamente presença de chave.

## `Dict` — iteração com `each` — decidido

- `each key in dict` percorre as chaves na ordem de inserção.
- `each key, value in dict` percorre chave e valor na ordem de inserção.
- A chave é sempre binding somente leitura; `each var key, ...` não é permitido.
- O valor é somente leitura por padrão; `each key, var value in dict` permite mutabilidade do binding local e, quando o valor é um objeto gerenciado, mutação através desse binding.
- Reatribuir `var value` não substitui automaticamente a entrada correspondente no `Dict`.
- O `Dict` iterado não pode ser modificado enquanto a iteração estiver ativa, inclusive por alias. Inserção, substituição, remoção e `clear()` ficam proibidos sobre a mesma identidade durante o loop.
- `keys()` produz uma nova `List`, portanto pode ser usada quando for necessário percorrer chaves e modificar o `Dict` original.
- Cada iteração cria bindings próprios para chave e valor, evitando que closures criadas no loop capturem uma única variável compartilhada entre todas as iterações.
- Os bindings do `each` são locais ao loop e não escapam após seu `end`.

Regra pedagógica: **o `for` cria bindings locais e a coleção percorrida permanece estável durante a iteração.**

## `Dict` — chaves válidas na V1 — decidido

- As únicas categorias válidas como chave na V1 são `String`, `Int` e `Bool`.
- `Float` não será chave na V1 para evitar complexidade desnecessária com igualdade numérica cruzada, hashing de `1` versus `1.0`, `-0.0` e efeitos de representação binária.
- `none` permanece proibido como chave, mas permitido como valor.
- `List`, `Dict`, `Struct`, Function e Closure não podem ser chaves na V1.
- Funções/closures continuam permitidas como valores de `Dict`.
- `String` usa sua forma normalizada NFC para igualdade e hashing de chave.
- `Bool` é distinto de `Int`; `true` e `1` são chaves diferentes.
- A regra pedagógica é: uma chave de `Dict` deve ser um valor simples e estável.
- Numericamente, `Int` e `Float` permanecem tipos distintos. Operações aritméticas mistas usam promoção `Int -> Float`, mas igualdade e ordenação mistas não convertem ingenuamente o `Int` para binary64, pois isso perderia precisão acima de 2^53. Comparações `Int`/`Float` comparam os valores numéricos exatos: `1 == 1.0` é verdadeiro, `1 == 1.5` é falso e `9_007_199_254_740_993 == 9_007_199_254_740_992.0` é falso. `Bool` continua fora da família numérica, portanto `true == 1` é falso.

## `Dict`: ordem, reinserção e duplicatas — decidido

- `Dict` preserva ordem de inserção.
- Substituir o valor de uma chave existente não altera sua posição.
- Remover uma chave remove também sua posição; se a mesma chave for adicionada novamente, ela entra no final.
- `clear()` remove todas as entradas e toda a ordem anterior.
- Um literal de `Dict` não pode conter duas chaves iguais. Quando a duplicata é detectável estaticamente, o compilador acusa; quando só pode ser descoberta em runtime, a construção falha em runtime.
- Literais de `Dict` são avaliados de forma determinística, da esquerda para a direita / de cima para baixo.

## `struct`: forma, campos e construção — decidido

- `struct` possui **forma fechada**: todos os campos precisam ser declarados na definição; não é permitido adicionar campos dinamicamente a uma instância.
- `Dict` permanece a estrutura apropriada para associações abertas/dinâmicas.
- Contratos de tipo nos campos são opcionais; campos sem contrato continuam válidos em código dinâmico.
- Campos podem ter valores padrão com `=`.
- Campos sem valor padrão são obrigatórios na construção; campos obrigatórios devem vir antes dos campos com default, seguindo o mesmo modelo mental dos parâmetros de função.
- A V1 não terá `let`/`var` individual por campo. Mutabilidade é controlada pelo binding usado para acessar a instância: `let` impede mutação através daquele binding e `var` permite.
- Toda `struct` recebe construção automática via `Nome(...)`, sem conceito separado de classe, `new`, construtor ou método `init`.
- A construção automática cria a instância, preenche campos, aplica defaults e verifica contratos declarados.
- Instâncias de `struct` têm identidade gerenciada; atribuição/passagem compartilha identidade e `copy()` continua sendo cópia superficial explícita.

```
struct Player
    name: String
    health: Int = 100
    inventory: List = []
    nickname: String? = none
end

let player = Player("Ana")
```

**Regra pedagógica:** `struct` descreve algo com forma conhecida; `Dict` representa associações que podem variar livremente.

## `struct`: defaults avaliados por construção — decidido

- Valores padrão de campos são **expressões avaliadas quando uma instância é construída**, e não quando a `struct` é declarada.
- Cada campo omitido avalia novamente sua expressão de default para aquela construção.
- Portanto `inventory: List = []`, `settings: Dict = {}` e `position: Position = Position(0.0, 0.0)` criam novas identidades para cada instância.
- Referenciar explicitamente um objeto já existente, como `inventory: List = shared_inventory`, continua compartilhando aquela identidade segundo a semântica normal da linguagem.
- Se o campo for fornecido explicitamente na construção, seu default não é avaliado.
- Defaults omitidos são avaliados na ordem de declaração dos campos, de cima para baixo.
- O resultado do default passa pela verificação do contrato do campo, quando houver.
- Defaults não possuem `self` especial na V1 e não podem depender implicitamente da instância em construção.

```
struct Player
    name: String
    inventory: List = []
end

var ana = Player("Ana")
var bia = Player("Bia")
ana.inventory.add("Sword")
# bia.inventory continua []
```

**Regra pedagógica:** um valor padrão é calculado quando você cria a `struct`, não quando declara a `struct`.

## `impl` e receiver `self` — decidido

- `impl Type ... end` é a forma canônica para agrupar operações receiver-associated de uma `struct`.
- Dentro de `impl`, o primeiro parâmetro receiver é escrito explicitamente como `self` ou `self!`.
- `self` é o binding do receiver somente leitura; `self!` concede acesso mutável através desse caminho.
- Não existe segundo nome de receiver como `self player`; a grafia é deliberadamente única.
- `this` não fará parte do modelo da Aipo.
- Funções sem receiver não entram em `impl` na V1; factories e helpers permanecem funções normais de módulo.
- `impl` organiza funções associadas e habilita a superfície de dot-call, mas não introduz classes, herança, dispatch virtual ou um sistema OO separado.
- A forma histórica `fn Type.foo(receiver, ...)` está superseded na superfície V1.

```
struct Player
    name: String
    health: Int = 100
end

fn Player.is_alive(self player) -> Bool
    return player.health > 0
end

fn Player.damage(self var player, amount: Int)
    player.health -= amount
end

fn Player.create(name: String)
    return Player(name)
end
```

A chamada por ponto permanece conceitualmente açúcar para passar o receiver como primeiro argumento da função associada.

## Funções associadas, `self` e dot-call — decidido

- Funções podem ser associadas a uma `struct` com a forma `fn Type.nome(...)`.
- `self` é uma keyword contextual que marca explicitamente o receiver; não cria um binding implícito chamado `self`.
- O receiver sempre ocupa a primeira posição da lista de parâmetros e recebe um nome escolhido pelo programador, por exemplo `self player`.
- `self var player` declara receiver mutável e exige acesso mutável na chamada; `self player` é somente leitura.
- Funções associadas sem `self` são funções namespaced pelo tipo, por exemplo `Player.create(...)`; não são chamadas por instância.
- Dot-call só resolve funções associadas que declaram `self`. Assim, `player.damage(20)` é açúcar sintático para `Player.damage(player, 20)` quando `Player.damage` declara receiver.
- Funções globais comuns não entram automaticamente na resolução de dot-call na V1; `fn damage(player, amount)` continua sendo chamada como `damage(player, amount)`.
- Não haverá `this`, receiver implícito, métodos armazenados na instância nem regras de prioridade entre funções globais e associadas.
- Builtins/stdlib devem seguir o mesmo modelo conceitual quando aplicável, evitando um sistema paralelo de métodos mágicos.

```
struct Player
    name: String
    health: Int = 100
end

fn Player.create(name: String)
    return Player(name)
end

fn Player.is_alive(self player) -> Bool
    return player.health > 0
end

fn Player.damage(self var player, amount: Int)
    player.health -= amount
end

var player = Player.create("Ana")
player.damage(20)            # Player.damage(player, 20)
let alive = player.is_alive() # Player.is_alive(player)
```

## Funções associadas e propriedade do tipo — decidido

- Somente o módulo que declara uma `struct` pode declarar funções associadas a ela com `fn Type.foo(...)`.
- Outros arquivos do mesmo módulo podem contribuir para a API associada; a restrição é por módulo, não por arquivo.
- Módulos externos podem criar funções normais que recebem o tipo, mas não podem injetar novos dot-calls na API desse tipo.
- Imports não alteram silenciosamente a API associada de tipos existentes.
- A V1 não terá `extension`, `extend`, `impl` ou mecanismo equivalente para extensão externa de métodos.
- `String`, `List`, `Dict` e demais tipos de biblioteca seguem a mesma regra: sua API associada é controlada pelo módulo proprietário.

**Princípio:** o módulo que cria um tipo controla sua API associada; outros módulos compõem comportamento por funções normais.

## Funções associadas: sem overloading — decidido

- Aipo V1 não terá overloading de funções.
- Dentro de um namespace, um nome completo identifica exatamente uma função.
- Duas funções globais com o mesmo nome são inválidas; duas funções associadas com o mesmo nome no mesmo tipo também são inválidas, independentemente de número de parâmetros, contratos de tipo ou retorno.
- Nomes iguais em namespaces diferentes são válidos, por exemplo `Player.clear`, `List.clear` e `Dict.clear`.
- Uma função global e uma função associada também podem compartilhar o nome simples, pois `save` e `Player.save` são nomes completos diferentes.
- Parâmetros default e argumentos nomeados são os mecanismos preferidos para variações simples de chamada.
- Quando operações forem semanticamente diferentes, preferem-se nomes diferentes e explícitos em vez de overload.
- Builtins fundamentais como `len(value)` podem internamente suportar várias categorias de valor sem expor um sistema de overload declarativo ao usuário.
- A regra reforça funções first-class: referências como `Player.damage` sempre identificam uma única função.

## `struct` e campos que armazenam funções — decidido

- `struct` continua responsável por declarar a forma dos dados; declarações de funções associadas não ficam aninhadas dentro do bloco da `struct`.
- Campos de `struct` podem armazenar funções e closures normalmente, porque funções são valores first-class.
- Um campo função pode ser fornecido na construção, por exemplo `Button(label = "Salvar", action = fn() ... end)`.
- Valores padrão também podem ser funções. Para defaults não triviais, o estilo recomendado é referenciar uma função já declarada, como `action = do_nothing`, em vez de embutir um corpo grande dentro da `struct`.
- Uma função anônima inline como valor/default é semanticamente permitida; corpos extensos dentro da declaração da `struct` são desencorajados por estilo, não proibidos pela semântica.
- Comportamento pertencente à API estável do tipo continua sendo declarado externamente com `fn Type.name(...)`, usando `self` quando houver receiver.
- Não haverá sintaxe de função associada aninhada dentro de `struct` na V1 (`struct ... fn show(...) ... end ... end`).
- Quando um campo contém uma função, `button.action()` significa ler `button.action` e chamar o valor armazenado. Isso permanece distinto de dot-call de função associada.

**Princípio:** `struct` declara dados; funções associadas declaram a API do tipo; campos podem guardar funções quando comportamento variável faz parte dos dados.

## Regras de valor e restrições de campo — decidido

Aipo terá uma sintaxe pequena de **regras declarativas** aplicada com `@` em campos de `struct`.

Forma geral:

```
field: Type @rule @rule(arguments) = default
```

### Regras de valor iniciais

- `@min(value)` — exige valor mínimo.
- `@max(value)` — exige valor máximo.
- `@between(min, max)` — exige valor dentro do intervalo indicado.
- `@not_empty` — exige `String` ou coleção não vazia quando aplicável.
- `@length(min, max)` — exige comprimento dentro do intervalo indicado quando o valor possui comprimento.
- `@one_of(...)` — restringe o valor a uma lista explícita de alternativas.
- `@check(function)` — usa uma função predicado personalizada para validar o valor.

Contratos/regras de valor **validam; não transformam**. Valores default continuam sendo declarados separadamente com `=`.

Exemplo:

```
struct Player
    id: Int @fixed
    name: String @not_empty @length(2, 32)
    health: Int @between(0, 100) = 100
    class: String @one_of("warrior", "mage", "archer")
    code: String @check(valid_code)
end
```

### Restrição de campo inicial

A única restrição de campo da V1, por enquanto, será `@fixed`.

- `@fixed` permite que o campo receba seu valor durante a construção da instância.
- Depois que a construção termina, o campo não pode receber outro valor.
- `@fixed` não fornece o valor; defaults permanecem separados. Exemplo: `version: Int @fixed = 1`.
- `@fixed` é propriedade do campo; `let`/`var` continuam sendo propriedades do binding.

**Histórico superseded:** esta fase usou `@` para regras de campo. A revisão de 2026-09-07 removeu essa mini-DSL da superfície V1 em favor de `fixed` + `invariant` com expressões normais.

## Visibilidade e encapsulamento de `struct` — decidido

- A V1 não terá `public`, `private` nem `protected` por membro.
- Se uma `struct` está acessível, seus campos declarados também são acessíveis diretamente.
- `_nome` não cria privacidade; continua sendo um identificador comum.
- Não haverá getters/setters automáticos nem privilégio secreto para funções em `impl`.
- Funções de `impl` acessam campos pelas mesmas regras normais de acesso; `self` não concede privilégio de visibilidade.
- `let`/`var` e `self!` controlam a capacidade de mutação através do caminho; `fixed` pode impedir a substituição de campos específicos.
- Invariantes simples devem ser expressos preferencialmente pelas regras de campo/valor já definidas, evitando boilerplate de encapsulamento OO.
- A fronteira maior de API será tratada pelo sistema de módulos quando ele for especificado.

**Princípio:** `struct` descreve dados de forma explícita; contratos/regras protegem invariantes locais; módulos definem fronteiras de API.

## Igualdade e cópia quando há funções — decidido

- `Function` e `Closure` não suportam `==`/`!=` na V1; identidade é verificada com `same()`.
- Igualdade estrutural de `Struct`, `List` ou `Dict` só é definida quando todo o conteúdo alcançado é comparável. Se a comparação estrutural alcançar `Function` ou `Closure`, a operação gera um erro de comparação claro; funções não são ignoradas nem comparadas implicitamente por identidade.
- `copy(value)` permanece cópia superficial. Copiar uma `struct`, `List` ou `Dict` cria a nova estrutura correspondente, mas valores gerenciados contidos — inclusive Functions e Closures — continuam compartilhando identidade.
- `copy()` não reavalia defaults de `struct`; ele copia os valores já presentes. Já construções independentes reavaliam os defaults, portanto um default `fn() ... end` cria closures distintas em construções distintas.
- Regras e restrições declaradas no tipo continuam valendo na instância copiada, inclusive `@fixed`.

Regra pedagógica: **`copy()` copia superficialmente os valores existentes; `==` só compara estruturas cujo conteúdo inteiro seja comparável; funções têm identidade, mas não igualdade de conteúdo.**

## Construção de `struct` — decidido

- Instâncias de `struct` são construídas diretamente com `Type(...)`; `new` não faz parte da V1.
- Argumentos posicionais e nomeados são permitidos.
- Argumentos posicionais seguem a ordem de declaração dos campos.
- Argumentos nomeados usam `name = value` dentro da chamada; essa forma é sintaxe de argumento nomeado, não expressão geral de atribuição.
- Argumentos posicionais devem aparecer antes dos nomeados.
- Argumentos nomeados podem aparecer em qualquer ordem e podem pular campos que possuam default; o formatter/guia de estilo pode recomendar a ordem da declaração.
- Campos obrigatórios sem default precisam ser fornecidos.
- Campos obrigatórios devem ser declarados antes de campos com default para manter construção posicional previsível.
- Fornecer o mesmo campo duas vezes, por posição e nome ou por dois argumentos nomeados, é erro.
- Argumento nomeado que não corresponda a um campo declarado é erro.
- Contratos de tipo e regras `@...` são verificados durante a construção antes de a instância ser considerada válida.
- `@fixed` permite fornecer o valor durante a construção; a restrição impede escritas posteriores naquele campo.
- Diretriz de estilo: use posição quando a construção continuar óbvia; use nomes quando eles tornarem a chamada mais clara.

### Exemplos

```
struct Player
    id: Int @fixed
    name: String @not_empty
    health: Int @between(0, 100) = 100
    active: Bool = true
end

let a = Player(1, "Ana")
let b = Player(2, "Bia", health = 80)
let c = Player(
    id = 3,
    name = "Caio",
    active = false
)
```

## Receivers e parâmetros mutáveis — decidido

- `self` não fará parte da V1.
- Em uma função associada declarada como `fn Type.name(...)`, o primeiro parâmetro é sempre o receiver.
- `receiver` sem marcador fornece acesso somente de leitura ao valor recebido.
- `receiver!` declara que a função pode modificar através daquele parâmetro.
- O mesmo `!` pode ser usado em parâmetros comuns: `fn update(value!)`.
- `!` não significa ponteiro, referência, ownership, borrowing nem passagem do binding por referência; significa apenas acesso mutável através do parâmetro.
- O binding do parâmetro continua não reatribuível; `value = other` não é habilitado por `!`.
- `var` fica reservado à mutabilidade de bindings locais e deixa de ser usado para declarar mutabilidade de parâmetros.
- Um argumento acessado por `let` não pode ser passado a parâmetro `!`; um argumento acessado por `var` pode ser passado tanto a parâmetro somente leitura quanto a parâmetro `!`.
- Funções associadas sem receiver ficam fora da V1. Operações que não atuam sobre uma instância devem ser funções normais de módulo; construção básica continua `Type(...)`.
- Dot-call permanece açúcar: `player.damage(20)` resolve a operação `damage` declarada em `impl Player`, usando `player` como `self`.

```
impl Player
    fn is_alive(self) -> Bool
        return self.health > 0
    end

    fn damage(self!, amount: Int)
        self.health = max(self.health - amount, 0)
    end
end
```

**Princípio:** `var` torna um binding variável; `!` concede acesso mutável através de um parâmetro.

## Histórico superseded — experimento de regras de campo `where` / `@`

Durante uma fase experimental, Aipo avaliou **duas superfícies sintáticas equivalentes** para regras declarativas de campo: `where` e `@(...)`. **Ambas estão superseded** pela revisão de 2026-09-07, que adotou `fixed` + `invariant`.

Exemplos equivalentes:

```
struct Player
    id: Int where fixed
    name: String where not_empty, length(2, 32)
    health: Int where between(0, 100) = 100
end
```

```
struct Player
    id: Int @(fixed)
    name: String @(not_empty, length(2, 32))
    health: Int @(between(0, 100)) = 100
end
```

Regras desta fase:

- `where` e `@(...)` devem produzir exatamente a mesma representação interna/AST-HIR de regras; não haverá duas semânticas.
- O conjunto de regras continua pequeno: `min`, `max`, `between`, `not_empty`, `length`, `one_of`, `check` e `fixed`.
- **Invariantes arbitrários em expressão ficam fora desta fase/V1.** Casos customizados devem usar `check(function)`.
- A implementação e os testes devem comparar legibilidade, facilidade de ensino, parsing, formatter, diagnósticos e experiência prática antes de selecionar uma forma canônica.
- Antes de publicação para uso externo, apenas uma sintaxe deverá permanecer canônica; a outra poderá ser removida para preservar o princípio de uma forma canônica por operação.

## Mutabilidade por caminho de acesso — decidido

A mutabilidade da Aipo é uma propriedade do **caminho de acesso**, não um congelamento global do objeto.

- Um caminho iniciado por `let` permanece somente leitura até o fim; não é permitido modificar campos, elementos de `List`/`Dict` ou objetos gerenciados alcançados através dele.
- Um parâmetro comum é um caminho somente leitura.
- Um binding `var` e um parâmetro marcado com `!` permitem mutação através do caminho, inclusive em objetos gerenciados aninhados.
- O marcador `!` não precisa ser repetido em níveis internos: se `player!` é mutável, `player.inventory.add(...)` pode modificar `inventory`, respeitando as regras dos campos encontrados no caminho.
- Não é permitido criar um binding `var` para um valor gerenciado obtido através de um caminho somente leitura, pois isso permitiria contornar `let` ou um parâmetro comum. Ex.: `let player = ...; var inventory = player.inventory` é inválido.
- Atribuir um valor gerenciado obtido por caminho mutável a um novo binding `var` continua compartilhando identidade normalmente.
- `fixed` é uma regra do campo e impede **substituir o valor armazenado naquele campo após a construção**, mas não congela profundamente o objeto gerenciado armazenado nele.
- Assim, em um caminho mutável, `player.inventory = []` é inválido quando `inventory` é `fixed`, mas `player.inventory.add("Sword")` é válido.
- `fixed` continua valendo mesmo quando o caminho é mutável; `!` concede permissão de mutação, não ignora contratos/regras do campo.
- A V1 não terá deep immutability, borrow checker, referências explícitas ou propagação de `!` escrita em cada nível do caminho.

**Regra pedagógica:** `let`/parâmetro comum = somente leitura pelo caminho; `var`/`parameter!` = acesso mutável pelo caminho; `fixed` = aquele campo específico não pode receber outro valor após a construção.

## Namespace público de `struct` — decidido

- Campos e funções associadas compartilham o mesmo namespace público do tipo.
- Um campo e uma função associada não podem ter o mesmo nome.
- Não existe regra de precedência entre campo e função; colisões são rejeitadas na definição.
- Se `foo` for um campo, `value.foo` acessa esse campo; se ele contiver uma função, `value.foo()` chama o valor armazenado.
- Se `foo` não for um campo e existir `fn Type.foo(receiver, ...)`, então `value.foo(...)` é dot-call para `Type.foo(value, ...)`.
- Parâmetros e variáveis locais podem ter o mesmo nome de campos, pois pertencem ao escopo lexical local.
- Princípio: **dentro da API pública de um tipo, um nome representa uma única coisa.**

## Construção de `struct` — ordem e validação — decidido

A construção de uma `struct` segue uma ordem determinística e fail-fast:

1. validar estruturalmente a chamada (campos desconhecidos, duplicados ou obrigatórios ausentes);
2. avaliar os argumentos explícitos da esquerda para a direita, na ordem escrita pelo programador;
3. avaliar apenas os defaults necessários, em ordem de declaração dos campos;
4. verificar contratos de tipo;
5. verificar regras de valor na ordem declarada, interrompendo na primeira falha;
6. finalizar a instância e ativar restrições estruturais como `fixed`.
- Um valor fornecido explicitamente impede a avaliação do default correspondente.
- Defaults continuam sendo avaliados no momento da construção e produzem novos valores gerenciados quando a expressão cria um novo valor, como `[]` ou `{}`.
- Regras de valor só são executadas após o contrato de tipo correspondente ser satisfeito.
- `fixed` participa da superfície de regras de campo, mas internamente é uma restrição estrutural: não valida o valor e passa a impedir a substituição do campo depois da construção.
- Nenhuma instância parcialmente válida fica acessível ao programa; se qualquer contrato ou regra falhar, a construção não produz uma instância.
- Efeitos já realizados durante a avaliação de argumentos ou defaults não são revertidos automaticamente em caso de falha posterior; não haverá rollback transacional da construção.
- Chamadas estruturalmente inválidas devem ser rejeitadas antes de iniciar a construção sempre que o compilador puder determinar isso.

## `check(function)` — decidido

- `check(function)` é a porta de validação customizada para regras de campo; invariantes arbitrários por expressão continuam fora da V1.
- A função recebe exatamente o valor candidato como único argumento, por acesso somente leitura; funções cujo parâmetro correspondente usa `!` não são válidas em `check`.
- O retorno deve ser exatamente `Bool`: `true` aceita o valor e `false` produz violação da regra. Retorno diferente de `Bool` é erro da função de validação, não simples rejeição do dado.
- `check` valida, não transforma, normaliza ou substitui o valor.
- Validators devem ser determinísticos e livres de efeitos colaterais por convenção e documentação; a V1 não terá sistema de efeitos/pureza para provar isso globalmente.
- Regras de campo são verificadas na construção e em toda substituição direta posterior do campo. A nova atribuição só é efetivada depois de contrato de tipo e regras passarem; se falhar, o valor anterior permanece.
- Regras de campo não monitoram nem revalidam automaticamente mutações internas transitivas de valores gerenciados como `List`, `Dict` ou outras `structs`. Invariantes fortes sobre mutação interna devem pertencer ao próprio tipo/API que controla essa mutação.

```
fn valid_code(code: String) -> Bool
    return code.starts_with("AIP-")
end

struct Item
    code: String where check(valid_code)
end
```

A grafia `where` acima permanece experimental em paralelo com `@(...)`; ambas representam a mesma regra interna durante o desenvolvimento.

## Falhas de contrato e validação de dados — decidido

- Violações de contrato de tipo em campos e parâmetros de dados são erros recuperáveis, não faults de programação.
- Regras declarativas como `between`, `not_empty`, `length`, `one_of` e `check(...)` rejeitando um valor também produzem erro recuperável de validação.
- Construção inválida não produz instância; atribuição inválida preserva o valor anterior.
- Erros recuperáveis de contrato/validação devem carregar informação estruturada suficiente para diagnóstico, incluindo ao menos tipo/struct, campo, contrato ou regra aplicável e valor recebido.
- Um validator usado em `check(...)` que retorna algo diferente de `Bool` é um runtime fault de programação, pois indica implementação incorreta da regra, e não dado inválido.
- A V1 não terá mensagens customizadas declaradas junto às regras; bibliotecas e ferramentas podem mapear os dados estruturados do erro para mensagens de interface.
- A integração sintática exata dessas falhas com `or` e `try` será fechada junto do modelo final de tratamento de erros.

**Princípio:** dados inválidos podem acontecer normalmente; uma regra de validação implementada incorretamente é um bug.

## Histórico superseded — experimento de erros com `or` e `try`

**Fechado:**

- `expression or handler` trata localmente uma falha recuperável; `or` não trata `none`.
- O handler possui um binding contextual read-only chamado `error`.
- `or` pode ser usado em uma linha (`load() or fallback()`) ou em bloco; no bloco, a última expressão produz o valor de fallback quando o fluxo não termina antes.
- `try expression` propaga a falha para quem chamou; `try` continua sendo operador sobre expressão, não bloco `try/catch`.
- Construções fallible de `struct` usam o mesmo modelo: `Player(...) or ...` para recuperação local e `try Player(...)` para propagação.
- Escritas em campos validados podem usar `field = value or handler`; em falha, a escrita não é efetivada e o valor anterior permanece.
- Fallbacks produzidos por `or` continuam sujeitos aos contratos normais de tipo do contexto.

**Em aberto / em teste:**

- A sintaxe `try field = value` foi rejeitada como superfície desejável e não entra como decisão.
- Será estudado um bloco de propagação para evitar repetir `try` em várias linhas consecutivas, preservando a mesma semântica de falha e sem introduzir `catch`/exceptions tradicionais.
- A forma exata de propagar uma falha de atribuição validada sem `try` prefixando a atribuição permanece aberta.

## Histórico superseded — experimento de bloco `try ... else ... end`

- `try expression` continua sendo a forma de propagar uma única operação fallible.
- `try ... end` delimita um bloco em que falhas recuperáveis não tratadas são propagadas automaticamente para a função chamadora.
- `try ... else ... end` trata localmente a primeira falha recuperável não tratada do bloco.
- Dentro de `else`, `error` fica disponível como binding read-only.
- `else` executa somente para falha recuperável; `none`, `false`, `0`, `""` e outros valores de sucesso não acionam o ramo.
- O bloco pode produzir valor: em sucesso, usa-se o resultado normal/última expressão relevante; em falha tratada, o ramo `else` pode fornecer o valor alternativo ou encerrar o fluxo.
- Um `or`/tratamento local dentro do bloco intercepta a falha daquela operação e impede que ela chegue ao `else` do bloco.
- O bloco `try` não é transacional: efeitos já executados não são revertidos automaticamente se uma operação posterior falhar.
- Runtime faults, como falhas numéricas de programação já classificadas como faults, não são convertidos em erros recuperáveis pelo bloco.
- A forma `try assignment` foi rejeitada; atribuições validadas em sequências podem ser cobertas naturalmente por um bloco `try`.

**Em aberto:** a grafia da recuperação de uma única operação (`value or fallback` versus alternativas como `value else fallback`).

## Expressão condicional e `else` de recuperação — decidido

- A forma antiga de `if` inline como statement fica substituída por uma **expressão condicional** iniciada por `if`.
- A expressão condicional sempre possui dois resultados: ramo verdadeiro e ramo alternativo; o `else` é obrigatório porque a expressão precisa produzir um valor em qualquer caminho.
- O `if` de bloco permanece inalterado: `if` / `elif` / `else` / `end` para controle de fluxo por statements.
- `elif` não fará parte da expressão condicional na V1; para mais casos, usar `match` ou `if` de bloco.
- A recuperação de uma única operação fallible passa a usar `expression else fallback`, substituindo a proposta anterior `expression or fallback`.
- `else` após uma operação fallible reage somente a erro recuperável; `none`, `false`, `0` e outros valores normais não acionam o fallback.
- `try expression` continua significando propagação de uma única falha; `try ... end` propaga falhas não tratadas de uma região; `try ... else ... end` trata a falha na fronteira do bloco.
- A grafia exata do separador entre a condição e o valor verdadeiro da expressão condicional ainda está aberta. `then` é o principal candidato, mas sua necessidade será avaliada antes de fechar a sintaxe final.

## Expressão condicional inline — decidido

A antiga forma de `if` inline como statement abreviado é substituída por uma **expressão condicional** própria.

Forma canônica:

```
if condition then value_if_true else value_if_false
```

Regras:

- A condição deve produzir `Bool`.
- `then` separa explicitamente a condição do valor do ramo verdadeiro.
- `else` é obrigatório, pois a construção sempre produz um valor.
- Apenas o ramo selecionado é avaliado.
- `elif` não faz parte da expressão condicional na V1; para múltiplos casos usa-se `if` em bloco ou `match`.
- O `if` de bloco permanece para controle de fluxo com statements:

```
if condition
    ...
elif other_condition
    ...
else
    ...
end
```

- `then` será tratado como **keyword contextual** dessa forma sintática, não como palavra reservada especulativamente em todos os contextos.

Princípio pedagógico:

> **`if ... end` controla execução; `if ... then ... else ...` escolhe um valor.**
> 

## Semântica de `expression else fallback` — decidido

- O `else` de recuperação tem precedência baixa e trata a falha recuperável da expressão completa à sua esquerda.
- Em uma expressão composta como `parse_json(read_file(path)) else default_config()`, o handler cobre qualquer falha recuperável ocorrida durante a avaliação de `read_file(path)` ou de `parse_json(...)` antes que a expressão produza seu valor.
- O binding contextual somente leitura `error` fica disponível durante a avaliação do fallback, tanto na forma inline quanto na forma em bloco.
- Se o próprio fallback falhar, essa nova falha segue a regra normal da linguagem e é propagada automaticamente, salvo se também for tratada.
- Encadeamento é permitido: `a else b else c` é avaliado da esquerda para a direita e equivale conceitualmente a `(a else b) else c`: tenta `a`; se falhar, tenta `b`; se `b` também falhar, tenta `c`.
- Runtime faults de programação continuam fora desse mecanismo e não são capturados por `else`.

**Princípio:** `else` representa uma alternativa somente para falhas recuperáveis; ausência (`none`), valores falsos e demais resultados normais não ativam o fallback.

## Atribuição validada sob propagação automática — decidido

- Atribuir a um campo com contrato ou regras declarativas é uma operação fallible normal.
- A escrita segue a ordem `candidate -> contrato de tipo -> regras declaradas -> commit`.
- Se contrato ou regra rejeitar o valor, a escrita não é efetivada e o valor anterior do campo é preservado.
- Sem handler local, a falha recuperável é propagada automaticamente ao chamador segundo o Modelo B; nenhum marcador como `try` é necessário.
- `field = value else handler` trata localmente a falha da atribuição validada e disponibiliza o binding contextual read-only `error` no handler.
- O `else` anexado à atribuição é um handler de falha; ele não transforma automaticamente a expressão à direita em um segundo valor para tentar atribuir. Para tentar outra escrita, ela deve ser escrita explicitamente no handler.
- Formas inline e em bloco de handler seguem o mesmo modelo geral de `else` de recuperação.

**Princípio:** atribuição validada não é uma exceção sintática; é apenas outra operação que pode falhar e cuja falha propaga até ser tratada.

## Regras declarativas de campo — aplicabilidade decidida

- `min(x)`, `max(x)` e `between(a, b)` aplicam-se a campos com contrato `Int` ou `Float`; limites `Int` podem participar de contratos `Float` pela promoção numérica já definida `Int -> Float`.
- `not_empty` aplica-se a `String`, `List` e `Dict` e exige comprimento maior que zero.
- `length(n)` aplica-se a `String`, `List` e `Dict`; para `String`, usa a semântica normal de comprimento em codepoints, não bytes.
- `one_of(...)` usa exatamente a semântica de `==` da linguagem e pode ser usado com valores comparáveis compatíveis com o contrato do campo.
- `check(function)` pode validar qualquer tipo e é a única regra declarativa permitida em campo sem contrato de tipo na V1.
- Built-ins dependentes de categoria (`min`, `max`, `between`, `not_empty`, `length`, `one_of`) exigem contrato de campo compatível na V1; não funcionam como um segundo sistema de tipos implícito para campos totalmente dinâmicos.
- Se o contrato for opcional (`T?`) e o valor candidato for `none`, as regras normais do valor são puladas: `none` é aceito pelo contrato e não é submetido a `min`, `not_empty`, `length` etc. Quando houver um valor `T`, as regras são avaliadas normalmente.
- Uma regra declarada de forma incompatível com o contrato do campo é erro de programa/declaração, preferencialmente diagnosticado pelo compilador, e não uma falha recuperável de dados.
- A ordem permanece: candidato -> contrato de tipo -> regras na ordem declarada -> commit. A primeira rejeição encerra a validação.

**Princípio:** regras declarativas refinam um contrato já conhecido; `check(...)` é o escape explícito para validação dinâmica/customizada.

## Avaliação dos argumentos de regras declarativas — decidido

- Os argumentos de regras built-in como `min`, `max`, `between`, `length` e `one_of` são resolvidos uma única vez quando a definição da `struct` é carregada/criada; não são reavaliados a cada construção nem a cada atribuição.
- Esses argumentos aceitam literais e bindings `let` já existentes/resolvidos no ponto da definição.
- Chamadas arbitrárias e outras expressões com efeitos, como `between(0, calculate_limit())`, ficam fora da V1 para preservar o caráter declarativo das regras.
- `check(fn)` continua sendo a porta explícita para lógica customizada; a função validator é resolvida na definição da `struct` e aplicada aos valores candidatos nas validações futuras.
- Configurações impossíveis ou incoerentes de regras são erro de programa/declaração, não falha recuperável de dados. Exemplos: `between(100, 0)`, `length(-1)`, `one_of()` sem opções ou argumento incompatível com o contrato do campo.
- Duplicatas em `one_of(...)` não invalidam a definição; tooling/linter pode emitir aviso.

**Princípio:** regras built-in descrevem restrições estáveis de dados; execução arbitrária deve permanecer explícita em `check(...)`.

## `fixed` e `invariant` — sintaxe canônica decidida

- A mini-DSL de regras por campo com `where`/`@(...)` foi removida da superfície V1.
- `fixed field: Type` é a forma canônica para um campo cuja identidade/valor de campo não pode ser substituído após a construção.
- Restrições de estado são declaradas em um bloco `invariant` dentro da `struct`, usando expressões Boolean normais da própria Aipo.

```
struct Player
    fixed id: Int
    name: String
    health: Int = 100

    invariant
        name != ""
        health >= 0
        health <= 100
    end
end
```

- Todas as expressões do bloco precisam ser verdadeiras após a construção e após uma mutação validada da própria struct.
- Na V1, invariants não dependem do estado mutável profundo de identidades gerenciadas alcançáveis por alias, evitando rastreamento oculto de aliasing.
- `fixed` controla substituição do campo; não significa deep freeze e não define visibilidade.

### Exemplos

```
struct Player
    id: Int where fixed
    name: String where not_empty
    health: Int where between(0, 100) = 100
    inventory: List where fixed = []
end
```

**Princípio:** preferir uma única forma textual, legível e autoexplicativa a atributos simbólicos redundantes.

## `match` V1 — decidido

- `match` será a construção canônica de seleção múltipla por valor; não haverá `switch` separado.
- Sintaxe canônica:

```
match expression
when value [, value...]
    statements
when value [, value...]
    statements
else
    statements
end
```

- Cada `when` compara o valor do `match` usando o operador `==` já definido pela linguagem.
- Um mesmo ramo pode listar múltiplos valores separados por vírgula; eles são testados da esquerda para a direita.
- `else` é opcional. Se nenhum `when` corresponder e não houver `else`, o `match` simplesmente não executa nenhum ramo.
- `match` será statement-only na V1; não produzirá valor diretamente.
- Ficam fora da V1: destructuring, patterns estruturais, ranges como patterns, guards em `when`, pattern matching por tipo e demais formas avançadas.
- Para condições booleanas arbitrárias, usar `if` / `elif` / `else`; para teste de tipo, usar `value is Type`.

**Princípio:** `if` pergunta se uma condição é verdadeira; `match` pergunta qual destes valores é o valor observado.

## Interfaces — implementação default fora da V1 — decidido

- Interfaces da Aipo V1 conterão **somente assinaturas de operações**; não haverá corpos/default implementations dentro de `interface`.
- Interfaces descrevem capacidades/contratos, não fornecem comportamento nem funcionam como traits/mixins.
- Comportamento reutilizável será escrito em funções normais que recebem um valor contratado pela interface; o dot-call pode manter a ergonomia de chamada sem injetar comportamento na struct.
- Essa separação preserva o modelo: `struct` descreve dados, `interface` descreve capacidades e `fn` fornece comportamento.
- A sintaxe exata para declarar explicitamente a intenção de uma struct satisfazer uma ou mais interfaces permanece em discussão.

## Módulos e import — decidido

- Na V1, **cada arquivo `.aipo` define um módulo**.
- O caminho de diretórios forma o caminho lógico do módulo, por exemplo `game/player.aipo` → `game.player`.
- `import game.player` importa o módulo como namespace e cria localmente o nome final `player`; o acesso normal permanece qualificado, por exemplo `player.Player(...)`.
- `as` permite alias explícito quando necessário, por exemplo `import editor.player as editor_player`.
- A V1 também permite **múltiplos módulos na mesma declaração `import`**, separados por vírgula:

```
import game.player, game.enemy, ui.button
```

- Não haverá import seletivo que espalhe nomes individuais no escopo local na V1; formas equivalentes a `from x import A, B` ficam fora do núcleo inicial.
- Código executável no top-level é permitido, preservando o foco scripting-first; `fn main()` não é obrigatório pela linguagem.
- Cada módulo é inicializado/executado no máximo uma vez por execução do programa; imports posteriores reutilizam o módulo já carregado.
- Dependências circulares entre módulos são proibidas na V1 e devem produzir diagnóstico com o ciclo encontrado.

**Princípio:** um arquivo define um módulo; `import` traz o módulo, não espalha seus nomes pelo escopo atual.

## Export e reexport de módulos — decidido

- Declarações de módulo são **privadas por padrão**. Só entram na API pública do módulo quando listadas explicitamente em `export`.
- A forma canônica será uma lista de nomes, separada das declarações, por exemplo:

```
export Player, create_player, Drawable
```

- `export` pode publicar declarações locais relevantes do módulo, como `struct`, `interface`, funções e bindings top-level apropriados. Visibilidade de módulo não exporta campos individuais de `struct`.
- `import` e `export` têm papéis distintos: `import` torna um módulo disponível para uso interno; isso **não** o reexporta automaticamente.
- A mesma keyword `export` também realiza reexport; não haverá uma keyword separada `reexport` na V1.
- Um nome público importado pode ser reexportado de forma achatada:

```
import game.player, game.enemy
export player.Player, enemy.Enemy
```

Quem importar esse módulo poderá usar `game.Player` e `game.Enemy`.

- Um submódulo inteiro também pode ser reexportado:

```
import game.player
export player
```

Nesse caso, quem importar o módulo fachada poderá usar `game.player.Player`.

- Reexport respeita a API do módulo de origem: somente nomes já públicos no módulo importado podem ser reexportados; um módulo não pode tornar público um detalhe privado de outro.
- Exportar um nome importado não copia nem recria a declaração; apenas a inclui na API pública do módulo atual.
- O objetivo é permitir módulos-fachada sem espalhar dependências internas pela API e sem introduzir outro conceito sintático para reexport.

**Princípio:** `import` define o que o módulo atual pode usar; `export` define o que quem importa esse módulo pode usar.

## Aliases e colisões em `export` — decidido

- `export` aceita alias explícito com `as` para declarações locais, nomes reexportados e submódulos reexportados.
- Exemplos válidos: `export create_player as create`, `export player.Player as GamePlayer` e `export player as game_player`.
- `as` altera somente o nome público visto por quem importa o módulo; não renomeia a declaração ou o binding original dentro do módulo exportador.
- Múltiplos itens continuam permitidos na mesma linha, inclusive combinando itens com e sem alias, por exemplo `export player.Player as Player, enemy.Enemy, camera.Camera`.
- O nome público de cada item é o alias quando `as` está presente; caso contrário, é o último nome do item exportado (`Player` em `player.Player`).
- Cada nome público de um módulo deve ter **uma única origem inequívoca**.
- Duas declarações, reexports ou combinações de declaração local + reexport que produzam o mesmo nome público geram erro de compilação.
- Não haverá sobrescrita silenciosa nem regra de “último export vence”. O programador deve resolver a colisão explicitamente com `as` ou removendo um dos exports.
- O diagnóstico de colisão deve mostrar o nome público duplicado e suas origens quando possível.

**Princípio:** a API pública de um módulo é explícita e não ambígua; `as` resolve nomes, nunca esconde uma colisão silenciosamente.

## Mutabilidade através de módulos importados — decidido

- O namespace criado por `import` é sempre **somente leitura** para o módulo importador.
- `export` concede acesso público ao binding, mas não transfere autoridade de reatribuição ou mutação sobre o estado pertencente ao módulo de origem.
- Um binding top-level declarado com `var` continua mutável para o próprio módulo que o possui, mas não se torna publicamente gravável apenas por ter sido exportado.
- Assim, atribuições como `config.version = "2.0"` ou `config.settings = {}` são inválidas fora do módulo de origem.
- A regra de mutabilidade por caminho também se aplica ao conteúdo alcançado através do namespace importado: operações mutáveis como `config.settings["theme"] = "light"` ou métodos que exigem acesso mutável são inválidos através desse caminho somente leitura.
- Não é permitido criar um alias mutável a partir de um caminho iniciado em namespace importado somente leitura, por exemplo `var settings = config.settings` quando `settings` é um valor gerenciado.
- Se um módulo quiser permitir mudanças em seu estado, deve expor funções públicas que realizem a mutação internamente, como `config.set_theme("light")`.

**Princípio:** `export` concede acesso; não transfere autoridade de mutação sobre os bindings e o estado possuídos pelo módulo.

## Resolução de caminhos de módulos — decidido

- Imports de módulos do projeto usam **caminhos absolutos a partir de uma raiz de módulos definida pelo ambiente de execução/projeto**.
- O mesmo módulo deve ter uma forma canônica de referência; imports relativos como `.player`, `..player` ou `../player` ficam fora da V1.
- `import game.player` resolve sempre o mesmo módulo independentemente da pasta do arquivo que faz o import.
- Por padrão, o último segmento do caminho vira o nome local: `import game.player` cria o namespace local `player`.
- `as` permite renomear o namespace importado quando necessário, inclusive para resolver colisões.
- Colisões de nomes locais de módulos importados são erro; não há sobrescrita silenciosa.
- A biblioteca padrão usa a raiz reservada `std`, por exemplo `import std.fs, std.math, std.json`.
- O formato exato do manifesto, a convenção de diretório `src/` e a resolução de packages externos permanecem fora desta decisão e serão tratados separadamente.

**Princípio:** cada módulo possui uma referência canônica e previsível; a localização do arquivo atual não altera o significado de um import.

## Scripts e projetos — decidido

Aipo distingue **script simples** de **projeto estruturado** sem obrigar todo programa a usar manifesto.

- Um arquivo `.aipo` isolado pode ser executado diretamente, sem manifesto obrigatório.
- Em modo script, a pasta do arquivo executado funciona como raiz de módulos para imports locais.
- Projetos estruturados usam opcionalmente um manifesto `aipo.toml` na raiz do projeto.
- A pasta que contém `aipo.toml` define a raiz do projeto.
- Em projetos, `src/` é a raiz padrão do código importável e dos caminhos lógicos de módulo; `src` não aparece no caminho de `import`.
- Assim, `src/game/player.aipo` corresponde a `import game.player`.
- O manifesto começa pequeno e declarativo; configuração de dependências/packages será definida separadamente.
- Forma inicial aprovada do manifesto:

```toml
[project]
name = "my_game"
version = "0.1.0"
entry = "main"
```

- `entry = "main"` referencia o módulo `src/main.aipo`.
- O entry point de um projeto é um **módulo executável**, aproveitando top-level code; `fn main()` não é obrigatório.
- A ferramenta pode localizar o projeto subindo a árvore de diretórios até encontrar `aipo.toml`.

**Princípio:** scripts continuam imediatos e sem cerimônia; projetos ganham uma raiz, organização e metadata explícitas somente quando necessário.

## Packages e dependências — decidido

Aipo adotará um modelo pequeno e explícito de packages na V1.

- Um **package** é um projeto Aipo reutilizável, com `aipo.toml` e código sob `src/`; aplicação e biblioteca compartilham o mesmo sistema de módulos.
- Dependências são declaradas em `[dependencies]` no manifesto.

```toml
[dependencies]
json_tools = "2.1.3"
physics = { path = "../physics" }
math = { package = "math_extra", version = "1.2.0" }
```

- O nome da dependência, ou seu alias no manifesto, torna-se a raiz de módulos daquele package. Exemplos: `import json_tools.parser`, `import physics.body`, `import math.vector`.
- A V1 usa **versões exatas** para dependências publicadas; ranges semânticos como `^`, `~` e comparadores ficam fora do modelo inicial.
- `aipo.lock` é gerado automaticamente para registrar a resolução completa e reproduzível do grafo, incluindo dependências transitivas.
- Dependências locais são suportadas com `path` e mantêm o mesmo namespace usado por uma dependência publicada.
- Alias de package é permitido no manifesto com `package = "nome_original"`; o código usa o alias como raiz de import.
- Não há resolução silenciosa por precedência: se uma raiz de módulo puder vir de mais de uma origem — por exemplo módulo local e package com o mesmo nome — isso é erro de projeto e exige renomeação/alias explícito.
- `std` permanece reservado exclusivamente à biblioteca padrão e não pode ser usado como raiz de package externo.
- Registry oficial, publicação, Git dependencies, workspaces, features, grupos de dependência, scripts de build e ranges sofisticados ficam para decisões posteriores.

**Princípio:** o código importa por uma raiz estável; a origem concreta da dependência pertence ao manifesto e à resolução do projeto, não à sintaxe da linguagem.

## Dependências transitivas — decidido

- Somente packages declarados diretamente no `aipo.toml` do projeto concedem namespaces importáveis ao código desse projeto.
- Dependências transitivas podem existir no grafo resolvido e no `aipo.lock`, mas não ficam automaticamente disponíveis para `import`.
- Se um projeto quiser importar diretamente um package que hoje chega apenas de forma transitiva, deverá declará-lo explicitamente em `[dependencies]`.
- Um package pode reexportar deliberadamente símbolos públicos vindos de uma dependência própria; nesse caso, consumidores usam esses símbolos através da API pública do package reexportador, sem ganhar acesso direto ao namespace da dependência transitiva.
- Mudanças internas no grafo transitivo de um package não devem, por si só, quebrar imports de seus consumidores.

**Princípio:** dependência direta concede namespace; dependência transitiva é detalhe interno.

## Packages — resolução de versão única decidida

A V1 adota **single-version resolution** para packages.

- Para cada identidade real de package, existe no máximo **uma única versão resolvida** em todo o grafo de dependências de uma execução/projeto.
- Se duas dependências exigirem versões exatas diferentes do mesmo package, a resolução falha com diagnóstico explícito mostrando as cadeias conflitantes.
- A V1 não permite coexistência silenciosa de múltiplas versões do mesmo package.
- Alias de dependência altera apenas o namespace local usado nos imports; **não altera a identidade real do package** e não pode ser usado para contornar conflito de versão.
- Dependências por `path` seguem a mesma regra: mudar a origem física não cria uma nova identidade lógica de package.
- O `aipo.lock` registra a versão única resolvida de cada package e não deve conter duas versões diferentes para a mesma identidade de package na V1.
- O resolvedor não escolhe automaticamente uma versão diferente daquela declarada para esconder conflito; conflitos devem ser resolvidos deliberadamente pelo projeto ou pelas dependências envolvidas.

**Princípio:** um package possui uma única versão efetiva por grafo, mantendo identidade de tipos, módulos, tooling e runtime inequívocos.

## Packages — identidade e versionamento — decidido

- A identidade real de um package é definida por `project.name` no `aipo.toml`; o nome da pasta, caminho físico ou alias do consumidor não alteram essa identidade.
- Nomes de package usam `snake_case`, como `json_tools`, `math_extra` e `http_client`. Hífens, `PascalCase` e pontos ficam fora da identidade de package na V1.
- A versão declarada usa o formato `MAJOR.MINOR.PATCH`, por exemplo `1.4.2`.
- A V1 adota o formato e a disciplina conceitual de Semantic Versioning: `MAJOR` para mudança incompatível, `MINOR` para funcionalidade compatível e `PATCH` para correção compatível.
- O resolvedor continua usando **versões exatas**; o formato SemVer não implica ranges automáticos na V1.
- Versões `0.x.y` indicam API ainda em desenvolvimento, sem estabilidade garantida, mas continuam sendo tratadas como versões exatas normais pelo resolvedor.
- Um alias de dependência muda apenas o namespace local usado pelo consumidor. Exemplo: `math = { package = "math_extra", version = "2.3.1" }` continua apontando para a identidade real `math_extra`.
- Aliases diferentes para o mesmo package e mesma versão não criam packages, módulos ou tipos distintos; a identidade interna permanece ligada ao package real e ao caminho real do módulo/tipo, não ao alias local.
- Dependências locais via `path` devem ter manifesto coerente com a identidade esperada; o caminho físico não substitui `project.name`.

**Princípio:** o manifesto define quem o package é; o alias define apenas como o consumidor o chama.

## Raiz pública de package — decidido

- O **package** é o projeto/diretório identificado pelo seu `aipo.toml`; `package.aipo` não é o package em si.
- A V1 manterá **`.aipo` como única extensão de código-fonte**. Não haverá uma extensão separada para arquivos de package.
- Um package pode definir opcionalmente `src/package.aipo` como seu **módulo-raiz público/fachada**.
- `package.aipo` é um arquivo Aipo normal: usa o mesmo parser, semântica, `import`, `export`, contratos e execução de módulo que qualquer outro `.aipo`. Sua especialidade vem apenas da posição/nome convencional.
- `import package_name` resolve para `src/package.aipo` daquele package.
- Arquivos normais continuam formando submódulos normalmente: por exemplo, `src/vector.aipo` corresponde a `package_name.vector`.
- Se um package não possuir `src/package.aipo`, seus submódulos continuam importáveis, mas `import package_name` gera diagnóstico claro informando que o package não define módulo-raiz público.
- A fachada não exporta submódulos automaticamente; sua API pública é definida explicitamente por `export` e reexports.
- `src/main.aipo` e `src/package.aipo` têm papéis distintos e podem coexistir no mesmo projeto: `main.aipo` é entry point executável; `package.aipo` é a API da raiz importável.
- Aplicações e bibliotecas continuam usando o mesmo modelo de projeto/package; não será introduzida uma separação rígida entre tipos de projeto na V1.

**Princípio:** `aipo.toml` define o package; `package.aipo` define, quando presente, a sua fachada pública na raiz do namespace.

## Packages — importação da raiz e submódulos — decidido

- `import package_name` importa somente a fachada pública definida em `src/package.aipo`.
- Apenas nomes explicitamente exportados por `package.aipo` ficam acessíveis pela raiz, por exemplo `package_name.Vector` ou `package_name.distance(...)`.
- Importar a raiz do package não disponibiliza automaticamente seus submódulos, diretórios internos ou namespaces intermediários.
- Para acessar diretamente um submódulo, o código deve importá-lo explicitamente, por exemplo `import package_name.vector`.
- A existência de um submódulo físico não o torna membro automático da fachada pública do package.
- `package.aipo` funciona como fachada estável, permitindo reorganizar módulos internos sem quebrar consumidores que dependem apenas da raiz pública.

**Princípio:** importar a raiz de um package importa sua fachada; importar um submódulo é uma decisão explícita separada.

## Visibilidade externa de submódulos de package — decidido

- A existência física de um arquivo ou diretório sob `src/` **não torna automaticamente esse módulo importável por consumidores externos do package**.
- A estrutura interna do filesystem não é, por si só, parte da API pública do package.
- Um caminho como `math_extra.internal.cache` não pode ser importado externamente apenas porque `src/internal/cache.aipo` existe.
- A exposição de submódulos precisa ser deliberada e declarada explicitamente pelo package; a forma sintática exata dessa declaração será fechada separadamente.
- Módulos internos continuam disponíveis normalmente para outros módulos do próprio package, conforme as regras de imports internos.

**Princípio:** a organização física implementa o package; somente declarações explícitas definem sua superfície pública.

## Visibilidade recursiva de submódulos — decidido

- A existência física de um arquivo ou diretório em `src/` **não** o torna automaticamente importável por consumidores externos.
- A V1 não adicionará keywords separadas como `pub`, `public` ou `private` para módulos. A mesma construção `export` controla a superfície pública.
- Um submódulo só pode ser atravessado externamente quando o módulo-pai exporta explicitamente esse namespace.
- `export module.Symbol` expõe o símbolo reexportado, mas **não** torna automaticamente importável o caminho interno que o originou.
- `export module` expõe o namespace do módulo naquele nível.
- Para caminhos profundos, a regra é recursiva: cada módulo intermediário controla quais namespaces filhos atravessam sua própria fronteira pública.

Exemplo:

```
# src/package.aipo
import tools
export tools

# src/tools.aipo
import tools.parser, tools.formatter
export parser, formatter
```

Isso permite:

```
import package_name.tools.parser
import package_name.tools.formatter
```

Um módulo filho não exportado continua inacessível externamente, mesmo que o arquivo exista fisicamente.

**Princípio:** arquivos definem módulos; `export` define quais nomes e namespaces atravessam a fronteira pública. Um caminho externo só existe quando todos os segmentos necessários foram deliberadamente expostos.

## Packages — fronteira interna vs. externa — decidido

- `export` controla somente o que atravessa a fronteira pública do package.
- Módulos pertencentes ao mesmo package podem importar outros módulos do próprio package mesmo quando esses módulos não foram exportados publicamente.
- Um módulo interno pode, por exemplo, fazer `import internal.cache` sem tornar `internal.cache` acessível a consumidores externos.
- Para consumidores de outro package, continuam valendo as regras de visibilidade pública já decididas: apenas caminhos explicitamente exportados existem externamente.
- Não serão introduzidos conceitos adicionais como `friend`, `internal`, `private` ou `pub(crate)` para essa colaboração interna na V1.
- A organização interna do package pode mudar sem transformar automaticamente esses caminhos em parte da API pública.

**Princípio:** `export` controla o que atravessa a fronteira do package; não restringe imports entre módulos pertencentes ao mesmo package.

## Resolução interna de imports em packages — decidido

- Dentro do próprio package/projeto, imports usam caminhos absolutos a partir de `src/`, sem prefixar o nome do package.
- Exemplo: `src/internal/cache.aipo` é importado internamente como `import internal.cache`.
- O nome do package, definido em `aipo.toml`, só entra no caminho quando esse package é consumido externamente por outro projeto/package.
- Assim, um consumidor externo pode usar `import graphics.renderer`, enquanto um módulo pertencente ao próprio package `graphics` usa `import renderer` ou `import internal.cache`.
- A forma interna `import graphics.internal.cache` não é válida como segunda forma alternativa para o mesmo módulo; a V1 mantém uma referência canônica em cada contexto de resolução.
- Renomear o package no manifesto não exige reescrever os imports internos entre seus próprios módulos.

**Princípio:** dentro do package, `src/` é a raiz canônica; fora dele, o nome/alias da dependência é a raiz canônica.

## Módulos e submódulos no filesystem — decidido

Aipo mantém a regra fundamental **um arquivo `.aipo` = um módulo**. Diretórios não se tornam módulos implicitamente; eles apenas organizam possíveis submódulos.

- `src/foo.aipo` define o módulo `foo`.
- `src/foo/bar.aipo` define o módulo `foo.bar`.
- `src/foo/bar/baz.aipo` define o módulo `foo.bar.baz`.
- Um arquivo `foo.aipo` pode coexistir com um diretório `foo/`.
- Nessa combinação, `foo.aipo` pode funcionar como fachada/controlador do namespace `foo`, enquanto `foo/*.aipo` define seus submódulos.
- Não haverá `package.aipo` dentro de subdiretórios para representar módulos intermediários; `package.aipo` permanece reservado ao módulo-raiz público opcional do package inteiro.

### Exemplo

```
src/
    tools.aipo
    tools/
        parser.aipo
        formatter.aipo
```

Mapeamento lógico:

```
tools.aipo             -> tools
tools/parser.aipo      -> tools.parser
tools/formatter.aipo   -> tools.formatter
```

`tools.aipo` pode controlar a superfície pública de seus filhos:

```
import tools.parser, tools.formatter

export parser, formatter
```

**Princípio:** um módulo é um arquivo; um diretório apenas organiza os possíveis submódulos desse módulo.

## Módulos e diretórios organizacionais — decidido

- Um módulo Aipo só existe quando há um arquivo `.aipo` correspondente.
- Diretórios podem formar segmentos de caminho lógico sem definirem módulos por si mesmos.
- Assim, `src/game/player.aipo` define o módulo `game.player` mesmo que `src/game.aipo` não exista.
- Nesse caso, `import game.player` é válido, enquanto `import game` é inválido porque não existe um módulo `game`.
- Se posteriormente `src/game.aipo` for criado, passam a existir simultaneamente `game`, `game.player`, `game.enemy` e demais filhos correspondentes.
- Um arquivo-pai como `game.aipo` pode servir de fachada para seus submódulos através de `import` + `export`, mas essa fachada é opcional.
- Diretórios permanecem apenas organização física e composição de caminho; eles não executam código, não possuem exports próprios e não criam namespaces importáveis isoladamente.

**Princípio:** diretórios podem formar segmentos de caminho; módulos são definidos exclusivamente por arquivos `.aipo`.

## API pública e segmentos intermediários — decidido

- Diretórios podem existir apenas como organização interna de caminhos e não criam módulos por si mesmos.
- Para que um caminho de módulo seja exposto publicamente através da fronteira de um package, cada segmento intermediário desse caminho precisa corresponder a um módulo `.aipo` real.
- Esse módulo intermediário controla deliberadamente sua própria superfície pública por meio de `export`.
- Exemplo: para permitir `import my_game.game.player`, o package precisa conter `src/game.aipo` e `src/game/player.aipo`; `package.aipo` deve exportar `game`, e `game.aipo` deve exportar `player`.
- Um diretório como `src/game/` sem `src/game.aipo` pode continuar organizando módulos internos como `game.player`, mas não cria por si só um namespace público atravessável externamente.
- Reorganizações puramente internas de diretórios não devem alterar silenciosamente a API pública do package.

**Princípio:** segmentos organizacionais podem existir internamente; todo segmento de uma API pública precisa ser um módulo real que exponha deliberadamente o próximo nível.

## Inicialização de módulos e resolução de caminhos públicos — decidido

- Resolver um caminho público não executa automaticamente os módulos intermediários usados apenas para compor a superfície pública do namespace.
- `import my_game.graphics.sprite` carrega e executa o módulo `graphics.sprite` e os módulos que ele importar de fato; não executa `package.aipo` nem `graphics.aipo` apenas por esses módulos participarem da cadeia pública do caminho.
- `import my_game.graphics` executa `graphics.aipo`.
- `import my_game` executa `package.aipo`.
- Cada módulo executável continua sendo inicializado no máximo uma vez por execução do programa; imports posteriores reutilizam o módulo já carregado.
- Módulos intermediários podem existir para controlar `export` e visibilidade externa sem introduzir efeitos colaterais implícitos em imports de submódulos mais específicos.
- A resolução de namespace e a inicialização de módulo são operações conceitualmente distintas.

**Princípio:** resolver um namespace público não significa executar todos os módulos que formam esse caminho.

## Inicialização e falha de módulos — decidido

- Cada módulo é inicializado no máximo uma vez por execução.
- O runtime pode distinguir internamente estados equivalentes a `not_loaded`, `initializing`, `initialized` e `failed`; esses estados são detalhe de implementação.
- Se a inicialização top-level de um módulo concluir com sucesso, o módulo fica disponível e imports seguintes reutilizam a mesma instância inicializada.
- Se a inicialização produzir uma falha recuperável, essa falha propaga normalmente pelo Modelo B para o importador.
- Um módulo cuja inicialização falhou não publica bindings, exports ou estado parcialmente inicializados.
- O runtime não tenta reinicializar automaticamente um módulo que já terminou em falha durante a mesma execução; imports posteriores do mesmo módulo observam a falha já registrada, em vez de repetir a inicialização e seus possíveis efeitos colaterais.
- Efeitos colaterais executados antes da falha não são revertidos automaticamente.
- Como ciclos de módulos são proibidos na V1, o estado interno de `initializing` não precisa ser exposto como conceito ao programador.

**Princípio:** **um módulo é inicializado uma única vez por execução: ou termina inicializado, ou termina falho; uma falha de inicialização propaga normalmente e nunca publica um módulo parcialmente inicializado.**

## Inicialização top-level e resolução de declarações — decidido

- O **código executável** de um módulo continua sendo executado de cima para baixo, na ordem textual em que foi escrito.
- `import` pertence à fase estrutural do módulo: dependências são resolvidas estaticamente, e os módulos realmente importados são inicializados antes de começar o código executável top-level do módulo atual. A posição textual do `import` não define um ponto de inicialização runtime.
- Aipo não terá hoisting de valores ao estilo JavaScript. Bindings `let` e `var`, atribuições e demais statements executáveis só existem/produzem efeitos a partir do ponto em que a execução os alcança.
- Declarações estruturais de top-level `fn`, `struct` e `interface` são conhecidas pelo módulo independentemente de sua posição textual. Elas podem ser referenciadas por código ou por outras declarações do mesmo módulo mesmo quando aparecem depois no arquivo.
- Portanto, uma função pode chamar outra declarada posteriormente e recursão mútua entre funções não exige forward declarations adicionais.
- O mesmo princípio permite referências entre `struct` e `interface` declaradas em ordem diferente, respeitando naturalmente as demais regras semânticas e de contratos da linguagem.
- Essa resolução antecipada de declarações estruturais é uma fase estática do módulo; não significa que statements, bindings ou inicializações sejam executados antecipadamente.
- `export` é declaração estrutural totalmente estática da superfície pública e não produz efeito runtime por si só.
- Sua resolução é independente da posição textual: pode referenciar declarações locais ou nomes importados que apareçam antes ou depois no arquivo, inclusive bindings top-level `let`/`var` que serão inicializados posteriormente na fase executável.
- Resolver um `export` antecipadamente não inicializa nem antecipa o valor exportado; bindings continuam adquirindo valor somente quando sua inicialização top-level é alcançada.
- Nenhum export se torna observável externamente até que a inicialização completa do módulo termine com sucesso. Se a inicialização falhar, o módulo não publica API parcial.
- Exportar um binding mutável concede visibilidade, não autoridade externa para mutá-lo; a regra de namespace importado somente leitura permanece válida.

**Princípio:** **`export` descreve estaticamente a superfície pública final de um módulo; sua posição textual não importa e nenhum nome exportado se torna observável até que a inicialização do módulo termine com sucesso.**

**Princípio:** **declarações estruturais podem ser conhecidas independentemente da ordem; execução e valores continuam seguindo a ordem textual.**

- O objetivo é manter previsível a parte executável: depois que a estrutura e as dependências do módulo forem resolvidas, statements top-level são executados de cima para baixo.

**Princípio:** **estrutura do módulo é resolvida estaticamente; código executável é executado de cima para baixo.**

## Imports e exports no top-level — decidido

- Na V1, `import` e `export` só podem aparecer no **top-level do módulo**.
- Eles não podem aparecer dentro de funções, condicionais, loops, `try` ou qualquer outro bloco local.
- `import` e `export` podem aparecer em qualquer ponto do top-level; sua resolução estrutural não depende da posição textual. Formatter/linter pode agrupá-los perto do início por estilo.
- A posição de um `import` no arquivo não cria um ponto temporal de carregamento. Dependências são resolvidas e inicializadas antes da execução dos statements top-level do módulo atual.
- Ferramentas podem recomendar agrupar imports perto do início por estilo, mas isso não altera a semântica.
- `export` permanece declaração estrutural da API pública e não produz efeito runtime por si só.

```
io.print("before")

import config

io.print("after")
```

Válido no top-level.

```
fn load()
    import config
end
```

Inválido na V1.

**Princípio:** `import` e `export` descrevem a estrutura entre módulos; por isso pertencem ao módulo, não ao fluxo local de uma função ou bloco.

## `export` e referências posteriores — decidido

- `export` é uma declaração estrutural da API pública e é resolvido estaticamente considerando todo o módulo.
- Um `export` pode referenciar qualquer declaração top-level válida do mesmo módulo, mesmo quando essa declaração aparece depois no arquivo.
- Isso inclui `fn`, `struct`, `interface` e também bindings top-level como `let` e `var`.
- A posição textual do `export` não antecipa, executa nem inicializa o valor exportado.
- A ordem de inicialização continua sendo determinada exclusivamente pelo fluxo top-level executável e pelos `import` encontrados durante essa execução.
- Como módulos só são publicados após inicialização bem-sucedida, consumidores externos nunca observam um binding exportado antes de ele estar efetivamente inicializado.

Exemplo válido:

```
export Player, create_player, version

struct Player
    name: String
end

fn create_player(name)
    return Player(name)
end

let version = "1.0.0"
```

**Princípio:** **`export` descreve estaticamente o que será público; ele não interfere na ordem em que o módulo é executado ou inicializado.**

## `satisfy` e ordem de declarações — decidido

- `satisfy` é uma declaração estrutural de análise semântica; não executa código e não participa da ordem de inicialização top-level.
- Pode referenciar o tipo concreto e as interfaces correspondentes mesmo quando essas declarações aparecem depois no mesmo módulo.
- A verificação ocorre depois que o módulo teve suas declarações estruturais coletadas; ao fim da análise semântica, todos os nomes referenciados precisam existir e a conformidade precisa satisfazer as regras oficiais de assinatura.
- Isso vale para exemplos como:

```
satisfy Sprite: Drawable, Resettable

struct Sprite
    texture
end

interface Drawable
    fn draw(value)
end

interface Resettable
    fn reset(value!)
end
```

- Não são necessárias forward declarations apenas para ordenar `satisfy`, `struct` e `interface`.
- Essa regra é coerente com a resolução estática order-independent já adotada para `fn`, `struct`, `interface` e `export`.

**Princípio:** **`satisfy` declara uma relação estrutural do módulo; sua validade depende do conjunto final de declarações, não da posição textual em que aparece.**

## Imports na fase estrutural — decidido

- Declarações estruturais locais do módulo, como `fn`, `struct`, `interface`, `satisfy` e referências usadas por `export`, podem ser resolvidas independentemente da ordem textual conforme suas regras próprias.
- Namespaces introduzidos por `import` participam da resolução estrutural do módulo e podem ser referenciados independentemente da posição textual do `import` no top-level.
- Portanto, código ou declarações podem referenciar um namespace importado mesmo quando o `import` aparece posteriormente no arquivo; isso não antecipa execução de statements nem valores.
- Exemplo válido:

```
import std.json

fn load() -> json.Value
    return json.parse("{}")
end
```

- Também válido:

```
fn load() -> json.Value
    return json.parse("{}")
end

import std.json
```

- A regra evita criar uma espécie de temporal dead zone para módulos, na qual um namespace seria conhecido estaticamente antes de seu ponto real de introdução e inicialização.

**Regra supersedida:** a posição textual de `import` não limita mais a resolução de namespaces; vale o modelo estrutural de duas fases definido abaixo.

## Modelo de duas fases do módulo — decidido

Aipo separa claramente **estrutura** e **execução** do módulo.

- Fase estrutural, independente da ordem textual: `import`, `export`, `satisfy`, `fn`, `struct` e `interface` são coletados e resolvidos para o conjunto completo do módulo.
- Referências entre essas declarações podem apontar para declarações que aparecem antes ou depois no arquivo, inclusive namespaces introduzidos por `import` e reexports por `export`.
- `import` não representa um statement temporal de carregamento. Os módulos realmente importados e suas dependências são resolvidos/inicializados antes do código executável top-level do módulo atual, preservando as regras já decididas de inicialização única e de não executar fachadas intermediárias apenas para resolver um namespace público.
- Fase executável, dependente da ordem textual: bindings `let`/`var`, expressões, chamadas, atribuições, controle de fluxo e demais statements executáveis rodam de cima para baixo.
- Não há hoisting de valores: um binding top-level só possui valor depois que sua inicialização executável foi alcançada.
- Formatter/linter pode organizar `import`/`export` no início do arquivo por estilo, sem efeito semântico.

**Princípio:** **a estrutura do módulo não depende da posição; a execução depende.**

## Ordem canônica de inicialização das dependências — decidido

- `import` é uma declaração estrutural e sua posição textual não determina a ordem de inicialização das dependências.
- Antes de executar o código top-level de um módulo, o runtime inicializa transitivamente todas as dependências exigidas por esse módulo.
- A ordem respeita primeiro as relações do grafo de dependências: uma dependência é inicializada antes do módulo que depende dela.
- Quando duas ou mais dependências são independentes entre si e poderiam ser inicializadas em qualquer ordem topologicamente válida, o desempate usa o **caminho canônico do módulo** em ordem determinística.
- Reordenar linhas `import` sem alterar o conjunto de dependências não muda a ordem de inicialização nem o comportamento observável do programa.
- Cada módulo continua sendo inicializado no máximo uma vez por execução; dependências compartilhadas são reutilizadas após a primeira inicialização bem-sucedida.
- Ciclos de módulos continuam proibidos na V1, portanto o grafo de inicialização deve ser acíclico e qualquer ciclo é diagnosticado antes da execução normal.

**Princípio:** **a posição das declarações não altera a semântica estrutural nem a ordem de inicialização; somente o código executável possui ordem textual observável.**

## Imports como dependências de inicialização — decidido

- Todo `import` representa uma dependência real de inicialização do módulo atual.
- Todas as dependências importadas são inicializadas **antes** de qualquer código executável top-level do módulo importador começar.
- Isso vale mesmo quando o namespace importado só aparece dentro de uma função que nunca é chamada, ou quando nenhum uso runtime ocorre naquela execução.
- Imports não são lazy na V1: a inicialização da dependência não é adiada até o primeiro uso.
- A ordem entre dependências independentes segue a ordem canônica já definida pelo grafo de módulos, com desempate pelo caminho canônico; a posição textual dos `import` não altera essa ordem.
- Se qualquer dependência falhar durante sua inicialização, o código executável top-level do módulo atual não começa; a falha propaga normalmente pelo Modelo B.
- Uma dependência já inicializada com sucesso é reutilizada e não executa seu top-level novamente durante a mesma execução.

**Princípio:** **todo módulo importado é uma dependência de inicialização; as dependências terminam de inicializar antes que o módulo importador execute seu próprio código top-level.**

## Imports — modelo V1 finalizado

A V1 fecha `import` como uma **declaração estrutural de dependência de módulo**, não como statement temporal de carregamento.

- `import` só é permitido no top-level do módulo.
- `import`, `export`, `satisfy`, `fn`, `struct` e `interface` pertencem à fase estrutural e são resolvidos independentemente da posição textual.
- Um namespace importado pode ser referenciado em qualquer declaração ou função do mesmo módulo, mesmo que a linha `import` apareça depois no arquivo.
- A posição textual de `import` não define disponibilidade, ordem de inicialização ou ponto de execução; formatter/linter pode agrupá-los no início apenas por estilo.
- Todo `import` cria uma dependência real e **eager**: todas as dependências importadas são inicializadas antes de qualquer código executável top-level do módulo importador.
- Imports não são lazy na V1; primeiro uso runtime não controla inicialização.
- A ordem de inicialização respeita o grafo de dependências; dependências vêm antes de seus consumidores. Entre dependências independentes, o desempate é determinístico pelo caminho canônico do módulo.
- Reordenar linhas `import` sem mudar o conjunto de dependências não altera o comportamento do programa.
- Cada módulo é inicializado no máximo uma vez por execução; dependências compartilhadas reutilizam a mesma inicialização e identidade de módulo.
- Ciclos de módulos continuam proibidos e são diagnosticados antes da execução normal.
- Se uma dependência falhar ao inicializar, o top-level executável do importador não começa; a falha propaga pelo Modelo B e o módulo falho não publica estado parcial.
- **Imports apenas por efeito colateral são permitidos**. Um módulo pode ser importado exclusivamente para que sua inicialização ocorra, mesmo sem referências posteriores ao namespace.
- Por isso, ausência de referências ao namespace não torna o import semanticamente inútil. O compilador não deve remover um import da árvore de inicialização apenas por ele não ter usos de nomes.
- A V1 não terá sintaxe especial como `import for init` ou equivalente. O `import module` normal já expressa tanto dependência de namespace quanto dependência de inicialização.
- O compilador não emite erro nem warning obrigatório de “unused import” apenas pela ausência de referências. Tooling pode mostrar informação não bloqueante de que o namespace não é referenciado diretamente.
- Para inicializações importantes, caras ou que dependam de ordem de aplicação, a recomendação de biblioteca é preferir uma função explícita (`init()`, `start()` ou API equivalente) em vez de esconder toda a intenção em efeitos colaterais de import; isso é orientação de design, não restrição semântica.
- Otimizações podem transformar o código interno de um módulo, mas não podem apagar uma aresta de `import` da semântica de inicialização somente porque seus símbolos não são usados.

**Princípio final:** **`import` descreve uma dependência estrutural e de inicialização: sua posição textual não importa, a dependência inicializa antes do importador e continua semanticamente relevante mesmo sem uso direto de nomes.**

## Funções locais e closures — decidido

- Aipo permite funções locais/aninhadas com a mesma keyword `fn`.
- Funções `fn` no top-level pertencem à estrutura do módulo e continuam sendo resolvidas independentemente da ordem textual.
- Funções `fn` declaradas dentro de outra função pertencem ao fluxo e ao escopo local: o binding da função local passa a existir quando a execução alcança sua declaração.
- Portanto, uma função local não pode ser chamada antes de sua declaração no mesmo fluxo local.
- Funções locais podem capturar automaticamente bindings lexicais de escopos externos; não haverá keywords adicionais como `capture`, `nonlocal`, `move` ou equivalentes na V1.
- Cada execução do escopo externo cria seu próprio ambiente capturado quando necessário, permitindo closures independentes.
- A mutabilidade de valores capturados segue exatamente as regras normais de `let` e `var`: um binding `var` pode ser alterado pela closure; um binding `let` não pode ser mutado por ela.
- Uma função local pode referenciar o próprio nome dentro de seu corpo, permitindo autorrecursão.
- A V1 não cria uma fase estrutural adicional dentro de blocos locais para suportar recursão mútua entre funções locais; referências a outra função local continuam obedecendo à ordem sequencial de declaração.

```
fn create_counter()
    var count = 0

    fn increment()
        count += 1
        return count
    end

    return increment
end
```

**Princípio:** **no módulo, funções descrevem estrutura; dentro de uma função, funções locais fazem parte da execução daquele escopo.**

## Funções anônimas — decidido

- A V1 terá funções anônimas como valores de primeira classe usando a mesma keyword `fn` já usada pelas funções nomeadas.
- Forma canônica:

```
let double = fn(value)
    return value * 2
end
```

- Funções anônimas podem aparecer diretamente como argumento, retorno ou qualquer outra posição de expressão que aceite uma função.
- Elas podem capturar bindings externos segundo as mesmas regras de closures já decididas.
- A V1 não adiciona uma segunda sintaxe de função, como `lambda`, `=>`, `|x| ...` ou equivalente.
- A diferença conceitual permanece pequena: `fn nome(...) ... end` introduz uma função nomeada; `fn(...) ... end` produz uma função como valor.
- **Consideração futura:** uma forma curta para funções anônimas poderá ser estudada posteriormente como açúcar sintático, especialmente para callbacks e transformações de uma única expressão. Essa possibilidade não faz parte da V1 e nenhuma sintaxe curta específica está reservada ou aprovada neste momento.

**Princípio:** **Aipo usa `fn` como forma canônica de função; eventual shorthand futuro deve ser apenas açúcar sobre a mesma semântica, não um segundo conceito de função.**

## Closures em `each` — captura por iteração — decidido

- Cada iteração de um `each` cria um binding de iteração próprio para o valor atual do iterador.
- Closures criadas durante uma iteração capturam esse binding daquela iteração, não um único binding compartilhado por todas as iterações.
- Assim, em:

```
var actions = []

each i in 0..3
    actions.add(fn()
        io.print(i)
    end)
end
```

as closures preservam os valores correspondentes às iterações e observam `0`, `1` e `2` quando chamadas depois.

- Essa regra se aplica ao binding introduzido pelo `for`; outros bindings externos continuam seguindo as regras normais de captura e mutabilidade já definidas para closures.
- A implementação da VM deve preservar a identidade léxica do binding de cada iteração quando ele escapar por uma closure.

**Princípio:** **cada iteração possui seu próprio binding capturável; uma closure criada no loop preserva o valor lexical daquela iteração, não o estado final do iterador.**

## Closures — modelo V1 final decidido

Aipo adota **closures lexicais com captura automática de bindings**.

- Uma closure captura os bindings locais e parâmetros externos que realmente referencia; não é necessária lista explícita de captura.
- A captura é semanticamente **do binding**, não de uma fotografia do valor no instante da criação.
- Um binding `var` capturado permanece compartilhado entre o escopo externo e todas as closures que capturam aquele mesmo binding. Reatribuições posteriores são observadas por essas closures e mutações realizadas por uma closure são observadas pelas demais.
- Um binding `let` capturado permanece somente leitura. A implementação pode armazenar diretamente seu valor quando isso for semanticamente equivalente, mas essa otimização não altera o modelo da linguagem.
- Parâmetros seguem as mesmas permissões que já possuem no escopo externo: captura não concede mutabilidade adicional. Um parâmetro somente leitura continua somente leitura; uma capacidade mutável já concedida por `!` pode ser usada pela closure sem permitir rebinding do parâmetro.
- Funções locais nomeadas são criadas quando a execução alcança sua declaração; funções anônimas `fn(...) ... end` produzem a closure quando a expressão é avaliada.
- Funções locais podem ser autorrecursivas. A V1 não adiciona resolução especial para recursão mútua entre funções locais.
- Cada iteração de `for` possui seu próprio binding de iteração capturável; closures criadas em iterações diferentes não compartilham acidentalmente um único binding final.
- Se uma closure escapar do escopo em que foi criada, os bindings locais capturados necessários permanecem vivos enquanto forem alcançáveis. O runtime/GC administra essa vida útil automaticamente.
- Conceitualmente, um `var` local capturado que precise sobreviver ao frame é representável como uma célula compartilhada no ambiente da closure. Otimizações como manter valores não escapantes no frame ou eliminar ambientes desnecessários são permitidas se preservarem a semântica observável.
- Referências a bindings de módulo não precisam ser copiadas para cada ambiente de closure: semanticamente continuam apontando para o binding possuído pelo módulo.
- A V1 não terá `capture`, `nonlocal`, `move capture`, captura explícita por valor/referência ou outras modalidades de captura.
- Closures podem formar ciclos de referência; o runtime gerenciado deve tratá-los corretamente, sem exigir gerenciamento manual do programador.

Exemplo:

```
fn counter()
    var count = 0

    return fn()
        count += 1
        return count
    end
end
```

Duas closures criadas por duas chamadas diferentes de `counter()` possuem bindings `count` independentes; chamadas repetidas da mesma closure observam e atualizam o mesmo binding capturado.

**Princípio:** **closures capturam automaticamente o ambiente lexical de que precisam; `var` preserva estado compartilhado e mutável, `let` preserva acesso somente leitura, e a memória gerenciada mantém os bindings capturados vivos pelo tempo necessário.**

## Processo de design e implementação — decidido

- O restante do desenho de **sintaxe e semântica da Aipo** pode ser fechado autonomamente pelo assistente, sempre priorizando coerência técnica, simplicidade, baixa carga cognitiva e consistência com as decisões já consolidadas.
- O usuário participará diretamente das decisões sobre **frontend, compilador, representações intermediárias, bytecode/VM, runtime, backend e arquitetura de implementação**, pois esses são os fundamentos que deseja aprender construindo sua primeira linguagem.
- A **escolha de ferramentas e tecnologias** — incluindo linguagem hospedeira, bibliotecas, geradores de parser, runtime/GC, bytecode tooling, backends e infraestrutura — será estudada e decidida em conjunto, não escolhida unilateralmente.
- As explicações de implementação devem assumir **nenhuma experiência prévia em criação de linguagens**, introduzir termos técnicos somente após explicar a ideia concreta e usar tópicos pequenos, exemplos visuais e progressão incremental adequada a TDAH e dislexia.
- A implementação será ensinada distinguindo claramente três camadas: **linguagem** (o que o programador vê), **compilador** (como o código é analisado/traduzido) e **runtime/backend** (como o programa é executado).

**Princípio pedagógico:** **um conceito novo por vez, primeiro de forma concreta e visual; o nome técnico e as alternativas vêm depois.**

## Código-fonte e quebras de linha — decidido

- Arquivos-fonte Aipo usam **UTF-8** como única codificação oficial da V1.
- Um BOM UTF-8 inicial é permitido por compatibilidade e deve ser ignorado pelo carregador de source.
- Arquivos sem BOM são o formato normal e recomendado.
- Tanto `LF` (`\n`) quanto `CRLF` (`\r\n`) são aceitos como quebra de linha.
- O frontend normaliza `LF` e `CRLF` para uma única noção interna de newline antes das etapas sintáticas relevantes.
- Sequências de bytes que não formem UTF-8 válido geram diagnóstico de source antes da análise léxica normal.
- O componente responsável por carregar o source mantém nome/caminho do arquivo e informação de posição suficiente para diagnósticos de linha/coluna.

**Princípio:** **a forma física do arquivo não deve introduzir diferenças semânticas entre sistemas operacionais; o frontend recebe texto UTF-8 válido com uma noção uniforme de quebra de linha.**

[Capítulo 1 — Fundamentos do Frontend](Capítulo 1 — Fundamentos do Frontend 3d19bb7d023f81bdbf7fd7bb5e711315.md)

## Caderno de implementação e aprendizado

[Capítulo 1 — Fundamentos do Frontend](Capítulo 1 — Fundamentos do Frontend 3d19bb7d023f81bdbf7fd7bb5e711315.md)

- Material didático incremental para registrar **como** a linguagem está sendo construída, separado da especificação semântica.
- O caderno deverá ser atualizado à medida que estudarmos lexer, tokens, parser, AST, análise semântica, representações intermediárias, bytecode, VM, runtime e backends.

### Frontend — decisões iniciais

- Arquivos `.aipo` usam UTF-8 como codificação oficial.
- BOM UTF-8 inicial é aceito e ignorado.
- LF e CRLF são aceitos e representam a mesma quebra de linha para o frontend.
- O carregamento/validação de `Source` é uma responsabilidade anterior e separada do lexer.
- O modelo inicial do lexer usa `NEWLINE` explícito e `EOF` como sentinela; espaços comuns e comentários `#` são descartados lexicalmente.
- `SourceSpan` usa intervalo semiaberto `[start, end)` em offsets de bytes UTF-8.
- Linha e coluna são informações derivadas pelo `Source` para apresentação de diagnósticos; não são a representação primária do span.

**Princípio de implementação:** **preservar posições precisas desde o primeiro estágio do frontend para permitir diagnósticos claros sem misturar representação interna com apresentação ao usuário.**

## Frontend — representação de tokens decidida

- Cada token da Aipo será representado conceitualmente por **`TokenKind + SourceSpan`**.
- `TokenKind` identifica a categoria lexical, como `LET`, `IDENTIFIER`, `INTEGER`, `PLUS`, `NEWLINE` ou `EOF`.
- O texto original do token não precisa ser duplicado dentro de cada token: ele permanece no `Source` e pode ser recuperado por meio do `SourceSpan`.
- Keywords, identificadores, literais, operadores, pontuação e tokens estruturais possuem `TokenKind` próprios quando isso simplifica o parser.
- O lexer classifica literais numéricos como `INTEGER` ou `FLOAT`, mas **não converte o texto para o valor numérico final** nesta etapa. A conversão ocorrerá posteriormente, quando a representação sintática do literal for criada.
- Strings terão tratamento específico estudado separadamente, pois escapes distinguem o texto escrito do valor lógico da String.

**Princípio:** o lexer identifica **o que** existe e **onde** existe; etapas posteriores constroem os valores e significados sintáticos/semânticos.

## Identificadores, keywords e Unicode — decidido

- O lexer lê o identificador completo antes de decidir se ele corresponde a uma keyword.
- Keywords só são reconhecidas quando o nome inteiro coincide exatamente; por exemplo, `let` é keyword, enquanto `letter` e `if_value` são identificadores normais.
- Aipo aceita identificadores Unicode.
- O primeiro caractere lógico de um identificador deve satisfazer `XID_Start` ou ser `_`; os caracteres seguintes devem satisfazer `XID_Continue` ou ser `_`.
- Identificadores são **case-sensitive**: `player`, `Player` e `PLAYER` são nomes diferentes.
- `_` sozinho é o descarte (*discard*); nomes como `_cache`, `player_` e `_temp` continuam identificadores comuns.
- A identidade lógica de identificadores usa normalização Unicode **NFC**. O `Source` original não é reescrito: spans continuam apontando para os bytes exatamente escritos pelo usuário, enquanto a forma normalizada é usada para resolução e comparação de nomes.
- O reconhecimento lexical segue: ler nome completo → obter lexema pelo `SourceSpan` → normalizar NFC → verificar `_` → verificar tabela de keywords → caso contrário produzir `IDENTIFIER`.

**Princípio:** **o código-fonte preserva os bytes originais; a identidade lógica dos nomes usa uma forma Unicode canônica e previsível.**

## Implementação — linguagem hospedeira decidida

- **Odin é a linguagem principal de implementação da Aipo.**
- O desenvolvimento cotidiano prioriza Odin simples, explícito e modular, usando `struct`, `enum`, procedures, packages, arrays/slices e testes antes de recorrer a recursos mais avançados.
- O compilador deve aproveitar **arenas e allocators de Odin** para objetos de vida útil agrupada, especialmente Source/Tokens/AST/temporários, mantendo essa memória separada do heap gerenciado da Aipo.
- O runtime/VM terá seu próprio heap e collector; o fato de Odin usar gerenciamento manual não aparece na superfície da linguagem Aipo.
- `odin check`, `odin test`, o test runner com rastreamento de memória e sanitizers disponíveis na toolchain fazem parte do ciclo de feedback da implementação.
- Parser inicial: handwritten recursive descent + Pratt para expressões; geradores de parser permanecem opcionais para estudo, não dependência do núcleo.
- O ecossistema `core`/`vendor` de Odin pode sustentar integração nativa do host para games e embedding, enquanto o código Aipo continua bytecode.

**Princípio pedagógico:** aprender Odin em espiral construindo a Aipo; introduzir recursos low-level somente quando o frontend, VM, GC ou embedding realmente os exigirem.

## Método oficial de aprendizagem e implementação

A construção da Aipo também será a trilha prática de aprendizagem de **Odin**, compiladores e runtimes. Não haverá um curso completo de Odin antes do projeto.

- Conceitos de Odin serão introduzidos **somente quando resolverem um problema real da implementação da Aipo**.
- A progressão será em espiral: um conceito aparece de forma simples, volta em novos contextos e ganha profundidade gradualmente.
- Cada etapa parte de um problema concreto da Aipo, apresenta apenas a teoria mínima necessária, implementa, testa, revisa e então avança.
- Ponteiros, allocators customizados, arenas, alinhamento, representação compacta de valores, sanitizers, FFI/embedding e outros recursos low-level entram somente quando houver necessidade arquitetural concreta.
- Regra de carga cognitiva: uma etapa pode conter vários fatos novos, mas deve introduzir no máximo **um conceito estruturalmente difícil principal**.
- O suporte didático diminui progressivamente: exemplo resolvido → previsão/execução/investigação → modificação → implementação parcial → implementação autônoma.
- Retrieval practice e revisão espaçada serão incorporados naturalmente: conceitos já aprendidos voltam em lexer, parser, AST, bytecode, VM e runtime.

**Princípio:** aprender D construindo a Aipo e aprender compiladores construindo a Aipo.

[Construindo a Aipo — Livro de Implementação](Construindo a Aipo — Livro de Implementação 3d29bb7d023f81538eafcf25ebb1e9a8.md)

[Aipo V1 — Sintaxe Canônica Consolidada](Aipo V1 — Sintaxe Canônica Consolidada 3d59bb7d023f8184b297c100cde50c68.md)

## Revisão estrutural consolidada — 2026-09-07

A superfície V1 atual usa:

- `fixed` + `invariant` em `struct`, sem mini-DSL `where`/`@`;
- `each` como única iteração, sem alias `for`;
- `impl Type` com `self`/`self!` para operações associadas;
- `match` com ramos `when`;
- `attempt ... failed binding ... end` para tratamento agrupado de falhas;
- funções sem valor de resultado distintas de funções que retornam `none`.

A distinção de valores permanece: `none`, `Bool`, `Int`, `Float` e `String` possuem semântica de valor; `struct`, `List`, `Dict` e `Function/Closure` possuem identidade gerenciada.

Referência normativa detalhada: [Aipo V1 — Sintaxe Canônica Consolidada](Aipo V1 — Sintaxe Canônica Consolidada 3d59bb7d023f8184b297c100cde50c68.md).

## Inicialização explícita de `struct` com `init` — decidido

Aipo V1 terá um `init` opcional dentro de `impl Type` para definir exclusivamente como uma `struct` recebe seus campos iniciais quando a construção automática por campos não é suficiente.

```
struct Player
    fixed id: Int
    name: String
    health: Int

    invariant
        health >= 0
        health <= 100
    end
end

impl Player
    init(id: Int, name: String, health: Int = 100)
        self.id = id
        self.name = name
        self.health = health
    end
end
```

- Há no máximo um `init` por `struct` na V1.
- Quando existe, `Player(...)` segue a assinatura de `init` e executa esse bloco para inicializar a nova instância.
- `self` dentro de `init` representa a instância em construção; `init` não é exposto como operação normal/dot-call.
- Ao terminar, todos os campos obrigatórios precisam ter sido inicializados.
- `fixed` pode ser escrito durante a fase de construção por `init`; depois da publicação da instância, o campo não pode ser substituído.
- Sem `init`, a linguagem conserva sua construção automática baseada nos campos.
- `init` e `invariant` permanecem conceitos distintos: `init` define **como o estado inicial nasce**; `invariant` define **quais condições precisam permanecer verdadeiras** após a construção e nas mutações validadas posteriores.

**Princípio:** inicialização explícita deve melhorar ergonomia e encapsular a construção sem transformar `struct` em classe ou introduzir construtores como categoria OO separada.

[Governança de Design e Evolução da Aipo](Governança de Design e Evolução da Aipo 3d59bb7d023f8162a363c89994b8caca.md)

## Governança da evolução

A evolução da Aipo é regida por [Governança de Design e Evolução da Aipo](Governança de Design e Evolução da Aipo 3d59bb7d023f8162a363c89994b8caca.md). Essa governança define critérios de entrada de features, orçamento de complexidade, classificação de mudanças, política de aliases, regras de experimentação e o processo ADP para extensões relevantes.

[Aipo V1 — Language Reference](Aipo V1 — Language Reference 3d59bb7d023f811ab6a1f55cfcdc97c1.md)

[Interlúdio — Hooks Estruturais e Blocos como Argumentos](Interlúdio — Hooks Estruturais e Blocos como Argum 3d59bb7d023f81329bfcdfb74573e481.md)

[Interlúdio — Núcleo Dinâmico e Contratos de Assinatura](Interlúdio — Núcleo Dinâmico e Contratos de Assina 3d59bb7d023f812b8260ef925cf08cbc.md)

## Fechamento progressivo da V1 — Lote 1

O primeiro lote de fechamento semântico foi concluído e passa a ser normativo.

- **Contratos de assinatura:** `name: Type`/`-> Type`; `name!: Type` quando houver mutabilidade do caminho; violações runtime são contract faults de programação.
- **`is` e narrowing:** teste Bool para tipos concretos/interfaces, narrowing flow-sensitive, forma múltipla `a, b is T` e negação canônica com `not`.
- **`T?`:** exatamente `T` ou `none`, sem wrapper e sem composição `T??`; restrito a assinaturas e testes de tipo.
- **Retornos:** `return` sem valor é distinto de `return expression` e `return none`; funções value-producing exigem completude dos caminhos normais, enquanto retornos heterogêneos permanecem válidos quando não há contrato explícito.

Detalhamento: [Interlúdio — Núcleo Dinâmico e Contratos de Assinatura](Interlúdio — Núcleo Dinâmico e Contratos de Assina 3d59bb7d023f812b8260ef925cf08cbc.md).

[Interlúdio — Números, Igualdade, Coleções e Ranges](Interlúdio — Números, Igualdade, Coleções e Ranges 3d69bb7d023f81f6895cec63bd012e7d.md)

[Interlúdio — Fluxo, Loops, Falhas e Escopo](Interlúdio — Fluxo, Loops, Falhas e Escopo 3d69bb7d023f81d6903eccbcf98b6675.md)

## Fluxo, loops, falhas e escopo — decidido

- `if` exige `Bool`, não é expressão e cria escopos próprios por ramo.
- `match` avalia o alvo uma vez, compara por `==`, não possui fallthrough e mantém pattern matching complexo fora da V1.
- `loop`, `while`, `repeat` e `each` têm responsabilidades distintas; `break`/`continue` afetam o loop mais próximo.
- `repeat` exige `Int >= 0` avaliado uma vez; `each` itera coleções, strings e ranges e cria bindings frescos por iteração.
- Mutação estrutural da coleção atualmente iterada é runtime fault.
- Aipo distingue `Failure` recuperável de runtime fault de programação; apenas Failure é tratável por fallback ou `attempt`/`failed`.
- Falhas propagam automaticamente; `fail(message)` cria Failure, `fail(err)` repropaga e `attempt` não realiza rollback automático.
- Escopo é lexical; shadowing interno é permitido, redeclaração no mesmo escopo é erro e closures capturam bindings, não snapshots.

Documento de decisão: [Interlúdio — Fluxo, Loops, Falhas e Escopo](Interlúdio — Fluxo, Loops, Falhas e Escopo 3d69bb7d023f81d6903eccbcf98b6675.md).

[Interlúdio — Chamadas, Construção, Impl e Módulos](Interlúdio — Chamadas, Construção, Impl e Módulos 3d69bb7d023f81a68cbde7a1f2d71297.md)

## Chamadas, construção, `impl` e módulos — decidido

- Chamadas usam `()`; construção de `struct` usa exclusivamente `{}` na forma `Type{...}`.
- Argumentos são avaliados da esquerda para a direita, uma única vez. Posicionais precedem nomeados; defaults são avaliados a cada chamada e podem referenciar parâmetros anteriores.
- Dot-call existe apenas para funções associadas em `impl Type`; funções globais comuns não ganham essa superfície automaticamente.
- Sem `init`, `Type{...}` usa a assinatura automática dos campos. Com `init(...)`, a mesma superfície `Type{...}` é validada exclusivamente contra a assinatura de `init`.
- `fixed` recebe no máximo uma atribuição durante construção. `invariant()` é verificado após construção e ao final bem-sucedido de fronteiras mutáveis estáveis, com rollback apenas dos campos diretos protegidos da(s) instância(s) participante(s).
- Múltiplos blocos `impl` são permitidos; operações não podem ser redefinidas e cada `struct` possui no máximo um `init()` e um `invariant()` no total.
- Interfaces continuam estruturais e sem estado/default implementation/herança na V1; `satisfy` permanece uma promessa verificável, não requisito para conformidade estrutural runtime.
- Um arquivo `.aipo` corresponde a um módulo. Tudo é privado por padrão; `export` expõe nomes. Imports canônicos: `import module`, `import module: name, name` e `import module as alias`.
- Declarações estruturais são conhecidas durante a resolução do módulo inteiro; bindings executáveis seguem ordem textual. Inicialização top-level ocorre uma única vez e ciclos de import ficam proibidos na V1.

Documento de decisão: [Interlúdio — Chamadas, Construção, Impl e Módulos](Interlúdio — Chamadas, Construção, Impl e Módulos 3d69bb7d023f81a68cbde7a1f2d71297.md).

**Princípio de superfície:** `()` chama comportamento; `{}` constrói dados estruturados.

## Expressões, mutação, strings e callables — decidido

- A precedência da V1 é fixa e pequena: postfix; unários numéricos; multiplicativos; aditivos; range; comparações/`is`; `not`; `and`; `or`; fallback `else`. Assignment permanece statement.
- Lógica exige `Bool`; `and`/`or` fazem short-circuit e não retornam operandos arbitrários.
- Assignment e compound assignment só operam por caminhos mutáveis; `++`, `--`, assignment expressions e destructuring ficam fora.
- `+` concatena apenas `String + String`; conversões textuais são explícitas por `String(value)` para valores fundamentais. Interpolação `f` reaproveita essa conversão e não cria hooks mágicos de stringificação.
- `Function` é a categoria chamável da V1; closures usam o mesmo modelo de chamada com ambiente capturado. O contrato `Function` não codifica assinatura detalhada na V1.
- Não há bound-method values automáticos, callable objects, operator overloading ou hooks `call`/string mágicos.

O fechamento normativo deste lote está consolidado na Language Reference e na Sintaxe Canônica. Documento de decisão: [Interlúdio — Expressões, Mutação, Strings e Callables](Interlúdio — Expressões, Mutação, Strings e Callab 3d69bb7d023f810a8628cfe5ac111878.md).

<!-- lote5-interlude-link-pending -->

[Interlúdio — Expressões, Mutação, Strings e Callables](Interlúdio — Expressões, Mutação, Strings e Callab 3d69bb7d023f810a8628cfe5ac111878.md)

[Aipo V1 — Exemplo Integrado de Sintaxe](Aipo V1 — Exemplo Integrado de Sintaxe 3d79bb7d023f8118a234daa9b29634fc.md)

[Aipo — Backlog de Sintaxe e Recursos Pós-V1](Aipo — Backlog de Sintaxe e Recursos Pós-V1 3d79bb7d023f8128ab0af9327dadf75d.md)

[Aipo — Biblioteca de Referências Técnicas](Aipo — Biblioteca de Referências Técnicas 3d79bb7d023f8189a98addd2f2a00701.md)

[Aipo — Rust/Poppy Pivot, Host Profiles e Roadmap 10/10](Aipo — Rust Poppy Pivot, Host Profiles e Roadmap 1 3dc9bb7d023f81a4b03fe8f7e6de3508.md)