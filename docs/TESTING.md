# Testing y aceptación — Khodes

**Estado:** plan de pruebas v1.

La estrategia sigue el modelo de Buffer Uppercut: tests rápidos locales, corpus
de audio congelado, modo `rt-paranoid`, validadores de formato, screenshots del
editor y aceptación en DAWs reales. Ningún release se publica solo porque los
tests unitarios pasen.

---

## 1. Pirámide de pruebas

```text
                 ┌──────────────────────────────┐
                 │ Aceptación en DAW real       │  manual, por release
                 ├──────────────────────────────┤
                 │ Validadores CLAP/VST3        │  CI de release
                 ├──────────────────────────────┤
                 │ Screenshots Slint            │  CI
                 ├──────────────────────────────┤
                 │ Corpus de audio congelado    │  CI
                 ├──────────────────────────────┤
                 │ Tests de integración MIDI    │  CI
                 ├──────────────────────────────┤
                 │ Tests unitarios DSP/kit      │  local + CI
                 └──────────────────────────────┘
```

---

## 2. Checks rápidos

```sh
cargo test --locked --workspace
```

Cubre:

- DSP de voces, envelopes, bus y allocator;
- parámetros y traducción;
- MIDI;
- códec `.khodespreset`;
- estado del host;
- lógica de editor sin UI.

