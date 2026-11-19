# Especificación de la voz — Khodes

**Estado:** especificación inicial derivada de Keys / Studio Suite.
**Fuente autoritativa del comportamiento actual:**
`/home/dandi/studio-suite/src/lib/rhodes.ts` y
`/home/dandi/studio-suite/src/components/keys/RhodesControls.tsx`.

Este documento describe exactamente qué hace hoy la voz Rhodes de Keys, qué se
conserva en Khodes y qué debe cambiar para funcionar como instrumento MIDI en
tiempo real dentro de un plugin.

---

## 1. Resumen ejecutivo

La voz de Keys es un **piano eléctrico de tipo tine** sintetizado, no sampleado.
Cada nota se construye con:

- un oscilador senoidal en la frecuencia fundamental;
- un segundo oscilador senoidal al doble de frecuencia (parcial metálico o
  “campana”);
- un envelope ADSR aplicado a la fundamental;
- un envelope de caída independiente para la campana;
- un burst de ruido filtrado opcional (Key Click) que simula el golpe mecánico.

Todo el audio de todas las notas pasa por un **bus de efectos compartido**:
drive → brillo → EQ → chorus → tremolo → reverb → volumen maestro. En Khodes
ese bus se conserva como cadena global post-mezcla de voces.

La diferencia principal entre Keys y Khodes no es tímbrica sino de **ciclo de
vida**: en la web cada nota crea nodos de Web Audio y se dispara con
`setTimeout`; en un plugin las notas llegan por MIDI, deben agendarse con
precisión de muestra, tener polyphony acotada y liberar recursos sin asignar
memoria en el hilo de audio.

---

## 2. Arquitectura actual en Keys

### 2.1 Bus compartido

`getRhodesBus()` crea una sola vez una cadena de nodos Web Audio y la reutiliza
para todas las notas. Los setters de la UI modifican los nodos existentes en
caliente; no recrean la cadena. Esto evita clics y mantiene coherencia entre
teclado físico, mouse, escalas y piezas de pianola.

```text
Nota (osciladores + envelope + click)
        │
        ▼
   input (GainNode)
        │
        ▼
   drive (WaveShaper, tanh, 4x oversample)
        │
        ▼
   brightness (BiquadFilter lowpass)
        │
        ▼
   bass (BiquadFilter lowshelf 250 Hz)
        │
        ▼
   treble (BiquadFilter highshelf 4 kHz)
        │
        ├──────────────► chorusDry (gain 1) ─────────────┐
        │                                                │
        └──► chorusDelay (22 ms) ◄── LFO sine 0–5 Hz     │
                    │                                    │
                    ▼                                    │
              chorusWet (gain = depth) ─────────────────┤
                                                         ▼
                                                   tremoloGain ◄── LFO sine 0–10 Hz
                                                         │
                        ┌────────────────────────────────┤
                        ▼                                ▼
                  reverbDry (1)                    convolver (IR 2.5 s)
                        │                                │
                        │                                ▼
                        │                          reverbWet (= reverb)
                        └──────────────► master ◄────────┘
                                          │
                                          ▼
                                     destination
```

### 2.2 Voz por nota

`playRhodesNote(freq, duration, volume)` crea, por cada nota:

1. `fundamental`: oscilador `sine` a `freq`.
2. `bell`: oscilador `sine` a `freq * 2`.
3. `g1`: gain del envelope principal.
4. `g2`: gain del envelope de campana.
5. Opcionalmente, una fuente de ruido para el Key Click.

Los dos osciladores se conectan al `input` del bus compartido. Al terminar la
nota, los nodos se detienen y el navegador los recolecta.

### 2.3 Disparo actual

`useScalePlayer` llama a `playRhodesNote` con:

- duración fija de `0.25 s` para escalas y `0.5 s` para notas sueltas;
- piezas con duración `min(0.6, beats * beatMs * 0.001)`;
- volumen `0.3`;
- scheduling con `setTimeout` cada `300 ms` o según el tempo de la pieza.

Esto **no es apto para un plugin**: `setTimeout` no tiene precisión de muestra,
no conoce el transporte del host y no garantiza orden dentro de un bloque.

---

## 3. Síntesis de una nota

