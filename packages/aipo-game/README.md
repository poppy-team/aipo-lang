# aipo.game

Pacote oficial da linguagem **Aipo** para desenvolvimento ágil de jogos 2D, com arquitetura orientada a **Atores e Cenas**, **Comportamentos (*Behaviors*) plugáveis em 1 linha**, **Sistema de Nós Visuais anti-espaguete** e simulação determinística de alta performance.

Inspirado nas melhores ideias de fluxo e facilidade de motores como **Construct 3**, **ct.js** e **GameMaker Studio**, o `aipo.game` remove toda a cerimônia técnica de game dev sem sacrificar a robustez e a tipagem forte do ecossistema Aipo.

---

## Características Principais

1. **Atores e Cenas Claros:**
   Cada elemento do jogo é um `Actor` modular com ciclo de vida intuitivo (`on_create`, `on_update`, `on_collision`, `on_destroy`), posicionado dentro de uma `Scene` com suporte a câmeras e camadas de paralaxe.
2. **Behaviors Pré-Fabricados em 1 Linha:**
   Adicione movimentação completa de plataforma (`Platformer`), movimentação top-down (`TopDown`), física de projétil (`Bullet`) ou barreiras intransponíveis (`Solid`) instantaneamente.
3. **Simulação Determinística:**
   Graças ao sistema de tipos da Aipo e ao controle estrito de efeitos, a simulação matemática do jogo é 100% determinística — habilitando **Rollback Netcode** nativo para multiplayer sem esforço de sincronização e saves/replays de poucos kilobytes.
4. **Sistema de Nós Visuais Anti-Espaguete:**
   Esquema arquitetural "Gatilho $\to$ Filtro $\to$ Ação" que previne emaranhados de fios e compila bidirecionalmente para código `.aipo` transparente e limpo.
5. **Backend Multiplataforma Nativo:**
   Compatível com o runtime Rust nativo (Desktop via Skia/Miniquad com GPU acelerada) e WebAssembly (WebGL2/Canvas 2D para navegadores) a partir da mesma base de código.

---

## Instalação

Adicione ao seu `aipo.toml`:

```toml
[dependencies]
"aipo.game" = { path = "packages/aipo-game" }
```

---

## Exemplo Rápido: Nave vs Asteroides

```aipo
import aipo.game as g

// Ator Jogador
let Nave = g.actor("Nave", {
    sprite: "nave.png",
    behaviors: [
        g.behaviors.TopDown(speed: 240.0),
        g.behaviors.KeepInScreen()
    ],

    on_create: actor => {
        actor.vida = 100
        actor.cooldown = 0.0
    },

    on_update: (actor, dt) => {
        actor.cooldown -= dt
        if g.input.key_down("Space") and actor.cooldown <= 0.0 {
            g.spawn(Laser, x: actor.x, y: actor.y - 20)
            g.audio.play("laser.wav")
            actor.cooldown = 0.15
        }
    },

    on_collision: (actor, other) => {
        if other.is_a(Asteroide) {
            actor.vida -= 25
            other.destroy()
            g.fx.screen_shake(intensity: 8.0, duration: 0.3)
        }
    }
})

// Ator Laser
let Laser = g.actor("Laser", {
    sprite: "laser.png",
    behaviors: [
        g.behaviors.Bullet(speed: 600.0, angle: -90.0),
        g.behaviors.DestroyOutsideScreen()
    ]
})

// Ator Asteroide
let Asteroide = g.actor("Asteroide", {
    sprite: "asteroide.png",
    behaviors: [
        g.behaviors.Bullet(speed: 120.0, angle: 90.0),
        g.behaviors.DestroyOutsideScreen()
    ]
})

// Cena Principal
let Jogo = g.scene("Espaco", {
    width: 800,
    height: 600,
    background: "#0a0a12",

    on_load: scene => {
        g.spawn(Nave, x: 400, y: 520)

        // Gerar asteroides periodicamente
        scene.timer(interval: 0.8, repeat: true, _ => {
            let spawn_x = g.random.range(40, 760)
            g.spawn(Asteroide, x: spawn_x, y: -30)
        })
    }
})

// Inicialização
fn main() {
    g.start({
        title: "Space Defender — Aipo Game",
        width: 800,
        height: 600,
        initial_scene: Jogo
    })
}
```
