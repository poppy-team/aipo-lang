# Wave 6 — Rumo à Release v0.1.0 da Linguagem

A **Wave 6 (P05)** consolida a fronteira formal do produto e prepara a primeira release canônica da linguagem: **`aipo v0.1.0`**.

---

## Delimitação Canônica do Produto (ADP-008)

Em conformidade com a decisão arquitetural **ADP-008**, a versão v0.1.0 adota uma estratégia **language-first e escopo fechado**:

- **Dentro do Escopo da v0.1.0**:
  - A linguagem completa: compilador, bytecode, VM, scheduler assíncrono cooperativo, contratos e invariantes.
  - A biblioteca padrão completa com paridade diferencial VM ↔ JS.
  - O utilitário CLI unificado (`aipo run`, `check`, `build`, `disasm`, `fmt`, `package`).
  - O test runner minimalista embutido.
  - A C ABI síncrona versionada (ADP-009).
  - A API de embedding em Rust e três provas finas de interoperabilidade (*thin proofs*) em Rust, C e JavaScript (ADP-010).

- **Fora do Escopo da v0.1.0 (Pós-V1)**:
  - Motores de jogos completos e editores visuais.
  - Registro público centralizado de pacotes na nuvem.
  - Profiles web amplos e extensões para editores de código complexas.

---

## C ABI Síncrona Versionada (ADP-009)

Para permitir que a linguagem Aipo seja incorporada com facilidade em projetos escritos em C, C++, Zig ou qualquer outra linguagem com FFI, a Wave 6 define uma interface binária de aplicação (*Application Binary Interface* - ABI) estável em C:

- Tipos opacos para instâncias da VM e bytecode.
- Execução síncrona com passagem de valores primitivos e captura de códigos de diagnóstico.
- Zero alocação de memória no lado do chamador.

---

## Provas Finas de Interoperabilidade (ADP-010)

Antes de declarar a v0.1.0 estável, o projeto valida sua viabilidade em três ambientes distintos através de testes de integração dedicados:

1. **Rust Host Embedding**: Uma aplicação em Rust inicializando a VM, registrando funções nativas personalizadas e executando scripts Aipo.
2. **C Host Integration**: Um binário em C consumindo a biblioteca estática/dinâmica compilada e trocando dados primitivos.
3. **JavaScript / Node.js Runtime**: Um projeto em Node.js importando o código emitido por `aipo-js` e rodando o runtime shim em conjunto com bibliotecas do ecossistema JS.
