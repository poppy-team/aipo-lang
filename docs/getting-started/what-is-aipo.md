# O que é Aipo?

O **Aipo** é uma linguagem de programação moderna de propósito geral, com tipagem dinâmica e forte, dotada de suporte nativo a contratos de assinatura e invariantes estruturais.

Projetada com uma filosofia de **simplicidade, robustez e previsibilidade**, o Aipo foi construído do zero em Rust com arquitetura limpa e sem dependências ocultas.

---

## Pilares Fundamentais

### 1. Tipagem Dinâmica e Forte

No Aipo, valores possuem tipos concretos e o sistema não realiza conversões arbitrárias ou silenciosas entre tipos incompatíveis:

```aipo
let x = "42"
let y = 10

// Erro de tipo em tempo de execução:
// Operador '+' não aplica concatenação entre String e Int
let z = x + y // Falha explícita!
```

Para concatenar ou converter valores, exige-se intenção explícita ou formatação declarada.

### 2. Contratos Estruturais & Invariantes

Enquanto linguagens tradicionais exigem que você espalhe `assert` ou validações manuais em todos os métodos, o Aipo introduz o bloco `invariant()` diretamente na declaração de `struct`:

```aipo
struct Temperature {
  var celsius: Float,

  invariant() {
    self.celsius >= -273.15 // Não pode ser inferior ao zero absoluto
  }
}
```

Qualquer mutação que viole a invariante é interceptada na fronteira da operação. Dentro de blocos de transação `attempt ... recover`, as alterações são revertidas atomicamente para o estado anterior.

### 3. Concorrência Determinística

O modelo de concorrência do Aipo é baseado em **fibras cooperativas e tempo virtual**. As chamadas assíncronas usam `async fn` e combinadores de alto nível (`task.spawn`, `task.sleep`, `task.all`, `task.race`), orquestrados por um scheduler determinístico que permite testes 100% reproduzíveis.

### 4. Dois Destinos: VM Nativa e JavaScript

O compilador do Aipo foi desenvolvido com duas metas principais:
1. **VM de Bytecode em Rust**: Execução veloz, serialização binária determinística (`.aibc`) e inspeção de baixo nível com desassemblador de linha/coluna.
2. **Backend JavaScript (`aipo-js`)**: Emissão direta de JavaScript moderno (ES2022) com runtime shim modular e paridade diferencial bit a bit garantida por suíte de testes.

---

## Comparativo Rápido

| Recurso | Aipo | Python | Lua | JavaScript |
| :--- | :--- | :--- | :--- | :--- |
| **Tipagem** | Dinâmica e Forte | Dinâmica e Forte | Dinâmica e Fraca | Dinâmica e Fraca |
| **Invariantes Nativas** | Sim (`invariant()`) | Não (manual) | Não | Não |
| **Rollback Transacional** | Sim (`attempt`) | Não | Não | Não |
| **Async Nativo** | Sim (Determinístico) | Sim (`asyncio`) | Corrotinas (baixo nível) | Sim (Event Loop) |
| **Módulos / Dependências** | SHA Pinada + Offline | Pip / Virtualenv | Externa (Luarocks) | NPM / Node Modules |
| **Implementação** | Rust (Compilador + VM) | C / C++ | ANSI C | C++ (V8) / Rust (Deno) |
