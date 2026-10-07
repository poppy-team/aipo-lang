#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILL_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=== Verificando exemplos canônicos da skill lang-aipo ==="
python3 "${SCRIPT_DIR}/lint_aipo.py" "${SKILL_ROOT}/examples"

echo "=== Todos os exemplos estão estritamente em conformidade com Aipo V1 ==="