### 3.1 Frecuencia

La frecuencia se calcula desde MIDI con afinación estándar A4 = 440 Hz:

```text
f = 440 * 2^((midi - 69) / 12)
```

En Keys, C4 = MIDI 60. Khodes conserva esa convención y documenta que los
hosts pueden mostrar octavas distintas; la fuente de verdad es el número MIDI.

### 3.2 Osciladores

| Componente | Forma | Frecuencia | Papel |
|---|---|---|---|
| Fundamental | Seno | `f` | Cuerpo tonal del tine |
| Campana | Seno | `2f` | Ataque metálico, brillo |
| Key Click | Ruido | — | Golpe mecánico del martillo |

No hay saturación por voz en la versión actual: el drive es global.

### 3.3 Envelope principal

Variables de entrada:

- `A` = attack en segundos (`attack`);
- `D` = decay en segundos (`decay`);
- `S` = nivel de sustain normalizado (`sustain`);
- `R` = release en segundos (`release`);
- `V` = volumen de la nota (hoy `0.3`);
- `T` = duración de la nota (hoy fija o derivada de la pieza).

Secuencia actual:

```text
t0 = ahora
t1 = t0 + A
t2 = t1 + D
t3 = t2 + T
t4 = t3 + R + 0.05

g1(t0) = 0
g1(t1) = V                    (rampa lineal)
g1(t2) = max(0.0001, V*S)    (rampa exponencial)
g1(t3) = V*S                 (sostiene)
g1(t4) = 0.001               (rampa exponencial)
```

Observaciones importantes:

- `decay` no es una constante de tiempo exponencial: es el **tiempo** que tarda
  en llegar al nivel de sustain.
- El sustain se mantiene durante `T`; en Khodes, `T` será el tiempo entre
  note-on y note-off, no una duración fija.
- El release empieza después de `T`; en Khodes empieza en el note-off.
- El mínimo `0.0001` evita que la rampa exponencial reciba cero.

### 3.4 Envelope de campana

La campana tiene su propio ciclo, independiente del sustain:

```text
t1b = t0 + A*0.6
t2b = t1b + D*0.6 + 0.02

g2(t0) = 0
g2(t1b) = V*0.35             (rampa lineal)
g2(t2b) = 0.001              (rampa exponencial)
```

La campana **no sostiene**: se apaga naturalmente aunque la tecla siga
presionada. Es una parte esencial del carácter “tine”.

### 3.5 Key Click

Si `keyClick > 0`:

- se usa un buffer de ruido de `0.05 s`;
- se filtra con un highpass a `3 kHz`;
- el gain ataca de 0 a `V * keyClick` en `1 ms`;
- cae exponencialmente a `0.001` en `30 ms`;
- se conecta al `input` del bus.

El click es independiente de la nota y no se realimenta al envelope principal.

---

## 4. Cadena global de efectos

### 4.1 Drive

WaveShaper con curva tanh y oversampling 4x.

```text
k = 0                        si drive <= 0
k = 1 + drive * 10           si drive > 0

curve(x) = x                          si k == 0
curve(x) = tanh(k*x) / tanh(k)        si k > 0
```

Con `drive = 0` la curva es la identidad (bypass). Con `drive = 1`, `k = 11`,
una saturación fuerte de carácter valvular.

### 4.2 Brightness

Filtro lowpass de 2º orden con frecuencia mapeada logarítmicamente:

```text
f(t) = 500 * 40^t           t ∈ [0, 1]
t(f) = log(f / 500) / log(40)
```

Rango: `500 Hz` (opaco) a `20000 Hz` (abierto). Default: `20000 Hz`.

La UI usa un fader vertical con posición normalizada `t` y la inversa exacta
para que el thumb no derive.

### 4.3 EQ

| Banda | Tipo | Frecuencia | Rango | Default |
|---|---|---|---|---|
| Bass | Lowshelf | 250 Hz | −12…+12 dB | 0 dB |
| Treble | Highshelf | 4 kHz | −12…+12 dB | 0 dB |

### 4.4 Chorus

Delay modulado:

- delay base: `22 ms`;
- LFO senoidal: `0–5 Hz`, default `0.8 Hz`;
- profundidad de modulación: `±4 ms` (constante, no escala con el control);
- mezcla dry/wet: dry siempre `1`, wet = `chorusDepth` (0–1).

