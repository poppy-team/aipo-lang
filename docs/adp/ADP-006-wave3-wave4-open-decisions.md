# ADP-006 — Decisões em Aberto das Waves 3/4 (async, coleções, packing, host)

**Status:** aceito (as decisões abaixo estão implementadas; os follow-ups estão anotados)
**Relacionado:** Fechamento §6–§8, canon da Stdlib (Task/Sequence/Time), Poppy Pivot
**Autoridade:** subordinada ao canon; todo item cita sua fonte no canon ou é
explicitamente marcado como a menor escolha consistente.

## A. Construção de `Set`

O canon mostra `Set` como um tipo com `.lazy()`, mas não define literal nem forma de
chamada, e a V1 estabeleceu que `List`/`Dict`/`Bytes` não têm forma de chamada. Decisão:
chamada de conversão `Set(values: List) -> Set` (remove duplicatas, ordem da primeira
ocorrência, ordenado por inserção conforme o canon). Justificativa: espelha o padrão de
chamada de conversão de `Bytes(count)`/`Int(x)`; nenhuma sintaxe de literal nova é inventada.

## B. Fontes e vocabulário de `Sequence`

Diagrama do canon: `List / Dict / Set → .lazy()`. Decisão: `.lazy()` existe em
List (elementos), Dict (valores) e Set (ordem de inserção). Fontes Range/String
**não** são adicionadas (não especificadas). Vocabulário implementado: `map`, `filter`,
`flat_map`, `find`, `any`, `all`, `count`, `reduce` (valor inicial obrigatório),
`take`, `skip`, `group_by` (→ Dict), `distinct`, `zip` (→ List de pares),
`chain`, `chunk` (→ List de Lists), `window` (→ List de Lists), `enumerate`
(→ List de `[index, value]`), `collect` (→ List). A laziness é uma cadeia de thunks;
`take` faz short-circuit, de modo que `take` sobre fontes grandes permanece limitado.

## C. Packing de Bytes

O canon nomeia apenas formatos de armazenamento (`i8…f64`), sem forma de API. Decisão:
métodos com receiver primeiro `read_i8/u8/i16/u16/i32/u32/i64/u64/f32/f64(index)` e
`write_*(index, value)`, **little-endian** (convenção de jogos/binário; consistente
na VM e no JS por construção), mais `String.encode()` (UTF-8 → Bytes) e
`Bytes.decode()` (UTF-8 → String, Failure se inválido). Violações de índice exato
são faults (consistente com a indexação exata); escritas fora do intervalo nunca
expandem o bloco silenciosamente.

## D. `Duration` e tipos de relógio adiados

O canon exige `Duration` mais `Date`/`TimeOfDay`/`DateTime` com parsing ISO,
mas o banco de dados de fusos horários IANA pertence a um pacote posterior. Decisão:
conversão `Duration(seconds:
Int|Float)` na Wave 3 com `+`, `-`, comparação e
`total_seconds()`; os tipos de calendário puros `Date`, `TimeOfDay` e `DateTime` são
implementados sem identidade de fuso horário. O provider de `timezone` baseado em IANA
continua adiado. `task.sleep(seconds)` aceita
segundos Int/Float diretamente; sleep negativo é uma Failure recuperável.

## E. Combinadores de Task

O canon lista `sleep`, `all`, `race`, `timeout`, `cancel`, `group`, `spawn`
sem assinaturas. Decisões: `task.spawn(fn)` e `task.spawn(fn, args:
List)`; `task.all(tasks: List) -> List` (fail-fast, cancela o restante, a primeira
Failure vence); `task.race(tasks: List)` (a primeira conclusão na ordem determinística
do scheduler vence, as perdedoras são canceladas; lista vazia → Failure); `task.timeout
(task, ticks: Int)` (valor ou `Failure("timeout")`, task cancelada na expiração);
`task.cancel(task)` (marca como cancelada, efetivo nos pontos de suspensão);
`task.group()` → Group com `group.spawn(task)` / `group.wait()` (semântica de
all; as restantes são canceladas após o fail-fast). `sleep(0)` é um ponto de
reagendamento. Os ticks são tempo virtual u64 — totalmente determinístico, sem relógio de parede.

