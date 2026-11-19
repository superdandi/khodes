# Arquitectura — Khodes

**Estado:** diseño objetivo. El repositorio aún no contiene el código del plugin.

Khodes sigue el modelo de ingeniería de
[Buffer Uppercut](REFERENCE_BUFFER_UPPERCUT.md): un núcleo DSP puro, un wrapper
de plugin provisto por TRUCE, una UI nativa en Slint y un sitio estático
separado. La diferencia es que Khodes es un **instrumento**: la entrada
principal es MIDI y la salida es audio generado, no audio procesado.

---

## 1. Vista de sistema

```text
                 ┌──────────────────────────────────────────┐
                 │                DAW / Host                │
                 │  MIDI, transporte, automatización, estado │
                 └───────────────────┬──────────────────────┘
                                     │
                                     ▼
                 ┌──────────────────────────────────────────┐
                 │        Wrapper TRUCE (CLAP / VST3)       │
                 │  parámetros, eventos, estado, ventana     │
                 └───────────┬──────────────────┬───────────┘
                             │                  │
                             ▼                  ▼
                 ┌───────────────────┐  ┌───────────────────┐
                 │   src/ (producto) │  │   ui/ (Slint)      │
                 │  params, midi,    │  │  editor, controles │
                 │  editor, lógica   │  │  visualización     │
                 └─────────┬─────────┘  └─────────┬─────────┘
                           │                      │
                           ▼                      │
                 ┌───────────────────┐            │
                 │   dsp/ (puro)     │◄───────────┘
                 │  voces, bus,      │  snapshot atómico
                 │  allocator, FX    │
                 └─────────┬─────────┘
                           │
                           ▼
                 ┌───────────────────┐
                 │   kit/ (datos)    │
                 │  presets, códec   │
                 └───────────────────┘

                 ┌───────────────────┐
                 │   site/ (Astro)   │  fuera del binario
                 │  páginas, releases│
                 └───────────────────┘
```

Regla de oro: **el audio nunca espera a la UI y la UI nunca procesa audio**.

---

## 2. Mapa del repositorio

| Ruta | Posee | No debe poseer |
|---|---|---|
| `dsp/` | Síntesis de voz, allocator, bus de efectos, envelopes, parámetros DSP, telemetría | TRUCE, Slint, host, archivos, MIDI del host |
| `dsp/src/voice.rs` | Osciladores, envelope, key click, estado de una voz | Asignación global, parámetros del host |
| `dsp/src/allocator.rs` | Pool de voces, robo, note IDs | MIDI parsing, UI |
| `dsp/src/bus.rs` | Drive, brillo, EQ, chorus, tremolo, reverb, master | Asignación de voces |
| `dsp/src/lib.rs` | Orquestación de bloque, suma, límites, snapshot | Tipos de TRUCE o Slint |
| `kit/` | Presets de fábrica, códec `.khodespreset` | Diálogos de archivo, automatización |
| `kit/src/lib.rs` | Definiciones y serialización validada | Audio en tiempo real |
| `src/` | Integración con TRUCE, parámetros, MIDI, editor | DSP pesado, layout visual |
| `src/lib.rs` | Ciclo de vida, `process`, traducción parámetro→DSP | Layout, diálogos |
| `src/params.rs` | Esquema de parámetros y metadatos | Procesamiento de audio |
| `src/midi.rs` | Parseo y mapeo MIDI | Síntesis |
| `src/editor.rs` | Bindings Slint, gestos de automatización, presets | Procesamiento de audio |
| `src/input.rs` | Teclado del computador, puntero, ownership | DSP |
| `ui/` | Componentes y layout Slint | Lógica de producto |
| `contract/` | Corpus de regresión DSP versionado | Código de producción |
| `screenshots/` | Baselines visuales | Arte generado en runtime |
| `site/` | Astro estático, metadata de releases en build | Lógica del plugin, servidor |
| `docs/` | Especificaciones, ADRs, contratos | Código |
| `vendor/` | Backports estrechos de TRUCE/Slint documentados | Features de producto |

`dsp/` y `kit/` deben poder compilar y testear sin el wrapper.

---

## 3. Responsabilidades por capa

### 3.1 `dsp/` — motor puro

- No conoce el host, el sample rate hasta `prepare`, ni el estado del editor.
- Recibe un bloque planar de salida (y, si en el futuro hay efecto, entrada).
- Recibe eventos ya traducidos: `NoteOn { note, velocity, sample_offset }`,
  `NoteOff { note, sample_offset }`, `PitchBend`, `Sustain`, cambios de
  parámetros con offset.
- Mantiene el estado de voces y bus.
- Publica un snapshot de telemetría (nivel, voces activas, waveform) para la UI.
- No asigna memoria después de `prepare`.

### 3.2 `src/` — integración de producto

- Implementa `PluginLogic` de TRUCE.
- Traduce parámetros normalizados del host a valores DSP.
- Convierte eventos MIDI del host a eventos internos.
- Gestiona el ciclo de vida: `reset`, activación, cambio de sample rate,
  cambio de block size.
- Publica el estado del host y los presets nativos.
- Construye el editor Slint y conecta callbacks.

