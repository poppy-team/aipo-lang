---
title: Estado da implementação e backends
---
# O que está implementado?

**Base editorial:** `main` em 9 de outubro de 2026, após o merge do PR #19. Esta página não substitui execução de testes na versão instalada.

| Componente | Estado documentado | Observação |
| --- | --- | --- |
| Stack VM | Implementação principal | Substrato de referência do corpus |
| Register VM | Experimental | Cobertura parcial; rejeita operações não representadas |
| Emissor JavaScript | Existente | Paridade deve ser verificada caso a caso |
| Emissor Wasm | Existente | Compila a partir de HIR; suporte e runner dependem de features |
| CLI (`run`, `check`, `test`, `fmt`, `build`) | Implementado | Flags variam de acordo com features e versão |
| ABI C e Host Schema | Implementação disponível | Compatibilidade, permissões e limites exigem validação |
| Perfil Embedded/Shell mínimo | Proposta/evolução | Não declarar como runtime finalizado |
| Gramática ADR-001 V1 | Alvo parcialmente migrado | `SYNTAX.md` documenta o delta |
| Frameworks e pacotes de UI/jogos | Maturidade heterogênea | Não confundir documentação e integração de host completa |

## Como verificar uma afirmação

1. Identifique o commit e as features de compilação.
2. Escolha uma fixture em `docs/conformance/`.
3. Execute na Stack VM.
4. Repita nos demais destinos relevantes.
5. Registre comando, resultado e diferença de saída.

O documento de [hardening de runtime](https://github.com/poppy-team/aipo-lang/blob/main/docs/development/runtime-hardening-guide.md) registra correções recentemente publicadas, mas testes não foram executados naquela reconstrução. **Não exiba selo `verified` por mera presença de código ou teste adicionado.**
