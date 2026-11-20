/**
 * Fixture local para verificar el build con RELEASES_SOURCE=github sin red.
 *
 * Uso:
 *   KHODES_MOCK_GITHUB=1 RELEASES_SOURCE=github \
 *   GITHUB_REPOSITORY=superdandi/khodes GITHUB_TOKEN=token-de-prueba \
 *   NODE_OPTIONS="--import $PWD/scripts/mock-github-releases.mjs" \
 *   npm run build
 *
 * Este módulo solo se activa cuando KHODES_MOCK_GITHUB=1. No se usa en
 * producción ni se importa desde el sitio generado.
 */

if (process.env.KHODES_MOCK_GITHUB === '1') {
  const VERSION = '0.1.0-preview.1';
  const REPO = 'superdandi/khodes';
  const ASSETS = [
    `Khodes-${VERSION}-windows-x64-unsigned.zip`,
    `Khodes-${VERSION}-macos-unsigned.zip`,
    `Khodes-${VERSION}-linux-x64-unsigned.zip`,
  ];

  function checksumFor(name) {
    let hash = '';
    for (let index = 0; index < 64; index += 1) {
      const code = name.charCodeAt(index % name.length) + index;
      hash += (code % 16).toString(16);
    }
    return hash;
  }

  const checksums = `${ASSETS.map((name) => `${checksumFor(name)}  ${name}`).join('\n')}\n`;

  function makeRelease(tag, publishedAt, draft) {
    const version = tag.replace(/^v/, '');
    const assets = [
      ...ASSETS.map((name, index) => ({
        name: name.replace(VERSION, version),
        state: 'uploaded',
        size: 2_048_000 + index * 131_072,
        browser_download_url: `https://github.com/${REPO}/releases/download/${tag}/${name.replace(VERSION, version)}`,
      })),
      {
        name: 'SHA256SUMS',
        state: 'uploaded',
        browser_download_url: `https://github.com/${REPO}/releases/download/${tag}/SHA256SUMS`,
        url: `https://api.github.com/repos/${REPO}/releases/assets/${draft ? 100 : 99}`,
      },
    ];

    return {
      tag_name: tag,
      name: draft ? 'Borrador interno' : 'Khodes preview',
      draft,
      prerelease: true,
      published_at: publishedAt,
      created_at: publishedAt,
      html_url: `https://github.com/${REPO}/releases/tag/${tag}`,
      body: [
        '## Cambios de la versión de prueba',
        '',
        '- Editor Slint con cinco grupos de parámetros.',
        '- MIDI sample-accurate y CC7 como ganancia de performance.',
        '- Paquetes sin firma: verifica SHA256SUMS antes de instalar.',
        '',
        '<script>esto-no-debe-renderizarse</script>',
      ].join('\n'),
      assets,
    };
  }

  const releases = [
    makeRelease('v0.2.0-preview.1', '2026-12-01T00:00:00Z', true),
    makeRelease(`v${VERSION}`, '2026-11-20T10:00:00Z', false),
  ];

  const realFetch = globalThis.fetch;
  globalThis.fetch = async function mockedFetch(input, init) {
    const url = String(input);
    if (url === `https://api.github.com/repos/${REPO}/releases?per_page=20`) {
      if (init?.headers?.Authorization && !String(init.headers.Authorization).includes('token-de-prueba')) {
        throw new Error('El fixture recibió un token inesperado');
      }
      return new Response(JSON.stringify(releases), {
        status: 200,
        headers: { 'Content-Type': 'application/json' },
      });
    }
    if (url === `https://api.github.com/repos/${REPO}/releases/assets/99`) {
      return new Response(checksums, {
        status: 200,
        headers: { 'Content-Type': 'text/plain' },
      });
    }
    if (url === `https://api.github.com/repos/${REPO}/releases/assets/100`) {
      throw new Error('El build intentó leer el checksum de un draft');
    }
    return realFetch(input, init);
  };
}
