# Sitio de producto — Khodes

**Estado:** especificación del sitio, basada en el sitio de Buffer Uppercut.

El sitio de Khodes es una **web estática** que presenta el plugin, documenta la
instalación y ofrece descargas leídas de GitHub Releases. No hay servidor de
aplicación, no hay API en runtime y no hay React en el cliente.

---

## 1. Stack

| Componente | Elección |
|---|---|
| Framework | Astro 7.x |
| Output | `static` |
| Node | >= 22.23.4 |
| Hosting | GitHub Pages |
| Dominio | `SITE_URL`; local `http://localhost:4321`, en CI `<owner>.github.io` salvo `vars.SITE_URL` |
| Base path | `SITE_BASE`, default `/khodes` |
| Releases | GitHub Releases API en build |
| Tests | `node --test` + chequeo de build |

Referencia observada:

```js
// buffer-uppercut/site/astro.config.mjs
export default defineConfig({
  site: 'https://piducan.dev',
  base: '/buffer-uppercut',
  output: 'static',
  trailingSlash: 'always',
});
```

Para Khodes, los valores se derivan del entorno en `src/lib/site.mjs`:

```js
import { defineConfig } from 'astro/config';
import { BASE_PATH, SITE_URL } from './src/lib/site.mjs';

export default defineConfig({
  site: SITE_URL,
  base: BASE_PATH,
  output: 'static',
  trailingSlash: 'always',
  build: { format: 'directory' },
});
```

`SITE_URL` y `SITE_BASE` deben fijarse con valores reales antes de publicar.
Si se usa GitHub Pages sin dominio propio, `site` será
`https://TU-USUARIO.github.io` y `base` seguirá siendo `/khodes`.

---

## 2. Mapa de páginas

| Ruta | Archivo | Contenido |
|---|---|---|
| `/khodes/` | `src/pages/index.astro` | Home: qué es Khodes, sonido, formatos, CTA de descarga |
| `/khodes/downloads/` | `src/pages/downloads.astro` | Versiones, plataformas, checksums, notas |
| `/khodes/installation/` | `src/pages/installation.astro` | Requisitos, instalación, DAWs, troubleshooting |
| `/khodes/docs/` | `src/pages/docs.astro` | Enlaces a documentación o resumen |
| `/khodes/changelog/` | `src/pages/changelog.astro` | Historial de versiones |
| `/khodes/license/` | `src/pages/license.astro` | MIT, TRUCE, Slint y avisos |
| `/khodes/support/` | `src/pages/support.astro` | Contacto, bugs, FAQ |

Opcional:

- `/khodes/presets/` — presets de fábrica y demos.
- `/khodes/about/` — historia del proyecto y créditos.
- `/khodes/press/` — kit de prensa.

La referencia tiene tres páginas principales: home, downloads e installation.
Khodes puede empezar con esas tres y crecer.

---

## 3. Contenido de la home

Secciones sugeridas:

1. **Hero:** nombre, una frase, imagen del editor, CTA “Descargar”.
2. **Sonido:** tres o cuatro características (tine, campana, key click, bus).
3. **Controles:** captura del panel Voz/Env/Amp/FX.
4. **Formatos:** CLAP y VST3; plataformas.
5. **Estado:** preview/estable, limitaciones.
6. **Enlaces:** documentación, changelog, licencia.
7. **Footer:** créditos, licencia, atribución de Slint si aplica.

No prometer “compatible con todos los DAWs”. Solo puede afirmarse “probado en
REAPER y Ableton Live” cuando la matriz de [TESTING](TESTING.md) esté verde;
hoy no es así. El texto publicado dice únicamente que Khodes ofrece CLAP y VST3
y que la compatibilidad con hosts concretos se documenta tras validarla.

---

## 4. Descargas

### 4.1 Fuente de verdad

GitHub Releases. El sitio **no** almacena binarios ni los reescribe.

- Drafts: nunca aparecen.
- Prereleases: aparecen si están publicadas.
- La release más nueva con paquetes elegibles alimenta los botones.
- Si una plataforma falta en esa release, se muestra “no disponible”; no se
  mezcla con una versión anterior.

### 4.2 Contrato de nombres

```text
Khodes-VERSION-windows-x64-unsigned.zip
Khodes-VERSION-macos-unsigned.zip
Khodes-VERSION-linux-x64-unsigned.zip
SHA256SUMS
```

Solo esos archivos generan botones. Se excluyen:

- candidatos internos;
- paquetes `not-for-distribution`;
- standalone;
- estados de firma desconocidos.

### 4.3 Metadata visible

Por release:

- versión;
- fecha;
- plataformas disponibles;
- arquitectura;
- requisitos de runtime;
- tamaño;
- checksum;
- notas (texto escapado);
- estado de firma.

---

## 5. Sync de releases en build

### 5.1 Modos

| Modo | Variable | Comportamiento |
|---|---|---|
| Offline | `RELEASES_SOURCE=offline` | Lista vacía; estado honesto “sin publicar” |
| GitHub | `RELEASES_SOURCE=github` | Lee releases publicadas en build |

