# Referencia de ingeniería — Buffer Uppercut (piducancore)

Este documento analiza el plugin público que sirve de referencia tecnológica
para Khodes. No es documentación oficial del proyecto de Piducan; es una lectura
del repositorio y del sitio publicados, con foco en lo que Khodes debe adoptar y
lo que no.

- Repositorio: <https://github.com/piducancore/buffer-uppercut>
- Sitio: <https://piducan.dev/buffer-uppercut/>
- Licencia del código original: MIT
- Versión observada: `0.1.0-preview.6` (tag `v0.1.0-preview.6`)

---

## 1. Identidad del producto

| Campo | Valor |
|---|---|
| Nombre | Buffer Uppercut |
| Vendor | `piducancore` |
| Crate/paquete | `buffer-uppercut` |
| Bundle slug | `buffer-uppercut` |
| Tipo | Efecto de audio estéreo/mono con entrada MIDI |
| Formatos | CLAP y VST3 |
| Host de desarrollo | Standalone interno, compilado en CI, no distribuido |
| Categoría VST3 | `Delay` |
| FourCC | `BUTr` |

La identidad vive en `truce.toml`; no se duplica en el código.

---

## 2. Stack tecnológico

| Componente | Versión / elección | Papel |
|---|---|---|
| Rust | 1.92+ | Lenguaje del plugin y DSP |
| Edition | 2024 | Edición de Cargo |
| TRUCE | 6.3 | Framework de plugin y wrappers CLAP/VST3 |
| `cargo-truce` | 6.3.0 | Build, run, install, screenshot |
| Slint | 1.15.1 | Editor nativo |
| TRUCE Slint | 6.3 | Puente TRUCE ↔ Slint |
| Astro | 7.3.2 | Sitio estático |
| Node | >=22.12.0 | Build del sitio |

### 2.1 Rasgos arquitectónicos

- Núcleo DSP en `dsp/` sin dependencias de TRUCE, Slint, host o filesystem.
- Códec de presets en `kit/` sin UI ni automatización.
- Integración de producto en `src/`.
- UI declarativa en `ui/`.
- Sitio en `site/`, desplegado aparte del binario.
- Corpus de regresión de audio en `contract/`.
- Backports mínimos de framework en `vendor/`, cada uno con `PATCH.md`.

---

## 3. Mapa del repositorio

```text
buffer-uppercut/
├── Cargo.toml
├── Cargo.lock
├── truce.toml
├── build.rs
├── AGENTS.md
├── README.md
├── LICENSE
├── dsp/                 Motor de audio puro
├── kit/                 Kits de fábrica y códec .bupreset v3
├── src/                 PluginLogic, parámetros, MIDI, editor
├── ui/                  Componentes Slint
├── site/                Astro estático
├── contract/            Corpus DSP v1–v4 + SHA256SUMS
├── screenshots/         Baselines del editor
├── design/              Material de diseño
├── docs/                Arquitectura, contratos, testing, ADRs
├── tools/               Utilidades (notices, etc.)
└── vendor/              Backports de TRUCE/Slint
```

### 3.1 Propiedad por módulo

| Ruta | Posee |
|---|---|
| `dsp/` | Motor serial, estado por slot, admisión, efectos, visualización |
| `kit/` | Kits de fábrica, códec `.bupreset` |
| `src/lib.rs` | Ciclo de vida, `process`, transporte, eventos, agregación |
| `src/params.rs` | Esquema de parámetros, metadata, publicación atómica |
| `src/midi.rs` | Mapeo de notas y máscaras por canal |
| `src/keyboard.rs` | Conversión de teclas físicas |
| `src/input.rs` | Ownership de teclado/puntero y gestos |
| `src/editor.rs` | Bindings Slint, gestos de automatización, kits |
| `ui/` | Layout y componentes visuales |
| `site/` | Páginas estáticas y metadata de releases |
| `vendor/` | Parches estrechos de framework |

---

