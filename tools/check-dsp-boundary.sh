#!/usr/bin/env sh
# Falla si el núcleo DSP menciona capas prohibidas.
# Ver docs/adr/0003-dsp-ui-separation.md.
set -eu

if grep -RInE '\b(truce|slint|std::fs|std::net)\b' dsp/src; then
    echo "ERROR: dsp/ importa una capa prohibida (TRUCE, Slint, filesystem o red)." >&2
    exit 1
fi

echo "OK: dsp/ no importa TRUCE, Slint, filesystem ni red."
