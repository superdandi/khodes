#!/usr/bin/env node
/**
 * Verifica el conjunto de assets de una release antes de crear el draft.
 *
 * Uso: node tools/verify-release-assets.mjs <directorio> <vX.Y.Z[-preview.N]>
 *
 * Comprueba que:
 * - existen exactamente los tres ZIP de la versión;
 * - SHA256SUMS contiene solo esos tres archivos;
 * - los hashes coincide con los ficheros reales;
 * - no hay assets internos, standalone, debug ni firmados;
 * - están los avisos legales y la ficha de versión.
 */

import { createHash } from 'node:crypto';
import { readFile, readdir } from 'node:fs/promises';
import path from 'node:path';

const [directoryArg, versionArg] = process.argv.slice(2);

if (!directoryArg || !versionArg) {
  console.error('Uso: node tools/verify-release-assets.mjs <directorio> <vX.Y.Z[-preview.N]>');
  process.exit(2);
}

const version = String(versionArg).replace(/^v/, '');
if (!/^\d+\.\d+\.\d+(-preview\.\d+)?$/.test(version)) {
  console.error(`Versión inválida: ${versionArg}`);
  process.exit(2);
}

const directory = path.resolve(directoryArg);
const expected = [
  `Khodes-${version}-windows-x64-unsigned.zip`,
  `Khodes-${version}-macos-unsigned.zip`,
  `Khodes-${version}-linux-x64-unsigned.zip`,
];
const requiredDocs = ['LICENSE', 'INSTALL.txt', 'THIRDPARTY.json', 'VERSION.md'];
const forbidden = /(^|[-_])(internal|not[-_]for[-_]distribution|standalone|signed|debug|test)([-_]|\.|$)/i;
const errors = [];

function fail(message) {
  errors.push(message);
}

function parseChecksums(text) {
  const result = new Map();
  for (const line of String(text).split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    const match = /^([a-fA-F0-9]{64})\s+\*?(\S.*)$/.exec(trimmed);
    if (!match) {
      fail(`Línea de SHA256SUMS no válida: ${trimmed}`);
      continue;
    }
    const name = match[2];
    if (result.has(name)) fail(`Entrada duplicada en SHA256SUMS: ${name}`);
    result.set(name, match[1].toLowerCase());
  }
  return result;
}

async function sha256Of(file) {
  const buffer = await readFile(file);
  return createHash('sha256').update(buffer).digest('hex');
}

async function main() {
  const files = await readdir(directory);

  for (const name of expected) {
    if (!files.includes(name)) fail(`Falta el asset ${name}`);
    if (forbidden.test(name)) fail(`El asset ${name} contiene un fragmento prohibido`);
  }
  for (const name of files) {
    if (name.endsWith('.zip') && !expected.includes(name)) {
      fail(`Asset inesperado: ${name}`);
    }
  }
  for (const doc of requiredDocs) {
    if (!files.includes(doc)) fail(`Falta el documento ${doc}`);
  }

  const checksums = parseChecksums(await readFile(path.join(directory, 'SHA256SUMS'), 'utf8'));

  for (const name of expected) {
    if (!checksums.has(name)) {
      fail(`SHA256SUMS no incluye ${name}`);
      continue;
    }
    const actual = await sha256Of(path.join(directory, name));
    if (actual !== checksums.get(name)) {
      fail(`Hash incorrecto para ${name}: esperado ${checksums.get(name)}, real ${actual}`);
    }
  }
  for (const name of checksums.keys()) {
    if (!expected.includes(name)) fail(`SHA256SUMS incluye un archivo inesperado: ${name}`);
  }

  if (errors.length > 0) {
    console.error(`verify-release-assets: ${errors.length} error(es)`);
    for (const error of errors) console.error(`- ${error}`);
    process.exit(1);
  }

  console.log(
    `verify-release-assets: OK (${expected.length} ZIP, ${expected.length} hashes, versión ${version})`,
  );
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
