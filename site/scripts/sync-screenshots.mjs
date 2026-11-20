import { copyFile, mkdir, readFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SITE_ROOT = fileURLToPath(new URL('..', import.meta.url));
const REPO_ROOT = path.resolve(SITE_ROOT, '..');
const SOURCE_DIR = path.join(REPO_ROOT, 'screenshots');
const TARGET_DIR = path.join(SITE_ROOT, 'public', 'screenshots');

const FILES = [
  'slint-default.png',
  'slint-active.png',
  'slint-minimum.png',
  'slint-wide.png',
];

const PNG_MAGIC = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

async function main() {
  await mkdir(TARGET_DIR, { recursive: true });

  for (const file of FILES) {
    const source = path.join(SOURCE_DIR, file);
    const header = await readFile(source).then((buffer) => buffer.subarray(0, 8));
    if (!header.equals(PNG_MAGIC)) {
      throw new Error(`La captura ${file} no es un PNG válido`);
    }
    await copyFile(source, path.join(TARGET_DIR, file));
  }

  console.log(`Sincronizadas ${FILES.length} capturas en site/public/screenshots/`);
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
