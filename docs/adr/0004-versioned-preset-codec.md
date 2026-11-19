# 0004 — Códec de preset versionado

- **Estado:** Aceptado
- **Fecha:** 2026-11-19
- **Decisores:** equipo Khodes
- **Relacionado:** [STATE_AND_PRESETS](../STATE_AND_PRESETS.md), [PARAMETERS](../PARAMETERS.md), [0005](0005-site-and-release-versioning.md)

---

## Contexto

Khodes necesita dos caminos de persistencia:

1. **Estado del host**: el proyecto del DAW guarda los parámetros y los restaura
   al reabrir. TRUCE lo gestiona como un bloque opaco.
2. **Preset nativo**: un archivo portátil entre hosts, versionado y validado,
   que también sirve para los presets de fábrica.

El proyecto de referencia usa `.bupreset` v3 con un layout explícito y validación
estricta: rechaza versiones no soportadas, UTF-8 inválido, valores no finitos,
truncamiento, bytes extra y archivos grandes. Las versiones 1 y 2 se rechazan
deliberadamente.

El riesgo con formatos de preset es doble:

- **Compatibilidad:** un preset guardado hoy debe abrir mañana, o el usuario
  pierde trabajo.
- **Seguridad:** un archivo malformado no debe crashear el plugin ni ejecutar
  nada.

Además, la versión del **formato** no debe confundirse con la versión del
**producto** ni con la del **sonido**. Mezclarlas obliga a migraciones
innecesarias y a decisiones ambiguas.

---

## Decisión

Khodes define `.khodespreset` versión 1 con layout binario little-endian,
validación estricta y carga atómica.

### Layout v1

| Offset | Tamaño | Campo | Regla |
|---:|---:|---|---|
| 0 | 8 | `magic` | `KHODESP\0` |
| 8 | 4 | `version` | `u32` = 1 |
| 12 | 4 | `param_count` | `u32` = 19 |
| 16 | 4 | `name_len` | `u32`, 0–63 |
| 20 | `name_len` | `name` | UTF-8 válido |
| … | 4 × n | `id` | `u32`, IDs `1..19` |
| … | 8 × n | `value` | `f64` normalizado `0..1` |
| … | 4 | `crc32` | CRC-32 del payload anterior |

- Tamaño máximo: **4096 bytes**.
- Sin padding, sin bytes extra.
- `param_count` debe ser 19 en v1.
- IDs duplicados, desconocidos o fuera de rango se rechazan.
- Valores `NaN`, `inf` o fuera de `0..1` se rechazan.
- La carga es **atómica**: o se aplica todo, o no se aplica nada.

### Versionado independiente

| Contrato | Variable | Cambia cuando |
|---|---|---|
| Producto | `CARGO_PKG_VERSION` | Semver del release |
| Preset | `PRESET_VERSION` | Cambia el layout binario |
| Sonido | `DSP_CONTRACT_VERSION` | Cambia el audio intencionalmente |
| Estado del host | versión del wrapper | La gestiona TRUCE |

Reglas:

1. Un cambio de sonido **no** cambia `PRESET_VERSION`.
2. Un parámetro nuevo se añade al final; no se renumera nada.
3. Un parámetro eliminado deja su ID reservado.
4. Una versión de códec no soportada se rechaza; no se interpreta “lo mejor
   posible”.
5. Un preset v1 debe cargar en toda versión 1.x del producto.

### Lo que nunca se guarda

- Notas sostenidas.
- Estado del pedal.
- Pitch bend.
- Fase de osciladores o LFOs.
- Cola de reverb.
- Estado de voice stealing.
- Última nota tocada.

Al restaurar, todas las voces parten libres, el pedal se considera suelto y el
bend vuelve a centro.

### Diálogos de archivo

El editor **no** abre diálogos de archivo propios. La importación/exportación
nativa se hace desde el host o desde una herramienta de línea de comandos. Los
presets de fábrica se eligen desde un combo y se aplican como parámetros del
host.

---

## Consecuencias

### Positivas

- Formato simple, auditable y fácil de testear.
- CRC y validación estricta detectan corrupción y errores de versión.
- La separación producto/preset/sonido evita migraciones falsas.
- Los presets son portátiles entre CLAP y VST3 y entre plataformas.
- El estado del host sigue siendo la fuente de verdad del proyecto.

### Negativas

- `f64` por parámetro ocupa más que un `f32`; irrelevante para 19 parámetros.
- Un layout binario exige documentación y tests de ida y vuelta.
- El rechazo estricto puede sorprender a usuarios con archivos “casi válidos”.
- Añadir metadatos requiere una versión nueva del códec.

### Neutrales

- El CRC-32 detecta corrupción accidental, no es una firma de seguridad.
- El límite de 4096 bytes deja espacio para crecer sin cambiar de estrategia.
- La herramienta CLI puede compartir el mismo crate `kit/`.

---

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|---|---|
| JSON | Más tamaño, ambigüedad de números, necesidad de parser en la UI; aceptable pero no necesario |
| TOML | Cómodo para humanos, menos estricto para binarios y rangos |
| Solo estado del host | No hay presets portátiles ni de fábrica |
| Guardar estado completo del motor | Reanudaría notas y haría proyectos no reproducibles |
| Usar `.bupreset` | Modelo de datos distinto; no es un formato de Khodes |
| Sin versión en el archivo | Imposible migrar sin heurísticas frágiles |

---

## Implementación

- [ ] Crear crate `kit/` sin dependencias de host ni UI.
- [ ] Definir `PRESET_VERSION = 1` y el layout en código.
- [ ] Implementar `encode` / `decode` con validación completa.
- [ ] Implementar CRC-32.
- [ ] Añadir tests de round-trip, extremos y rechazos.
- [ ] Definir los 6 presets de fábrica como datos inmutables.
- [ ] Integrar carga/guardado con el estado del host en `src/`.
- [ ] Documentar la migración futura en [STATE_AND_PRESETS](../STATE_AND_PRESETS.md).

Tests mínimos:

1. Round-trip de los 19 parámetros.
2. Round-trip de los 6 presets de fábrica.
3. Rechazo de magic, versión, CRC, UTF-8, IDs, valores y tamaño.
4. Rechazo de truncamiento y bytes extra.
5. Nombre de 63 bytes y nombre multibyte.
6. Carga atómica: un error no deja parámetros a medio aplicar.

---

## Referencias

- [`../STATE_AND_PRESETS.md`](../STATE_AND_PRESETS.md)
- [`../PARAMETERS.md`](../PARAMETERS.md)
- [`../CONTRACTS.md`](../CONTRACTS.md) §8
- [Buffer Uppercut — Contracts](https://github.com/piducancore/buffer-uppercut/blob/main/docs/CONTRACTS.md)
