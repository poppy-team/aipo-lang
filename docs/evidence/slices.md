# Registro Histórico de Slices

Cada fatia vertical (*vertical slice*) e meta de implementação no Aipo é concluída somente após a validação de evidências automatizadas e o registro de um relatório formal em `docs/evidence/`.

---

## Histórico Consolidado de Slices

### Fase P00: MVP Foundation & Fechamento de Contratos
- **P00-G01 a P00-G08**: Workspace, Lexer, Parser, HIR, SEMA, Core IR, Bytecode, VM e Modelo de Erros.
- **P00-G09**: Crate `aipo-runtime` e primeira biblioteca padrão (*MVP subset*).
- **P00-G10**: Fechamento do pipeline ponta a ponta (Syntax $\rightarrow$ IR $\rightarrow$ Bytecode $\rightarrow$ VM $\rightarrow$ CLI).
- **P00-G11**: Utilitário CLI e formatador de código canônico.
- **P00-G12**: Portão de qualidade e conformidade do MVP.
- **P00-G13**: Hooks de construção `init()` e tipo `Bytes`.
- **P00-G14**: Imposição de invariantes em mutações estáveis e rollback transacional via journal em `attempt`.
- **P00-G15**: Contratos estáticos em `interface`, conformidade estrutural em chamadas dinâmicas e normalização NFC.
- **P00-G16**: Revisão final de encerramento da Wave 1 com gauntlet 100% verde (13/13, 11/11, 3/3, 146/146).

### Fase P01: Paridade JavaScript & Deep Quality Gauntlet
- **P01-G01**: Emissor `aipo-js`, runtime shim versionado, Source Maps e 100% de paridade diferencial VM ↔ Node.js.
- **P01-G02**: Suíte profunda de fuzzing (libFuzzer), testes de propriedade (proptest) e auditoria de supply chain (`cargo deny`).

### Fase P02: Wave 3 — Tipos Ricos & Async
- **P02-G01**: `Set` com ordem de inserção, `Sequence` lazy e serialização LE/BE em `Bytes`.
- **P02-G02**: Combinadores assíncronos (`task.*`) e scheduler cooperativo com tempo virtual determinístico.
- **P02-G03**: Sintaxe `async fn`, bloco `await do ... end`, diagnósticos semânticos e detecção de ciclos de espera.

### Fase P03: Wave 4 — Host ABI & Poppy Engine
- **P03-G01**: Crate `aipo-host`, Host Schema (AHS), permissões deny-by-default, handles geracionais e prevenção de escape de escopo.
- **P03-G02**: Crate `aipo-poppy`, módulo `poppy`, buffer de comandos e simulação headless determinística com semente.

### Fase P04: Wave 5 — Gestor de Pacotes Hermético
- **P04-G01 a P04-G11**: Sistema de pacotes, manifesto `aipo.toml`, lockfiles determinísticos, dependências GitHub pinadas por commit SHA, cache local atômico com verificação SHA-256 e ferramentas de auditoria (`verify`) e poda (`prune`).
