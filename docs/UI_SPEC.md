# Especificación de UI — Khodes

**Estado:** diseño objetivo, derivado del panel “Rhodes · Voz del instrumento”
de Keys.

La UI de Khodes conserva el lenguaje visual HUD cyberpunk de Studio Suite, pero
se adapta a un editor de plugin: tamaño fijo/redimensionable, sin scroll
obligatorio, sin diálogos de archivo y con estados de host.

---

## 1. Principios

1. **El sonido primero.** Cada control debe ser identificable y manipulable sin
   ambigüedad.
2. **Secciones con color.** Voz, Env, Amp y FX tienen acentos distintos.
3. **Lectura inmediata.** Cada control muestra valor numérico y unidad.
4. **Interacción de plugin.** Arrastre vertical, doble clic para default, teclado
   y accesibilidad.
5. **Sin cromo innecesario.** El HUD decora, no distrae.
6. **Consistencia con Keys.** Mismos nombres, orden y defaults.

---

## 2. Paleta

| Uso | Color | Hex |
|---|---|---|
| Fondo principal | Negro azulado | `#0a0a14` |
| Panel | Carbón | `#12121f` |
| Borde | Azul grisáceo | `#2a2a4a` |
| Texto principal | Gris claro | `#c8c8e0` |
| Texto secundario | Gris medio | `#8888aa` |
| Texto tenue | Gris oscuro | `#555566` |
| Voz | Ámbar | `#ffdd44` |
| Env | Verde | `#00dd88` |
| Amp | Magenta | `#ff00ff` |
| FX | Cyan | `#00ffff` |
| Global | Azul | `#7aa2ff` |
| Alerta / clipping | Rojo | `#ff4455` |
| Éxito / activo | Verde | `#00ff66` |

El fondo puede usar gradientes radiales suaves y una textura de rejilla muy
tenue. No usar animaciones que compitan con los controles.

---

## 3. Layout

### 3.1 Tamaños

| Modo | Tamaño |
|---|---|
| Default | 1120 × 700 |
| Mínimo | 920 × 620 |
| Wide | 1440 × 760 |

El layout debe ser responsive dentro de esos rangos, sin scroll horizontal.

### 3.2 Estructura

```text
┌─────────────────────────────────────────────────────────────┐
│ KHODES        [preset ▼]   [Init]   [MIDI]   [voces: 0/16]  │
├─────────────────────────────────────────────────────────────┤
│ VOZ                         ENV                             │
│  Brightness   Key Click      Attack  Decay  Sustain  Release│
│  [fader]      [fader]        [fader] [fader][fader] [fader] │
├─────────────────────────────────────────────────────────────┤
│ AMP                                         FX              │
│  Volume  Drive  Bass  Treble   Trem Rate/Depth  Chorus ...  │
│  [knob]  [knob] [slider][slider] ...                        │
├─────────────────────────────────────────────────────────────┤
│ GLOBAL                                                      │
│  Polyphony  Bend Range  Velocity  Fine Tune                 │
├─────────────────────────────────────────────────────────────┤
│ [medidor L/R]                         [estado / versión]    │
└─────────────────────────────────────────────────────────────┘
```

En tamaño mínimo, las secciones pueden apilarse en dos columnas; en wide, se
distribuyen en cuatro.

---

## 4. Componentes

### 4.1 `Knob`

Usado en Amp y FX.

- Cuerpo circular con gradiente radial.
- Marcas de escala alrededor.
- Arco de valor desde `-135°` a `+135°`.
- Aguja y punto central.
- Readout numérico debajo.
- Arrastre vertical relativo normalizado al rango: ~150 px recorren todo el
  rango.
- Doble clic → default.
- Teclado ↑/→ `+step`, ↓/← `−step`.
- `role="slider"` y `aria-*`.

### 4.2 `Fader` vertical

Usado en Voz y Env.

- Pista de 96 px.
- Relleno en gradiente desde la base.
- Cap tipo LED que se intensifica al arrastrar.
- Interacción **absoluta**: el thumb salta al punto tocado y sigue al puntero.
- Doble clic → default.
- Teclado igual que el knob.
- `role="slider"`.

### 4.3 `Slider` horizontal

Usado en Bass y Treble.

- Input range estilizado.
- Readout con signo y unidad.
- Teclado nativo.
- `aria-label`.

### 4.4 `Readout`

