# Desarrollo y toolchain — Khodes

**Estado:** guía objetivo. Los comandos de build aplican cuando exista el
esqueleto Rust del plugin.

---

## 1. Prerrequisitos

| Herramienta | Versión mínima | Uso |
|---|---|---|
| Rust | 1.92 | Plugin, DSP y tests |
| `cargo-truce` | 6.3.0 | Build/run/install/screenshot de TRUCE |
| Node.js | 22.12.0 | Sitio Astro |
| npm | incluido con Node | Dependencias del sitio |
| Git | reciente | Control de versiones |
| Xcode Command Line Tools | macOS | Compilación nativa |
| Bibliotecas de audio/ventana | Linux | ALSA, X11, fontconfig, udev, etc. |
| Visual Studio Build Tools | Windows | Compilación MSVC |

### 1.1 macOS

```sh
xcode-select --install
```

### 1.2 Linux (Debian/Ubuntu, orientativo)

```sh
sudo apt install build-essential pkg-config libasound2-dev libx11-dev \
  libxkbcommon-dev libfontconfig1-dev libudev-dev
```

Los nombres exactos dependen de la distribución y de la versión de TRUCE/Slint.

### 1.3 Windows

- Instalar Rust con `rustup` usando el toolchain MSVC.
- Instalar Visual Studio Build Tools con “Desktop development with C++”.
- Asegurar que `cargo` y `rustc` estén en el `PATH`.

---

## 2. Instalación de Rust y TRUCE

```sh
rustup update stable
rustup default stable
cargo install cargo-truce --version 6.3.0 --locked
```

Verificar:

```sh
rustc --version
cargo --version
cargo truce --version
cargo truce doctor
```

`cargo truce doctor` debe reportar OK para el toolchain y las dependencias de
plataforma. Si falla, no continuar con el build.

---

## 3. Estructura de trabajo

Cuando el esqueleto exista:

```text
khodes/
├── Cargo.toml
├── truce.toml
├── build.rs
├── dsp/
├── kit/
├── src/
├── ui/
├── contract/
├── screenshots/
├── docs/
└── site/
```

Reglas:

- `dsp/` compila y testea sin TRUCE ni Slint.
- `kit/` compila sin host ni UI.
- `src/` es el único lugar que conoce TRUCE.
- `ui/` solo declara Slint.
- `site/` es un proyecto Node independiente.

---

## 4. Ciclo de desarrollo

```sh
cargo truce doctor
cargo test --locked --workspace
cargo truce run
```

`cargo truce run` levanta el host standalone de desarrollo. Es la forma más
rápida de inspeccionar UI, DSP y MIDI sin abrir un DAW.

