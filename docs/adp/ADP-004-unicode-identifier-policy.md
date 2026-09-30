# ADP-004 — Política de Identificadores Unicode e de Segurança

**Status:** rascunho (questões em aberto — apenas caracterização, nenhuma restrição adicionada)
**Relacionado:** `crates/aipo-cli/tests/unicode_security.rs` (fixa o comportamento atual),
`docs/evidence/P01-G02-*.md`, UAX #31, UTS #39 (consultados, não adotados)
**Autoridade:** subordinada à Language Reference e à política de não invenção

## Comportamento atual verificado (medido e depois fixado em testes)

- Os caracteres de identificador são `char::is_alphabetic` (primeiro) e
  `is_alphanumeric` (demais) do Rust, mais `_`. Consequência: letras não ASCII pré-compostas
  funcionam (`é`, `中`, `α`, cirílico); marcas combinantes, caracteres de largura zero e
  controles bidi são rejeitados com `AIPO_LEX_UNEXPECTED_TOKEN`.
- O source loader **não** normaliza para NFC: um identificador decomposto `e` + U+0301
  é rejeitado em vez de se compor em `é`. (Os *literais* de string são normalizados pelo
  lexer conforme ADP-001 Q5; os identificadores não.)
- Scripts mistos e confusáveis não têm restrição: o `сount` cirílico e o `count` latino
  coexistem como bindings distintos, sem nenhum aviso.
- U+00A0 (espaço sem quebra) não é whitespace de código-fonte: um NBSP no final é um erro
  do lexer, não um separador ignorado.
- Sequências de emoji/ZWJ são preservadas em strings; `len` conta code points.

## Questões em aberto (todas indecididas — NÃO altere o comportamento aqui)

1. O loader deve normalizar para NFC (fazendo os identificadores decompostos se comporem), ou
   a rejeição é o comportamento especificado?
2. Identificadores confusáveis/de scripts mistos são um risco que justifica um aviso
   (detecção highly-restrictive/confusable da UTS #39), ou a aceitação irrestrita é a
   posição da V1?
3. A rejeição de U+00A0 como whitespace é intencional, ou ele deveria entrar no conjunto de
   whitespace ignorável?
4. As rejeições de largura zero/bidi merecem um código de diagnóstico próprio, ou
   `AIPO_LEX_UNEXPECTED_TOKEN` é o sinal correto (ainda que genérico)?
5. O *conteúdo* das strings deveria sofrer alguma restrição (hoje não sofre nenhuma)?

## Não-objetivos deste ADP

- Adicionar qualquer restrição, aviso ou normalização sem uma decisão de design de
  acompanhamento. Estes testes caracterizam; uma mudança de política precisa de um goal
  próprio, com patrocínio do canon e migração do corpus/fixtures que ela afetar.
