---
title: "Executar na VM, JavaScript e WebAssembly"
description: "Referência de uso organizada por tarefas e comportamento do Aipo."
---

# Executar na VM, JavaScript e WebAssembly

O Aipo possui **mais de um caminho de execução**, mas isso não significa que uma feature tem o mesmo suporte em todos eles. Escolha o destino pela necessidade do projeto e verifique a compatibilidade.

## Visão rápida

| Destino | Para quê | Cuidado |
| --- | --- | --- |
| VM de pilha (`vm`) | Referência funcional para programas Aipo | Validar com testes reais |
| RegVM (`reg`) | Backend experimental de registradores | Subset; não presumir paridade |
| JavaScript (`js`) | Gerar bundle ESM | Runtime shim e divergências a verificar |
| WebAssembly (`wasm`) | Executar binário Wasm/WASI | Recursos do host e cobertura variam |

## Gerar JavaScript

```bash
cargo run -q -p aipo-cli -- build examples/06_variables_and_values.aipo --target js --out dist/
```

## Gerar WebAssembly

```bash
cargo run -q -p aipo-cli -- build examples/06_variables_and_values.aipo --target wasm --out dist/
```

O compilador Wasm trabalha diretamente sobre HIR em partes de seu pipeline. A existência do backend não certifica o suporte a todas as operações de host, async ou biblioteca padrão.

## RegVM: cuidado com falsas promessas

A auditoria [P07-G01](https://github.com/poppyTM/aipo-lang/blob/main/docs/evidence/P07-G01-runtime-hardening.md) registra limitações de closures/upvalues, async, hooks, invariantes, journal e host. O projeto fornece diagnósticos de recursos não suportados; confira antes de substituir a VM de referência.

**Pratique:** execute o mesmo programa simples na VM e compile para JS. Compare apenas recursos que o destino declara suportar.

[Matriz de compatibilidade](/reference/status) · [Exemplos reais](/examples/).
