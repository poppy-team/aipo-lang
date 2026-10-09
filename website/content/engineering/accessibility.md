---
title: Acessibilidade cognitiva e técnica
---
# Acessibilidade como contrato de engenharia

Aipo considera a legibilidade importante para pessoas com TDAH, dislexia e diferentes estilos cognitivos. A avaliação deve combinar texto, ferramentas e tarefas reais, sem presumir que uma única preferência serve para todos.

## Sintaxe

- Uma forma canônica por operação, quando isso reduz ambiguidade.
- Nomes completos e erros com localização, causa e próxima ação.
- Blocos e efeitos explícitos, evitando sobrecarga de pontuação.
- Casos simples fáceis de ler antes dos recursos avançados.

## CLI e diagnósticos

- Código e mensagem não dependem apenas de cor.
- Texto human-readable e JSONL mantêm significado equivalente.
- Mensagens evitam termos acusatórios e caminhos vagos.
- O diagnóstico identifica o lugar, o contrato e como reparar o programa.
- Nunca remova pistas importantes para ficar “mais bonito”.

## Site documental

- Estrutura HTML semântica e cabeçalhos previsíveis.
- Links discerníveis e foco visível.
- Navegação por teclado, modo foco e opção de espaçamento.
- Conteúdo acessível em zoom alto; sem navegação apenas por hover.
- Respeito a `prefers-reduced-motion`.
- Código com legendas, saídas textuais e exercícios de uma mudança por vez.

## Critérios de validação

Executar auditoria de teclado, zoom e leitores de tela; realizar teste de tarefa com diferentes pessoas e registrar dificuldades concretas. Conformidade declarada com WCAG requer auditoria real: tokens e CSS por si só não certificam o produto.

Veja [Princípios da linguagem](/concepts/philosophy) e [Rubrica de diagnósticos](https://github.com/poppy-team/aipo-lang/blob/main/docs/testing/diagnostic-accessibility-rubric.md).
