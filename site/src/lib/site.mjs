/**
 * Configuración del sitio derivada del entorno.
 *
 * Este módulo se ejecuta solo en tiempo de build. No se importa desde código
 * que llegue al navegador.
 */

const LOCAL_SITE_URL = 'http://localhost:4321';
const DEFAULT_BASE_PATH = '/khodes';
const REPO_SLUG_RE = /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/;

export function normalizeSiteUrl(value) {
  const candidate = String(value ?? '').trim();
  if (!candidate) return LOCAL_SITE_URL;

  const url = new URL(candidate);
  if (url.protocol !== 'http:' && url.protocol !== 'https:') {
    throw new Error('SITE_URL debe usar http o https');
  }
  if (url.username || url.password) {
    throw new Error('SITE_URL no debe contener credenciales');
  }
  return url.href.replace(/\/$/, '');
}

export function normalizeBasePath(value) {
  const candidate = String(value ?? '').trim();
  if (!candidate || candidate === '/') return '/';
  if (!candidate.startsWith('/')) {
    throw new Error('SITE_BASE debe empezar por /');
  }
  if (/[?#]/.test(candidate)) {
    throw new Error('SITE_BASE no debe contener query ni fragmento');
  }
  return candidate.replace(/\/+$/, '') || '/';
}

export function normalizeRepoSlug(value) {
  const candidate = String(value ?? '').trim();
  if (!candidate) return '';
  if (!REPO_SLUG_RE.test(candidate)) {
    throw new Error('GITHUB_REPOSITORY debe tener el formato owner/repo');
  }
  return candidate;
}

export const SITE_URL = normalizeSiteUrl(
  process.env.SITE_URL ?? process.env.URL ?? LOCAL_SITE_URL,
);

export const BASE_PATH = normalizeBasePath(process.env.SITE_BASE ?? DEFAULT_BASE_PATH);

export const REPO_SLUG = normalizeRepoSlug(
  process.env.GITHUB_REPOSITORY ?? process.env.GITHUB_REPO ?? '',
);

export const REPO_URL = REPO_SLUG ? `https://github.com/${REPO_SLUG}` : '';

export const IS_PRODUCTION_BUILD = process.env.GITHUB_ACTIONS === 'true';

export function withBase(path = '/') {
  const clean = String(path).startsWith('/') ? String(path) : `/${String(path)}`;
  if (BASE_PATH === '/') return clean;
  return `${BASE_PATH}${clean}`;
}

export function absoluteSiteUrl(path = '/') {
  return new URL(withBase(path), `${SITE_URL}/`).href;
}