### 3.3 `ui/` — presentación

- Solo declara componentes, layout, colores, textos y bindings.
- No calcula audio ni escribe parámetros directamente: emite intenciones al
  editor de `src/`.
- Usa el snapshot de telemetría para medidores y waveform.

### 3.4 `kit/` — datos durables

- Define presets de fábrica.
- Implementa el códec `.khodespreset`.
- Valida rangos, versiones, UTF-8 y tamaño.
- No conoce el host ni el filesystem.

### 3.5 `site/` — producto fuera del binario

- Sitio Astro estático.
- Lee releases publicados desde GitHub en tiempo de build.
- No se ejecuta dentro del plugin.
- No comparte código con `dsp/`.

---

## 4. Flujo de audio por bloque

```text
process(output, events, params, transport):
  1. Aplicar cambios de parámetros con smoothing.
  2. Procesar eventos MIDI en orden de sample_offset:
       - note-on  -> allocator.note_on(...)
       - note-off -> allocator.note_off(...)
       - sustain  -> actualizar máscara de sostenidas
       - bend     -> actualizar pitch global
  3. Para cada muestra del bloque:
       a. Para cada voz activa: generar fundamental + campana + click.
       b. Sumar voces.
       c. Aplicar bus global (drive, brillo, EQ, chorus, tremolo, reverb).
       d. Aplicar master.
       e. Escribir en los canales de salida.
  4. Flush de denormales.
  5. Actualizar telemetría (cada N muestras, no cada bloque).
  6. Publicar snapshot atómico si corresponde.
```

### 4.1 Por qué muestra a muestra y no por bloques de voz

El chorus y el tremolo son LFOs globales; la reverb es una convolución. Procesar
la suma de voces muestra a muestra simplifica el orden y evita sumas parciales
inconsistentes. Si el costo es alto, se puede procesar el bus por sub-bloques
con el mismo resultado, pero la interfaz debe seguir siendo determinista.

### 4.2 Eventos con offset

Los eventos no se aplican todos al inicio del bloque. Cada evento lleva un
`sample_offset` dentro del bloque. El motor parte el bloque en segmentos:

```text
[0, offset1)  estado anterior
[offset1, offset2)  después del evento 1
...
```

Esto es obligatorio para notas rápidas y para automatización rítmica. La
referencia Buffer Uppercut acepta eventos a borde de bloque; Khodes no debe
heredar esa limitación para MIDI.

---

## 5. Asignación de voces

### 5.1 Estado

```text
enum VoiceState {
    Free,
    Attack,
    Decay,
    Sustain,
    Release,
    StealFade,
}
```

Cada voz guarda:

- `note: u8`, `channel: u8`, `note_id: u64`;
- fase de osciladores;
- estado de envelope;
- nivel actual y objetivo;
- flag de sostenida por pedal;
- edad de asignación.

### 5.2 Algoritmo

```text
note_on(note, velocity):
  if existe voz con misma note+channel en Attack/Decay/Sustain:
      retrigger suave (opcional) o robar
  else:
      voz = primera Free
      si no hay:
          voz = voz en Release más antigua
          si no hay:
              voz = voz sostenida más antigua
          iniciar StealFade(3–5 ms)
      asignar note, channel, note_id, velocity
      reiniciar fases y envelope
```

### 5.3 Note-off

```text
note_off(note, channel):
  para cada voz con note+channel:
      si sustain activo: marcar key_released, no liberar
      si no: pasar a Release
```

### 5.4 Robo y clic

El robo nunca corta en seco. Se aplica un fade corto y luego se reutiliza la
voz. El fade se completa dentro del mismo bloque o en el siguiente, sin asignar
memoria.

---

## 6. Parámetros y automatización

- El host entrega parámetros normalizados `0..1` con IDs estables.
- `src/params.rs` define el esquema: ID, nombre, rango, default, unidad,
  agrupación, flags de automación.
- `src/lib.rs` traduce a valores DSP una vez por bloque (o por evento de
  automatización) y el DSP aplica smoothing.
- Los parámetros continuos usan constante de tiempo ~20 ms.
- Los parámetros discretos (p. ej. polyphony) se aplican en borde de bloque.
- Ningún parámetro del host se lee desde `ui/` directamente.

Ver [PARAMETERS](PARAMETERS.md).

---

## 7. MIDI

Flujo:

```text
Host MIDI event
    -> src/midi.rs (parseo, canal, nota, velocity)
    -> src/lib.rs (agrupación por sample_offset)
    -> dsp::Event (note_on / note_off / bend / sustain)
    -> dsp::Engine
```

Reglas:

- Todos los canales se aceptan por defecto (modo omni).
- Note-on con velocity 0 equivale a note-off.
- Sustain CC64 se evalúa por canal y se agrega globalmente.
- Pitch bend se aplica a todas las voces; en v1 no hay bend por nota.
- All Notes Off / All Sound Off liberan voces.
- No se usa `setTimeout` ni relojes del sistema.

Ver [MIDI](MIDI.md).

---

## 8. Estado y presets

