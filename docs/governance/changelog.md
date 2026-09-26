# Changelog Oficial do Projeto

Todas as alterações notáveis, releases e marcos do Aipo são documentados neste arquivo, em conformidade com o formato [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/) e aderindo ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

---

## [Não Lançado — Rumo à v0.1.0]

### Produto & Fronteiras
- **Recorte da `aipo v0.1.0` (ADP-008)**: A primeira release oficial é focada exclusivamente na linguagem, com async cooperativo, CLI unificada, test runner minimalista, C ABI síncrona versionada (ADP-009) e thin proofs de interoperabilidade Rust/C/JS (ADP-010).

### Performance & Rigor Empírico
- **Cross-Language Benchmark Suite**: Ferramenta `aipo-bench --compare` cobrindo 6 workloads determinísticos contra Lua, LuaJIT, Wren, Luau, CPython, PyPy, Ruby e Rust nativo.
- **Cache Monomórfico de Field Slot**: Aceleração de `GetField`/`SetField` resultando em redução de 28,31% no workload `fields`.
- **Cache de Frame Base**: Redução consistente em `arithmetic` (-3,71%), `fields` (-6,65%) e `recursion` (-4,17%).
- **Eliminação de Clones em `SetField`**: Eliminação determinística de 600.000 clones de `Value` para estruturas sem proteção.
- **Relatório Transparente de Experimentos Revertidos**: Documentação de 7 experimentos de micro-otimização revertidos por falta de ganho mensurável no wall-clock, consolidando a metodologia de medição por tempo de CPU e encerrando a trilha de micro-ajustes locais.

### Gestor de Pacotes & Ecossistema Hermético (P04-G01 a P04-G11)
- **Manifesto & Lockfile**: Coordenadas `namespace.package`, arquivo `aipo.toml` e `aipo.lock` determinístico.
- **Dependências Remotas**: Suporte a dependências GitHub pinadas obrigatoriamente por commit SHA de 40 dígitos.
- **Cache Local & Verificação**: Subcomandos `aipo package fetch-github`, `aipo package cache verify` e `aipo package cache prune --apply`.
- **Autenticação Segura**: Suporte a tokens bearer via variável de ambiente (`--github-token-env`), sem jamais gravar segredos em disco ou logs.

### Bytecode & Desassembly
- **Serialização Binária `.aibc`**: Formato binário nativo determinístico com verificação mágica `AIBC` v1.
- **Execução Direta**: `aipo run app.aibc` executa sem passar pelo pipeline frontend.
- **Desassemblador `aipo disasm`**: Mapeamento preciso de instruções de bytecode para linhas e colunas do código-fonte.

---

## [Wave 4] — Host ABI & Poppy Simulation
- Crate `aipo-host` com árvore de permissões (`CapabilitySet`) deny-by-default.
- Handles geracionais (`HandleTable`) imunes a falhas de use-after-free.
- Prevenção estrita de escape de escopo (`Scope Escape`) nos 6 pontos de publicação da VM.
- Crate `aipo-poppy` com simulação headless determinística via ECS e buffer de comandos.

---

## [Wave 3] — Tipos Ricos & Async Cooperativo
- Tipos `Set` com ordem de inserção, `Sequence` lazy e `Bytes` packing.
- Sintaxe `async fn` e bloco `await do ... end`.
- Scheduler cooperativo com tempo virtual e combinadores `task.*`.
- Detecção em runtime de ciclos transitivos de await (`AIPO_RT_AWAIT_CYCLE`).

---

## [Wave 2] — Paridade JavaScript & Deep Quality
- Compilador semântico `aipo-js` com emissão de ES2022 e Source Maps.
- Suíte de conformance diferencial VM ↔ Node.js.
- Fuzzing com libFuzzer e testes de propriedade com proptest.
- Auditoria de segurança de dependências com `cargo deny`.

---

## [Wave 1] — Contratos Estruturais & Transações
- Hooks `init()` e `invariant()`.
- Rollback automático de mutações através de journal em blocos `attempt ... recover`.
- Interfaces formais e checagem de conformidade de assinaturas.
- Funções locais aninhadas com auto-recursão (`FillSelfCapture`).

---

## [Wave 0] — Fundação da Linguagem (MVP)
- Implementação inicial dos 11 slices (Lexer, Parser, HIR, SEMA, IR, Bytecode, VM, Stdlib, Formatter, CLI, Conformance).
