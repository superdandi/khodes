# MIDI — Khodes

**Estado:** especificación v1.

Khodes es un instrumento: la entrada MIDI es su fuente principal de eventos. La
precisión, el orden y la independencia de canales son parte del contrato de
producto.

---

## 1. Modelo de entrada

| Aspecto | v1 |
|---|---|
| Canales | 1–16, modo omni por defecto |
| Notas | 0–127; rango recomendado 21–108 (A0–C8) |
| Velocity | 1–127; note-on con velocity 0 = note-off |
| Pitch bend | 14 bits, centro 8192 |
| Sustain | CC64 |
| Mod wheel | CC1 (performance macro) |
| Aftertouch | No soportado en v1 |
| MPE | Diferido |
| MIDI 2.0 | Diferido |
| Program change | Ignorado en v1; los presets se manejan desde el host |
| Clock / transporte | No requerido para v1; LFOs libres |

Khodes no genera MIDI: no tiene salida MIDI, arpegiador ni secuenciador en v1.

---

## 2. Notas y afinación

### 2.1 Convención de octavas

- **C4 = MIDI 60**.
- A4 = MIDI 69 = 440 Hz.
- Frecuencia: `f = 440 * 2^((midi - 69) / 12)`.
- `fine_tune` (ID 19) desplaza en cents: `f' = f * 2^(cents / 1200)`.
- `pitch_bend` se aplica como multiplicador adicional.

Los hosts muestran octavas de forma distinta (C3 vs C4). La documentación y los
tests usan siempre el número MIDI.

### 2.2 Note-on

Al recibir `NoteOn(note, velocity, channel, offset)`:

1. Validar `note <= 127` y `velocity > 0`.
2. Aplicar la curva de velocity según el parámetro 18.
3. Asignar voz (ver [ARCHITECTURE](ARCHITECTURE.md)).
4. Si la misma nota y canal ya está sonando, aplicar retrigger suave o robar,
   según la política de voces.
5. Agendar el inicio en `offset` muestras dentro del bloque.

### 2.3 Note-off

Al recibir `NoteOff(note, channel, offset)` o `NoteOn(note, 0, ...)`:

- Si el pedal de sustain está activo para ese canal, marcar la tecla como
  liberada pero no pasar a release todavía.
- Si no, pasar la voz a release en `offset`.
- Si la voz ya fue robada, ignorar el evento por `note_id`.

### 2.4 Notas repetidas y solapadas

- Dos note-on de la misma nota y canal sin note-off intermedio: el segundo
  reinicia el envelope o roba la voz, según configuración.
- Dos canales distintos con la misma nota son voces independientes.
- Un note-off de un canal no debe liberar la nota de otro canal.

---

## 3. Velocity

La voz original de Keys no usa velocity. Khodes la añade con una profundidad
configurable:

```text
v = velocity / 127
gain = 0.3 + 0.7 * (1 - s + s * v)
```

donde `s = velocity_sensitivity` (ID 18, default 0.5).

| Sensibilidad | Comportamiento |
|---:|---|
| 0.0 | Todas las notas al mismo nivel |
| 0.5 | Respuesta moderada |
| 1.0 | Nivel proporcional a velocity |

La fórmula conserva un piso de `0.3` para que las notas suaves no desaparezcan.
Se validará perceptual y espectralmente contra Keys.

---

## 4. Sustain (CC64)

- Umbral: `value >= 64` activa; `< 64` desactiva.
- Se evalúa **por canal**.
- Mientras está activo, los note-off se recuerdan.
- Al desactivar, se liberan todas las voces cuya tecla ya fue soltada.
- Si se recibe All Notes Off, se limpia también el estado de sustain.
- El pedal no se guarda en el preset ni en el estado del host.

---

## 5. Pitch bend

- Mensaje de 14 bits: `value = (MSB << 7) | LSB`, rango `0..16383`.
- Centro: `8192`.
- Desviación normalizada: `(value - 8192) / 8192` en `[-1, +1]`.
- Semitonos: `bend * pitch_bend_range` (ID 17, default ±2).
- Se aplica a todas las voces en v1.
- Se suaviza para evitar escalones audibles.
- Al reiniciar el plugin o restaurar estado, el bend vuelve a centro.

---

## 6. Mod wheel (CC1)

Propuesta v1:

