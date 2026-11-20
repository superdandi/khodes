# Roadmap — Khodes

**Estado:** plan vivo. Solo contiene estado de fases, alcance y criterios de
salida. La lista ejecutable de tareas y el avance detallado viven en
[`PLAN.md`](PLAN.md).

---

## Visión

Khodes v1 es un piano eléctrico de tine, derivado de la voz Rhodes de Keys,
que suena fiel en cualquier DAW mediante CLAP y VST3, con una UI propia y un
proceso de release reproducible.

---

## Fases

### M0 — Documentación y contratos

**Estado:** completado con este entregable.

Entregables:

- [x] `README.md`
- [x] `AGENTS.md`
- [x] `VOICE_SPEC.md`
- [x] `ARCHITECTURE.md`
- [x] `PARAMETERS.md`
- [x] `MIDI.md`
- [x] `STATE_AND_PRESETS.md`
- [x] `UI_SPEC.md`
- [x] `REFERENCE_BUFFER_UPPERCUT.md`
- [x] `DEV_SETUP.md`
- [x] `TESTING.md`
- [x] `RELEASES.md`
- [x] `SITE.md`
- [x] `LICENSING.md`
- [x] `CONTRACTS.md`
- [x] ADRs 0001–0005
- [x] `PLAN.md` con la lista maestra de tareas y gates

Criterio de salida: revisión humana de los documentos y aceptación de los
contratos de parámetros, MIDI y preset. El avance tarea por tarea se marca en
[`PLAN.md`](PLAN.md).

---

### M1 — Esqueleto del plugin

**Estado:** esqueleto y builds completados. Bundles CLAP y VST3 generados e
instalados, standalone compilado y smoke host CLAP verde. Pendientes: primera
ejecución real de CI (M1-22) y validación en DAW real (M1-18; REAPER no está
instalado en este equipo, así que no se afirma compatibilidad). Detalle en
[`PLAN.md`](PLAN.md).

Entregables:

- `Cargo.toml`, `truce.toml`, `build.rs`.
- Workspace con `dsp/`, `kit/`, `src/`, `ui/`.
- `PluginLogic` mínimo con salida silenciosa.
- CLAP y VST3 compilan e instancian.
- Standalone de desarrollo (`target/release/khodes-standalone`, no distribuido).
- CI básico de fmt, Clippy, tests y smoke host CLAP.
- `tools/clap-smoke.sh`: carga del bundle y ciclo de vida verificado sin DAW.

Criterio de salida: el plugin carga en REAPER y produce silencio sin crashear.
**Pendiente de verificar en host real**; la comprobación disponible es el smoke
host CLAP de [`TESTING.md`](TESTING.md) §8.1.

---

### M2 — Voz monofónica

**Estado:** implementada y con tests verdes. `dsp::Params`, eventos con
`sample_offset`, PRNG SplitMix64, dos osciladores, ADSR con la cola de 50 ms de
Keys, campana, key click, velocity, pitch bend y fine tune; integrados en el
wrapper. `cargo test --workspace --release`: 16 tests, 0 fallos. Pendiente el
render A/B contra Keys (M2-19), que requiere render offline de Web Audio no
disponible en este entorno. Detalle en [`PLAN.md`](PLAN.md).

Entregables:

- Oscilador fundamental y campana.
- Envelope ADSR.
- Key Click.
- Una voz, una nota MIDI.
- Tests de afinación y envelope.

Criterio de salida: MIDI 60 suena con la afinación y el carácter esperados;
A/B inicial contra Keys documentado. **A/B pendiente**; la afinación está
verificada a ±1 cent en MIDI 21, 60, 69 y 108.

---

### M3 — Polifonía y allocator

**Estado:** implementado y verificado (M3-01 a M3-17). Pool fijo de 32 voces,
robo determinista, fade de robo de 4 ms, sustain por canal e IDs internos de
nota. 22 tests verdes; baseline de CPU 6,0× tiempo real con 16 voces a 48 kHz
(véase [`TESTING.md`](TESTING.md) §11.1). Detalle en [`PLAN.md`](PLAN.md).

Entregables:

- Pool de 1–32 voces.
- Voice stealing determinista.
- Note-on/off, velocity, sustain.
- 32 notas simultáneas sin crashes ni samples no finitos.

Criterio de salida: acordes y pasajes rápidos estables; tests de allocator
verdes.

---

### M4 — Bus global

**Estado:** implementado y verificado; corpus v1 congelado. Queda abierta la
incidencia de CPU de la reverb (M4-22): 0,7× tiempo real en la CPU local cuando
se activa; el preset por defecto no la activa. Ver
[`TESTING.md`](TESTING.md) §11.2.

Entregables:

