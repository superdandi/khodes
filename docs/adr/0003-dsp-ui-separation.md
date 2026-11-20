# 0003 — Separación estricta DSP / UI

- **Estado:** Aceptado
- **Fecha:** 2026-11-19
- **Decisores:** equipo Khodes
- **Relacionado:** [ARCHITECTURE](../ARCHITECTURE.md), [0001](0001-stack-rust-truce-slint.md)

---

## Contexto

Un plugin de audio tiene dos mundos con requisitos opuestos:

- **Audio:** debe correr en un hilo de tiempo real, sin asignaciones, sin locks,
  sin I/O, con un presupuesto de milisegundos por bloque. Un fallo se oye como
  un clic o un *dropout*.
- **UI:** corre en el hilo principal, puede bloquear, asignar, leer archivos y
  repintar. Un fallo se ve como lentitud, no como audio roto.

Mezclar ambos produce los bugs clásicos de plugins: la UI lee un parámetro que
el audio está escribiendo, el audio espera un lock de la UI, el editor asigna
memoria mientras se procesa un bloque, o el tamaño de un buffer cambia en
caliente.

La referencia Buffer Uppercut resuelve esto con capas separadas: `dsp/` puro,
`src/` como integración de producto, `ui/` declarativo y un snapshot atómico
para la visualización. Khodes adopta el mismo principio con más énfasis, porque
es un instrumento y tiene estado por voz.

---

## Decisión

Khodes se organiza en capas con dependencias **unidireccionales**:

```text
ui/  ──emite intenciones──►  src/  ──eventos y parámetros──►  dsp/
 ▲                             │                                │
 └────snapshot atómico─────────┴────────────────────────────────┘
```

Reglas duras:

1. **`dsp/` no conoce TRUCE, Slint, el host ni el sistema de archivos.**
   Su API es Rust puro: eventos, parámetros, buffers.
2. **`ui/` no procesa audio ni escribe parámetros directamente.**
   Emite intenciones al editor de `src/`, que decide y aplica.
3. **`src/` es el único lugar que conoce TRUCE.**
   Traduce host → DSP y DSP → UI.
4. **La comunicación audio → UI es por snapshot atómico**, no por locks.
   La UI puede leer un frame viejo; nunca bloquea el audio.
5. **La comunicación UI → audio es por parámetros/eventos preasignados**,
   con gestos de automatización cuando corresponde.
6. **`process` no asigna, no bloquea, no registra y no toca disco.**

---

## Consecuencias

### Positivas

- El DSP se testea con `cargo test` sin DAW, sin Slint y sin TRUCE.
- El motor se puede portar a otro wrapper sin reescribir la síntesis.
- Los fallos de tiempo real se acotan a `dsp/` y `src/lib.rs`, no a la UI.
- La UI puede redibujarse a 60 fps sin afectar el audio.
- El código es más fácil de razonar: cada capa tiene una responsabilidad.

### Negativas

- Más plomería: snapshots, colas, traducción de eventos.
- La UI no tiene acceso directo al estado del DSP; hay latencia de un frame.
- Cualquier dato nuevo visible requiere diseñar su paso por el snapshot.
- Disciplina constante: es fácil “hacer una excepción” y romper la regla.

### Neutrales

- El snapshot puede ser parcial; la UI conserva el último frame válido.
- La telemetría se actualiza cada N muestras, no cada bloque, para no gastar CPU.

---

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|---|---|
| UI y DSP en el mismo módulo | Acopla hilos y viola tiempo real |
| Compartir estado con `Mutex` | Bloqueos en audio; inaceptable |
| Leer parámetros del host desde Slint | Duplica fuente de verdad y rompe automatización |
| Un canal por parámetro | Costoso y difícil de versionar |
| Recalcular todo en la UI | Duplica lógica y puede divergir del audio |

---

## Implementación

- [x] Crear `dsp/` como crate independiente sin dependencias de TRUCE/Slint.
- [x] Definir `dsp::Event` y `dsp::Params` como tipos puros.
- [ ] Implementar el snapshot de telemetría con atomics y secuencia (M5).
- [x] Prohibir en `dsp/` imports de `truce`, `slint`, `std::fs`, `std::net`.
- [x] Añadir un check local (`tools/check-dsp-boundary.sh`) que verifique esas
      prohibiciones; pendiente de ejecutarlo en CI cuando haya remote.
- [x] Ejecutar `rt-paranoid` en las verificaciones locales como gate.
- [x] Documentar la API en `dsp/src/lib.rs`.

Sugerencia de verificación automática:

```sh
# Falla si dsp/ menciona capas prohibidas.
! grep -RInE '\b(truce|slint|std::fs|std::net)\b' dsp/src
```

El comando exacto se ajustará al árbol final.

---

## Referencias

- [`../ARCHITECTURE.md`](../ARCHITECTURE.md)
- [`../CONTRACTS.md`](../CONTRACTS.md) §6
- [`../TESTING.md`](../TESTING.md) §7
- [Buffer Uppercut — DEVELOPMENT](https://github.com/piducancore/buffer-uppercut/blob/main/docs/DEVELOPMENT.md)
