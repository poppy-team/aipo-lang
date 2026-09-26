# Decisões Arquiteturais (ADPs)

Na engenharia do Aipo, seguimos uma rigorosa **política contra invenções não documentadas** (*no-invention policy*): nenhuma decisão técnica arbitrária é tomada silenciosamente no código.

Toda questão aberta ou mudança estrutural é formalizada como uma **Proposta de Decisão Arquitetural** (*Architectural Decision Proposal* - ADP), contendo contexto, alternativas avaliadas, justificativa, impacto na documentação e critérios de aceitação.

---

## Registro de Decisões Aprovadas

| ADP | Título | Status | Resumo da Decisão |
| :--- | :--- | :--- | :--- |
| **[ADP-001](/decisions/adp-001)** | Tipos Core e Bytes como Valores | Aprovado | Trata `Bytes` como tipo de primeira classe na VM; fatiamento tolerante e limites de `clamp`. |
| **[ADP-002](/decisions/adp-002)** | Hooks de Construção e Contratos em Runtime | Aprovado | Formalização dos hooks `init()` e `invariant()` com rollback em blocos `attempt`. |
| **[ADP-003](/decisions/adp-003)** | Orçamentos de Execução | Aprovado | Mecanismo de contagem de passos de instrução para sandboxing de scripts não confiáveis. |
| **[ADP-004](/decisions/adp-004)** | Política de Identificadores Unicode | Aprovado | Normalização Unicode NFC obrigatória em identificadores e literais de string. |
| **[ADP-005](/decisions/adp-005)** | Limites de Recursão do Parser | Aprovado | Limite estrito de profundidade recursiva no parser para mitigar ataques de negação de serviço (DoS). |
| **[ADP-006](/decisions/adp-006)** | Decisões Abertas das Waves 3 e 4 | Aprovado | Resolução de semântica assíncrona, combinadores de tarefas e escopo de capacidades. |
| **[ADP-007](/decisions/adp-007)** | Identidade e Distribuição de Pacotes | Aprovado | Coordenadas `namespace.package`, dependências GitHub pinadas por commit SHA e cache offline. |
| **[ADP-008](/decisions/adp-008)** | Delimitação da Release v0.1.0 da Linguagem | Aprovado | Foco language-first com escopo fechado; posterga engines gráficas e registro web para pós-v1. |
| **[ADP-009](/decisions/adp-009)** | C ABI Síncrona Versionada | Aprovado | Interface binária C estável com tipos opacos para embutir o Aipo em C/C++/Zig. |
| **[ADP-010](/decisions/adp-010)** | Provas Finas de Interoperabilidade | Aprovado | Três provas finas (*thin proofs*) comprovando embedding em Rust, C e JavaScript. |
| **[ADP-011](/decisions/adp-011)** | Roteiro de Performance & Ergonomia | Aprovado | Compactação de `Value`, fusão `InvokeMethod`, laço hoisted, falhas tipadas e pattern matching. |
| **[ADP-012](/decisions/adp-012)** | Modernização Ergonômica de Sintaxe (Gleam/Swift Pivot) | Aprovado | Blocos delimitados por chaves `{ ... }`, imutabilidade por padrão em structs, `var self`, `:` simétrico e interfaces automáticas. |
