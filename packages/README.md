# Pacotes Canônicos e Oficiais da Linguagem Aipo

Este diretório contém os pacotes e frameworks oficiais mantidos pelo ecossistema Aipo.

Conforme a arquitetura da linguagem e o princípio de separação de domínios, **recursos de domínio específico (como manipulação de DOM, bibliotecas gráficas de UI, simulações físicas ou engines de jogos) não inflam a gramática do compilador central**. Todos eles são distribuídos como pacotes canônicos gerenciados pelo comando `aipo package` e configurados via `aipo.toml`.

## Pacotes Disponíveis

- **[`aipo-html`](./aipo-html/README.md)**: DSL declarativa para HTML5, CSS-in-Aipo tipado e reatividade com MVU de Granularidade Fina no navegador.
- **[`aipo-ui`](./aipo-ui/README.md)**: Framework universal e multiplataforma de interfaces declarativas (Desktop GPU via Skia/Freya, WebGL/Canvas e Terminal TUI) com layout Flexbox/Grid alimentado pelo motor Taffy.
- **[`aipo-egui`](./aipo-egui/README.md)**: Adaptador de interfaces em modo imediato (Immediate-Mode GUI / IMGUI) agnóstico alimentado pelo motor egui com extração universal de formas geométricas e segurança por capabilities.
- **[`aipo-freya`](./aipo-freya/README.md)**: Framework de interfaces declarativas modernas inspirado no Freya UI com motor de layout Torin, hooks reativos (`use_state`) e paleta Catppuccin Mocha.
- **[`aipo-game`](./aipo-game/README.md)**: Micro-engine 2D orientada a Atores e Cenas com comportamentos plugáveis em 1 linha, nós visuais anti-espaguete, HUD imediato, tilemaps 2D e simulação determinística.

