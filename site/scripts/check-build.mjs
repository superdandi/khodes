import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { isValidAssetName, parseChecksums } from '../src/lib/releases.mjs';
import {
  BASE_PATH,
  IS_PRODUCTION_BUILD,
  REPO_SLUG,
  SITE_URL,
  absoluteSiteUrl,
  normalizeBasePath,
} from '../src/lib/site.mjs';

const SITE_ROOT = fileURLToPath(new URL('..', import.meta.url));
const DIST = path.join(SITE_ROOT, 'dist');
const SOURCE = process.env.RELEASES_SOURCE ?? 'offline';

const EXPECTED_PAGES = [
  'index.html',
  'downloads/index.html',
  'installation/index.html',
  'docs/index.html',
  'changelog/index.html',
  'license/index.html',
  'support/index.html',
];

const SITEMAP_PAGES = [
  '/',
  '/downloads/',
  '/installation/',
  '/docs/',
  '/changelog/',
  '/license/',
  '/support/',
];

const errors = [];

function fail(message) {
  errors.push(message);
}

function decodeEntities(text) {
  return String(text)
    .replace(/&amp;/g, '&')
    .replace(/&lt;/g, '<')
    .replace(/&gt;/g, '>')
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'")
    .replace(/&nbsp;/g, ' ');
}

function parseAttributes(tag) {
  const result = {};
  for (const match of tag.matchAll(/([A-Za-z][A-Za-z0-9-]*)="([^"]*)"/g)) {
    result[match[1]] = match[2];
  }
  return result;
}

async function listFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) files.push(...(await listFiles(full)));
    else files.push(full);
  }
  return files;
}

function localPathFor(href) {
  if (!href || href.startsWith('#') || /^(https?:|mailto:|tel:|data:|javascript:)/i.test(href)) {
    return null;
  }
  const url = new URL(href, 'http://localhost/');
  let pathname = decodeURIComponent(url.pathname);
  if (BASE_PATH !== '/' && pathname.startsWith(`${BASE_PATH}/`)) {
    pathname = pathname.slice(BASE_PATH.length);
  } else if (BASE_PATH !== '/' && pathname === BASE_PATH) {
    pathname = '/';
  }
  if (pathname === '' || pathname === '/') return 'index.html';
  if (pathname.endsWith('/')) return `${pathname.slice(1)}index.html`;
  return pathname.slice(1);
}

async function checkFiles(filesByPath) {
  for (const page of EXPECTED_PAGES) {
    if (!filesByPath.has(page)) fail(`Falta la página generada ${page}`);
  }
  for (const asset of ['favicon.svg', 'og-image.svg', 'robots.txt', 'sitemap.xml']) {
    if (!filesByPath.has(asset)) fail(`Falta el archivo ${asset}`);
  }
  if (!filesByPath.has('screenshots/slint-default.png')) {
    fail('Falta la captura real screenshots/slint-default.png');
  }
  if (!filesByPath.has('screenshots/slint-active.png')) {
    fail('Falta la captura real screenshots/slint-active.png');
  }
}

function checkSecrets(filesByPath, files) {
  const tokenValues = [process.env.GITHUB_TOKEN, process.env.SITE_TOKEN].filter(Boolean);
  const secretPattern = /\bgh[pousr]_[A-Za-z0-9]{20,}\b|github_pat_[A-Za-z0-9_]{20,}|Authorization:\s*Bearer\s/i;

  for (const file of files) {
    const text = decodeEntities(filesByPath.get(path.relative(DIST, file)) ?? '');
    if (tokenValues.some((token) => text.includes(token))) {
      fail(`Se encontró un token en ${path.relative(DIST, file)}`);
    }
    if (secretPattern.test(text)) {
      fail(`Se encontró un patrón de credencial en ${path.relative(DIST, file)}`);
    }
    if (/\bGITHUB_TOKEN\b/.test(text)) {
      fail(`Se renderizó el nombre GITHUB_TOKEN en ${path.relative(DIST, file)}`);
    }
  }
  if (IS_PRODUCTION_BUILD && /localhost|TU-DOMINIO/i.test(SITE_URL)) {
    fail(`SITE_URL no está configurado para producción: ${SITE_URL}`);
  }
  if (/TU-DOMINIO/.test(filesByPath.get('sitemap.xml') ?? '')) {
    fail('sitemap.xml contiene el dominio placeholder');
  }
}

function checkLinks(filesByPath) {
  for (const [name, text] of filesByPath) {
    if (!name.endsWith('.html')) continue;
    for (const match of text.matchAll(/<(?:a|img|link|source)\b[^>]*>/gi)) {
      const attributes = parseAttributes(match[0]);
      const href = attributes.href ?? attributes.src ?? '';
      if (!href) continue;
      const relative = localPathFor(href);
      if (!relative) continue;
      if (/\.(css|js|svg|png|ico|txt|xml)$/i.test(relative)) {
        if (!filesByPath.has(relative)) {
          fail(`${name} referencia el archivo inexistente ${relative}`);
        }
      } else if (!filesByPath.has(relative)) {
        fail(`${name} referencia la página inexistente ${relative}`);
      }
    }
  }
}