El control `Chorus Depth` cambia la **mezcla**, no la amplitud del LFO. Es una
diferencia sutil pero importante para reproducir el comportamiento actual.

### 4.5 Tremolo

- LFO senoidal `0–10 Hz`, default `4 Hz`;
- profundidad `0–1`, default `0`;
- el LFO se suma a una ganancia base `1`, por lo que la amplitud oscila
  alrededor de 1 con excursión `±depth`.

### 4.6 Reverb

Convolver con una respuesta al impulso (IR) generada proceduralmente:

```text
IR(t) = ruido(t) * exp(-t / 0.6)
duración = 2.5 s
canales = 1 (mono)
```

Mezcla: dry `1`, wet = `reverb` (0–1), default `0`.

No es una reverb algorítmica de sala: es una cola corta tipo spring/room. En
Khodes se generará de forma determinista en la activación, para que el sonido
sea repetible y no dependa de `Math.random()`.

### 4.7 Master

Gain final = `volume` (0–1), default `0.3`. En Khodes se añadirá un medidor de
salida y protección contra picos, sin cambiar el nivel nominal.

---

## 5. Tabla completa de parámetros de la voz

Estos son los 15 controles de la voz original. Los IDs definitivos de Khodes se
definen en [PARAMETERS](PARAMETERS.md); aquí se documenta el comportamiento.

| # | Parámetro | Sección | Control en Keys | Rango | Default | Unidad mostrada | Implementación |
|---:|---|---|---|---:|---:|---|---|
| 1 | Brightness | Voz | Fader vertical | 500–20000 | 20000 | Hz / kHz | Lowpass, escala log |
| 2 | Key Click | Voz | Fader vertical | 0–1 | 0 | % | Ruido HPF 3 kHz |
| 3 | Attack | Env | Fader vertical | 0.001–0.2 | 0.005 | ms | Rampa lineal |
| 4 | Decay | Env | Fader vertical | 0.01–2 | 0.15 | s | Caída al sustain |
| 5 | Sustain | Env | Fader vertical | 0–1 | 0.6 | % | Nivel sostenido |
| 6 | Release | Env | Fader vertical | 0.01–2 | 0.2 | ms | Caída final |
| 7 | Volume | Amp | Knob | 0–1 | 0.3 | % | Gain maestro |
| 8 | Drive | Amp | Knob | 0–1 | 0 | % | Tanh 4x |
| 9 | Bass | Amp | Slider horizontal | −12…+12 | 0 | dB | Lowshelf 250 Hz |
| 10 | Treble | Amp | Slider horizontal | −12…+12 | 0 | dB | Highshelf 4 kHz |
| 11 | Tremolo Rate | FX | Knob | 0–10 | 4 | Hz | LFO seno |
| 12 | Tremolo Depth | FX | Knob | 0–1 | 0 | % | Amplitud LFO |
| 13 | Chorus Rate | FX | Knob | 0–5 | 0.8 | Hz | LFO seno |
| 14 | Chorus Depth | FX | Knob | 0–1 | 0 | % | Mezcla wet |
| 15 | Reverb | FX | Knob | 0–1 | 0 | % | Convolver |

### 5.1 Notas de UI

- `Brightness` se muestra en Hz o kHz según magnitud.
- `Attack` y `Release` se muestran en milisegundos aunque se almacenen en
  segundos.
- `Bass` y `Treble` muestran signo (`+3 dB`, `−2 dB`).
- Los knobs usan arrastre vertical normalizado al rango: ~150 px recorren todo
  el rango, independientemente de la magnitud.
- Doble clic restaura el default de Rhodes; en faders con default definido,
  restaura ese default, no el mínimo.

---

## 6. Comportamiento temporal actual y problemas de port