- Drive, brightness, EQ (implementados en `dsp/src/bus.rs`).
- Chorus, tremolo (implementados).
- Reverb convolucional determinista de 2,5 s (implementada).
- Master y limitador de seguridad (implementados).
- Smoothing de parámetros en sub-bloques de 32 frames (implementado).
- Salida estéreo en el wrapper y 13 fixtures en `contract/v1/` con `SHA256SUMS`.

Criterio de salida: barridos sin clics; CPU dentro de presupuesto; corpus v1
congelado y verificado por el harness.

---

### M5 — Editor Slint

**Estado:** implementado y verificado.

Entregables:

- Panel Voz/Env/Amp/FX/GLOBAL (`ui/main.slint`).
- Knobs, faders, sliders y readouts formateados en Rust (`src/params.rs`).
- Medidor de voces/salida y LEDs de nota/sustain vía slots atómicos.
- Estados hover/active/focus/disabled y navegación por Tab.
- Combo de presets de fábrica (tabla provisional en `src/editor.rs`).
- Cuatro baselines en `screenshots/`, generados con `cargo run --example screenshots`.

Criterio de salida: editor usable en default (1120×700), mínimo (920×620) y
wide (1440×760); accesibilidad básica; sin diálogos de archivo propios. Detalles
en [`UI_SPEC.md`](UI_SPEC.md) §10 y §13.

---

### M6 — Estado y presets

Entregables:

- Estado del host vía TRUCE.
- Presets de fábrica.
- Códec `.khodespreset` v1.
- Validación y tests de ida y vuelta.
- Migración y rechazo de versiones.

Criterio de salida: guardar/reabrir en dos DAWs; presets portables; sin notas
reanudadas.

---

### M7 — MIDI avanzado

Entregables:

- Pitch bend.
- Mod wheel.
- CC120/121/123.
- Eventos sample-accurate.
- Pruebas de canales y overlapping holds.

Criterio de salida: matriz MIDI de [TESTING](TESTING.md) verde.

---

### M8 — Primer preview público

Entregables:

- Sitio Astro con home, downloads e installation.
- Workflow de release y Pages.
- `VERSION.md`, `THIRDPARTY.json`, `APPROVED.txt`.
- ZIPs unsigned para Windows, macOS y Linux.
- Notas de release.

Criterio de salida: draft revisado, checksums verificados, release publicada a
mano, sitio actualizado.

---

### M9 — Estabilización

Entregables:

- Matriz de DAWs obligatoria completa.
- Corrección de bugs.
- CPU y memoria aceptadas.
- Documentación de limitaciones.
- Firma/notarización si aplica.

Criterio de salida: versión `0.1.0` estable publicada.

---

## Fuera de alcance v1

| Feature | Motivo |
|---|---|
| AU | Requiere otro wrapper y pruebas en Logic |
| AAX | Requiere SDK de Avid y licencias |
| VST2 | Formato legado |
| MPE | Requiere pitch bend por nota y allocator más complejo |
| MIDI 2.0 | Ecosistema todavía limitado |
| Samples multimuestra | Cambia el modelo de producto |
| Modelado físico | Fuera del alcance de fidelidad a Keys |
| Arpegiador/secuenciador | Khodes es instrumento, no estación |
| Presets en la nube | Requiere backend y cuentas |
| Pan por voz | Puede romper la fidelidad del bus |
| Reverb algorítmica | La convolución determinista es suficiente para v1 |

---

## Riesgos

| Riesgo | Impacto | Mitigación |
|---|---|---|
| Divergencia con Keys | Pérdida de identidad | Corpus A/B y criterios de VOICE_SPEC |
| CPU de reverb | Dropouts | IR corta, benchmark, bypass |
| Aliasing de campana | Dureza en agudos | Bandlimit / atenuación cerca de Nyquist |
| Complejidad de TRUCE | Curva de aprendizaje | Empezar con el standalone y el ejemplo mínimo |
| Licencias | Bloqueo de release | Revisión temprana de TRUCE/Slint |
| Falta de DAWs | Compatibilidad no probada | Matriz obligatoria y documentación honesta |
| Presets incompatibles | Pérdida de confianza | IDs estables y códec versionado |
| Release accidental | Binarios rotos | Draft manual y `APPROVED.txt` |

---

## Métricas de éxito v1

- El plugin carga en REAPER y Ableton Live sin crashear.
- 16 voces simultáneas a 48 kHz usan menos del 10 % de un core en un CPU moderno.
- El sonido es reconociblemente la voz de Keys en A/B.
- Guardar/reabrir conserva los 19 parámetros.
- Los presets v1 cargan en versiones 1.x futuras.
- El sitio muestra la versión y los checksums correctos.
- Un usuario puede instalar y tocar en menos de 10 minutos.
