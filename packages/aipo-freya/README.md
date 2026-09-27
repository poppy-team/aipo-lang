# aipo.freya

Framework declarativo de interface de usuário (GUI) para a linguagem **Aipo**, diretamente inspirado no [Freya UI](https://github.com/marc2332/freya) do ecossistema Rust e em sua engine de layout **Torin**.

O `aipo.freya` oferece uma arquitetura moderna para construção de aplicativos desktop e interfaces interativas através de uma árvore declarativa de elementos, gerenciamento de estado reativo com hooks (`use_state`), componentes estilizados na paleta **Catppuccin Mocha** e renderização acelerada por GPU sobre o `aipo-game-host` (Miniquad/Macroquad).

---

## Características Principais

1. **Árvore Declarativa de Componentes:**
   Construção de interfaces limpas e aninhadas utilizando funções puras de alto nível (`rect`, `label`, `container`, `button`, `switch`, `slider`, `card`, `badge`).

2. **Engine de Layout Torin:**
   Algoritmo hierárquico de medição e posicionamento que resolve:
   - Dimensões absolutas em pixels (`Float` / `Int`).
   - Dimensões relativas em porcentagem (`"100%"`, `"50%"`).
   - Dimensões intrínsecas e dimensionamento flexível (`"flex"`, `"auto"`).
   - Alinhamento transversal (`align_items`: `"start"`, `"center"`, `"end"`).
   - Distribuição no eixo principal (`justify_content`: `"start"`, `"center"`, `"end"`).
   - Espaçamento interno (`padding`, `padding_x`, `padding_y`) e espaçamento entre filhos (`gap`).

3. **Reatividade Fina com Hooks:**
   Runtime de sinais reativos (`use_state`, `set_state`) com rastreamento automático de quadros sujos (*dirty flags*), disparando reconstrução e re-layout apenas quando o estado sofre mutação.

4. **Catppuccin Mocha Embutido:**
   Paleta completa de cores modernas acessíveis via utilitários `rgb`, `rgba`, `hex` e constantes temáticas (`crust`, `mantle`, `base`, `surface_0..2`, `blue`, `lavender`, `green`, `red`, etc.).

5. **Aceleração por GPU Nativa:**
   Renderização com 60+ FPS no desktop nativo conectada diretamente ao host Miniquad através do `aipo-game-host`.

---

## Instalação

Adicione a dependência ao seu `aipo.toml`:

```toml
[dependencies]
"aipo.freya" = { path = "packages/aipo-freya" }
```

Gere ou audite o lockfile do seu projeto:

```bash
aipo package lock .
aipo package audit .
```

---

## Exemplo Rápido: Contador Reativo

```aipo
import aipo.freya as freya

fn view() {
    let count = freya.use_state(0)

    return freya.rect(
        {
            "direction": "column",
            "align_items": "center",
            "justify_content": "center",
            "width": "100%",
            "height": "100%",
            "background": freya.rgb(30, 30, 46), # Catppuccin Base
            "gap": 16.0
        },
        [
            freya.label(f"Contador: {count.get()}", {
                "font_size": 24.0,
                "color": freya.rgb(205, 214, 244)
            }),
            freya.button("Incrementar +1", _ => freya.set_state(count, count.get() + 1), {
                "variant": "primary",
                "background": freya.rgb(137, 180, 250)
            })
        ]
    )
}

fn setup() {
    freya.mount(view)
}

fn update(dt) {
    freya.step(dt)
}

fn draw() {
    freya.draw_ui()
}
```

---

## Executando o Exemplo de Demonstração

Um dashboard completo e interativo com contador, switches, sliders e barra de progresso está disponível em `examples/dashboard.aipo`:

```bash
cargo run -p aipo-game-host -- packages/aipo-freya/examples/dashboard.aipo
```

---

## Testes Automatizados

O pacote conta com uma suíte de testes de unidade e integração:

```bash
# Executa a suíte de testes do framework Freya UI
cargo test -p aipo-game-host --test bridge_tests test_freya_ui_unit_test_suite

# Executa o teste de compilação e execução estável multi-frame do Dashboard
cargo test -p aipo-game-host --test bridge_tests test_freya_ui_dashboard_compilation_and_execution
```
