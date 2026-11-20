/**
 * Cliente de GitHub Releases para el build estático de Khodes.
 *
 * No se ejecuta en el navegador: Astro llama a estas funciones durante
 * `astro build`. El token nunca se escribe en el HTML ni en los logs.
 */

export const PLATFORMS = Object.freeze([
  {
    id: 'windows-x64',
    label: 'Windows x64',
    requirement: 'Windows 10 o posterior, x64.',
    location: 'Carpeta del sistema del host o carpeta de plugins de la DAW.',
  },
  {
    id: 'macos',
    label: 'macOS',
    requirement: 'macOS con Apple silicon; sin notarizar en previews.',
    location: '~/Library/Audio/Plug-Ins/CLAP y ~/Library/Audio/Plug-Ins/VST3.',
  },
  {
    id: 'linux-x64',
    label: 'Linux x64',
    requirement: 'Linux x64 con glibc; entorno de escritorio con soporte Slint.',
    location: '~/.clap y ~/.vst3.',
  },
]);

const VERSION_SOURCE = String.raw`\d+\.\d+\.\d+(?:-preview\.\d+)?`;
const ASSET_RE = new RegExp(
  `^Khodes-(${VERSION_SOURCE})-(windows-x64|macos|linux-x64)-unsigned\\.zip$`,
);
const CHECKSUM_RE = /^([a-fA-F0-9]{64})\s+\*?(\S.*)$/;
const FORBIDDEN_FRAGMENT_RE =
  /(^|[-_])(internal|not[-_]for[-_]distribution|standalone|signed|debug|test)([-_]|$)/i;
const API_REPO_RE = /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/;
const API_ASSET_RE = /^https:\/\/api\.github\.com\/repos\/[^/]+\/[^/]+\/releases\/assets\/\d+$/;

const MAX_API_BYTES = 1024 * 1024;
const MAX_CHECKSUM_BYTES = 64 * 1024;
const REQUEST_TIMEOUT_MS = 15_000;

/** Escapa texto no confiable para insertarlo como contenido textual. */
export function escapeHtml(value) {
  const map = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#39;',
  };
  return String(value ?? '')
    .replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/g, '')
    .replace(/[&<>"']/g, (character) => map[character]);
}

/** Parsea un nombre de asset según el contrato de nombres. */
export function parseAssetName(name) {
  if (typeof name !== 'string' || FORBIDDEN_FRAGMENT_RE.test(name)) return null;
  const match = ASSET_RE.exec(name);
  if (!match) return null;
  return { version: match[1], platform: match[2] };
}

export function isValidAssetName(name) {
  return parseAssetName(name) !== null;
}

/** Convierte `v0.1.0-preview.1` o `0.1.0-preview.1` en una versión comparable. */
export function parseVersion(value) {
  const raw = String(value ?? '').replace(/^v/, '');
  const match = new RegExp(`^(${VERSION_SOURCE})$`).exec(raw);
  if (!match) return null;

  const [core, preview] = raw.split('-preview.');
  const parts = core.split('.').map(Number);
  if (parts.length !== 3 || parts.some((part) => !Number.isInteger(part))) return null;
  if (preview !== undefined && !Number.isInteger(Number(preview))) return null;

  return { parts, preview: preview === undefined ? null : Number(preview) };
}

/** Compara dos versiones: devuelve > 0 si `a` es posterior a `b`. */
export function compareVersions(a, b) {
  const parsedA = typeof a === 'string' ? parseVersion(a) : a;
  const parsedB = typeof b === 'string' ? parseVersion(b) : b;
  if (!parsedA || !parsedB) return 0;

  for (let index = 0; index < 3; index += 1) {
    if (parsedA.parts[index] !== parsedB.parts[index]) {
      return parsedA.parts[index] - parsedB.parts[index];
    }
  }

  if (parsedA.preview === null && parsedB.preview === null) return 0;
  if (parsedA.preview === null) return 1;
  if (parsedB.preview === null) return -1;
  return parsedA.preview - parsedB.preview;
}

function publishedTime(release) {
  const time = Date.parse(release?.published_at ?? release?.created_at ?? '');
  return Number.isFinite(time) ? time : 0;
}

/** Ordena por fecha de publicación y, en empate, por versión semver. */
export function sortReleases(releases) {
  return [...(Array.isArray(releases) ? releases : [])]
    .filter((release) => release && typeof release === 'object' && release.draft !== true)
    .sort((a, b) => {
      const byDate = publishedTime(b) - publishedTime(a);
      if (byDate !== 0) return byDate;

      const byVersion = compareVersions(b.tag_name, a.tag_name);
      if (byVersion !== 0) return byVersion;

      return String(b.tag_name ?? '').localeCompare(String(a.tag_name ?? ''));
    });
}

function assetIsUploaded(asset) {
  return !asset?.state || asset.state === 'uploaded';
}

export function listEligibleAssets(release) {
  return (Array.isArray(release?.assets) ? release.assets : []).filter(
    (asset) => assetIsUploaded(asset) && parseAssetName(asset.name),
  );
}

export function findChecksumAsset(release) {
  return (Array.isArray(release?.assets) ? release.assets : []).find(
    (asset) => assetIsUploaded(asset) && asset?.name === 'SHA256SUMS',
  );
}

/** Devuelve la release publicada más reciente con assets elegibles. */
export function findCandidateRelease(releases) {
  return sortReleases(releases).find(
    (release) => listEligibleAssets(release).length > 0 && findChecksumAsset(release),
  );
}

export function parseChecksums(text) {
  const result = new Map();
  for (const rawLine of String(text ?? '').split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line || line.startsWith('#')) continue;
    const match = CHECKSUM_RE.exec(line);
    if (match) result.set(match[2], match[1].toLowerCase());
  }
  return result;
}

