# Sitio web de Khodes

Sitio estático de producto para **Khodes**, construido con Astro. Presenta el
plugin, documenta la instalación y publica las descargas leídas de **GitHub
Releases** en tiempo de build. No hay backend, ni API en runtime, ni base de
datos: el resultado es HTML, CSS y JS estático servido por GitHub Pages.

> El contrato funcional vive en [`docs/SITE.md`](../docs/SITE.md). Este README
> es la guía operativa del proyecto `site/`.

---

## 1. Requisitos

| Herramienta | Versión |
|---|---|
| Node.js | >= 22.23.4 (exigido por Astro 7.4.13) |
| npm | el incluido con Node |
| Git | reciente |
| `gh` (opcional) | para inspeccionar releases |

No se necesita Rust para trabajar en el sitio.

---

## 2. Estructura implementada

```text
site/
├── README.md                     Este documento
├── astro.config.mjs              site, base, output, trailingSlash desde env
├── package.json                  scripts y dependencias
├── package-lock.json             lockfile reproducible
├── public/
│   ├── favicon.svg
│   ├── og-image.svg
│   └── screenshots/              generado; ignorado por git
├── scripts/
│   ├── sync-screenshots.mjs      copia capturas reales desde screenshots/
│   ├── check-build.mjs           validación del HTML generado
│   └── mock-github-releases.mjs  fixture local, sin red
├── src/
│   ├── pages/
│   │   ├── index.astro           Home
│   │   ├── downloads.astro       Descargas, notas y checksums
│   │   ├── installation.astro    Instalación y troubleshooting
│   │   ├── docs.astro            Índice de documentación
│   │   ├── changelog.astro       Última release publicada
│   │   ├── license.astro         MIT, TRUCE, Slint y avisos
│   │   ├── support.astro         Contacto, bugs, FAQ, privacidad
│   │   ├── robots.txt.ts         endpoint prerenderizado
│   │   └── sitemap.xml.ts        endpoint prerenderizado
│   ├── layouts/
│   │   └── BaseLayout.astro      Head, navegación, canonical y footer
│   ├── components/
│   │   ├── DownloadButton.astro
│   │   ├── ReleaseNotes.astro
│   │   ├── PlatformGrid.astro
│   │   └── SlintBadge.astro
│   ├── lib/
│   │   ├── site.mjs              Variables de entorno y helpers de URL
│   │   ├── releases.mjs          Cliente y normalización de releases
│   │   └── format.mjs            Fechas, tamaños y checksums
│   └── styles/
│       └── global.css
└── tests/
    └── releases.test.mjs         10 tests del cliente de releases
```

Las páginas y el flujo de releases son contrato de producto. Las capturas
autoritativas viven en `screenshots/` en la raíz del repositorio; el directorio
`public/screenshots/` se regenera en `predev` y `prebuild`.

---

## 3. Configuración de Astro

`astro.config.mjs` no contiene dominios fijos. Los valores se derivan del
entorno en `src/lib/site.mjs`:

| Variable | Default local | Uso |
|---|---|---|
| `SITE_URL` | `http://localhost:4321` | Dominio canónico, sin barra final |
| `SITE_BASE` | `/khodes` | Subdirectorio de GitHub Pages; `/` con dominio propio |
| `RELEASES_SOURCE` | `offline` | `offline` o `github` |
| `GITHUB_REPOSITORY` | vacío | Repositorio `owner/repo` del que se leen releases |
| `GITHUB_REPO` | vacío | Alias de `GITHUB_REPOSITORY` |
| `GITHUB_TOKEN` | vacío | Solo en tiempo de build; nunca se renderiza |

```js
import { defineConfig } from 'astro/config';
import { BASE_PATH, SITE_URL } from './src/lib/site.mjs';

export default defineConfig({
  site: SITE_URL,
  base: BASE_PATH,
  output: 'static',
  trailingSlash: 'always',
  build: { format: 'directory' },
  server: { host: true },
});
```

