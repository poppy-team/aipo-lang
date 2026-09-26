# Manual da Linguagem Aipo

Este manual documenta formalmente a sintaxe, os tipos de dados, o sistema de módulos, os contratos de integridade, a concorrência assíncrona e a biblioteca padrão da linguagem Aipo.

---

## Seções do Manual

- **[Sintaxe & Tipos](/manual/syntax-and-types)**: Tipos primitivos, literais, coleções dinâmicas (`List`, `Dict`, `Set`, `Sequence`), manipulação de bytes e operadores.
- **[Controle de Fluxo & Falhas](/manual/control-flow)**: Estruturas condicionais, laços determinísticos, manipulação de erros com `fail` e o bloco transacional `attempt ... recover`.
- **[Funções, Closures & Lambdas](/manual/functions-and-closures)**: Declaração de funções, parâmetros opcionais com valor padrão, funções anônimas, sintaxe concisa `=>` e escopo léxico de variáveis capturadas.
- **[Interfaces & Contratos](/manual/interfaces-and-contracts)**: Estruturas, campos imutáveis `fixed`, interfaces com `satisfy`, hooks de construção `init`, invariantes de integridade `invariant()` e contratos de assinatura.
- **[Concorrência & Async](/manual/async-and-concurrency)**: Funções assíncronas `async fn`, sintaxe sequencial `await do ... end`, combinadores `task.spawn`, `task.sleep`, `task.all`, `task.race` e o scheduler de tempo virtual.
- **[Biblioteca Padrão (Stdlib)](/manual/stdlib)**: Módulos utilitários essenciais (`math`, `random`, `json`, `encoding`, `path`, `url`, `regex`, `binary`, `time`, `testing`, `log`, `env`, `fs`).
- **[Pacotes & Módulos](/manual/packages-and-modules)**: Organização de projetos, manifesto `aipo.toml`, lockfiles reproduzíveis, consumo offline e dependências seguras do GitHub.

---

## Princípio Fundamental: Explicitude e Robustez

O Aipo adota o princípio de que código de missão crítica deve ser legível por humanos e auditável por agentes inteligentes sem ambiguidades ocultas. Todo operador tem comportamento estrito e todo erro em potencial é canalizado para o modelo de falhas estruturado da linguagem.