Antes de entregar:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace
cargo test --locked --workspace --features rt-paranoid
cargo truce build --clap --vst3
```

En la CPU local (Intel i3 M380, sin AVX2) añadir `--target-cpu v2` al comando
anterior: el valor por defecto de `cargo truce` es `x86-64-v3` y produce
`SIGILL`. No usar `RUSTFLAGS` para esto: se aplica también a los build scripts
de las dependencias y provoca el mismo fallo.

`rt-paranoid` es bloqueante: cualquier asignación en el hilo de audio falla la
aceptación.

---

## 3. Tests unitarios de DSP

### 3.1 Afinación

- MIDI 21, 60, 69, 108 → frecuencia esperada ±1 cent.
- Fine tune ±100 cents.
- Pitch bend a centro y extremos.
- Dos voces simultáneas no desafinan entre sí.

### 3.2 Osciladores

- Fundamental presente en el espectro.
- Parcial 2f presente y decreciente.
- Sin aliasing audible por encima de 0.45 × sample rate.
- Fase inicial estable y reproducible.

### 3.3 Envelope

- Attack: 0 → pico en el tiempo configurado (±5 %).
- Decay: pico → sustain en el tiempo configurado.
- Sustain: nivel estable mientras la tecla está presionada.
- Release: silencio en el tiempo configurado.
- Sin clics en note-on, note-off ni retrigger.
- El envelope de campana no sostiene.

### 3.4 Key Click

- A cero: silencio adicional.
- A uno: burst audible, HPF ~3 kHz, duración ~30 ms.
- No contamina el sustain.
- Determinista con la misma semilla.

### 3.5 Drive

- `drive = 0`: salida igual a entrada (dentro de tolerancia).
- Aumento de armónicos medible.
- Sin samples no finitos en extremos.
- Oversampling no introduce latencia no reportada.

### 3.6 Brightness

- 500 Hz atenúa agudos.
- 20 kHz es prácticamente abierto.
- Barrido logarítmico sin zipper noise.
- Ida y vuelta de la posición normalizada ±1e-6.

### 3.7 EQ

- Bass +12 dB sube 250 Hz.
- Treble +12 dB sube 4 kHz.
- Cambios suaves sin clics.

### 3.8 Chorus

- Delay base 22 ms.
- LFO 0–5 Hz.
- Profundidad cambia mezcla wet, no la excursión.
- Sin realimentación inestable.

### 3.9 Tremolo

- Rate 0–10 Hz.
- Depth 0 = sin modulación.
- Depth 1 = excursión completa sin invertir fase.
- Sin discontinuidades al cambiar rate/depth.

### 3.10 Reverb

- IR determinista (misma semilla → mismo buffer).
- Mezcla 0 = seco.
- Mezcla 1 = solo wet.
- Cola finita y sin denormales.
- Sin asignación en `process`.

### 3.11 Allocator

- 1–32 voces.
- Robo de la release más antigua.
- Robo de la sostenida más antigua.
- Note-off tardío ignorado.
- Sin voces zombie.
- Sin samples no finitos con 32 notas.

### 3.12 Bus y suma

- 16 voces sumadas no saturan la salida.
- Limitador de seguridad no distorsiona en niveles normales.
- Salida estéreo coherente.
- Flush de subnormales.

---

## 4. Corpus de regresión

```text
contract/
├── v1/
│   ├── *.khdsp
│   └── SHA256SUMS
```

Reglas:

- El corpus v1 se congela con la primera implementación aceptada.
- Un cambio de sonido intencional crea `v2/`; no se reescribe `v1/`.
- Cada fixture tiene entrada, parámetros, sample rate, block size y salida
  esperada.
- `.khdsp` es texto UTF-8 determinista; los valores `f64` se guardan como 16
  dígitos hexadecimales de sus bits, así que el round-trip es exacto y el
  SHA256 estable entre plataformas IEEE 754.
- Tolerancia: absoluta + relativa `1e-7` en `f64`.
- El harness (`dsp/tests/contract.rs`) verifica el render actual contra los
  fixtures congelados. Nunca los reescribe.

Generación (solo la primera vez, o al crear `v2/`):

```sh
cargo run --locked -p khodes-dsp --release --example gen_contract
(cd contract/v1 && sha256sum *.khdsp > SHA256SUMS)
```

Verificación:

```sh
(cd contract/v1 && sha256sum -c SHA256SUMS)
cargo test -p khodes-dsp --test contract
```

### 4.1 Casos mínimos del corpus

1. Nota C4 con defaults.
2. Nota C4 con drive 0.5.
3. Nota C4 con brightness 2 kHz.
4. Nota C4 con chorus 0.5/0.5.
5. Nota C4 con tremolo 5 Hz/0.5.
6. Nota C4 con reverb 0.5.
7. Acorde de 4 notas.
8. 32 notas con robo.
9. Sustain con note-off.
10. Pitch bend ±2.
11. Cambio de sample rate (48 kHz y 96 kHz).
12. Bloque de 1 muestra y de 2048 muestras.

---

## 5. Tests de integración MIDI

- Note-on/off con offsets dentro del bloque.
- Velocity 1/64/127.
- Dos canales con la misma nota.
- Sustain CC64.
- Pitch bend.
- CC1 mod wheel.
- CC120/121/123.
- All Notes Off.
- Nota repetida sin note-off.
- 32 notas simultáneas.
- Eventos fuera de orden.
- Note-off sin note-on previo (no debe crashear).

---

## 6. Tests de estado y presets

- Round-trip de los 6 presets de fábrica.
- Round-trip de extremos.
- Rechazo de magic, versión, CRC, UTF-8, IDs, valores y tamaño.
- Estado del host byte-estable.
- Cargar estado con notas activas: todo se cancela.
- Cargar preset con editor abierto/cerrado.
- Guardar dos veces sin cambios.
- Compatibilidad v1 → 1.x futura.

---

## 7. rt-paranoid

```sh
cargo test --locked --workspace --features rt-paranoid
```

Debe cubrir:

- `process` con 0, 1, 16 y 32 voces;
- cambios de parámetros;
- note-on/off;
- robo de voz;
- carga de preset fuera de `process`;
- reverb y chorus;
- flush de denormales.

Cualquier asignación, lock, log o acceso a disco en `process` falla el test.

---

## 8. Validadores de formato

Construir bundles release:

```sh
cargo truce build --clap --vst3
```

CLAP:

```sh
clap-validator validate "target/bundles/Khodes.clap"
```

VST3:

```sh
pluginval --strictness-level 10 --skip-gui-tests --validate "target/bundles/Khodes.vst3"
```

- `clap-validator`: 0 fallos, 0 warnings.
- `pluginval`: strictness 10; GUI tests en sesión interactiva antes de release.
- Los validadores corren sobre los bundles exactos del tag, no sobre un build
  local distinto.

### 8.1 Smoke host CLAP

Cuando no hay `clap-validator` o DAW disponible, se usa el host mínimo de
`tools/`: carga el bundle como un DSO, comprueba el símbolo `clap_entry`, crea
una instancia desde la factory, verifica el descriptor y ejecuta cuatro bloques
de 256 frames con salida finita.

```sh
sh tools/clap-smoke.sh target/bundles/Khodes.clap
```

El script clona las cabeceras oficiales de CLAP en `$TMPDIR/khodes-clap-smoke`,
compila el host con `cc` y lo ejecuta. Es una comprobación de carga y ciclo de
vida, **no** una validación de cumplimiento ni una prueba en DAW; CI la ejecuta
como comprobación adicional, sin sustituir a los validadores formales ni a la
matriz de DAW de la sección 10.

---

## 9. Screenshots del editor

Baselines comprometidos:

```sh
cargo truce screenshot --out screenshots/slint-default.png --check --debug --scale 1
KHODES_EDITOR_SIZE=920x620 cargo truce screenshot --out screenshots/slint-minimum.png --check --debug --scale 1
KHODES_EDITOR_SIZE=1440x760 cargo truce screenshot --out screenshots/slint-wide.png --check --debug --scale 1
```

Casos visuales:

- default;
- mínimo;
- wide;
- nota activa / medidor;
- preset de fábrica cargado;
- control deshabilitado si aplica.

Revisar visualmente antes de actualizar un baseline.

---

## 10. Matriz de DAW

| DAW | Formato | Prioridad | Qué probar |
|---|---|---|---|
| REAPER | CLAP / VST3 | Obligatorio | Carga, MIDI, editor, estado, automatización |
| Ableton Live | VST3 | Obligatorio | Carga, MIDI, editor, guardar/reabrir |
| Bitwig Studio | CLAP / VST3 | Deseable | CLAP, MIDI, estado |
| Cubase | VST3 | Deseable | Categoría, MIDI, estado |
| Studio One | VST3 | Deseable | Carga, MIDI, estado |
| FL Studio | VST3 | Deseable | Carga, MIDI, editor |
| Logic Pro | AU | Fuera de v1 | — |
| Pro Tools | AAX | Fuera de v1 | — |

### 10.1 Checklist por DAW

1. El plugin aparece con el nombre **Khodes**.
2. Instancia en pista de instrumento.
3. Abre el editor al tamaño default.
4. Redimensiona al mínimo y vuelve.
5. Suena una nota MIDI 60.
6. Velocity cambia el nivel.
7. Sustain funciona.
8. Pitch bend funciona.
9. Cambian todos los parámetros sin clics.
10. Se guarda el proyecto y se reabre con los mismos valores.
11. No se reanudan notas al reabrir.
12. Se descarga el plugin sin colgar el host.
13. Cambio de sample rate y block size sin crash.
14. Automatización de brightness y volume.
15. Preset del host: guardar, cambiar, cargar.

---

## 11. Pruebas de rendimiento

```sh
cargo run --locked -p khodes-dsp --release --example cpu_bench
```

El ejemplo `cpu_bench` mide el render puro del DSP (sin wrapper ni host), con
16 voces sostenidas a 48 kHz y bloques de 256 frames. Es una referencia
reproducible entre cambios, no una medida de consumo en DAW.

Medir:

- 1, 8, 16 y 32 voces;
- 44.1, 48, 96 y 192 kHz;
- bloques de 64, 128, 256 y 512;
- reverb on/off;
- chorus on/off (éstos al cerrar M4).

Registrar:

- máquina, OS, CPU, Rust, sample rate, block size;
- mediana de 5 corridas;
- % de un core;
- memoria en activación.

Criterio: una regresión repetible > 10 % en el callback de audio bloquea la
aceptación salvo decisión explícita.

### 11.1 Baseline M3 (2026-11-20)

| Parámetro | Valor |
|---|---|
| Voces | 16 sostenidas |
| Sample rate | 48 kHz |
| Block size | 256 frames |
| Audio renderizado | 10.00 s |
| Tiempo de CPU | 1.659 s |
| Factor tiempo real | 6,0× |
| ms por bloque | 0,8846 ms |
| CPU | Intel i3 M380 (sin AVX2; build `--target-cpu v2`) |
| Rust | 1.92.0 |
| Estado del bus | sin FX (M3) |

Este número es el punto de partida. No es comparable contra un bus con reverb y
chorus activos; la medición de M4 está en §11.2.

### 11.2 Baseline M4 (2026-11-20)

Misma máquina, Rust y condiciones que §11.1 (`--target-cpu v2`), 16 voces a
48 kHz, bloques de 256 frames, 10.00 s de audio por caso. Medida única (no
mediana de 5):

| Caso | Tiempo de CPU | Factor tiempo real | ms/bloque |
|---|---:|---:|---:|
| `fx_off` | 1.758 s | 5,7× | 0,9376 |
| `chorus_on` (0,5) | 1.887 s | 5,3× | 1,0065 |
| `reverb_on` (0,5) | 13.659 s | 0,7× | 7,2850 |
| `all_on` | 14.084 s | 0,7× | 7,5115 |

**Incidencia de rendimiento:** la reverb convolucional de 2,5 s no cabe en el
presupuesto de un bloque de 256 frames a 48 kHz en esta CPU (5,33 ms): consume
≈7,3 ms. El preset por defecto tiene `reverb = 0`, por lo que el funcionamiento
normal se mantiene en 5,7× tiempo real. Antes de publicar un release hay que
optimizar la convolución o verificar el consumo en una CPU objetivo moderna;
esta medición no se presenta como resultado aceptado. Registrado como M4-22.

---

## 12. Pruebas de seguridad y robustez

- Entradas de preset malformadas (fuzz básico).
- Parámetros `NaN`, `inf`, fuera de rango.
- Bloques de tamaño 1 y 4096.
- Sample rates extremos.
- 1000 note-on sin note-off.
- 1000 note-off sin note-on.
- Cambio de preset durante una nota.
- Cierre del editor durante una nota.
- Cambio de dispositivo de audio.
- Suspensión/reanudación del host.

---

## 13. Definición de done

Una feature está completa cuando:

- tests enfocados cubren comportamiento y fallos;
- workspace, fmt, Clippy y `rt-paranoid` pasan;
- CLAP y VST3 compilan;
- corpus y checksums pasan;
- screenshots o pruebas de DAW cubren cambios visibles;
- `VOICE_SPEC`, `PARAMETERS`, `CONTRACTS`, `ROADMAP` y ADR están actualizados;
- no se commitean artefactos;
- se documentan limitaciones conocidas.

Un release está listo cuando además:

- los bundles exactos del tag pasan validadores;
- la matriz obligatoria de DAWs está probada;
- `VERSION.md`, `THIRDPARTY.json` y `APPROVED.txt` están revisados;
- el draft se revisó y se publicó manualmente;
- el sitio muestra la versión y los enlaces correctos.