## 4. Arquitectura de audio

Buffer Uppercut es un efecto con 16 *performance pads*. Cada pad puede tener un
tipo de efecto y siete macros. Los pads se apilan en serie en orden de presión.

Flujo simplificado:

```text
Host audio + MIDI + transporte + automatización
        │
        ▼
TRUCE PluginLogic64 (src/lib.rs)
        │
        ├── src/midi.rs
        ├── src/params.rs
        └── wrapper I/O preasignado
        │
        ▼
DSP Engine (dsp/src/lib.rs)
        │
        ▼
snapshot atómico de visualización
        │
        ▼
Slint (src/editor.rs + ui/main.slint)
```

### 4.1 Procesamiento por bloque

1. Leer tempo del host.
2. Aplicar eventos MIDI a 16 máscaras de held independientes.
3. Agregar MIDI y triggers automatizables.
4. Detectar roles de Pitch.
5. Copiar entrada a scratch planar `f64`.
6. Resolver slots activos, retenidos y suspendidos bajo un tope de 6.
7. Procesar etapas activas en orden de presión.
8. Grabar historial de buffers por slot.
9. Publicar snapshot de visualización a 30 Hz.
10. Flush de subnormales y copia a la salida.

### 4.2 Modelo serial y admisión

- Cada efecto continuo retenido es una etapa independiente.
- Se permiten hasta 6 procesadores continuos activos.
- Al llegar a 8, se suspende la etapa activa más antigua.
- Las etapas suspendidas conservan su estado y se restauran por orden de retención
  más reciente.
- El orden audible es el orden de presión original.
- Los roles de acción (Pitch Down/Up) no entran a la cadena.

### 4.3 Historial de buffers

- 16 historiales estéreo `f32` de al menos 8 segundos.
- La señal de proceso es `f64`; la conversión ocurre solo en el límite del
  historial.
- No se asigna ni redimensiona en `process`.
- El historial sigue grabando aunque la etapa esté suspendida.

---

## 5. Parámetros y MIDI

### 5.1 Parámetros

- Exactamente **144 parámetros automatizables**, IDs `1..144`.
- ID `0` reservado.
- 16 slots × 9 parámetros:
  - trigger momentáneo;
  - tipo de efecto;
  - 7 macros normalizados.
- Para el slot `p` (0-based):

```text
trigger = 1 + p * 9
type    = trigger + 1
macro n = trigger + 2 + n, n in 0..6
```

- Tipos: Off, Beat Repeat, Reverse, Tape Stop, Gate, Pitch, Filter, LoFi,
  Vinyl.
- Los macros se muestran con formato semántico (divisiones, %, semitonos, dB,
  ms, Hz, bits).

### 5.2 MIDI

- Notas `60..75` disparan los 16 pads.
- Roles de Pitch aceptan también `57..59` y `81..83`.
- El estado MIDI se mantiene independientemente por canal.
- Teclado físico por posición, no por carácter: `1234/QWER/ASDF/ZXCV`.
- Learn / Clear / Reset Layout / swap de asignaciones.
- Los eventos se aplican a borde de bloque; no hay segmentación sample-accurate.

---

## 6. Estado y presets

### 6.1 Estado del host

- `BufferUppercutParams` es la fuente de verdad.
- 144 parámetros + metadata de nombre de kit y mapa de teclas.
- Los holds recordados se suprimen para no reanudar una performance guardada.
- El estado del host y los kits nativos restauran la misma configuración durable.

### 6.2 `.bupreset` v3

Layout documentado:

1. magic `BUPRESET` (8 bytes);
2. `u32` versión, slot count, macro count, longitud del nombre;
3. hasta 63 bytes UTF-8 de nombre;
4. por cada uno de 16 slots: `u32` tipo + 7 `f64` normalizados;
5. 16 bytes de identificadores de tecla física.

Validación:

