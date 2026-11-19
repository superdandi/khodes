# Parámetros — Khodes

**Estado:** esquema propuesto para la v1. Es un contrato: una vez publicado un
release, estos IDs no se renumeran ni se reutilizan.

---

## 1. Convenciones

- El host ve todos los parámetros como valores normalizados `0.0..1.0`.
- Cada parámetro tiene un **ID entero estable**, un nombre interno en
  `snake_case`, una etiqueta visible y una unidad.
- El ID `0` no se usa.
- Los IDs `1..19` son la v1.
- Los IDs `20..63` quedan reservados para parámetros de voz y bus futuros.
- Los IDs `64..127` quedan reservados para *performance macros*.
- Los IDs `128..255` no se publican como parámetros de host; se usan para
  metadatos internos si hace falta.
- Cambiar un rango, unidad o curva requiere actualizar este documento, el
  código, los tests y, si afecta el sonido, un ADR.

---

## 2. Tabla de parámetros v1

### 2.1 Voz

| ID | Nombre | Etiqueta | Unidad | Rango real | Default | Mapeo normalizado | Automatizable |
|---:|---|---|---|---:|---:|---|---|
| 1 | `brightness` | Brightness | Hz | 500–20000 | 20000 | log: `500 * 40^n` | Sí |
| 2 | `key_click` | Key Click | % | 0–1 | 0 | lineal | Sí |

### 2.2 Envelope

| ID | Nombre | Etiqueta | Unidad | Rango real | Default | Mapeo normalizado | Automatizable |
|---:|---|---|---|---:|---:|---|---|
| 3 | `attack` | Attack | s | 0.001–0.2 | 0.005 | lineal | Sí |
| 4 | `decay` | Decay | s | 0.01–2 | 0.15 | lineal | Sí |
| 5 | `sustain` | Sustain | % | 0–1 | 0.6 | lineal | Sí |
| 6 | `release` | Release | s | 0.01–2 | 0.2 | lineal | Sí |

### 2.3 Amplificador

| ID | Nombre | Etiqueta | Unidad | Rango real | Default | Mapeo normalizado | Automatizable |
|---:|---|---|---|---:|---:|---|---|
| 7 | `volume` | Volume | % | 0–1 | 0.3 | lineal | Sí |
| 8 | `drive` | Drive | % | 0–1 | 0 | lineal | Sí |
| 9 | `bass` | Bass | dB | −12…+12 | 0 | lineal | Sí |
| 10 | `treble` | Treble | dB | −12…+12 | 0 | lineal | Sí |

### 2.4 Efectos

| ID | Nombre | Etiqueta | Unidad | Rango real | Default | Mapeo normalizado | Automatizable |
|---:|---|---|---|---:|---:|---|---|
| 11 | `tremolo_rate` | Trem. Rate | Hz | 0–10 | 4 | lineal | Sí |
| 12 | `tremolo_depth` | Trem. Depth | % | 0–1 | 0 | lineal | Sí |
| 13 | `chorus_rate` | Chorus Rate | Hz | 0–5 | 0.8 | lineal | Sí |
| 14 | `chorus_depth` | Chorus Depth | % | 0–1 | 0 | lineal | Sí |
| 15 | `reverb` | Reverb | % | 0–1 | 0 | lineal | Sí |

### 2.5 Interpretación global

| ID | Nombre | Etiqueta | Unidad | Rango real | Default | Mapeo normalizado | Automatizable |
|---:|---|---|---|---:|---:|---|---|
| 16 | `polyphony` | Polyphony | voces | 1–32 | 16 | `round(1 + 31n)` | Sí (discreto) |
| 17 | `pitch_bend_range` | Bend Range | semitonos | 0–24 | 2 | `round(24n)` | Sí (discreto) |
| 18 | `velocity_sensitivity` | Velocity | % | 0–1 | 0.5 | lineal | Sí |
| 19 | `fine_tune` | Fine Tune | cents | −100…+100 | 0 | lineal | Sí |

Total v1: **19 parámetros de host**.

---

## 3. Semántica de cada parámetro

### 3.1 Brightness (ID 1)

Frecuencia de corte del lowpass global.

```text
f = 500 * 40^n
n = log(f / 500) / log(40)
```

- `n = 0` → 500 Hz (opaco).
- `n = 1` → 20 kHz (abierto).
- Default musical: 20 kHz.

### 3.2 Key Click (ID 2)

Cantidad de ruido de ataque.

- `0` = sin click.
- `1` = click al nivel de la nota.
- El ruido se filtra a 3 kHz y dura ~30 ms.

### 3.3 Attack (ID 3)

Tiempo de subida lineal de 0 a `volume` de la nota.

- Rango audible: 1 ms a 200 ms.
- Default: 5 ms.

### 3.4 Decay (ID 4)

Tiempo desde el final del attack hasta el nivel de sustain.

- No es una constante de tiempo exponencial; es un tiempo de llegada.
- Default: 150 ms.

### 3.5 Sustain (ID 5)

Nivel que se mantiene mientras la tecla está presionada.

- `0` = la nota se apaga tras el decay.
- `1` = mantiene el nivel pico.
- Default: 0.6.

### 3.6 Release (ID 6)

Tiempo de caída desde el nivel actual hasta silencio tras el note-off.

- Default: 200 ms.
- Se aplica también a voces robadas con un fade de seguridad.

### 3.7 Volume (ID 7)

Ganancia maestra de salida.

- Default: 0.3 (hereda el nivel de Keys).
- El limitador de seguridad actúa después.

### 3.8 Drive (ID 8)

Cantidad de saturación tanh.

```text
k = 0                 si drive = 0
k = 1 + drive * 10    si drive > 0
```

