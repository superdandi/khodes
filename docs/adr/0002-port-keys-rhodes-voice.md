# 0002 — Port de la voz Rhodes de Keys

- **Estado:** Aceptado
- **Fecha:** 2026-11-19
- **Decisores:** equipo Khodes
- **Relacionado:** [VOICE_SPEC](../VOICE_SPEC.md), [PARAMETERS](../PARAMETERS.md), [0003](0003-dsp-ui-separation.md)

---

## Contexto

La voz a portar existe hoy en Studio Suite como un grafo de **Web Audio**:

- `RhodesControls.tsx` define 15 controles, rangos, defaults y presentación.
- `rhodes.ts` crea un bus compartido y, por nota, dos osciladores seno
  (fundamental y `2f`), un envelope ADSR, un envelope de campana y un burst de
  ruido para el key click.
- `useScalePlayer.ts` dispara notas con `setTimeout` y duraciones fijas o
  derivadas de una pieza.

Ese código no es directamente portable:

1. `setTimeout` no tiene precisión de muestra y no conoce el transporte del host.
2. Cada nota crea nodos del navegador; en un plugin las voces deben preasignarse.
3. `Math.random()` hace que el ruido y la IR de reverb no sean reproducibles.
4. No hay velocity, pitch bend, sustain ni polifonía acotada.
5. Los setters imperativos no son automatización de host.

El riesgo principal no es técnico sino **de identidad**: si el port se “mejora”
sin control, Khodes deja de sonar como Keys y pierde su razón de ser.

---

## Decisión

Khodes reimplementa la voz en Rust dentro de `dsp/`, conservando el
**comportamiento musical** de Keys y cambiando solo el **ciclo de vida**:

### Se conserva

- Fundamental senoidal en `f` y parcial senoidal en `2f`.
- Envelope principal A/D/S/R con la misma semántica:
  - `decay` es tiempo de llegada al sustain, no constante exponencial;
  - `sustain` se mantiene mientras la tecla está presionada;
  - `release` empieza en el note-off.
- Envelope de campana independiente que **no sostiene**:
  `t1 = A*0.6`, `t2 = t1 + D*0.6 + 0.02`, pico en `V*0.35`.
- Key Click opcional: ruido highpass a 3 kHz, ataque 1 ms, caída a ~30 ms.
- Orden del bus global:
  `drive tanh` → `lowpass de brillo` → `lowshelf 250 Hz` →
  `highshelf 4 kHz` → `chorus` → `tremolo` → `reverb` → `master`.
- Chorus: delay base 22 ms, LFO 0–5 Hz, excursión fija ±4 ms, el control cambia
  la **mezcla wet**, no la profundidad del LFO.
- Tremolo: LFO senoidal 0–10 Hz sumado a ganancia base 1.
- Reverb: IR de 2.5 s con decaimiento `exp(-t/0.6)`, mezcla dry/wet.
- Defaults de la voz original (brightness 20 kHz, attack 5 ms, decay 150 ms,
  sustain 0.6, release 200 ms, volume 0.3, etc.).

### Se cambia

| Aspecto | Keys | Khodes |
|---|---|---|
| Disparo | `setTimeout` | Evento MIDI con `sample_offset` |
| Duración | Fija / de la pieza | Note-on → note-off |
| Polifonía | Ilimitada | 1–32, default 16 |
| Asignación | Una por llamada | Pool con *stealing* determinista |
| Velocity | No existe | Curva configurable, piso 0.3 |
| Ruido | `Math.random()` | PRNG determinista con semilla |
| Reverb IR | Generada al crear el bus | Generada en `prepare`, reproducible |
| Parámetros | Setters | Automatización + smoothing ~20 ms |
| Campana aguda | Sin control | Atenuada/bandlimitada cerca de Nyquist |
| Denormales | No gestionados | Flush a cero |

### Extensiones deliberadas

- `velocity_sensitivity` (ID 18), con fórmula
  `gain = 0.3 + 0.7 * (1 - s + s*v)`.
- `polyphony` (ID 16), `pitch_bend_range` (ID 17), `fine_tune` (ID 19).
- `CC1` como *performance macro* de tremolo, no persistente.

Ninguna extensión puede alterar el sonido con velocity 100, bend a centro,
fine tune 0 y polyphony suficiente. Ese es el caso de referencia A/B.

---

## Consecuencias

### Positivas

- Khodes hereda un timbre ya validado por uso real.
- Los defaults y la UI pueden compararse 1:1 con Keys.
- El corpus de regresión puede congelarse contra el comportamiento documentado.
- Las extensiones son aditivas y verificables por separado.

### Negativas

- El port arrastra decisiones de Web Audio que no siempre son ideales
  (por ejemplo, chorus sin feedback, reverb convolucional corta).
- La fidelidad limita la experimentación tímbrica inicial.
- La comparación A/B exige renderizar Keys offline, lo que añade trabajo.

### Neutrales

- El bus global se mantiene mono en la suma de voces; el estéreo lo generan
  chorus y reverb.
- Una reverb algorítmica queda para el roadmap, no para v1.

---

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|---|---|
| Copiar el grafo Web Audio dentro de una web view | No es un plugin nativo; latencia y empaquetado |
| Rediseñar la voz desde cero | Pierde la identidad y no hay base de comparación |
| Samplear Keys | Cambia el modelo de producto y el tamaño; no es síntesis |
| Modelado físico completo de tine | Fuera de alcance y sin garantía de fidelidad perceptual |
| Portar “mejorando” el chorus/reverb | Rompe el A/B y oculta diferencias |

---

## Implementación

- [x] Implementar `dsp/src/voice.rs` con osciladores, envelopes y click.
- [x] Implementar `dsp/src/bus.rs` en el orden documentado.
- [x] Implementar PRNG determinista para click e IR.
- [x] Implementar allocator con stealing y fade de 3–5 ms.
- [x] Implementar segmentación por `sample_offset`.
- [x] Implementar smoothing de parámetros.
- [ ] Congelar `contract/v1/` con el primer render aceptado (M4-19/M4-20).
- [ ] Documentar el procedimiento A/B de [VOICE_SPEC](../VOICE_SPEC.md) §8.
- [x] Verificar afinación ±1 cent y tiempos de envelope ±5 %.

---

## Referencias

- `studio-suite/src/lib/rhodes.ts`
- `studio-suite/src/components/keys/RhodesControls.tsx`
- `studio-suite/src/hooks/useScalePlayer.ts`
- [`../VOICE_SPEC.md`](../VOICE_SPEC.md)
- [`../PARAMETERS.md`](../PARAMETERS.md)