- rechaza versiones/layouts no soportados;
- rechaza UTF-8 inválido;
- rechaza valores no finitos o fuera de rango;
- rechaza truncamiento y bytes extra;
- límite de 4096 bytes.

Versiones 1 y 2 se rechazan deliberadamente. La versión 3 añade el mapa de
teclas.

### 6.3 Kits de fábrica

Cinco kits, entre ellos **Classic**, **Glitch Grid**, **Tape Lab**,
**Filter & Pitch** y **Vinyl Cuts**.

---

## 7. Sitio de producto

### 7.1 Stack

- Astro 7.3.2, `output: 'static'`.
- `site: 'https://piducan.dev'`.
- `base: '/buffer-uppercut'`.
- `trailingSlash: 'always'`.
- Node >= 22.12.0.

### 7.2 Páginas observadas

| Ruta | Archivo | Contenido |
|---|---|---|
| `/buffer-uppercut/` | `src/pages/index.astro` | Home del producto |
| `/buffer-uppercut/downloads/` | `src/pages/downloads.astro` | Descargas por release y plataforma |
| `/buffer-uppercut/installation/` | `src/pages/installation.astro` | Instalación y requisitos |

Además: `src/layouts/`, `src/lib/`, `src/styles/`, `src/assets/`, `scripts/`,
`tests/`.

### 7.3 Descargas

- GitHub Releases es la fuente de verdad.
- Los drafts nunca aparecen.
- La release más nueva con paquetes elegibles alimenta los botones.
- Plataformas ausentes en esa release quedan no disponibles; no se mezclan
  versiones.
- Notas de release escapadas como texto plano, no HTML no confiable.

Contrato inicial de nombres:

```text
Buffer-Uppercut-VERSION-windows-x64-unsigned.zip
Buffer-Uppercut-VERSION-macos-unsigned.zip
Buffer-Uppercut-VERSION-linux-x64-unsigned.zip
SHA256SUMS
```

Solo esos archivos explícitamente unsigned generan botones. Se excluyen
candidates internos, standalone y estados de firma desconocidos.

### 7.4 Build y despliegue

```sh
cd site
npm ci
npm run dev
npm test
npm run build
npm run test:build
```

- Build local: lista de releases vacía por defecto.
- Sync real: `RELEASES_SOURCE=github npm run build`.
- `GITHUB_TOKEN` solo en build; nunca se renderiza.
- Fallo de API falla el build de producción; no se vacía el sitio en silencio.
- GitHub Pages se publica con GitHub Actions.
- El workflow `pages.yml` corre en push a `main` (paths `site/**`,
  `screenshots/**`), PR, release published/edited/deleted, manual y cron diario.
- Los PR solo construyen; no despliegan.

---

## 8. Sistema de versiones

### 8.1 Versión del producto

- Cargo: `0.1.0-preview.6`.
- Tag: `v0.1.0-preview.6`.
- Releases: prerelease, unsigned.
- Formato: `vMAJOR.MINOR.PATCH-preview.N`.
- La versión de producto es independiente de la versión del códec `.bupreset`
  (v3) y de la versión del corpus DSP (v4).

### 8.2 Flujo de release

1. Actualizar versión en Cargo y lockfile.
2. Escribir `VERSION.md` con arquitecturas, requisitos, revisión fuente,
   instalación, limitaciones y problemas conocidos.
3. Generar `THIRDPARTY.json` con `cargo-bundle-licenses 4.2.0`.
4. Revisar licencias y recursos embebidos.
5. Registrar smoke tests en DAW reales por plataforma y formato.
6. Añadir la versión exacta a `APPROVED.txt`.
7. Crear commit y tag `v...`.
8. El workflow `release.yml` valida, empaqueta y crea un **draft prerelease**.
9. Revisar el draft, descargar assets, verificar `SHA256SUMS`.
10. Publicar manualmente (`gh release edit --draft=false --prerelease`).
11. Disparar o esperar el workflow del sitio.
12. Verificar que el sitio muestra la nueva versión y los tres enlaces.

