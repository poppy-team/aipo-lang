# aipo-game-host

Host nativo da engine de jogos **Aipo** construído em Rust sobre o ecossistema **Miniquad / Macroquad**.

Fornece:
- Criação de janelas aceleradas por hardware (OpenGL / Vulkan / Metal).
- Loop de jogo em 60 FPS com processamento e renderização de lotes 2D.
- Entrada em tempo real de teclado, mouse e gamepads sem bloqueio.
- Áudio dinâmico e sintetizador procedural chiptune conectado à placa de som.
- Execução direta de arquivos `.aipo` com o pacote `aipo.game`.
