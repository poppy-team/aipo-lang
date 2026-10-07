#!/usr/bin/env python3
"""
Linter léxico e sintático estrito para Aipo V1 (ADR-001).
Detecta violações de tokens proibidos e alucinações comuns de LLMs.
"""

import sys
import re
from pathlib import Path

FORBIDDEN_KEYWORDS = [
    (r"\bclass\b", "Palavra-chave 'class' não existe em Aipo. Use 'struct' e métodos 'Tipo:nome'."),
    (r"\bimpl\b", "'impl' foi removido no ADR-001. Declare métodos diretamente com 'Tipo:nome'."),
    (r"\bsatisfy\b", "'satisfy' foi removido no ADR-001. A satisfação é puramente estrutural."),
    (r"\bself!\b", "'self!' foi removido no ADR-001. Use 'var self' no receptor do método."),
    (r"\bend\b", "Keyword 'end' foi removida no ADR-001. Blocos fecham com '}'."),
    (r"\bdiv\b", "'div' foi removido no ADR-001. Use '//' para divisão inteira."),
    (r"\btry\b", "'try' não existe em Aipo. Use 'attempt { ... } failed erro { ... }'."),
    (r"\bcatch\b", "'catch' não existe em Aipo. Use 'failed' junto de 'attempt'."),
    (r"\bfinally\b", "'finally' não existe em Aipo. Use transações atômicas com journal."),
    (r"\bthrow\b", "'throw' não existe em Aipo. Use 'fail expr'."),
    (r"\braise\b", "'raise' não existe em Aipo. Use 'fail expr'."),
    (r"\bdefer\b", "'defer' não existe em Aipo."),
    (r"\bfor\s+\w+\s+in\b", "'for' não existe em Aipo. Use 'each item in colecao { ... }'."),
    (r"\bconst\b", "'const' não existe em Aipo. Use 'let' (imutável por padrão)."),
    (r"\b(pub|public|private)\b", "Modificadores de acesso não existem. Visibilidade é por módulo com 'export'."),
    (r"\bvoid\b", "'void' não existe em Aipo. Use 'none'."),
]

FORBIDDEN_OPERATORS = [
    (r"&&", "Operador '&&' não existe. Use 'and'."),
    (r"\|\|", "Operador '||' não existe. Use 'or'."),
    (r"\+\+", "'++' não existe. Use '+= 1'."),
    (r"--", "'--' não existe. Use '-= 1'."),
    (r";\s*$", "Ponto e vírgula ';' no fim de linha é proibido. Quebra de linha termina statement."),
    (r"/\*", "Comentários de bloco '/*' não existem. Use '#' para comentários de linha."),
]

FORBIDDEN_PATTERNS = [
    (r"^\s*//", "'//' no início da linha não é comentário. Em Aipo, comentários usam '#' e '//' é divisão inteira."),
    (r"\s//\s+[A-Za-zÀ-ÿ]", "'//' parece ter sido usado como comentário. Use '#' para comentários."),
    (r"\b(fn\s+[A-Z]\w*:[a-z_]\w*|[A-Z]\w*:fn\s+[a-z_]\w*)", "Métodos associados 'Tipo:nome' NUNCA usam 'fn'."),
    (r"\bif\s+.*\{.*then", "'then' nunca deve ser usado com blocos '{}'."),
]


def lint_file(file_path: Path) -> list:
    errors = []
    lines = file_path.read_text(encoding="utf-8").splitlines()

    for idx, line in enumerate(lines, start=1):
        # Ignora strings simples para evitar falsos positivos
        clean_line = re.sub(r'"([^"\\]|\\.)*"', '""', line)

        for pattern, msg in FORBIDDEN_KEYWORDS:
            if re.search(pattern, clean_line):
                errors.append((idx, line, msg))

        for pattern, msg in FORBIDDEN_OPERATORS:
            if re.search(pattern, clean_line):
                errors.append((idx, line, msg))

        for pattern, msg in FORBIDDEN_PATTERNS:
            if re.search(pattern, clean_line):
                errors.append((idx, line, msg))

    return errors


def main():
    if len(sys.argv) < 2:
        print("Uso: lint_aipo.py <caminho_ou_arquivo.aipo>")
        sys.exit(1)

    target = Path(sys.argv[1])
    files = [target] if target.is_file() else list(target.rglob("*.aipo"))

    total_errors = 0
    for f in sorted(files):
        errs = lint_file(f)
        if errs:
            total_errors += len(errs)
            print(f"\n[ERRO] {f}:")
            for line_no, content, msg in errs:
                print(f"  Linha {line_no}: {msg}")
                print(f"    > {content.strip()}")

    if total_errors > 0:
        print(f"\nTotal de violações encontradas: {total_errors}")
        sys.exit(1)
    else:
        print(f"Sucesso: {len(files)} arquivo(s) inspecionado(s) sem violações.")
        sys.exit(0)


if __name__ == "__main__":
    main()
