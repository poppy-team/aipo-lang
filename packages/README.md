# Pacotes Canônicos e Oficiais da Linguagem Aipo

Este diretório contém os pacotes e frameworks oficiais mantidos pelo ecossistema Aipo.

Conforme a arquitetura da linguagem e o princípio de separação de domínios, **recursos de domínio específico (como manipulação de DOM, bibliotecas gráficas de UI, simulações físicas ou engines de jogos) não inflam a gramática do compilador central**. Todos eles são distribuídos como pacotes canônicos gerenciados pelo comando `aipo package` e configurados via `aipo.toml`.

## Pacotes Disponíveis

- **[`aipo-html`](./aipo-html/README.md)**: DSL declarativa para HTML5, CSS-in-Aipo tipado e reatividade com MVU de Granularidade Fina no navegador.
- **[`aipo-ui`](./aipo-ui/README.md)**: Framework universal e multiplataforma de interfaces declarativas (Desktop GPU via Skia/Freya, WebGL/Canvas e Terminal TUI) com layout Flexbox/Grid alimentado pelo motor Taffy.
