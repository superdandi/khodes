# 0008 — Backport de TRUCE CLAP: RESCAN_VALUES tras cargar estado

- **Estado:** Aceptado
- **Fecha:** 2026-11-20
- **Decisores:** equipo Khodes
- **Relacionado:** [0001](0001-stack-rust-truce-slint.md), [0005](0005-site-and-release-versioning.md), [STATE_AND_PRESETS](../STATE_AND_PRESETS.md)

---

## Contexto

Al ejecutar `clap-validator 0.4.1` sobre el bundle CLAP construido con TRUCE
6.3.0, fallaron tres pruebas:

- `state-reproducibility-basic`
- `state-reproducibility-binary`
- `state-reproducibility-buffered`

El mensaje era:

```text
After reloading the state, these parameter values changed without a rescan request
```

La causa está en `truce-clap 6.3.0`: `state_load` aplica los valores de los
parámetros en el hilo del host mediante `state::apply_params`, pero no marca
`needs_rescan`. En cambio, las ediciones iniciadas por la GUI sí lo hacen y
terminan emitiendo `CLAP_PARAM_RESCAN_VALUES` desde `on_main_thread`.

El validador compara los valores vistos antes y después de `state.load()`. Como
la carga restaura valores aleatorios distintos a los defaults de una instancia nueva,
detecta un cambio sin notificación y falla, aunque los valores restaurados sean
correctos.

Desactivar esas pruebas está prohibido por `AGENTS.md` y por
`docs/RELEASES.md`. Actualizar TRUCE a una versión mayor también está fuera del
contrato de stack fijado en `Cargo.toml` y en el ADR 0001.

---

## Decisión

Incluir un **backport estrecho y auditado** de `truce-clap 6.3.0` en
`vendor/truce-clap/` y redirigir la dependencia con `[patch.crates-io]`.

El backport tiene dos cambios funcionales respecto al crate publicado.

**1. Notificación de parámetros en `state_load`.** Después de
`state::apply_params`, se marca `needs_rescan` y se solicita el callback del
host, reutilizando exactamente el mecanismo que ya usa TRUCE para las ediciones
de parámetros desde la GUI:

```rust
if !data.needs_rescan.swap(true, Ordering::Relaxed)
    && !data.host.is_null()
    && let Some(req_cb) = (*data.host).request_callback
{
    req_cb(data.host);
}
```

`on_main_thread` ya existente convierte esa bandera en
`CLAP_PARAM_RESCAN_VALUES`.

**2. Publicación del snapshot inicial en `clap_plugin_init`.** Se llama a
`instance.republish_snapshot()` justo después de `instance.init()`. Sin esto, un
host que guarda antes del primer bloque escribe un sobre sin bloque `extra`,
mientras que un guardado posterior a una carga de estado sí lo escribe; la
prueba `state-reproducibility-binary` detectaba 32 bytes de diferencia. Publicar
el estado inicial en `init` hace que ambos guardados sean byte-idénticos.

No se cambia la serialización del estado, no se tocan los IDs de parámetros y
no se modifica DSP, UI ni VST3.

Se conserva:

- el código completo del crate en `vendor/truce-clap/src/`;
- `Cargo.toml`, `Cargo.toml.orig` y `README.md` originales;
- la licencia exacta `The Truce License, Version 1.0` en
  `vendor/truce-clap/LICENSE`, obtenida del tag `v6.3.0` del repositorio
  oficial de TRUCE;
- un comentario en el punto parcheado que enlaza este ADR.

---

## Consecuencias

### Positivas

- Los tres tests de reproducibilidad de estado pasan sin relajar el validador.
- El host recibe una notificación CLAP estándar tras una carga de estado.
- El parche es de pocas líneas, queda bajo control del repo y es auditable
  por diff contra el crate publicado.
- La licencia del código vendorizado se distribuye explícitamente.

### Negativas

- El repo pasa a ser responsable de mantener una copia de un crate de terceros.
- Cualquier actualización futura de TRUCE debe reaplicar o retirar el parche.
- `Cargo.lock` referencia ahora una ruta local; los builds limpios deben
  incluir el directorio `vendor/`.

### Neutrales

- El comportamiento observable para el usuario no cambia aparte de la
  notificación al host.
- El formato del estado y el contrato `.khodespreset` se mantienen.

---

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|---|---|
| Saltarse los tests de estado en `clap-validator` | Prohibido; ocultaría un fallo real de contrato CLAP |
| Publicar sin pasar validadores | Incumple el gate de release y ADR 0005 |
| Actualizar a TRUCE 6.4 u otra rama | Rompe el stack fijado y no garantiza el fix |
| Reimplementar el wrapper CLAP propio | Coste desproporcionado y alto riesgo para un preview |
| Parchear el registro de crate global en la máquina local | No sería reproducible en CI ni para otros desarrolladores |

---

## Implementación

- [x] Copiar `truce-clap 6.3.0` a `vendor/truce-clap/`.
- [x] Añadir `LICENSE` oficial del tag `v6.3.0`.
- [x] Aplicar el cambio en `state_load`.
- [x] Publicar el snapshot inicial en `clap_plugin_init`.
- [x] Añadir `[patch.crates-io]` en `Cargo.toml`.
- [x] Actualizar `Cargo.lock`.
- [x] Documentar el componente en `THIRDPARTY.json`.
- [x] Recompilar bundles y repetir `clap-validator` y `pluginval`.

---

## Referencias

- [ADR 0001](0001-stack-rust-truce-slint.md).
- [ADR 0005](0005-site-and-release-versioning.md).
- [`STATE_AND_PRESETS.md`](../STATE_AND_PRESETS.md).
- [`release.yml`](../../.github/workflows/release.yml).
