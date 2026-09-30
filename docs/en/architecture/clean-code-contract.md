# Architecture and Clean Code Contract

This document defines the **mandatory software engineering contract** for all implementations in this project.

## 1. Clean Code principles

1. **Explicit Responsibilities (SRP)**: Each module, class or package has a single reason to change.
2. **High Cohesion and Low Coupling**: Modules must be self-contained and interact only through abstract interfaces or typed contracts.
3. **Expressive Domain Names**: Variables, functions and types must reflect the ubiquitous language of the business. Do not use generic names such as `manager`, `helper`, `utils` or `data`.
4. **Small, Focused Functions**: Functions must perform only one logical action and ideally fit on one screen.
5. **Explicit Errors**: Silently swallowing exceptions (`bare except`, ignoring errors) is forbidden. Every error must be handled, wrapped or propagated with context.
6. **Zero Speculative Abstraction (YAGNI)**: Implement abstractions only when there are two or more proven concrete use cases.

## 2. Dependency direction (Clean Architecture)

- The dependency flow always points **inward**, toward the essential business rules.
- External mechanisms (databases, web frameworks, CLI, third-party libraries) are infrastructure details encapsulated by adapters.
- The application core is unaware of external protocols or specific cloud vendors.

## 3. Modularity and decoupling

- No package or module may import its consumer.
- Dependency cycles are strictly forbidden and checked in the CI pipeline.
- Every repository folder must be self-explanatory and contain its own `README.md`.
