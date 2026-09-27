# aipo.game — Módulos do Pacote

Este diretório contém a implementação pura em Aipo dos módulos que estruturam o framework `aipo.game`:

- **[`lib.aipo`](./lib.aipo)**: Ponto de entrada do pacote e re-exportação unificada da API pública (`actor`, `scene`, `behaviors`, `input`, `audio`, `spawn`, `nodes`).
- **[`ffi.aipo`](./ffi.aipo)**: Contrato de ponte FFI de baixo nível (`__aipo_game_*`) para comunicação desacoplada com o runtime host (Desktop via Rust/Miniquad/Raylib ou Navegador via Wasm).
- **[`input.aipo`](./input.aipo)**: Gerenciador de entradas com mapeamento amigável de teclado, botões de mouse e eixos de gamepads.
- **[`audio.aipo`](./audio.aipo)**: Subsistema de efeitos sonoros, música de fundo em loop e controle de volume/pitch.
- **[`behaviors.aipo`](./behaviors.aipo)**: Catálogo de comportamentos pré-fabricados reutilizáveis (`TopDown`, `Platformer`, `Bullet`, `Solid`, `WrapScreen`, `DestroyOutsideScreen`).
- **[`actor.aipo`](./actor.aipo)**: Definição da estrutura e ciclo de vida de entidades (*Actors*), transformações espaciais 2D, caixas delimitadoras de colisão e despacho de eventos.
- **[`scene.aipo`](./scene.aipo)**: Gerenciador de cenas, registro de instâncias de atores, controle de câmera 2D, timers e laço principal de jogo (`start`).
- **[`nodes.aipo`](./nodes.aipo)**: Especificação de dados e compilador do Sistema de Nós Visuais (arquitetura *Trigger-Filter-Action*) com serialização direta para código canônico `.aipo`.
- **[`sfx.aipo`](./sfx.aipo)**: Sintetizador procedural de efeitos sonoros chiptune (estilo SFXR/BFXR) para prototipação instantânea com zero arquivos externos de áudio.
- **[`tween.aipo`](./tween.aipo)**: Motor de interpolação e animações elásticas (*Game Juice*) com suporte a curvas de aceleração (`linear`, `ease_in`, `ease_out`, `bounce_out`).