function checkAccessibilityAndSeo(filesByPath) {
  for (const page of EXPECTED_PAGES) {
    const html = filesByPath.get(page) ?? '';
    if (!/<html\s+lang="es"/.test(html)) fail(`${page} no declara lang="es"`);
    if (!/<main[\s>]/.test(html)) fail(`${page} no tiene main`);
    if (!/<nav\b[^>]*aria-label="Navegación principal"/.test(html)) fail(`${page} no tiene nav accesible`);
    if (!/rel="canonical"/.test(html)) fail(`${page} no tiene canonical`);
    if (!/property="og:title"/.test(html)) fail(`${page} no tiene og:title`);
    if (!/class="skip-link"/.test(html)) fail(`${page} no tiene enlace de salto`);
  }

  for (const [name, text] of filesByPath) {
    if (!name.endsWith('.html')) continue;
    for (const match of text.matchAll(/<img\b[^>]*>/gi)) {
      if (!/\balt="/.test(match[0])) fail(`${name} tiene una imagen sin alt`);
      if (!/\bwidth="/.test(match[0]) || !/\bheight="/.test(match[0])) {
        fail(`${name} tiene una imagen sin dimensiones explícitas`);
      }
    }
  }

  const sitemap = filesByPath.get('sitemap.xml') ?? '';
  for (const page of SITEMAP_PAGES) {
    const expected = absoluteSiteUrl(page);
    if (!sitemap.includes(expected)) fail(`sitemap.xml no incluye ${expected}`);
  }
  const robots = filesByPath.get('robots.txt') ?? '';
  const sitemapUrl = absoluteSiteUrl('/sitemap.xml');
  if (!robots.includes(`Sitemap: ${sitemapUrl}`)) fail(`robots.txt no anuncia ${sitemapUrl}`);
}

function checkReleases(filesByPath) {
  const html = filesByPath.get('downloads/index.html') ?? '';
  const anchors = [...html.matchAll(/<a\b[^>]*>/gi)].map((match) => parseAttributes(match[0]));
  const downloadAnchors = anchors.filter((attributes) => attributes['data-download-asset']);

  const hasEmptyState = /id="no-release-title"/.test(html);

  if (SOURCE === 'offline') {
    if (downloadAnchors.length > 0) fail('El build offline no debe tener botones de descarga');
    if (!hasEmptyState) fail('El build offline no muestra el estado honesto sin descargas');
  } else if (SOURCE === 'github') {
    if (downloadAnchors.length === 0 && !hasEmptyState) {
      fail('El build de GitHub no tiene descargas ni estado honesto sin release');
    }
    if (downloadAnchors.length > 0 && hasEmptyState) {
      fail('La página mezcla descargas con el estado de sin release');
    }
  }

  const notes = /<div id="release-notes"[^>]*>([\s\S]*?)<\/div>/.exec(html)?.[1] ?? '';
  if (/<(?:script|img|a|iframe|svg|style)\b/i.test(notes)) {
    fail('Las notas de release contienen HTML sin escapar');
  }

  const checksumBlock = /<pre id="sha256sums-text"[^>]*>([\s\S]*?)<\/pre>/.exec(html)?.[1] ?? '';
  const checksums = parseChecksums(decodeEntities(checksumBlock));

  for (const attributes of downloadAnchors) {
    const asset = attributes['data-download-asset'];
    const checksum = attributes['data-checksum'];
    if (!isValidAssetName(asset)) fail(`Nombre de asset inválido en botón: ${asset}`);
    if (!/^[a-fA-F0-9]{64}$/.test(checksum ?? '')) {
      fail(`El botón de ${asset} no tiene un checksum SHA-256 válido`);
    }
    if (!/^https:\/\/github\.com\//.test(attributes.href ?? '')) {
      fail(`El botón de ${asset} no apunta a GitHub Releases`);
    }
    if (REPO_SLUG && !attributes.href.startsWith(`https://github.com/${REPO_SLUG}/releases/download/`)) {
      fail(`El botón de ${asset} no pertenece a ${REPO_SLUG}`);
    }
    if (checksums.size && checksums.get(asset) !== (checksum ?? '').toLowerCase()) {
      fail(`El checksum visible de ${asset} no coincide con SHA256SUMS`);
    }
  }

  if (SOURCE === 'github' && downloadAnchors.length > 0 && checksums.size === 0) {
    fail('No se encontró el bloque de SHA256SUMS en downloads');
  }
  if (SOURCE === 'github' && downloadAnchors.length === 0 && checksumBlock.trim()) {
    fail('El estado sin release no debe mostrar un bloque de SHA256SUMS');
  }
}

async function main() {
  let files;
  try {
    files = await listFiles(DIST);
  } catch {
    fail('No existe site/dist; ejecuta npm run build antes de npm run test:build');
    console.error(errors.join('\n'));
    process.exit(1);
  }

  const filesByPath = new Map(
    await Promise.all(
      files.map(async (file) => [path.relative(DIST, file), await readFile(file, 'utf8')]),
    ),
  );

  // Evita que la verificación del sitemap se rompa si alguien cambia el base.
  if (normalizeBasePath(process.env.SITE_BASE ?? '/khodes') !== BASE_PATH) {
    fail('BASE_PATH inconsistente entre check-build y astro.config.mjs');
  }

  await checkFiles(filesByPath);
  checkSecrets(filesByPath, files);
  checkLinks(filesByPath);
  checkAccessibilityAndSeo(filesByPath);
  checkReleases(filesByPath);

  if (errors.length > 0) {
    console.error(`check-build: ${errors.length} error(es)`);
    for (const error of errors) console.error(`- ${error}`);
    process.exit(1);
  }

  console.log(
    `check-build: OK (${files.length} archivos, base ${BASE_PATH}, releases ${SOURCE})`,
  );
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
