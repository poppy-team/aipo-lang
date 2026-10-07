# Aipo — Guia Canônico de Engenharia e Codificação para LLMs

Este diretório contém a especificação formal, regras gramaticais negativas/positivas, padrões de Clean Code e arquiteturas de projeto voltadas para modelos de linguagem (LLMs) codificarem em **Aipo V1** sem alucinações.

---

## 1. Por que este guia existe?

Modelos de linguagem modernos (Claude, GPT-4o, DeepSeek, Gemini, Llama) foram treinados em bilhões de linhas de Python, JavaScript, C, C++, Rust e Go. Como Aipo é uma linguagem com convenções sintáticas deliberadamente desenhadas para baixa sobrecarga cognitiva e neurodivergência (ADR-001):

1. **Aipo não existe no pré-treino das LLMs:** O modelo tenta operar por analogia com outras linguagens.
2. **Vieses fortes causam erros fatais:** 
   - A LLM tenta usar `//` para comentário (em Aipo, `//` é divisão inteira).
   - A LLM tenta usar `for item in lista` (em Aipo, `for` não existe; usa-se `each item in lista`).
   - A LLM tenta usar `try / catch / throw` (em Aipo, usa-se `attempt / failed` e `fail`).
   - A LLM tenta usar `class` ou `impl Tipo { fn metodo() }` (em Aipo, usa-se `struct` pura e associação `Tipo:metodo()`).
   - A LLM tenta usar `&&` e `||` (em Aipo, operadores lógicos são palavras inteiras: `and` e `or`).
3. **Dialeto em evolução:** O compilador anterior aceitava `end`, `div`, `self!`, `impl` e `satisfy`. A V1 canônica **removeu** todas essas formas. Este guia blinda a LLM contra dialetos legados.

---

## 2. Estrutura deste Guia

| Documento | Foco | Uso Recomendado |
|---|---|---|
| [`rules.md`](rules.md) | **Regras Positivas e Negativas** | Tabela de proibições imediatas, mapeamento "De $\rightarrow$ Para", armadilhas críticas e gramática estrita. |
| [`clean-code.md`](clean-code.md) | **Clean Code & Design Cognitivo** | Princípios de clareza cognitiva, modelagem de dados imutáveis por padrão, contratos (`invariant`, `#!satisfies`), tratamento transacional de erros. |
| [`project-structure.md`](project-structure.md) | **Organização de Projetos** | Padrões estruturais completos para **Aplicações** (CLI, Backend/API, GUI) e **Bibliotecas / Módulos / Frameworks** (`aipo.toml`, exports, testes). |
| [`cheatsheet.md`](cheatsheet.md) | **Cheatsheet para Injeção de Contexto** | Versão ultra-densa (< 120 linhas) ideal para injeção em system prompts ou regras de projeto (`.cursorrules`, `CLAUDE.md`, Custom GPTs). |

---

## 3. Skill e Ferramentas para Agentes

Além da documentação estática, o repositório fornece a skill oficial `lang-aipo`:

- **Definição da Skill:** `skills/lang-aipo/SKILL.md` (e registrada em `~/.claude/skills/lang-aipo/`).
- **Exemplos Canônicos:** `skills/lang-aipo/examples/` (código testado e parseável cobrindo apps, libs, async e transações).
- **Base de Conhecimento:** `skills/lang-aipo/knowledge/` (inventário da stdlib e matriz sintática).
- **Script de Validação:** `skills/lang-aipo/scripts/lint_aipo.py` (linter léxico para checagem rápida de tokens proibidos gerados por LLMs).
- **Adaptadores de Plataforma:** `skills/lang-aipo/adapters/` (instruções prontas para Claude, GPT, OpenCode e providers genéricos).

---

## 4. Ordem de Consulta da LLM

Quando uma LLM for encarregada de ler, gerar ou refatorar código Aipo:
1. Carregar [`cheatsheet.md`](cheatsheet.md) ou ativar a skill `lang-aipo`.
2. Verificar [`rules.md`](rules.md) para checagem de sanitização contra palavras proibidas.
3. Seguir as convenções de modularidade em [`project-structure.md`](project-structure.md).
4. Em caso de dúvida sobre decisões fundamentais, consultar a referência canônica em [`../../SYNTAX.md`](../../SYNTAX.md) e [`../decisions/adr-001-canonical-syntax.md`](../decisions/adr-001-canonical-syntax.md).