| Aspecto | Keys (web) | Khodes (plugin) |
|---|---|---|
| Disparo de nota | `setTimeout` / `playRhodesNote` | Evento MIDI con offset de muestra |
| Duración | Fija o derivada de la pieza | Note-on → note-off |
| Polifonía | Ilimitada (nodos del navegador) | 16 voces por defecto, máximo 32 |
| Asignación de voz | Una por llamada | Pool con stealing determinista |
| Sample rate | El del `AudioContext` | El del host, 44.1–192 kHz |
| Bloque | No aplica | 1–2048 muestras |
| Release | Programado por duración | Disparado por note-off |
| Parámetros | Setters imperativos | Automatización + smoothing |
| Ruido | `Math.random()` | PRNG determinista con semilla |
| Reverb IR | Generada al crear el bus | Generada en activación, reproducibilidad |
| Denormales | No gestionados | Flush a cero |
| Aliasing | Senos, bajo riesgo | Campana a 2f debe limitarse cerca de Nyquist |

### 6.1 Reglas de port

1. **No usar `setTimeout` ni relojes de pared.** Todo evento se agenda en
   muestras dentro del bloque.
2. **No crear nodos por nota.** Las voces se preasignan en `reset`.
3. **No asignar en `process`.** Buffers, IR, delays y estados se preparan en
   activación.
4. **Separar voz y bus.** Cada voz produce una señal mono; el bus global se
   aplica después de la suma de voces.
5. **Mantener el carácter del envelope.** La campana no sostiene; el click es
   independiente.
6. **Suavizar parámetros.** Cambios de cutoff, gains y mezclas con constante de
   tiempo ~20 ms para evitar zipper noise.
7. **Limitar la campana.** Si `2f >= 0.45 * sample_rate`, atenuar o bandlimitar
   el parcial para evitar aliasing.
8. **Determinismo.** La IR de reverb y el ruido del click usan PRNG con semilla
   fija; no `Math.random()`.

---

## 7. Modelo de voz propuesto para Khodes

### 7.1 Estructura por voz

```text
MIDI note-on
    │
    ▼
VoiceAllocator
    │  elige voz libre / roba la más antigua
    ▼
Voice
├── Oscilador fundamental (sine, fase inicial 0)
├── Oscilador campana (sine, 2f, fase inicial 0)
├── Envelope principal (A/D/S/R)
├── Envelope de campana (ataque + caída)
├── Key Click (ruido HPF, one-shot)
└── Gain de velocity
    │
    ▼
Suma de voces (mono)
    │
    ▼
Bus global (drive, brightness, EQ, chorus, tremolo, reverb, master)
    │
    ▼
Salida estéreo
```

### 7.2 Asignación de voces

- `polyphony` configurable: 1–32, default 16.
- Una voz está libre si su envelope terminó y su release completó.
- Si no hay voz libre:
  1. robar la voz en release más antigua;
  2. si no hay, robar la voz sostenida más antigua;
  3. aplicar un fade de 3–5 ms para evitar clic.
- El robo es determinista: mismo estado de entrada → misma voz elegida.
- Cada voz tiene un `noteId` para ignorar note-off de notas ya robadas.

### 7.3 Velocity

La voz original no usa velocity. Khodes propone:

```text
gain = 0.3 + 0.7 * velocity^velocityCurve
```

con `velocityCurve` configurable (default 1.0) y un parámetro de sensibilidad
(ver [PARAMETERS](PARAMETERS.md)). El valor `0.3` conserva el nivel base de Keys.

### 7.4 Sustain y note-off

- `CC64 >= 64` activa sustain: los note-off se recuerdan pero no liberan la voz.
- Al bajar `CC64 < 64`, se liberan todas las voces cuya tecla ya fue soltada.
- `CC123` (All Notes Off) y `CC120` (All Sound Off) liberan de inmediato; el
  segundo puede hacer un fade corto.
- Si el host descarga el plugin, no se guarda ninguna nota sostenida.

### 7.5 Pitch bend

- Rango default: ±2 semitonos.
- Afecta a la fundamental y a la campana de todas las voces activas.
- Se aplica como multiplicador de frecuencia, con suavizado.
- Rango configurable 0–24 semitonos (propuesta, ver roadmap).

### 7.6 Mod wheel

Propuesta inicial: `CC1` controla la profundidad de tremolo de forma aditiva
sobre el valor del parámetro, con cantidad configurable. Se documentará como
*performance macro* y no como parámetro automatable adicional en v1.

---

## 8. Fidelidad y aceptación

