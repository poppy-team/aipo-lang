# Começando com Aipo

Bem-vindo à documentação oficial do **Aipo**, uma linguagem de programação dinâmica, fortemente tipada e com suporte a contratos opcionais, construída do zero em Rust.

---

## Roteiro de Primeiros Passos

Se você está tendo o primeiro contato com a linguagem, recomendamos seguir a ordem abaixo:

1. **[O que é Aipo?](/getting-started/what-is-aipo)**  
   Entenda a filosofia, os princípios de design, por que evitamos coerções implícitas e onde o Aipo se posiciona em relação a linguagens como Lua, Python, Wren e TypeScript.

2. **[Instalação & Setup](/getting-started/installation)**  
   Como compilar o utilitário `aipo` a partir do código-fonte em Rust ou utilizar binários pré-compilados no Linux, macOS e Windows.

3. **[Seu Primeiro Programa em 5 Minutos](/getting-started/first-program)**  
   Um guia prático com exemplos de sintaxe, criação de estruturas, declaração de invariantes e execução imediata no interpretador e na VM.

4. **[Visão Geral do CLI (`aipo`)](/getting-started/cli-overview)**  
   Conheça os subcomandos do utilitário de linha de comando: `run`, `check`, `build`, `disasm`, `fmt` e `package`.

---

## Por que Aipo?

- **Zero Coerção Implícita**: Evita erros sutis de tipos comuns em JavaScript e Python (`"10" + 2` é um erro de tipo, nunca `"102"` ou `12`).
- **Contratos & Invariantes**: Declare regras semânticas diretamente nas estruturas (`invariant()`) com garantia de rollback automático caso uma mutação quebre o contrato em blocos `attempt`.
- **Concorrência Cooperativa Determinística**: Suporte de primeira classe para `async fn` e combinadores de tarefas (`task.*`), com scheduler virtual sem efeitos colaterais temporais não-determinísticos.
- **Ecossistema Hermético**: Dependências pinadas por commit SHA com integridade criptográfica SHA-256 e reprodução 100% offline.
