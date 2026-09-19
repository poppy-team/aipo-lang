# aipo-lexer

`aipo-lexer` provides lexical analysis, tokenization, Unicode NFC handling, string literal prefix extraction, and lexical diagnostic collection for Aipo V1.

## Architecture & Guarantees
- **No Evaluation at Lex Time**: Numbers are preserved as exact source slices (`Number { raw }`) with no premature floating-point parsing or truncation.
- **String Prefixes**: Supports `""`, `f""`, `r""`, `fr""`, and multiline `"""` strings.
- **Significant Newlines**: Significant newlines after statements are tokenized; irrelevant newlines (e.g. inside delimiters) are filtered.
- **Resilient Recovery**: Unknown characters produce `AIPO_LEX_UNEXPECTED_CHAR` diagnostics and advance without aborting tokenization.
- **Safety**: `#![forbid(unsafe_code)]`.
