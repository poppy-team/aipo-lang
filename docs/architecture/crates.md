# Contratos das Crates do Workspace

O repositório do Aipo é composto por **crates modulares em Rust**, cada uma com limites estritos de responsabilidade e dependências lineares sem ciclos.

---

## Inventário de Crates

| Crate | Responsabilidade Primária | Dependências Internas |
| :--- | :--- | :--- |
| **`aipo-source`** | Carregamento de fontes e indexação segura UTF-8 | Nenhuma |
| **`aipo-diagnostics`** | Catálogo estável de erros e spans de código | `aipo-source` |
| **`aipo-lexer`** | Tokenização de alta velocidade sem alocação | `aipo-source`, `aipo-diagnostics` |
| **`aipo-ast`** | Tipos da árvore sintática abstrata | `aipo-source`, `aipo-diagnostics` |
| **`aipo-syntax`** | Parser recursivo com recuperação de erros | `aipo-lexer`, `aipo-ast`, `aipo-diagnostics` |
| **`aipo-hir`** | Representação intermediária de alto nível | `aipo-ast`, `aipo-diagnostics` |
| **`aipo-sema`** | Análise semântica e contratos estáticos | `aipo-hir`, `aipo-diagnostics` |
| **`aipo-ir`** | Core Intermediate Representation linearizada | `aipo-hir`, `aipo-diagnostics` |
| **`aipo-bytecode`** | Emissão e serialização `.aibc` | `aipo-ir`, `aipo-diagnostics` |
| **`aipo-vm`** | Máquina virtual de execução e scheduler async | `aipo-bytecode`, `aipo-diagnostics` |
| **`aipo-js`** | Emissor JavaScript ES2022 e Source Maps | `aipo-hir`, `aipo-diagnostics` |
| **`aipo-host`** | Host ABI, capabilities e handles geracionais | `aipo-diagnostics` |
| **`aipo-runtime`** | Registro e orquestração de módulos nativos | `aipo-vm`, `aipo-host` |
| **`aipo-stdlib`** | Implementação canônica da biblioteca padrão | `aipo-runtime`, `aipo-vm` |
| **`aipo-poppy`** | Adaptador ECS e simulação determinística | `aipo-host`, `aipo-vm` |
| **`aipo-formatter`** | Formatador automático de sintaxe canônica | `aipo-syntax`, `aipo-ast` |
| **`aipo-cli`** | Ponto de entrada de linha de comando (`aipo`) | Todas as crates acima |
