# Arquitetura do Perfil Embedded & Shell da Linguagem Aipo

Este documento detalha o desenho técnico e as diretrizes arquiteturais para o **Perfil Embedded & Shell** da linguagem Aipo, visando equiparar a linguagem ao patamar de eficiência, pegada de memória e velocidade de inicialização estabelecido pela linguagem **Lua**.

---

## 1. Visão Geral e Justificativa

A arquitetura do Aipo foi historicamente dividida entre um runtime nativo baseado em pilha (`aipo-vm`) e um destino WebAssembly (`aipo-wasm`). Embora o WebAssembly resolva a portabilidade para a web e sandboxes isoladas, a dependência de motores JIT pesados (como o Wasmtime com backend Cranelift) impõe barreiras intransponíveis para dois casos de uso estratégicos:

1. **Ambientes Embarcados e Microcontroladores (MCUs)**:
   - Dispositivos como ARM Cortex-M, ESP32 e RP2040 possuem orçamentos estritos de hardware: tipicamente entre **64 KB e 512 KB de Flash** e entre **16 KB e 64 KB de RAM estática**.
   - Não possuem MMU (Memory Management Unit) nem suporte a páginas de código executável dinâmico (`mprotect`/W^X).
   - Não comportam runtimes com dependências pesadas do sistema operacional (`std`, threads preemptivas, alocadores globais complexos).

2. **Linguagem de Shell Interativa e Utilitários de Terminal**:
   - Um utilitário de terminal ou uma shell de sistema (`aipo-sh`) requer **tempo de inicialização (*cold start*) inferior a 2 milissegundos**.
   - Não pode tolerar os 20 ms a 50 ms necessários para carregar compiladores JIT, descompactar metadados ou validar módulos WebAssembly.
   - Requer acesso direto e sem intermediários às primitivas de processo do sistema operacional (`fork`, `exec`, descritores de arquivo, pipes `|>` e sinais).

Para viabilizar esses cenários, o Aipo não cria uma bifurcação (*fork*) de linguagem, mas introduz um **Perfil Reduzido de Compilação e Runtime**, projetado sob a filosofia de Lua.

---

## 2. A Filosofia "Estilo Lua": Metas de Engenharia

O ecossistema Lua (Lua 5.4 / Luau) é o padrão ouro de embutibilidade e velocidade de inicialização na indústria. As metas do perfil embedded do Aipo são calibradas diretamente contra essa referência:

| Métrica | Referência Lua 5.4 (C) | Aipo Atual (Host Dev) | Alvo Aipo Embedded / Shell |
| :--- | :--- | :--- | :--- |
| **Tamanho do Binário Final (Stripped)** | ~280 KB | ~25 MB a 30 MB (com Wasmtime) | **~350 KB a 550 KB** |
| **Tamanho do Runtime Mínimo (VM Only)** | ~120 KB | ~1,5 MB (`.rlib`) | **~50 KB a 90 KB** |
| **Consumo Mínimo de RAM Inicial** | ~4 KB a 8 KB | Heap aberto do SO | **~8 KB a 16 KB** |
| **Tempo de Boot / Cold Start** | < 1 ms | 15 ms a 40 ms | **< 2 ms** |
| **Largura do Tipo `Value`** | 16 bytes (`TValue`) | 48 bytes (Enum Rust + `Rc`) | **16 bytes** (Tagged Union / NaN-boxing) |
| **Gerenciador de Memória** | GC Tricolor Incremental | Alocador do SO (`Rc`/`RefCell`) | **Arena Linear de Tamanho Fixo** |
| **Compatibilidade Bare-Metal** | ANSI C puro | Depende de `std` | **`#![no_std]` + `alloc` opcional** |

---

## 3. O que o Perfil Embedded Sacrifica

