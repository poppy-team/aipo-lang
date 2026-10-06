# Arquitetura do Perfil Embedded & Shell da Linguagem Aipo

Este documento estabelece a especificação técnica detalhada para o **Perfil Embedded & Shell** da linguagem Aipo, definindo as decisões de microarquitetura, modelo de memória, sistema de tipos e laço de despacho da máquina virtual (`aipo-vm`) calibrados contra a referência da linguagem **Lua 5.4**.

---

## 1. Visão Geral e Justificativa

A linguagem Aipo possui dois destinos principais de compilação: a máquina virtual nativa (`aipo-vm`) e o emissor WebAssembly (`aipo-wasm`). Embora o WebAssembly viabilize execução segura em navegadores e plataformas de nuvem isoladas, a dependência de um motor JIT massivo (Wasmtime com Cranelift) inviabiliza dois cenários estratégicos:

1. **Ambientes Embarcados e Microcontroladores (MCUs)**:
   - Dispositivos bare-metal (Cortex-M0/M3/M4, ESP32, RP2040) operam com orçamentos restritos: **64 KB a 512 KB de Flash** e **16 KB a 64 KB de RAM**.
   - Não possuem MMU (Memory Management Unit) nem permissão de memória executável dinâmica (`mprotect`/W^X).
   - Não suportam a biblioteca padrão completa do sistema operacional (`std`, threads de kernel, alocadores abertos).

2. **Linguagem de Shell Interativa e Utilitários de Terminal**:
   - Um utilitário de terminal ou uma shell de sistema (`aipo-sh`) requer **tempo de inicialização (*cold start*) inferior a 2 milissegundos**.
   - A sobretaxa de compilação JIT (15 ms a 50 ms) degrada sensivelmente a experiência em scripts encadeados e comandos de linha interativa.
   - Requer integração direta e síncrona com os descritores de arquivo, processos do sistema operacional e streams de pipes `|>` sem a virtualização restritiva do WASI.

Para atender a esses requisitos sem fragmentar o ecossistema, o Aipo estabelece uma estratégia **Dual-Track** combinada a uma **evolução progressiva em 3 camadas**.

---

## 2. A Estratégia Dual-Track (Wasm Web vs VM Nativa)

```
                  ┌────────────────────────────────────────┐
                  │             Código Aipo                │
                  │   (Frontend Canônico: Lexer / Sema)    │
                  └───────────────────┬────────────────────┘
                                      │
            ┌─────────────────────────┴─────────────────────────┐
            ▼                                                   ▼
 ┌──────────────────────┐                           ┌───────────────────────┐
 │     Track Web/Cloud  │                           │    Track Nativo/Embed │
 │   `aipo-wasm` (.wasm)│                           │  `aipo-bytecode` (.aibc)
 ├──────────────────────┤                           ├───────────────────────┤
 │ • Emissor: wasm-enc. │                           │ • VM: `aipo-vm`       │
 │ • Destino: Navegador,│                           │ • Destino: CLI Rápido,│
 │   Workers, Sandboxes │                           │   Shell, Jogos nativos│
 │ • Runner Wasmtime:   │                           │   e MCUs (`no_std`)   │
 │   Opcional (Feature) │                           │ • Sem dependências JIT│
 └──────────────────────┘                           └───────────────────────┘
```

- **O WebAssembly não é abandonado**: O crate `aipo-wasm` permanece ativo como emissor oficial para a Web e ambientes serverless que já possuem runtimes Wasm nativos.
- **O Wasmtime torna-se opcional**: O runtime JIT Wasmtime deixa de estar embutido por padrão na CLI, virando uma *feature flag* opcional (`--features wasmtime-runner`).
- **A `aipo-vm` assume o caminho crítico**: Todo o desenvolvimento nativo foca na nossa própria máquina virtual, garantindo autonomia, compacidade e velocidade de boot.

---

## 3. Modelo de Evolução Progressiva em 3 Camadas

O design da máquina virtual segue um modelo de três camadas concêntricas, onde a camada central viabiliza as camadas superiores sem reescrita de código:

```
┌────────────────────────────────────────────────────────┐
│ Camada 3: Bare-Metal MCU (Horizonte Futuro)            │
│   • HAL de periféricos (GPIO, UART, I2C, Timers)       │
│   • `#![no_std]` estrito, zero dependência de SO       │
│   • Execução direta da memória Flash (XIP)             │
└───────────────────────────▲────────────────────────────┘
                            │
