---
title: Estado dos backends
description: Como entender cobertura e maturidade sem misturar código existente com teste aprovado.
---

# Compatibilidade e limites

O Aipo possui implementações reais para diferentes destinos. **Não existe uma certificação universal de paridade** apenas porque os crates estão presentes.

## Entenda a diferença

| Motor | Implementação identificada | Precisa ser confirmado |
| --- | --- | --- |
| **VM de pilha** | `aipo-vm`: valor, chamadas e execução | Testes de regressão no commit atual |
| **RegVM** | `aipo-vm/src/reg_vm.rs`: emissor e executor | Cobertura parcial de closures, async, host e contratos |
| **JavaScript** | `aipo-js`: emissão ESM e runtime shim | Comparação diferencial com a VM em cada recurso |
| **WebAssembly** | `aipo-wasm`: HIR, emissor e runner | Recursos de host, traps, fuel e subset suportado |
| **C ABI** | `aipo-c-abi` e harness Rust/C | Ownership, versão ABI e integração com consumidor |

## Limitações registradas

A auditoria [P07-G01](https://github.com/poppyTM/aipo-lang/blob/main/docs/evidence/P07-G01-runtime-hardening.md) documenta partes da RegVM ainda sem paridade completa: métodos, closures/upvalues, async, hooks, invariantes, journal e host. Ela também registra verificações de compilação e testes **não executados na rodada descrita**.

A presença de uma suite de testes não é prova de que ela passou na última revisão. Consulte o [painel de progresso](/progress/) para ver critérios e evidências, sem interpretar seus percentuais como a porcentagem de linguagem pronta.

## Como testar sua necessidade

1. Escolha um [programa completo](/examples/) que use o recurso.
2. Execute primeiro na VM de referência.
3. Compile ou rode no segundo destino, usando as flags adequadas.
4. Compare resultados e erros; anote o SHA, o ambiente e os comandos.
5. Se houver divergência, abra um bug e não anuncie paridade geral.

## Código e testes que ajudam

- [Conformance de programas](https://github.com/poppyTM/aipo-lang/tree/main/docs/conformance/programs).
- [Testes diferenciais da CLI](https://github.com/poppyTM/aipo-lang/tree/main/crates/aipo-cli/tests).
- [RegVM](https://github.com/poppyTM/aipo-lang/blob/main/crates/aipo-vm/src/reg_vm.rs).
- [Wasm](https://github.com/poppyTM/aipo-lang/tree/main/crates/aipo-wasm).

[Escolher backend](/manual/backends) · [Comandos de build](/reference/cli).
