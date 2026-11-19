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
| Node.js | >= 22.12.0 |
| npm | el incluido con Node |
| Git | reciente |
| `gh` (opcional) | para inspeccionar releases |

No se necesita Rust para trabajar en el sitio.

---

## 2. Estructura prevista

```text
site/
├── README.md                 Este documento
├── astro.config.mjs          site, base, output, trailingSlash
├── package.json              scripts y dependencias
├── package-lock.json         lockfile reproducible
├── public/
│   ├── favicon.svg
│   ├── robots.txt
│   └── CNAME                 solo si hay dominio propio
├── scripts/
│   ├── releases.mjs          cliente de GitHub Releases en build
│   └── check-build.mjs       validación del HTML generado
├── src/
│   ├── pages/
│   │   ├── index.astro       Home
│   │   ├── downloads.astro   Descargas por plataforma
│   │   ├── installation.astro Instalación y requisitos
│   │   ├── changelog.astro   Historial de versiones
│   │   ├── license.astro     MIT, TRUCE, Slint y avisos
│   │   └── support.astro     Contacto, bugs, FAQ
│   ├── layouts/
│   │   └── BaseLayout.astro  Head, navegación, footer
│   ├── components/
│   │   ├── DownloadButton.astro
│   │   ├── ReleaseNotes.astro
│   │   ├── PlatformGrid.astro
│   │   └── SlintBadge.astro
│   ├── lib/
│   │   ├── releases.ts       Tipos y normalización de releases
│   │   └── format.ts         Formato de fechas, tamaños y checksums
│   ├── styles/
│   │   └── global.css
│   └── assets/
│       ├── khodes-editor.png
│       └── og-image.png
└── tests/
    ├── releases.test.mjs     Parseo, filtrado y orden
    └── build.test.mjs        Contrato del HTML generado
```

La estructura puede ajustarse, pero las páginas y el flujo de releases son
contrato de producto.

---

## 3. Configuración de Astro

```js
// astro.config.mjs
import { defineConfig } from 'astro/config';

export default defineConfig({
  site: 'https://TU-DOMINIO',
  base: '/khodes',
  output: 'static',
  trailingSlash: 'always',
});
```

- `site`: dominio real, sin barra final.
- `base`: `/khodes` si se sirve como subdirectorio de GitHub Pages.
- `output: 'static'`: sin SSR ni adaptadores.
- `trailingSlash: 'always'`: URLs canónicas terminadas en `/`.

Con GitHub Pages sin dominio propio:

```text
site = https://TU-USUARIO.github.io
base = /khodes
```

Con dominio propio:

```text
site = https://khodes.example
base = /
```

En ese caso, ajustar `base` y añadir `public/CNAME` si Pages lo requiere.

---

## 4. Scripts

```json
{
  "scripts": {
    "dev": "astro dev",
    "build": "astro build",
    "preview": "astro preview",
    "test": "node --test tests/*.test.mjs",
    "test:build": "node scripts/check-build.mjs"
  }
}
```

| Comando | Uso |
|---|---|
| `npm run dev` | Servidor local con recarga |
| `npm run build` | Genera `site/dist/` |
| `npm run preview` | Sirve `dist/` localmente |
| `npm test` | Tests unitarios del sitio |
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

1. Se dispara en push a `main` con cambios en `site/**`, `screenshots/**` o el
   propio workflow.
2. Se dispara en pull request con esos cambios, solo para construir.
3. Se dispara en release publicada, editada o eliminada.
4. Se puede ejecutar manualmente.
5. Corre a diario para capturar releases creadas con `GITHUB_TOKEN`.

Jobs:

- **build:** `npm ci`, `npm test`, `npm run build`, `npm run test:build`,
  `actions/upload-pages-artifact`.
- **deploy:** solo en `main` o release; `actions/deploy-pages`.

Permisos:

```yaml
permissions:
  contents: read
```

El job de deploy añade `pages: write` e `id-token: write`. Los PR nunca
despliegan.

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
- [Astro](https://astro.build/)
- [GitHub Pages](https://docs.github.com/pages)
- [Buffer Uppercut — sitio](https://piducan.dev/buffer-uppercut/)
