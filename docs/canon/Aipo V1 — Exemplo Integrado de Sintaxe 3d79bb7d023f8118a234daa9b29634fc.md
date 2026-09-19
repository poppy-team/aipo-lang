# Aipo V1 — Exemplo Integrado de Sintaxe

<aside>
🧪

**Objetivo:** manter um único exemplo amplo e coerente que exercite a maior parte da superfície sintática da Aipo V1. Ele é material de conformidade pedagógica, não substitui a Language Reference nem a Sintaxe Canônica.

</aside>

## Exemplo integrado

```
# demo.aipo

import io
import math: clamp
import graphics.renderer as gfx

export Player, Drawable, Damageable, create_player
export gfx.Sprite as GameSprite

interface Drawable
    fn draw(self)
end

interface Damageable
    fn damage(self!, amount: Int)
end

struct Player
    fixed id
    name
    health
    position
end

impl Player
    init(id: Int, name: String, health: Int = 100)
        self.id = id
        self.name = name
        self.health = health
        self.position = {
            "x": 0.0,
            "y": 0.0
        }
    end

    invariant()
        self.id is Int
        self.name is String
        self.health is Int
        self.name != ""
        self.health >= 0
        self.health <= 100
    end

    fn alive(self) -> Bool
        return self.health > 0
    end

    fn damage(self!, amount: Int)
        self.health -= amount
    end

    fn heal(self!, amount: Int)
        self.health += amount
        self.health = clamp(self.health, 0, 100)
    end

    fn draw(self)
        io.print(f"Player {self.name}: {self.health}")
    end
end

satisfy Player: Drawable, Damageable

fn create_player(id: Int, name: String, health: Int = 100) -> Player
    return Player{id, name, health}
end

fn find_player(players, id: Int) -> Player?
    each player in players
        if player.id == id then return player
    end

    return none
end

fn validate_score(score: Int)
    if score < 0
        fail("score cannot be negative")
    end
end

fn load_score() -> Int
    # Exemplo de operação que poderia falhar em implementação real.
    return 10
end

fn log(message: String)
    io.print(message)
end

fn run_callback(callback: Function)
    callback()
end

fn with_message(callback: Function)
    callback("Aipo trailing block")
end

let enabled = true
let disabled = false
let nothing = none

let integer = 10
let decimal = 3.5

let normal_text = "Aipo"
let interpolated = f"Language: {normal_text}"
let raw_path = r"C:\games\aipo"
let raw_asset = fr"C:\games\{normal_text}\player.png"
let multiline = """Aipo
is a small
scripting language"""

let real_division = 5 / 2
let integer_division = 5 div 2
let remainder = 5 % 2

let as_float = Float(10)
let as_int = Int(3.8)
let as_text = String(42)

let arithmetic = (10 + 5) * 2 - 4
let comparison = arithmetic >= 10 and arithmetic != 100
let opposite = not comparison

var counter = 10
counter += 2
counter -= 1
counter *= 3
counter div= 2
counter %= 7

var ratio = 10.0
ratio /= 4

var players = [
    create_player(1, "Ana"),
    create_player(2, "Bia", health = 80)
]

let first = players[0]
let last = players[-1]
let beginning = players[..1]
let ending = players[1..]
let all_players = players[..]

var settings = {
    "fullscreen": false,
    "volume": 80,
    "language": "pt-BR"
}

settings["volume"] = 90

if settings.has("volume")
    io.print(settings["volume"])
end

let original = players[0]
let alias = original
let clone = copy(original)

io.print(same(original, alias))
io.print(same(original, clone))

let maybe_player = find_player(players, 1)
let maybe_name = maybe_player?.name

if maybe_player.some()
    maybe_player.draw()
end

if integer is Int
    io.print("integer")
end

if integer, counter is Int
    io.print("both Int")
end

if not normal_text is Int
    io.print("not an Int")
end

if counter > 10
    io.print("high")
elif counter == 10
    io.print("ten")
else
    io.print("low")
end

if enabled then io.print("enabled")
if disabled then io.print("disabled") else io.print("enabled")

match maybe_name
when "Ana"
    io.print("found Ana")
when "Bia", "Maria"
    io.print("found another player")
else
    io.print("unknown")
end

each player in players
    player.draw()
end

each index, player in players
    io.print(index, player.name)
end

each key, value in settings
    io.print(key, value)
end

each char in "Aipo"
    io.print(char)
end

each index, char in "Aipo"
    io.print(index, char)
end

each i in 0..5
    io.print(i)
end

repeat 3
    io.print("repeat")
end

var remaining = 3
while remaining > 0
    remaining -= 1
end

var number = 0
loop
    number += 1
    if number == 2 then continue
    if number >= 5 then break
    io.print(number)
end

let double = fn(value: Int) -> Int
    return value * 2
end

io.print(double(10))

fn create_counter()
    var value = 0

    return fn()
        value += 1
        return value
    end
end

let next_count = create_counter()
io.print(next_count())
io.print(next_count())

run_callback(fn()
    io.print("callback")
end)

with_message() do message
    io.print(message)
end

let safe_score = load_score() or_else 0

attempt
    validate_score(safe_score)
    each player in players
        player.draw()
    end
failed err
    io.print(f"Could not process: {err.message}")
end

attempt
    validate_score(safe_score)
failed _
    io.print("validation failed")
end

log("Aipo demo finished")
```

