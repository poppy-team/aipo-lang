# Interlúdio — Núcleo Dinâmico e Contratos de Assinatura

<aside>
✅

**Decisão fechada.** Aipo adota um núcleo dinamicamente tipado e fortemente verificado em runtime, com contratos opcionais restritos às fronteiras de funções: parâmetros e retornos.

</aside>

## Princípio central

**Valores têm tipos; bindings e campos não precisam declarar tipos. Funções podem declarar opcionalmente o que aceitam e o que retornam.**

A arquitetura separa quatro responsabilidades:

- contratos de parâmetro/retorno: descrevem e protegem fronteiras de API;
- inferência/análise de fluxo: observa tipos dentro do corpo sem criar contratos persistentes;
- `is`: testa e estreita o tipo conhecido de um valor em um ponto do fluxo;
- `invariant()`: protege propriedades duradouras do estado de uma `struct`.

## Superfície canônica

```
let name = "Ana"
var value = 10
value = "hello"

fn load_user(id: Int) -> User?
    ...
end
```

Não existem anotações em bindings:

```
let age: Int = 20    # não faz parte da V1
var name: String     # não faz parte da V1
```

Campos de `struct` também não carregam contratos:

```
struct Player
    fixed id
    name
    health = 100
end
```

Quando uma propriedade do estado precisa ser garantida, usa-se `invariant()`:

```
impl Player
    invariant()
        self.id, self.health is Int
        self.name is String
        self.health >= 0
        self.health <= 100
    end
end
```

## Contratos de assinatura

Parâmetros podem declarar contrato com `name: Type`; retornos podem declarar contrato com `-> Type`.

```
fn damage(self!, amount: Int)
    self.health -= amount
end

fn find_user(id: Int) -> User?
    ...
end
```

Um parâmetro sem anotação continua completamente dinâmico. Um retorno sem `-> T` não significa `Void`: significa apenas ausência de contrato declarado de retorno. O corpo continua determinando se a função produz valor ou é uma função sem valor de resultado.

Contratos são promessas reais da linguagem, não meras hints. Incompatibilidades prováveis podem ser diagnosticadas estaticamente; quando não puderem ser provadas antes, a fronteira é verificada em runtime.

## `T?`

`T?` permanece como forma compacta para `T` ou `none` em contratos de assinatura e pode também ser usada como expressão de tipo em testes `is`.

```
fn find(id: Int) -> Player?
    ...
end

if value is Player?
    ...
end
```

## Interfaces

Interfaces continuam estruturais e podem declarar contratos opcionais em seus parâmetros e retornos.

```
interface Storage
    fn load(self, key: String) -> Data?
    fn save(self!, key: String, value: Data)
end
```

Isso concentra informação estável justamente nas APIs, plugins, adapters e outras fronteiras arquiteturais, sem tipar o corpo inteiro do programa.

## O que fica fora da V1

Para impedir que o modelo evolua silenciosamente para gradual typing completo, ficam fora inicialmente:

- contratos em `let`/`var`;
- contratos em campos de `struct`;
- `Any` como placeholder obrigatório;
- `Void`/`Unit` público;
- generics definidos pelo usuário;
- unions arbitrárias e intersections;
- overloads por tipo;
- contratos profundos `List[T]` e `Dict[K, V]`.

`List` e `Dict` podem aparecer como categorias simples em assinaturas. Contratos parametrizados de coleção permanecem uma decisão futura separada porque sua verificação profunda pode ter custo proporcional ao tamanho da coleção e introduzir estado contratual persistente.

## Tooling e implementação

O objetivo é obter a maior parte do benefício de tooling sem espalhar contratos por toda a AST.

O frontend precisa armazenar contrato opcional apenas em parâmetros e retorno de funções/hooks que possuam parâmetros. Em termos conceituais:

```
Parameter
├── name
├── mutable
├── contract?
└── default?

Function
├── parameters
├── return_contract?
└── body
```

Bindings, campos e assignments permanecem dinamicamente analisados. Isso facilita autocomplete, hover, signature help, navegação, diagnóstico entre módulos e APIs públicas sem transformar a linguagem em estaticamente tipada.

**Regra de tooling:** informação inferida é conhecimento atual do analisador; contrato escrito é promessa estável da API. O tooling deve distinguir as duas.

## Racional

Essa arquitetura busca o ponto de equilíbrio entre scripting e projetos maiores:

