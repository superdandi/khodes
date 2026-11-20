# Estado y presets — Khodes

**Estado:** contrato v1 implementado y verificado (falta validación en DAW real).

La implementación vive en `kit/src/lib.rs` (códec y presets), `src/state.rs`
(bloque custom) y `PluginLogic::snapshot_into` / `load_state` en `src/lib.rs`.

Khodes tiene dos caminos de persistencia:

1. **Estado del host** — el proyecto del DAW guarda los parámetros a través de
   TRUCE.
2. **Preset nativo `.khodespreset`** — archivo portátil entre hosts, versionado
   y validado.

Ninguno de los dos guarda estado de performance: notas sostenidas, fases de
oscilador, colas de reverb, estado de voice stealing ni posición de LFO.

---

## 1. Fuente de verdad

| Dato | Estado del host | Preset nativo |
|---|---|---|
| Valores de los 19 parámetros | Sí | Sí |
| Nombre del preset | Sí (metadata) | Sí |
| Versión del esquema | Sí (wrapper) | Sí (cabecera) |
| Notas activas | No | No |
| Sustain | No | No |
| Pitch bend | No | No |
| Fase de LFO | No | No |
| Cola de reverb | No | No |
| Estado de voces | No | No |

El estado del host es la fuente de verdad para el proyecto. El preset nativo es
un formato de intercambio y de fábrica.

---

## 2. Estado del host

- TRUCE expone el estado del plugin como un bloque opaco.
- El bloque contiene los 19 parámetros normalizados y el nombre del preset.
- No se añaden parámetros de host para metadatos.
- **Implementación:** TRUCE persiste automáticamente los 19 IDs en el sobre del
  host (`params`). El bloque `extra` lo produce `snapshot_into`, que serializa
  solo el nombre del preset con `#[derive(State)]` y devuelve siempre `true`
  para que el host invoque `load_state`. El snapshot se publica en un slot
  lock-free de 256 bytes reutilizables; por eso el bloque se mantiene pequeño.
- `load_state` corre en el hilo de audio entre bloques: valida el bloque y
  llama a `Engine::reset()`, `MidiMapper::reset()` y limpia `sustain`. No llama
  a `Engine::prepare()` porque reasignaría buffers. Un blob malformado se
  rechaza sin mutar el plugin ni cancelar notas.
- Al restaurar:
  - se cancelan todas las notas;
  - se limpia el sustain;
  - el pitch bend vuelve a centro;
  - las voces parten libres;
  - las colas de reverb se descartan;
  - el nombre del preset se muestra en la UI.
- El estado debe ser byte-estable: guardar dos veces sin cambios produce el
  mismo bloque (salvo metadatos del wrapper).

### 2.1 Reglas de restauración

```text
load_host_state(bytes):
  validar versión y longitud
  aplicar parámetros con smoothing desactivado (estado inicial)
  reset de voces y bus
  nombre = metadata o "Init"
  notificar a la UI
```

No se reproduce ninguna nota al cargar. Si el host estaba grabando una nota
sostenida, el note-off posterior se ignora porque no hay voz activa.

---

## 3. Presets de fábrica

Los presets de fábrica son datos inmutables compilados en `kit/`. No se leen de
disco en runtime. El usuario puede guardar una copia como preset nativo o como
preset del host.

| Nombre | Carácter | Ajustes principales |
|---|---|---|
| **Tine Classic** | Default fiel a Keys | Todos los defaults de [PARAMETERS](PARAMETERS.md) |
| **Suitcase Warm** | Cálido, con cuerpo | Brightness 8 kHz, Bass +3 dB, Treble −2 dB, Chorus 0.3 Hz / 20 %, Reverb 15 % |
| **Bell Air** | Ataque metálico, cola larga | Brightness 16 kHz, Key Click 10 %, Attack 2 ms, Decay 0.4 s, Sustain 30 %, Release 0.6 s, Chorus 0.4 Hz / 25 %, Reverb 35 % |
| **Dirty Drive** | Saturación marcada | Drive 60 %, Brightness 6 kHz, Bass +4 dB, Treble +2 dB |
| **Tremolo Lounge** | Movimiento clásico | Tremolo 3.5 Hz / 60 %, Chorus 0.2 Hz / 15 %, Reverb 20 % |
| **Silent Room** | Suave y oscuro | Volume 0.25, Brightness 4 kHz, Reverb 50 %, Chorus 0.1 Hz / 10 % |

Los valores exactos se fijan en `kit/src/lib.rs` y se prueban en el códec.

---

## 4. Formato `.khodespreset` v1

### 4.1 Layout binario

Little-endian, sin padding.

| Offset | Tamaño | Campo | Valor / regla |
|---:|---:|---|---|
| 0 | 8 | `magic` | ASCII `KHODESP` + `0x00` |
| 8 | 4 | `version` | `u32` = 1 |
| 12 | 4 | `param_count` | `u32` = 19 |
| 16 | 4 | `name_len` | `u32`, 0–63 |
| 20 | `name_len` | `name` | UTF-8 válido |
| … | 4 | `id_0` | `u32` = 1 |
| … | 8 | `value_0` | `f64` normalizado 0–1 |
| … | … | … | repetir 19 veces |
| … | 4 | `crc32` | CRC-32 del payload anterior |

