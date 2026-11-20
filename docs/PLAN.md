# Plan de implementación — Khodes

**Estado:** plan vivo. Es la lista maestra de trabajo y el registro de avance.
**Última actualización:** 2026-11-19.
**Fase actual:** M1 — Fundaciones del repositorio (parcialmente bloqueada por toolchain).

Este documento descompone el [ROADMAP](ROADMAP.md) en tareas ejecutables,
ordenadas y verificables. Cada tarea tiene un identificador estable
(`M<fase>-<número>`) que no se reutiliza. El avance se marca aquí, no en el
roadmap.

---

## 1. Cómo se usa este plan

### 1.1 Leyenda

| Marca | Significado |
|---|---|
| `- [x]` | Completada y verificada |
| `- [ ]` | Pendiente |
| `- [ ] **EN CURSO**` | Se está trabajando en ella ahora |
| `- [ ] **BLOQUEADA**` | No se puede avanzar sin resolver una dependencia |

### 1.2 Reglas

1. Una tarea se marca `[x]` solo cuando su **criterio de aceptación** se
   cumplió de verdad, no cuando se escribió el código.
2. Las tareas no se borran ni se renumeran. Si una deja de aplicar, se marca
   `[x]` con la nota “descartada” y el motivo.
3. Las tareas bloqueadas indican el bloqueo entre paréntesis.
4. Cada fase tiene un **gate** que se cumple antes de pasar a la siguiente. Un
   gate incompleto no se compensa con tareas de otra fase.
5. Todo cambio de contrato (parámetros, MIDI, preset, sonido) exige actualizar
   los documentos correspondientes y, si es estructural, un ADR.
