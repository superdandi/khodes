import assert from 'node:assert/strict';
import test from 'node:test';

import {
  escapeHtml,
  findCandidateRelease,
  isValidAssetName,
  loadReleaseData,
  normalizeRelease,
  parseAssetName,
  parseChecksums,
  selectRelease,
  sortReleases,
} from '../src/lib/releases.mjs';

const VERSION = '0.1.0-preview.1';
const REPO = 'superdandi/khodes';
const WINDOWS = `Khodes-${VERSION}-windows-x64-unsigned.zip`;
const MACOS = `Khodes-${VERSION}-macos-unsigned.zip`;
const LINUX = `Khodes-${VERSION}-linux-x64-unsigned.zip`;

function checksumFor(name) {
  let hash = '';
  for (let index = 0; index < 64; index += 1) {
    const code = name.charCodeAt(index % name.length) + index;
    hash += (code % 16).toString(16);
  }
  return hash;
}

function checksumText(names) {
  return `${names.map((name) => `${checksumFor(name)}  ${name}`).join('\n')}\n`;
}

function asset(name, overrides = {}) {
  return {
    name,
    state: 'uploaded',
    size: 1024,
    browser_download_url: `https://github.com/${REPO}/releases/download/v${VERSION}/${name}`,
    ...overrides,
  };
}

function release(overrides = {}) {
  return {
    tag_name: `v${VERSION}`,
    name: 'Khodes preview',
    draft: false,
    prerelease: true,
    published_at: '2026-11-20T10:00:00Z',
    html_url: `https://github.com/${REPO}/releases/tag/v${VERSION}`,
    body: 'Notas de prueba',
    assets: [
      asset(WINDOWS),
      asset(MACOS),
      asset(LINUX),
      { name: 'SHA256SUMS', state: 'uploaded', browser_download_url: `https://github.com/${REPO}/releases/download/v${VERSION}/SHA256SUMS`, url: `https://api.github.com/repos/${REPO}/releases/assets/99` },
    ],
    ...overrides,
  };
}

function githubFetch(releases, text = checksumText([WINDOWS, MACOS, LINUX])) {
  return async function fakeFetch(url) {
    if (String(url).includes('/releases?')) {
      return new Response(JSON.stringify(releases), { status: 200 });
    }
    if (String(url).includes('/releases/assets/')) {
      return new Response(text, { status: 200 });
    }
    throw new Error(`URL inesperada: ${url}`);
  };
}

test('parsea una respuesta de API y expone solo assets elegibles', async () => {
  const data = await loadReleaseData({
    source: 'github',
    repoSlug: REPO,
    fetchImpl: githubFetch([release()]),
  });

  assert.equal(data.release.version, VERSION);
  assert.equal(data.release.prerelease, true);
  assert.equal(data.release.platforms['windows-x64'].name, WINDOWS);
  assert.equal(data.release.platforms.macos.checksum, checksumFor(MACOS));
  assert.equal(data.release.checksumsUrl?.endsWith('/SHA256SUMS'), true);
});

test('descarta drafts aunque sean más recientes', () => {
  const draft = release({
    draft: true,
    published_at: '2026-12-01T00:00:00Z',
    tag_name: 'v0.2.0-preview.1',
  });
  const published = release();
  assert.equal(findCandidateRelease([draft, published]).tag_name, `v${VERSION}`);
  assert.equal(sortReleases([draft, published]).length, 1);
});

test('incluye prereleases publicadas', () => {
  const selected = selectRelease(
    [release({ prerelease: true })],
    checksumText([WINDOWS, MACOS, LINUX]),
    { repoSlug: REPO },
  );
  assert.equal(selected.prerelease, true);
});

test('ordena por fecha y, en empate, por versión', () => {
  const older = release({ tag_name: 'v0.1.0-preview.1' });
  const newer = release({ tag_name: 'v0.1.0-preview.2' });
  newer.assets = release({
    tag_name: 'v0.1.0-preview.2',
  }).assets.map((item) =>
    item.name === 'SHA256SUMS'
      ? item
      : { ...item, name: item.name.replace(VERSION, '0.1.0-preview.2'), browser_download_url: item.browser_download_url.replace(VERSION, '0.1.0-preview.2') },
  );

  const ordered = sortReleases([older, newer]);
  assert.equal(ordered[0].tag_name, 'v0.1.0-preview.2');
});

test('valida el contrato de nombres', () => {
  assert.deepEqual(parseAssetName(WINDOWS), {
    version: VERSION,
    platform: 'windows-x64',
  });
  assert.equal(isValidAssetName(`Khodes-${VERSION}-windows-x64.zip`), false);
  assert.equal(isValidAssetName('khodes-0.1.0-preview.1-macos-unsigned.zip'), false);
  assert.equal(isValidAssetName('Khodes-0.1.0-preview.1-macos-signed.zip'), false);
});

test('excluye candidatos internos y sin distribución', () => {
  const internal = `Khodes-${VERSION}-windows-x64-unsigned-internal.zip`;
  const notForDistribution = `Khodes-${VERSION}-macos-not-for-distribution-unsigned.zip`;
  const standalone = `Khodes-standalone-${VERSION}-linux-x64-unsigned.zip`;
  assert.equal(isValidAssetName(internal), false);
  assert.equal(isValidAssetName(notForDistribution), false);
  assert.equal(isValidAssetName(standalone), false);
});

test('una plataforma ausente queda como no disponible', () => {
  const onlyWindows = release({ assets: [asset(WINDOWS)] });
  const normalized = normalizeRelease(
    onlyWindows,
    checksumText([WINDOWS]),
    { repoSlug: REPO },
  );
  assert.equal(normalized.platforms['windows-x64'].name, WINDOWS);
  assert.equal(normalized.platforms.macos, null);
  assert.equal(normalized.platforms['linux-x64'], null);
});

test('escapa notas como texto plano', () => {
  const escaped = escapeHtml('<script>alert("x")</script> & \'y\'');
  assert.equal(
    escaped,
    '&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt; &amp; &#39;y&#39;',
  );
  assert.equal(parseChecksums(`${'a'.repeat(64)}  ${WINDOWS}\n`).get(WINDOWS), 'a'.repeat(64));
});

test('modo offline devuelve un estado honesto', async () => {
  const data = await loadReleaseData({ source: 'offline' });
  assert.equal(data.release, null);
  assert.match(data.message, /No hay releases publicadas/);
});

test('un error de API falla el build y no filtra el token', async () => {
  const failingFetch = async () => new Response('error', { status: 500 });
  await assert.rejects(
    loadReleaseData({
      source: 'github',
      repoSlug: REPO,
      token: 'ghp_super_secreto_no_renderizar',
      fetchImpl: failingFetch,
    }),
    /HTTP 500/,
  );

  const token = 'ghp_error_token';
  await assert.rejects(
    loadReleaseData({
      source: 'github',
      repoSlug: REPO,
      token,
      fetchImpl: async () => {
        throw new Error('boom');
      },
    }),
    (error) => {
      assert.match(error.message, /No se pudo conectar con GitHub Releases/);
      assert.equal(error.message.includes(token), false);
      return true;
    },
  );
});
