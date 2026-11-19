# 0005 — Sitio estático y versionado de releases

- **Estado:** Aceptado
- **Fecha:** 2026-11-19
- **Decisores:** equipo Khodes
- **Relacionado:** [RELEASES](../RELEASES.md), [SITE](../SITE.md), [0004](0004-versioned-preset-codec.md)

---

## Contexto

Khodes necesita una presencia web con tres funciones:

1. Explicar qué es el producto y cómo suena.
2. Ofrecer descargas correctas por plataforma.
3. Publicar notas de versión e instrucciones de instalación.

Las opciones van desde una web dinámica con backend hasta un sitio puramente
estático. Un backend añade superficie de ataque, coste, disponibilidad y
complejidad de despliegue; para un plugin con releases en GitHub, es
innecesario.

El proyecto de referencia resuelve esto con Astro estático, `base` por
subdirectorio, lectura de GitHub Releases en tiempo de build, GitHub Pages y un
workflow que nunca publica solo. También separa tres versiones que no deben
confundirse: producto, códec de preset y corpus de sonido.

El riesgo principal es **publicar algo incorrecto**: un draft visible, un enlace
a una plataforma que no existe en esa release, una versión mezclada con otra, o
notas de release interpretadas como HTML.

---

## Decisión

### Sitio

- **Astro** con `output: 'static'`.
- `site` configurado con el dominio real; `base: '/khodes'`.
- `trailingSlash: 'always'`.
- Páginas iniciales: home, downloads, installation.
- Páginas posteriores: changelog, license, support.
- GitHub Pages como hosting; despliegue con GitHub Actions.
- Sin backend, sin API en runtime, sin base de datos.

### Descargas

- **GitHub Releases es la fuente de verdad.** El sitio no almacena binarios.
- Las releases se leen en **tiempo de build**, no en el navegador.
- Los drafts nunca aparecen.
- La release más nueva con paquetes elegibles alimenta los botones; no se
  mezclan versiones entre plataformas.
- Solo los archivos que cumplen el contrato de nombres generan botones.
- Las notas se escapan como texto plano.
- `GITHUB_TOKEN` se usa solo en build y nunca se renderiza.
- Un fallo de la API **falla el build**; no se vacía el sitio en silencio.

Contrato de nombres:

```text
Khodes-VERSION-windows-x64-unsigned.zip
Khodes-VERSION-macos-unsigned.zip
Khodes-VERSION-linux-x64-unsigned.zip
SHA256SUMS
```

### Versionado

```text
vMAJOR.MINOR.PATCH-preview.N
```

- `MAJOR`: cambios incompatibles de contrato.
- `MINOR`: funcionalidad nueva compatible.
- `PATCH`: correcciones compatibles.
- `-preview.N`: preestreno, sin promesa de estabilidad, sin firma.

Reglas:

1. La versión del producto es independiente de `PRESET_VERSION` y de
   `DSP_CONTRACT_VERSION`.
2. El tag es `v` + versión de Cargo.
3. El CI crea un **draft prerelease**; nunca publica.
4. La publicación es manual, tras revisar assets y checksums.
5. `APPROVED.txt` contiene la versión exacta aprobada.
6. Un tag publicado no se mueve ni se sobrescribe.
7. Un rollback es un release nuevo, no una edición del anterior.
8. La promoción a stable es un proceso separado, con firma si aplica.

### Workflow de Pages

Disparadores:

- push a `main` con cambios en `site/**`, `screenshots/**` o el workflow;
- pull request con esos cambios (solo build);
- release publicada, editada o eliminada;
- ejecución manual;
- cron diario para capturar releases creadas con `GITHUB_TOKEN`.

Permisos: `contents: read` para build; `pages: write` e `id-token: write` solo
para deploy.

---

## Consecuencias

### Positivas

- Superficie mínima: HTML estático, sin servidor ni secretos en runtime.
- Descargas siempre coherentes con GitHub Releases.
- Drafts y versiones mezcladas quedan fuera por construcción.
- Coste de hosting nulo o mínimo.
- El build falla de forma visible ante un error de API, en lugar de degradar.
- La separación de versiones evita migraciones y confusión.

### Negativas

- Cada cambio de release requiere un rebuild del sitio.
- El sitio no puede mostrar datos en vivo (descargas, estrellas) sin JS extra.
- Depende de la disponibilidad y del formato de la API de GitHub en build.
- La publicación manual exige una persona disponible.
- Los previews sin firma pueden requerir pasos manuales en macOS y Windows.

### Neutrales

- Astro y Node añaden una toolchain separada de Rust.
- El sitio puede crecer a más páginas sin cambiar la arquitectura.
- El dominio y el base path pueden cambiar con ajustes de configuración.

---

## Alternativas consideradas

| Alternativa | Motivo de descarte |
|---|---|
| Web dinámica con backend | Complejidad, coste y superficie innecesarios |
| Descargas desde el propio dominio | Duplica binarios y rompe la fuente de verdad |
| Leer releases en el navegador | Dependencia de API en runtime y riesgo de tokens |
| Publicación automática desde CI | Elimina la revisión humana y facilita releases rotas |
| Mezclar plataformas de distintas releases | Botones incoherentes y confusión de versión |
| Un solo binario universal | Tamaño y compatibilidad; mejor por plataforma |

---

## Implementación

- [ ] Crear `site/` con Astro y `base: '/khodes'`.
- [ ] Implementar home, downloads e installation.
- [ ] Implementar el cliente de releases en build con modos `github` y `offline`.
- [ ] Validar el contrato de nombres y excluir drafts.
- [ ] Escapar notas de release.
- [ ] Añadir tests de parseo, filtrado, orden y escape.
- [ ] Añadir `scripts/check-build.mjs` para verificar el HTML generado.
- [ ] Crear `.github/workflows/pages.yml`.
- [ ] Crear `.github/workflows/release.yml` con draft y `APPROVED.txt`.
- [ ] Verificar el sitio con una release real antes del primer anuncio.

Checklist de publicación:

1. Tag y draft creados por CI.
2. Assets descargados y `SHA256SUMS` verificado.
3. `LICENSE`, `INSTALL.txt` y avisos presentes.
4. Draft publicado manualmente.
5. Workflow de Pages ejecutado.
6. Versión y enlaces verificados en el navegador.

---

## Referencias

- [`../RELEASES.md`](../RELEASES.md)
- [`../SITE.md`](../SITE.md)
- [`../../site/README.md`](../../site/README.md)
- [Buffer Uppercut — sitio](https://piducan.dev/buffer-uppercut/)
- [Buffer Uppercut — pages.yml](https://github.com/piducancore/buffer-uppercut/blob/main/.github/workflows/pages.yml)
