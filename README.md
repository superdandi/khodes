# Khodes

**Khodes** es un sintetizador de piano eléctrico de tipo *tine* derivado de la
sección de voz del sintetizador **Keys** de **Studio Suite**, empaquetado como
plugin de audio **CLAP + VST3** para usarse en cualquier DAW sobre macOS,
Windows y Linux.

> **Estado del proyecto:** fases M1–M6 implementadas. M4 cerrado con corpus
> `contract/v1/` congelado (13 fixtures) y verificado; M5 cerrado con editor
> Slint accesible y cuatro baselines en `screenshots/`; M6 cerrado con estado
> del host vía TRUCE, códec `.khodespreset` v1 y 6 presets de fábrica. El
> workspace compila, existen bundles CLAP y VST3, el standalone de desarrollo
> compila y el smoke host CLAP verifica la carga y el ciclo de estado. El DSP
> incluye la voz de Keys, un allocator polifónico de hasta 32 voces y el bus de
> efectos (drive, brillo, EQ, chorus, tremolo, reverb convolucional y master
> estéreo): 64 tests verdes. Aún no está implementado el sitio. Pendientes:
> optimizar el consumo de CPU de la reverb cuando se activa (M4-22), validar en
> DAW real (incluido guardar/reabrir, M6-17), ejecutar CI con remote y hacer el
> A/B contra Keys. El avance tarea por tarea se marca en
> [`docs/PLAN.md`](docs/PLAN.md).

## Origen y referencias

