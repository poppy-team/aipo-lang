# 04 — Web, JavaScript, Unicode, Binário e Interoperabilidade

<aside>
🌐

**Objetivo:** preservar a mesma semântica Aipo na VM Odin e no backend JavaScript. JavaScript é target; suas diferenças internas não devem vazar para o código Aipo.

</aside>

## ECMAScript / JavaScript

### Especificação normativa

- [ECMAScript 2026 / ECMA-262](https://tc39.es/ecma262/2026/)
- [Living ECMAScript Specification](https://tc39.es/ecma262/)

**Uso:** referência final quando MDN e comportamento observado deixarem dúvida sobre semântica JavaScript.

### Inteiros seguros

- [MDN — Number.MAX_SAFE_INTEGER](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number/MAX_SAFE_INTEGER)

**Decisão Aipo relacionada:** `Int` público limitado a `±(2^53-1)` para equivalência exata VM↔JS sem `BigInt` normal.

## Binário e buffers

- [MDN — TypedArray](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/TypedArray)
- [MDN — DataView](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/DataView)

**Uso:** implementar `Bytes` e packing `i8/u8/i16/u16/i32/u32/i64/u64/f32/f64` no target JS. Os nomes são formatos de armazenamento Aipo, não tipos cotidianos.

### Luau buffer

- [Luau Standard Library — buffer](https://luau.org/library/#buffer-library)

**Uso:** principal referência conceitual para manter um tipo numérico simples enquanto buffers oferecem formatos compactos.

## UTF-8 e Strings

### Unicode Standard Annex #15

- [UAX #15 — Unicode Normalization Forms](https://www.unicode.org/reports/tr15/)

**Decisão Aipo relacionada:** source/identifiers/strings normalizados em NFC onde a especificação determinar.

### utf8proc

- [utf8proc](https://juliastrings.github.io/utf8proc/)
- [utf8proc API](https://juliastrings.github.io/utf8proc/doc/)

**Uso:** dependência externa preferida para NFC, case-folding e operações Unicode que não vale reimplementar.

### JavaScript encoding

- [MDN — TextEncoder](https://developer.mozilla.org/en-US/docs/Web/API/TextEncoder)
- [MDN — TextDecoder](https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder)

**Uso:** `String.encode()` e `Bytes.decode()` UTF-8 no backend Web.

## Source maps e debugging

- [ECMA-426 — Source Map Format](https://ecma-international.org/publications-and-standards/standards/ecma-426/)
- [Latest ECMA-426 draft](https://tc39.es/ecma426/)

**Uso futuro:** gerar `.map` para permitir stack traces e debugging apontando para `.aipo`, não apenas para JavaScript gerado.

## Backend JS — regra arquitetural

```
Aipo Source
    ↓
Semantic/Core IR
    ↓
JavaScript Emitter
    ↓
app.js + aipo-runtime.js + app.js.map
```

`aipo-runtime.js` deve conter apenas diferenças semânticas inevitáveis, por exemplo:

- operações de String por code point quando JS usar UTF-16 internamente;
- range checking de `Int`/`Byte`;
- `Failure`/`or_else` lowering quando necessário;
- `Bytes`/packing via `Uint8Array`/`DataView`;
- helpers de igualdade/identidade quando o mapeamento direto não preservar a semântica.

## Async futuro

- [Python asyncio](https://docs.python.org/3/library/asyncio.html) — ergonomia e structured concurrency.
- [Janet Event Loop](https://janet-lang.org/docs/event_loop.html) — fibers cooperativas + event loop.

**Uso:** comparar API pública da Aipo com mecanismos do host; não copiar toda a implementação de nenhum deles.

## Conformance do backend

Sempre que possível, cada teste semântico deve poder rodar em:

```
Aipo VM
versus
Aipo → JavaScript
```

Comparar stdout, valores observáveis, `Failure`, Unicode, `Int`, `Float`, `Byte`, `Bytes`, collections e control flow. Divergência entre os dois backends é bug até prova em contrário.