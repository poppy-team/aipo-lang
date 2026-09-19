# aipo-source

`aipo-source` manages UTF-8 source files, span encodings, line/column calculations, and BOM/newline normalizations for the Aipo programming language compiler.

## Architecture & Guarantees
- **Span Tracking**: Zero-allocation, lightweight `SourceSpan` (u32 start and end byte offsets).
- **Encoding Normalization**: Strips leading UTF-8 BOM (`\u{FEFF}`) and normalizes CRLF (`\r\n`) and CR (`\r`) to LF (`\n`).
- **Precomputed Line Indexing**: O(log N) line and column lookup using binary search on precalculated line start offsets.
- **Safety**: `#![forbid(unsafe_code)]` with 100% safe Rust.
