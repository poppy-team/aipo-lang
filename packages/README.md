# Pacotes Canônicos e Oficiais da Linguagem Aipo

Este diretório contém os pacotes e frameworks oficiais mantidos pelo ecossistema Aipo.

Conforme a arquitetura da linguagem e o princípio de separação de domínios, **recursos de domínio específico (como manipulação de DOM, bibliotecas gráficas de UI, simulações físicas ou engines de jogos) não inflam a gramática do compilador central**. Todos eles são distribuídos como pacotes canônicos gerenciados pelo comando `aipo package` e configurados via `aipo.toml`.

## Pacotes Disponíveis

- **[`aipo-html`](./aipo-html/README.md)**: DSL declarativa para HTML5, CSS-in-Aipo tipado, serializador SSR e reatividade com MVU de Granularidade Fina (*TEA with Commands*).
- **[`aipo-http`](./aipo-http/README.md)**: Roteador HTTP zero-regex em árvore de segmentos, pipeline de middlewares estilo cebola (*onion-style*), context engine tipado e dispatching desacoplado para microsserviços.
- **[`aipo-zoe`](./aipo-zoe/README.md)**: Framework de interfaces declarativas modernas com motor de layout Leona, hooks reativos (`use_state`), widgets prontos e subsistema 3D retro (*Retro 3D Engine*).
- **[`aipo-ui`](./aipo-ui/README.md)**: Framework universal e multiplataforma de interfaces declarativas (Desktop GPU via Skia/Freya, WebGL/Canvas e Terminal TUI) com layout Flexbox/Grid alimentado pelo motor Taffy.
- **[`aipo-egui`](./aipo-egui/README.md)**: Adaptador de interfaces em modo imediato (Immediate-Mode GUI / IMGUI) agnóstico alimentado pelo motor egui com extração universal de formas geométricas e segurança por capabilities.
- **[`aipo-game`](./aipo-game/README.md)**: Micro-engine 2D orientada a Atores e Cenas com comportamentos plugáveis em 1 linha, nós visuais anti-espaguete, HUD imediato, tilemaps 2D e simulação determinística.