No se busca una copia muestra a muestra de Web Audio, sino equivalencia
perceptual y medible. Criterios:

1. **Afinación:** fundamental dentro de ±1 cent respecto a `440 * 2^((m-69)/12)`.
2. **Envelope:** tiempos de attack, decay y release dentro de ±5 % o ±2 ms.
3. **Nivel:** diferencias de loudness integrada ≤ 0.5 dB con los mismos ajustes.
4. **Espectro:** centroide espectral y energía del parcial 2f dentro de ±10 % en
   el ataque.
5. **Click:** el burst de ruido existe, está centrado en el ataque y dura
   ~30 ms.
6. **Efectos:** barridos de Drive, Brightness, Bass, Treble, Chorus, Tremolo y
   Reverb sin discontinuidades audibles.
7. **Reproducibilidad:** dos renders con los mismos ajustes producen el mismo
   resultado (salvo ruido determinista sembrado).

### 8.1 Procedimiento de comparación

1. Renderizar en Keys (OfflineAudioContext o captura) una nota MIDI 60 con
   ajustes neutros y con cada efecto al 50 %.
2. Renderizar lo mismo en Khodes con los mismos ajustes.
3. Comparar forma de onda, espectro, envolvente de amplitud y loudness.
4. Guardar los resultados en `contract/v1/` como evidencia, no como copia
   binaria de Web Audio.

---

## 9. Extensiones y decisiones abiertas

| Tema | Estado | Nota |
|---|---|---|
| Velocity | Propuesto | No existe en Keys; se añade con sensibilidad configurable |
| Stereo por voz | Diferido | v1 suma mono; el estéreo lo generan chorus/reverb |
| Detune/analog drift | Diferido | No presente en Keys; no introducir sin ADR |
| Modelos alternativos | Diferido | Rhodes, Wurlitzer, FM, etc. |
| Reverb algorítmica | Diferido | v1 conserva convolución determinista |
| MPE | Diferido | Requiere asignación por nota y pitch bend por voz |
| Ajuste de curva de velocity | Propuesto | Parámetro global |
| Pan por voz | Diferido | Puede romper fidelidad del bus original |
| Oversampling global | Diferido | El drive usa 4x; evaluar CPU |

---

## 10. Mapa de código fuente

| Archivo | Responsabilidad |
|---|---|
| `studio-suite/src/lib/rhodes.ts` | Bus, síntesis, envelope, setters, reset |
| `studio-suite/src/components/keys/RhodesControls.tsx` | UI, controles, defaults, interacción |
| `studio-suite/src/hooks/useScalePlayer.ts` | Disparo de notas/escalas/piezas |
| `studio-suite/src/lib/music-theory.ts` | MIDI, frecuencia, escalas, progresiones |
| `studio-suite/src/lib/audio.ts` | `AudioContext` compartido y utilidades |
| `studio-suite/DOCUMENTACION.md` | Decisiones y correcciones históricas de la voz |

Constantes relevantes de `rhodes.ts`:

```text
DRIVE_CURVE_SIZE   = 1024
CHORUS_DELAY_BASE  = 0.022 s
CHORUS_LFO_DEPTH   = 0.004 s
BRIGHTNESS_MIN     = 500 Hz
BRIGHTNESS_MAX     = 20000 Hz
IR_DURATION        = 2.5 s
IR_DECAY           = 0.6 s
```

---

## 11. Checklist de implementación de la voz

- [ ] Definir IDs y rangos normalizados en `PARAMETERS.md`.
- [ ] Implementar osciladores y envelope por voz en `dsp/`.
- [ ] Implementar key click determinista.
- [ ] Implementar bus global en el orden documentado.
- [ ] Implementar chorus con delay modulado y mezcla correcta.
- [ ] Implementar tremolo con LFO senoidal.
- [ ] Implementar reverb convolucional determinista.
- [ ] Implementar voice allocator y stealing.
- [ ] Implementar note-on/note-off, sustain y pitch bend.
- [ ] Implementar smoothing de parámetros.
- [ ] Implementar flush de denormales y protección de salida.
- [ ] Crear corpus de regresión y pruebas A/B contra Keys.
- [ ] Validar en CLAP y VST3 en al menos dos DAWs.