Valores que debe fijar una persona antes de publicar:

```text
SITE_URL=https://khodes.example    # o https://TU-USUARIO.github.io
SITE_BASE=/                        # o /khodes en subdirectorio
GITHUB_REPOSITORY=owner/khodes
```

El workflow `pages.yml` usa `vars.SITE_URL` y `vars.SITE_BASE` si están
definidos; si no, calcula `https://<owner>.github.io` y `/khodes`. El token
nunca debe aparecer en `dist/`: `test:build` lo comprueba.

Con dominio propio, crear `public/CNAME` con el dominio exacto.

---

## 4. Scripts

```json
{
  "scripts": {
    "predev": "node scripts/sync-screenshots.mjs",
    "dev": "astro dev",
    "prebuild": "node scripts/sync-screenshots.mjs",
    "build": "astro build",
    "preview": "astro preview",
    "test": "node --test tests/*.test.mjs",
    "test:build": "node scripts/check-build.mjs"
  }
}
```

| Comando | Uso |
|---|---|
| `npm run dev` | Sincroniza capturas y abre el servidor local |
| `npm run build` | Sincroniza capturas y genera `site/dist/` |
| `npm run preview` | Sirve `dist/` localmente |
| `npm test` | 10 tests del cliente de releases |
| `npm run test:build` | Verifica el HTML generado |

Flujo recomendado antes de un PR:

```sh
cd site
npm ci
npm test
npm run build
npm run test:build
```

---

## 5. Releases en build

### 5.1 Modos

| Modo | Variable | Comportamiento |
|---|---|---|
| Offline | `RELEASES_SOURCE=offline` | Lista vacía; el sitio muestra “sin publicar” |
| GitHub | `RELEASES_SOURCE=github` | Lee releases publicadas desde la API |

- En desarrollo local el default es `offline`, para no depender de la red ni de
  un token.
- En producción se usa `github`.
- Un fallo de API **falla el build**. No se publica un sitio vacío en silencio.

### 5.2 Build local con releases reales

```sh
cd site
RELEASES_SOURCE=github \
GITHUB_REPOSITORY=owner/khodes \
GITHUB_TOKEN="$(gh auth token)" \
npm run build
```

`GITHUB_TOKEN` se usa solo durante el build. Nunca se escribe en el HTML ni en
los assets.

### 5.3 Filtrado

El cliente de releases debe:

1. Descartar drafts (`draft === true`).
2. Aceptar prereleases publicadas (`prerelease === true`).
3. Ordenar por fecha de publicación y, en empate, por versión semver.
4. Tomar la release más nueva que tenga al menos un paquete elegible.
5. Validar cada asset contra el contrato de nombres.
6. Excluir candidatos internos, `not-for-distribution`, standalone y estados de
   firma desconocidos.

Contrato de nombres:

```text
Khodes-VERSION-windows-x64-unsigned.zip
Khodes-VERSION-macos-unsigned.zip
Khodes-VERSION-linux-x64-unsigned.zip
SHA256SUMS
```

Solo esos archivos generan botones. Si una plataforma falta en la release
elegida, se muestra “no disponible”; **no** se toma de una release anterior.

### 5.4 Seguridad

- Escapar las notas de release como texto plano.
- No interpolar HTML no confiable.
- No exponer el token en logs ni en el HTML.
- Validar nombres de archivo antes de construir URLs.
- Limitar el tamaño de la respuesta de la API.
- No seguir redirecciones arbitrarias.

---

## 6. Páginas

### 6.1 Home (`/khodes/`)

- Hero con nombre, frase y captura del editor.
- Tres o cuatro características de sonido.
- Formatos: CLAP y VST3.
- Plataformas disponibles según la última release.
- CTA a descargas.
- Enlaces a documentación, changelog y licencia.
- Atribución de Slint si aplica.

### 6.2 Downloads (`/khodes/downloads/`)

