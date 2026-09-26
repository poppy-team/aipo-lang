# Controle de Fluxo & Falhas

O Aipo oferece estruturas de controle de fluxo limpas e expressivas, combinadas com um modelo transacional de tratamento de erros com rollback atômico.

---

## Estruturas Condicionais

### Bloco `if ... elif ... else ... end`

Em blocos convencionais, cada ramo é delimitado sem chaves, utilizando `elif` para condições intermediárias e finalizando com a palavra-chave `end`:

```aipo
var pontuacao = 85
var status = ""

if pontuacao >= 90
    status = "Excelente"
elif pontuacao >= 70
    status = "Aprovado"
else
    status = "Recuperação"
end

io.println(status) # "Aprovado"
```

### Expressão Inline `if condition then a else b`

O Aipo também suporta expressões condicionais de valor em linha única utilizando `then`:

```aipo
let ativo = true
let mensagem = if ativo then "Online" else "Offline"
io.println(mensagem) # "Online"
```

---

## Seleção por Padrão (`match ... when`)

O comando `match` permite bifurcar o fluxo comparando uma expressão contra um ou mais padrões por ramo:

```aipo
let status = "aprovado"

match status
when "pendente"
    io.println("Aguardando confirmação...")
when "aprovado", "concluido"
    io.println("Operação finalizada com sucesso!")
else
    io.println("Status não reconhecido")
end
```

---

## Estruturas de Repetição

Todas as estruturas de repetição em Aipo são finalizadas com a palavra-chave `end` e **não utilizam** a palavra `do`.

### `while`

Executa o corpo enquanto a condição booleana for verdadeira:

```aipo
var i = 0
while i < 3
    io.println(f"Passo: {i}")
    i += 1
end
```

### `loop`

Laço contínuo canônico, projetado para repetições que dependem de `break` explícito:

```aipo
var tentativas = 0
loop
    tentativas += 1
    if tentativas >= 3
        break
    end
end
io.println(f"Total de tentativas: {tentativas}")
```

### `repeat`

Repete o bloco um número fixo de vezes com um contador opcional (`repeat count as indice`):

```aipo
# Executa 3 vezes (com índices 0, 1 e 2)
repeat 3 as idx
    io.println(f"Iteração número: {idx}")
end
```

### `each`

Iteração canônica sobre coleções (`List`, `Dict`, `Set`, `Sequence`):

```aipo
# Iteração simples sobre lista
let frutas = ["Maçã", "Banana", "Laranja"]
each fruta in frutas
    io.println(fruta)
end

# Iteração com índice e elemento
each idx, fruta in frutas
    io.println(f"{idx}: {fruta}")
end

# Iteração sobre dicionário (chave e valor na ordem de inserção)
let config = {
    "host": "127.0.0.1",
    "port": 5432,
}
each chave, valor in config
    io.println(f"{chave} => {valor}")
end
```

---

## Modelo de Falhas & Transações (`attempt ... failed`)

Em Aipo, erros não são exceções globais com saltos de pilha descontrolados, nem códigos de status que podem ser esquecidos. Falhas operacionais são disparadas explicitamente com `fail` (ou retornadas com `return fail(...)`) e tratadas por blocos transacionais com **journaling e rollback automático**.

```aipo
struct Cofre
    saldo = 0.0
end

impl Cofre
    invariant()
        self.saldo >= 0.0
    end
end

let c = Cofre{saldo = 100.0}

attempt
    # Esta operação temporariamente reduz o saldo para -100.0
    c.saldo = c.saldo - 200.0
failed erro
    # Como violou a invariante, o rollback restaura self.saldo para 100.0!
    io.println(f"Falha capturada: {erro.message}")
end

# O saldo permanece exatamente no valor anterior à tentativa!
io.println(f"Saldo preservado: {c.saldo}") # 100.0
```

Se qualquer operação dentro do bloco `attempt` disparar um `fail` ou violar uma invariante estrutural, todas as mutações ocorridas nos objetos rastreados no journal são revertidas atomicamente para o estado original.

### Fallback Imediato com `or_else`

Para expressões onde você deseja apenas prover um valor padrão de recuperação sem a verbosidade de um bloco `attempt`, utilize o operador canônico `or_else`:

```aipo
fn ler_arquivo(caminho)
    # Se o arquivo não existir ou falhar, retorna fail
    return fail("arquivo não encontrado")
end

# Se ler_arquivo disparar fail, or_else avalia e retorna a alternativa:
let conteudo = ler_arquivo("config.toml") or_else "host = 127.0.0.1"
io.println(conteudo) # "host = 127.0.0.1"
```