┌───────────────────────────┴────────────────────────────┐
│ Camada 2: System Shell (CLI Standalone)                │
│   • Builtins de processo (`spawn`, `pipe`, `env`, `fs`)│
│   • REPL interativo e pipes de dados `|>`              │
│   • Binário único estático (< 1 MB), boot < 2 ms       │
└───────────────────────────▲────────────────────────────┘
                            │
┌───────────────────────────┴────────────────────────────┐
│ Camada 1: Host Scripting (Motor Central)               │
│   • `aipo-vm` como biblioteca C ABI (`libaipo.a`)      │
│   • Estado isolado por instância (`VmContext` reentrante│
│   • Zero dependência de SO: I/O via callbacks do host  │
│   • Modelo de memória compacto (16 bytes por valor)    │
│   • Gerenciamento por arenas com desalocação O(1)      │
└────────────────────────────────────────────────────────┘
```

### Camada 1: Host Scripting (A Base Reentrante)
- **Biblioteca Estática (`libaipo.a`)**: Exporta interface C estável (`aipo_new_state`, `aipo_load_bytecode`, `aipo_call`, `aipo_free_state`).
- **Estado 100% Reentrante**: Zero variáveis globais estáticas (`static mut`) ou singletons. Todas as variáveis de execução residem na struct `VmContext`.
- **I/O Abstrato**: A VM não realiza chamadas diretas de disco ou rede; o hospedeiro (seja o motor Poppy, uma aplicação C++ ou uma engine de jogos) fornece os bytes e recebe os eventos.

### Camada 2: System Shell (Automação de Infraestrutura)
- **Executável Único (`aipo-sh`)**: Um binário stripped de ~400 KB a 600 KB.
- **Syscalls Injetadas**: Injeta no `VmContext` primitivas de automação (`sh.run`, `sh.env`, `sh.pipe`).
- **Substituição de Bash/Python**: Scripts tipados estaticamente, prevenindo falhas silenciosas no meio de deploys e automações.

### Camada 3: Bare-Metal MCU (Firmware e Sensores)
- **`#![no_std]` + `alloc` opcional**: Funciona diretamente sobre registradores de hardware e tabelas de vetores de interrupção.
- **Execute-In-Place (XIP)**: Lê o bytecode diretamente da memória Flash, preservando a memória RAM exclusivamente para variáveis ativas.

---

## 4. Metas de Engenharia Calibradas contra Lua 5.4

| Métrica | Referência Lua 5.4 (C) | Aipo Atual (Host Dev) | Alvo Aipo Embedded / Shell |
| :--- | :--- | :--- | :--- |
| **Tamanho do Binário Final (Stripped)** | ~280 KB | ~25 MB a 30 MB (com Wasmtime) | **~350 KB a 550 KB** |
| **Tamanho do Runtime Mínimo (VM Only)** | ~120 KB | ~1,5 MB (`.rlib`) | **~50 KB a 90 KB** |
| **Consumo Mínimo de RAM Inicial** | ~4 KB a 8 KB | Heap aberto do SO | **~8 KB a 16 KB** |
| **Tempo de Boot / Cold Start** | < 1 ms | 15 ms a 40 ms | **< 2 ms** |
| **Largura do Tipo `Value`** | 16 bytes (`TValue`) | 48 bytes (Enum Rust + `Rc`) | **16 bytes** (Tagged Union / NaN-boxing) |
| **Arquitetura de Bytecode**| Registradores Virtuais | Pilha Pura (Stack VM) | **Registradores Virtuais (32-bit u32)** |
| **Gerenciador de Memória** | GC Tricolor Incremental | Alocador do SO (`Rc`/`RefCell`) | **Arena Linear com Reset O(1)** |
| **Compatibilidade Bare-Metal** | Totalmente portável (`no_std` C)| Depende de `std` | **`#![no_std]` + `alloc` opcional** |

---

## 5. Modelo de Memória e Modelo de Valores (`Value`)

### 5.1 O Tipo `Value` de 16 Bytes
Atualmente, `Value` ocupa 48 bytes. A nova arquitetura adota um layout compacto de **duas palavras de 64 bits (16 bytes)**:

```
┌─────────────────────────────────┬─────────────────────────────────┐
│         Palavra 0 (8B)          │         Palavra 1 (8B)          │
│   Tag de Tipo (u8) + Metadados  │        Payload Primitivo        │
│                                 │     ou Ponteiro de Referência   │
└─────────────────────────────────┴─────────────────────────────────┘
```

- **Inteiros (`Int`)**: Armazenados inline na Palavra 1 (`i64` ou 53 bits seguros).
- **Booleanos, Bytes e None**: Armazenados inline sem tocar no heap.
- **Ponto Flutuante (`Float`)**: Armazenado inline como `f64`.
- **Strings, Listas, Structs e Closures**: A Palavra 1 armazena um ponteiro de 64 bits (ou offset relativo de 32 bits em MCUs) apontando para a arena de memória gerenciada.

> **Ganho Microarquitetural:** Triplica a quantidade de valores que cabem na mesma linha de cache L1 (64 bytes), reduzindo drasticamente falhas de cache (*cache misses*).

### 5.2 Gerenciamento de Memória por Arenas com Reset O(1)
Em vez de um Garbage Collector complexo que causa pausas imprevisíveis (*latency spikes*) e fragmentação de heap:

1. **Alocação Sequencial (*Bump Allocation*)**: Todas as strings intermediárias, listas e instâncias temporárias criadas durante um comando ou ciclo de script são alocadas sequencialmente em um bloco contíguo de memória pré-alocado (ex.: 32 KB).
2. **Desalocação Instantânea em O(1)**: Ao término do comando de shell ou do frame de execução, o ponteiro de topo da arena é resetado para a base. Todas as alocações são liberadas simultaneamente sem chamadas individuais a `free()`.
3. **Cota Rígida (*Hard Quota*)**: Se o script ultrapassar a capacidade da arena, a VM interrompe a execução com um erro seguro de esgotamento (`E_OUT_OF_MEMORY`), impedindo corrupção de memória.

### 5.3 O Contexto Reentrante (`VmContext`)
```rust
pub struct VmContext<'a> {
    /// Array contíguo de registradores virtuais ativos (Values de 16 bytes).
    registers: &'a mut [Value],
    /// Tabela de frames de chamada ativos.
    call_frames: &'a mut [CallFrame],
    /// Alocador de arena linear para objetos dinâmicos.
    arena: ArenaAllocator<'a>,
    /// Ponteiros de funções nativas injetadas pelo hospedeiro.
    host_callbacks: &'a [HostCallback],
}
```

---

## 6. Sistema de Tipos Estratificado

Para conciliar a riqueza do Aipo com o suporte a chips minúsculos, os tipos são divididos em dois níveis:

### Tier 1: O Núcleo Universal Mandatório
Disponível em **todos** os alvos, operando sem alocador de sistema e em menos de **30 KB de código compilado**:
- **`Int`**: Inteiros com sinal de 64 bits (ou 32 bits em MCUs).
- **`Bool`**: Booleano inline.
- **`Byte`**: Unidade binária básica de 8 bits.
- **`Bytes`**: Fat pointer `(&[u8], len)` para I/O e buffers sem cópia.
- **`String`**: Fat pointer `(&str, len)` para strings UTF-8 imutáveis.
- **`Enum`**: Tag numérica `u16` + payload opcional. Custo idêntico a um inteiro primitivo.
- **`Struct` Estático**: Array contíguo de `Value` com acesso a campos resolvido pelo compilador para offsets numéricos fixos (`GetField 0`, `GetField 1`). Zero custo de tabelas hash.
- **`Function` / `Native`**: Despacho direto de instrução ou ponteiro C.

### Tier 2: Tipos Configuráveis por Perfil (`Features`)
- **`Float` (Ponto Flutuante IEEE-754)**: Desativado em microcontroladores sem FPU de hardware, economizando de 15 KB a 35 KB de rotinas matemáticas de software.
- **`Dict` (Tabelas Hash Dinâmicas)**: Ativado em shell e desktops; em MCUs, substituído por structs estáticos ou listas lineares de pares chave-valor.
- **`List` de Capacidade Aberta**: Ativado em shell e desktops; em MCUs, substituído por arrays de tamanho fixo (`Array[T, N]`) alocados na arena.

### Perfis Finais de Compilação no Workspace
1. **`profile-nano`**: Tier 1 puro, `#![no_std]`, arena estática, sem float por software. Binário da VM: **~40 KB a 70 KB**.
2. **`profile-shell`**: Tier 1 + Tier 2 completo, builtins de processo (`run`, `pipe`, `env`), inicialização < 2 ms. Binário final: **~400 KB a 600 KB**.
3. **`profile-full`**: Tudo do perfil shell + gerador WebAssembly (`aipo-wasm`), diagnósticos formatados e ferramentas estáticas.

---

## 7. Bytecode de Registradores Virtuais e Laço de Despacho

### 7.1 Transição de Stack VM para Register VM
A máquina virtual migra do modelo baseado em pilha para uma **máquina de registradores virtuais**:

- **Modelo Antigo (Pilha)**:
  `c = a + b` exigia 4 instruções (`GetLocal a`, `GetLocal b`, `Add`, `SetLocal c`) com 4 ciclos de despacho e 4 acessos de topo de pilha.
- **Novo Modelo (Registradores)**:
  `c = a + b` executa uma única instrução:
  ```text
  Add R(c), R(a), R(b)
  ```
- **Resultado Prático**: Redução de 35% a 50% nas instruções executadas por programa, reduzindo pela metade os desvios de previsão de salto (*branch mispredictions*) na CPU.

### 7.2 Formato de Instrução Fixa de 32 Bits (`u32`)
Cada instrução ocupa exatamente 4 bytes alinhados:

```
 0       6 7             14 15            23 24            31
┌─────────┬────────────────┬────────────────┬────────────────┐
│ Opcode  │       A        │       B        │       C        │
│ (7 bits)│    (8 bits)    │    (9 bits)    │    (9 bits)    │
└─────────┴────────────────┴────────────────┴────────────────┘
   128          256             512              512
 instruções  registradores    registradores /  registradores /
                              constantes       constantes
```

- **`iABC`**: Operações binárias e chamadas (`Opcode A B C`).
- **`iABx`**: Carga de constantes e literais (`Opcode A Bx`, com `Bx` não-sinalizado de 18 bits).
- **`iAsBx`**: Saltos e desvios condicionais (`Opcode A sBx`, com `sBx` sinalizado de 18 bits).

### 7.3 Decodificação e Despacho Direto
- A decodificação é realizada via operações diretas de deslocamento de bits (*bitwise shifts*) sem ramificações lógicas condicionais:
  ```rust
  let opcode = (raw & 0x7F) as u8;
  let reg_a = ((raw >> 7) & 0xFF) as usize;
  let reg_b = ((raw >> 15) & 0x1FF) as usize;
  let reg_c = ((raw >> 24) & 0x1FF) as usize;
  ```
- No modo seguro Rust, o laço de execução utiliza tabela de saltos densa gerada pelo LLVM. Na C ABI (`libaipo.a`), adota-se *Direct Threaded Code* com ponteiros calculados (`goto *dispatch_table[opcode]`).

### 7.4 Integração Nativa com Pipes de Shell (`|>`)
Pipelines do Aipo conectam-se diretamente aos registradores virtuais:
```aipo
ps() |> grep("node") |> count()
```
O compilador emite transferências diretas de registradores de stream:
```text
Call      R(0), ps, 0
CallPipe  R(1), grep, R(0), "node"
CallPipe  R(2), count, R(1)
```
Em nível de sistema operacional, os descritores de arquivo são vinculados diretamente via `pipe2` do kernel, transmitindo buffers de bytes diretamente entre comandos sem cópias intermediárias na memória da VM.

---

## 8. Roteiro de Engenharia em 4 Marcos

1. **Marco 1 — Desacoplamento e Enxugamento**:
   - Criação da feature flag `wasmtime-runner` no `crates/aipo-cli` (opcional).
   - Inclusão do perfil `[profile.nano]` no `Cargo.toml`.
2. **Marco 2 — Compactação de `Value` e Arenas**:
   - Refatoração do layout de `Value` para 16 bytes.
   - Implementação de `ArenaAllocator` com reset O(1).
3. **Marco 3 — Bytecode de Registradores Virtuais**:
   - Implementação da especificação de instruções de 32 bits em `crates/aipo-bytecode`.
   - Reestruturação do laço de despacho em `crates/aipo-vm`.
4. **Marco 4 — CLI de Shell (`aipo-sh`) e Exportação C ABI**:
   - Criação do binário standalone de shell com builtins de processo.
   - Validação da biblioteca estática `libaipo.a` em provas finas de C e Rust.
