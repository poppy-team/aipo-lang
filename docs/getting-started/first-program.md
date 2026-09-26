# Seu Primeiro Programa em 5 Minutos

Neste tutorial rápido, vamos criar, verificar, compilar e executar o seu primeiro programa em Aipo.

---

## 1. Olá, Mundo!

Crie um arquivo chamado `hello.aipo`:

```aipo
// hello.aipo
print("Olá do Aipo!")
```

Execute diretamente pelo CLI:

```bash
aipo run hello.aipo
```

**Saída esperada:**
```
Olá do Aipo!
```

---

## 2. Estruturas, Invariantes e Métodos

Vamos criar um programa que modela uma conta bancária com invariante de saldo positivo. Crie o arquivo `conta.aipo`:

```aipo
// conta.aipo
struct Conta {
  titular: String,
  var saldo: Float,
  fixed numero: Int,

  // Invariante: executada em todas as criações e mutações
  invariant() {
    self.saldo >= 0.0
  }

  fn depositar(valor: Float) {
    if valor <= 0.0 then
      fail "Valor de depósito deve ser positivo"
    end
    self.saldo += valor
  }

  fn sacar(valor: Float) {
    if valor <= 0.0 then
      fail "Valor de saque deve ser positivo"
    end
    
    // Tenta aplicar a operação. Se violar self.saldo >= 0.0,
    // o bloco attempt reverte automaticamente a mutação!
    attempt
      self.saldo -= valor
    recover erro
      fail "Saque recusado: saldo insuficiente"
    end
  }
}

// Instanciando a conta
let c = Conta {
  titular: "Maria Silva",
  saldo: 150.0,
  numero: 1042
}

print("Conta criada para: " + c.titular)
print("Saldo inicial: " + c.saldo)

c.depositar(50.0)
print("Saldo após depósito: " + c.saldo)

c.sacar(75.0)
print("Saldo após saque: " + c.saldo)
```

Execute o programa:

```bash
aipo run conta.aipo
```

**Saída esperada:**
```
Conta criada para: Maria Silva
Saldo inicial: 150.0
Saldo após depósito: 200.0
Saldo após saque: 125.0
```

---

## 3. Inspecionando o Bytecode

Uma das grandes forças do Aipo é a transparência do compilador. Você pode visualizar as instruções de bytecode geradas para qualquer arquivo:

```bash
aipo disasm conta.aipo
```

O comando exibirá o desassembly com linhas e colunas mapeadas, demonstrando a alocação de registradores, frames de chamada e tabelas de constantes.

---

## 4. Compilando para JavaScript

Você pode transpilar o mesmo código para JavaScript executável via Node.js:

```bash
aipo build conta.aipo -o dist/conta.js
node dist/conta.js
```

O código gerado possui paridade comportamental completa com a VM em Rust.
