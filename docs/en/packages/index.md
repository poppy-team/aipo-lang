# Official Packages & Frameworks

The official ecosystem of the **Aipo** programming language consists of high-level libraries and frameworks engineered with the same rigor, safety-by-default, and determinism as the core compiler.

In accordance with language governance, **domain-specific features (such as HTML, CSS, UI, simulations, and audio) do not bloat the compiler grammar**. Instead, they are distributed as canonical packages managed via `aipo package`.

---

## Official Packages Catalog

| Package | Version | Domain | Description |
|---|:---:|---|---|
| **[`aipo.html`](/en/packages/aipo-html)** | `v0.1.0` | **Web & DOM** | Declarative HTML5 DSL, typed CSS-in-Aipo, and **Fine-Grained MVU** reactivity for browsers. |
| **[`aipo.ui`](/en/packages/aipo-ui)** | `v0.1.0` | **Multiplatform UI** | Universal declarative UI framework (Desktop GPU via Skia/Freya, WebGL, and TUI) with Flexbox/Taffy layout. |
| **`aipo.game`** *(Roadmap)* | `v0.1.0-alpha` | **2D Game Engine** | Actor and Scene-driven 2D micro-engine featuring deterministic simulation, rollback netcode, and multiplatform export. |

---

## How to Use an Official Package

In your project's `aipo.toml`, declare the dependency under `[dependencies]`:

```toml
[package]
name = "my-project"
version = "0.1.0"

[dependencies]
"aipo.html" = { path = "packages/aipo-html" }
```

Then in your `.aipo` source code:

```aipo
import aipo.html as h

h.div(class: "app") {
    h.h1("Built with Aipo!")
}
```