El workflow nunca publica solo ni sobrescribe una release publicada. La
aprobación es un archivo versionado (`APPROVED.txt`), no un botón de CI.

### 8.3 Firma

- Previews sin firma: sin Authenticode, sin Developer ID, sin notarización.
- macOS puede requerir pasos manuales; se documenta en `INSTALL.txt`.
- `SHA256SUMS` verifica integridad, no identidad del editor.
- La firma y la promoción a stable son pasos separados.

---

## 9. Testing

Niveles:

1. `cargo test --locked --workspace`.
2. `cargo fmt`, `clippy -D warnings`.
3. `cargo test --features rt-paranoid`.
4. Corpus `contract/v1..v4` con checksums.
5. `clap-validator` y `pluginval`.
6. Screenshots Slint en tamaños default, mínimo, wide, capturado, suspendido,
   Vinyl.
7. Matriz de DAWs: REAPER, Ableton Live, etc.
8. Benchmarks de CPU y memoria.

Definición de done:

- tests enfocados;
- workspace, Clippy, fmt y `rt-paranoid` verdes;
- formatos anunciados compilan;
- corpus y checksums pasan;
- screenshots/DAW cubren cambios visibles;
- contratos, arquitectura, roadmap y ADRs actualizados;
- sin artefactos de build commiteados.

---

## 10. Licencias

- Código original: MIT.
- TRUCE 6.3: `LicenseRef-TruceLicense-1.0`; permite distribuir plugins bajo su
  concesión, pero no se etiqueta como MIT.
- Slint 1.15.1: GPL-3.0-only, royalty-free 2.0 o comercial. El plugin usa la
  opción royalty-free y muestra el badge de atribución de Slint.
- Dependencias transitivas, fuentes y componentes nativos pueden exigir avisos
  adicionales.
- Cada ZIP público debe incluir `LICENSE`, instrucciones y avisos revisados.

---

## 11. Qué adopta Khodes

| Práctica | Aplicación en Khodes |
|---|---|
| Rust + TRUCE 6.3 + Slint | Stack base |
| `dsp/` puro | Núcleo de síntesis y bus |
| `kit/` separado | Presets y `.khodespreset` |
| `src/` como integración | Parámetros, MIDI, editor |
| `ui/` declarativo | Editor HUD |
| `contract/` versionado | Regresión de audio |
| `rt-paranoid` | Seguridad en tiempo real |
| Validadores CLAP/VST3 | Gate de release |
| Sitio Astro estático | Páginas de Khodes |
| Releases como fuente de verdad | Descargas |
| Versión preview + draft manual | Proceso de release |
| ADRs | Decisiones trazables |
| `AGENTS.md` | Reglas para agentes |

## 12. Qué no se copia

| Elemento | Motivo |
|---|---|
| DSP de efectos de buffer | Khodes es un instrumento, no un efecto |
| 16 pads y 144 parámetros | Modelo de producto distinto |
| Admisión serial de 6 etapas | No aplica a voces de sintetizador |
| `.bupreset` | Khodes usa `.khodespreset` |
| Textos y diseño visual | Khodes tiene su propia identidad |
| Código C++/WRAC antiguo | La propia referencia lo declara no soportado |

## 13. Lecciones para Khodes

1. **Separar el DSP del framework desde el día uno.** Es lo que permite testear
   sin host y portar a otros formatos.
2. **Fijar los parámetros como contrato.** La numeración estable evita romper
   proyectos de usuarios.
3. **Versionar el sonido.** Un corpus congelado evita “arreglos” silenciosos.
4. **No publicar desde CI.** El draft revisado es una barrera real.
5. **El sitio es estático y lee releases en build.** Menos superficie, menos
   secretos, menos fallos.
6. **La documentación es parte del producto.** Contratos, ADRs y testing viven
   junto al código.
7. **La licencia no es solo el LICENSE raíz.** TRUCE y Slint tienen condiciones
   propias.
