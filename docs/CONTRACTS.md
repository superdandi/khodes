# Contratos del producto — Khodes

Estos son los contratos canónicos de Khodes. No son promesas de compatibilidad
con experimentos descartados. Antes de un release pueden cambiar mediante una
actualización explícita de código, tests, documentación y ADR.

---

## 1. Identidad

| Campo | Valor |
|---|---|
| Nombre visible | Khodes |
| Vendor | `khodes` (ajustar al titular) |
| Crate | `khodes` |
| Bundle slug | `khodes` |
| Tipo | Instrumento (MIDI in, audio out) |
| Formatos | CLAP y VST3 |
| Host de desarrollo | Standalone interno, no distribuido |
| Categoría | Instrument / Synthesizer |
| Licencia | MIT (código original) |

La identidad vive en `truce.toml`; no se duplica en el código.

---

## 2. Formatos y plataformas

- **CLAP**: soportado.
- **VST3**: soportado.
- macOS: Apple silicon como objetivo primario; Intel si CI lo cubre.
- Windows: x64.
- Linux: x64, glibc; compatibilidad de DAW sujeta a pruebas reales.
- AU, AAX, VST2: no soportados en v1.
- El standalone no se distribuye.

No se anuncia una plataforma sin CI y prueba en host real.

---

## 3. Esquema de parámetros

- Exactamente **19 parámetros de host** en v1.
- IDs `1..19`; ID `0` no se usa.
- Todos normalizados `0..1` en el host.
- Sin parámetros ocultos que afecten el sonido guardado.
- IDs estables: no se renumeran ni se reutilizan.
- Cambiar curva/rango es cambio de sonido: requiere ADR y nuevo corpus.

Ver [PARAMETERS](PARAMETERS.md).

---

## 4. MIDI

- Canales 1–16, omni por defecto.
- C4 = MIDI 60; A4 = 440 Hz.
- Note-on con velocity 0 = note-off.
- Velocity afecta el nivel según `velocity_sensitivity`.
- CC64 sustain por canal.
- Pitch bend ±`pitch_bend_range` semitonos, default 2.
- CC1 mod wheel afecta tremolo (performance, no persistente).
- CC7 escala la ganancia maestra en su offset (performance, no persistente).
- CC120/121/123 soportados; CC121 restaura bend, sustain, mod wheel y CC7.
- Aftertouch, MPE, MIDI 2.0, Program Change y MIDI Out: no en v1.
- Eventos con `sample_offset`; no `setTimeout`.

Ver [MIDI](MIDI.md).

---

## 5. Timing

- Los eventos se aplican en el offset de muestra indicado por el host.
- El motor parte el bloque en segmentos.
- No se pierden note-off por note-on dentro del mismo bloque.
- Si el host no provee offsets, se aplican al inicio del bloque y se documenta.
- El transporte del host no altera la afinación ni el estado de voces.

---

## 6. Seguridad en tiempo real

`process` y sus llamadas:

- no asignan memoria;
- no redimensionan buffers;
- no bloquean;
- no registran logs;
- no acceden a disco, red o entorno;
- no ejecutan trabajo no acotado;
- no hacen `panic!` no controlado.

Todo buffer, IR, delay y estado se prepara en `prepare`/`reset`.

La salida debe ser finita; los subnormales se fuerzan a cero.

---

## 7. Estado y presets

- El estado del host es la fuente de verdad del proyecto.
- El preset nativo es `.khodespreset`, versión 1.
- `PRESET_VERSION` es independiente de la versión del producto.
- El códec valida magic, versión, IDs, rangos, UTF-8, CRC y tamaño.
- No se guardan notas, sustain, bend, fases de LFO, colas de reverb ni estado de
  voces.
- Al restaurar, todas las voces parten libres.
- El editor no abre diálogos de archivo propios.

Ver [STATE_AND_PRESETS](STATE_AND_PRESETS.md).

---

## 8. Formato `.khodespreset` v1

- Magic: `KHODESP\0` (8 bytes).
- `version: u32` = 1.
- `param_count: u32` = 19.
- `name_len: u32` ≤ 63.
- `name`: UTF-8.
- 19 × (`id: u32`, `value: f64`).
- `crc32: u32`.
- Little-endian.
- Tamaño máximo: 4096 bytes.
- Sin bytes extra.
- Carga atómica: todo o nada.

---

## 9. Versionado

- Producto: `vMAJOR.MINOR.PATCH-preview.N`.
- Tag de release: `v...`.
- Previews: prerelease, unsigned, draft revisado y publicado a mano.
- Stable: proceso separado, con firma/notarización si aplica.
- No se mueve un tag publicado.
- No se publica desde CI.
- `APPROVED.txt` es la aprobación explícita.

Ver [RELEASES](RELEASES.md).

---

## 10. Sitio y descargas

- Sitio Astro estático.
- GitHub Releases es la fuente de verdad.
- Drafts nunca aparecen.
- Contrato de nombres:

```text
Khodes-VERSION-windows-x64-unsigned.zip
Khodes-VERSION-macos-unsigned.zip
Khodes-VERSION-linux-x64-unsigned.zip
SHA256SUMS
```

- Solo esos archivos generan botones.
- Notas escapadas como texto plano.
- `GITHUB_TOKEN` solo en build.
- El sitio no sirve binarios desde su propio dominio.

Ver [SITE](SITE.md).

---

## 11. Latencia

- Khodes no introduce latencia adicional deliberada.
- El drive con oversampling puede introducir latencia; si es así, se reporta al
  host y se documenta.
- La reverb y el chorus no se reportan como latencia: son efectos, no
  pre-delay del instrumento.
- La latencia reportada debe coincidir con la real.

---

## 12. Compatibilidad de presets

- Un preset v1 debe cargar en toda versión 1.x.
- Un parámetro nuevo se añade al final; los presets antiguos usan su default.
- Un parámetro eliminado deja su ID reservado.
- Una versión de códec no soportada se rechaza; no se interpreta a medias.

---

## 13. Compatibilidad de sonido

- El corpus `contract/v1/` se congela con la primera versión aceptada.
- Un cambio de sonido intencional crea `v2/`.
- No se reescribe un corpus congelado para que pase un cambio.
- Las diferencias se documentan en notas de release.

---

## 14. Licencias

- Código original: MIT.
- TRUCE: `LicenseRef-TruceLicense-1.0`.
- Slint: royalty-free 2.0 (o comercial), con atribución.
- Cada ZIP incluye `LICENSE`, `INSTALL.txt` y avisos revisados.

Ver [LICENSING](LICENSING.md).

---

## 15. Lo que no es un contrato

- La apariencia exacta del editor puede evolucionar.
- El orden de los controles puede cambiar si se documenta.
- Los presets de fábrica pueden añadirse; no se renombran sin aviso.
- El rendimiento depende del hardware.
- La compatibilidad con DAWs no listados no está garantizada.
