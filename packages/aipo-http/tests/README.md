# aipo.http — Testes Automatizados

Este diretório contém a suíte de testes de integração e conformidade para o pacote `aipo.http`:

- **[`http_test.aipo`](./http_test.aipo)**: Cobertura completa de:
  - Roteamento básico e respostas tipadas de contexto (`text`, `json`, `html`)
  - Resolução de parâmetros dinâmicos simples (`:id`) e múltiplos aninhados
  - Captura de rotas coringa / catch-all (`*filepath`)
  - Extração de query string (`?q=...&page=...`) e parsing de corpo JSON
  - Pipeline de middlewares estilo cebola com ordem antes/depois e curto-circuito (autenticação)
  - Middlewares embutidos: CORS com requisições preflight OPTIONS (204) e Recover com captura de erros de runtime (500)
  - Agrupamento de rotas com prefixos aninhados (`/api/v1`) e rotas não encontradas (404)