- `0` = bypass exacto.
- `1` = saturación fuerte.

### 3.9 Bass (ID 9) y Treble (ID 10)

Shelves de la cadena global.

- Bass: lowshelf a 250 Hz.
- Treble: highshelf a 4 kHz.
- Rango: ±12 dB, paso 1 dB en la UI.

### 3.10 Tremolo Rate (ID 11) y Depth (ID 12)

LFO senoidal de amplitud.

- Rate: 0–10 Hz.
- Depth: 0–1; `0` = sin modulación.
- El LFO se suma a una ganancia base 1.

### 3.11 Chorus Rate (ID 13) y Depth (ID 14)

Delay modulado global.

- Rate: 0–5 Hz.
- Depth: mezcla wet 0–1.
- La modulación de delay es fija en ±4 ms; el control cambia la mezcla.

### 3.12 Reverb (ID 15)

Mezcla wet de la convolución.

- `0` = seco.
- `1` = solo reverb (no recomendado musicalmente, pero válido).
- IR determinista de 2.5 s.

### 3.13 Polyphony (ID 16)

Número máximo de voces simultáneas.

- Default: 16.
- Al bajarlo, las voces excedentes se liberan con fade.
- Es un parámetro discreto; la UI muestra el entero.

### 3.14 Pitch Bend Range (ID 17)

Semitonos de recorrido del pitch bend.

- Default: 2.
- Rango: 0–24.
- Afecta a todas las voces en v1.

### 3.15 Velocity Sensitivity (ID 18)

Profundidad de la respuesta a velocity.

```text
gain = 0.3 + 0.7 * (1 - s + s * velocity)
```

donde `s` es `velocity_sensitivity` y `velocity` es `0..1`.

- `s = 0` → todas las notas suenan al mismo nivel (`0.3 + 0.7 = 1.0`).
- `s = 1` → el nivel sigue la velocity (`0.3 + 0.7 * velocity`).
- Default: `0.5`.

Esta fórmula es una propuesta; se validará A/B contra la ausencia de velocity en
Keys.

### 3.16 Fine Tune (ID 19)

Desviación de afinación en cents.

- `−100…+100` cents.
- Default: 0.
- Se aplica a fundamental y campana.

---

## 4. Defaults de fábrica

Los defaults del host deben coincidir con el preset **Tine Classic**:

| Parámetro | Valor |
|---|---:|
| Brightness | 20000 Hz |
| Key Click | 0 |
| Attack | 0.005 s |
| Decay | 0.15 s |
| Sustain | 0.6 |
| Release | 0.2 s |
| Volume | 0.3 |
| Drive | 0 |
| Bass | 0 dB |
| Treble | 0 dB |
| Tremolo Rate | 4 Hz |
| Tremolo Depth | 0 |
| Chorus Rate | 0.8 Hz |
| Chorus Depth | 0 |
| Reverb | 0 |
| Polyphony | 16 |
| Bend Range | 2 |
| Velocity | 0.5 |
| Fine Tune | 0 cents |

---

## 5. Agrupación y presentación

| Grupo | Color HUD | Parámetros |
|---|---|---|
| Voz | Ámbar `#ffdd44` | Brightness, Key Click |
| Env | Verde `#00dd88` | Attack, Decay, Sustain, Release |
| Amp | Magenta `#ff00ff` | Volume, Drive, Bass, Treble |
| FX | Cyan `#00ffff` | Tremolo Rate/Depth, Chorus Rate/Depth, Reverb |
| Global | Azul `#7aa2ff` | Polyphony, Bend Range, Velocity, Fine Tune |

La UI puede mostrar textos más cortos, pero las etiquetas de accesibilidad deben
usar el nombre completo.

---

## 6. Automatización

- Todos los parámetros de `1..19` son automatizables.
- Los parámetros continuos emiten gestos `begin / set / end`.
- Los discretos (`polyphony`, `pitch_bend_range`) redondean al entero más
  cercano y aplican el cambio en borde de bloque.
- El host manda valores normalizados; el smoothing se aplica en el DSP.
- `fine_tune` se aplica con smoothing para evitar saltos de pitch.
- No hay parámetros ocultos que afecten el sonido guardado.

---

## 7. Estabilidad y compatibilidad

1. Un ID publicado no cambia de significado.
2. Un parámetro eliminado deja su ID reservado para siempre.
3. Añadir un parámetro usa el siguiente ID libre; no se inserta en medio.
4. Cambiar la curva de un parámetro existente es un cambio de sonido y requiere
   ADR y nueva versión de corpus.
5. El preset nativo guarda IDs + valores; si un ID falta en una versión futura,
   se usa el default y se registra una advertencia de migración.
6. El estado del host conserva los valores normalizados; la UI formatea.

---

## 8. Validación

Tests obligatorios:

- `id` único y monótono;
- `default` dentro de rango;
- mapeo normalizado ida/vuelta dentro de `1e-6`;
- parámetros discretos devuelven enteros válidos;
- ningún parámetro con `NaN` o `inf`;
- automatización de cada parámetro no produce samples no finitos;
- el preset round-trip conserva los 19 valores;
- los IDs `0` y reservados no se publican.

---

## 9. Parámetros futuros reservados

| Rango | Uso previsto |
|---|---|
| 20–29 | Modelo de voz (detune, stereo spread, hammer noise, etc.) |
| 30–39 | Reverb avanzada (size, decay, damping, predelay) |
| 40–49 | Chorus/phaser adicionales |
| 50–63 | Otros modelos de piano eléctrico |
| 64–127 | Performance macros (mod wheel, aftertouch, MPE) |

Ninguno se implementa en v1 sin actualizar este documento y un ADR.
