# Interfaces & Contratos

O sistema de tipos do Aipo une a ergonomia da tipagem dinâmica com a precisão dos **contratos de assinatura**, **invariantes de dados** e **interfaces estruturais**.

---

## Estruturas (`struct`)

Estruturas agregam campos nomeados e são fechadas com `end` (sem chaves). Por padrão, campos são mutáveis, a menos que prefixados com `fixed`:

```aipo
struct Servidor
    fixed id
    fixed criado_em
    status = "offline"
    carga = 0.0
end

let s = Servidor{
    id = "srv-1",
    criado_em = 1600000000,
    status = "online",
    carga = 0.42,
}

io.println(s.id)     # "srv-1"
io.println(s.status) # "online"
```

Tentar reatribuir um campo `fixed` após a construção da instância dispara o diagnóstico semântico estático `AIPO_SEM_FIXED_REASSIGN`.

---

## Hook de Construção (`init`)

O hook `init` é declarado dentro do bloco `impl StructName` e permite validar, transformar e inicializar os campos da instância antes de sua publicação:

```aipo
struct Usuario
    email
    nome
end

impl Usuario
    init(email, nome)
        if not email.contains("@")
            return fail("Formato de e-mail inválido")
        end
        self.email = email
        self.nome = nome
    end
end

let u = Usuario{email = "user@example.com", nome = "Dev"}
io.println(u.email) # "user@example.com"
```

---

## Invariantes Estruturais (`invariant`)

As invariantes declaram predicados lógicos dentro do bloco `impl` que **devem permanecer verdadeiros durante todo o ciclo de vida do objeto**:

```aipo
struct Intervalo
    inicio = 0
    fim = 0
end

impl Intervalo
    init(inicio, fim)
        self.inicio = inicio
        self.fim = fim
    end

    invariant()
        self.inicio <= self.fim
    end
end

let inter = Intervalo{inicio = 5, fim = 10}
io.println(inter.inicio) # 5
io.println(inter.fim)    # 10
```

Sempre que um campo de uma estrutura com `invariant()` for alterado, o motor de execução verifica automaticamente o predicado. Caso a verificação falhe, a operação é rejeitada. Se estiver dentro de um bloco `attempt`, as mutações anteriores sofrem rollback automático pelo journal transacional.

---

## Interfaces e Conformidade (`interface` / `satisfy`)

Interfaces declaram contratos estruturais de métodos. Em Aipo, conformidade não é herança rígida: é tipagem estrutural validada e declarada explicitamente com `satisfy`:

```aipo
interface Renderizavel
    fn desenhar(self) -> String
end

struct Botao
    texto
end

impl Botao
    fn desenhar(self) -> String
        return f"[Botão: {self.texto}]"
    end
end

# Declaração canônica de conformidade estrutural
satisfy Botao: Renderizavel

# Aceita qualquer valor que satisfaça o contrato de Renderizavel
fn renderizar_elemento(item: Renderizavel) -> String
    return item.desenhar()
end

let btn = Botao{texto = "Salvar"}
io.println(renderizar_elemento(btn)) # "[Botão: Salvar]"
```

A cláusula `satisfy` é validada pelo analisador semântico (`aipo-sema`), verificando aridade de parâmetros, nome de métodos, compatibilidade do receptor `self` (ou `self!` para métodos mutadores) e assinaturas assíncronas antes da execução.

