#!/usr/bin/env bash
# Gera tudo: o lote normal (tools/output) e o lote de teste com foto de
# fundo + mascara circular (tools/output_teste). Roda com pouco barulho no
# terminal (saida de tudo vai pro /dev/null); se algo falhar o script para
# e mostra o erro, ja que -e/-o pipefail continuam ativos.
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE="$DIR/gerador"

cd "$CRATE"

cargo build --release > /dev/null 2>&1 || {
    echo "erro ao compilar:" >&2
    cargo build --release
    exit 1
}

./target/release/gerar_lote > /dev/null 2>&1
./target/release/gerar_lote_teste > /dev/null 2>&1

echo "ok: output e output_teste gerados"