## F. Diagnósticos de await (novos códigos)

O canon determina as restrições; a escolha dos códigos é detalhe de implementação:
`AIPO_SEM_AWAIT_IN_SUBEXPRESSION` (`await` explícito fora da posição de statement /
inicializador / return), `AIPO_SEM_FORGOTTEN_TASK` (valor sabidamente Task
descartado sem await/combinador de group), `AIPO_SEM_NESTED_AWAIT_DO`
(`await do` aninhado redundante), `AIPO_SEM_PARAMETRIC_CONTRACT` (sintaxe `Task[T]`
enquanto não existirem contratos paramétricos), `AIPO_RT_CANCELLED` (fault,
nunca capturável), `AIPO_RT_AWAIT_CYCLE` (fault: task aguardando a si mesma,
direta ou transitivamente).

## G. Contratos paramétricos `Task[T]` adiados

O canon chama `Task[T]` de contrato built-in, mas a linguagem não tem generics
(backlog: generics de usuário fora da V1). Decisão: `Task` puro corresponde a qualquer valor
Task em runtime; `Task[X]` em posição de contrato é um diagnóstico dedicado
que aponta para cá (`AIPO_SEM_PARAMETRIC_CONTRACT`). Aguardar um literal comprovadamente
não Task é uma violação estática de contrato (`AIPO_SEM_CONTRACT_VIOLATION_STATIC`,
certificada por `docs/conformance/diagnostics/29_sem_await_literal.aipo`);
aguardar um não Task em runtime é um fault de contrato
(`AIPO_RT_TYPE_MISMATCH`).

## H. Riscos de race/deadlock eliminados por construção

Não existe paralelismo (scheduler single-threaded, run queue FIFO, condução de await
em profundidade, salto de tick até o wakeup mínimo). `race` é determinístico quanto à ordem
por design. Ciclos de await geram fault em vez de travar. O cancelamento é verificado
apenas nos pontos de suspensão, de modo que nenhuma race de inspecionar-e-matar é possível.

## I. Desvios da Wave 4 registrados

- Sem dependência de `bevy`: o canon proíbe *expor* internals de bevy/Ecs, e uma
  dependência de mais de 100 crates para um adapter de demonstração violaria a regra
  de superfície mínima; `aipo-poppy` é autocontido e prova a ABI em vez disso.
- **Códigos de fault do host para restrições que o canon determina mas não nomeia.** O canon
  fixa a proibição (nenhum use-after-free em um handle obsoleto, nenhum binding com escopo
  escapando de seu callback) e nomeia apenas `AIPO_RT_CAPABILITY_DENIED`; os *nomes* dos
  códigos são detalhe de implementação, a mesma latitude que a §F registrou para os
  diagnósticos de await. Escolhidos: `AIPO_RT_STALE_HANDLE` e `AIPO_RT_SCOPE_ESCAPE`,
  junto de `AIPO_RT_CAPABILITY_DENIED`. Um valor do host que não consegue satisfazer seu
  contrato declarado é `AIPO_RT_TYPE_MISMATCH`, porque o canon classifica uma violação de
  contrato descoberta em uma fronteira como type mismatch.
- **Os caminhos de capability são a árvore.** A §11 do canon lista uma hierarquia plana que
  inclui a família `poppy.*`, enquanto o relógio é tratado como a capability única `clock`
  no ponto de chamada (`time.now`/`time.monotonic`). Ambos são a mesma regra a partir do
  momento em que um caminho concede seus descendentes: `clock` cobre `clock.wall` e
  `clock.monotonic`, e `poppy` cobre todo `poppy.<name>`. Nenhuma sintaxe de wildcard é
  introduzida, e um profile que concede apenas `clock.wall` ainda não consegue ler o relógio
  monotônico.
- A imposição de escape com escopo cobre SetGlobal, Return, SetField, SetIndex,
  BuildList e BuildDict (todo ponto de publicação no heap da VM).
- `time.now`/`time.monotonic` exigem a capability `clock` (canon: relógios são
  capabilities); a negação é um fault `AIPO_RT_CAPABILITY_DENIED`. O profile de script
  padrão da CLI concede `clock`; os testes constroem hosts negados diretamente.
