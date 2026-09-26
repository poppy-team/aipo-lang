# Wave 5 — Gestor de Pacotes Hermético & Offline

A **Wave 5** desenvolveu o sistema de empacotamento, resolução de dependências e distribuição da linguagem Aipo (P04-G01 a P04-G11), com foco em **hermeticidade, determinismo e segurança de supply-chain**.

---

## Marcos Conquistados

### 1. Fundação do Sistema de Pacotes (P04-G01)
- Identidade formal baseada em coordenadas `namespace.package`.
- Manifesto canônico `aipo.toml` e geração de lockfile determinístico `aipo.lock`.
- Resolução de dependências locais por caminho (`path`), imports qualificados e rastreamento de proveniência.

### 2. Dependências Remotas Pinadas no GitHub (P04-G04 a P04-G06)
- Suporte a pacotes hospedados no GitHub, exigindo obrigatoriamente um **commit SHA completo pinado** no manifesto:
  ```toml
  [dependencies]
  helper = { github = "organizacao/repo", commit = "4b24a71c08" }
  ```
- Resolução recursiva do grafo de dependências com limite estrito de segurança de 256 pacotes.
- Download atômico através do subcomando explícito `aipo package fetch-github`, suportando autenticação opt-in com token bearer sem nunca expor nem logar segredos.

### 3. Consumo Offline & Integridade Criptográfica (P04-G07 a P04-G10)
- Armazenamento em cache local estruturado (`.aipo/cache`) verificado por hashes SHA-256 independentes.
- Execução regular (`run`, `check`, `build`) em modo estritamente **offline**:
  - O runtime nunca faz chamadas de rede durante a execução comum.
  - Se um pacote estiver ausente ou modificado no cache, a execução falha imediatamente (*fail-closed*).
- Ferramental de auditoria e manutenção de cache:
  - `aipo package cache verify`: Varre e valida a assinatura criptográfica de todos os pacotes em cache.
  - `aipo package cache prune`: Poda com segurança pacotes antigos não mais referenciados no lockfile.
