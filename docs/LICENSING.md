# Licencias y distribución — Khodes

**Estado:** política objetivo. No sustituye asesoría legal.

Khodes combina código propio, frameworks y dependencias con licencias
distintas. El `LICENSE` raíz no relicencia las dependencias.

---

## 1. Código original

El código original de Khodes —DSP, códec de presets, integración, UI y sitio—
se publica bajo **MIT**.

```text
MIT License

Copyright (c) 2026 Khodes contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## 2. Dependencias principales

### 2.1 TRUCE 6.3

- Licencia: `LicenseRef-TruceLicense-1.0`.
- Permite distribuir plugins de audio bajo su concesión.
- No se debe etiquetar como MIT.
- Conservar el texto de licencia y los avisos aplicables.
- Revisar la versión exacta usada y su rider.

### 2.2 Slint 1.15.x

Slint ofrece tres opciones:

1. GPL-3.0-only.
2. `LicenseRef-Slint-Royalty-free-2.0`.
3. Comercial.

Khodes usa la opción **royalty-free** para distribuir el plugin de escritorio,
no GPL. Obligaciones:

- conservar el texto de licencia;
- mostrar el badge oficial de atribución de Slint en la página de descarga,
  conforme a la sección 2(b) de la licencia royalty-free empaquetada;
- no implicar endorsement.

### 2.3 Rust y crates transitivos

- Cada crate conserva su licencia.
- Revisar las versiones seleccionadas, no solo los rangos de Cargo.
- Incluir avisos de licencias y copyright requeridos.
- Revisar crates de build y dev que puedan terminar empaquetados.

### 2.4 Astro y dependencias del sitio

- Astro y sus dependencias tienen sus propias licencias.
- El sitio se distribuye como HTML/JS estático; conservar avisos cuando
  corresponda.
- Las fuentes e imágenes deben tener licencia compatible.

### 2.5 Fuentes e imágenes

- Usar fuentes con licencia explícita (OFL, Apache, MIT, etc.).
- Conservar el aviso de copyright.
- No empaquetar fuentes sin permiso.
- Las capturas del editor son del propio proyecto.

---

## 3. Avisos de terceros

Cada release público debe incluir:

- `LICENSE` con el texto MIT;
- `THIRDPARTY.json` o un documento equivalente;
- textos de licencia de dependencias;
- atribución de Slint;
- avisos de fuentes y recursos nativos;
- `INSTALL.txt`.

### 3.1 Generación

```sh
cargo install cargo-bundle-licenses --version 4.2.0 --locked
cargo bundle-licenses --format json --output target/THIRDPARTY-audit.json
node tools/check-notices.mjs target/THIRDPARTY-audit.json
```

- El generador incluye un conjunto conservador de dependencias.
- Un baseline generado no es una revisión legal.
- Resolver manualmente textos faltantes o personalizados.
- Revisar fuentes embebidas y recursos nativos.

---

## 4. Gate de release

Antes de publicar:

- [ ] `LICENSE` incluido en cada ZIP.
- [ ] `INSTALL.txt` incluido.
- [ ] Avisos de terceros revisados.
- [ ] TRUCE correctamente atribuido.
- [ ] Slint correctamente atribuido y badge visible en el sitio.
- [ ] Fuentes e imágenes con licencia.
- [ ] Sin código de terceros sin aviso.
- [ ] Sin marcas registradas de terceros en el nombre o la UI.

Un release que falle el gate no se publica.

---

## 5. Marcas y nombre

- “Khodes” es el nombre del proyecto.
- No usar marcas de Fender, Rhodes, Yamaha, etc. para promocionar el producto
  como si fuera oficial.
- Descripciones permitidas: “inspirado en pianos eléctricos de tine”,
  “piano eléctrico sintetizado”.
- Evitar “Rhodes” como nombre de producto o modelo; puede usarse en
  documentación técnica descriptiva con cuidado.
- El código de referencia Buffer Uppercut pertenece a `piducancore`; no se
  reclama autoría sobre él.

---

## 6. Distribución de binarios

- Los binarios se distribuyen vía GitHub Releases.
- Los previews son unsigned.
- `SHA256SUMS` verifica integridad, no identidad.
- La firma de código y la notarización son pasos separados.
- El usuario debe seguir `INSTALL.txt` y usar un proyecto de prueba.

---

## 7. Contribuciones

Al aceptar una contribución:

- el contribuyente acepta licenciarla bajo MIT;
- no se acepta código con licencia incompatible;
- se revisan dependencias nuevas;
- se actualizan avisos si corresponde;
- se documenta cualquier excepción.

---

## 8. Garantía

El software se distribuye “AS IS”, sin garantía. Esto no exime de las
obligaciones de las licencias de terceros ni de las normas de consumo
aplicables.