## Regras que este exemplo reforça

- Construção de `struct` usa `Type{...}`; chamadas usam `()`.
- Campos de `struct` não recebem contratos declarativos na V1.
- `each` é a iteração canônica; `for` não é alias.
- `if ... then ... else ...` também pode produzir valor em contexto de expressão; a V1 não possui ternário `?:`.
- `do ... end` é closure passada como último argumento.
- `none`, Failure e função sem resultado continuam semanticamente distintos.
- O exemplo deve ser atualizado sempre que uma mudança normativa da V1 for aprovada.

## Extensões de ergonomia V1 aprovadas em 2026-09-10

O exemplo principal acima antecede este fechamento; os próximos refreshes editoriais devem incorporar estas formas diretamente ao programa integrado.

```
# escolha condicional de valor
let status_label = if enabled then "enabled" else "disabled"

# repeat com índice opt-in
repeat 3 as i
    io.print(i)
end

# pipeline
let prepared = source
    |> tokenize
    |> parse
    |> analyze

# destructuring superficial
let [x, y] = [10, 20]
let {name, age} = {
    "name": "Ana",
    "age": 20
}

# continuação de comparação
if age is Int and >= 0 and <= 130
    io.print("valid age")
end

# chamada sem () antes de trailing block + builder explícito
html do page
    page.body do body
        body.h1(f"Hello {name}")
    end
end

# concatenação continua útil quando já temos duas Strings
let label = "Aipo" + " V1"
```

A forma canônica já está fechada: escolha condicional de valor usa `if condition then value else value`; fallback recuperável usa `or_else`; não existe ternário `?:` na V1.

## Modelo numérico/binário V1 — exemplos

```
# Int usa o intervalo inteiro exato comum VM ↔ JavaScript
let port = Int("8080") or_else 8080

# Float continua sendo binary64
let speed = Float("4.5") or_else 1.0

# Byte é explicitamente 0..255
let alpha = Byte("255") or_else Byte(255)

# String continua separada de números
let label = f"port={port}, speed={speed}, alpha={alpha}"

# Binário usa Bytes + formatos de armazenamento
let data = Bytes(32)
data.write_i32(0, port)
data.write_f32(4, speed)
data.write_u8(8, alpha)

let stored_port = data.read_i32(0)
let stored_speed = data.read_f32(4)
let stored_alpha = data.read_u8(8)

# Texto ↔ bytes usa UTF-8 explicitamente
let encoded = "Olá".encode()
let decoded = encoded.decode() or_else "invalid UTF-8"
```

Os nomes `i8/u8/i16/u16/i32/u32/i64/u64/f32/f64` descrevem formatos de packing, não tipos numéricos cotidianos da linguagem.