function isHttpsUrl(value) {
  try {
    const url = new URL(String(value));
    return url.protocol === 'https:' && !url.username && !url.password;
  } catch {
    return false;
  }
}

function repoPrefix(repoSlug, tag) {
  if (!repoSlug || !tag) return '';
  return `https://github.com/${repoSlug}/releases/download/${tag}/`;
}

function isValidDownloadUrl(value, repoSlug, tag, name) {
  if (!isHttpsUrl(value)) return false;
  const prefix = repoPrefix(repoSlug, tag);
  if (!prefix) return /^https:\/\/github\.com\/[^/]+\/[^/]+\/releases\/download\//.test(String(value));
  return String(value) === `${prefix}${name}`;
}

function isValidReleaseUrl(value, repoSlug) {
  if (!isHttpsUrl(value)) return false;
  if (!repoSlug) return /^https:\/\/github\.com\/[^/]+\/[^/]+\/releases\//.test(String(value));
  return String(value).startsWith(`https://github.com/${repoSlug}/releases/`);
}

/**
 * Normaliza una release. Si falta un checksum o una URL no cumple el contrato,
 * la plataforma se marca como no disponible.
 */
export function normalizeRelease(release, checksumText, options = {}) {
  if (!release || typeof release !== 'object' || release.draft === true) return null;

  const repoSlug = options.repoSlug ?? '';
  const tag = String(release.tag_name ?? '');
  const tagsVersion = parseVersion(tag);
  const eligible = listEligibleAssets(release);
  const checksums = parseChecksums(checksumText);
  const checksumAsset = findChecksumAsset(release);
  if (!eligible.length || !checksums.size) return null;

  const assetVersions = new Set(eligible.map((asset) => parseAssetName(asset.name).version));
  if (assetVersions.size !== 1) return null;
  const assetVersion = [...assetVersions][0];
  if (tagsVersion && compareVersions(tagsVersion, parseVersion(assetVersion)) !== 0) return null;

  const platforms = {};
  for (const platform of PLATFORMS) {
    platforms[platform.id] = null;
  }

  for (const asset of eligible) {
    const parsed = parseAssetName(asset.name);
    const checksum = checksums.get(asset.name);
    if (
      !checksum ||
      !isValidDownloadUrl(asset.browser_download_url, repoSlug, tag, asset.name)
    ) {
      continue;
    }
    platforms[parsed.platform] = {
      name: asset.name,
      url: asset.browser_download_url,
      size: Number.isFinite(Number(asset.size)) ? Number(asset.size) : 0,
      checksum,
    };
  }

  if (!Object.values(platforms).some(Boolean)) return null;

  return {
    version: assetVersion,
    tag,
    name: typeof release.name === 'string' && release.name.trim() ? release.name : tag,
    publishedAt: release.published_at ?? release.created_at ?? null,
    prerelease: release.prerelease === true,
    notes: typeof release.body === 'string' ? release.body : '',
    platforms,
    checksumsText: String(checksumText ?? ''),
    checksumsUrl:
      checksumAsset && isValidDownloadUrl(checksumAsset.browser_download_url, repoSlug, tag, 'SHA256SUMS')
        ? checksumAsset.browser_download_url
        : null,
    releaseUrl: isValidReleaseUrl(release.html_url, repoSlug) ? release.html_url : null,
  };
}

/** Selecciona y normaliza la release más nueva con paquetes utilizables. */
export function selectRelease(releases, checksumText, options = {}) {
  const candidate = findCandidateRelease(releases);
  if (!candidate) return null;
  return normalizeRelease(candidate, checksumText, options);
}

