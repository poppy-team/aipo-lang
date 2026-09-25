# ADP-010 — Thin proofs de interoperabilidade Rust, C e JavaScript

**Status:** accepted
**Date:** 2026-09-24
**Related:** `docs/adp/ADP-008-v0.1.0-language-release.md`, `docs/adp/ADP-009-synchronous-c-abi.md`
**Authority:** subordinate ao canon; define o critério de interoperabilidade da primeira release da linguagem.

## Decisão

A `aipo v0.1.0` não promete portar bibliotecas completas. Ela entrega uma prova pequena, real e testável para cada fronteira:

- Rust: um módulo host real; egui pode ser a prova preferencial.
- C: uma biblioteca pequena com ciclo de vida e ownership explícitos.
- JavaScript: um módulo com bridge explícito, sem globais implícitos de Node ou browser.

A escolha da biblioteca considera API clara, licença compatível, manutenção, ownership e testabilidade. Popularidade e tamanho não são critérios suficientes.

## Contrato comum

Cada proof deve demonstrar:

- superfície exposta à linguagem;
- conversão de entrada e saída;
- ownership ou handle lifetime;
- capability exigida;
- tradução de erros;
- comportamento observável no teste;
- documentação de integração e exemplo executável.

A prova deve ser pequena o suficiente para revelar problemas de boundary e grande o suficiente para usar uma biblioteca real.

## Rust

O proof Rust deve passar pelo caminho de host API, não por uma extensão específica da linguagem. Uma integração GUI pode usar egui como prova, mas a release não exige um wrapper completo nem um editor.

## C

O proof C deve usar a ABI de `docs/adp/ADP-009-synchronous-c-abi.md`. A biblioteca deve ter create/use/release ou uma forma equivalente de ciclo de vida. Não basta chamar uma função sem erro ou ownership.

## JavaScript

O proof JavaScript deve usar uma bridge nomeada e limitada. O runtime não deve implicitamente expor `window`, `globalThis`, módulos Node ou `eval`. A escolha entre browser e Node é uma decisão do perfil, não uma identidade da linguagem.

## Fora do escopo

- Portar frameworks ou bibliotecas inteiras.
- Node como linguagem hospedeira completa.
- DOM, `fetch`, workers ou WebAssembly.
- Registro de pacotes ou publicação.
- Integração com engines.
- Interop implícita por globais ou `eval`.

## Critério de conclusão

A release pode declarar a interoperabilidade concluída quando os três proofs possuem testes, docs, ownership, capabilities, erros e exemplos executáveis. A implementação de bibliotecas adicionais continua em goals separados.
