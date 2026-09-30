# ADP-003 — Orçamentos de Execução para Programas Não Confiáveis (fuel, memória, interrupção)

**Status:** rascunho (questão em aberto — nenhuma semântica implementada, nada decidido)
**Relacionado:** `docs/evidence/P01-G02-*.md` (suíte de exaustão de recursos), contratos das crates
(linha Owns de `aipo-vm` corrigida pelo mesmo goal), Fechamento Arquitetural §10–11
**Autoridade:** subordinada a `docs/canon/Aipo V1 — Language Reference…` e
`docs/language/authority-map.md` (política de não invenção)

## Fatos verificados (não são decisões)

- Hoje a VM impõe exatamente um orçamento de execução: o limite de profundidade da
  operand stack (1024), que se manifesta como `AIPO_RT_OVERFLOW` com uma dica de recursão.
- **Não** existe orçamento de instruções/fuel, nem contabilização de alocação/memória, nem
  mecanismo de interrupção dentro da VM. Um programa `loop … end` executa até que o host o
  mate; a suíte de testes prova isso com um watchdog externo, não com uma garantia da VM.
- O contrato das crates listava historicamente "contabilização de fuel/débito no nível da VM"
  em Owns de `aipo-vm`. Essa linha descrevia uma aspiração, não a implementação, e foi
  corrigida para o limite de profundidade da stack, com um ponteiro para este documento.
- A execução de `cargo-fuzz`/libFuzzer, o Miri e as execuções com sanitizers são avaliações
  de nível CI, não gates atuais (veja o registro de evidência do gauntlet).

## Questões em aberto (todas indecididas)

1. **Sinal de exaustão:** quando um orçamento estoura, o resultado é uma `Failure`
   recuperável, um fault de runtime com um novo código de diagnóstico, ou um abort do
   processo? Cada escolha altera o contrato de Failure/fault e exige respaldo na Language
   Reference que ainda não existe.
2. **Escopo do orçamento:** por chamada, por execução de módulo, por sessão do host? Quem o
   define — sintaxe do código-fonte, flags da CLI ou apenas a API do host?
3. **Contabilização de memória:** o que conta (coletas, closures, strings, bytecode), e a
   contabilização é exata ou amostrada?
4. **Interrupção:** cooperativa (verificada entre instruções) ou preemptiva? Que estado é
   observável depois de uma interrupção?
5. **Backend JS:** qualquer orçamento precisa de uma regra de paridade VM↔JS definida; o shim
   atualmente também não tem orçamento.

## Não-objetivos deste ADP

- Inventar semântica de fuel dentro de um goal de teste/qualidade. Orçamentos alteram o
  comportamento observável e precisam de um goal de design próprio, com patrocínio do canon.
- Afirmar sandboxing: a saída do Aipo "não é, por si só, um sandbox de segurança"
  (Fechamento §10). Scripts não confiáveis ainda exigem sandboxing de VM/host ou isolamento
  externo.

## Critérios de saída

Este ADP se encerra quando um goal de design especificar o sinal, o escopo, a contabilização
e a regra de paridade acima, com fixtures que provem cada um — ou quando tirar explicitamente
os orçamentos da V1, com uma justificativa documentada.