async function readBodyWithLimit(response, maxBytes) {
  const declared = Number(response.headers?.get?.('content-length'));
  if (Number.isFinite(declared) && declared > maxBytes) {
    throw new Error('La respuesta de GitHub excede el tamaño permitido');
  }
  if (!response.body) return '';

  const reader = response.body.getReader();
  const chunks = [];
  let total = 0;
  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      total += value.byteLength;
      if (total > maxBytes) {
        await reader.cancel();
        throw new Error('La respuesta de GitHub excede el tamaño permitido');
      }
      chunks.push(value);
    }
  } finally {
    reader.releaseLock();
  }
  return new TextDecoder().decode(Buffer.concat(chunks));
}

/** Petición limitada a GitHub. Nunca incluye el token en errores ni en URLs. */
export async function fetchTextFromGithub(url, options = {}) {
  const token = String(options.token ?? '').trim();
  const headers = {
    Accept: options.accept ?? 'application/vnd.github+json',
    'User-Agent': 'khodes-site-build',
    'X-GitHub-Api-Version': '2022-11-28',
  };
  if (token) headers.Authorization = `Bearer ${token}`;

  let response;
  try {
    response = await (options.fetchImpl ?? fetch)(url, {
      headers,
      redirect: 'error',
      signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
    });
  } catch {
    throw new Error('No se pudo conectar con GitHub Releases');
  }

  if (!response.ok) {
    throw new Error(`GitHub Releases devolvió HTTP ${response.status}`);
  }
  return readBodyWithLimit(response, options.maxBytes ?? MAX_API_BYTES);
}

function isValidRepoSlug(repoSlug) {
  return API_REPO_RE.test(String(repoSlug ?? ''));
}

function validateAssetApiUrl(url, repoSlug) {
  if (!API_ASSET_RE.test(String(url))) return false;
  const expected = `https://api.github.com/repos/${repoSlug}/releases/assets/`;
  return String(url).startsWith(expected);
}

/**
 * Carga los datos de releases para el build.
 *
 * - `offline`: estado honesto “sin publicar”.
 * - `github`: falla de forma visible si la API falla o el contrato no se cumple.
 */
export async function loadReleaseData(options = {}) {
  const source = options.source ?? process.env.RELEASES_SOURCE ?? 'offline';
  const repoSlug = String(options.repoSlug ?? process.env.GITHUB_REPOSITORY ?? '').trim();
  const repoUrl = isValidRepoSlug(repoSlug) ? `https://github.com/${repoSlug}` : '';

  if (source === 'offline') {
    return {
      source,
      repoSlug,
      repoUrl,
      release: null,
      message: 'No hay releases publicadas en este build.',
    };
  }

  if (source !== 'github') {
    throw new Error(`RELEASES_SOURCE desconocido: ${source}`);
  }
  if (!isValidRepoSlug(repoSlug)) {
    throw new Error('GITHUB_REPOSITORY es obligatorio cuando RELEASES_SOURCE=github');
  }

  const apiUrl = `https://api.github.com/repos/${repoSlug}/releases?per_page=20`;
  const body = await fetchTextFromGithub(apiUrl, { ...options, maxBytes: MAX_API_BYTES });

  let releases;
  try {
    releases = JSON.parse(body);
  } catch {
    throw new Error('La respuesta de GitHub Releases no es JSON válido');
  }
  if (!Array.isArray(releases)) {
    throw new Error('La respuesta de GitHub Releases no es una lista');
  }

  const candidate = findCandidateRelease(releases);
  if (!candidate) {
    return {
      source,
      repoSlug,
      repoUrl,
      release: null,
      message: 'No hay releases publicadas con assets elegibles.',
    };
  }

  const checksumAsset = findChecksumAsset(candidate);
  const checksumApiUrl = String(checksumAsset?.url ?? '');
  if (!validateAssetApiUrl(checksumApiUrl, repoSlug)) {
    throw new Error('La URL del asset SHA256SUMS no pertenece al repositorio');
  }

  const checksumText = await fetchTextFromGithub(checksumApiUrl, {
    ...options,
    accept: 'application/octet-stream',
    maxBytes: MAX_CHECKSUM_BYTES,
  });

  const release = normalizeRelease(candidate, checksumText, { repoSlug });
  if (!release) {
    throw new Error('La release elegida no cumple el contrato de nombres y checksums');
  }

  return { source, repoSlug, repoUrl, release, message: '' };
}

let cachedData = null;

/** Memoiza la carga durante un mismo build de Astro. */
export async function getReleaseData(options = {}) {
  const source = options.source ?? process.env.RELEASES_SOURCE ?? 'offline';
  const repoSlug = String(options.repoSlug ?? process.env.GITHUB_REPOSITORY ?? '').trim();
  const token = String(options.token ?? process.env.GITHUB_TOKEN ?? '');
  const key = JSON.stringify({ source, repoSlug, tokenPresent: Boolean(token) });

  if (!cachedData || cachedData.key !== key) {
    cachedData = { key, promise: loadReleaseData(options) };
  }
  return cachedData.promise;
}