- CC1 modula la **profundidad de tremolo** de forma aditiva:
  `depth_efectiva = clamp(tremolo_depth + cc1/127 * mod_amount, 0, 1)`.
- `mod_amount` es una constante de producto (propuesta: 0.5) hasta que se
  publique como parámetro.
- No se guarda en el preset: es performance en vivo.
- Si el host no envía CC1, no afecta.

Esta asignación se revisará si se añade una matriz de modulación en el roadmap.

---

## 7. Otros controladores

| CC | Uso v1 | Notas |
|---:|---|---|
| 1 | Mod wheel → tremolo | Performance, no persistente |
| 7 | Volume | Mapeo directo a `volume` si el host lo envía; opcional |
| 11 | Expression | Diferido |
| 64 | Sustain | Ver sección 4 |
| 120 | All Sound Off | Libera voces con fade corto |
| 121 | Reset All Controllers | Bend a centro, sustain off, mod a 0 |
| 123 | All Notes Off | Libera todas las voces |

Los CC no listados se ignoran sin error.

---

## 8. Timing y orden de eventos

### 8.1 Regla

Todo evento MIDI lleva un `sample_offset` dentro del bloque. El motor procesa
los eventos en orden de offset; si dos eventos coinciden, conserva el orden de
llegada del host.

```text
bloque de N muestras:
  offset 0    -> note-on C4
  offset 37   -> note-off C4
  offset 120  -> note-on E4
  ...
```

### 8.2 Prohibiciones

- No usar `setTimeout`, `Instant::now`, `SystemTime` ni relojes de pared para
  agendar notas.
- No aplicar todos los eventos al inicio del bloque.
- No perder note-off por un note-on dentro del mismo bloque.
- No asumir que un note-off llega antes del siguiente bloque.

### 8.3 Limitaciones conocidas

- Si el host entrega eventos sin offsets (algunos wrappers antiguos), se aplican
  al inicio del bloque y se documenta la limitación.
- El sustain se evalúa en el offset del CC; las notas afectadas se procesan en
  segmentos.

---

## 9. Voice stealing desde MIDI

Cuando no hay voces libres:

1. Robar la voz en release más antigua.
2. Si no hay, robar la voz sostenida más antigua.
3. Si la nota robada es la misma que la nueva, reutilizar la voz con retrigger.
4. Aplicar fade de 3–5 ms.
5. Ignorar el note-off tardío de la voz robada mediante `note_id`.

El robo debe ser determinista y testeable con una secuencia MIDI fija.

---

## 10. Tabla de implementación MIDI

| Función | Estado v1 |
|---|---|
| Note On / Off | Sí |
| Velocity | Sí |
| Pitch Bend | Sí |
| CC64 Sustain | Sí |
| CC1 Mod Wheel | Sí (tremolo) |
| CC120 / 121 / 123 | Sí |
| Aftertouch | No |
| Poly Pressure | No |
| MPE | No |
| Program Change | No |
| MIDI Clock | No |
| MIDI Out | No |
| SysEx | Ignorado |

---

## 11. Pruebas obligatorias

1. Nota simple: C4 on/off, afinación y release.
2. Velocity 1, 64, 127: diferencias de nivel según sensibilidad.
3. Dos canales con la misma nota: holds independientes.
4. Sustain: note-off con pedal, liberación al soltar pedal.
5. Pitch bend: centro, extremos, retorno suave.
6. All Notes Off con notas sostenidas.
7. 32 notas simultáneas: robo determinista y sin samples no finitos.
8. Note-on y note-off en el mismo bloque con offsets distintos.
9. Nota repetida sin note-off: sin voces zombie.
10. Automatización de pitch bend range durante una nota: sin clic.
11. Guardar y reabrir: sin notas reanudadas.
12. Reproducir la misma secuencia dos veces: resultado idéntico (salvo PRNG
    sembrado).

---

## 12. Compatibilidad de hosts

| Host | Formato | Estado objetivo |
|---|---|---|
| REAPER | CLAP / VST3 | Obligatorio |
| Ableton Live | VST3 | Obligatorio |
| Bitwig Studio | CLAP / VST3 | Deseable |
| Cubase | VST3 | Deseable |
| Studio One | VST3 | Deseable |
| FL Studio | VST3 | Deseable |
| Logic Pro | AU | Fuera de v1 |
| Pro Tools | AAX | Fuera de v1 |

La matriz completa y el procedimiento están en [TESTING](TESTING.md).
