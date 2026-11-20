# 0006 — CC7 como ganancia de performance

- **Estado:** Aceptado
- **Fecha:** 2026-11-20
- **Decisores:** equipo Khodes
- **Relacionado:** [MIDI](../MIDI.md), [CONTRACTS](../CONTRACTS.md), [0003](0003-dsp-ui-separation.md)

---

## Contexto

El contrato MIDI v1 describe CC7 (Volume) como "mapeo directo a `volume` si el
host lo envía; opcional". `volume` es el parámetro de host ID 7: pertenece a los
19 IDs estables, es automatizable y se guarda en presets y en el estado del
host. Khodes recibe los parámetros como `&Params` inmutable dentro de
`process`, así que un mensaje MIDI no puede escribirlos sin romper la propiedad
del host sobre la automatización.

Además, CC1 ya se comporta como *performance macro* no persistente: modula la
profundidad de tremolo sin aparecer en el preset. CC7 necesita un trato
equivalente para no introducir un "parámetro oculto que afecte el sonido
guardado", que está prohibido en `CONTRACTS.md` §3.

El bus ya suaviza `volume` con ~20 ms, de modo que escalar la ganancia maestra
es una forma natural de evitar clics sin añadir otro filtro.

---

## Decisión

CC7 se traduce a un evento DSP `MasterVolume { level }` con `level = value /
127`. El motor mantiene `cc_volume`, neutro en `1.0`, y calcula la ganancia
efectiva del bus como `params.volume * cc_volume`. El evento se aplica en su
`sample_offset`, por lo que respeta el contrato sample-accurate de M7-06.

`cc_volume` vuelve a `1.0` con `Engine::reset()`, `ResetControllers` (CC121) y
la carga de estado. CC120 y CC123 afectan a las voces, no al nivel maestro.

CC7 **no** modifica el parámetro ID 7, no se guarda en `.khodespreset` ni en el
estado del host, y no cambia el corpus `contract/v1/`. El valor por defecto
`1.0` deja el sonido existente bit a bit idéntico.

---

## Consecuencias

### Positivas

- El host conserva la propiedad exclusiva de los 19 parámetros publicados.
- El cambio de volumen por CC7 es sample-accurate y se suaviza en el bus.
- Los presets y el estado del host siguen sin contener datos de performance.
- El corpus congelado no se reescribe: el valor neutro no altera ningún fixture.

### Negativas

- Un controlador externo que envíe CC7 no mueve el fader `Volume` de la UI; el
  fader representa el parámetro del host, no el macro de performance.
- Si un host ya mapea CC7 al parámetro ID 7 por MIDI learn, ambos caminos se
  multiplican. Es una configuración del host, documentada como tal.

### Neutrales

- El formato `.khdsp` del corpus acepta un `kind = 8` nuevo para
  `MasterVolume`; los fixtures existentes siguen siendo válidos.

---

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|---|---|
| Escribir el parámetro ID 7 desde `process` | `params` es inmutable; rompería la automatización del host y el contrato de IDs estables. |
| Ignorar CC7 | La checklist M7-05 pide soporte "si el host lo envía" y `MIDI.md` ya lo describe como opcional. |
| Añadir un parámetro de host `cc_volume` | Ampliaría los 19 IDs y requeriría guardarlo, violando §3 de `CONTRACTS.md`. |
| Aplicar ganancia en el wrapper antes del bus | Perdería el suavizado del bus y la precisión de offset. |

---

## Implementación

- [x] `EventBody::MasterVolume` en `dsp/src/lib.rs`.
- [x] Campo `cc_volume` y Ganancia efectiva en el bus.
- [x] Traducción de CC7 en `src/midi.rs`.
- [x] Tests DSP y de integración MIDI.
- [x] Actualizar `docs/MIDI.md` y `docs/CONTRACTS.md`.

---

## Referencias

- [MIDI](../MIDI.md) §§5–8
- [CONTRACTS](../CONTRACTS.md) §§3–6
- [PARAMETERS](../PARAMETERS.md) §3.7
