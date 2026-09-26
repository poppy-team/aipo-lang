---
layout: home

hero:
  name: Aipo
  text: Linguagem Dinâmica, Forte e Concorrente com Contratos Opcionais
  tagline: Pipeline completo em Rust com compilador dedicado, máquina virtual determinística, scheduler assíncrono cooperativo, backend JavaScript com paridade diferencial e governança com Prumo CLI.
  image:
    src: /assets/logo.svg
    alt: Aipo Language Logo
  actions:
    - theme: brand
      text: Começar Agora (5 min)
      link: /getting-started/first-program
    - theme: alt
      text: Conhecer a Trajetória
      link: /trajectory/
    - theme: alt
      text: Manual da Linguagem
      link: /manual/
    - theme: alt
      text: Repositório GitHub
      link: https://github.com/poppy-team/aipo-lang

features:
  - icon: ⚡
    title: Pipeline Nativo em Rust
    details: Compilador do zero em Rust moderno. Lexer com fronteiras seguras UTF-8, parser com recuperação resiliente, HIR, SEMA estrito com inferência de contratos e bytecode determinístico .aibc.
    link: /architecture/compiler-frontend
  - icon: 🛡️
    title: Contratos & Invariantes Transacionais
    details: Checagem estática e runtime. Invariantes de estrutura com rollback atômico sob falha em blocos attempt, validação de assinaturas e conformidade estrutural de interfaces.
    link: /manual/interfaces-and-contracts
  - icon: 🔄
    title: Concorrência Assíncrona & Tempo Virtual
    details: Sintaxe nativa async fn e await do, combinadores assíncronos determinísticos (task.spawn, task.all, task.race), scheduler cooperativo e detecção de ciclos de espera.
    link: /manual/async-and-concurrency
  - icon: 🌐
    title: Paridade Diferencial VM ↔ JavaScript
    details: Emissor aipo-js com runtime shim versionado, gerando código ES2022 limpo para Node.js e navegadores com os exatos mesmos checksums e diagnósticos da VM nativa.
    link: /architecture/js-emitter
  - icon: 🔒
    title: Host ABI Segura & Poppy Simulation
    details: Sandboxing com capabilities deny-by-default, handles geracionais sem use-after-free, prevenção estrita contra escape de escopo e simulação determinística headless.
    link: /architecture/host-abi
  - icon: 📦
    title: Gestor de Pacotes Hermético & Offline
    details: Manifesto aipo.toml, lockfiles determinísticos, dependências GitHub pinadas por commit SHA com digest SHA-256 verificado, cache local atômico e poda segura.
    link: /manual/packages-and-modules
---

<div class="vp-doc">

## Uma Linguagem Moderna com Disciplina de Engenharia

O **Aipo** foi concebido para entregar uma sintaxe expressiva e agradável sem abrir mão do rigor técnico e da previsibilidade. Em vez de coerções implícitas perigosas (como `"1" + 2 == "12"`), o Aipo combina tipagem dinâmica e forte com **contratos estruturais opcionais**, **recuperação transacional de falhas** e um **sistema assíncrono determinístico**.

```aipo
# Exemplo canônico de Aipo: structs, contratos invariantes e async
struct Account
    fixed id
    balance = 0.0
    fixed created_at
end

impl Account
    init(id, balance = 0.0, created_at = 0)
        self.id = id
        self.balance = balance
        self.created_at = created_at
    end

    invariant()
        self.balance >= 0.0
    end

    fn transfer(self!, target: Account, amount: Float)
        if amount <= 0.0
            return fail("Amount must be positive")
        end

        # Se qualquer invariante falhar, a mutação sofre rollback
        attempt
            self.balance = self.balance - amount
            target.balance = target.balance + amount
        failed err
            return fail(f"Transfer cancelled: {err.message}")
        end
    end
end

interface Payable
    fn transfer(self!, target: Account, amount: Float)
end

satisfy Account: Payable

async fn process_payment(acc: Account, amount: Float)
    let timer = task.sleep(100)
    await do
        timer
        io.println(f"Processamento concluído para {acc.id}")
    end
end
```

::: tip 💡 Começando em 3 passos simples
1. **Instale ou compile o CLI**: `cargo build --release -p aipo-cli` (ou instale o binário `aipo`)
2. **Execute seu primeiro arquivo**: Siga o tutorial de 5 minutos em [Seu Primeiro Programa](/getting-started/first-program)
3. **Explore as decisões e a jornada**: Conheça todas as waves de implementação na [Trajetória de Desenvolvimento](/trajectory/)
:::

---

## A Trajetória de Engenharia

Ao contrário de linguagens experimentais mantidas sem rastreabilidade, o Aipo foi construído segundo o protocolo de **Engenharia Dirigida por Evidências** governado pelo **Prumo CLI**. Cada fase do compilador, máquina virtual e ecossistema é atestada por suítes completas de testes, benchmarks comparativos e documentação canônica:

| Wave / Fase | Foco Central | Entregáveis & Status |
| :--- | :--- | :--- |
| **Wave 0 (MVP)** | Slices S1 a S11 | Lexer, Parser, HIR, Sema, IR, Bytecode, VM, CLI, Conformance *(100% Concluído)* |
| **Wave 1** | Contratos & Integridade | Invariantes estruturais, rollback com `attempt`, interfaces e funções locais *(100% Concluído)* |
| **Wave 2 (P01)** | Backend JS & Fuzzing | Emissor `aipo-js`, paridade diferencial bit a bit, testes de propriedade e `cargo deny` *(100% Concluído)* |
| **Wave 3 (P02)** | Tipos Ricos & Async | `Set`, `Sequence`, `Bytes`, `async fn`, `await do`, scheduler determinístico *(100% Concluído)* |
| **Wave 4 (P03)** | Host ABI & Poppy | Capabilities deny-by-default, handles geracionais e simulação determinística com semente *(100% Concluído)* |
| **Wave 5 (P04)** | Gestor de Pacotes | Dependências GitHub pinadas por commit SHA, digest verificado e cache offline *(100% Concluído)* |
| **Wave 6 (P05)** | Release v0.1.0 | Delimitação da release da linguagem (ADP-008), C ABI síncrona (ADP-009) e thin proofs *(Em curso)* |

</div>