Tamaño máximo: **4096 bytes**.

### 4.2 Ejemplo lógico

```text
KHODESP\0 | 00000001 | 00000013 | 00000007 | "MyTine"
| 00000001 | 3ff0000000000000   (brightness = 1.0)
| 00000002 | 0000000000000000   (key_click = 0.0)
| ...
| crc32
```

### 4.3 Validación obligatoria

El decodificador rechaza:

- magic incorrecto;
- versión desconocida;
- `param_count` distinto de 19;
- `name_len > 63`;
- UTF-8 inválido;
- IDs duplicados, desconocidos o fuera de `1..19`;
- valores `NaN`, `inf` o fuera de `0..1`;
- truncamiento;
- bytes sobrantes;
- CRC incorrecto;
- archivos mayores a 4096 bytes.

No se acepta un preset parcialmente válido. La carga es atómica: o se aplica
todo, o no se aplica nada.

### 4.4 Escritura

- Se escribe en memoria y se entrega al host o al adaptador de archivo.
- El editor no abre diálogos propios; la importación/exportación nativa se hace
  mediante el host o una herramienta de línea de comandos.
- El nombre se normaliza a UTF-8 y se trunca a 63 bytes sin cortar un carácter.

---

## 5. Versionado del códec

| Versión | Estado | Cambios |
|---:|---|---|
| 1 | Actual | 19 parámetros, nombre, CRC |
| 2 | Reservada | Nuevos parámetros o metadatos |

Reglas:

1. `PRESET_VERSION` es independiente de la versión del producto.
2. Un cambio de layout incrementa la versión.
3. El decodificador puede aceptar versiones antiguas solo si hay migración
   explícita y testeada.
4. Las versiones no soportadas se rechazan con un error claro; no se interpretan
   “lo mejor posible”.
5. Un preset v1 debe seguir cargando en todas las versiones 1.x del producto.

---

## 6. Migraciones

### 6.1 Añadir un parámetro

- Nuevo ID al final.
- `param_count` aumenta.
- Preset v1 se migra rellenando el nuevo ID con su default.
- Nueva versión de códec si cambia el layout.

### 6.2 Eliminar un parámetro

- El ID queda reservado.
- El valor se ignora en la migración.
- No se renumera nada.

### 6.3 Cambiar una curva

- No cambia el layout, pero cambia el sonido.
- Requiere ADR, nueva versión de corpus y nota de release.
- Los presets antiguos pueden sonar distinto; se documenta.

---

## 7. Presets y estado: flujo completo

```text
Usuario elige preset de fábrica
        │
        ▼
kit::factory(name) -> [19 valores]
        │
        ▼
editor aplica parámetros al host (gestos begin/set/end)
        │
        ▼
host guarda estado del proyecto
        │
        ├── guardar preset nativo -> .khodespreset
        └── cargar preset nativo -> validar -> aplicar -> reset de voces
```

---

## 8. A/B y undo

- La UI puede ofrecer A/B de parámetros (dos buffers en memoria) sin tocar el
  formato de preset.
- A/B no se guarda en el estado del host salvo que el usuario lo confirme.
- El undo/redo de parámetros es responsabilidad del host; Khodes no implementa
  su propio historial de proyecto.

---

## 9. Pruebas

1. Round-trip de los 6 presets de fábrica.
2. Round-trip de valores extremos (mínimos y máximos).
3. Rechazo de magic, versión, CRC, UTF-8, IDs y valores inválidos.
4. Rechazo de archivos truncados y con bytes extra.
5. Nombre de 63 bytes y nombre con caracteres multibyte.
6. Estado del host: guardar, reabrir, comparar 19 valores.
7. Cargar estado mientras hay notas: todas se cancelan.
8. Cargar preset con el editor cerrado y abierto.
9. Guardar dos veces sin cambios: mismo bloque.
10. Preset v1 cargado por una versión futura 1.x.

**Cobertura actual:** los puntos 1–7 y 9 están cubiertos por 16 tests en
`kit/tests/codec.rs`, tests de estado en `src/state.rs` y `src/lib.rs` y el
test de reset en `dsp/src/lib.rs`. El punto 8 se cubre en parte: el smoke CLAP
ejecuta el ciclo guardar/cargar sin editor abierto y `src/editor.rs`
sincroniza el combo desde el snapshot del host. El punto 10 queda para la
primera versión 1.x futura. La prueba en DAW real (REAPER y Ableton Live,
M6-17 del plan) sigue pendiente.

---

## 10. Lo que nunca se guarda

- Notas sostenidas.
- Estado del pedal.
- Pitch bend.
- Fase de osciladores o LFOs.
- Cola de reverb.
- Estado de voice stealing.
- Última nota tocada.
- Posición de la UI (salvo preferencia de tamaño del editor, si se implementa).

Esto evita que un proyecto se abra “sonando solo” y mantiene el estado
reproducible.
