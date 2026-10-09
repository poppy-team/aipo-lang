---
title: Biblioteca padrão
---
# Biblioteca padrão

A documentação extensa da Stdlib permanece em [docs/manual/stdlib.md](https://github.com/poppy-team/aipo-lang/blob/main/docs/manual/stdlib.md), e os contratos mínimos de runtime em [docs/stdlib/mvp-subset.md](https://github.com/poppy-team/aipo-lang/blob/main/docs/stdlib/mvp-subset.md). A nova referência separa conceitos, módulos e capacidades.

| Família | Operações e responsabilidade |
| --- | --- |
| `io` | Saída textual e fronteiras I/O |
| `math` | Operações numéricas |
| `string` | Texto, transformação, Unicode |
| `List`, `Dict`, `Set`, `Sequence` | Coleções e métodos associados |
| `json` | Parse e serialização estruturada |
| `binary` e `Bytes` | Manipulação de buffers |
| `path`, `url`, `regex` | Processamento de caminhos e padrões |
| `random`, `time` | Geradores e noções temporais |
| `task` | Tarefas e combinadores assíncronos |
| `testing`, `expect` | Asserções e suporte a testes |
| `fs`, `env` | Recursos condicionados a capacidades de host |

## Regras importantes

**Métodos têm aridade definida.** Por exemplo, `Dict.get(chave)` não deve ser documentado com segundo argumento de fallback. **Capacidade de host é distinta de existência da função**: o módulo pode existir e seu acesso ser negado em um ambiente.

## Antes de usar uma chamada

1. Consulte seu nome e assinatura na referência específica.
2. Confira a implementação em `crates/aipo-stdlib`.
3. Verifique as fixtures e o backend escolhido.
4. Trate retornos opcionais e falhas recuperáveis explicitamente.

Esta página substitui a necessidade de decorar um índice monolítico, mas ainda não é uma referência gerada automaticamente de todas as assinaturas. Essa automação é um item de qualidade da [engenharia](/engineering/stdlib-packages).
