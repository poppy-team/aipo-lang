# Seu Primeiro Programa em 5 Minutos

Neste tutorial rápido, vamos criar, verificar, compilar e executar o seu primeiro programa em Aipo.

---

## 1. Olá, Mundo!

Crie um arquivo chamado `hello.aipo`:

```aipo
# hello.aipo
io.println("Olá do Aipo!")
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
# conta.aipo
struct Conta {
    titular
    numero
    var saldo = 0.0
}

impl Conta {
    fn init(titular, numero = 0, saldo = 0.0) {
        self.titular = titular
        self.numero = numero
        self.saldo = saldo
    }

    # Invariante: executada em todas as criações e mutações de campos
    invariant {
        self.saldo >= 0.0
    }

    fn depositar(var self, valor: Float) {
        if valor <= 0.0 {
            return fail("Valor de depósito deve ser positivo")
        }
        self.saldo += valor
    }

    fn sacar(var self, valor: Float) {
        if valor <= 0.0 {
            return fail("Valor de saque deve ser positivo")
        }
        
        # Tenta aplicar a operação. Se violar self.saldo >= 0.0,
        # o bloco attempt reverte automaticamente a mutação!
        attempt {
            self.saldo -= valor
        } failed erro {
            return fail("Saque recusado: saldo insuficiente")
        }
    }
}

# Instanciando a conta usando dois-pontos (:) consistente
let c = Conta{ titular: "Maria Silva", numero: 1042, saldo: 150.0 }

io.println(f"Conta criada para: {c.titular}")
io.println(f"Saldo inicial: {c.saldo}")

c.depositar(50.0)
io.println(f"Saldo após depósito: {c.saldo}")

c.sacar(75.0)
io.println(f"Saldo após saque: {c.saldo}")
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
