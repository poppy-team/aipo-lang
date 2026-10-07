# Adaptador Aipo V1 para Anthropic Claude (Claude Code & Claude Desktop)

Insira este bloco no seu arquivo de instruções do Claude (ex: `CLAUDE.md`, `.claude/CLAUDE.md` ou nos prompts de sistema):

```markdown
<!-- BEGIN AIPO V1 LANGUAGE CONTRACT -->
<aipo_language_contract version="1.0" normative="ADR-001">
  <core_principles>
    - Uma forma só para cada coisa.
    - Palavra completa vence símbolo (`and`, `or`, `not`).
    - Zero cerimônia para o caso comum: todo bloco abre com `{` e fecha com `}`.
    - Comentário é `#`. Jamais use `//` para comentário (`//` é divisão inteira).
  </core_principles>

  <forbidden_tokens severity="fatal">
    - `// comentário` -> USE `# comentário`
    - `end` -> USE `}`
    - `for item in lista` -> USE `each item in lista { ... }`
    - `repeat N:` -> USE `repeat N as i { ... }`
    - `try / catch / finally` -> USE `attempt { ... } failed erro { ... }`
    - `throw / raise` -> USE `fail "mensagem"`
    - `class` -> USE `struct Nome { ... }` e `Nome:metodo()`
    - `impl` / `satisfy` -> USE `Nome:metodo()` e `#!satisfies Interface`
    - `self!` -> USE `var self`
    - `div` / `div=` -> USE `//` e `//=`
    - `&&`, `||`, `!` -> USE `and`, `or`, `not`
    - `;` -> Quebra de linha termina statement
  </forbidden_tokens>

  <canonical_patterns>
    <struct>
      # Tipagem em campos é opcional no compilador, mas EXTREMAMENTE RECOMENDADA
      struct Entidade {
          id: Int
          var status: String = "ativo"
      }
    </struct>

    <methods>
      Entidade:consultar() -> String { return self.status }
      Entidade:atualizar(var self, novo: String) { self.status = novo }
    </methods>

    <batch_binding>
      # '::' promove funções livres que declaram 'self' ou 'var self' como 1º param
      fn resetar(var self) { self.status = "ativo" }
      Entidade::[resetar]
    </batch_binding>

    <enum>
      # Enum canônico com 3 formas de variante e match exaustivo
      enum Status {
          Pendente,
          Processando { inicio: Int },
          Concluido(codigo: Int),
      }
    </enum>

    <free_functions>
      fn calcular(a: Int, b: Int) -> Int { return a + b }
    </free_functions>

    <directives>
      # Diretivas suportadas: #!satisfies, #!test, #!deprecated, #!todo
      #!deprecated("use calcular()")
      fn calcular_legado() { }
    </directives>
  </canonical_patterns>
</aipo_language_contract>
<!-- END AIPO V1 LANGUAGE CONTRACT -->
```