Para que o compilador e a máquina virtual caibam nessas restrições sem comprometer o núcleo da linguagem, certos subsistemas do compilador de desenvolvimento são substituídos ou cortados no modo reduzido:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       COMPILADOR HOST (Desenvolvimento)                     │
│  • Diagnósticos visuais com cores, sublinhados e sugestões contextuais      │
│  • Tabelas completas de normalização Unicode NFC e regex DFA de 500 KB      │
│  • Suporte obrigatório a Float IEEE-754 de 64 bits                          │
│  • Concorrência preemptiva multi-thread com canais assíncronos              │
└──────────────────────────────────────┬──────────────────────────────────────┘
                                       │ Compilação / Validação Estática
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                       PERFIL EMBEDDED / RUNTIME NANO                        │
│  [Cortado / Substituído]                                                   │
│  ✗ Mensagens de erro extensas ──► Substituídas por códigos (ex: E042:12:4)  │
│  ✗ Tabelas Unicode pesadas   ──► Strings tratadas como UTF-8 cru sem NFC    │
│  ✗ Ponto flutuante forçado    ──► Float desativável via feature flag        │
│  ✗ Heap irrestrito           ──► Arena de memória fixa (16 KB - 64 KB)     │
│  ✗ Threads preemptivas       ──► Concorrência cooperativa ou single-thread  │
│                                                                             │
│  [Preservado e Intacto]                                                     │
│  ✓ `let` imutável por padrão e `var` restrito (zero variáveis globais)      │
│  ✓ `enum` fechado de soma e `match` exaustivo com custo mínimo              │
│  ✓ Operador de pipe funcional e de stream (`|>`)                            │
│  ✓ Tratamento explícito de erros via `Result[T, E]` (zero exceções ocultas) │
│  ✓ Structs com imutabilidade estrutural e métodos vinculados                │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Onde o Aipo Supera Lua no Papel de Shell e Embedded

Embora adote as métricas de tamanho de Lua, o Aipo resolve falhas históricas de ergonomia e segurança que tornam Lua arriscada para automação de infraestrutura:

1. **Eliminação de Globais Invisíveis**: Em Lua, um simples erro tipográfico cria uma nova variável global (`myVar = 10` cria `myvar = nil`). Em Aipo, o escopo léxico estrito com `let` e `var` impede essa classe de falhas estaticamente.
2. **Pipelines Nativos para Shell (`|>`)**: A sintaxe `cmd1() |> cmd2() |> cmd3()` oferece a mesma fluidez do operador `|` do Bash, porém com tipagem estruturada entre comandos em vez de strings desestruturadas.
3. **Casamento de Padrões Confiável**: A instrução de `match` com verificação estática de exaustividade garante que novos estados de processo ou variantes de erro nunca sejam ignorados.
4. **Erros Explícitos vs `pcall`**: Em vez do mecanismo frágil de `pcall` e strings de erro desestruturadas de Lua, Aipo opera com `Result[T, E]` manipulável via operador `?`.

---

## 5. Arquitetura Dual-Track (Wasm vs VM Nativa)

A introdução do perfil embedded consolida a estratégia **Dual-Track** de execução:

1. **Track Web / Cloud (`aipo-wasm`)**:
   - Geração pura de arquivos binários `.wasm` padronizados via `wasm-encoder`.
   - Execução em ambientes onde runtimes Wasm já estão integrados nativamente: navegadores, workers serverless e sandboxes em nuvem.
   - O runner Wasmtime da CLI torna-se uma funcionalidade opcional (`--features wasmtime-runner`), reduzindo o binário padrão da ferramenta.

2. **Track Nativo / Embedded / Shell (`aipo-vm`)**:
   - Motor de execução nativo de alta prioridade técnica.
   - Interpretador de bytecode de passagem direta, sem abstrações intermediárias pesadas.
   - Suporte a exportação como biblioteca estática em C (`libaipo.a`), permitindo embutir Aipo em aplicações escritas em C, C++, Zig ou Rust tão facilmente quanto `liblua.a`.

---

## 6. Pilares de Otimização da `aipo-vm`

Para alcançar o perfil nano, o desenvolvimento da `aipo-vm` focará em três pilares microarquiteturais:

### 1. Compactação de `Value` para 16 Bytes
Atualmente, `Value` ocupa 48 bytes em memória devido aos payloads do enum Rust e referências `Rc`. Reduzir essa estrutura para 16 bytes (usando tagged pointer de duas palavras ou NaN-boxing para arquiteturas de 64 bits) trará os seguintes benefícios:
- Triplicação do número de valores que cabem na mesma linha de cache L1 do processador (64 bytes).
- Redução drástica de falhas de cache (*cache misses*) durante loops intensivos de script.

### 2. Gestão de Memória por Arenas com Desalocação em Bloco
Para scripts curtos de shell e ciclos de processamento de comandos:
- A VM recebe um buffer de memória fixo (exemplo: 32 KB).
- Todas as strings temporárias, listas intermediárias e estruturas são alocadas linearmente nessa arena.
- Ao final da execução do comando ou script, a arena inteira é resetada em tempo O(1) pelo ajuste de um único ponteiro de topo, eliminando a sobretaxa de desalocação individual de objetos.

### 3. Fast-Path de Compilação Direta (Single-Pass)
Para uso interativo em terminal (REPL):
- Criação de um gerador direto de bytecode a partir da AST básica, dispensando passes profundos de análise de grafos quando o script for uma sequência curta de comandos imperativos.