| Fuente | Ubicación | Qué aporta |
|---|---|---|
| Voz original | `/home/dandi/studio-suite/src/components/keys/RhodesControls.tsx` | Controles, rangos, defaults y presentación de la voz |
| Motor original | `/home/dandi/studio-suite/src/lib/rhodes.ts` | Grafo Web Audio, síntesis, envelope y efectos |
| Reproducción | `/home/dandi/studio-suite/src/hooks/useScalePlayer.ts` | Disparo de notas, escalas y piezas |
| Documentación previa | `/home/dandi/studio-suite/DOCUMENTACION.md` | Decisiones y correcciones de la voz Rhodes |
| Referencia de ingeniería | [`piducancore/buffer-uppercut`](https://github.com/piducancore/buffer-uppercut) | Stack Rust + TRUCE + Slint, arquitectura, releases y sitio |
| Sitio de referencia | <https://piducan.dev/buffer-uppercut/> | Modelo de páginas, descargas y versionado |

La referencia **Buffer Uppercut** es un efecto, no un sintetizador. De ella se
adopta la **tecnología de desarrollo y el proceso de producto**, no su DSP ni su
modelo de parámetros. Khodes es un instrumento: recibe MIDI y genera audio.

## Decisiones fundacionales

| Decisión | Valor |
|---|---|
| Nombre de producto | Khodes |
| Tipo | Instrumento sintetizador (MIDI in, audio out) |
| Formatos soportados | CLAP y VST3 |
| Host de desarrollo | Standalone interno, compilado en CI, no distribuido |
| Lenguaje | Rust (edition 2024) |
| Framework de plugin | TRUCE 6.3 |
| UI | Slint 1.15.x, editor nativo |
| DSP | Núcleo propio en `dsp/`, sin dependencias de TRUCE/Slint |
| Presets | Códec nativo versionado `.khodespreset` + presets del host |
| Sitio | Astro estático + GitHub Pages + GitHub Releases |
| Versionado | `vMAJOR.MINOR.PATCH-preview.N` para previews |
| Licencia del código original | MIT |

## Principios de diseño

1. **Fidelidad a la voz de Keys.** Khodes debe sonar como la voz Rhodes de
   Studio Suite antes de sonar “mejor”. Cualquier desviación debe ser
   deliberada, medida y documentada.
2. **Tiempo real primero.** `process` no asigna memoria, no bloquea, no registra
   logs, no toca disco y no hace trabajo no acotado.
3. **Contratos antes que código.** Parámetros, MIDI, estado y formato de preset
   son contratos estables. Cambiarlos exige actualizar código, tests, docs y un
   ADR.
4. **DSP y UI separados.** El motor no conoce TRUCE, Slint, el host ni el sistema
   de archivos. La UI no procesa audio.
5. **Estado versionado.** El preset nativo y el estado del host son datos
   durables; nunca reanudan notas sostenidas ni estado transitorio.
6. **Documentación mantenida.** Cada dato vive en un solo documento autoritativo
   y los demás lo enlazan.

## Mapa del repositorio

```text
khodes/
├── README.md                  Este documento
├── AGENTS.md                  Reglas para agentes y colaboradores
├── Cargo.toml                 Espacio de trabajo y crate raíz del plugin
├── Cargo.lock                 Versiones fijadas (TRUCE 6.3.0, Slint 1.15.1)
├── rust-toolchain.toml        Rust 1.92.0
├── build.rs                   Compilación de la UI Slint
├── truce.toml                 Identidad, formatos y categoría del plugin
├── .github/workflows/         CI de build, tests y validadores
├── dsp/                       Motor de audio puro (voz monofónica implementada)
├── kit/                       Presets y códec .khodespreset (esqueleto)
├── examples/screenshots.rs    Generador de baselines del editor
├── src/                       Integración TRUCE, parámetros y editor
├── ui/                        Componentes Slint (panel Khodes)
├── tools/                     Guarda de frontera DSP y smoke host CLAP
├── contract/                  Corpus de regresión DSP (pendiente)
├── screenshots/               Baselines del editor (default, mínimo, wide, activo)
├── docs/                      Documentación técnica y plan
└── site/                      Sitio Astro de producto (pendiente)
```

## Documentación

| Documento | Contenido |
|---|---|
| [VOICE_SPEC](docs/VOICE_SPEC.md) | Especificación exhaustiva de la voz heredada de Keys |
| [ARCHITECTURE](docs/ARCHITECTURE.md) | Capas, crates, flujo de audio y concurrencia |
| [PARAMETERS](docs/PARAMETERS.md) | Esquema de parámetros, IDs, rangos y automatización |
| [MIDI](docs/MIDI.md) | Notas, velocity, sustain, pitch bend y asignaciones |
| [STATE_AND_PRESETS](docs/STATE_AND_PRESETS.md) | Estado del host, presets de fábrica y códec nativo |
| [UI_SPEC](docs/UI_SPEC.md) | Diseño del editor Slint y lenguaje visual |
| [REFERENCE_BUFFER_UPPERCUT](docs/REFERENCE_BUFFER_UPPERCUT.md) | Análisis del plugin y proceso de referencia |
| [DEV_SETUP](docs/DEV_SETUP.md) | Toolchain, build, instalación y desarrollo |
| [TESTING](docs/TESTING.md) | Pruebas, validadores y aceptación en DAW |
| [RELEASES](docs/RELEASES.md) | Versionado, tags, drafts y publicación |
| [SITE](docs/SITE.md) | Sitio de producto, descargas y GitHub Pages |
| [LICENSING](docs/LICENSING.md) | MIT, TRUCE, Slint y obligaciones de terceros |
| [ROADMAP](docs/ROADMAP.md) | Fases, entregables y alcance diferido |
| [PLAN](docs/PLAN.md) | **Plan maestro ejecutable: tareas, gates y registro de avance** |
| [CONTRACTS](docs/CONTRACTS.md) | Contratos duros e invariantes del producto |
| [ADR](docs/adr/README.md) | Registro de decisiones de arquitectura |
| [Site README](site/README.md) | Detalle operativo del sitio |

### Orden de lectura recomendado

1. `README.md` (este documento).
2. `docs/VOICE_SPEC.md` para entender qué se va a construir.
3. `docs/ARCHITECTURE.md` para entender cómo se organiza.
4. `docs/PARAMETERS.md`, `docs/MIDI.md` y `docs/STATE_AND_PRESETS.md` para los
   contratos del producto.
5. `docs/REFERENCE_BUFFER_UPPERCUT.md` para el proceso de ingeniería.
6. `docs/DEV_SETUP.md`, `docs/TESTING.md` y `docs/RELEASES.md` para ejecutar.
7. `docs/PLAN.md` para saber **qué toca ahora** y qué está bloqueado.

## Alcance de la primera versión

Incluido:

- Voz de piano eléctrico tine con fundamental, parcial de campana y key click.
- Envelope Attack / Decay / Sustain / Release.
- Cadena global: drive, brillo, EQ de graves/agudos, chorus, tremolo y reverb.
- Polifonía configurable con *voice stealing* determinista.
- MIDI: notas, velocity, sustain, pitch bend y mod wheel.
- CLAP y VST3; estado del host y presets nativos versionados.
- Editor Slint con la estética HUD de Keys.
- Sitio estático con descargas desde GitHub Releases.

Fuera de alcance inicial (ver [ROADMAP](docs/ROADMAP.md)):

- AU, AAX, VST2.
- MPE y MIDI 2.0.
- Samples multimuestra o modelado físico completo.
- Secuenciador, arpegiador o escalas integradas.
- Presets en la nube o cuentas de usuario.

## Convenciones de esta documentación

- `[Keys]` identifica comportamiento verificado en Studio Suite.
- `[Khodes]` identifica una decisión o propuesta para este proyecto.
- `[Referencia]` identifica comportamiento del repo Buffer Uppercut.
- Los valores numéricos se expresan con unidades explícitas.
- Los IDs de parámetros y versiones de formato son contratos: no se renumeran.

## Licencia

El código original de Khodes se publicará bajo licencia **MIT**. Las
dependencias conservan sus propias licencias; en particular TRUCE y Slint no son
MIT. Ver [LICENSING](docs/LICENSING.md) antes de distribuir binarios.
