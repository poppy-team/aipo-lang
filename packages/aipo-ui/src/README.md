# aipo.ui — Módulos do Pacote

Este diretório contém a implementação pura em Aipo dos módulos que estruturam o framework `aipo.ui`:

- **[`lib.aipo`](./lib.aipo)**: Ponto de entrada e re-exportação unificada da API do framework.
- **[`types.aipo`](./types.aipo)**: Tipos fundamentais de geometria, alinhamento (`Align`, `Justify`, `Direction`), eventos de ponteiro/teclado e nós de interface (`UINode`).
- **[`color.aipo`](./color.aipo)**: Construtores tipados de cor (`rgb`, `rgba`, `hex`, `hsl`) e paleta canônica acessível.
- **[`layout.aipo`](./layout.aipo)**: Gerenciamento da pilha de nós de interface (*UINodeCollector*) e ponte de serialização de propriedades para o motor de layout Taffy.
- **[`primitives.aipo`](./primitives.aipo)**: Componentes primitivos de layout e controles declarativos (`Box`, `Row`, `Column`, `Stack`, `Text`, `Button`, `TextInput`, `Slider`, `ScrollArea`, `Spacer`).
- **[`renderer.aipo`](./renderer.aipo)**: Interface abstrata de renderização e lote de comandos gráficos emitidos após o cálculo de layout.
- **[`app.aipo`](./app.aipo)**: Ciclo de vida da aplicação, laço de eventos reativo e rotinas de montagem multiplataforma (`mount_desktop`, `mount_canvas`, `mount_tui`).
