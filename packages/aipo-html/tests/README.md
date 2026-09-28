# aipo.html — Testes Automatizados

Este diretório contém a suíte de testes de integração e conformidade para o pacote `aipo.html`:

- **[`html_test.aipo`](./html_test.aipo)**: Cobertura completa de:
  - Construção hierárquica de nós HTML5 com trailing do-blocks
  - Elementos void/folha e atributos booleanos/escapados
  - Serialização SSR com `render_to_string` e mitigação XSS via entity escaping
  - Renderização de fragmentos virtuais (`fragment`) sem containers intermediários
  - CSS-in-Aipo com escopo gerado por hash, pseudo-classes (`:hover`) e `@media` queries
  - Ciclo de vida TEA/MVU com despacho reativo e execução de comandos encadeados (`Cmd`)
