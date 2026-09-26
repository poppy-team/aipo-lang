# Interfaces & Contratos

O sistema de tipos do Aipo une a ergonomia da tipagem dinâmica com a precisão dos **contratos de assinatura**, **invariantes de dados** e **interfaces com subtipagem estrutural automática**.

---

## Estruturas (`struct`)

Estruturas agregam campos nomeados e são delimitadas por chaves `{ ... }`. 

Por padrão de design seguro e previsível, **todos os campos de uma estrutura são imutáveis**. Quando for necessário permitir mutação de um campo durante o ciclo de vida da instância, declare-o explicitamente com a palavra-chave `var`:

```aipo
struct Servidor {
    id
    criado_em
    var status = "offline"
    var carga = 0.0
}

# Instanciação estrutural usando chave-valor simétrico com ':'
let s = Servidor{
    id: "srv-1",
    criado_em: 1600000000,
    status: "online",
    carga: 0.42,
}

io.println(s.id)     # "srv-1"
io.println(s.status) # "online"
```

Tentar reatribuir um campo imutável após a construção da instância dispara o diagnóstico semântico estático `AIPO_SEM_IMMUTABLE_FIELD_REASSIGN`.

---

## Hook de Construção (`init`)

O hook `init` é declarado dentro do bloco `impl StructName { ... }` e permite validar, transformar e inicializar os campos da instância antes de sua publicação final:

```aipo
struct Usuario {
    email
    nome
}

impl Usuario {
    init(email, nome) {
        if not email.contains("@") {
            return fail("Formato de e-mail inválido")
        }
        self.email = email
        self.nome = nome
    }
}

let u = Usuario{ email: "user@example.com", nome: "Dev" }
io.println(u.email) # "user@example.com"
```

---

## Invariantes Estruturais (`invariant`)

As invariantes declaram predicados lógicos dentro do bloco `impl` que **devem permanecer verdadeiros durante todo o ciclo de vida do objeto**:

```aipo
struct Intervalo {
    var inicio = 0
    var fim = 0
}

impl Intervalo {
    init(inicio, fim) {
        self.inicio = inicio
        self.fim = fim
    }

    invariant {
        self.inicio <= self.fim
    }
}

let inter = Intervalo{ inicio: 5, fim: 10 }
io.println(inter.inicio) # 5
io.println(inter.fim)    # 10
```

Sempre que um campo de uma estrutura com bloco `invariant` for alterado, o motor de execução verifica automaticamente o predicado. Caso a verificação falhe, a operação é rejeitada. Se estiver dentro de um bloco `attempt { ... }`, as mutações anteriores sofrem rollback automático pelo journal transacional.

---

## Métodos e Mutabilidade Universal (`var self`)

Métodos associados a um tipo são definidos dentro de blocos `impl StructName { ... }`. 

Por padrão de segurança, o receptor `self` é **somente leitura**. Quando um método precisa alterar o estado interno da instância, ele declara explicitamente `var self`, alinhando a mutabilidade de métodos à mesma regra universal de variáveis da linguagem:

```aipo
struct Contador {
    var valor = 0
}

impl Contador {
    # Método de leitura: self é imutável
    fn atual(self) -> Int {
        return self.valor
    }

    # Método mutador: var self declara explicitamente a intenção de modificar
    fn incrementar(var self) {
        self.valor += 1
    }
}
```

---

## Interfaces e Subtipagem Estrutural Automática (`interface`)

Interfaces declaram contratos estruturais de métodos. Em Aipo, conformidade não exige declarações burocráticas no topo do arquivo: **a subtipagem é estrutural e automática** (modelo inspirado em linguagens modernas de alta produtividade como Go e Luau).

Se uma estrutura implementa todos os métodos exigidos por uma `interface` com assinaturas e contratos compatíveis (incluindo aridade e mutabilidade de `self`), ela **automaticamente satisfaz a interface**, sem necessidade de nenhum comando adicional:

```aipo
interface Renderizavel {
    fn desenhar(self) -> String
}

struct Botao {
    texto
}

impl Botao {
    fn desenhar(self) -> String {
        return f"[Botão: {self.texto}]"
    }
}

# Botao satisfaz Renderizavel AUTOMATICAMENTE!
# Não há palavras-chave 'implements' nem comandos órfãos 'satisfy'.

# Aceita qualquer valor que satisfaça a interface Renderizavel
fn renderizar_elemento(item: Renderizavel) -> String {
    return item.desenhar()
}

let btn = Botao{ texto: "Salvar" }
io.println(renderizar_elemento(btn)) # "[Botão: Salvar]"
```

A conformidade é verificada estaticamente pelo analisador semântico (`aipo-sema`), checando a existência dos métodos, número de argumentos, tipos de parâmetros e retornos, e a mutabilidade compatível do receptor (`self` vs `var self`).
