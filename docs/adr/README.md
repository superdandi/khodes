# Registro de decisiones de arquitectura (ADR) — Khodes

Un **ADR** (*Architecture Decision Record*) documenta una decisión técnica
significativa: el contexto que la motivó, la decisión tomada y sus
consecuencias. Su objetivo no es justificar el pasado, sino permitir que
alguien dentro de seis meses entienda **por qué** el sistema es como es y qué
costaría cambiarlo.

---

## 1. Cuándo escribir un ADR

Escribe un ADR cuando la decisión:

- afecta un contrato del producto (parámetros, MIDI, preset, versionado);
- cambia el sonido de forma intencional;
- introduce o elimina una dependencia estructural (framework, formato, codec);
- separa o fusiona capas de la arquitectura;
- cambia el proceso de release, firma o distribución;
- es difícil o costosa de revertir.

No escribas un ADR para:

- un refactor interno sin cambio de contrato;
- renombrar variables o mover archivos;
- corregir un bug que no altera comportamiento documentado;
- una decisión trivial y local.

En esos casos basta el commit y, si aplica, un test.

---

## 2. Estados

| Estado | Significado |
|---|---|
| **Propuesto** | Escrito, pendiente de aceptación. No obliga todavía. |
| **Aceptado** | Vigente. El código y los documentos deben cumplirlo. |
| **Rechazado** | Se evaluó y se descartó. Se conserva para no repetir el debate. |
| **Sustituido por NNNN** | Reemplazado por otro ADR. El nuevo es el vigente. |
| **Obsoleto** | Ya no aplica por cambios de contexto. |

Los ADRs aceptados **no se editan** para cambiar la decisión. Se crea uno nuevo
que lo sustituye y se actualiza el estado del anterior.

---

## 3. Numeración y nombres

```text
docs/adr/
├── README.md
├── 0001-stack-rust-truce-slint.md
├── 0002-port-keys-rhodes-voice.md
├── 0003-dsp-ui-separation.md
├── 0004-versioned-preset-codec.md
├── 0005-site-and-release-versioning.md
└── 0006-cc7-performance-gain.md
```

- Número de cuatro dígitos, secuencial, nunca reutilizado.
- Nombre en `kebab-case`, en inglés, corto y descriptivo.
- Un ADR por decisión. Si una decisión tiene varias partes inseparables, puede
  ser un solo documento.

---

## 4. Plantilla

Copia esta plantilla para un ADR nuevo:

```markdown
# NNNN — Título de la decisión

- **Estado:** Propuesto | Aceptado | Rechazado | Sustituido por NNNN
- **Fecha:** YYYY-MM-DD
- **Decisores:** nombres o roles
- **Relacionado:** enlaces a otros ADRs o documentos

## Contexto

Qué problema existe, qué restricciones aplican y qué se sabe hoy.
Sin proponer todavía una solución.

## Decisión

La decisión en una frase, seguida de los detalles necesarios para aplicarla.

## Consecuencias

### Positivas

- ...

### Negativas

- ...

### Neutrales

- ...

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|---|---|
| ... | ... |

## Implementación

Qué código, tests y documentos deben cambiar. Checklist si aplica.

## Referencias

- Enlaces internos y externos.
```

---

## 5. Índice

| ADR | Título | Estado | Fecha |
|---|---|---|---|
| [0001](0001-stack-rust-truce-slint.md) | Stack Rust + TRUCE + Slint | Aceptado | 2026-11-19 |
| [0002](0002-port-keys-rhodes-voice.md) | Port de la voz Rhodes de Keys | Aceptado | 2026-11-19 |
| [0003](0003-dsp-ui-separation.md) | Separación estricta DSP / UI | Aceptado | 2026-11-19 |
| [0004](0004-versioned-preset-codec.md) | Códec de preset versionado | Aceptado | 2026-11-19 |
| [0005](0005-site-and-release-versioning.md) | Sitio estático y versionado de releases | Aceptado | 2026-11-19 |
| [0006](0006-cc7-performance-gain.md) | CC7 como ganancia de performance | Aceptado | 2026-11-20 |

---

## 6. Relación con otros documentos

| Documento | Relación |
|---|---|
| [`../CONTRACTS.md`](../CONTRACTS.md) | Los ADRs explican el **porqué** de los contratos |
| [`../ARCHITECTURE.md`](../ARCHITECTURE.md) | Estructura que los ADRs justifican |
| [`../PARAMETERS.md`](../PARAMETERS.md) | Contrato afectado por 0002, 0003 y 0004 |
| [`../MIDI.md`](../MIDI.md) | Contrato afectado por 0006 |
| [`../STATE_AND_PRESETS.md`](../STATE_AND_PRESETS.md) | Contrato afectado por 0004 |
| [`../RELEASES.md`](../RELEASES.md) | Contrato afectado por 0005 |
| [`../SITE.md`](../SITE.md) | Contrato afectado por 0005 |
| [`../../AGENTS.md`](../../AGENTS.md) | Reglas de trabajo que citan los ADRs |

Regla: si un cambio contradice un ADR aceptado, **primero** se escribe el ADR
que lo sustituye; el código cambia después.
