# Aipo — Sistema de Documentação de Implementação e Diretivas para Code Agents

<aside>
🧭

**Status: canônico para implementação.** Esta página define como a implementação da Aipo deve ser documentada, planejada, executada, auditada e entregue por humanos e code agents. O objetivo é transformar especificação em contratos executáveis, preservar contexto mínimo e impedir drift entre código, documentação, testes e decisões.

</aside>

## Princípio central

A documentação da Aipo é parte do sistema de engenharia. Não é uma camada posterior ao código. Nenhuma feature relevante está completa se o código, os contratos, os testes, os diagnostics, o tooling e a documentação não concordarem entre si.

A fonte durável de verdade da implementação será o repositório Git estruturado pelo **Prumo CLI**. Notion permanece caderno de arquitetura, decisões e exploração; antes da implementação, decisões canônicas devem ser materializadas no repositório em formatos versionáveis e consumíveis por agentes.

## Uso obrigatório do Prumo CLI

Todo code agent responsável por estruturar, iniciar ou atualizar a documentação de implementação deve usar o **Prumo CLI** como harness documental e de governança. Não criar uma segunda estrutura paralela quando o Prumo já modelar o conceito.

Fluxo baseline:

```bash
prumo --help
prumo init <project> --profile <profile> --non-interactive
prumo validate <project>
prumo doctor <project>
prumo docs contracts
prumo docs profiles
prumo docs audit --json
prumo docs contradictions --json
```

Para trabalho associado a Goal, utilizar as superfícies de Goal/Docs da versão instalada e confirmar a sintaxe com `prumo <command> --help`. Para automação e agentes, preferir `--json` sempre que disponível.

Prumo é responsável por estruturar canonical state, Goals, Plans, Evidence, Gates, documentação, readiness, impact, deltas, contradictions, adapters e contexto progressivo. Generated adapters/caches/indexes são derivados; Markdown/JSON/JSON Schema/Git e demais arquivos canônicos definidos pelo Prumo são a fonte de verdade.

## Hierarquia de autoridade

Quando fontes divergirem, usar esta ordem:

1. Decisão canônica mais recente explicitamente aprovada e materializada no repositório.
2. Language Reference / Syntax / Semantic contracts / ADPs aprovadas.
3. Architecture contracts e crate contracts.
4. Tests/fixtures de conformance aprovados.
5. Implementation docs e code comments.
6. Generated docs/adapters/indexes.
7. Historical docs, experiments e rejected designs.

**Recência isolada nunca resolve contradição.** Contradições devem gerar finding/delta/ADP, não ser silenciosamente “corrigidas” por um agente.

## Famílias documentais obrigatórias

### 1. Product & Language Canon

- visão e filosofia;
- language reference;
- sintaxe canônica;
- semântica normativa;
- stdlib reference;
- target/host profiles;
- non-goals e deferred features.

### 2. Architecture

- system overview;
- crate map;
- dependency DAG;
- Core IR contracts;
- VM/runtime/GC boundaries;
- JS backend;
- host ABI/AHS;
- Poppy profile;
- security model;
- package/tooling architecture.

### 3. ADPs — Aipo Design Proposals/Decisions

Toda alteração semântica, sintática, protocolar ou arquitetural importante deve possuir ADP com: context, problem, goals, non-goals, alternatives, chosen design, semantics, lowering, diagnostics, VM/JS impact, security, performance, migration, tests, acceptance criteria, status e supersession.

Estados mínimos: `draft`, `experimental`, `accepted`, `superseded`, `rejected`.

### 4. Crate Contracts

Cada crate possui documento curto e normativo contendo: responsabilidade única, inputs/outputs, public API surface, invariants, dependencies allowed, dependencies forbidden, ownership of data, error model, threading assumptions, unsafe policy, test strategy, performance constraints e links para ADPs relevantes.

### 5. Feature Packs

Cada feature implementável recebe um pack com:

```
feature.md
semantics.md
grammar.md
examples/
fixtures/pass/
fixtures/fail/
expected/diagnostics/
expected/syntax/
expected/hir/
expected/ir/
expected/bytecode/
conformance.md
implementation-plan.md
```

Nem toda feature precisa de todos os arquivos físicos; o princípio é que todo estágio aplicável tenha contrato e evidência.

### 6. Diagnostics Catalog

Cada diagnostic estável possui código, categoria, trigger, primary span, related spans, message, explanation, fix suggestion, machine-readable fields e fixtures positivas/negativas.

### 7. Aipo Host Schema

AHS é documento/schema machine-readable para modules, types, host values, handles, functions, params, return contracts, mutability, async, docs, capabilities, version/deprecation e trailing-block subject metadata. Poppy e outros hosts emitem AHS; compiler/LSP/code agents consomem.

### 8. Implementation Journal

Registrar decisões de implementação, experimentos, benchmark findings, rejected approaches, debt assumido, incidents e migrations. Journal não substitui ADP; descobertas que alteram contrato devem promover mudança formal.

## Micro-contextos e Progressive Context

Nenhum agente deve receber todo o caderno/repositório por padrão. O Prumo deve gerar/selecionar o menor contexto suficiente para a tarefa.

