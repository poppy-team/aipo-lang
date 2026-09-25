# ADP-009 — C ABI síncrona e versionada

**Status:** accepted
**Date:** 2026-09-24
**Related:** `docs/adp/ADP-008-v0.1.0-language-release.md`, `docs/crates/crate-contracts.md`, `crates/aipo-host/`
**Authority:** subordinate ao canon; define a primeira boundary de embedding, não uma engine.

## Decisão

A `aipo v0.1.0` expõe uma ABI C síncrona, single-threaded e versionada. A ABI será implementada em um crate separado, sem expor structs, referências, lifetimes ou panics de Rust.

A API interna `aipo-host` continua sendo o contrato de dados. A ABI C é o adaptador para linguagens e engines externas.

## Contrato mínimo

- runtime opaco com create/destroy explícitos;
- carregamento de módulo e chamada de código Aipo;
- registro de funções host por módulo e capability;
- valores simples copiados por valor;
- handles opacos para objetos controlados pelo host;
- erros estáveis e texto sanitizado;
- regras de ownership documentadas para cada valor e handle;
- ABI versionada para permitir evolução compatível.

A primeira versão não cria threads nem expõe `Future`, callbacks async ou um event loop próprio. A semântica async da linguagem continua interna ao runtime; `step` e `run_until_idle` poderão ser adicionados em uma versão futura.

## Segurança e ciclo de vida

- O caller é dono do runtime e dos handles que ele vê.
- O host libera handles; o runtime nunca inventa ownership para referências externas.
- Capability negada, handle stale e erro de contrato viram erros estáveis.
- Nenhum erro de Rust cruza a fronteira.
- Nenhum módulo dinâmico ou `eval` entra no perfil sandboxed.

## Prova obrigatória

Um host C headless deve:

1. criar e destruir um runtime;
2. carregar um módulo Aipo;
3. registrar uma função host;
4. chamar a função com valor simples e handle;
5. observar sucesso, `Failure` e fault;
6. repetir o fluxo sem panic, leak ou acesso inválido.

A prova deve ter fixtures, documentação e teste de ownership. Godot, engines e plugins são consumidores posteriores, não parte deste ADR.

## Evolução futura

Async será evolução aditiva da ABI, não uma mudança silenciosa da v0. A API futura deverá preservar a execução síncrona e adicionar apenas operações de pump/event loop necessárias ao host.

## Fora do escopo

- Threads e `Send`/`Sync` no VM.
- Carregamento dinâmico de bibliotecas.
- `eval` ou acesso a memória arbitrária.
- Registro global de módulos.
- Integração específica com Godot ou qualquer engine.
- Codec, filesystem, assets ou policies específicas de produto.