- o iniciante pode ignorar tipos declarativos por completo;
- código local permanece limpo;
- bibliotecas e interfaces podem documentar/proteger suas fronteiras;
- `invariant()` oferece segurança de domínio mais expressiva que tipos de campo;
- `is` e análise de fluxo mantêm o dinamismo tratável;
- o compilador evita a complexidade sistêmica de um gradual type system completo.

**Resumo:** o programador declara tipos onde valores atravessam fronteiras; o compilador observa tipos dentro dessas fronteiras.

## Fechamento normativo — Lote 1

### 1. Contratos opcionais de parâmetros e retorno

- `name: Type` é contrato opcional de entrada; `-> Type`, contrato opcional do valor de sucesso retornado.
- Violações comprováveis devem ser diagnosticadas antes da execução; quando a incompatibilidade só puder ser conhecida em runtime, a fronteira produz **contract fault** de programação, não falha recuperável capturável por `attempt`.
- Parâmetro mutável com contrato usa `name!: Type`; o `!` pertence à permissão do caminho, enquanto `: Type` pertence ao contrato do valor.
- Receivers em `impl Type` não repetem contrato de tipo: `self`/`self!` já são determinados pelo `impl`.
- `init(...)`, funções anônimas, interfaces e parâmetros de trailing blocks podem usar os mesmos contratos opcionais.
- Na V1, compatibilidade de contratos em interfaces permanece exata; não há covariance/contravariance implícita.

### 2. `is` e narrowing

- `value is Type` é sempre expressão `Bool`; incompatibilidade produz `false`, nunca falha.
- Para `struct`, testa o tipo concreto; para `interface`, testa conformidade estrutural.
- O analisador faz narrowing flow-sensitive somente enquanto o fluxo justificar aquela informação; narrowing nunca cria contrato persistente.
- `a, b, c is T` é açúcar geral para `a is T and b is T and c is T`, não uma regra exclusiva de `invariant()`.
- As expressões da forma múltipla são avaliadas da esquerda para a direita e uma única vez cada.
- A negação canônica é `not value is T`, interpretada como `not (value is T)`; não existe `is not` como alias.

### 3. `T?`, `none` e optional

- `T?` significa exatamente `T` ou `none`; não cria wrapper `Optional`/`Option`/`Maybe`.
- É permitido somente onde a V1 aceita expressões de tipo: contratos de assinatura e testes `is`.
- `value is T?` testa se o valor é `T` ou `none`; dentro do ramo, o conhecimento permanece `T | none`, não apenas `T`.
- `T??` é inválido; unions arbitrárias continuam fora da V1.
- `?.` permanece navegação segura e `expression else fallback` permanece a forma canônica de fallback; não é adicionado `.or(...)` equivalente.

### 4. Resultado de funções

- Aipo distingue semanticamente funções **com valor** e funções **sem resultado**, sem expor `Void`/`Unit`.
- `return` encerra a função/closure sem produzir valor; `return expression` produz um valor. `return` e `return none` são semanticamente distintos.
- Se uma função possui qualquer `return expression` alcançável, ela é value-producing; todos os caminhos normais alcançáveis devem produzir valor ou encerrar o fluxo (`fail`, loop comprovadamente não terminante etc.). Misturar `return` sem valor com `return expression` numa função value-producing é erro.
- Sem contrato de retorno, funções value-producing podem retornar categorias diferentes em ramos diferentes; unions resultantes pertencem apenas à análise interna/tooling e não viram sintaxe pública.
- Uma função sem `-> T` pode produzir valor normalmente; a omissão significa apenas ausência de contrato de retorno.
- `none` é um valor real e pode ser retornado/armazenado; o resultado de uma função no-result não pode ser usado em posição de valor.
- `fail(...)` encerra aquele caminho e não precisa satisfazer o contrato de retorno; o compilador pode representar isso internamente como fluxo bottom/Never sem expor `Never` na V1.
- Funções e closures carregam internamente um `result kind` (`VALUE` ou `NO_RESULT`) independente da presença de contrato.
- Em trailing blocks, `return`/`return value` retornam da própria closure, nunca da função externa.

<aside>
✅

**Lote 1 fechado:** contratos de assinatura, `is`/narrowing, `T?` e semântica de retorno estão normativos e não permanecem como itens em aberto da V1.

</aside>