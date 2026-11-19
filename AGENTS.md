# AGENTS.md — Reglas de trabajo en Khodes

Este archivo es obligatorio para cualquier agente, colaborador o automatización
que modifique el repositorio. Si hay conflicto entre una instrucción verbal y
este documento, gana el contrato escrito en `docs/CONTRACTS.md`.

## Lectura obligatoria antes de tocar código

1. `README.md`
2. `docs/VOICE_SPEC.md`
3. `docs/ARCHITECTURE.md`
4. `docs/CONTRACTS.md`
5. `docs/PARAMETERS.md`
6. `docs/STATE_AND_PRESETS.md`
7. El ADR relacionado con el área que se va a cambiar.

## Invariantes que nunca se rompen

- `dsp/` no importa TRUCE, Slint, tipos de host, sistema de archivos ni UI.
- `process` no asigna memoria, no redimensiona buffers, no bloquea, no registra
  logs, no accede a disco y no ejecuta trabajo no acotado.
- Los IDs de parámetros publicados no se renumeran ni se reutilizan.
- La versión del códec `.khodespreset` es independiente de la versión del
  producto.
- Un preset o estado del host no reanuda notas sostenidas ni estado transitorio.
- El editor no abre diálogos de archivo propios: los presets completos se
  cargan y guardan desde el host.
- Los binarios y artefactos de build no se commitean.
- Ningún cambio de sonido intencional se hace pasar reescribiendo un corpus de
  regresión congelado.
- No se anuncia un formato, plataforma o DAW sin CI y validación en host real.

## Flujo de trabajo

```sh
cargo truce doctor
cargo test --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --features rt-paranoid
cargo truce build --clap --vst3
```

Antes de entregar, ejecutar además los checks específicos del área:

- DSP: tests unitarios + corpus `contract/` + benchmark.
- UI: screenshots Slint en los tamaños comprometidos.
- Host: matriz de [TESTING](docs/TESTING.md).
- Sitio: `npm test`, `npm run build`, `npm run test:build`.

## Reglas por tipo de cambio

### Cambiar DSP

1. Trabajar en `dsp/`.
2. Añadir tests unitarios con vectores de entrada/salida.
3. Si el sonido cambia a propósito, crear una nueva versión del corpus de
   contrato; no editar el corpus anterior.
4. Documentar la decisión en un ADR.
5. Actualizar `docs/VOICE_SPEC.md` si cambia la especificación de la voz.

### Cambiar un parámetro

1. Actualizar `src/params.rs`.
2. Actualizar la traducción parámetro→DSP en `src/lib.rs`.
3. Actualizar bindings del editor y Slint si es visible.
4. Actualizar `docs/PARAMETERS.md` y `docs/CONTRACTS.md`.
5. Actualizar tests de esquema, automatización y estado.

### Cambiar MIDI

1. Actualizar el mapeo en `src/midi.rs`.
2. Actualizar la agregación de holds y el scheduling en `src/lib.rs`.
3. Probar todos los canales, aliases, note-off y overlapping holds.
4. Actualizar `docs/MIDI.md` y la checklist de DAW.

### Cambiar el códec de presets

1. Incrementar `PRESET_VERSION` en `kit/`.
2. Actualizar tests de ida y vuelta, validación y rechazo.
3. Documentar compatibilidad y migración en `docs/STATE_AND_PRESETS.md`.
4. Añadir ADR si la decisión es estructural.

### Cambiar el sitio

1. El sitio es estático; no introducir un servidor de API en runtime.
2. Las descargas se leen de GitHub Releases en tiempo de build.
3. Los drafts nunca se publican.
4. Actualizar `site/README.md` y `docs/SITE.md` juntos.

## Documentación

- Un dato, un documento autoritativo. Los demás enlazan.
- `CONTRACTS.md`: reglas estables.
- `ARCHITECTURE.md`: estructura y propiedad de cada módulo.
- `PARAMETERS.md`: esquema de parámetros.
- `VOICE_SPEC.md`: comportamiento de la voz.
- `TESTING.md`: procedimientos de aceptación.
- `ROADMAP.md`: solo estado actual y trabajo pendiente.
- `PLAN.md`: lista maestra de tareas, gates y registro de avance. Se actualiza
  al cerrar cada sesión.
- `adr/`: razones de las decisiones.

No copiar tablas enteras entre documentos; usar enlaces.

## Estilo

- Documentación en español.
- Identificadores de código, archivos y comandos en inglés.
- Números con unidades (`ms`, `s`, `Hz`, `dB`, `%`).
- Fechas en ISO 8601 (`YYYY-MM-DD`).
- Nombres de producto y formatos con su grafía oficial: CLAP, VST3, TRUCE,
  Slint, Rust, Astro, GitHub Pages.

## Prohibiciones

- No commitear `target/`, `node_modules/`, `dist/`, bundles instalados,
  screenshots temporales ni caches.
- No desactivar tests, validadores o checks de licencias para “hacer pasar” un
  release.
- No publicar un release automáticamente desde CI.
- No afirmar compatibilidad con un DAW no probado.
- No usar código de terceros sin revisar y conservar su licencia.
