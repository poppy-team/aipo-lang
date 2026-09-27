# aipo.html — Código-Fonte do Pacote

Este diretório contém os módulos Aipo que compõem o pacote oficial `aipo.html`:

- **[`lib.aipo`](./lib.aipo)**: Ponto de entrada do pacote e re-exportação da API pública (`tags`, `css`, `mvu`).
- **[`tags.aipo`](./tags.aipo)**: Construtores declarativos de tags HTML5 (`div`, `h1`, `p`, `button`, etc.) e gerenciamento da pilha de nós filhos (*NodeCollector*).
- **[`css.aipo`](./css.aipo)**: DSL para CSS-in-Aipo com hashing determinístico de regras de estilo, suporte a pseudo-classes e injeção automática no `<head>`.
- **[`mvu.aipo`](./mvu.aipo)**: Executor da arquitetura MVU (*Model-View-Update*) com granularidade fina, despacho assíncrono de mensagens e vinculação cirúrgica com o DOM.