### 4.1 Checks antes de entregar

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace
cargo test --locked --workspace --features rt-paranoid
cargo truce build --clap --vst3
```

### 4.2 Tests por área

```sh
cargo test -p khodes-dsp
cargo test -p khodes-kit
cargo test -p khodes-dsp --features rt-paranoid
cargo test -p khodes-dsp --test contract
```

Los nombres exactos de los crates se fijarán en `Cargo.toml`.

---

## 5. Build e instalación

```sh
cargo truce build --clap
cargo truce build --vst3
cargo truce build --clap --vst3
cargo truce install --clap --vst3
```

### 5.1 CPU objetivo en x86_64

Por defecto, `cargo truce build` compila con `-C target-cpu=x86-64-v3`
(AVX2 + FMA + BMI2). Es correcto para los bundles publicados y para CI, pero
falla en CPUs antiguas sin AVX2 con `SIGILL: illegal instruction` al ejecutar
los build scripts.

Para desarrollo en una máquina así:

```sh
cargo truce build --clap --vst3 --target-cpu v2
# o, solo para pruebas locales:
cargo truce build --clap --vst3 --target-cpu native
```

No usar `native` para artefactos distribuidos: el binario deja de ser portable
a otras CPUs.

Artefactos:

```text
target/bundles/Khodes.clap
target/bundles/Khodes.vst3
```

Instalación macOS:

```text
~/Library/Audio/Plug-Ins/CLAP/Khodes.clap
~/Library/Audio/Plug-Ins/VST3/Khodes.vst3
```

Instalación Linux (orientativa):

```text
~/.clap/Khodes.clap
~/.vst3/Khodes.vst3
```

Instalación Windows (orientativa):

```text
%COMMONPROGRAMFILES%\CLAP\Khodes.clap
%COMMONPROGRAMFILES%\VST3\Khodes.vst3
```

Cerrar y reabrir el DAW después de instalar: muchos hosts mantienen el binario
cargado durante toda la sesión.

---

## 6. Standalone de desarrollo

```sh
cargo build --locked --release --bin khodes-standalone
```

o

```sh
cargo truce run
```

El standalone:

- permite elegir dispositivo de audio y MIDI;
- muestra la UI;
- no se distribuye;
- no se incluye en releases públicas;
- se compila en CI para detectar roturas.

---

## 7. Editor Slint

```sh
cargo truce run
cargo truce screenshot --out /tmp/khodes.png --debug --scale 1
```

Baselines comprometidos (cuando existan):

```sh
cargo truce screenshot --out screenshots/slint-default.png --check --debug --scale 1
KHODES_EDITOR_SIZE=920x620 cargo truce screenshot --out screenshots/slint-minimum.png --check --debug --scale 1
KHODES_EDITOR_SIZE=1440x760 cargo truce screenshot --out screenshots/slint-wide.png --check --debug --scale 1
```

Reglas:

- Revisar visualmente el render antes de actualizar un baseline.
- No commitear screenshots temporales.
- Los cambios de layout deben cubrir default, mínimo y wide.

---

## 8. Sitio

```sh
cd site
npm ci
npm run dev
npm test
npm run build
npm run test:build
```

- `npm run dev` sirve el sitio con base `/khodes/`.
- `npm run build` genera `site/dist/`.
- `RELEASES_SOURCE=github npm run build` sincroniza releases reales.
- `GITHUB_TOKEN` solo se usa en build.
- No commitear `site/dist/` ni `node_modules/`.

---

## 9. Convenciones de código

- Rust edition 2024.
- `rustfmt` obligatorio.
- Clippy con `-D warnings`.
- Sin `unwrap()` en rutas de audio; usar manejo explícito.
- Sin `println!` en `process`.
- Nombres de parámetros en `snake_case`.
- Nombres de archivos y módulos en inglés.
- Comentarios y documentación en español.
- Tests junto al módulo o en `tests/`.

---

## 10. Troubleshooting

| Problema | Causa probable | Solución |
|---|---|---|
| `cargo truce doctor` falla | Faltan libs de plataforma | Instalar dependencias del sistema |
| El DAW no ve el plugin | Bundle no instalado o DAW cacheado | `cargo truce install`, cerrar/reabrir DAW |
| El editor no refleja cambios | Slint/Rust no recompilado | Rebuild y cerrar/reabrir editor |
| El plugin crashea al cargar | Estado o preset inválido | Probar con estado limpio; revisar validación |
| Sonido con clics | Sin smoothing o robo sin fade | Revisar [VOICE_SPEC](VOICE_SPEC.md) y tests |
| CPU alta | Reverb o polyphony excesiva | Medir con benchmark; bajar polyphony |
| `SIGILL` en build scripts | CPU sin AVX2 y target `x86-64-v3` por defecto | `cargo truce build --target-cpu v2` |
| El sitio no muestra descargas | Releases en draft o sin assets | Publicar release; revisar workflow |
| El sitio falla el build | API de GitHub o token | Revisar logs; no silenciar el error |

---

## 11. Variables de entorno útiles

| Variable | Uso |
|---|---|
| `KHODES_EDITOR_SIZE` | Tamaño del editor para screenshots |
| `KHODES_EDITOR_PREVIEW` | Estado visual de preview |
| `RELEASES_SOURCE` | `github` u `offline` para el sitio |
| `GITHUB_TOKEN` | API de releases en build |
| `RUST_LOG` | Logs de desarrollo (nunca en `process`) |
| `RT_PARANOID` | Según TRUCE, para tests de asignación |

---

## 12. Primeros pasos sugeridos

1. Crear `Cargo.toml`, `truce.toml` y el workspace.
2. Implementar `dsp/` con una voz senoidal y tests.
3. Añadir envelope ADSR y key click.
4. Añadir bus global y efectos.
5. Integrar TRUCE y MIDI.
6. Construir la UI Slint.
7. Añadir presets y códec.
8. Crear el sitio y el primer preview.
