# aipo.http — Código-Fonte do Pacote

Este diretório contém os módulos Aipo que compõem o pacote oficial `aipo.http`:

- **[`lib.aipo`](./lib.aipo)**: Ponto de entrada do pacote e exportação da fachada pública (`create`, `router`, `logger`, `cors`, `recover`, etc.).
- **[`app.aipo`](./app.aipo)**: Estrutura central da aplicação (`App`), registro de rotas por verbo HTTP (`get`, `post`, `put`, `delete`, `patch`), agrupamento de rotas (`group`) e dispatcher puro (`handle_request`).
- **[`router.aipo`](./router.aipo)**: Roteador de árvore de segmentos (*Segment/Radix Trie*) sem expressões regulares, suportando rotas estáticas, parâmetros (`:param`) e coringas (`*wildcard`).
- **[`context.aipo`](./context.aipo)**: Estrutura `Context` com métodos utilitários para parsing de query strings, parâmetros de rota, cabeçalhos, corpo de requisição JSON e serializadores de resposta (`json`, `text`, `html`, `status`).
- **[`middleware.aipo`](./middleware.aipo)**: Pipeline sequencial estilo cebola (*onion-style runner*) e implementações prontas para uso (`logger`, `cors`, `recover`).