- Versión, fecha y estado (preview/estable).
- Botones por plataforma.
- Arquitectura y requisitos.
- Tamaño y checksum.
- Notas de release escapadas.
- Enlace a `SHA256SUMS`.
- Aviso de binarios sin firma cuando corresponda.

### 6.3 Installation (`/khodes/installation/`)

- Requisitos de sistema.
- Rutas de instalación por plataforma.
- Cómo verificar que el DAW detecta el plugin.
- Notas sobre macOS Gatekeeper y Windows SmartScreen para binarios sin firma.
- Preguntas frecuentes.

### 6.4 Licencia (`/khodes/license/`)

- Texto MIT del código original.
- Aviso de TRUCE.
- Aviso de Slint y badge de atribución.
- Enlaces a avisos de terceros.

### 6.5 Changelog y Support

- Changelog generado o enlazado desde las notas de release.
- Support con canal de bugs y política de privacidad básica.

---

## 7. Identidad visual

Khodes usa la estética HUD de Keys, no la de Buffer Uppercut.

| Uso | Color |
|---|---|
| Fondo | `#0a0a14` |
| Panel | `#12121f` |
| Borde | `#2a2a4a` |
| Texto | `#c8c8e0` |
| Voz | `#ffdd44` |
| Env | `#00dd88` |
| Amp | `#ff00ff` |
| FX | `#00ffff` |
| Global | `#7aa2ff` |

Reglas:

- Contraste AA.
- Sin animaciones que compitan con el contenido.
- Respetar `prefers-reduced-motion`.
- Imágenes con dimensiones explícitas y `alt`.
- No usar marcas de terceros (Fender, Rhodes, Yamaha) como identidad.

---

## 8. Tests

### 8.1 `tests/releases.test.mjs`

Debe cubrir:

- parseo de una respuesta de API;
- descarte de drafts;
- inclusión de prereleases publicadas;
- orden por fecha y versión;
- contrato de nombres;
- exclusión de assets internos;
- plataforma ausente → “no disponible”;
- escape de notas;
- respuesta vacía → estado offline honesto;
- error de API → excepción, no lista vacía.

### 8.2 `scripts/check-build.mjs`

Verifica sobre `dist/`:

- que existen las páginas esperadas;
- que no hay `GITHUB_TOKEN` ni secretos en el HTML;
- que los enlaces internos resuelven;
- que los botones apuntan a URLs de GitHub Releases del repo correcto;
- que los checksums visibles corresponden al asset;
- que no se renderiza HTML crudo de las notas;
- que `robots.txt` y `sitemap.xml` existen si se prometen.

---

## 9. Despliegue

El sitio se publica con GitHub Actions en GitHub Pages. El workflow
`.github/workflows/pages.yml`:

1. Push a `main`.
2. Pull request, solo para construir y verificar.
3. Release publicada (`types: [published]`).
4. Ejecución manual (`workflow_dispatch`).
5. Cron diario a las 04:17 UTC para capturar releases publicadas.

Jobs:

- **build:** configura `SITE_URL`, `SITE_BASE`, `RELEASES_SOURCE=github` y
  `GITHUB_REPOSITORY`; ejecuta `npm ci`, `npm test`, `npm run build`,
  `npm run test:build` y `actions/upload-pages-artifact`.
- **deploy:** solo en `main` o release publicada; `actions/deploy-pages`.

Permisos a nivel de workflow:

```yaml
permissions:
  contents: read
  pages: write
  id-token: write
```

Los PR nunca despliegan. Un fallo de la API de Releases falla el build: no se
publica un sitio con enlaces vacíos.

### 9.1 Configuración inicial en GitHub

1. Settings → Pages → Source: **GitHub Actions**.
2. Configurar dominio o usar `usuario.github.io`.
3. Verificar HTTPS.
4. Ejecutar el workflow manualmente una vez.
5. Comprobar que la home carga en la URL con `base`.

### 9.2 Releases

El workflow de release del repositorio crea un **draft prerelease**. El sitio
solo verá la release cuando una persona la publique. Ver
[`../docs/RELEASES.md`](../docs/RELEASES.md).