- En desarrollo local, el default es offline.
- En producción, se usa GitHub.
- Un fallo de API **falla el build**; no se vacía el sitio en silencio.
- `GITHUB_TOKEN` se usa solo en build y nunca se renderiza.

### 5.2 Flujo

```text
GitHub Releases API
        │
        ▼
script de build (Node)
        │
        ├── filtra drafts
        ├── valida nombres
        ├── ordena por versión/fecha
        └── genera datos estáticos
        │
        ▼
Astro build
        │
        ▼
site/dist/
        │
        ▼
GitHub Pages
```

### 5.3 Seguridad

- Escapar notas de release como texto plano.
- No interpolar HTML no confiable.
- No exponer tokens.
- No confiar en nombres de archivo sin validar el contrato.
- No seguir redirecciones arbitrarias.
- Limitar el tamaño de la respuesta.

---

## 6. Workflow de GitHub Pages

Disparadores implementados en `.github/workflows/pages.yml`:

- push a `main`;
- pull request (solo build, no deploy);
- release publicada (`types: [published]`);
- ejecución manual (`workflow_dispatch`);
- cron diario a las 04:17 UTC.

Jobs:

1. **build:** configura `SITE_URL`, `SITE_BASE`, `RELEASES_SOURCE=github` y
   `GITHUB_REPOSITORY`; checkout, Node 22.23.4, `npm ci`, `npm test`,
   `npm run build`, `npm run test:build` y upload artifact.
2. **deploy:** solo en release publicada o push a `main`; `actions/deploy-pages`.

Permisos:

```yaml
permissions:
  contents: read
  pages: write
  id-token: write
```

Los PR nunca despliegan. Un fallo de la API de Releases falla el build.

---

## 7. Configuración de GitHub Pages

1. Repositorio → Settings → Pages.
2. Source: **GitHub Actions**.
3. Dominio: el que corresponda (custom domain o `usuario.github.io`).
4. Base path: `/khodes` si se sirve como subdirectorio.
5. Verificar HTTPS.

Si se usa un dominio propio:

- configurar `site` con el dominio;
- añadir `public/CNAME` si Pages lo requiere;
- verificar DNS y certificado.

---

## 8. Accesibilidad

- Contraste AA en texto y controles.
- Navegación por teclado.
- `alt` en imágenes y capturas.
- No depender solo del color para estados.
- Respetar `prefers-reduced-motion`.
- Enlaces con foco visible.
- Textos de descarga claros (“Descargar para macOS”, no “click aquí”).

---

## 9. SEO y performance

- Título y descripción por página.
- Open Graph e imagen social.
- `sitemap.xml` y `robots.txt`.
- HTML estático, sin JS innecesario.
- Imágenes optimizadas y con dimensiones explícitas.
- Cache de assets con hash.
- No bloquear el render con fuentes externas.

---

## 10. Tests del sitio

```sh
cd site
npm ci
npm test
npm run build
npm run test:build
```

Build GitHub simulado, sin red:

```sh
cd site
KHODES_MOCK_GITHUB=1 RELEASES_SOURCE=github \
GITHUB_REPOSITORY=superdandi/khodes GITHUB_TOKEN=token-de-prueba \
NODE_OPTIONS="--import $PWD/scripts/mock-github-releases.mjs" \
npm run build
RELEASES_SOURCE=github GITHUB_REPOSITORY=superdandi/khodes npm run test:build
```

Cubrir:

- parseo de releases;
- filtrado de drafts;
- contrato de nombres;
- escape de notas;
- orden de versiones;
- páginas generadas;
- enlaces internos;
- estado honesto sin release publicada (`id="no-release-title"`), sin botones
  ni checksums inventados;
- ausencia de tokens en el HTML;
- build offline y build GitHub simulado.

---

## 11. Relación con el sitio de referencia

El sitio de Buffer Uppercut demuestra el patrón:

- Astro estático;
- `base` por subdirectorio;
- releases leídas en build;
- páginas de home, downloads e installation;
- GitHub Pages con Actions;
- drafts excluidos;
- checksums visibles.

Khodes adopta el patrón, no el contenido ni el diseño. La identidad visual de
Khodes viene del HUD de Keys (ámbar, verde, magenta, cyan), no de la paleta crema
y carbón de Buffer Uppercut.

---

## 12. Checklist de lanzamiento del sitio

- [x] `astro.config.mjs` derivado de `SITE_URL`/`SITE_BASE`.
- [x] Home, downloads, installation, docs, changelog, license y support
  implementadas.
- [x] Sync de releases probado en modo offline y GitHub simulado.
- [x] Drafts excluidos.
- [x] Nombres de archivo validados.
- [x] Checksums visibles.
- [x] Notas escapadas.
- [x] Accesibilidad y SEO revisados por `test:build`.
- [ ] `SITE_URL` y `SITE_BASE` con los valores reales de producción.
- [ ] Workflow de Pages verde en GitHub.
- [ ] HTTPS y dominio correctos.
- [ ] Enlaces de descarga verificados desde una ventana limpia.