```text
Host state (TRUCE)
    ├── parámetros normalizados
    ├── nombre del preset de fábrica (metadata)
    └── versión del estado del wrapper

Preset nativo .khodespreset
    ├── magic + versión
    ├── nombre UTF-8
    ├── parámetros durables
    └── checksum/validación
```

- El estado del host es la fuente de verdad para el proyecto del DAW.
- El preset nativo permite portabilidad entre hosts.
- Ninguno guarda notas sostenidas, fase de LFO, colas de reverb ni estado de
  voice stealing.
- Al restaurar, todas las voces parten libres y el pedal se considera suelto.

Ver [STATE_AND_PRESETS](STATE_AND_PRESETS.md).

---

## 9. Concurrencia

| Dato | Escritor | Lector | Sincronización |
|---|---|---|---|
| Parámetros del host | Host / UI | Audio / UI | almacenamiento atómico de TRUCE |
| Eventos MIDI | Host | Audio | cola preasignada por bloque |
| Estado de voces | Audio | Audio | acceso exclusivo del motor |
| Snapshot de telemetría | Audio | UI | atomics con secuencia |
| Nivel de salida | Audio | UI | atomic `f32` |
| Nombre de preset | UI / host | UI / host | persistencia no-audio |
| Códec de preset | UI | UI | hilo de UI, nunca en `process` |

No hay locks en el hilo de audio. La UI puede leer un snapshot incompleto; en
ese caso conserva el frame anterior.

---

## 10. Seguridad en tiempo real

Prohibido dentro de `process` y sus llamadas:

- `Vec::push`, `Box::new`, `String`, `format!`, `println!`;
- locks (`Mutex`, `RwLock`), canales bloqueantes, `join`;
- acceso a archivos, red, variables de entorno;
- `panic!` no controlado;
- bucles no acotados;
- resizing de buffers;
- construcción de la IR de reverb;
- parsing de presets o JSON.

Permitido:

- aritmética `f32`/`f64`;
- acceso a buffers preasignados;
- atomics sin bloqueo;
- iteración sobre un número fijo de voces y etapas.

El perfil `rt-paranoid` de TRUCE debe fallar si hay una asignación en
`process`.

---

## 11. Sample rate y block size

- Sample rates soportados: 44.1, 48, 88.2, 96, 176.4, 192 kHz.
- Block sizes: 1–2048 muestras (el host puede usar más; el motor debe adaptarse
  sin asignar).
- `prepare(sample_rate, max_block_size)`:
  - calcula coeficientes de filtros;
  - dimensiona buffers de delay y reverb;
  - reinicia estados;
  - preasigna la cola de eventos.
- Cambio de sample rate o block size: reiniciar voces y buffers, sin clics
  patológicos (fade de seguridad).

---

## 12. Salida y estéreo

- La suma de voces es mono en v1.
- Chorus, tremolo y reverb generan la imagen estéreo.
- El master aplica volumen y un limitador de seguridad muy suave para evitar
  clipping por suma de voces.
- La salida debe ser siempre finita; samples subnormales se fuerzan a cero.

---

## 13. Build y artefactos

```sh
cargo truce build --clap --vst3
cargo truce install --clap --vst3
```

Salidas esperadas:

```text
target/bundles/Khodes.clap
target/bundles/Khodes.vst3
```

Instalación en macOS:

```text
~/Library/Audio/Plug-Ins/CLAP/Khodes.clap
~/Library/Audio/Plug-Ins/VST3/Khodes.vst3
```

El standalone es un host de desarrollo interno, no un formato distribuido.

---

## 14. Frontera con el sitio

El sitio no comparte binario ni estado con el plugin. Su única relación es:

- nombre y versión del producto;
- notas de release;
- contrato de nombres de archivo;
- enlaces de descarga.

Ver [SITE](SITE.md) y [RELEASES](RELEASES.md).

---

## 15. Decisiones relacionadas

- [ADR 0001](adr/0001-stack-rust-truce-slint.md): stack Rust/TRUCE/Slint.
- [ADR 0002](adr/0002-port-keys-rhodes-voice.md): port de la voz.
- [ADR 0003](adr/0003-dsp-ui-separation.md): separación DSP/UI.
- [ADR 0004](adr/0004-versioned-preset-codec.md): códec versionado.
- [ADR 0005](adr/0005-site-and-release-versioning.md): sitio y versionado.

---

## 16. Riesgos arquitectónicos

| Riesgo | Mitigación |
|---|---|
| Reverb convolucional costosa en CPU | IR corta, convolución por bloques, medir y permitir bypass |
| Aliasing de la campana en notas agudas | Limitar/atenuar `2f` cerca de Nyquist |
| Clics por robo de voz | Fade de 3–5 ms y tests de discontinuidad |
| Zipper noise | Smoothing de 20 ms y tests de barrido |
| Divergencia con Keys | Corpus A/B y criterios de [VOICE_SPEC](VOICE_SPEC.md) |
| Crecimiento del estado del host | Esquema estable y tests de ida y vuelta |
| Dependencia de TRUCE/Slint | Versiones fijas y `vendor/` mínimo documentado |
| Deriva de documentación | Un dato, un documento; revisión en cada PR |
