# Controle de Fluxo & Falhas

O Aipo oferece estruturas de controle de fluxo limpas e expressivas, combinadas com um modelo transacional de tratamento de erros.

---

## Estruturas Condicionais

### Expressões `if ... then ... else`

O `if` em Aipo pode ser utilizado como comando em bloco ou como expressão de valor inline:

```aipo
// Como bloco convencional
if pontuacao >= 90 then
  status = "Excelente"
else if pontuacao >= 70 then
  status = "Aprovado"
else
  status = "Recuperação"
end

// Como expressão condicional de valor inline
let mensagem = if ativo then "Online" else "Offline"
```

---

## Estruturas de Repetição

### `while`

Executa o corpo enquanto a condição for verdadeira:

```aipo
let var i = 0
while i < 10 do
  print(i)
  i += 1
end
```

### `loop`

Laço infinito canônico que deve ser encerrado com `break`:

```aipo
let var tentativas = 0
loop do
  tentativas += 1
  if verificar_conexao() then
    break
  end
  if tentativas >= 5 then
    fail "Timeout após 5 tentativas"
  end
end
```

### `repeat`

Repetição com verificação de condição ao final do bloco:

```aipo
let var contador = 0
repeat do
  contador += 1
until contador >= 5
```

### `each`

Iteração canônica sobre coleções (`List`, `Dict`, `Set`, `Sequence`):

```aipo
let frutas = ["Maçã", "Banana", "Laranja"]
each item in frutas do
  print(item)
end

// Iteração sobre dicionários (chaves na ordem de inserção)
let config = { "host": "127.0.0.1", "port": 5432 }
each chave in config do
  print(chave + " => " + config[chave])
end
```

---

## Modelo de Falhas & Transações (`attempt ... recover`)

Em Aipo, erros não são exceções com propagação desenfreada nem códigos de status opacos. Falhas operacionais são disparadas explicitamente com `fail` e tratadas por blocos transacionais com **journaling e rollback automático**.

```aipo
struct Cofre {
  var saldo: Float,
  invariant() {
    self.saldo >= 0.0
  }
}

let c = Cofre { saldo: 100.0 }

attempt
  c.saldo -= 200.0 // Quebra a invariante do Cofre!
recover erro
  print("Falha capturada: " + erro)
end

// Como a operação falhou, o saldo permanece intacto em 100.0!
print("Saldo recuperado: " + c.saldo) // 100.0
```

Se qualquer operação dentro do bloco `attempt` disparar um `fail` ou violar uma invariante estrutural, todas as mutações ocorridas nos objetos rastreados no journal são revertidas atomicamente para o estado original.