- Fondo oscuro, borde del color de la sección.
- Texto monoespaciado.
- Formato por unidad:
  - Hz/kHz para brightness;
  - % para niveles;
  - ms/s para tiempos;
  - dB para EQ;
  - Hz para rates;
  - cents para fine tune.

### 4.5 `Section`

- LED + título en mayúsculas + línea degradada.
- Color de acento por sección.
- Controles agrupados con `flex-wrap`.

### 4.6 Medidor de voces

- Texto `voces: N/16`.
- Barra o LEDs que se encienden con la actividad.
- No debe causar repaint costoso en el hilo de audio; usa snapshot atómico.

---

## 5. Estados

| Estado | Apariencia |
|---|---|
| Normal | Borde `#2a2a4a`, texto secundario |
| Hover | Borde del acento al 50 % |
| Activo / drag | Glow del acento |
| Seleccionado | Borde del acento completo |
| Deshabilitado | Opacidad 0.35, sin interacción |
| Clipping | Rojo parpadeante suave en el medidor |

Estados de nota:

- Nota activa: LED del medidor encendido.
- Sin voces: medidor en reposo.
- Sustain: indicador opcional “SUS”.

---

## 6. Interacción

- Arrastre vertical en knobs y faders.
- `pointer capture` para no perder el gesto fuera del control.
- Limpieza en `pointercancel` y `lostpointercapture`.
- Doble clic restaura default.
- Teclado accesible.
- Rueda del mouse: ajuste fino (opcional, documentado).
- Shift + arrastre: ajuste fino (opcional).
- No usar scroll para cambiar valores salvo que el control tenga foco.

### 6.1 Reglas de plugin

- El editor no abre diálogos de archivo.
- El preset se elige desde el host o desde un combo de presets de fábrica.
- Los cambios de parámetro emiten gestos de automatización al host.
- La UI no escribe audio directamente.

---

## 7. Accesibilidad

- Todos los controles tienen nombre accesible.
- `aria-valuemin`, `aria-valuemax`, `aria-valuenow`.
- Navegación por Tab.
- Foco visible.
- Contraste AA.
- No depender solo del color.
- Textos escalables.
- Respetar `prefers-reduced-motion` en animaciones.

---

## 8. Slint: notas de implementación

- Un componente `Knob` reutilizable con propiedades `value`, `min`, `max`,
  `step`, `label`, `accent`.
- Un componente `Fader` con geometría absoluta.
- Un componente `Section` con título y acento.
- Un `Readout` con formato delegado a Rust para no duplicar lógica.
- El estado visual se alimenta de un snapshot; no leer parámetros del host
  directamente desde Slint.
- Evitar animaciones costosas; el editor debe poder abrirse con el audio
  corriendo.

---

## 9. Textos

- Títulos de sección en español: VOZ, ENV, AMP, FX, GLOBAL.
- Etiquetas de control en inglés, igual que Keys: Brightness, Key Click,
  Attack, Decay, Sustain, Release, Volume, Drive, Bass, Treble, Trem. Rate,
  Trem. Depth, Chorus Rate, Chorus Depth, Reverb, Polyphony, Bend Range,
  Velocity, Fine Tune.
- Tooltips en español.
- Formato de números con separador decimal local, pero sin ambigüedad.

---

## 10. Screenshots

Baselines:

```text
screenshots/slint-default.png
screenshots/slint-minimum.png
screenshots/slint-wide.png
screenshots/slint-active.png
```

Reglas:

- Revisar visualmente antes de actualizar.
- No commitear capturas temporales.
- Cubrir default, mínimo, wide y una nota activa.

---

## 11. Lo que no se copia de Buffer Uppercut

- Paleta crema/carbón/amarillo ácido.
- Pads de performance.
- Waveform de cadena serial.
- Controles de key mapping.
- Layout de 16 slots.

Khodes usa la identidad de Keys, no la de Buffer Uppercut.

---

## 12. Checklist de UI

- [ ] Layout default, mínimo y wide.
- [ ] Secciones con color y LED.
- [ ] Knobs, faders y sliders funcionales.
- [ ] Readouts con unidades correctas.
- [ ] Doble clic restaura default.
- [ ] Teclado y accesibilidad.
- [ ] Medidor de voces.
- [ ] Sin diálogos de archivo.
- [ ] Screenshots baseline.
- [ ] Sin repaint costoso en audio.