---

## 10. Solución de problemas

| Síntoma | Causa probable | Solución |
|---|---|---|
| La home da 404 | `base` no coincide con la URL | Ajustar `base` y reconstruir |
| Los assets dan 404 | Rutas absolutas sin `base` | Usar `import.meta.env.BASE_URL` |
| No aparecen descargas | Release en draft o sin assets | Publicar la release; revisar nombres |
| Aparece una versión vieja | Caché o build sin release event | Ejecutar `pages.yml` a mano |
| El build falla por API | Token ausente o rate limit | Revisar `GITHUB_TOKEN` y logs |
| Se muestran notas con HTML | Escape desactivado | Escapar como texto plano |
| El sitio se ve vacío | `RELEASES_SOURCE` mal configurado | Usar `github` en producción |
| `npm run build` termina con `EXIT:132` | Binding nativo de Astro exige AVX2/BMI2 | Aplicar el fallback WASI local descrito abajo |
| `Class extends value undefined` tras fijar `NAPI_RS_NATIVE_LIBRARY_PATH` | La variable secuestra el binding de `rolldown` | Usar el parche local `KHODES_FORCE_WASI_BINDING`, no la variable global |

### 10.1 Fallback WASI en CPU x86-64-v2

Esta CPU no soporta AVX2 ni BMI2, y
`@astrojs/compiler-binding-linux-x64-gnu@0.5.3` produce `SIGILL`. El workaround
es **local y no se commitea**:

1. Extraer `@astrojs/compiler-binding-wasm32-wasi@0.5.3` en
   `node_modules/@astrojs/compiler-binding-wasm32-wasi/`.
2. Instalar sin guardar `@napi-rs/wasm-runtime@1.2.7`, `@emnapi/core@1.11.3` y
   `@emnapi/runtime@1.11.3`.
3. Parchear `node_modules/@astrojs/compiler-binding/index.js` para que la rama
   `KHODES_FORCE_WASI_BINDING` cargue
   `../compiler-binding-wasm32-wasi/astro.wasi.cjs`.
4. Ejecutar `KHODES_FORCE_WASI_BINDING=1 npm run build`.

No usar `NAPI_RS_FORCE_WASI=1` (el loader llama al binario nativo antes del
fallback) ni `NAPI_RS_NATIVE_LIBRARY_PATH` (afecta a todos los módulos napi-rs).
`npm ci` elimina el fallback: hay que repetir estos pasos tras reinstalar.
En CI, con CPU moderna, no hace falta.

---

## 11. Checklist de PR del sitio

- [ ] `npm ci` sin cambios inesperados en el lockfile.
- [ ] `npm test` verde.
- [ ] `npm run build` verde en modo offline y GitHub.
- [ ] `npm run test:build` verde.
- [ ] Sin secretos en el HTML generado.
- [ ] Enlaces internos y de descarga verificados.
- [ ] Notas de release escapadas.
- [ ] Capturas actualizadas si cambió la UI.
- [ ] Accesibilidad y contraste revisados.
- [ ] `docs/SITE.md` actualizado si cambió el contrato.

---

## 12. Referencias

- [`../docs/SITE.md`](../docs/SITE.md)
- [`../docs/RELEASES.md`](../docs/RELEASES.md)
- [`../docs/LICENSING.md`](../docs/LICENSING.md)
- [`../docs/adr/0005-site-and-release-versioning.md`](../docs/adr/0005-site-and-release-versioning.md)
- [`../tools/verify-release-assets.mjs`](../tools/verify-release-assets.mjs)
- [`../.github/workflows/pages.yml`](../.github/workflows/pages.yml)
- [`../.github/workflows/release.yml`](../.github/workflows/release.yml)
- [Astro](https://astro.build/)
- [GitHub Pages](https://docs.github.com/pages)
- [Buffer Uppercut — sitio](https://piducan.dev/buffer-uppercut/)
