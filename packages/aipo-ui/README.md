# aipo.ui

Pacote oficial da linguagem **Aipo** para construção de interfaces declarativas universais e multiplataforma com layout **Taffy** (CSS Flexbox e CSS Grid) e renderizadores desacoplados (**Desktop GPU Skia**, **WebGL/Canvas** e **Terminal TUI**).

---

## Filosofia & Separação de Domínios

Diferente do [`aipo.html`](../aipo-html/README.md) (que opera estritamente sobre a árvore DOM de navegadores e CSS web tradicional), o `aipo.ui` é **100% agnóstico de plataforma**:

1. **Mesma Árvore Declarativa em Qualquer Lugar:** Um único código de interface roda nativamente no Desktop com aceleração por hardware (Vulkan/Metal/DirectX via Skia), no navegador via WebAssembly/Canvas sem custos de reflow do DOM, ou diretamente no Terminal com cores ANSI 24-bit TrueColor.
2. **Motor de Layout Taffy Integrado:** Implementação de ponta de CSS Flexbox e Grid em Rust, compilada com zero overhead de interpretação, calculando coordenadas com precisão de submícron.
3. **Reatividade MVU Previsível:** Estado imutável gerenciado pelo padrão *Model-View-Update* com despachos tipados e zero mutação oculta.

---

## Instalação

Adicione ao seu `aipo.toml`:

```toml
[dependencies]
"aipo.ui" = { path = "packages/aipo-ui" }
```

---

## Exemplo Rápido: Contador Universal

```aipo
import aipo.ui as ui
import aipo.ui.color as color

struct Model {
    count: Int
}

enum Msg {
    Increment,
    Decrement,
    Reset
}

fn update(m: Model, msg: Msg) -> Model {
    match msg {
        Msg::Increment => Model{ count: m.count + 1 },
        Msg::Decrement => Model{ count: m.count - 1 },
        Msg::Reset => Model{ count: 0 }
    }
}

fn view(m: Model, dispatch: Fn) {
    ui.Column(
        gap: 16,
        padding: 32,
        align: ui.Align::Center,
        justify: ui.Justify::Center,
        background: color.rgb(24, 24, 27)
    ) {
        ui.Text(
            f"Valor Atual: {m.count}",
            font_size: 28,
            font_weight: "bold",
            color: color.white
        )

        ui.Row(gap: 12) {
            ui.Button("+1", on_click: _ => dispatch(Msg::Increment), variant: "primary")
            ui.Button("-1", on_click: _ => dispatch(Msg::Decrement), variant: "secondary")
            ui.Button("Zerar", on_click: _ => dispatch(Msg::Reset), variant: "danger")
        }
    }
}

// Inicialização Multiplataforma
fn main() {
    ui.mount_desktop(
        title = "Contador Aipo UI",
        width = 400,
        height = 300,
        init = Model{ count: 0 },
        update = update,
        view = view
    )
}
```

---

## Renderizadores Pluggáveis

| Renderizador | Alvo | Motor de Desenho | Características |
|---|---|---|---|
| **`aipo-ui-renderer-skia`** | Desktop (Linux, macOS, Windows) | Skia 2D / GPU (Vulkan, Metal, DX12) | 120+ FPS, anti-aliasing de texto subpixel, sombras com blur gaussiano, aceleração total de GPU. |
| **`aipo-ui-renderer-canvas`** | WebAssembly (Navegadores) | HTML5 Canvas 2D / WebGL | Aplicação executando em bytecode Wasm de alto desempenho, contornando a árvore DOM e repaints do browser. |
| **`aipo-ui-renderer-tui`** | Terminal ANSI | Terminal Virtual / Crossterm | Execução instantânea em servidores e terminais sem servidor X11/Wayland, com suporte a mouse e 24-bit TrueColor. |
