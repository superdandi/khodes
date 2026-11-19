# 0001 — Stack Rust + TRUCE + Slint

- **Estado:** Aceptado
- **Fecha:** 2026-11-19
- **Decisores:** equipo Khodes
- **Relacionado:** [0003](0003-dsp-ui-separation.md), [REFERENCE_BUFFER_UPPERCUT](../REFERENCE_BUFFER_UPPERCUT.md)

---

## Contexto

Khodes nace como un plugin de instrumento derivado de la voz Rhodes de Keys
(Studio Suite). El objetivo es distribuirlo como **CLAP y VST3** en macOS,
Windows y Linux, con un editor propio y un proceso de release reproducible.

Existen varios caminos técnicos:

- un framework C++ (JUCE, iPlug2) con DSP en C++;
- un framework Rust con wrappers generados;
- Web Audio embebido en un contenedor;
- un desarrollo desde cero contra las APIs CLAP y VST3.

El proyecto de referencia, **Buffer Uppercut** (`piducancore`), ya resolvió este
problema con un stack concreto: Rust 1.92+, TRUCE 6.3 para los wrappers de
plugin, Slint 1.15.x para el editor y Astro para el sitio. Su código es MIT y su
arquitectura está documentada públicamente.

Las restricciones reales son:

1. El equipo escribe Rust con comodidad y quiere un núcleo DSP testeable sin
   host.
2. El editor debe ser nativo, no una web view, para reducir dependencias y
   latencia de UI.
3. Se necesitan CLAP y VST3 desde el primer preview, sin mantener dos
   implementaciones de la lógica de producto.
4. El presupuesto de tiempo no permite construir wrappers de formato desde cero.
5. La licencia del plugin debe permitir distribución gratuita y comercial sin
   obligar a abrir el código propio.

---

## Decisión

Khodes usará:

| Capa | Tecnología | Versión objetivo |
|---|---|---|
| Lenguaje | Rust (edition 2024) | 1.92+ |
| Wrappers de plugin | TRUCE | 6.3 |
| Build/run/install | `cargo-truce` | 6.3.0 |
| Editor | Slint | 1.15.x |
| Puente editor | TRUCE Slint | 6.3 |
| Sitio | Astro estático | 7.x |
| Node (sitio) | Node.js | >= 22.12.0 |

Reglas asociadas:

- La identidad del plugin vive en `truce.toml`, no en el código.
- `src/` es el único lugar que conoce TRUCE.
- `dsp/` y `kit/` compilan y se testean sin TRUCE.
- El standalone es un host de desarrollo interno; no se distribuye.
- Las versiones de TRUCE y Slint se fijan exactamente y se actualizan mediante
  un ADR nuevo.

---

## Consecuencias

### Positivas

- Un solo núcleo de producto genera CLAP y VST3 mediante feature flags.
- El DSP se puede testear con `cargo test` sin abrir un DAW.
- El editor Slint se compila dentro del binario, sin servidor local ni web view.
- El modelo de `PluginLogic`, parámetros y estado de TRUCE ya está probado en un
  plugin publicado.
- Se hereda un proceso de build, validación y release conocido.
- El equipo evita mantener código específico de CLAP y de VST3.

### Negativas

- TRUCE y Slint **no son MIT**: imponen obligaciones de licencia y atribución
  (ver [LICENSING](../LICENSING.md)).
- El ecosistema es más pequeño que JUCE; menos ejemplos y menos contratación.
- Slint añade peso de compilación y requiere cuidado con el hilo de UI.
- Cualquier limitación de TRUCE (por ejemplo, eventos MIDI a borde de bloque) se
  convierte en trabajo propio o en un *backport* en `vendor/`.
- La versión de Rust y de las dependencias queda relativamente acoplada al
  framework.

### Neutrales

- El sitio es independiente del stack del plugin.
- El códec de presets es propio y no depende de TRUCE.
- Se puede añadir AU, AAX o VST2 después con features adicionales, sin reescribir
  el DSP.

---

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|---|---|
| JUCE (C++) | Otro lenguaje para el equipo; licencia y tamaño; sin ventaja clara para este alcance |
| iPlug2 | Menos maduro para CLAP; más trabajo de integración |
| nih-plug (Rust) | Buena opción, pero sin editor nativo integrado ni el modelo de estado/UI que ya usa la referencia |
| Web Audio + contenedor | Latencia, ciclo de vida y empaquetado nativo problemáticos |
| CLAP/VST3 desde cero | Coste alto y riesgo de bugs de formato; no aporta al producto |
| Solo VST3 en v1 | CLAP es un objetivo de producto y TRUCE lo da casi gratis |

---

## Implementación

- [ ] `truce.toml` con nombre, vendor, bundle, categoría y formatos.
- [ ] `Cargo.toml` con `clap`, `vst3` y `standalone` como features.
- [ ] Workspace con `dsp/`, `kit/`, `src/` y `ui/`.
- [ ] `cargo truce doctor` en CI antes de compilar.
- [ ] CI con `cargo test --locked --workspace`, fmt, Clippy y `rt-paranoid`.
- [ ] Documentar la versión exacta de TRUCE y Slint en `Cargo.lock`.

---

## Referencias

- [Buffer Uppercut — Cargo.toml](https://github.com/piducancore/buffer-uppercut/blob/main/Cargo.toml)
- [Buffer Uppercut — truce.toml](https://github.com/piducancore/buffer-uppercut/blob/main/truce.toml)
- [TRUCE](https://github.com/truce-audio/truce)
- [Slint](https://slint.dev/)
- [`../ARCHITECTURE.md`](../ARCHITECTURE.md)
- [`../DEV_SETUP.md`](../DEV_SETUP.md)
