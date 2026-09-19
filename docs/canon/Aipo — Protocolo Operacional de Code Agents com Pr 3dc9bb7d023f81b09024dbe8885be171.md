# Aipo — Protocolo Operacional de Code Agents com Prumo CLI

<aside>
🤖

**Regra canônica:** code agents estruturam e governam o trabalho através do Prumo CLI. Prompts soltos não substituem Goal, Plan, Contracts, Evidence e Gates.

</aside>

## Bootstrap

O agent começa verificando a versão e superfície real instalada:

```bash
prumo version
prumo --help
prumo <command> --help
```

Se o projeto ainda não estiver inicializado/adotado, utilizar `prumo init` com profile aprovado, seguido de `prumo validate` e `prumo doctor`. Nunca assumir flags/comandos futuros; confirmar help da versão instalada.

## Machine mode

Quando suportado, usar `--json`. Stdout JSON é protocolo; stderr contém diagnostics. Parsing ad-hoc de output humano é fallback, não baseline.

## Antes de implementar

1. Resolver Goal/task atual e estado.
2. Rodar audit/readiness documental pertinente.
3. Avaliar impact para paths/contratos tocados.
4. Carregar micro-contexto mínimo.
5. Confirmar ADP/contract autoritativo.
6. Definir acceptance criteria vertical e evidence exigida.
7. Só então editar código.

Comandos documentados no Prumo atual incluem:

```bash
prumo docs contracts
prumo docs contracts show <contract>
prumo docs profiles
prumo docs audit --json
prumo docs readiness --goal <goal> --json
prumo docs impact --path <project> <changed-path> --json
prumo docs delta --goal <goal> --json
prumo docs delta propose --goal <goal> --path <project> <changed-path> --json
prumo docs delta list --path <project> --json
prumo docs contradictions --json
```

Usar somente comandos realmente disponíveis na versão instalada.

## Goals e locks

Goals formalizam objetivo e scope. Goal locked não é alterado por edição direta; mudanças passam pelo mecanismo de amendment do Prumo. O agent preserva locks e registra mudança de objetivo, não ajusta silenciosamente documento para encaixar implementação.

## Plan DAG

Tasks declaram dependências. Paralelismo de agents só é permitido quando o DAG provar independência suficiente de invariants, arquivos e boundaries. “Há várias coisas para fazer” não é critério de paralelismo.

## Micro-context

Context package deve conter Goal/task, arquivos esperados, contracts/ADPs relevantes, invariants, acceptance criteria, checks, dependencies, risks/open questions e fora de escopo. O agent expande contexto apenas por dependência concreta.

## Handoff

Ao finalizar ou trocar de agent, produzir handoff estruturado: Goal/task, estado, commits/diffs, decisões, evidence, tests executados, failures restantes, docs delta, open questions, exact next action e contexto mínimo para continuação.

## Evidence

Afirmações como implementado, seguro, compatível, rápido ou 10/10 exigem evidence. Exemplos: tests, conformance snapshot, benchmark, Miri/fuzz report, diagnostic fixture, source-map test, security audit, package audit ou output de ferramenta.

## Scope discipline

Agents recebem o menor scope necessário. Mudança fora do scope exige impact review. Não refatorar módulos adjacentes “enquanto está aqui” sem relação necessária com Goal/task.

## No invention policy

Se documentação canônica não define comportamento material: não escolher silenciosamente; registrar open question/ADP proposal; criar blocked/failing acceptance case quando útil; implementar apenas partes não ambíguas.

## Drift control

Após mudanças, rodar audit/impact/delta/contradictions conforme aplicável. Contradição não é resolvida automaticamente por recência. Documentação canônica e locks governam resolução.

## Compile adapters

Quando precisar gerar instructions/adapters para harness específico, usar `prumo compile --target <target> --path <project>` para targets suportados. Generated adapters são derivados e não viram fonte canônica manual.

## Encerramento

Antes de marcar task/Goal complete: anexar evidence/gates, atualizar docs, executar checks determinísticos, validar projeto com Prumo e deixar handoff reproduzível. O próximo agent deve retomar sem depender do histórico de chat.