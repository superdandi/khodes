#!/usr/bin/env sh
# Valida un bundle CLAP cargándolo como un host mínimo.
# Uso: sh tools/clap-smoke.sh [ruta/al/bundle.clap]
# Por defecto usa el bundle instalado en ~/.clap/Khodes.clap.
set -eu

HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
WORK=${TMPDIR:-/tmp}/khodes-clap-smoke
BUNDLE=${1:-$HOME/.clap/Khodes.clap}

if [ ! -f "$BUNDLE" ]; then
    echo "ERROR: no existe el bundle CLAP: $BUNDLE" >&2
    exit 1
fi

if [ ! -d "$WORK/clap/include/clap" ]; then
    mkdir -p "$WORK"
    git clone -q --depth 1 https://github.com/free-audio/clap.git "$WORK/clap"
fi

cc -std=c11 -Wall -Wextra -I"$WORK/clap/include" \
    "$HERE/clap_smoke_test.c" -o "$WORK/clap_smoke_test" -ldl -lm

"$WORK/clap_smoke_test" "$BUNDLE"