Todo micro-contexto deve conter:

- Goal e task atuais;
- arquivos permitidos/esperados;
- contratos e ADPs diretamente relevantes;
- invariants que não podem ser quebrados;
- acceptance criteria;
- comandos de validação;
- dependencies upstream/downstream relevantes;
- known risks/open questions;
- definição explícita de “fora de escopo”.

O agente deve pedir/expandir contexto apenas quando uma dependência concreta exigir. Contexto maior não é presumido melhor.

## Fluxo documental por mudança

```
Goal
 → impact/readiness
 → ADP ou contrato existente
 → vertical acceptance criteria
 → implementation plan
 → code + fixtures
 → syntax/HIR/IR/bytecode snapshots
 → VM/JS/host conformance
 → security/performance checks
 → docs delta
 → evidence
 → gate
 → merge
```

Antes de codificar, o agente deve conseguir responder: qual contrato estou implementando, qual comportamento observável muda, quais crates podem mudar, quais crates não podem mudar, como provarei o resultado e quais documentos ficarão stale se eu alterar este ponto.

## Vertical slices

Implementar features verticalmente, evitando “completar lexer inteiro, depois parser inteiro, depois VM inteira” sem comportamento integrado. Um slice típico atravessa:

```
source → lexer → syntax → AST/HIR → sema → IR → bytecode → VM → JS → diagnostics → formatter/LSP → tests/docs
```

Somente estágios aplicáveis são obrigatórios, mas omissões devem ser justificadas.

## Waves

As waves organizam dependências maiores. Cada wave deve possuir objective, prerequisites, included goals, excluded work, exit gates, risks, evidence matrix e rollback/recovery strategy. Uma wave só fecha quando os gates mensuráveis fecharem; não por “parece pronto”.

## Gauntlet Loop

Features e waves relevantes usam Gauntlet Loop configurável pelo Prumo/contexto:

1. Implementar o menor slice completo.
2. Executar checks determinísticos.
3. Avaliar qualidade por dimensões explícitas de 1–10.
4. Registrar evidências e defects concretos.
5. Corrigir defects reais, sem inflar nota artificialmente.
6. Reexecutar testes/conformance/audits.
7. Encerrar apenas quando gates e thresholds documentados forem satisfeitos.

Notas são consequência da evidência. Nunca alterar critérios ou pesos apenas para atingir 10.

## Documentação para humanos e agentes

Todo documento normativo deve ser legível por humanos e parseável por agentes. Preferir headings estáveis, IDs estáveis, tabelas pequenas, schemas explícitos, exemplos positivos e negativos e termos consistentes. Prosa não deve esconder regra operacional essencial.

Cada documento normativo começa com: status, authority, scope, owner/domain, dependencies, update triggers e links para superseded/related decisions.

## Atualização e prevenção de drift

Mudança em grammar, semantics, IR, bytecode, host ABI, stdlib, diagnostics, CLI ou package schema deve disparar atualização dos bindings documentais afetados. Usar `prumo docs impact`, `docs delta`, `docs audit`, `docs readiness` e `docs contradictions` quando disponíveis na versão instalada.

Documentos stale não devem ser silenciosamente ignorados. Generated docs nunca sobrescrevem fonte canônica sem review/delta governado.

## Machine-readable first

Para agents/CI, preferir JSON/JSONL/JSON Schema sobre parsing de texto humano quando existir uma representação estruturada. Stable IDs devem existir para diagnostics, ADPs, Goals, Evidence, Gates, crates/contracts e AHS entities.

## Definition of Done documental

Uma mudança não está concluída enquanto:

- contrato e implementação concordarem;
- fixtures pass/fail existirem quando aplicáveis;
- snapshots/conformance estiverem atualizados;
- diagnostics estiverem documentados;
- docs impact/delta tiver sido tratado;
- evidence estiver anexada ao Goal/Plan/Gate;
- nenhuma contradiction bloqueante permanecer aberta;
- comandos determinísticos de validação estiverem verdes;
- documentação gerada tiver sido regenerada quando necessária.

## Regra final para agentes

O code agent não deve “preencher lacunas” inventando decisão semântica. Quando o contrato estiver insuficiente, deve registrar open question/proposal/ADP e implementar apenas o que puder ser provado pelas fontes canônicas.

[Aipo — Rust Engineering Standard: Segurança, Clean Code e Qualidade](Aipo — Rust Engineering Standard Segurança, Clean  3dc9bb7d023f8184b9f9fc7e922448f6.md)

[Aipo — Arquitetura Modular do Workspace e Dependency Rules](Aipo — Arquitetura Modular do Workspace e Dependen 3dc9bb7d023f81158049ffca594c8b79.md)

[Aipo — Protocolo Operacional de Code Agents com Prumo CLI](Aipo — Protocolo Operacional de Code Agents com Pr 3dc9bb7d023f81b09024dbe8885be171.md)

[Aipo — Waves, Vertical Slices, Gauntlet Loops e Gates de Implementação](Aipo — Waves, Vertical Slices, Gauntlet Loops e Ga 3dc9bb7d023f81f2b405f3e105449ac9.md)