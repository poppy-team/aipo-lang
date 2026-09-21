# ADR 001: Linha de Base Arquitetural e Princípios de Clean Architecture

## Status
Parcialmente superseded — ver nota abaixo. O conteúdo genérico (SRP, DIP,
YAGNI) continua como orientação; as decisões de arquitetura do compilador
vivem em `docs/architecture/overview.md` + `docs/crates/crate-contracts.md`.

> Nota de supersessão (2026-09-20, auditoria de docs): este ADR foi escrito
> como boilerplate (menciona DTOs, persistência, drivers, "classes") e não
> descreve a arquitetura real — pipeline frontend → Core IR → dois backends,
> sem banco de dados, sem frameworks web, sem classes (Rust). Nada aqui foi
> apagado; onde este documento conflitar com `overview.md`/crate-contracts,
> estes vencem (ver `docs/language/authority-map.md`: contratos de arquitetura
> têm precedência como fonte do estado implementado).

## Contexto
O projeto requer alta manutenibilidade, isolamento rigoroso de regras de negócio em relação a frameworks e dependências externas, e suporte a testes unitários e de integração sem dependências de infraestrutura pesada.

## Decisão
Adotamos a Clean Architecture (Portas e Adaptadores). Todas as dependências devem apontar para o domínio central. Comunicações com infraestrutura externa devem ocorrer exclusivamente por meio de interfaces de portas.

## Consequências
- **Positivas**: Testabilidade completa em memória; facilidade de substituição de drivers de persistência; independência de frameworks.
- **Negativas**: Introdução de camadas intermediárias de mapeamento de dados (DTOs e entidades).
