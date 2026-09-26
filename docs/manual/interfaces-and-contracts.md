# Interfaces & Contratos

O sistema de tipos do Aipo une a ergonomia da tipagem dinâmica com a precisão dos **contratos de assinatura**, **invariantes de dados** e **interfaces estruturais**.

---

## Estruturas (`struct`)

Estruturas agregam campos nomeados. Por padrão, campos são mutáveis apenas se declarados com `var`, ou estritamente imutáveis se declarados como `fixed`:

```aipo
struct Servidor {
  fixed id: String,     // Imutável após inicialização
  fixed criado_em: Int, // Imutável após inicialização
  var status: String,   // Mutável
  var carga: Float      // Mutável
}
```

Tentar reatribuir um campo `fixed` após a construção da instância dispara o diagnóstico semântico estático `AIPO_SEM_FIXED_REASSIGN`.

---

## Hook de Construção (`init`)

O hook `init` permite validar e normalizar dados no momento em que a estrutura é instanciada:

```aipo
struct Usuario {
  email: String,
  nome: String,

  init() {
    if not self.email.contains("@") then
      fail "Formato de e-mail inválido"
    end
  }
}
```

---

## Invariantes Estruturais (`invariant`)

As invariantes declaram predicados lógicos que **devem permanecer verdadeiros durante todo o ciclo de vida do objeto**.

```aipo
struct Intervalo {
  var inicio: Int,
  var fim: Int,

  invariant() {
    self.inicio <= self.fim
  }
}
```

Sempre que um campo de uma estrutura com `invariant()` for alterado, o motor de execução verifica automaticamente o predicado. Caso a verificação falhe, a operação é rejeitada. Se estiver dentro de um bloco `attempt`, as mutações anteriores são desfeitas pelo journal transacional.

---

## Interfaces e Implementações (`interface` / `satisfy`)

Interfaces definem contratos de métodos que uma estrutura deve implementar:

```aipo
interface Renderizavel {
  fn renderizar(contexto) -> None
  fn obter_limites() -> List
}

struct Botao {
  texto: String
}

impl Renderizavel for Botao {
  fn renderizar(contexto) {
    contexto.desenhar_texto(self.texto)
  }

  fn obter_limites() {
    return [0, 0, 100, 30]
  }
}
```

A cláusula `satisfy` é validada pelo analisador semântico (`aipo-sema`), verificando aridade de parâmetros, nome de métodos, compatibilidade do receptor `self` e assinaturas assíncronas antes da execução.