6. Al cerrar una sesión de trabajo se actualiza la tabla de estado y el
   [registro de avances](#9-registro-de-avances).

### 1.3 Orden de ejecución

```text
M0 ──► M1 ──► M2 ──► M3 ──► M4 ──► M5 ──► M6 ──► M7 ──► M8 ──► M9
                  │                   │
                  └── corpus v1 ──────┘
```

M2 y M3 pueden solaparse parcialmente. M8 (sitio) no espera a M9, pero no se
anuncia hasta que M9 valide los binarios.

---

## 2. Tabla de estado

| Fase | Nombre | Estado | Gate |
|---|---|---|---|
| M0 | Documentación y contratos | **Completada** (salvo revisión humana) | Documentos aceptados |
| M1 | Fundaciones y esqueleto | **Implementada** (falta DAW real y CI) | Plugin silencioso carga en REAPER |
| M2 | Voz monofónica | **Implementada** (falta A/B) | MIDI 60 afinado y con carácter |
| M3 | Polifonía y allocator | **Implementada y verificada** | 32 notas sin fallos |
| M4 | Bus global y efectos | Pendiente | Corpus v1 congelado |
| M5 | Editor Slint | Pendiente | Editor usable en 3 tamaños |
| M6 | Estado y presets | Pendiente | Guardar/reabrir en 2 DAWs |
| M7 | MIDI avanzado | Pendiente | Matriz MIDI verde |
| M8 | Sitio y release | Pendiente | Draft revisado y publicado |
| M9 | Validación y estabilización | Pendiente | `v0.1.0` estable |

**Bloqueo activo:** ninguno crítico. Toolchain validado, M1–M3 implementados.
Pendientes de validación externa: carga en DAW real (M1-18), CI con remote
(M1-22) y render A/B contra Keys (M2-19). Siguiente tarea de código: M4 — bus
global y efectos.

---

## 3. M0 — Documentación y contratos

**Objetivo:** fijar el producto antes de escribir código.
**Gate:** revisión humana de contratos.

- [x] **M0-01** Crear `README.md` con alcance, decisiones y mapa documental.
- [x] **M0-02** Crear `AGENTS.md` con invariantes y flujo de trabajo.
- [x] **M0-03** Crear `docs/VOICE_SPEC.md` con la voz heredada y reglas de port.
- [x] **M0-04** Crear `docs/ARCHITECTURE.md` con capas, crates y concurrencia.
- [x] **M0-05** Crear `docs/PARAMETERS.md` con 19 IDs, rangos, defaults y curvas.
- [x] **M0-06** Crear `docs/MIDI.md` con notas, velocity, sustain y bend.
- [x] **M0-07** Crear `docs/STATE_AND_PRESETS.md` con el códec v1.
- [x] **M0-08** Crear `docs/UI_SPEC.md` con layout, paleta y componentes.
- [x] **M0-09** Crear `docs/REFERENCE_BUFFER_UPPERCUT.md` con el análisis de la referencia.
- [x] **M0-10** Crear `docs/DEV_SETUP.md` con toolchain y build.
- [x] **M0-11** Crear `docs/TESTING.md` con la estrategia de pruebas.
- [x] **M0-12** Crear `docs/RELEASES.md` con versionado y publicación.
- [x] **M0-13** Crear `docs/SITE.md` con el contrato del sitio.
- [x] **M0-14** Crear `docs/LICENSING.md` con MIT, TRUCE y Slint.
- [x] **M0-15** Crear `docs/CONTRACTS.md` con los contratos duros.
- [x] **M0-16** Crear `docs/ROADMAP.md` con las fases M0–M9.
- [x] **M0-17** Crear `docs/adr/` con el índice y los ADRs 0001–0005.
- [x] **M0-18** Crear `site/README.md` con la guía operativa del sitio.
- [x] **M0-19** Verificar automáticamente los enlaces internos (106/106 existen).
- [x] **M0-20** Crear este `docs/PLAN.md` y enlazarlo desde README, AGENTS y ROADMAP.
- [ ] **M0-21** Revisión humana y aceptación formal de `PARAMETERS`, `MIDI`,
      `CONTRACTS` y `STATE_AND_PRESETS`.
- [ ] **M0-22** Etiquetar los contratos aceptados en git (`contracts-v1`).

**Criterio de salida:** los contratos quedan aceptados y etiquetados. Hasta
entonces, M2 en adelante puede avanzar en prototipo, pero no publicarse.

---

## 4. M1 — Fundaciones y esqueleto del plugin

**Objetivo:** repositorio reproducible, workspace compilable y plugin silencioso
que carga en un DAW.
**Gate:** CLAP y VST3 cargan en REAPER sin crashear y producen silencio.

### 4.1 Repositorio

- [x] **M1-01** Inicializar repositorio git en `/home/dandi/khodes`.
- [x] **M1-02** Crear `.gitignore` para Rust, Node, bundles y capturas temporales.
- [x] **M1-03** Añadir `LICENSE` MIT para el código original.
- [x] **M1-04** Añadir `rust-toolchain.toml` fijando la versión objetivo.
- [x] **M1-05** Instalar Rust 1.92+ con `rustup` y verificar `rustc --version`
      (instalado 1.92.0, commit `ded5c06cf`).
- [x] **M1-06** Instalar `cargo-truce` 6.3.0 y verificar `cargo truce --version`.
- [x] **M1-07** Ejecutar `cargo truce doctor` y resolver faltantes de plataforma
      (toolchain, compiladores y `truce.toml` en OK; validadores opcionales
      pendientes en M9).
- [ ] **M1-08** Configurar identidad de git y proteger la rama `main`.
- [x] **M1-09** Crear `Cargo.toml` virtual con `dsp/` y `kit/`, resolver 3.
- [x] **M1-10** Crear `truce.toml` con identidad, formatos, categoría y bundle
      (`category = "instrument"`, `midi_input = true`, `vst3_subcategory = "Synth"`).
- [x] **M1-11** Crear `build.rs` para la compilación de la UI Slint.
- [x] **M1-12** Crear el crate `dsp/` sin dependencias de framework.
- [x] **M1-13** Crear el crate `kit/` sin dependencias de host ni UI.
- [x] **M1-14** Crear el crate raíz `src/` con `PluginLogic` mínimo (compila).
- [x] **M1-15** Crear `ui/` con una ventana Slint vacía.
- [x] **M1-16** Compilar CLAP y VST3. En local se usa
      `cargo truce build --clap --vst3 --target-cpu v2` (CPU sin AVX2); los
      bundles están en `target/bundles/` y tienen los símbolos `clap_entry` y
      `GetPluginFactory`.
- [x] **M1-17** Compilar el standalone de desarrollo
      (`target/release/khodes-standalone`, 12 MB). No se distribuye.
- [ ] **M1-18** Instalar y cargar los bundles en REAPER. Instalados en
      `~/.clap` y `~/.vst3`; carga real verificada con el smoke host CLAP
      (`tools/clap-smoke.sh`). REAPER no está instalado en este equipo: queda
      pendiente la validación en DAW real.
- [x] **M1-19** Crear `.github/workflows/ci.yml`; incluye el smoke host CLAP
      (pendiente primera ejecución real).
- [x] **M1-20** Añadir chequeo de dependencias prohibidas en `dsp/`
      (`tools/check-dsp-boundary.sh`, pasa en local).
- [x] **M1-21** Documentar la versión exacta de TRUCE y Slint en el lockfile
      (`truce 6.3.0`, `truce-slint 6.3.0`, `slint 1.15.1`).
- [ ] **M1-22** Primer commit verde con CI. Commit local creado (`bf983e3`),
      pero el repositorio no tiene `remote`; la primera ejecución real de CI
      queda pendiente hasta configurar GitHub.
- [x] **M1-23** Crear el smoke host CLAP (`tools/clap_smoke_test.c` y
      `tools/clap-smoke.sh`), documentarlo en `TESTING.md` §8.1 y ejecutarlo en
      CI. Pasa localmente sobre el bundle instalado.

**Criterio de salida:** `cargo test --locked --workspace` pasa, CLAP y VST3 se
instalan, y REAPER muestra **Khodes** como instrumento sin errores.

---

## 5. M2 — Voz monofónica

**Objetivo:** una nota MIDI 60 suena como la voz de Keys.
**Gate:** afinación ±1 cent, envelope dentro de tolerancia y A/B documentado.

### 5.1 Fundamentos DSP

- [x] **M2-01** Definir `dsp::Params` con los 19 valores y defaults.
- [x] **M2-02** Implementar conversión MIDI → frecuencia (`440 * 2^((n-69)/12)`).
- [x] **M2-03** Implementar PRNG determinista (SplitMix64) para ruido e IR.
- [x] **M2-04** Implementar flush de denormales y finitud de salida.
- [x] **M2-05** Definir `dsp::Event` con `sample_offset`.

### 5.2 Osciladores y envelopes

- [x] **M2-06** Implementar oscilador seno con fase acumulada estable.
- [x] **M2-07** Implementar parcial de campana a `2f` con límite de Nyquist.
- [x] **M2-08** Implementar envelope ADSR con la semántica de `VOICE_SPEC` §3.3,
      incluida la cola de 50 ms de `rhodes.ts`.
- [x] **M2-09** Implementar envelope de campana independiente (sin sustain).
- [x] **M2-10** Implementar key click (ruido HPF 3 kHz, ~30 ms).
- [x] **M2-11** Implementar ganancia de velocity con piso 0.3.
- [x] **M2-12** Implementar fine tune y pitch bend como multiplicadores
      suavizados.

### 5.3 Integración y pruebas

- [x] **M2-13** Procesar una sola voz por bloque con segmentación por offset.
- [x] **M2-14** Conectar parámetros del host a `dsp::Params`.
- [x] **M2-15** Test de afinación en MIDI 21, 60, 69 y 108 (±1 cent).
- [x] **M2-16** Test de tiempos de attack, decay, sustain y release.
- [x] **M2-17** Test de key click a 0 y a 1.
- [x] **M2-18** Test de reproducibilidad con la misma semilla.
- [ ] **M2-19** Render A/B contra Keys y guardar evidencia en `docs/`. Requiere
      render offline de Web Audio; no disponible en este entorno.
- [x] **M2-20** Verificación contra `rhodes.ts`: sin desviaciones; la cola de
      release de 50 ms ya estaba documentada en `VOICE_SPEC` §3.3.

**Criterio de salida:** una nota suena reconociblemente como Keys; los tests
pasan y la comparación A/B está documentada.

---

## 6. M3 — Polifonía y allocator

**Objetivo:** acordes y pasajes rápidos estables, sin clics ni voces zombie.
**Gate:** 32 notas simultáneas sin samples no finitos ni fugas.

- [x] **M3-01** Definir `SlotState` y el pool fijo de 1–32 voces (`allocator.rs`).
- [x] **M3-02** Implementar asignación de voz libre.
- [x] **M3-03** Implementar robo de la voz en release más antigua.
- [x] **M3-04** Implementar robo de la voz sostenida más antigua.
- [x] **M3-05** Implementar fade de 4 ms en el robo mediante una cola acotada
      de 32 `StealTail`, para no cortar la voz robada.
- [x] **M3-06** Implementar `note_id` en `src/midi.rs`: TRUCE no expone el ID
      del host, así que el wrapper asigna IDs secuenciales por (canal, nota).
- [x] **M3-07** Implementar retrigger de nota repetida.
- [x] **M3-08** Implementar note-off por nota y canal.
- [x] **M3-09** Implementar sustain CC64 por canal.
- [x] **M3-10** Implementar All Notes Off (CC123), All Sound Off (CC120) y Reset
      All Controllers (CC121).
- [x] **M3-11** Implementar `polyphony` como parámetro discreto (`dsp::Params`).
- [x] **M3-12** Test de 1, 8, 16 y 32 voces.
- [x] **M3-13** Test de robo determinista con secuencia fija.
- [x] **M3-14** Test de nota repetida sin note-off.
- [x] **M3-15** Test de note-off sin note-on previo.
- [x] **M3-16** Test de 1000 note-on sin note-off.
- [x] **M3-17** Medir CPU de 16 voces a 48 kHz y registrar baseline: 6,0×
      tiempo real, 0,8846 ms por bloque de 256 frames, 1.659 s de CPU para
      10.00 s de audio. Registrado en [`TESTING.md`](TESTING.md) §11.1.

**Criterio de salida:** la polifonía es estable y determinista; el baseline de
CPU queda registrado en [TESTING](TESTING.md).

---

## 7. M4 — Bus global y efectos

**Objetivo:** reproducir la cadena de efectos de Keys con parámetros suavizados.
**Gate:** corpus `contract/v1/` congelado y barridos sin clics.

- [ ] **M4-01** Implementar suma de voces y entrada al bus.
- [ ] **M4-02** Implementar drive tanh con oversampling 4x y bypass exacto.
- [ ] **M4-03** Implementar lowpass de brillo con escala log 500–20000 Hz.
- [ ] **M4-04** Implementar lowshelf 250 Hz y highshelf 4 kHz.
- [ ] **M4-05** Implementar delay modulado del chorus (22 ms, ±4 ms).
- [ ] **M4-06** Implementar mezcla dry/wet del chorus.
- [ ] **M4-07** Implementar tremolo con LFO senoidal 0–10 Hz.
- [ ] **M4-08** Implementar reverb convolucional determinista de 2.5 s.
- [ ] **M4-09** Implementar master con volumen y limitador de seguridad.
- [ ] **M4-10** Implementar smoothing de parámetros (~20 ms).
- [ ] **M4-11** Implementar flush de subnormales en el bus.
- [ ] **M4-12** Test de bypass de drive a 0.
- [ ] **M4-13** Test de barrido de brillo sin zipper noise.
- [ ] **M4-14** Test de EQ a ±12 dB.
- [ ] **M4-15** Test de chorus: delay base, LFO y mezcla.
- [ ] **M4-16** Test de tremolo: rate, depth y fase.
- [ ] **M4-17** Test de reverb: IR determinista y mezclas 0/1.
- [ ] **M4-18** Test de 16 voces sin saturar la salida.
- [ ] **M4-19** Crear `contract/v1/` con los 12 casos de [TESTING](TESTING.md) §4.1.
- [ ] **M4-20** Añadir `SHA256SUMS` y el harness de corpus.
- [ ] **M4-21** Benchmark de CPU con reverb y chorus on/off.

**Criterio de salida:** el sonido del bus coincide con la especificación y el
corpus v1 queda congelado con checksums.

---

## 8. M5 — Editor Slint

**Objetivo:** editor propio usable, accesible y coherente con Keys.
**Gate:** screenshots estables en default, mínimo y wide; sin diálogos de archivo.

- [ ] **M5-01** Definir la ventana y el layout base en Slint.
- [ ] **M5-02** Implementar `Knob` con arrastre vertical y doble clic.
- [ ] **M5-03** Implementar `Fader` vertical con interacción absoluta.
- [ ] **M5-04** Implementar `Slider` horizontal para Bass y Treble.
- [ ] **M5-05** Implementar `Readout` con formato por unidad.
- [ ] **M5-06** Implementar `Section` con LED y color de acento.
- [ ] **M5-07** Implementar las secciones VOZ, ENV, AMP, FX y GLOBAL.
- [ ] **M5-08** Implementar medidor de voces y nivel de salida.
- [ ] **M5-09** Implementar estados hover, activo, foco y deshabilitado.
- [ ] **M5-10** Conectar gestos de automatización begin/set/end.
- [ ] **M5-11** Implementar teclado y accesibilidad.
- [ ] **M5-12** Implementar combo de presets de fábrica.
- [ ] **M5-13** Verificar que no hay diálogos de archivo propios.
- [ ] **M5-14** Capturar baseline default (1120×700).
- [ ] **M5-15** Capturar baseline mínimo (920×620).
- [ ] **M5-16** Capturar baseline wide (1440×760).
- [ ] **M5-17** Capturar baseline con nota activa.
- [ ] **M5-18** Revisar contraste AA y navegación por Tab.
- [ ] **M5-19** Documentar atajos y comportamiento en `UI_SPEC`.

**Criterio de salida:** el editor se abre con el audio corriendo, responde sin
bloquear y los baselines están comprometidos.

---

## 9. M6 — Estado y presets

**Objetivo:** persistencia correcta y portátil.
**Gate:** guardar/reabrir en dos DAWs sin reanudar notas.

- [ ] **M6-01** Implementar el estado del host vía TRUCE.
- [ ] **M6-02** Definir los 6 presets de fábrica en `kit/`.
- [ ] **M6-03** Implementar `PRESET_VERSION = 1` y el layout binario.
- [ ] **M6-04** Implementar CRC-32.
- [ ] **M6-05** Implementar `encode` con nombre UTF-8 y límite de 63 bytes.
- [ ] **M6-06** Implementar `decode` con validación completa.
- [ ] **M6-07** Implementar carga atómica.
- [ ] **M6-08** Implementar rechazo de magic, versión, CRC y tamaño.
- [ ] **M6-09** Implementar rechazo de IDs, rangos y UTF-8 inválido.
- [ ] **M6-10** Test round-trip de los 19 parámetros.
- [ ] **M6-11** Test round-trip de los 6 presets.
- [ ] **M6-12** Test de valores extremos y nombres multibyte.
- [ ] **M6-13** Test de truncamiento y bytes extra.
- [ ] **M6-14** Test de estado byte-estable.
- [ ] **M6-15** Test de carga con notas activas (todo se cancela).
- [ ] **M6-16** Verificar que no se guardan sustain, bend, LFO ni reverb.
- [ ] **M6-17** Guardar y reabrir en REAPER y Ableton Live.
- [ ] **M6-18** Documentar migración futura y límites del formato.

**Criterio de salida:** los presets son portátiles, el estado es reproducible y
ningún proyecto se abre “sonando solo”.

---

## 10. M7 — MIDI avanzado

**Objetivo:** comportamiento MIDI completo y probado.
**Gate:** matriz de [TESTING](TESTING.md) §5 y §10 verde.

- [ ] **M7-01** Implementar pitch bend de 14 bits con suavizado.
- [ ] **M7-02** Implementar `pitch_bend_range` 0–24 semitonos.
- [ ] **M7-03** Implementar CC1 mod wheel → tremolo.
- [ ] **M7-04** Implementar CC120, CC121 y CC123.
- [ ] **M7-05** Implementar CC7 volume si el host lo envía.
- [ ] **M7-06** Implementar eventos sample-accurate en todos los canales.
- [ ] **M7-07** Implementar orden estable para eventos con el mismo offset.
- [ ] **M7-08** Test de dos canales con la misma nota.
- [ ] **M7-09** Test de overlapping holds y note-off cruzado.
- [ ] **M7-10** Test de note-on/off en el mismo bloque.
- [ ] **M7-11** Test de bend en centro, extremos y retorno.
- [ ] **M7-12** Test de CC123 con notas sostenidas.
- [ ] **M7-13** Test de host sin offsets (aplicación a inicio de bloque).
- [ ] **M7-14** Documentar limitaciones MIDI aceptadas.
- [ ] **M7-15** Completar la matriz de DAWs obligatoria.

**Criterio de salida:** todos los casos MIDI documentados pasan y las
limitaciones están escritas.

---

## 11. M8 — Sitio y pipeline de release

**Objetivo:** descargas correctas y publicación controlada.
**Gate:** draft revisado, checksums verificados y release publicada a mano.

### 11.1 Sitio

- [ ] **M8-01** Crear `site/` con Astro 7.x y `base: '/khodes'`.
- [ ] **M8-02** Implementar layout base, navegación y footer.
- [ ] **M8-03** Implementar la home con capturas reales.
- [ ] **M8-04** Implementar downloads con releases en build.
- [ ] **M8-05** Implementar installation y troubleshooting.
- [ ] **M8-06** Implementar changelog, license y support.
- [ ] **M8-07** Implementar el cliente `RELEASES_SOURCE=github|offline`.
- [ ] **M8-08** Implementar filtrado de drafts y contrato de nombres.
- [ ] **M8-09** Implementar escape de notas y protección del token.
- [ ] **M8-10** Añadir tests de releases y `check-build.mjs`.
- [ ] **M8-11** Verificar accesibilidad, SEO, sitemap y robots.
- [ ] **M8-12** Añadir el badge de atribución de Slint.

### 11.2 Release

- [ ] **M8-13** Crear `.github/workflows/pages.yml`.
- [ ] **M8-14** Crear `.github/workflows/release.yml` con draft y `APPROVED.txt`.
- [ ] **M8-15** Implementar `VERSION.md` y `THIRDPARTY.json`.
- [ ] **M8-16** Empaquetar ZIPs unsigned para Windows, macOS y Linux.
- [ ] **M8-17** Generar `SHA256SUMS` y `INSTALL.txt`.
- [ ] **M8-18** Verificar validadores sobre los bundles del tag.
- [ ] **M8-19** Publicar `v0.1.0-preview.1` manualmente.
- [ ] **M8-20** Confirmar que el sitio muestra versión y enlaces correctos.

**Criterio de salida:** un usuario puede descargar, instalar y tocar el preview
siguiendo el sitio, sin binarios mezclados ni enlaces rotos.

---

## 12. M9 — Validación y estabilización

**Objetivo:** convertir el preview en una versión estable confiable.
**Gate:** `v0.1.0` publicada con matriz de DAWs completa.

- [ ] **M9-01** Completar la matriz obligatoria de DAWs (REAPER, Ableton Live).
- [ ] **M9-02** Completar la matriz deseable (Bitwig, Cubase, Studio One, FL).
- [ ] **M9-03** Ejecutar `clap-validator` sin fallos ni warnings.
- [ ] **M9-04** Ejecutar `pluginval` strictness 10, incluida GUI.
- [ ] **M9-05** Ejecutar fuzz básico del decodificador de presets.
- [ ] **M9-06** Ejecutar pruebas de robustez de [TESTING](TESTING.md) §12.
- [ ] **M9-07** Medir CPU y memoria en 1, 8, 16 y 32 voces.
- [ ] **M9-08** Resolver regresiones de rendimiento > 10 %.
- [ ] **M9-09** Revisar licencias, avisos y atribución de Slint.
- [ ] **M9-10** Configurar firma y notarización si se distribuye stable firmado.
- [ ] **M9-11** Documentar limitaciones conocidas y problemas abiertos.
- [ ] **M9-12** Actualizar README, ROADMAP, TESTING y PLAN al estado final.
- [ ] **M9-13** Publicar `v0.1.0` con notas completas.
- [ ] **M9-14** Verificar instalación limpia en las tres plataformas.

**Criterio de salida:** `v0.1.0` estable, descargable y verificado; el plan queda
completo salvo tareas explícitamente diferidas.

---

## 13. Trabajo transversal (todas las fases)

Estas tareas no pertenecen a una fase única; se verifican en cada entrega.

- [ ] **X-01** Cada PR actualiza la documentación afectada.
- [ ] **X-02** Cada cambio de sonido crea un corpus nuevo, no reescribe el viejo.
- [ ] **X-03** Cada cambio estructural añade o sustituye un ADR.
- [ ] **X-04** Cada release incluye `LICENSE`, `INSTALL.txt` y avisos.
- [ ] **X-05** `rt-paranoid` se ejecuta en CI y es bloqueante.
- [ ] **X-06** No se commitean artefactos de build, `node_modules` ni `dist`.
- [ ] **X-07** No se anuncia compatibilidad no probada.
- [ ] **X-08** Los IDs de parámetros y las versiones de formato no se reutilizan.
- [ ] **X-09** El editor no abre diálogos de archivo propios.
- [ ] **X-10** La matriz de DAWs se actualiza con cada preview.
- [ ] **X-11** Los enlaces internos de la documentación se verifican antes de cada commit.
- [ ] **X-12** El plan se actualiza al cerrar cada sesión de trabajo.

---

## 14. Definición de “terminado” global

Khodes v1 está terminado cuando:

1. CLAP y VST3 cargan en REAPER y Ableton Live sin crashear.
2. La voz es reconociblemente la de Keys en A/B documentado.
3. 16 voces a 48 kHz usan menos del 10 % de un core en hardware moderno.
4. Guardar y reabrir conserva los 19 parámetros y no reanuda notas.
5. Los presets v1 cargan en toda versión 1.x.
6. `rt-paranoid`, corpus y validadores pasan sobre los bundles del tag.
7. El sitio muestra la versión correcta con enlaces y checksums válidos.
8. Las licencias y avisos están completos.
9. La documentación refleja el producto real.
10. El plan no tiene tareas abiertas salvo las diferidas por decisión explícita.

---

## 15. Registro de avances

Formato: fecha ISO, fase, tareas y nota. Lo más reciente arriba.

### 2026-11-19 — M0 / M1

- Completado M0-01 a M0-20: documentación, ADRs, sitio README, verificación de
  106 enlaces y este plan.
- Completado M1-01 a M1-04: git (`main`), `.gitignore`, `LICENSE`,
  `rust-toolchain.toml`.
- Completado M1-05 a M1-07: Rust 1.92.0, `cargo-truce` 6.3.0 y
  `cargo truce doctor` sin fallos de configuración.
- Completado M1-09 a M1-15 y M1-19 a M1-21: workspace, `truce.toml` validado,
  crates `dsp/`, `kit/` y raíz, `build.rs`, editor Slint mínimo, CI, guard de
  frontera DSP y lockfile con versiones fijadas.
- `cargo check --workspace`, `cargo fmt --all -- --check` y
  `cargo test --workspace` pasan (2 tests de parámetros y defaults).
- Incidencia M1-16: `cargo truce build` usa `x86-64-v3` (AVX2) por defecto y el
  CPU local no lo soporta; fallaba con `SIGILL` en build scripts. Resuelto con
  `--target-cpu v2` y documentado en `DEV_SETUP` §5.1 y troubleshooting.
- Completado M1-16: bundles CLAP y VST3 generados en `target/bundles/` con
  `--target-cpu v2`. Símbolos `clap_entry` y `GetPluginFactory` presentes.
- Completado M1-23 y parte de M1-18: bundles instalados en `~/.clap` y
  `~/.vst3`; el smoke host CLAP carga el bundle, lee el descriptor
  `dev.khodes.khodes v0.1.0-preview.1` y procesa 4 bloques de 256 frames sin
  muestras no finitas. REAPER no está instalado: no se afirma compatibilidad.
- Corrección de contrato: `src/params.rs` usaba mapeo lineal para Brightness;
  ahora usa la curva logarítmica `500 * 40^n` y redondeo en los parámetros
  discretos, según `PARAMETERS.md`. Tests ampliados.
- Completado M1-17: `target/release/khodes-standalone` compila (40 min en esta
  CPU; el perfil release incluye GUI y wgpu).
- Completado M2-01 a M2-18 y M2-20: `dsp::Params`, `dsp::Event` con
  `sample_offset`, PRNG SplitMix64, voz con dos senos, ADSR (con la cola de
  50 ms de `rhodes.ts`), campana, key click, velocity y bend/tune; integrado en
  el wrapper. `cargo test --workspace --release`: 16 tests, 0 fallos
  (3 de parámetros + 13 de DSP), incluidos afinación ±1 cent en MIDI 21/60/69/108.
- Pendiente M2-19: A/B render contra Keys; este entorno no dispone de render
  offline de Web Audio.
- Completado: `cargo fmt --all -- --check`, `cargo clippy --locked --workspace
  --all-targets --all-features --release -- -D warnings` y
  `cargo test --locked --workspace --release --features rt-paranoid`: 16 tests,
  0 fallos. Clippy detectó 3 `field_reassign_with_default` en tests, corregidos
  mediante inicialización con struct update.
- Completado: commit local `bf983e3` de M1+M2 (20 archivos). M1-22 sigue
  pendiente: no hay remote configurado, así que no se afirma CI verde.
- Regenerados bundles CLAP/VST3 con el código M2 y smoke host CLAP verde
  (descriptor `dev.khodes.khodes`, 4 bloques de 256 frames, salida finita).
  Incidencia: el standalone de esa tanda no compiló porque los cambios de M3 se
  introdujeron mientras rustc lo compilaba; se regenerará al cerrar M3.
- M3-01 a M3-16 implementados: `dsp/allocator.rs` con pool fijo de 32 voces,
  robo determinista, cola de desvanecimiento de 4 ms, sustain por canal,
  IDs internos de nota en `src/midi.rs` y eventos CC120/121/123. El motor
  comparte un único buffer de multiplicadores de frecuencia suavizados, así que
  el coste de bend no crece con la polifonía.
- Incidencia M3 detectada por tests: `find_free()` usaba los 32 slots aunque la
  polifonía configurada fuera menor, y `Engine` no propagaba `params.polyphony` al
  allocator. 3 tests fallaban (robo, note-off tardío y acorde de 16 notas).
  Corregido limitando la búsqueda a `polyphony` slots, moviendo a la cola de
  desvanecimiento los slots que quedan fuera al reducir, y propagando el
  parámetro en cada bloque.
- Completado M3: `cargo fmt --all -- --check`, `cargo check --workspace
  --all-targets` y `cargo test --workspace --release`: 22 tests, 0 fallos.
- M3-17: baseline de CPU registrado en [`TESTING.md`](TESTING.md) §11.1: 16
  voces a 48 kHz, bloques de 256 frames, 6,0× tiempo real y 0,8846 ms por
  bloque (1.659 s de CPU para 10.00 s de audio).
- Completado M3: verificación final verde en `/tmp/khodes-m3-final.log` (`EXIT:0`):
  `fmt`, `clippy -D warnings`, 22 tests con `rt-paranoid`, bundles CLAP/VST3
  regenerados, smoke host CLAP correcto (`dev.khodes.khodes`, 4 bloques de 256
  frames, salida finita) y standalone compilado. Bundles instalados en `~/.clap`
  y `~/.vst3`; el smoke también pasa sobre el bundle instalado.
- Siguiente acción: commit de M3 y M4 — bus global y efectos.
- Riesgo vigente: disco con ≈4,5 GB libres.

---

## 16. Decisiones diferidas

| Tema | Estado | Documento |
|---|---|---|
| AU / AAX / VST2 | Fuera de v1 | [ROADMAP](ROADMAP.md) |
| MPE y MIDI 2.0 | Fuera de v1 | [MIDI](MIDI.md) |
| Reverb algorítmica | Diferida | [VOICE_SPEC](VOICE_SPEC.md) |
| Modelos alternativos de piano | Diferido | [ROADMAP](ROADMAP.md) |
| Firma de código | Solo stable | [RELEASES](RELEASES.md) |
| Presets en la nube | Fuera de alcance | [ROADMAP](ROADMAP.md) |
