# ADP-007 — Package identity e distribuição

**Status:** accepted
**Date:** 2026-09-24
**Related:** Fechamento Arquitetural §13, §14; `docs/crates/crate-contracts.md`; `docs/canon/Aipo — Waves, Vertical Slices, Gauntlet Loops`
**Authority:** subordinate ao canon; esta ADP fixa a primeira implementação sem criar um serviço de plataforma próprio.

## A. Coordinate de package

Um packagecoordinate tem exatamente dois segmentos separados por ponto:

```text
namespace.package
```

Exemplo: `acme.http`. O coordinate é a identidade canônica usada no manifest, lockfile, SourceMap e diagnósticos. Os segmentos usam a gramática ASCII lowercase `[a-z][a-z0-9_]*`; esta decisão não aceita ainda submodules (`acme.http.client`).

A origem fica explícita na cadeia de resolução: `acme.http` não é confundido com o module embutido `http` ou com um package local de nome igual.

## B. Imports qualificados

A superfície de importação usa o coordinate completo:

```aipo
import acme.http
import acme.http as h
```

O alias continua siendo local ao arquivo que o declara. O coordinate original permanece disponível para proveniência, lockfile e diagnóstico. Módulos embutidos permanecem simples (`math`, `task`, `io`, `time`).

O primeiro slice suporta apenas o módulo de entrada de cada package. A forma de submodules fica para uma decisão posterior, para não ambiguar a fronteira entre coordinate e caminho interno.

## C. Distribuição

Aipo possui o protocolo de package; GitHub é apenas o backend inicial de distribuição. O modelo de source separa origem, revisão e artifact:

- `path` para desenvolvimento local;
- GitHub source com slug `owner/repository`, commit SHA exato de 40 hex e subpath relativo;
- registry source como adapter futuro, sem acoplar o resolver a GitHub.

O manifesto aceita dependencies `path` e GitHub pinado. A forma canônica para GitHub é `type = "github"`, com `repository`, `revision`, `subpath` e versão SemVer exata. O lockfile registra coordinate, versão exata, source, revisão, checksum, targets e capabilities relevantes. Branches, tags, URLs, credenciais e revisões mutáveis são rejeitados. Um `GitHubFetcher` e um store in-memory offline permitem testar a provenance sem rede; a feature `http` acrescenta GET público, timeout, limite de resposta, redirects bloqueados e cache local atômico.

`aipo package fetch-github` é a única operação que ativa a busca pública. Ela percorre recursivamente as dependencies GitHub pinadas, valida o grafo e grava o lockfile completo no snapshot de saída. Dependencies locais continuam disponíveis no resolver local; uma dependency path declarada dentro de um artifact remoto é rejeitada porque não existe um filesystem raiz confiável para interpretá-la. `run`, `check`, `build`, `disasm`, `package lock` e `package audit` não acessam a rede. Autenticação, downloads automáticos, lifecycle e registry publication continuam fora deste slice. Não há scripts arbitrários de instalação ou execução de lifecycle no V1.

## D. Capabilities

Capabilities declaradas por packages são um limite superior. A política do projeto e a política do host podem reduzir esse conjunto; nunca ampliá-lo silenciosamente. A união transitiva é auditável e deve caber no limite declarado pela raiz.

O `aipo-host` continua sendo a fonte da gramática e das regras de narrowing. `aipo-stdlib` publica a superfície da API; o serviço efetivo é instalado e verificado no host, como já ocorre com `clock` e Poppy. Os primeiros serviços novos são `env.read` e o slice host-only `fs.read_text`/`fs.roots`: cada provider vive no `HostContext` do VM, a instalação não concede capabilities e a política de raízes/paths pertence ao provider.

## E. Consequências

- `import` deixa de aceitar apenas um identificador e passa a carregar um caminho qualificado.
- Parser, AST/HIR, resolver, linker, formatter, source maps e diagnósticos precisam preservar o coordinate completo.
- O resolver de packages deve ser separado do `ModuleGraph` de runtime; este continua sendo o grafo de inicialização de módulos já resolvidos.
- O slice local não implementa rede, assinatura de artifacts nem plataforma web. A feature `http` só permite fetch público explícito; assinatura, cache distribuído e registries continuam atrás de fixtures e decisões de segurança próprias.

## F. Próximos slices

1. `aipo-package`: manifest, coordinate, lock e resolver `path` determinístico — concluído.
2. Capabilities e auditoria de dependências, com `env.read` como primeira prova per-VM — concluído.
3. Módulos capability-aware da stdlib, começando por `fs.read_text`/`fs.roots` e o contrato async-first — concluído no slice host-only.
4. Source GitHub pinado com provenance, digest e seam offline — concluído no slice P04-G04.
5. Adapter HTTP público, cache local e comando explícito `fetch-github` — concluído no slice P04-G05; autenticação e downloads via `aipo.toml` permanecem desligados.
6. Dependências GitHub recursivas em manifesto e lockfile, com fetch somente pelo comando explícito — concluído no slice P04-G06.
7. Registry próprio somente se os gatilhos de escala, privacidade ou assinatura forem atingidos.
