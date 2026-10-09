# Aipo — novo portal documental

Uma documentação em duas trilhas: **aprender e usar Aipo** e **contribuir com a implementação**. Base: VitePress com customizações acessíveis, Markdown versionado e verificações automatizadas.

## Uso
```sh
cd website
npm install
npm run check
npm run dev
npm run build
```
Node.js 20+ recomendado. A publicação é independente do site legado em `docs/`. Os documentos antigos, corpus de conformidade e arquivos do Prumo permanecem inalterados neste PR, para preservar histórico e consumidores atuais.

## Convenções
- `content/learn/`: livro didático progressivo, com metadados em `content/_meta/chapters.json`.
- `content/guides/`: como realizar tarefas.
- `content/reference/`: sintaxe, API e comandos.
- `content/concepts/`: fundamentos e explicações.
- `content/engineering/`: implementação, decisões, limites, estudo e agentes.
- `content/archive/`: registro e roteador para documentação histórica.
- `content/en/`: tradução parcial em inglês, identificada como tal.

A sintaxe V1 desejada e a linguagem implementada não são intercambiáveis. A matriz de compatibilidade documenta o estado encontrado. As amostras ainda não recebem selo de verificação sem um teste realmente executado.

## Integridade
`npm run check` verifica links internos, destinos de navegação, frontmatter, índices e referência dos capítulos. `npm run check:examples` exercita exemplos originais quando o CLI existe, sem declarar validação de todos os capítulos. Nunca registrar status 'verified' sem saída capturada e commit correspondente.

## Status de implementação

O portal disponibiliza [Progresso da implementação](content/progress/index.md), mantido em `public/progress/tasks.json`, e [protocolo de atualização](content/engineering/progress-protocol.md). `npm run check` valida documentação, integridade do inventário e testes do painel; `npm run build` gera o site.
